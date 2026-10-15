//! Modele de catalogue normalise (point de service : modele × effort × provider).

use serde::{Deserialize, Serialize};

use crate::capabilities::Capabilities;

/// Tier economique/qualitatif d'un modele.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTier {
    /// Petit modele ultra-rapide et bon marche.
    FastCheap,
    /// Modele polyvalent standard.
    Standard,
    /// Modele haut de gamme (raisonnement, code difficile).
    Elite,
    /// Modele auto-heberge (ollama, vllm, llama.cpp...).
    SelfHosted,
}

impl ModelTier {
    /// Ordre d'escalade pour le fallback : FastCheap < Standard < Elite < SelfHosted.
    pub fn rank(self) -> u8 {
        match self {
            ModelTier::FastCheap => 0,
            ModelTier::Standard => 1,
            ModelTier::Elite => 2,
            ModelTier::SelfHosted => 3,
        }
    }
}

/// Axes de qualite notes de 0 a 100.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct QualityAxes {
    pub reasoning: u8,
    pub code: u8,
    pub agentic: u8,
    pub knowledge: u8,
    pub chat: u8,
    pub vision: u8,
}

impl QualityAxes {
    /// Normalise chaque axe sur [0,1].
    pub fn normalized(&self) -> [f64; 6] {
        [
            f64::from(self.reasoning) / 100.0,
            f64::from(self.code) / 100.0,
            f64::from(self.agentic) / 100.0,
            f64::from(self.knowledge) / 100.0,
            f64::from(self.chat) / 100.0,
            f64::from(self.vision) / 100.0,
        ]
    }
}

/// Fiche d'un modele dans le registre.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRecord {
    /// Identifiant canonique (ex. `deepseek-v4.1-flash`).
    pub id: String,
    /// Provider (ex. `openrouter`, `ollama`).
    pub provider: String,
    /// Fenetre de contexte en tokens.
    pub context_length: u32,
    /// Cout d'entree USD par 1M tokens.
    pub cost_input_per_1m: f64,
    /// Cout de sortie USD par 1M tokens.
    pub cost_output_per_1m: f64,
    /// Vitesse mediane de sortie (tok/s).
    pub tokens_per_second: f32,
    /// Time-to-first-token en millisecondes.
    pub ttft_ms: u32,
    /// Axes de qualite.
    pub axes: QualityAxes,
    /// Capacites operationnelles.
    pub capabilities: Capabilities,
    /// Tier economique.
    pub tier: ModelTier,
}

impl ModelRecord {
    /// Cout moyen ponderé par 1M tokens (1/3 entree, 2/3 sortie).
    pub fn blended_cost_per_1m(&self) -> f64 {
        (self.cost_input_per_1m + 2.0 * self.cost_output_per_1m) / 3.0
    }
}
