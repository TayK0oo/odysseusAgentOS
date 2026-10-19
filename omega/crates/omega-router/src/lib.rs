//! `omega-router` — routeur de modeles multi-fournisseurs.
//!
//! Logique pure, deterministe, zero reseau. Responsabilites :
//! - classification de tache (`TaskClassifier`, implemente dans `omega-ports`) ;
//! - scoring composite (`ScoringPolicy`, implemente dans `omega-ports`) ;
//! - selection Pareto (`Selector`, implemente dans `omega-ports`) ;
//! - fallback + circuit breaker (`FallbackPolicy`, implemente dans `omega-ports`) ;
//! - cache de decision (`DecisionCache`).
//!
//! Les ports `TaskClassifier`, `ScoringPolicy`, `Selector` et `FallbackPolicy`
//! vivent dans `omega-ports`. `omega-router` fournit les implementations par
//! defaut et les re-exporte pour la commodite des consommateurs.

pub mod cache;
pub mod classifier;
pub mod fallback;
pub mod scoring;
pub mod selector;
pub mod task;

pub use cache::DecisionCache;
pub use classifier::HeuristicTaskClassifier;
pub use fallback::TierFallbackPolicy;
pub use scoring::CompositeScoringPolicy;

// Re-export des ports et types utilises par les consommateurs du routeur.
pub use omega_ports::{FallbackPolicy, ScoringPolicy, Selector, TaskClassifier};
pub use omega_types::{
    Attachment, AttachmentKind, Capabilities, ModelRecord, ModelTier, QualityAxes, RouterDecision,
    RouterError, RoutingRequest, TaskType, Weights,
};
