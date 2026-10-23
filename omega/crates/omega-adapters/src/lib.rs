//! `omega-adapters` — adaptateurs par defaut pour les 8 nouveaux ports.
//!
//! Chaque adaptateur est un plugin minimal qui implemente le port correspondant.
//! Ils servent de "legacy" fallback et de base pour les implementations specifiques.

pub mod composition;
pub mod trace;

use async_trait::async_trait;
use omega_ports::*;
use omega_types::OmegaResult;

/// Adaptateur par defaut du registre de plugins.
pub struct DefaultRegistry {
    name: String,
}
impl Default for DefaultRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultRegistry {
    pub fn new() -> Self {
        Self {
            name: "default-registry".into(),
        }
    }
}
impl Registry for DefaultRegistry {
    fn name(&self) -> &str {
        &self.name
    }
    fn register(&self, _manifest: &str) -> OmegaResult<()> {
        Ok(())
    }
    fn activate(&self, _id: &str) -> OmegaResult<()> {
        Ok(())
    }
    fn deactivate(&self, _id: &str) -> OmegaResult<()> {
        Ok(())
    }
    fn list(&self) -> Vec<String> {
        vec![]
    }
}

/// Adaptateur par defaut de la surface UI.
pub struct DefaultUiSurface {
    name: String,
    slot: String,
}
impl DefaultUiSurface {
    pub fn new(slot: &str) -> Self {
        Self {
            name: format!("default-ui-{}", slot),
            slot: slot.into(),
        }
    }
}
impl UiSurface for DefaultUiSurface {
    fn name(&self) -> &str {
        &self.name
    }
    fn slot(&self) -> &str {
        &self.slot
    }
    fn render(&self) -> OmegaResult<String> {
        Ok(String::new())
    }
}

