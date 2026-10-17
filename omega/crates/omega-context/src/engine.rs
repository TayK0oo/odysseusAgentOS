//! Moteur de contexte **par defaut** (`name = "default"`, `owns_compaction = false`).

use std::sync::Mutex;

use async_trait::async_trait;
use omega_ports::ContextEngine;
use omega_types::{Message, OmegaError, OmegaResult, Role};

use crate::budget::ContextBudget;
use crate::token::TokenEstimator;

/// Rapport d'assemblage : journalise ce qui a ete coupe (payload de test).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssembleReport {
    pub kept: usize,
    pub dropped: usize,
    pub estimated_tokens: usize,
    /// Vrai si le resultat depasse le budget (systeme seul plus grand que le plafond).
    ///
    /// Le prefixe systeme est toujours preserve (primaute du systeme, cf. W6 §4.6) ;
    /// ce drapeau rend le depassement **explicite** au lieu de le laisser silencieux.
    pub over_budget: bool,
    /// Apercus des messages non-systeme coupes (contenu tronque), dans l'ordre.
    pub dropped_previews: Vec<String>,
}

/// Nombre maximum de caracteres d'un apercu de message coupe.
const PREVIEW_MAX_CHARS: usize = 40;

fn preview(text: &str) -> String {
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(PREVIEW_MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

/// Moteur par defaut : assemble systeme + non-systeme sous budget, en tronquant les
/// plus anciens messages **non-systeme**. Le prefixe systeme est toujours preserve.
pub struct DefaultContextEngine {
    budget: ContextBudget,
    estimator: TokenEstimator,
    last_report: Mutex<AssembleReport>,
}

impl DefaultContextEngine {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            budget: ContextBudget::new(max_tokens),
            estimator: TokenEstimator::new(),
            last_report: Mutex::new(AssembleReport::default()),
        }
    }

    /// Plafond du budget de contexte du moteur.
    pub fn max_tokens(&self) -> usize {
        self.budget.max_tokens()
    }

    /// Dernier rapport d'assemblage (coupes journalisees).
    pub fn last_report(&self) -> AssembleReport {
        self.last_report.lock().expect("rapport empoisonne").clone()
    }

    /// Assemble en mode synchrone (coeur deterministe, testable sans runtime).
    ///
    /// Le prefixe systeme est toujours preserve. Si le systeme seul depasse deja le
    /// budget, le depassement est journalise via `AssembleReport.over_budget` (le
    /// resultat n'est jamais coupe au-dela du systeme, qui est intouchable).
    ///
    /// Retourne le resultat **et** son rapport ensemble : l'appelant async utilise le
    /// rapport retourne (pas une relecture du `Mutex` partage), ce qui rend la decision
    /// `Ok`/`BudgetExceeded` insensible aux appels concurrents (REVIEW-W6 N1).
    pub fn assemble_sync(&self, messages: &[Message]) -> (Vec<Message>, AssembleReport) {
        let total = self.estimator.estimate_messages(messages);
        if total <= self.budget.max_tokens() {
            let report = AssembleReport {
                kept: messages.len(),
                dropped: 0,
                estimated_tokens: total,
                over_budget: false,
                dropped_previews: Vec::new(),
            };
            *self.last_report.lock().expect("rapport empoisonne") = report.clone();
            return (messages.to_vec(), report);
        }

        let system: Vec<Message> = messages
            .iter()
            .take_while(|m| m.role == Role::System)
            .cloned()
            .collect();
        let non_system = &messages[system.len()..];

        // Suffixe maximal des messages les plus recents (on tronque les plus anciens).
        let mut kept: Vec<Message> = Vec::new();
        let mut kept_tokens = self.estimator.estimate_messages(&system);
        for msg in non_system.iter().rev() {
            let t = self.estimator.estimate(&msg.content);
            if kept_tokens + t <= self.budget.max_tokens() {
                kept_tokens += t;
                kept.push(msg.clone());
            } else {
                break;
            }
        }
        kept.reverse();
        let dropped = non_system.len() - kept.len();

        let dropped_previews: Vec<String> = non_system[..dropped]
            .iter()
            .map(|m| preview(&m.content))
            .collect();

        let mut result = system;
        result.extend(kept);

        let estimated_tokens = self.estimator.estimate_messages(&result);
        let report = AssembleReport {
            kept: result.len(),
            dropped,
            estimated_tokens,
            over_budget: estimated_tokens > self.budget.max_tokens(),
            dropped_previews,
        };
        *self.last_report.lock().expect("rapport empoisonne") = report.clone();
        (result, report)
    }
}

#[async_trait]
impl ContextEngine for DefaultContextEngine {
    fn name(&self) -> &str {
        "default"
    }

    fn owns_compaction(&self) -> bool {
        false
    }

    fn max_tokens(&self) -> usize {
        self.budget.max_tokens()
    }

