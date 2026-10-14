//! `omega-plugin-api` — la **surface d'extension** d'Omega.
//!
//! Volontairement **minuscule** (leçon n°1 du RE) : un plugin = un manifeste + une ou
//! plusieurs contributions. On enregistre dans un [`RegistryBuilder`] (mutable), puis on
//! gele le registre (immuable en usage).

use std::any::Any;
use std::sync::Arc;

use omega_registry::{Manifest, Registry, RegistryBuilder};
use omega_types::{Capability, OmegaError, OmegaResult};

/// Contexte d'installation : configuration effective disponible a l'instantiation.
#[derive(Debug, Clone, Default)]
pub struct InstallContext {
    pub config: serde_json::Map<String, serde_json::Value>,
}

impl InstallContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }
}

/// Une brique qui **contribue** des plugins au registre (elle ne connait que le builder).
pub trait Contributor: Send + Sync {
    fn name(&self) -> &str;
    fn contribute(&self, builder: &mut RegistryBuilder, ctx: &InstallContext) -> OmegaResult<()>;
}

/// Fabrique d'instances (injection de dependances par constructeur).
pub trait PluginFactory: Send + Sync {
    fn manifest(&self) -> Manifest;
    fn instantiate(&self, ctx: &InstallContext) -> OmegaResult<Arc<dyn Any + Send + Sync>>;
}

/// Compose un registre a partir d'une liste de contributeurs, puis le gele.
pub fn compose(contributors: &[&dyn Contributor], ctx: &InstallContext) -> OmegaResult<Registry> {
    let mut builder = RegistryBuilder::new();
    for c in contributors {
        c.contribute(&mut builder, ctx)?;
    }
    Ok(builder.build())
}

/// Chargeur de manifestes (Phase 1 : depuis une chaine JSON).
pub struct PluginLoader;

impl PluginLoader {
    /// Parse une liste de manifestes depuis du JSON.
    ///
    /// Format attendu : `[{"id":"...","capability":"llm-provider","version":"1","priority":0}]`.
    pub fn manifests_from_json(input: &str) -> OmegaResult<Vec<Manifest>> {
        #[derive(serde::Deserialize)]
        struct Raw {
            id: String,
            capability: Capability,
            version: String,
            #[serde(default)]
            priority: i32,
        }

        let raw: Vec<Raw> = serde_json::from_str(input)
            .map_err(|e| OmegaError::Config(format!("manifestes illisibles : {e}")))?;

        Ok(raw
            .into_iter()
            .map(|r| Manifest::new(r.id, r.capability, r.version).with_priority(r.priority))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::Capability;

    #[test]
    fn parses_manifests_from_json() {
        let json = r#"[
            {"id":"ollama","capability":"llm-provider","version":"1","priority":5},
            {"id":"ctx-default","capability":"context-engine","version":"1"}
        ]"#;
        let manifests = PluginLoader::manifests_from_json(json).unwrap();
        assert_eq!(manifests.len(), 2);
        assert_eq!(manifests[0].capability, Capability::LlmProvider);
        assert_eq!(manifests[0].priority, 5);
        assert_eq!(manifests[1].id, "ctx-default");
    }

    #[test]
    fn rejects_bad_json() {
        let err = PluginLoader::manifests_from_json("not json").unwrap_err();
        assert!(matches!(err, OmegaError::Config(_)));
    }

    #[test]
    fn compose_runs_contributors_and_freezes() {
        struct Noop;
        impl Contributor for Noop {
            fn name(&self) -> &str {
                "noop"
            }
            fn contribute(&self, _b: &mut RegistryBuilder, _c: &InstallContext) -> OmegaResult<()> {
                Ok(())
            }
        }
        let registry = compose(&[&Noop], &InstallContext::new()).unwrap();
        assert!(registry.active_ids(Capability::LlmProvider).is_empty());
    }
}
