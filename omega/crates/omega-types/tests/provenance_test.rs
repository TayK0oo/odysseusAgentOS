//! Test provenance non-Option sur Event/DurableEvent.

use omega_types::{DurableEvent, Event, Provenance};

#[test]
fn event_provenance_defaults_to_observed() {
    let e = Event::new("test", serde_json::json!(null));
    assert_eq!(e.provenance, Provenance::Observed);
}

#[test]
fn event_provenance_serializes() {
    let e = Event::new("test", serde_json::json!(null)).with_provenance(Provenance::Stated);
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"provenance\":\"stated\""));
}

#[test]
fn durable_event_provenance_defaults_to_observed() {
    let de = DurableEvent::new("run-1", "step.completed", serde_json::json!(null));
    assert_eq!(de.provenance, Provenance::Observed);
}

#[test]
fn provenance_tag() {
    assert_eq!(Provenance::Stated.tag(), "[stated]");
    assert_eq!(Provenance::Observed.tag(), "[observed]");
    assert_eq!(Provenance::Inferred.tag(), "[inferred]");
}
