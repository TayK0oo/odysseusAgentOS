//! `omega-server` — **adaptateur de bord HTTP** pour l'UI Cockpit.
//!
//! Categorie `app-edge` du garde-fou (`OMEGA-BLUEPRINT.md` §2.1) : compose `App`
//! (registre gele + boucle + journal durable) et l'expose en JSON type derriere
//! un schema versionne (`omega-protocol::SCHEMA_VERSION`).
//!
//! Endpoints (contrat detaille dans `docs/omega/workstreams/W7-cockpit.md`) :
//! - `GET /api/health` → `{status, schema_version}` ;
//! - `GET /api/plugins` → plugins actifs par capacite (`id, capability, active`) ;
//! - `POST /api/run` → classify + select + assemble/infer via `App`, retourne
//!   `{run_id, status, decision, summary}` ;
//! - `GET /api/trace/:run_id` → journal rejoue (`ReplayedRun` + `RunSummary`).
//!
//! Aucun LLM ne decide du routage : `HeuristicTaskClassifier` (deterministe) +
//! `ParetoSelector` sur un candidat local (Ollama). L'inference, elle, passe par
//! le `Provider` du registre (Ollama par defaut, cout nul).

mod config;
mod dto;
mod pipeline;
mod routes;
mod state;

pub use config::ServerConfig;
pub use dto::{
    ApiErrorBody, ApiMessage, ApiRunRequest, HealthResponse, PluginEntry, RunResponse,
    TraceResponse,
};
pub use pipeline::{execute_run, local_candidate};
pub use routes::{create_router, create_router_with_ui};
pub use state::{build_app_state, build_app_state_with, AppState};
