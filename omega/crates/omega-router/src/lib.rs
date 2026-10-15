//! `omega-router` — routeur de modeles multi-fournisseurs.
//!
//! Logique pure, deterministe, zero reseau. Responsabilites :
//! - classification de tache (`TaskClassifier`) ;
//! - scoring composite (`ScoringPolicy`) ;
//! - selection Pareto (`Selector`) ;
//! - fallback + circuit breaker (`FallbackPolicy`) ;
//! - cache de decision (`DecisionCache`).

pub mod cache;
pub mod classifier;
pub mod fallback;
pub mod scoring;
pub mod selector;
pub mod task;

pub use cache::DecisionCache;
pub use classifier::{
    Attachment, AttachmentKind, HeuristicTaskClassifier, RoutingRequest, TaskClassifier,
};
pub use fallback::{FallbackPolicy, TierFallbackPolicy};
pub use scoring::{CompositeScoringPolicy, ScoringPolicy, Weights};
pub use selector::{ParetoSelector, RouterDecision, RouterError, Selector};
pub use task::TaskType;
