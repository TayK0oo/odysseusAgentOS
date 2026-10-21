//! Politique d'exécution par règles ordonnées (DSL préfixes).
//!
//! Sémantique : la **première règle qui matche** décide. Sans match, le
//! `default` s'applique (`Forbidden` par défaut : deny-by-default).
//! Le matching se fait sur la commande **trimée à gauche**, avec le **nom de
//! commande (premier mot) normalisé en minuscules** avant comparaison.
//!
//! > **Casse** : seule la casse du **nom de commande** (premier mot, jusqu'à la
//! > première espace) est normalisée (`EcHo` matche la règle `echo`). Les
//! > **arguments gardent leur casse** : `ECHO Hello` matche le préfixe
//! > `echo Hello` mais pas `echo hello` (comparaison sensible à la casse
//! > au-delà du premier mot).
//!
//! Mode strict (inc2) : les substitutions `$(...)`, backticks et `<(...)` sont
//! **interdites même quotées** (en doubles quotes `$(...)` s'exécute !) et
//! donnent `Forbidden("substitution interdite en mode strict")`. En mode
//! non-strict (défaut), le comportement est inchangé : la politique décide par
//! préfixe et c'est le backend qui escalade les métacaractères hors quotes en
//! `Prompt` (voir `has_unquoted_metachars`).

use omega_types::{ExecDecision, OmegaError, OmegaResult};
use serde::{Deserialize, Serialize};

use crate::approvals::ApprovalQueue;

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

/// Détecte les métacaractères shell **hors quotes** : `;`, `|`, `&`, `>`, `<`,
/// `` ` ``, `$(`, retour ligne. Le scan respecte `'...'`, `"..."` et `\`.
///
/// Une commande composée (`echo hi; rm -rf /`) matche la règle `echo` mais exécute
/// aussi `rm` : la garde anti-composition (voir `LocalSandboxBackend`) l'escalade en
/// `Prompt` au lieu de l'autoriser silencieusement (REVIEW-W8-W9 B1).
pub fn has_unquoted_metachars(command: &str) -> bool {
    let mut chars = command.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(c) = chars.next() {
        if in_single {
            if c == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            match c {
                '"' => in_double = false,
                '\\' => {
                    chars.next();
                }
                '`' => return true,
                '$' if chars.peek() == Some(&'(') => return true,
                _ => {}
            }
            continue;
        }
        match c {
            '\'' => in_single = true,
            '"' => in_double = true,
            '\\' => {
                chars.next();
            }
            ';' | '|' | '&' | '>' | '<' | '\n' | '`' => return true,
            '$' if chars.peek() == Some(&'(') => return true,
            _ => {}
        }
    }
    false
}

/// Détecte les substitutions shell **même quotées** : `$(...)`, backticks,
/// `<(...)`. Seul l'échappement par backslash (`\$(`, `` \` ``) est ignoré.
///
/// En doubles quotes, `$(...)` et les backticks **s'exécutent** : c'est pourquoi
/// le mode strict les interdit partout (les simples quotes les neutralisent en
/// pratique, mais la règle stricte reste conservatrice et les refuse aussi).
pub fn has_any_substitution(command: &str) -> bool {
    let mut chars = command.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '`' => return true,
            '$' if chars.peek() == Some(&'(') => return true,
            '<' if chars.peek() == Some(&'(') => return true,
            _ => {}
        }
    }
    false
}

/// Normalise la casse du **nom de commande** (premier mot, jusqu'à la première
/// espace) en minuscules. Le reste (arguments) est conservé tel quel.
fn normalize_command_name(trimmed: &str) -> String {
    match trimmed.find(char::is_whitespace) {
        Some(idx) => {
            let (head, tail) = trimmed.split_at(idx);
            format!("{}{}", head.to_lowercase(), tail)
        }
        None => trimmed.to_lowercase(),
    }
}

/// Politique par règles ordonnées. Implémente le port `omega_ports::ExecPolicy`.
#[derive(Debug, Clone)]
pub struct RuleBasedExecPolicy {
    name: String,
    rules: Vec<ExecRule>,
    default: ExecDecision,
    strict: bool,
}

