use omega_context::{PinnedFact, PinnedFacts};

#[test]
fn empty_store_is_empty() {
    let pf = PinnedFacts::new();
    assert!(pf.is_empty());
    assert_eq!(pf.len(), 0);
}

#[test]
fn upsert_makes_non_empty() {
    let pf = PinnedFacts::new();
    pf.upsert(PinnedFact::new("id1", "content", "decision"));
    assert!(!pf.is_empty());
    assert_eq!(pf.len(), 1);
}
