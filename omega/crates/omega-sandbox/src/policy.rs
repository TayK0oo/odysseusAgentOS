//! Politique d'exécution par règles ordonnées (DSL préfixes).
//!
//! Sémantique : la **première règle qui matche** décide. Sans match, le
//! `default` s'applique (`Forbidden` par défaut : deny-by-default).
//! Le matching se fait sur la commande **trimée à gauche**.

use omega_types::ExecDecision;
use serde::{Deserialize, Serialize};

/// Une règle : préfixe → décision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecRule {
    pub prefix: String,
    pub decision: ExecDecision,
}

impl ExecRule {
    pub fn new(prefix: impl Into<String>, decision: ExecDecision) -> Self {
        Self {
            prefix: prefix.into(),
            decision,
        }
    }
}

/// Politique par règles ordonnées. Implémente le port `omega_ports::ExecPolicy`.
#[derive(Debug, Clone)]
pub struct RuleBasedExecPolicy {
    name: String,
    rules: Vec<ExecRule>,
    default: ExecDecision,
}

impl RuleBasedExecPolicy {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rules: Vec::new(),
            default: ExecDecision::Forbidden("deny-by-default".to_string()),
        }
    }

    pub fn rule(mut self, prefix: impl Into<String>, decision: ExecDecision) -> Self {
        self.rules.push(ExecRule::new(prefix, decision));
        self
    }

    pub fn with_default(mut self, default: ExecDecision) -> Self {
        self.default = default;
        self
    }

    /// Évalue une commande : première règle dont le préfixe matche, sinon défaut.
    pub fn decide(&self, command: &str) -> ExecDecision {
        let trimmed = command.trim_start();
        for rule in &self.rules {
            if trimmed.starts_with(&rule.prefix) {
                return rule.decision.clone();
            }
        }
        self.default.clone()
    }
}

impl omega_ports::ExecPolicy for RuleBasedExecPolicy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&self, command: &str) -> ExecDecision {
        self.decide(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> RuleBasedExecPolicy {
        RuleBasedExecPolicy::new("test")
            .rule("rm -rf", ExecDecision::Forbidden("destructeur".to_string()))
            .rule("sudo", ExecDecision::Prompt("élévation".to_string()))
            .rule("echo", ExecDecision::Allow)
            .rule("ls", ExecDecision::Allow)
    }

    #[test]
    fn first_matching_rule_wins() {
        let p = policy();
        assert_eq!(p.decide("echo hello"), ExecDecision::Allow);
        assert_eq!(
            p.decide("rm -rf /tmp/x"),
            ExecDecision::Forbidden("destructeur".to_string())
        );
    }

    #[test]
    fn prompt_rule_matches() {
        let p = policy();
        assert_eq!(
            p.decide("sudo apt update"),
            ExecDecision::Prompt("élévation".to_string())
        );
    }

    #[test]
    fn deny_by_default_and_trims_leading_space() {
        let p = policy();
        assert_eq!(
            p.decide("curl http://x"),
            ExecDecision::Forbidden("deny-by-default".to_string())
        );
        assert_eq!(p.decide("   echo hi"), ExecDecision::Allow);
    }
}
