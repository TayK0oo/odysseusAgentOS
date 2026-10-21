//! Budget à quatre axes (paliers T0–T3, PLAN-MAITRE.md §6 P1-3) : steps, tokens,
//! coût, temps. Chaque appel est consommé **atomiquement** et tout, ou rien :
//! si un seul axe dérape, l'appel est refusé et **rien n'est débité**.
//!
//! Frontière avec la boucle agent (fin-A2) : ce budget POLITIQUE plafonne les
//! COÛTS par appel (steps consommés par les appels outils, tokens, coût, temps)
//! et les PERMISSIONS (modules tiers.rs) ; la BOUCLE plafonne le nombre de
//! TOURS de son côté (`AgentLoop`, port omega-ports). Deux compteurs distincts
//! : la boucle appelle `Budget::consume_call` une fois par tour/apel outil ;
//! elle reste seule responsable de son propre nombre de tours.
//!
//! Garantie (testée en property, `tests/proptests.rs`) : aucune séquence
//! d'appels acceptés ne peut dépasser le plafond — jamais N+1 exécutions
//! pour un budget de N. Convention du port Python `BudgetEnforcer` : un
//! plafond de 0 signifie « illimité » (le refus strict n'est pas implémenté
//! pour l'illimité, suivre la convention existante).

use std::fmt;
use std::sync::Mutex;

/// Un axe de budget avec dépassement possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetAxis {
    Steps,
    Tokens,
    Cost,
    Time,
}

/// Ce que l'appelant demande de consommer pour UN appel.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Demand {
    pub steps: u64,
    pub tokens: u64,
    pub cost_usd: f64,
    pub time_ms: u64,
}

/// Cumul consommé à un instant donné.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Usage {
    pub steps: u64,
    pub tokens: u64,
    pub cost_usd: f64,
    pub time_ms: u64,
}

/// Plafonds. `0` = illimité (convention du port `ProjectBudget`).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    pub max_steps: u64,
    pub max_tokens: u64,
    pub max_cost_usd: f64,
    pub max_time_ms: u64,
}

impl Default for Limits {
    fn default() -> Self {
        // Valeurs prudentes alignées sur `ProjectBudget::default()` (port Python).
        Self {
            max_steps: 50,
            max_tokens: 200_000,
            max_cost_usd: 5.0,
            max_time_ms: 60_000,
        }
    }
}

/// Refus structuré d'un appel — jamais de panic sur entrée utilisateur.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetExhausted {
    /// Axe en dépassement. `None` si le refus vient d'une cause non
    /// classifiable (mutex empoisonné par une panique antérieure : fail-closed).
    pub axis: Option<BudgetAxis>,
    pub requested: Demand,
    pub used: Usage,
    pub limit: Limits,
}

impl fmt::Display for BudgetExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.axis {
            None => write!(f, "budget refuse (cause interne, fail-closed)"),
            Some(BudgetAxis::Steps) => write!(
                f,
                "steps budget exceeded: requested {} + used {} > max {}",
                self.requested.steps, self.used.steps, self.limit.max_steps
            ),
            Some(BudgetAxis::Tokens) => write!(
                f,
                "token budget exceeded: requested {} + used {} > max {}",
                self.requested.tokens, self.used.tokens, self.limit.max_tokens
            ),
            Some(BudgetAxis::Cost) => write!(
                f,
                "coût budget exceeded: ${:.4} + ${:.4} > ${:.4}",
                self.requested.cost_usd, self.used.cost_usd, self.limit.max_cost_usd
            ),
            Some(BudgetAxis::Time) => write!(
                f,
                "time budget exceeded: {} ms + {} ms > {} ms",
                self.requested.time_ms, self.used.time_ms, self.limit.max_time_ms
            ),
        }
    }
}

impl std::error::Error for BudgetExhausted {}

/// Budget monotone évaluable par appel : [`Budget::consume_call`] consomme
/// atomiquement (tout-ou-rien) et refuse si un axe dérape. Refuser un appel
/// ne débite rien — l'usage demeure monotone croissant et borné.
pub struct Budget {
    limit: Limits,
    usage: Mutex<Usage>,
}

impl Budget {
    pub fn new(limit: Limits) -> Self {
        Self {
            limit,
            usage: Mutex::new(Usage::default()),
        }
    }

