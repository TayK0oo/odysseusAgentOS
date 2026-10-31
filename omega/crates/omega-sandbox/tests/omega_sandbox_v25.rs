//! Tests v25 — `omega-sandbox`.

use omega_sandbox::{has_any_substitution, has_unquoted_metachars, ExecRule, RuleBasedExecPolicy};
use omega_types::ExecDecision;

#[test]
fn has_any_substitution_detects() {
    assert!(has_any_substitution("echo $(whoami)"));
    assert!(!has_any_substitution("echo hello"));
}

#[test]
fn has_unquoted_metachars_detects() {
    assert!(has_unquoted_metachars("echo a; rm -rf /"));
    assert!(!has_unquoted_metachars("echo 'a; b'"));
}

#[test]
fn exec_rule_construction() {
    let r = ExecRule::new("ls", ExecDecision::Allow);
    assert_eq!(r.prefix, "ls");
}

#[test]
fn rule_based_policy_new() {
    let p = RuleBasedExecPolicy::new("default");
    let _ = p.rules();
}
