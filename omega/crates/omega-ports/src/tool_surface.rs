//! Port `ToolSurface` — surface d'outils exécutable (P1-1, plan §6 ; ADR-0006).
//!
//! Le noyau (`omega-core::tool_loop`) détecte les tool-calls puis exécute via ce
//! trait ; les adaptateurs (`omega-tools`, MCP...) l'implémentent, comme les
//! autres ports du crate (aucune implémentation ici).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use omega_types::OmegaResult;

/// Origine du contenu d'un tool-call détecté (traçabilité ADR-0006) : champ
/// `tool_calls` natif du wire, ou repli ReAct (JSON texte canonique).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCallOrigin {
    Native,
    ReAct,
}

impl ToolCallOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            ToolCallOrigin::Native => "native",
            ToolCallOrigin::ReAct => "react",
        }
    }
}

/// Appel d'outil détecté. `call_id` : identifiant fourni par le wire (forme
/// native) ou attribué par la boucle (`call-N` par numéro de round, forme ReAct).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
    pub origin: ToolCallOrigin,
}

/// Observation renvoyée au modèle. L'échec de l'outil est `ok=false` avec un
/// contenu structuré `{"error": ...}` — l'exécution est tracée comme les autres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolObservation {
    pub call_id: String,
    pub ok: bool,
    pub content: Value,
}

impl ToolObservation {
    pub fn ok(call_id: impl Into<String>, content: Value) -> Self {
        Self {
            call_id: call_id.into(),
            ok: true,
            content,
        }
    }

    pub fn error(call_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            ok: false,
            content: json!({ "error": message.into() }),
        }
    }
}

/// Port de la surface d'outils exécutable (P1-1). Les outils concrets
/// (`omega-tools`, MCP...) s'implémentent derrière ce trait.
///
/// Contrat : `execute` renvoie **toujours** une observation structurée — un
/// outil inconnu ou en échec donne `ok=false`, jamais de panic.
#[async_trait::async_trait]
pub trait ToolSurface: Send + Sync {
    async fn execute(&self, call: &ToolCall) -> OmegaResult<ToolObservation>;
}
