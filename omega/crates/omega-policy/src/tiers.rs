//! Politique par paliers T0–T3 (PLAN-MAITRE.md §6 P1-3, défaut = REFUS).
//!
//! - **T0** lecture seule — autoris.
//! - **T1** écriture canari — autorisé, l'appelant trace le caractère canari.
//! - **T2** réseau / exécution — **interdit tant que G1.sandbox n'est pas vert**
//!   (cf. ADR via plan §6 P1-6) ; sinon passe par le port d'approbation.
//! - **T3** coût / sous-plan — autorisé, l'appelant consomme d'abord le
//!   [`Budget`](crate::budget::Budget) (garde quantitative séparée).
//!
//! **Défaut = REFUS (fail-closed)** : toute action non classée dans la table
//! est refusée, jamais autorisée par défaut. La table est une donnée
//! (`HashMap`) sérialisable — branchage par config sans toucher ce crate.

use std::collections::HashMap;
use std::fmt;

/// Paliers documentés.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// Lecture seule.
    T0,
    /// Écriture canari.
    T1,
    /// Réseau / exécution (booking requis ; interdit si G1.sandbox non vert).
    T2,
    /// Coût / sous-plan (garde quantitative : budget).
    T3,
}

/// Décision de la politique par paliers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TierDecision {
    Allow {
        tier: Tier,
    },
    /// Config de donnée : une raison traçable, jamais un refus muet.
    Deny {
        reason: String,
    },
}

impl fmt::Display for TierDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TierDecision::Allow { tier } => write!(f, "allow (palier {tier:?})"),
            TierDecision::Deny { reason } => write!(f, "deny : {reason}"),
        }
    }
}

/// Demande d'élévation (ex. T1 → T2) soumise au port d'approbation injecté.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ElevationRequest {
    pub action: String,
    pub tier: Tier,
    pub reason: String,
}

/// Port d'approbation injecté (boucle UI A2 ou humain). Aucune boucle UI ici :
/// juste le contrat. **Défaut = refus** ([`DenyAllApprovals`]).
pub trait ApprovalGate: Send + Sync {
    fn name(&self) -> &str;
    fn approve(&self, req: &ElevationRequest) -> bool;
}

/// Défaut fail-closed : toute demande d'élévation est refusée.
pub struct DenyAllApprovals;

impl ApprovalGate for DenyAllApprovals {
    fn name(&self) -> &str {
        "deny-all"
    }
    fn approve(&self, _req: &ElevationRequest) -> bool {
        false
    }
}

/// Politique par paliers, défaut = REFUS. La table des classifications est
/// une donnée : `standard()` fournit la table documentée, une config
/// externes peut la remplacer ligne par ligne via `with_classifications`.
pub struct TierPolicy {
    classification: HashMap<String, Tier>,
    /// T2 reste interdit tant que G1.sandbox n'est pas vert (P1-6).
    t2_open: bool,
    approvals: Box<dyn ApprovalGate>,
}

impl TierPolicy {
    /// Table statique standard, documentée (P1-3) — config de donnée.
    pub fn standard() -> Self {
        Self::new(
            [
                ("read", Tier::T0),
                ("list", Tier::T0),
                ("grep", Tier::T0),
                ("write", Tier::T1),
                ("create", Tier::T1),
                ("edit", Tier::T1),
                ("remove", Tier::T1),
                ("network", Tier::T2),
                ("exec", Tier::T2),
                ("subagent", Tier::T3),
                ("sub-plan", Tier::T3),
            ]
            .into_iter()
            .map(|(k, t)| (k.to_string(), t)),
        )
    }

    /// Construit depuis une donnée (config / adaptateur ConfigSource en A4).
    pub fn new<I, S>(classification: I) -> Self
    where
        I: IntoIterator<Item = (S, Tier)>,
        S: Into<String>,
    {
        Self {
            classification: classification
                .into_iter()
                .map(|(k, t)| (k.into(), t))
                .collect(),
            t2_open: false,
            approvals: Box::new(DenyAllApprovals),
        }
    }

    /// Marque qu'une action appartient à un palier (config de donnée).
    pub fn with_classification(mut self, action: impl Into<String>, tier: Tier) -> Self {
        self.classification.insert(action.into(), tier);
        self
    }

    /// Ouverture explicite de T2 (G1.sandbox vert) — défaut : fermé.
    pub fn with_t2_open(mut self, open: bool) -> Self {
        self.t2_open = open;
        self
    }