    async fn assemble(&self, messages: Vec<Message>) -> OmegaResult<Vec<Message>> {
        let (result, report) = self.assemble_sync(&messages);
        if report.over_budget {
            // Le systeme seul depasse le budget : signal type, jamais de resultat
            // hors contrat silencieux (REVIEW-W6 M1).
            return Err(OmegaError::BudgetExceeded {
                max: self.max_tokens(),
                used: report.estimated_tokens,
                additional: 0,
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::Role;
    use std::sync::Arc;

    fn sys(content: &str) -> Message {
        Message::new(Role::System, content)
    }

    fn user(content: &str) -> Message {
        Message::new(Role::User, content)
    }

    #[test]
    fn under_budget_is_passthrough() {
        let e = DefaultContextEngine::new(1000);
        let msgs = vec![sys("S"), user("hello")];
        let (out, _) = e.assemble_sync(&msgs);
        assert_eq!(out, msgs);
        assert_eq!(e.last_report().dropped, 0);
    }

    #[test]
    fn assemble_respects_budget_and_drops_oldest_non_system() {
        // system 2 tokens ; u1..u3 2 tokens chacun -> total 8, budget 6.
        let e = DefaultContextEngine::new(6);
        let msgs = vec![
            sys("SSSSSSSS"),
            user("11111111"),
            user("22222222"),
            user("33333333"),
        ];
        let (out, _) = e.assemble_sync(&msgs);

        // systeme + les 2 plus recents uniquement.
        assert_eq!(
            out,
            vec![sys("SSSSSSSS"), user("22222222"), user("33333333")]
        );
        let report = e.last_report();
        assert_eq!(report.dropped, 1);
        assert_eq!(report.kept, 3);
        assert_eq!(report.dropped_previews, vec!["11111111".to_string()]);
        assert!(report.estimated_tokens <= 6);
    }

    #[test]
    fn assemble_preserves_system_even_when_budget_tight() {
        // system 2 tokens ; budget 3 : aucun message non-systeme ne rentre.
        let e = DefaultContextEngine::new(3);
        let msgs = vec![sys("SSSSSSSS"), user("11111111"), user("22222222")];
        let (out, _) = e.assemble_sync(&msgs);

        assert_eq!(out, vec![sys("SSSSSSSS")]);
        let report = e.last_report();
        assert_eq!(report.dropped, 2);
        assert!(!report.over_budget);
    }

    #[test]
    fn assemble_system_alone_over_budget_is_signaled() {
        // Non-regression REVIEW-W6 M1 : systeme seul (10 tokens) > budget (5).
        // Le systeme est preserve (primaute), mais le depassement est explicite.
        let e = DefaultContextEngine::new(5);
        let msgs = vec![sys(&"S".repeat(40))]; // 40 caracteres -> 10 tokens
        let (out, _) = e.assemble_sync(&msgs);

        assert_eq!(out, msgs);
        let report = e.last_report();
        assert!(report.over_budget);
        assert_eq!(report.estimated_tokens, 10);
        assert_eq!(report.dropped, 0);
    }

    #[tokio::test]
    async fn assemble_returns_typed_error_when_system_alone_over_budget() {
        // Le port async expose le depassement en `BudgetExceeded` (jamais de `Ok`
        // hors budget).
        let e = DefaultContextEngine::new(5);
        let msgs = vec![sys(&"S".repeat(40))];
        let err = e.assemble(msgs).await.unwrap_err();
        assert!(matches!(
            err,
            OmegaError::BudgetExceeded {
                max: 5,
                used: 10,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn concurrent_assembles_use_own_report_not_shared() {
        // Non-regression REVIEW-W6 N1 : deux `assemble` concurrents sur le meme moteur
        // prennent chacun la decision coherente avec LEUR entree (pas de faux
        // negatif/positif par relecture du rapport partage).
        use std::sync::Arc;
        let engine = Arc::new(DefaultContextEngine::new(5));
        let over: Vec<Message> = vec![sys(&"S".repeat(40))]; // 10 tokens > 5
        let under: Vec<Message> = vec![user("hi")];
        for _ in 0..50 {
            let (e1, e2) = (engine.clone(), engine.clone());
            let (o1, o2) = (over.clone(), under.clone());
            let (r1, r2) = tokio::join!(e1.assemble(o1), e2.assemble(o2));
            assert!(matches!(r1, Err(OmegaError::BudgetExceeded { .. })));
            assert!(r2.is_ok());
        }
    }

    #[tokio::test]
    async fn trait_contract_name_and_ownership() {
        let e = DefaultContextEngine::new(100);
        let engine: Arc<dyn ContextEngine> = Arc::new(e);
        assert_eq!(engine.name(), "default");
        assert!(!engine.owns_compaction());

        let out = engine.assemble(vec![user("hi")]).await.unwrap();
        assert_eq!(out, vec![user("hi")]);
    }
}
