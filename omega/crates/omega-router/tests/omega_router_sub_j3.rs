//! Tests W3 v23 — `omega-router` sous-incrément J3 (fallback).
//! Le seul missed de fallback.rs (select_by_tier `>` → `>=`) est ÉQUI-PROUVÉ :
//! la pool `higher_tier` n'est consultée que quand `same_tier` est vide, donc
//! `>` et `>=` sélectionnent le même ensemble. Aucun test discriminant nécessaire.
//! Ce fichier contient un smoke test pour confirmer la compilation du crate.

use omega_router::fallback::TierFallbackPolicy;
use omega_router::scoring::CompositeScoringPolicy;

#[test]
fn fallback_smoke() {
    let fb = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
    assert_eq!(fb.max_failures(), 3);
    assert_eq!(fb.open_timeout().as_secs(), 60);
}
