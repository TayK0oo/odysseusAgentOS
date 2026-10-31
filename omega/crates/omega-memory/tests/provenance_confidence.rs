//! Tests contractuels — helpers de provenance et de confidence (omega-memory).

use omega_memory::{confidence_for, normalize_provenance, provenance_tag, MemoryRecord};

#[test]
fn normalize_provenance_canonicalizes_known() {
    assert_eq!(normalize_provenance("stated"), "stated");
    assert_eq!(normalize_provenance("OBSERVED"), "observed");
    assert_eq!(normalize_provenance(" [Inferred] "), "inferred");
    assert_eq!(normalize_provenance("[stated]"), "stated");
}

#[test]
fn normalize_provenance_defaults_to_stated_for_unknown() {
    assert_eq!(normalize_provenance(""), "stated");
    assert_eq!(normalize_provenance("xyzzy"), "stated");
    assert_eq!(normalize_provenance("  "), "stated");
}

#[test]
fn provenance_tag_wraps_canonical() {
    assert_eq!(provenance_tag("observed"), "[observed]");
    assert_eq!(provenance_tag("UNKNOWN"), "[stated]");
}

#[test]
fn confidence_for_maps_provenance() {
    assert_eq!(confidence_for("stated"), 1.0);
    assert_eq!(confidence_for("observed"), 0.7);
    assert_eq!(confidence_for("inferred"), 0.4);
    assert_eq!(confidence_for("unknown"), 1.0); // unknown normalizes to stated
}

#[test]
fn default_provenance_and_confidence() {
    let r = MemoryRecord::new("id", "text");
    assert_eq!(r.provenance, "stated");
    assert_eq!(r.confidence, 1.0);
}

#[test]
fn is_expired_respects_ttl() {
    let mut r = MemoryRecord::new("id", "text");
    r.timestamp = 100;
    r.ttl_seconds = Some(10);
    assert!(!r.is_expired(109));
    assert!(r.is_expired(110));
    assert!(r.is_expired(200));
}

#[test]
fn is_expired_no_ttl_never_expires() {
    let mut r = MemoryRecord::new("id", "text");
    r.timestamp = 100;
    r.ttl_seconds = None;
    assert!(!r.is_expired(10_000));
}

#[test]
fn is_expired_negative_ttl_never_expires() {
    let mut r = MemoryRecord::new("id", "text");
    r.timestamp = 100;
    r.ttl_seconds = Some(-5);
    assert!(!r.is_expired(10_000));
}

#[test]
fn defaults_on_deserialization() {
    let json = r#"{"id":"x","text":"y","timestamp":1}"#;
    let r: MemoryRecord = serde_json::from_str(json).unwrap();
    assert_eq!(r.provenance, "stated");
    assert_eq!(r.confidence, 1.0);
}

#[test]
fn is_expired_now_no_ttl_never_expires() {
    let mut r = MemoryRecord::new("id", "text");
    r.ttl_seconds = None;
    assert!(!r.is_expired_now());
}

#[test]
fn is_expired_now_expired_record() {
    let mut r = MemoryRecord::new("id", "text");
    r.timestamp = 0;
    r.ttl_seconds = Some(0);
    assert!(r.is_expired_now());
}
