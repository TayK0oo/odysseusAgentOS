//! `omega-ports` — les **ports** (traits) du noyau Omega.
//!
//! Regle (blueprint §2.2) : ce crate ne depend que de `omega-types`. Il ne contient
//! **aucune implementation** — seulement des contrats. Le noyau ne connait que ces
//! traits ; les adaptateurs vivent ailleurs (`omega-providers`, `plugins/*`...).

use async_trait::async_trait;
use omega_types::{
    ChatRequest, ChatResponse, CommandOutput, DurableEvent, Message, ModelRecord, OmegaResult,
    PermissionEffect, RawModel, RouterDecision, RouterError, RoutingRequest, TaskType,
    TelemetryEvent, ToolSpec, Weights,
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

/// Classifieur de tache. Regime : *slot exclusif*.
pub trait TaskClassifier: Send + Sync {
    /// Determine le type de tache (T1..T14) a partir d'une requete de routage.
    fn classify(&self, req: &RoutingRequest) -> TaskType;
}

/// Moteur de scoring composite. Regime : *slot exclusif* (capacite `router-policy`).
pub trait ScoringPolicy: Send + Sync {
    /// Poids associes a un type de tache.
    fn weights(&self, task: TaskType) -> Weights;

    /// Score composite d'un modele pour une tache donnee.
    /// Les normalisations se font relativement a `candidates`.
    fn score(
        &self,
        task: TaskType,
        required_context: u32,
        candidates: &[ModelRecord],
        model: &ModelRecord,
    ) -> f64;

    /// Qualite effective Q_eff(m) = melange des axes pour la tache.
    fn effective_quality(&self, task: TaskType, model: &ModelRecord) -> f64;
}

/// Selecteur de modele. Regime : *slot exclusif* (capacite `router-policy`).
pub trait Selector: Send + Sync {
    fn select(
        &self,
        task: TaskType,
        required_capabilities: omega_types::Capabilities,
        required_context: u32,
        candidates: &[ModelRecord],
    ) -> Result<RouterDecision, RouterError>;
}

/// Politique de fallback + circuit breaker. Regime : *slot exclusif* (capacite `router-policy`).
pub trait FallbackPolicy: Send + Sync {
    /// Enregistre un echec pour un modele.
    fn record_failure(&mut self, model_id: &str);

    /// Indique si le modele est disponible (circuit breaker ferme).
    fn is_available(&self, model_id: &str) -> bool;

    /// Choisit le prochain modele de fallback pour une decision echouee.
    ///
    /// L'etat circuit-breaker (`record_failure`/`is_available`) persiste sur
    /// l'instance, mais l'ensemble `tried` des modeles deja essayes est
    /// **par run** : l'appelant le fournit et le reinitialise a chaque requete.
    /// L'implementation y ajoute les modeles qu'elle propose pour eviter le
    /// ping-pong (cf. REVIEW-W4 M1/N2).
    fn next_fallback(
        &self,
        task: TaskType,
        required_capabilities: omega_types::Capabilities,
        required_context: u32,
        candidates: &[ModelRecord],
        failed: &RouterDecision,
        tried: &mut std::collections::HashSet<String>,
    ) -> Option<RouterDecision>;
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

/// Journal durable d'evenements. Regime : *slot exclusif* (capacite `event-store`).
///
/// Contrat de reprise (idempotence) :
/// - `append` ecrit durablement un [`DurableEvent`] et retourne le `seq` monotone attribue
///   (par run) ;
/// - `read_run` restitue le journal d'un run dans l'ordre d'append ;
/// - l'identifiant d'operation ([`DurableEvent::operation_id`]) est la cle d'idempotence :
///   une operation est consideree « completee » si le journal contient un evenement
///   `step.completed` portant son `operation_id`.
///
/// Phase 1 : interface synchrone (append local). Un port async avec barriere de flush
/// viendra si un adaptateur distant l'exige (cf. `OMEGA-BLUEPRINT.md` §2.1).
pub trait EventStore: Send + Sync {
    fn name(&self) -> &str;

    /// Append un evenement durable au run `event.run_id` ; retourne le `seq` attribue.
    fn append(&self, event: DurableEvent) -> OmegaResult<u64>;

    /// Lit le journal d'un run, dans l'ordre d'append (par `seq` croissant).
    fn read_run(&self, run_id: &str) -> OmegaResult<Vec<DurableEvent>>;
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
    use omega_types::ModelId;
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

    #[test]
    fn task_classifier_is_object_safe() {
        struct Dummy;
        impl TaskClassifier for Dummy {
            fn classify(&self, _req: &RoutingRequest) -> TaskType {
                TaskType::Chat
            }
        }
        let _c: Arc<dyn TaskClassifier> = Arc::new(Dummy);
    }

    #[test]
    fn scoring_policy_is_object_safe() {
        struct Dummy;
        impl ScoringPolicy for Dummy {
            fn weights(&self, _task: TaskType) -> Weights {
                Weights {
                    reasoning: 1.0,
                    code: 0.0,
                    agentic: 0.0,
                    knowledge: 0.0,
                    chat: 0.0,
                    vision: 0.0,
                    speed: 0.0,
                    cost: 0.0,
                    context: 0.0,
                }
            }
            fn score(
                &self,
                _task: TaskType,
                _required_context: u32,
                _candidates: &[ModelRecord],
                _model: &ModelRecord,
            ) -> f64 {
                0.0
            }
            fn effective_quality(&self, _task: TaskType, _model: &ModelRecord) -> f64 {
                0.0
            }
        }
        let _s: Arc<dyn ScoringPolicy> = Arc::new(Dummy);
    }

    #[test]
    fn selector_is_object_safe() {
        struct Dummy;
        impl Selector for Dummy {
            fn select(
                &self,
                _task: TaskType,
                _required_capabilities: omega_types::Capabilities,
                _required_context: u32,
                _candidates: &[ModelRecord],
            ) -> Result<RouterDecision, RouterError> {
                Ok(RouterDecision::new("m", 0.0, "dummy"))
            }
        }
        let _s: Arc<dyn Selector> = Arc::new(Dummy);
    }

    #[test]
    fn fallback_policy_is_object_safe() {
        struct Dummy;
        impl FallbackPolicy for Dummy {
            fn record_failure(&mut self, _model_id: &str) {}
            fn is_available(&self, _model_id: &str) -> bool {
                true
            }
            fn next_fallback(
                &self,
                _task: TaskType,
                _required_capabilities: omega_types::Capabilities,
                _required_context: u32,
                _candidates: &[ModelRecord],
                _failed: &RouterDecision,
                _tried: &mut std::collections::HashSet<String>,
            ) -> Option<RouterDecision> {
                None
            }
        }
        let _f: Arc<dyn FallbackPolicy> = Arc::new(Dummy);
    }

    #[test]
    fn event_store_is_object_safe() {
        struct Dummy;
        impl EventStore for Dummy {
            fn name(&self) -> &str {
                "dummy"
            }
            fn append(&self, event: DurableEvent) -> OmegaResult<u64> {
                Ok(event.seq)
            }
            fn read_run(&self, _run_id: &str) -> OmegaResult<Vec<DurableEvent>> {
                Ok(Vec::new())
            }
        }
        let _s: Arc<dyn EventStore> = Arc::new(Dummy);
    }
}