/// Adaptateur par defaut de la source de configuration.
pub struct DefaultConfigSource {
    name: String,
}
impl Default for DefaultConfigSource {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultConfigSource {
    pub fn new() -> Self {
        Self {
            name: "default-config".into(),
        }
    }
}
#[async_trait]
impl ConfigSource for DefaultConfigSource {
    fn name(&self) -> &str {
        &self.name
    }
    async fn load(&self, _layer: &str) -> OmegaResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
    async fn save(&self, _layer: &str, _value: serde_json::Value) -> OmegaResult<()> {
        Ok(())
    }
}

/// Adaptateur par defaut du magasin de skills.
pub struct DefaultSkillStore {
    name: String,
}
impl Default for DefaultSkillStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultSkillStore {
    pub fn new() -> Self {
        Self {
            name: "default-skill-store".into(),
        }
    }
}
#[async_trait]
impl SkillStore for DefaultSkillStore {
    fn name(&self) -> &str {
        &self.name
    }
    async fn discover(&self) -> OmegaResult<Vec<String>> {
        Ok(vec![])
    }
    async fn load(&self, _id: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
}

/// Adaptateur par defaut de la boucle d'agent.
pub struct DefaultAgentLoop {
    name: String,
}
impl Default for DefaultAgentLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultAgentLoop {
    pub fn new() -> Self {
        Self {
            name: "default-agent-loop".into(),
        }
    }
}
#[async_trait]
impl AgentLoop for DefaultAgentLoop {
    fn name(&self) -> &str {
        &self.name
    }
    async fn run(&self, _goal: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
    async fn step(&self) -> OmegaResult<String> {
        Ok(String::new())
    }
    async fn recover(&self, _error: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
}

/// Adaptateur par defaut du magasin d'agents.
pub struct DefaultAgentStore {
    name: String,
}
impl Default for DefaultAgentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultAgentStore {
    pub fn new() -> Self {
        Self {
            name: "default-agent-store".into(),
        }
    }
}
#[async_trait]
impl AgentStore for DefaultAgentStore {
    fn name(&self) -> &str {
        &self.name
    }
    async fn discover(&self) -> OmegaResult<Vec<String>> {
        Ok(vec![])
    }
    async fn load(&self, _id: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
    async fn save(&self, _id: &str, _spec: &str) -> OmegaResult<()> {
        Ok(())
    }
}

/// Adaptateur par defaut du moteur de workflow.
pub struct DefaultWorkflowEngine {
    name: String,
}
impl Default for DefaultWorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultWorkflowEngine {
    pub fn new() -> Self {
        Self {
            name: "default-workflow-engine".into(),
        }
    }
}
#[async_trait]
impl WorkflowEngine for DefaultWorkflowEngine {
    fn name(&self) -> &str {
        &self.name
    }
    async fn build(&self, _spec: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
    async fn compile(&self, _id: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
    async fn run(&self, _id: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
}

/// Adaptateur par defaut du backend sandbox v2.
pub struct DefaultSandboxBackendV2 {
    name: String,
}
impl Default for DefaultSandboxBackendV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultSandboxBackendV2 {
    pub fn new() -> Self {
        Self {
            name: "default-sandbox-v2".into(),
        }
    }
}
#[async_trait]
impl SandboxBackendV2 for DefaultSandboxBackendV2 {
    fn name(&self) -> &str {
        &self.name
    }
    async fn isolate(&self, _command: &str) -> OmegaResult<String> {
        Ok(String::new())
    }
    async fn merge(&self, _id: &str) -> OmegaResult<()> {
        Ok(())
    }
    async fn cleanup(&self, _id: &str) -> OmegaResult<()> {
        Ok(())
    }
}

// --- Compaction wiring tests (T4) --------------------------------------------

#[cfg(test)]
mod compaction_wired_tests {
    use super::*;
    use omega_context::{ContextContributor, PinnedFacts};
    use omega_plugin_api::{compose, InstallContext};
    use omega_ports::{ContextEngine, Provider};
    use omega_types::{Capability, ChatRequest, ChatResponse, Message, OmegaResult, Role};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    /// Provider stub qui echoue (pour tester le fallback).
    struct FailProvider;
    #[async_trait]
    impl Provider for FailProvider {
        fn id(&self) -> &str {
            "fail"
        }
        async fn complete(&self, _: ChatRequest) -> OmegaResult<ChatResponse> {
            Err(omega_types::OmegaError::NoImplementation(
                Capability::LlmProvider,
            ))
        }
    }

    /// Provider stub qui renvoie un resume fixe.
    struct EchoProvider {
        calls: AtomicUsize,
    }
    impl EchoProvider {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }
    #[async_trait]
    impl Provider for EchoProvider {
        fn id(&self) -> &str {
            "echo"
        }
        async fn complete(&self, req: ChatRequest) -> OmegaResult<ChatResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let last = req
                .messages
                .last()
                .map(|m| m.content.clone())
                .unwrap_or_default();
            Ok(ChatResponse {
                message: Message::new(Role::Assistant, format!("SUMMARY: {}", last)),
                usage: Default::default(),
            })
        }
    }

    #[tokio::test]
    async fn compaction_wired_llm_compaction_engine_active_when_provider_given() {
        // ContextContributor avec provider -> LlmCompactionEngine (owns_compaction = true)
        let provider = Arc::new(EchoProvider::new());
        let contributor = ContextContributor::new()
            .with_max_tokens(20)
            .with_llm_compaction(provider.clone());

        let registry = compose(&[&contributor], &InstallContext::new()).unwrap();
        let engine = registry
            .resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine)
            .expect("context-engine absent");

        assert_eq!(engine.name(), "llm-compaction");
        assert!(
            engine.owns_compaction(),
            "LlmCompactionEngine doit declarer owns_compaction=true"
        );

        // Test assemble: doit ajouter le resume LLM
        let msgs = vec![
            Message::new(Role::System, "SYS"),
            Message::new(Role::User, "a".repeat(40)),
            Message::new(Role::User, "b".repeat(40)),
        ];
        let out = engine.assemble(msgs).await.unwrap();
        assert!(
            out.iter()
                .any(|m| m.content.starts_with("Summary: SUMMARY:")),
            "doit contenir le resume LLM"
        );
    }

    #[tokio::test]
    async fn compaction_wired_falls_back_to_truncate_when_provider_fails() {
        // Provider qui echoue -> fallback sur truncation simple (owns_compaction = false via inner)
        let provider = Arc::new(FailProvider);
        let contributor = ContextContributor::new()
            .with_max_tokens(20)
            .with_llm_compaction(provider);

        let registry = compose(&[&contributor], &InstallContext::new()).unwrap();
        let engine = registry
            .resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine)
            .expect("context-engine absent");

        assert_eq!(engine.name(), "llm-compaction");
        assert!(engine.owns_compaction());

        let msgs = vec![
            Message::new(Role::System, "SYS"),
            Message::new(Role::User, "a".repeat(40)),
            Message::new(Role::User, "b".repeat(40)),
        ];
        let out = engine.assemble(msgs).await.unwrap();
        // Fallback: marqueur de truncation simple, pas de resume LLM
        assert!(
            out.iter().any(|m| m.content.starts_with("…[compacted")),
            "fallback: doit contenir le marqueur de truncation"
        );
        assert!(
            !out.iter().any(|m| m.content.starts_with("Summary:")),
            "fallback: pas de resume LLM"
        );
    }

    #[test]
    fn compaction_wired_pinned_facts_registered_and_resolvable() {
        // ContextContributor avec PinnedFacts -> capacite pinned-facts resolvable
        let facts = PinnedFacts::new();
        facts.upsert(
            omega_context::PinnedFact::new("f1", "User prefers Rust", "preference")
                .with_priority(10),
        );

        let contributor = ContextContributor::new()
            .with_max_tokens(100)
            .with_pinned_facts(facts);

        let registry = compose(&[&contributor], &InstallContext::new()).unwrap();
        let pf = registry
            .resolve::<PinnedFacts>(Capability::PinnedFacts)
            .expect("pinned-facts absent");

        assert_eq!(pf.len(), 1);
        let got = pf.get("f1").unwrap();
        assert_eq!(got.content, "User prefers Rust");
        assert_eq!(got.priority, 10);
    }

    #[test]
    fn compaction_wired_pinned_facts_survives_through_registry() {
        // Les faits sont accessibles via le registre et conservent leur ordre de priorite
        let facts = PinnedFacts::new();
        facts.upsert(omega_context::PinnedFact::new("low", "low prio", "cat").with_priority(0));
        facts.upsert(omega_context::PinnedFact::new("high", "high prio", "cat").with_priority(10));

        let contributor = ContextContributor::new()
            .with_max_tokens(100)
            .with_pinned_facts(facts);

        let registry = compose(&[&contributor], &InstallContext::new()).unwrap();
        let pf = registry
            .resolve::<PinnedFacts>(Capability::PinnedFacts)
            .unwrap();

        let list = pf.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "high"); // priorite 10 en premier
        assert_eq!(list[1].id, "low");
    }

    #[tokio::test]
    async fn compaction_wired_default_engine_without_provider_has_owns_compaction_false() {
        // Sans provider -> DefaultContextEngine (owns_compaction = false)
        let contributor = ContextContributor::new().with_max_tokens(100);

        let registry = compose(&[&contributor], &InstallContext::new()).unwrap();
        let engine = registry
            .resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine)
            .unwrap();

        assert_eq!(engine.name(), "default");
        assert!(!engine.owns_compaction());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn registry_is_object_safe() {
        let _r: Arc<dyn Registry> = Arc::new(DefaultRegistry::new());
    }

    #[test]
    fn ui_surface_is_object_safe() {
        let _u: Arc<dyn UiSurface> = Arc::new(DefaultUiSurface::new("main"));
    }

    #[tokio::test]
    async fn config_source_is_object_safe() {
        let c: Arc<dyn ConfigSource> = Arc::new(DefaultConfigSource::new());
        assert_eq!(c.name(), "default-config");
    }

    #[tokio::test]
    async fn skill_store_is_object_safe() {
        let s: Arc<dyn SkillStore> = Arc::new(DefaultSkillStore::new());
        assert_eq!(s.name(), "default-skill-store");
    }

    #[tokio::test]
    async fn agent_loop_is_object_safe() {
        let a: Arc<dyn AgentLoop> = Arc::new(DefaultAgentLoop::new());
        assert_eq!(a.name(), "default-agent-loop");
    }

    #[tokio::test]
    async fn agent_store_is_object_safe() {
        let a: Arc<dyn AgentStore> = Arc::new(DefaultAgentStore::new());
        assert_eq!(a.name(), "default-agent-store");
    }

    #[tokio::test]
    async fn workflow_engine_is_object_safe() {
        let w: Arc<dyn WorkflowEngine> = Arc::new(DefaultWorkflowEngine::new());
        assert_eq!(w.name(), "default-workflow-engine");
    }

    #[tokio::test]
    async fn sandbox_backend_v2_is_object_safe() {
        let s: Arc<dyn SandboxBackendV2> = Arc::new(DefaultSandboxBackendV2::new());
        assert_eq!(s.name(), "default-sandbox-v2");
    }
}

// Re-export retiré : omega-adapters ne peut pas dépendre de omega-memory
// (les deux sont "adapters" — règle ARCH_RULES). Les tests A1 utilisent
// omega_memory directement.
