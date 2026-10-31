//! Tests v27 — `omega-harness`.

use omega_harness::HarnessCatalogContributor;

#[test]
fn harness_contributor_new() {
    let c = HarnessCatalogContributor::new();
    let _ = c;
}

#[test]
fn harness_crate_compiles() {
    let _ = std::mem::size_of::<HarnessCatalogContributor>();
}
