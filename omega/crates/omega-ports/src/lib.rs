//! `omega-ports` — les **ports** (traits) du noyau Omega.
//!
//! Regle (blueprint §2.2) : ce crate ne depend que de `omega-types`. Il ne contient
//! **aucune implementation** — seulement des contrats. Le noyau ne connait que ces
//! traits ; les adaptateurs vivent ailleurs (`omega-providers`, `plugins/*`...).

use async_trait::async_trait;
use omega_types::{
    ChatRequest, ChatResponse, CommandOutput, Message, ModelId, OmegaResult, PermissionEffect,
    RawModel, RoutingRequest, TaskType, TelemetryEvent, ToolSpec,
};

/// Fournit de l'inference. Regime : *ensemble*.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Identifiant stable du provider (ex. `anthropic`, `ollama`).
    fn id(&self) -> &str;
    /// Completion (forme par defaut ; le streaming sera un port dedie en Phase 2).
    async fn complete(&self, req: ChatRequest) -> OmegaResult<ChatResponse>;
}

/// Assemble / compresse le contexte. Regime : *slot exclusif*.
#[async_trait]
pub trait ContextEngine: Send + Sync {
    fn name(&self) -> &str;
    /// Si vrai, ce moteur gere lui-meme la compaction (`ownsCompaction`).
    fn owns_compaction(&self) -> bool;
    /// Assemble le contexte effectif a envoyer au modele.
    async fn assemble(&self, messages: Vec<Message>) -> OmegaResult<Vec<Message>>;
}

/// Memoire persistante. Regime : *slot exclusif*.
#[async_trait]
pub trait MemoryStore: Send + Sync {
    fn name(&self) -> &str;
    async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>>;
    async fn remember(&self, fact: &str) -> OmegaResult<()>;
}

/// Un outil executable. Regime : *ensemble*.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    fn name(&self) -> &str;
    fn spec(&self) -> ToolSpec;
    async fn execute(&self, args: serde_json::Value) -> OmegaResult<serde_json::Value>;
}

/// Backend d'execution isolee. Regime : *ensemble*.
#[async_trait]
pub trait SandboxBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, command: &str) -> OmegaResult<CommandOutput>;
}

/// Politique de selection de modele. Regime : *slot exclusif*.
#[async_trait]
pub trait RouterPolicy: Send + Sync {
    fn name(&self) -> &str;
    /// Choisit un modele parmi les candidats pour un type de tache donne.
    async fn choose(&self, task: TaskType, candidates: &[ModelId]) -> OmegaResult<ModelId>;
}

/// Classifieur de tache. Regime : *slot exclusif*.
pub trait TaskClassifier: Send + Sync {
    /// Determine le type de tache (T1..T14) a partir d'une requete de routage.
    fn classify(&self, req: &RoutingRequest) -> TaskType;
}

/// Source de catalogue de modeles. Regime : *ensemble*.
#[async_trait]
pub trait CatalogSource: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch(&self) -> OmegaResult<Vec<RawModel>>;
}

/// Puits de telemetrie. Regime : *ensemble*.
pub trait TelemetrySink: Send + Sync {
    fn name(&self) -> &str;
    fn emit(&self, event: TelemetryEvent);
}

/// Hook de cycle de vie. Regime : *ensemble*.
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
}

/// Politique de permission (policy par code). Regime : *slot exclusif*.
pub trait PermissionPolicy: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, action: &str, resource: &str) -> PermissionEffect;
}

/// Executeur de sous-agents. Regime : *ensemble*.
#[async_trait]
pub trait SubagentRunner: Send + Sync {
    fn name(&self) -> &str;
    async fn spawn(&self, spec: &str) -> OmegaResult<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct EchoProvider;

    #[async_trait]
    impl Provider for EchoProvider {
        fn id(&self) -> &str {
            "echo"
        }
        async fn complete(&self, req: ChatRequest) -> OmegaResult<ChatResponse> {
            let last = req
                .messages
                .last()
                .map(|m| m.content.clone())
                .unwrap_or_default();
            Ok(ChatResponse {
                message: Message::new(omega_types::Role::Assistant, last),
                usage: Default::default(),
            })
        }
    }

    #[test]
    fn provider_is_object_safe() {
        // (e) object-safety : un port doit etre utilisable derriere `dyn`.
        let p: Arc<dyn Provider> = Arc::new(EchoProvider);
        assert_eq!(p.id(), "echo");
    }

    #[tokio::test]
    async fn provider_can_be_called_via_dyn() {
        let p: Arc<dyn Provider> = Arc::new(EchoProvider);
        let req = ChatRequest {
            model: ModelId::new("m"),
            messages: vec![Message::new(omega_types::Role::User, "hi")],
        };
        let resp = p.complete(req).await.unwrap();
        assert_eq!(resp.message.content, "hi");
    }
}