impl RuleBasedExecPolicy {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rules: Vec::new(),
            default: ExecDecision::Forbidden("deny-by-default".to_string()),
            strict: false,
        }
    }

    /// Construit une politique en **mode strict** (substitutions interdites,
    /// voir module doc). Équivalent à `new(name).with_strict(true)`.
    pub fn strict(name: impl Into<String>) -> Self {
        Self::new(name).with_strict(true)
    }

    pub fn rule(mut self, prefix: impl Into<String>, decision: ExecDecision) -> Self {
        self.rules.push(ExecRule::new(prefix, decision));
        self
    }

    pub fn with_default(mut self, default: ExecDecision) -> Self {
        self.default = default;
        self
    }

    /// Active ou désactive le mode strict (substitutions `$(...)`/backticks/
    /// `<(...)` interdites même quotées → `Forbidden`).
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Indique si la politique est en mode strict.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Règles ordonnées (inspection/tests).
    pub fn rules(&self) -> &[ExecRule] {
        &self.rules
    }

    /// Décision par défaut (aucune règle ne matche).
    pub fn default_decision(&self) -> ExecDecision {
        self.default.clone()
    }

    /// Évalue une commande : en mode strict, toute substitution donne
    /// `Forbidden` ; sinon première règle dont le préfixe matche (nom de
    /// commande normalisé en minuscules), sinon défaut.
    pub fn decide(&self, command: &str) -> ExecDecision {
        if self.strict && has_any_substitution(command) {
            return ExecDecision::Forbidden("substitution interdite en mode strict".to_string());
        }
        let trimmed = command.trim_start();
        let normalized = normalize_command_name(trimmed);
        for rule in &self.rules {
            if normalized.starts_with(&normalize_command_name(&rule.prefix)) {
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

/// Canon de refus unique de tous les backends sandbox (`local`, `namespace`,
/// `bwrap`) : une seule écriture de la décision.
///
/// - `Allow` **avec métacaractères hors quotes** → `Prompt` (REVIEW-W8-W9 B1 :
///   une règle `Allow echo` ne doit pas autoriser `echo hi; rm -rf /`) ;
/// - `Forbidden` → erreur, **sans fork ni file** ;
/// - `Prompt` → `ApprovalRequired` sans fork, sauf approbation exacte
///   consommée (usage unique) dans la file passée en argument.
pub(crate) fn gate(
    policy: &dyn omega_ports::ExecPolicy,
    command: &str,
    approvals: Option<&std::sync::Arc<ApprovalQueue>>,
) -> OmegaResult<()> {
    // Canon : octets nuls → refus explicite, jamais « au hasard » d'un
    // `execve ArgumentListNotValid` raté (le nul est indépassable pour argv).
    if command.contains('\0') {
        return Err(OmegaError::ForbiddenCommand {
            command: command.to_string(),
            reason: "octet nul interdit".to_string(),
        });
    }
    let decision = match policy.evaluate(command) {
        ExecDecision::Allow if has_unquoted_metachars(command) => {
            ExecDecision::Prompt("commande composée : approbation requise".to_string())
        }
        other => other,
    };
    match decision {
        ExecDecision::Forbidden(reason) => Err(OmegaError::ForbiddenCommand {
            command: command.to_string(),
            reason,
        }),
        ExecDecision::Prompt(reason) => {
            if approvals.map(|q| q.consume_for(command)) != Some(true) {
                return Err(OmegaError::ApprovalRequired {
                    command: command.to_string(),
                    reason,
                });
            }
            Ok(())
        }
        ExecDecision::Allow => Ok(()),
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
    fn gate_refuses_nul_bytes() {
        // Canon : le refus précède tout fork — aucun backend ne peut passer.
        let p = RuleBasedExecPolicy::new("t").rule("cat", ExecDecision::Allow);
        let err = gate(&p, "cat /etc/\0passwd", None).unwrap_err();
        assert!(
            matches!(err, OmegaError::ForbiddenCommand { .. }),
            "obtenu : {err:?}"
        );
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
    fn metachar_detector_respects_quotes_and_escapes() {
        // (B1) métacaractères hors quotes détectés...
        assert!(has_unquoted_metachars("echo hi; touch /tmp/x"));
        assert!(has_unquoted_metachars("echo a && echo b"));
        assert!(has_unquoted_metachars("cat f | grep x"));
        assert!(has_unquoted_metachars("echo $(whoami)"));
        assert!(has_unquoted_metachars("echo `whoami`"));
        assert!(has_unquoted_metachars("echo a > /tmp/f"));
        // ...mais ignorés dans les quotes et après backslash.
        assert!(!has_unquoted_metachars("echo \"a;b\""));
        assert!(!has_unquoted_metachars("echo 'a|b'"));
        assert!(!has_unquoted_metachars("echo a\\;b"));
        assert!(!has_unquoted_metachars("echo $HOME"));
        assert!(!has_unquoted_metachars("echo hello"));
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

    #[test]
    fn command_name_case_is_normalized_but_not_args() {
        // (inc2) seule la casse du nom est normalisée.
        let p = policy();
        assert_eq!(p.decide("EcHo hello"), ExecDecision::Allow);
        assert_eq!(p.decide("ECHO hello"), ExecDecision::Allow);
        assert_eq!(p.decide("eChO hello"), ExecDecision::Allow);
        assert_eq!(p.decide("   EcHo hi"), ExecDecision::Allow);
        assert_eq!(
            p.decide("SUDO apt update"),
            ExecDecision::Prompt("élévation".to_string())
        );
        // Les arguments gardent leur casse : préfixe `echo Hello` ≠ `echo hello`.
        let q = RuleBasedExecPolicy::new("t").rule("echo Hello", ExecDecision::Allow);
        assert_eq!(q.decide("ECHO Hello"), ExecDecision::Allow);
        assert_eq!(
            q.decide("ECHO hello"),
            ExecDecision::Forbidden("deny-by-default".to_string())
        );
    }

    #[test]
    fn strict_constructor_and_flag() {
        assert!(!RuleBasedExecPolicy::new("a").is_strict());
        assert!(RuleBasedExecPolicy::strict("b").is_strict());
        assert!(RuleBasedExecPolicy::new("c").with_strict(true).is_strict());
        assert!(!RuleBasedExecPolicy::strict("d")
            .with_strict(false)
            .is_strict());
    }

    #[test]
    fn substitution_detector_fires_even_quoted() {
        assert!(has_any_substitution("echo $(whoami)"));
        assert!(has_any_substitution("echo \"$(whoami)\""));
        assert!(has_any_substitution("echo '$(whoami)'"));
        assert!(has_any_substitution("echo `whoami`"));
        assert!(has_any_substitution("echo \"`whoami`\""));
        assert!(has_any_substitution("cat <(ls)"));
        assert!(has_any_substitution("echo \"<(ls)\""));
        // Échappées ou absentes : rien.
        assert!(!has_any_substitution("echo hello"));
        assert!(!has_any_substitution("echo $HOME"));
        assert!(!has_any_substitution("echo \\$(whoami)"));
        assert!(!has_any_substitution("echo a;b"));
    }

    #[test]
    fn strict_mode_forbids_substitutions_quoted_or_not() {
        let s = RuleBasedExecPolicy::strict("s").rule("echo", ExecDecision::Allow);
        for cmd in [
            "echo $(whoami)",
            "echo \"$(whoami)\"",
            "echo 'a $(whoami)'",
            "echo `whoami`",
            "echo \"`whoami`\"",
            "cat <(ls)",
            "EcHo $(whoami)",
        ] {
            assert_eq!(
                s.decide(cmd),
                ExecDecision::Forbidden("substitution interdite en mode strict".to_string()),
                "commande : {cmd}"
            );
        }
        // Sans substitution : règles normales (avec casse normalisée).
        assert_eq!(s.decide("EcHo hello"), ExecDecision::Allow);
        assert_eq!(s.decide("echo $HOME"), ExecDecision::Allow);
    }

    #[test]
    fn non_strict_mode_keeps_prefix_behavior() {
        // En mode non-strict, une substitution quotée ne bloque pas la décision
        // par préfixe (c'est le backend qui escalade les métacaractères hors
        // quotes en `Prompt`).
        let p = policy();
        assert!(!p.is_strict());
        assert_eq!(p.decide("echo \"$(whoami)\""), ExecDecision::Allow);
        assert_eq!(p.decide("echo $(whoami)"), ExecDecision::Allow);
    }
}
