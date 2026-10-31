//! Tests v26 — `omega-app`.

use omega_app::{is_valid_run_id, FileEventStore, InMemoryEventStore};

#[test]
fn test_is_valid_run_id() {
    assert!(is_valid_run_id("run-1"));
    assert!(!is_valid_run_id(""));
    assert!(!is_valid_run_id("run/1"));
}

#[test]
fn file_event_store_new() {
    let dir = std::env::temp_dir().join("omega-app-v26-test");
    let _ = std::fs::remove_dir_all(&dir);
    let store = FileEventStore::new(dir.clone());
    let _ = store;
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn in_memory_event_store_new() {
    let store = InMemoryEventStore::new();
    let _ = store;
}
