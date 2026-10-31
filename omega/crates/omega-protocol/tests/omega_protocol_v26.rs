//! Tests v26 — `omega-protocol` (smoke).

#[test]
fn schema_version_is_one() {
    assert_eq!(omega_protocol::SCHEMA_VERSION, 1);
}

#[test]
fn run_request_round_trip() {
    let req = omega_protocol::RunRequest::new("run-42", "implement feature x");
    let json = serde_json::to_string(&req).unwrap();
    let parsed: omega_protocol::RunRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.run_id, "run-42");
    assert_eq!(parsed.schema_version, omega_protocol::SCHEMA_VERSION);
}
