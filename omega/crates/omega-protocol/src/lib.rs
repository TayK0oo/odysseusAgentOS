//! `omega-protocol` — **wire types** source unique pour la frontière serveur ↔ UI.
//!
//! Leçon n°7 du reverse-engineering : la frontière Rust ↔ TS est générée et le schéma
//! est versionné. Chaque DTO expose `schema_version: u32` pour la compatibilité ascendante.
//!
//! Dépendance autorisée : `omega-types` uniquement (`OMEGA-BLUEPRINT.md` §2.1).

use omega_types::{Capability, ModelId, TaskType};
use serde::{Deserialize, Serialize};

/// Version commune du schéma wire. Incrémentée à chaque rupture de compatibilité.
pub const SCHEMA_VERSION: u32 = 1;

/// Requête de run envoyée par l'UI au serveur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRequest {
    pub schema_version: u32,
    pub run_id: String,
    pub task: String,
    pub config: serde_json::Value,
}

impl RunRequest {
    pub fn new(run_id: impl Into<String>, task: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            run_id: run_id.into(),
            task: task.into(),
            config: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }
}

/// Événement de run diffusé vers l'UI (server-sent events / websocket).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunEvent {
    pub schema_version: u32,
    pub run_id: String,
    pub name: String,
    pub payload: serde_json::Value,
    pub timestamp_ms: u64,
}

impl RunEvent {
    pub fn new(
        run_id: impl Into<String>,
        name: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            run_id: run_id.into(),
            name: name.into(),
            payload,
            timestamp_ms: 0,
        }
    }
}

/// Métadonnées d'un plugin exposées par le serveur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInfo {
    pub schema_version: u32,
    pub id: String,
    pub capability: Capability,
    pub version: String,
    pub priority: i32,
}

impl PluginInfo {
    pub fn new(id: impl Into<String>, capability: Capability, version: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: id.into(),
            capability,
            version: version.into(),
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Décision du routeur de modèles (traçabilité complète).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterDecision {
    pub schema_version: u32,
    pub task: TaskType,
    pub chosen_model: ModelId,
    pub scores: serde_json::Value,
}

impl RouterDecision {
    pub fn new(task: TaskType, chosen_model: ModelId) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            task,
            chosen_model,
            scores: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// Manifeste d'un plugin sérialisé (DTO wire).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestDto {
    pub schema_version: u32,
    pub id: String,
    pub capability: Capability,
    pub version: String,
    pub priority: i32,
}

impl ManifestDto {
    pub fn new(id: impl Into<String>, capability: Capability, version: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: id.into(),
            capability,
            version: version.into(),
            priority: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(value: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn run_request_round_trip() {
        let req = RunRequest::new("run-42", "implement feature x").config(serde_json::json!({
            "model": "cheap",
        }));
        assert_eq!(round_trip(&req), req);
        assert_eq!(req.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn run_event_round_trip() {
        let ev = RunEvent::new("run-42", "step.completed", serde_json::json!({"op": "a"}));
        assert_eq!(round_trip(&ev), ev);
        assert_eq!(ev.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn plugin_info_round_trip() {
        let info = PluginInfo::new("ollama", Capability::LlmProvider, "1").with_priority(5);
        assert_eq!(round_trip(&info), info);
    }

    #[test]
    fn router_decision_round_trip() {
        let decision =
            RouterDecision::new(TaskType::Implementation, ModelId::new("claude-sonnet-4"));
        assert_eq!(round_trip(&decision), decision);
    }

    #[test]
    fn manifest_dto_round_trip() {
        let manifest = ManifestDto::new("ctx-default", Capability::ContextEngine, "1");
        assert_eq!(round_trip(&manifest), manifest);
    }
}
