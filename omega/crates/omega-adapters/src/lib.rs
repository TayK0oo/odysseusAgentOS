//! `omega-adapters` — adaptateurs par defaut pour les 8 nouveaux ports.
//!
//! Chaque adaptateur est un plugin minimal qui implemente le port correspondant.
//! Ils servent de "legacy" fallback et de base pour les implementations specifiques.

pub mod composition;

use async_trait::async_trait;
use omega_ports::*;
use omega_types::OmegaResult;

/// Adaptateur par defaut du registre de plugins.
pub struct DefaultRegistry { name: String }
impl DefaultRegistry {
    pub fn new() -> Self { Self { name: "default-registry".into() } }
}
impl Registry for DefaultRegistry {
    fn name(&self) -> &str { &self.name }
    fn register(&self, _manifest: &str) -> OmegaResult<()> { Ok(()) }
    fn activate(&self, _id: &str) -> OmegaResult<()> { Ok(()) }
    fn deactivate(&self, _id: &str) -> OmegaResult<()> { Ok(()) }
    fn list(&self) -> Vec<String> { vec![] }
}

/// Adaptateur par defaut de la surface UI.
pub struct DefaultUiSurface { name: String, slot: String }
impl DefaultUiSurface {
    pub fn new(slot: &str) -> Self { Self { name: format!("default-ui-{}", slot), slot: slot.into() } }
}
impl UiSurface for DefaultUiSurface {
    fn name(&self) -> &str { &self.name }
    fn slot(&self) -> &str { &self.slot }
    fn render(&self) -> OmegaResult<String> { Ok(String::new()) }
}

/// Adaptateur par defaut de la source de configuration.
pub struct DefaultConfigSource { name: String }
impl DefaultConfigSource {
    pub fn new() -> Self { Self { name: "default-config".into() } }
}
#[async_trait]
impl ConfigSource for DefaultConfigSource {
    fn name(&self) -> &str { &self.name }
    async fn load(&self, _layer: &str) -> OmegaResult<serde_json::Value> { Ok(serde_json::json!({})) }
    async fn save(&self, _layer: &str, _value: serde_json::Value) -> OmegaResult<()> { Ok(()) }
}

/// Adaptateur par defaut du magasin de skills.
pub struct DefaultSkillStore { name: String }
impl DefaultSkillStore {
    pub fn new() -> Self { Self { name: "default-skill-store".into() } }
}
#[async_trait]
impl SkillStore for DefaultSkillStore {
    fn name(&self) -> &str { &self.name }
    async fn discover(&self) -> OmegaResult<Vec<String>> { Ok(vec![]) }
    async fn load(&self, _id: &str) -> OmegaResult<String> { Ok(String::new()) }
}

/// Adaptateur par defaut de la boucle d'agent.
pub struct DefaultAgentLoop { name: String }
impl DefaultAgentLoop {
    pub fn new() -> Self { Self { name: "default-agent-loop".into() } }
}
#[async_trait]
impl AgentLoop for DefaultAgentLoop {
    fn name(&self) -> &str { &self.name }
    async fn run(&self, _goal: &str) -> OmegaResult<String> { Ok(String::new()) }
    async fn step(&self) -> OmegaResult<String> { Ok(String::new()) }
    async fn recover(&self, _error: &str) -> OmegaResult<String> { Ok(String::new()) }
}

/// Adaptateur par defaut du magasin d'agents.
pub struct DefaultAgentStore { name: String }
impl DefaultAgentStore {
    pub fn new() -> Self { Self { name: "default-agent-store".into() } }
}
#[async_trait]
impl AgentStore for DefaultAgentStore {
    fn name(&self) -> &str { &self.name }
    async fn discover(&self) -> OmegaResult<Vec<String>> { Ok(vec![]) }
    async fn load(&self, _id: &str) -> OmegaResult<String> { Ok(String::new()) }
    async fn save(&self, _id: &str, _spec: &str) -> OmegaResult<()> { Ok(()) }
}

/// Adaptateur par defaut du moteur de workflow.
pub struct DefaultWorkflowEngine { name: String }
impl DefaultWorkflowEngine {
    pub fn new() -> Self { Self { name: "default-workflow-engine".into() } }
}
#[async_trait]
impl WorkflowEngine for DefaultWorkflowEngine {
    fn name(&self) -> &str { &self.name }
    async fn build(&self, _spec: &str) -> OmegaResult<String> { Ok(String::new()) }
    async fn compile(&self, _id: &str) -> OmegaResult<String> { Ok(String::new()) }
    async fn run(&self, _id: &str) -> OmegaResult<String> { Ok(String::new()) }
}

/// Adaptateur par defaut du backend sandbox v2.
pub struct DefaultSandboxBackendV2 { name: String }
impl DefaultSandboxBackendV2 {
    pub fn new() -> Self { Self { name: "default-sandbox-v2".into() } }
}
#[async_trait]
impl SandboxBackendV2 for DefaultSandboxBackendV2 {
    fn name(&self) -> &str { &self.name }
    async fn isolate(&self, _command: &str) -> OmegaResult<String> { Ok(String::new()) }
    async fn merge(&self, _id: &str) -> OmegaResult<()> { Ok(()) }
    async fn cleanup(&self, _id: &str) -> OmegaResult<()> { Ok(()) }
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
