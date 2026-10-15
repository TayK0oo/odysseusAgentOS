//! Moteur de contexte **par defaut** (`name = "default"`, `owns_compaction = false`).

use std::sync::Mutex;

use async_trait::async_trait;
use omega_ports::ContextEngine;
use omega_types::{Message, OmegaResult, Role};

use crate::budget::ContextBudget;
use crate::token::TokenEstimator;

/// Rapport d'assemblage : journalise ce qui a ete coupe (payload de test).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssembleReport {
    pub kept: usize,
    pub dropped: usize,
    pub estimated_tokens: usize,
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
    pub fn assemble_sync(&self, messages: &[Message]) -> Vec<Message> {
        let total = self.estimator.estimate_messages(messages);
        if total <= self.budget.max_tokens() {
            let report = AssembleReport {
                kept: messages.len(),
                dropped: 0,
                estimated_tokens: total,
                dropped_previews: Vec::new(),
            };
            *self.last_report.lock().expect("rapport empoisonne") = report;
            return messages.to_vec();
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

        let report = AssembleReport {
            kept: result.len(),
            dropped,
            estimated_tokens: self.estimator.estimate_messages(&result),
            dropped_previews,
        };
        *self.last_report.lock().expect("rapport empoisonne") = report;
        result
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

    async fn assemble(&self, messages: Vec<Message>) -> OmegaResult<Vec<Message>> {
        Ok(self.assemble_sync(&messages))
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
        let out = e.assemble_sync(&msgs);
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
        let out = e.assemble_sync(&msgs);

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
        let out = e.assemble_sync(&msgs);

        assert_eq!(out, vec![sys("SSSSSSSS")]);
        let report = e.last_report();
        assert_eq!(report.dropped, 2);
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
