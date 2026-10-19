//! `omega-providers` — types du catalogue de modeles.
//!
//! Ce crate est la feuille des types fournisseurs : `ModelRecord`, `QualityAxes`,
//! `Capabilities`, `ModelTier`. Aucune logique de routing ni d'appel reseau.

pub mod capabilities;
pub mod model;
pub mod ollama;
pub mod ollama_catalog;

pub use capabilities::Capabilities;
pub use model::{ModelRecord, ModelTier, QualityAxes};
pub use ollama::OllamaProvider;
pub use ollama_catalog::{
    local_record, parse_tags_response, self_hosted_first, OllamaCatalogSource,
    DEFAULT_LOCAL_CONTEXT, LOCAL_AXES, SELF_HOST_MARGIN,
};
