//! Tests v27 — `omega-adapters`.

use omega_adapters::composition::compose_ported_crates;

#[test]
fn test_compose_ported_crates() {
    let reg = compose_ported_crates();
    let _ = reg;
}

#[test]
fn ui_surface_slot() {
    use omega_adapters::composition::CompositionRegistry;
    let reg = compose_ported_crates();
    // Just verify it compiles and runs
    let _ = reg;
}