    /// Remplace le port d'approbation (défaut : [`DenyAllApprovals`] = refus).
    pub fn with_approvals(mut self, approvals: Box<dyn ApprovalGate>) -> Self {
        self.approvals = approvals;
        self
    }

    /// Palier classé pour une action, si connu.
    pub fn classify(&self, action: &str) -> Option<Tier> {
        self.classification.get(action).copied()
    }

    /// Décision fail-closed : action non classée ⇒ refus ; T2 fermé ⇒ refus
    /// (même si l'approbateur dit oui) ; T2 ouvert ⇒ approbation ; T0/T1/T3
    /// ⇒ autorisé avec son palier (à tracer par l'appelant).
    pub fn authorize(&self, action: &str) -> TierDecision {
        let Some(tier) = self.classification.get(action).copied() else {
            return TierDecision::Deny {
                reason: format!(
                    "action '{}' non categorie : defaut = refus (fail-closed)",
                    action
                ),
            };
        };
        match tier {
            Tier::T0 | Tier::T1 | Tier::T3 => TierDecision::Allow { tier },
            Tier::T2 => {
                if !self.t2_open {
                    return TierDecision::Deny {
                        reason: format!(
                            "T2 ('{}') interdit tant que G1.sandbox n'est pas vert",
                            action
                        ),
                    };
                }
                if self.approvals.approve(&ElevationRequest {
                    action: action.to_string(),
                    tier: Tier::T2,
                    reason: "elevation T1->T2".to_string(),
                }) {
                    TierDecision::Allow { tier: Tier::T2 }
                } else {
                    TierDecision::Deny {
                        reason: format!("approbation refusee pour '{}' (T2)", action),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Approbateur qui dit toujours oui — pour prouver que la fermeture T2
    /// (G1.sandbox) gagne Malgré lui.
    struct AlwaysApprove;
    impl ApprovalGate for AlwaysApprove {
        fn name(&self) -> &str {
            "always-approve"
        }
        fn approve(&self, _req: &ElevationRequest) -> bool {
            true
        }
    }

    #[test]
    fn standard_table_classifies_documented_tiers() {
        let p = TierPolicy::standard();
        assert_eq!(p.classify("read"), Some(Tier::T0));
        assert_eq!(p.classify("write"), Some(Tier::T1));
        assert_eq!(p.classify("network"), Some(Tier::T2));
        assert_eq!(p.classify("subagent"), Some(Tier::T3));
    }

    #[test]
    fn unclassified_action_is_denied_fail_closed() {
        let p = TierPolicy::standard();
        let d = p.authorize("invented_by_model");
        assert!(
            matches!(&d, TierDecision::Deny { reason } if reason.contains("non categorie")),
            "défaut = refus, jamais autorisé par défaut : {d}"
        );
    }

    #[test]
    fn t0_and_t1_allowed_t2_denied_while_sandbox_not_green() {
        let p = TierPolicy::standard()
            .with_approvals(Box::new(AlwaysApprove));
        assert_eq!(p.authorize("read"), TierDecision::Allow { tier: Tier::T0 });
        assert_eq!(p.authorize("write"), TierDecision::Allow { tier: Tier::T1 });
        // T2 fermé : refus malgré l'approbateur oui-oui.
        assert!(matches!(
            p.authorize("network"),
            TierDecision::Deny { .. }
        ));
    }

    #[test]
    fn t2_open_with_default_approvals_still_denied() {
        let p = TierPolicy::standard().with_t2_open(true);
        assert!(matches!(p.authorize("exec"), TierDecision::Deny { .. }));
    }

    #[test]
    fn t2_open_with_granted_approval_allowed() {
        let p = TierPolicy::standard()
            .with_t2_open(true)
            .with_approvals(Box::new(AlwaysApprove));
        assert_eq!(p.authorize("network"), TierDecision::Allow { tier: Tier::T2 });
    }

    #[test]
    fn config_data_extends_table_without_touching_crate() {
        // Une config externe ajoute une action classée ; le reste reste
        // fail-closed.
        let p = TierPolicy::standard().with_classification("rmtree", Tier::T2);
        assert_eq!(p.classify("rmtree"), Some(Tier::T2));
        assert!(matches!(p.authorize("rmtree"), TierDecision::Deny { .. })); // T2 fermé
    }
}
