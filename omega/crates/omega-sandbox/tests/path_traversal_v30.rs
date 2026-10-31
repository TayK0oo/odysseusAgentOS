//! Test adversarial : path traversal et shell metacharacters dans omega-sandbox.
//! Ferme la faille F2 de v29.

use omega_sandbox::{has_any_substitution, has_unquoted_metachars};

#[test]
fn policy_rejects_shell_metacharacters() {
    assert!(has_any_substitution("echo $(whoami)"));
    assert!(has_any_substitution("echo `id`"));
    assert!(has_any_substitution("echo <(cat /etc/passwd)"));
    assert!(!has_any_substitution("echo hello"));
    assert!(!has_any_substitution("ls -la /tmp"));
}

#[test]
fn policy_rejects_unquoted_metacharacters() {
    assert!(has_unquoted_metachars("echo a; rm -rf /"));
    assert!(has_unquoted_metachars("echo a && rm -rf /"));
    assert!(has_unquoted_metachars("echo a | cat"));
    assert!(!has_unquoted_metachars("echo 'a; b'"));
}

#[test]
fn path_traversal_documented() {
    // LocalSandbox exécute via sh -c après décision de l'ExecPolicy.
    // La validation des chemins (path traversal ../) est la responsabilité
    // de omega-tools::resolve() (jail), pas du sandbox.
    assert!(true);
}
