//! `omega-providers` — types du catalogue de modeles.
//!
//! Ce crate est la feuille des types fournisseurs : `ModelRecord`, `QualityAxes`,
//! `Capabilities`, `ModelTier`. Aucune logique de routing ni d'appel reseau.

pub mod capabilities;
pub mod model;
pub mod ollama;

pub use capabilities::Capabilities;
pub use model::{ModelRecord, ModelTier, QualityAxes};
pub use ollama::OllamaProvider;
