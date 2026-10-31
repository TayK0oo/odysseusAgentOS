//! Test F1 — redact sk- (OpenAI-style API key).

use omega_types::redact_secrets;

#[test]
fn redact_sk_openai_key() {
    let input = "key: sk-1234567890abcdefghijklmn";
    let out = redact_secrets(input);
    assert!(!out.contains("sk-1234567890"), "sk- key must be redacted");
    assert!(out.contains("[REDACTED:"), "redacted tag present");
}

#[test]
fn redact_sk_short_not_redacted() {
    // Short sk- (not a real key) should not be redacted
    let input = "sk-short";
    let out = redact_secrets(input);
    assert_eq!(out, "sk-short", "short sk- not a secret");
}
