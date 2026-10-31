//! Tests v25 — `omega-types` (secret_redact).

use omega_types::{redact_json, redact_secrets, secret_tag};

#[test]
fn secret_tag_masks_value() {
    let tag = secret_tag("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4");
    assert!(
        tag.contains("...")
            || tag.contains("***")
            || !tag.contains("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4")
    );
}

#[test]
fn redact_secrets_masks_in_text() {
    let input = "token=a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4 and other=b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5";
    let out = redact_secrets(input);
    assert!(!out.contains("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"));
    assert!(!out.contains("b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5"));
}

#[test]
fn redact_json_string_masks_secret() {
    let mut v = serde_json::json!({"key": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"});
    redact_json(&mut v);
    let s = v.to_string();
    assert!(!s.contains("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"));
}

#[test]
fn redact_json_array_masks_secrets() {
    let mut v = serde_json::json!(["a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4", "safe"]);
    redact_json(&mut v);
    let s = v.to_string();
    assert!(!s.contains("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"));
    assert!(s.contains("safe"));
}

#[test]
fn redact_json_object_masks_nested() {
    let mut v = serde_json::json!({"a": {"b": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"}});
    redact_json(&mut v);
    let s = v.to_string();
    assert!(!s.contains("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"));
}

#[test]
fn redact_json_number_unchanged() {
    let mut v = serde_json::json!(42);
    redact_json(&mut v);
    assert_eq!(v, serde_json::json!(42));
}

#[test]
fn redact_json_bool_unchanged() {
    let mut v = serde_json::json!(true);
    redact_json(&mut v);
    assert_eq!(v, serde_json::json!(true));
}

#[test]
fn redact_json_null_unchanged() {
    let mut v = serde_json::json!(null);
    redact_json(&mut v);
    assert_eq!(v, serde_json::json!(null));
}
