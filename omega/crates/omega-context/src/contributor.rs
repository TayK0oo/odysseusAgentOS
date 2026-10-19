//! Câblage : [`ContextContributor`] fournit le moteur et (optionnellement) les index
//! via le registre (`register_dyn` pour le port, `register_typed` pour les index).

use std::sync::Arc;

use omega_plugin_api::{Contributor, InstallContext};
use omega_ports::ContextEngine;
use omega_registry::{Manifest, RegistryBuilder};
use omega_types::{Capability, OmegaResult};

use crate::engine::DefaultContextEngine;
use crate::skills::SkillIndex;
use crate::tool_surface::ToolSurfaceIndex;

/// Budget de contexte par defaut (tokens) si la config n'en fournit pas.
pub const DEFAULT_MAX_TOKENS: usize = 4000;

/// Contribue `DefaultContextEngine` (slot `context-engine`) et, si fournis, un
/// `ToolSurfaceIndex` (capacite `tool-surface`) et un `SkillIndex` (`skill-store`).
///
/// Enregistrable dans `AppBuilder` : `add_contributor(Box::new(ContextContributor::new()))`.
/// Le moteur est enregistre en `Arc<dyn ContextEngine>` (`register_dyn`) pour etre
/// resolvable par `App::context_engine()` (contrat W3, `REVIEW-W3` M3).
pub struct ContextContributor {
    max_tokens: usize,
    tool_index: Option<Arc<ToolSurfaceIndex>>,
    skill_index: Option<Arc<SkillIndex>>,
}

impl Default for ContextContributor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextContributor {
    pub fn new() -> Self {
        Self {
            max_tokens: DEFAULT_MAX_TOKENS,
            tool_index: None,
            skill_index: None,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_tool_index(mut self, index: ToolSurfaceIndex) -> Self {
        self.tool_index = Some(Arc::new(index));
        self
    }

    pub fn with_skill_index(mut self, index: SkillIndex) -> Self {
        self.skill_index = Some(Arc::new(index));
        self
    }

    /// Budget effectif : `context.max_tokens` de la config, sinon le defaut du contributeur.
    fn resolve_max_tokens(&self, ctx: &InstallContext) -> usize {
        ctx.config
            .get("context.max_tokens")
            .and_then(serde_json::Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(self.max_tokens)
    }
}

impl Contributor for ContextContributor {
    fn name(&self) -> &str {
        "omega-context"
    }

    fn contribute(&self, builder: &mut RegistryBuilder, ctx: &InstallContext) -> OmegaResult<()> {
        let engine: Arc<dyn ContextEngine> =
            Arc::new(DefaultContextEngine::new(self.resolve_max_tokens(ctx)));
        builder.register_dyn(
            Manifest::new("default", Capability::ContextEngine, "1"),
            engine,
        )?;

        if let Some(index) = &self.tool_index {
            builder.register_typed(
                Manifest::new("bm25", Capability::ToolSurface, "1"),
                index.clone(),
            )?;
        }
        if let Some(index) = &self.skill_index {
            builder.register_typed(
                Manifest::new("progressive", Capability::SkillStore, "1"),
                index.clone(),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_plugin_api::compose;
    use omega_types::Capability;

    fn sample_tools() -> ToolSurfaceIndex {
        ToolSurfaceIndex::from_docs(vec![
            crate::ToolDoc::new("shell", "execute shell command"),
            crate::ToolDoc::new("file_read", "read a file"),
        ])
    }

    fn sample_skills() -> SkillIndex {
        SkillIndex::from_skills(vec![crate::Skill::new(
            "greet",
            "Greet",
            "salue",
            "body: bonjour",
        )])
    }

    #[test]
    fn contributor_wires_engine_and_indexes() {
        let contributor = ContextContributor::new()
            .with_tool_index(sample_tools())
            .with_skill_index(sample_skills());

        let registry = compose(&[&contributor], &InstallContext::new()).unwrap();

        let engine = registry
            .resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine)
            .unwrap();
        assert_eq!(engine.name(), "default");
        assert!(!engine.owns_compaction());

        let tools = registry
            .resolve::<ToolSurfaceIndex>(Capability::ToolSurface)
            .unwrap();
        assert_eq!(tools.search("shell", 5).len(), 1);

        let skills = registry
            .resolve::<SkillIndex>(Capability::SkillStore)
            .unwrap();
        assert_eq!(skills.list().len(), 1);
    }

    #[test]
    fn contributor_reads_budget_from_config() {
        // Config fournie -> budget lu depuis `context.max_tokens`.
        let contributor = ContextContributor::new().with_max_tokens(100);
        let ctx = InstallContext::new().with("context.max_tokens", serde_json::json!(1234));
        assert_eq!(contributor.resolve_max_tokens(&ctx), 1234);

        // Sans config -> defaut du contributeur.
        let plain = ContextContributor::new().with_max_tokens(77);
        assert_eq!(plain.resolve_max_tokens(&InstallContext::new()), 77);
    }
}
