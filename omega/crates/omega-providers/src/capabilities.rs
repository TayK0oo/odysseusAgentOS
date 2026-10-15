//! Capacites operationnelles d'un point de service (modele × provider).

use serde::{Deserialize, Serialize};

/// Capacites binaires requises pour l'execution d'une tache.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capabilities {
    /// Function-calling / tool use.
    pub function_calling: bool,
    /// Entree ou sortie image/audio/video.
    pub multimodal: bool,
    /// JSON schema / response_format.
    pub structured_output: bool,
    /// Modele de raisonnement pilotable (thinking).
    pub reasoning: bool,
}

impl Capabilities {
    /// True si ce modele satisfait toutes les capacites demandees.
    pub fn covers(self, required: Capabilities) -> bool {
        (!required.function_calling || self.function_calling)
            && (!required.multimodal || self.multimodal)
            && (!required.structured_output || self.structured_output)
            && (!required.reasoning || self.reasoning)
    }
}
