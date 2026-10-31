//! Tests v25 — `omega-core`.

use omega_core::{cancellation_pair, EventBus, InMemoryBus, EVENT_MATRIX};

#[test]
fn cancellation_pair_works() {
    let (token, handle) = cancellation_pair();
    assert!(!token.is_cancelled());
    handle.cancel();
    assert!(token.is_cancelled());
    assert!(token.check().is_err());
}

#[test]
fn in_memory_bus_new() {
    let _bus = InMemoryBus::new();
}

#[test]
fn event_matrix_non_empty() {
    assert!(!EVENT_MATRIX.is_empty());
}
