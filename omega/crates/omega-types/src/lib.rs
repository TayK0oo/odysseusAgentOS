//! `omega-types` — types purs du domaine Omega.
//!
//! Regle (RULES-OMEGA / blueprint §2) : **zero logique applicative, zero dependance
//! interne**. Ce crate est la feuille du graphe de dependances.

use serde::{Deserialize, Serialize};

mod error;
pub use error::{OmegaError, OmegaResult};

/// Identifiant de modele (opaque : c'est le registre qui connait sa semantique).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Role d'un message de conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Message de conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

/// Comptage de tokens d'un appel modele (cache inclus).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_write_tokens: u32,
}

impl Usage {
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// Requete d'inference adressee a un `Provider`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: ModelId,
    pub messages: Vec<Message>,
}

/// Reponse d'inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    pub usage: Usage,
}

/// Modele brut tel que renvoye par une source de catalogue (`CatalogSource`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawModel {
    pub id: String,
    pub provider: String,
    pub context_length: Option<u32>,
}

/// Specification d'un outil (exposee au modele).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
}

/// Resultat d'execution d'une commande en sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Effet d'une politique de permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionEffect {
    Allow,
    Ask,
    Deny,
}

/// Evenement de telemetrie (nom + champs), emis par un `TelemetrySink`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub name: String,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

/// Capacite = **besoin nomme du systeme** (jamais une marque).
///
/// Un plugin declare une capacite ; le registre choisit l'implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    LlmProvider,
    CatalogSource,
    ModelRegistry,
    TaskClassifier,
    RouterPolicy,
    RouterFeedback,
    ContextEngine,
    CompactionModel,
    PromptCachePolicy,
    SkillStore,
    MemoryStore,
    MemorySearch,
    Tool,
    ToolSurface,
    SandboxBackend,
    ExecPolicy,
    PermissionPolicy,
    Hook,
    SubagentRunner,
    SchedulerJob,
    Channel,
    TelemetrySink,
    EventStore,
    ProvenanceSink,
    UiPanel,
}

/// Regime d'activation d'une capacite (cf. ADR-0003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    /// Une seule implementation active a la fois (`context-engine`, `memory-store`...).
    Slot,
    /// N implementations actives concurrentes (`llm-provider`, `tool`...).
    Ensemble,
}

impl Capability {
    /// Regime de la capacite (slot exclusif vs ensemble).
    pub fn regime(self) -> Regime {
        use Capability::*;
        match self {
            ModelRegistry | TaskClassifier | RouterPolicy | ContextEngine | PromptCachePolicy
            | MemoryStore | ExecPolicy | PermissionPolicy | EventStore => Regime::Slot,
            _ => Regime::Ensemble,
        }
    }

    /// Nom stable (kebab-case), utile en config et en telemetrie.
    pub fn as_str(self) -> &'static str {
        use Capability::*;
        match self {
            LlmProvider => "llm-provider",
            CatalogSource => "catalog-source",
            ModelRegistry => "model-registry",
            TaskClassifier => "task-classifier",
            RouterPolicy => "router-policy",
            RouterFeedback => "router-feedback",
            ContextEngine => "context-engine",
            CompactionModel => "compaction-model",
            PromptCachePolicy => "prompt-cache-policy",
            SkillStore => "skill-store",
            MemoryStore => "memory-store",
            MemorySearch => "memory-search",
            Tool => "tool",
            ToolSurface => "tool-surface",
            SandboxBackend => "sandbox-backend",
            ExecPolicy => "exec-policy",
            PermissionPolicy => "permission-policy",
            Hook => "hook",
            SubagentRunner => "subagent-runner",
            SchedulerJob => "scheduler-job",
            Channel => "channel",
            TelemetrySink => "telemetry-sink",
            EventStore => "event-store",
            ProvenanceSink => "provenance-sink",
            UiPanel => "ui-panel",
        }
    }
}

/// Type de tache (T1..T14) pour le routeur — voir `MODEL-ROUTING-BENCHMARKS.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskType {
    TrivialCompletion,
    TargetedEdit,
    Refactor,
    Implementation,
    LongReasoning,
    Math,
    Research,
    AgenticTerminal,
    Vision,
    Audio,
    Embedding,
    MediaGeneration,
    Summarization,
    Chat,
}

impl TaskType {
    /// Code court T1..T14.
    pub fn code(self) -> &'static str {
        use TaskType::*;
        match self {
            TrivialCompletion => "T1",
            TargetedEdit => "T2",
            Refactor => "T3",
            Implementation => "T4",
            LongReasoning => "T5",
            Math => "T6",
            Research => "T7",
            AgenticTerminal => "T8",
            Vision => "T9",
            Audio => "T10",
            Embedding => "T11",
            MediaGeneration => "T12",
            Summarization => "T13",
            Chat => "T14",
        }
    }
}