    /// Cumul consommé.
    pub fn usage(&self) -> Usage {
        *self.usage.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Plafonds en vigueur.
    pub fn limits(&self) -> Limits {
        self.limit
    }

    /// Consomme `d` de façon atomique : elle est débitée **entièrement ou pas**.
    /// Un dépassement sur un seul axe refuse l'appel sans rien débiter.
    pub fn consume_call(&self, d: Demand) -> Result<Usage, BudgetExhausted> {
        // Fail-closed sur mutex empoisonné : une panique antérieure pendant une
        // consommation rend l'état suspect ; on refuse tout plutôt que de risquer
        // un comptage faux dans le sens large.
        let used = match self.usage.lock() {
            Ok(guard) => *guard,
            Err(_) => {
                return Err(BudgetExhausted {
                    axis: None,
                    requested: d,
                    used: Usage::default(),
                    limit: self.limit,
                })
            }
        };
        // Tentatif sur les quatre axes : tout-ou-rien.
        let proposed = Usage {
            steps: used.steps + d.steps,
            tokens: used.tokens + d.tokens,
            cost_usd: used.cost_usd + d.cost_usd,
            time_ms: used.time_ms + d.time_ms,
        };
        if self.limit.max_steps > 0 && proposed.steps > self.limit.max_steps {
            return Err(exhausted(BudgetAxis::Steps, d, used, self.limit));
        }
        if self.limit.max_tokens > 0 && proposed.tokens > self.limit.max_tokens {
            return Err(exhausted(BudgetAxis::Tokens, d, used, self.limit));
        }
        if self.limit.max_cost_usd > 0.0 && proposed.cost_usd > self.limit.max_cost_usd {
            return Err(exhausted(BudgetAxis::Cost, d, used, self.limit));
        }
        if self.limit.max_time_ms > 0 && proposed.time_ms > self.limit.max_time_ms {
            return Err(exhausted(BudgetAxis::Time, d, used, self.limit));
        }
        *self.usage.lock().unwrap_or_else(|e| e.into_inner()) = proposed;
        Ok(proposed)
    }
}

fn exhausted(axis: BudgetAxis, requested: Demand, used: Usage, limit: Limits) -> BudgetExhausted {
    BudgetExhausted {
        axis: Some(axis),
        requested,
        used,
        limit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits {
            max_steps: 3,
            max_tokens: 100,
            max_cost_usd: 1.0,
            max_time_ms: 10_000,
        }
    }

    fn demand(steps: u64, tokens: u64, cost: f64, ms: u64) -> Demand {
        Demand {
            steps,
            tokens,
            cost_usd: cost,
            time_ms: ms,
        }
    }

    #[test]
    fn accepts_until_exactly_the_limit() {
        let b = Budget::new(limits());
        assert!(b.consume_call(demand(1, 40, 0.4, 4_000)).is_ok()); // 1/3
        assert!(b.consume_call(demand(1, 40, 0.4, 4_000)).is_ok()); // 2/3
        assert!(b.consume_call(demand(1, 20, 0.2, 2_000)).is_ok()); // 3/3 pile
        assert_eq!(b.usage().steps, 3);
    }

    #[test]
    fn refuses_and_debits_nothing_when_one_axis_overflows() {
        let b = Budget::new(limits());
        // steps passe (1/3), tokens passe (40/100), mais le coût dérape.
        let e = b.consume_call(demand(1, 40, 1.5, 4_000)).unwrap_err();
        assert_eq!(e.axis, Some(BudgetAxis::Cost));
        // Rien n'a été débité (tout-ou-rien).
        assert_eq!(b.usage(), Usage::default());
    }

    #[test]
    fn refusal_debits_nothing_and_retry_fits() {
        let b = Budget::new(limits());
        // Remplit steps exactement (3/3) — l'ancienne version de ce test
        // (gel 78de85c1) supposait un refus après un seul step sur un plafond
        // de 3 : scénario faux l'implémentation étant correcte.
        assert!(b.consume_call(demand(1, 34, 0.0, 1_000)).is_ok());
        assert!(b.consume_call(demand(1, 66, 0.5, 4_000)).is_ok());
        assert!(b.consume_call(demand(1, 0, 0.5, 5_000)).is_ok());
        let e = b.consume_call(demand(1, 0, 0.0, 0)).unwrap_err();
        assert_eq!(e.axis, Some(BudgetAxis::Steps));
        // Le refus n'a rien débité : un appel qui tient passe toujours.
        assert!(b.consume_call(demand(0, 0, 0.0, 0)).is_ok());
        assert_eq!(b.usage().steps, 3);
    }

    #[test]
    fn zero_limit_means_unlimited_inherited_convention() {
        let b = Budget::new(Limits {
            max_steps: 0,
            max_tokens: 0,
            max_cost_usd: 0.0,
            max_time_ms: 0,
        });
        for _ in 0..1000 {
            assert!(b.consume_call(demand(1, 1000, 0.5, 500)).is_ok());
        }
        assert_eq!(b.usage().steps, 1000);
    }

    #[test]
    fn refusal_message_is_structured_not_panic() {
        let b = Budget::new(limits());
        let e = b.consume_call(demand(0, 500, 0.0, 0)).unwrap_err();
        assert!(e.to_string().contains("token budget exceeded"));
        assert_eq!(e.used.tokens, 0);
        assert_eq!(e.limit.max_tokens, 100);
    }
}
