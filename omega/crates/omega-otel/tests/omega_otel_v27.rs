//! Tests v27 — `omega-otel`.

use omega_otel::FileSink;

#[test]
fn file_sink_new() {
    let dir = std::env::temp_dir().join("omega-otel-v27");
    let _ = std::fs::remove_dir_all(&dir);
    let sink = FileSink::new(&dir);
    let _ = sink;
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn telemetry_event_construction() {
    // Just verify the crate compiles and links
    let _ = std::mem::size_of::<omega_otel::FileSink>();
}
