//! `omega-types` — types purs du domaine Omega.
//!
//! Regle (RULES-OMEGA / blueprint §2) : **zero logique applicative, zero dependance
//! interne**. Ce crate est la feuille du graphe de dependances.

use serde::{Deserialize, Serialize};

mod error;
pub mod model;
pub mod routing;

pub use error::{OmegaError, OmegaResult};
pub use model::{ModelRecord, ModelTier, QualityAxes};
pub use routing::{format_reason, RouterDecision, RouterError, Weights};

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

/// Capacites operationnelles binaires requises pour l'execution d'une tache.
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

/// Type de tache (T1..T14) pour le routeur — voir `MODEL-ROUTING-BENCHMARKS.md` §3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskType {
    /// T1 — Completion triviale / small-talk.
    TrivialCompletion,
    /// T2 — Resume / reformatage.
    Summarization,
    /// T3 — Edition ciblee (1 fichier, patch simple).
    TargetedEdit,
    /// T4 — Refactor / multi-fichiers.
    Refactor,
    /// T5 — Implementation from scratch.
    Implementation,
    /// T6 — Raisonnement long / plan.
    LongReasoning,
    /// T7 — Maths / sciences dures.
    Math,
    /// T8 — Recherche / synthese web.
    Research,
    /// T9 — Agentique terminal / ops.
    AgenticTerminal,
    /// T10 — Vision / OCR / UI.
    Vision,
    /// T11 — Transcription / audio.
    Audio,
    /// T12 — Embeddings / RAG.
    Embedding,
    /// T13 — Generation media (image/video/audio).
    MediaGeneration,
    /// T14 — Chat generaliste.
    Chat,
}

impl TaskType {
    /// Code court T1..T14.
    pub fn code(self) -> &'static str {
        use TaskType::*;
        match self {
            TrivialCompletion => "T1",
            Summarization => "T2",
            TargetedEdit => "T3",
            Refactor => "T4",
            Implementation => "T5",
            LongReasoning => "T6",
            Math => "T7",
            Research => "T8",
            AgenticTerminal => "T9",
            Vision => "T10",
            Audio => "T11",
            Embedding => "T12",
            MediaGeneration => "T13",
            Chat => "T14",
        }
    }
}

/// Type d'une piece jointe en entree de classifieur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentKind {
    Image,
    Audio,
    Video,
    Other,
}

/// Piece jointe detectee dans la requete de routage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attachment {
    pub kind: AttachmentKind,
}

/// Requete a classifier pour le routeur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub text: String,
    pub attachments: Vec<Attachment>,
    pub required_context: u32,
    pub required_capabilities: Capabilities,
}

impl RoutingRequest {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            attachments: Vec::new(),
            required_context: 0,
            required_capabilities: Capabilities::default(),
        }
    }

    pub fn with_attachment(mut self, kind: AttachmentKind) -> Self {
        self.attachments.push(Attachment { kind });
        self
    }

    pub fn with_required_context(mut self, ctx: u32) -> Self {
        self.required_context = ctx;
        self
    }

    pub fn with_required_capabilities(mut self, caps: Capabilities) -> Self {
        self.required_capabilities = caps;
        self
    }

    pub fn lower(&self) -> String {
        self.text.to_lowercase()
    }

    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
}
