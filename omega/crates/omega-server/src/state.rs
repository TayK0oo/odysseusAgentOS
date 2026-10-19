//! Etat partage du serveur : `App` assemble + acces direct au journal.
//!
//! `build_app_state()` cable le socle de production (W10 : 100 % local, cout
//! nul) : `OllamaProvider` (`localhost:11434`), `ContextContributor` et
//! `FileEventStore` (dossier temporaire). Le meme `Arc<FileEventStore>` est
//! enregistre dans le registre (capacite `event-store`) et conserve dans
//! l'etat pour la relecture des traces (`GET /api/trace/:run_id`).

use std::sync::Arc;

use omega_app::{App, AppBuilder, FileEventStore};
use omega_context::ContextContributor;
use omega_plugin_api::{Contributor, InstallContext};
use omega_ports::{EventStore, Provider};
use omega_registry::{Manifest, RegistryBuilder};
use omega_types::{Capability, OmegaResult};

use crate::config::ServerConfig;

/// Etat partage entre les handlers (derriere `Arc`, `Send + Sync`).
#[derive(Clone)]
pub struct AppState {
    /// Application assemblee (registre gele + boucle + journal durable).
    pub app: Arc<App>,
    /// Journal durable (meme instance que celle du registre).
    pub store: Arc<FileEventStore>,
    /// Configuration effective du serveur (port, Ollama, journal, budget, UI).
    pub config: ServerConfig,
}

impl AppState {
    /// Modele local courant (candidat de selection + inference).
    pub fn model(&self) -> &str {
        &self.config.ollama_model
    }
}

/// Contributeur `llm-provider` : expose l'`OllamaProvider` au registre.
struct ProviderContributor {
    provider: Arc<dyn Provider>,
}

impl Contributor for ProviderContributor {
    fn name(&self) -> &str {
        "ollama-provider"
    }

    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        builder
            .register_dyn(
                Manifest::new("ollama", Capability::LlmProvider, "1"),
                self.provider.clone(),
            )
            .map(|_| ())
    }
}

/// Contributeur `event-store` : expose le `FileEventStore` au registre.
struct StoreContributor {
    store: Arc<dyn EventStore>,
}

impl Contributor for StoreContributor {
    fn name(&self) -> &str {
        "file-store"
    }

    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        builder
            .register_dyn(
                Manifest::new("file", Capability::EventStore, "1"),
                self.store.clone(),
            )
            .map(|_| ())
    }
}

/// Construit l'etat de production depuis la config par defaut.
pub fn build_app_state() -> OmegaResult<AppState> {
    build_app_state_with(&ServerConfig::default())
}

/// Construit l'etat depuis une config explicite.
///
/// Erreur typee (`OmegaError`) si une capacite requise est absente — jamais de
/// panique au demarrage : le binaire affiche l'erreur et sort en echec.
pub fn build_app_state_with(cfg: &ServerConfig) -> OmegaResult<AppState> {
    let provider: Arc<dyn Provider> = Arc::new(omega_providers::OllamaProvider::new(
        cfg.ollama_base_url.clone(),
        cfg.ollama_model.clone(),
    ));
    let store = Arc::new(FileEventStore::new(cfg.event_dir.clone()));
    let app = AppBuilder::new()
        .add_contributor(Box::new(ProviderContributor {
            provider: provider.clone(),
        }))
        .add_contributor(Box::new(
            ContextContributor::new().with_max_tokens(cfg.max_tokens),
        ))
        .add_contributor(Box::new(StoreContributor {
            store: store.clone(),
        }))
        // Catalogue de reverse-engineering (données embarquées) comme capacité.
        .add_contributor(Box::new(omega_harness::HarnessCatalogContributor::new()))
        // Pont MCP : REA (`morluto/rea`) branché comme surface d'outils
        // (`Capability::ToolSurface`) — « everything is a plugin », sans toucher
        // au noyau. Les appels live lancent `npx rea-agents --mcp`.
        .add_contributor(Box::new(omega_mcp::McpContributor::rea()))
        .build()?;
    Ok(AppState {
        app: Arc::new(app),
        store,
        config: cfg.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::Capability;

    #[test]
    fn build_wires_expected_plugins() {
        let dir = std::env::temp_dir().join("omega-server-test-state");
        let cfg = ServerConfig {
            event_dir: dir,
            ..ServerConfig::default()
        };
        let state = build_app_state_with(&cfg).unwrap();
        // Les trois capacites cablées sont resolues (pas de NoImplementation).
        assert_eq!(
            state.app.active_ids(Capability::LlmProvider),
            vec!["ollama".to_string()]
        );
        assert_eq!(
            state.app.active_ids(Capability::ContextEngine),
            vec!["default".to_string()]
        );
        assert_eq!(
            state.app.active_ids(Capability::EventStore),
            vec!["file".to_string()]
        );
        // Le pont MCP REA est câblé comme surface d'outils.
        assert_eq!(
            state.app.active_ids(Capability::ToolSurface),
            vec!["mcp:rea".to_string()]
        );
        assert_eq!(state.app.provider().unwrap().id(), "ollama");
        assert_eq!(state.model(), "qwen2.5-coder:7b");
    }
}
