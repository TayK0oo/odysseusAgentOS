//! Re-export des types catalogue depuis `omega-types`.
//!
//! Les types `ModelRecord`, `QualityAxes` et `ModelTier` sont des types purs ;
//! leur source unique est desormais `omega-types`. Ce fichier preserve la
//! compatibilite d'import `omega_providers::model::*`.

pub use omega_types::{ModelRecord, ModelTier, QualityAxes};
