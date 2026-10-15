//! Types partages par les ports de politique de routage.
//!
//! `RouterDecision`, `RouterError` et `Weights` sont definis ici pour que les
//! traits `ScoringPolicy`, `Selector` et `FallbackPolicy` puissent vivre dans
//! `omega-ports` sans dependre d'un adaptateur.

use std::fmt::Write;

use serde::{Deserialize, Serialize};

/// Decision de routage : modele choisi, score et justification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterDecision {
    pub model_id: String,
    pub score: f64,
    pub reason: String,
}

impl RouterDecision {
    /// Helper pour construire une decision avec une raison descriptive.
    pub fn new(model_id: impl Into<String>, score: f64, reason: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            score,
            reason: reason.into(),
        }
    }

    /// Met a jour la raison en conservant les metriques de selection.
    pub fn with_reason(self, reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            ..self
        }
    }
}

/// Erreurs de routage.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RouterError {
    #[error("aucun candidat fourni")]
    EmptyRegistry,
    #[error("aucun modele eligible apres filtrage dur")]
    NoEligibleModel,
}

/// Poids d'un profil de tache. Tous les champs somment a 1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Weights {
    pub reasoning: f64,
    pub code: f64,
    pub agentic: f64,
    pub knowledge: f64,
    pub chat: f64,
    pub vision: f64,
    pub speed: f64,
    pub cost: f64,
    pub context: f64,
}

impl Weights {
    /// Verifie que les poids somment approximativement a 1 (tolerance flottante).
    pub fn is_normalized(&self) -> bool {
        let sum = self.reasoning
            + self.code
            + self.agentic
            + self.knowledge
            + self.chat
            + self.vision
            + self.speed
            + self.cost
            + self.context;
        (sum - 1.0).abs() < 1e-6
    }
}

/// Formate la raison d'une decision de routage.
pub fn format_reason(
    task_code: &str,
    eligible_count: usize,
    pareto_count: usize,
    selected_id: &str,
    score: f64,
) -> String {
    let mut reason = String::new();
    let _ = write!(
        reason,
        "task={} eligible={} pareto={} selected={} score={:.4}",
        task_code, eligible_count, pareto_count, selected_id, score
    );
    reason
}
