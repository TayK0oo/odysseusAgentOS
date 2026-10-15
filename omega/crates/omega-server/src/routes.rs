//! Routes Axum de l'adaptateur de bord.
//!
//! Chaque handler est une fonction pure de `(AppState, requete)` : testable
//! sans reseau en l'appelant directement (les tests le font avec un provider
//! bouche). Les codes d'erreur suivent une regle simple :
//! - `400` : requete invalide (validation d'entree) ;
//! - `502` : provider injoignable (`OmegaError::Io`, ex. Ollama eteint) ;
//! - `500` : toute autre erreur interne.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use omega_otel::TraceReplayer;
use omega_ports::EventStore;
use omega_types::{Capability, OmegaError};

use crate::dto::{
    ApiErrorBody, ApiRunRequest, HealthResponse, PluginEntry, RunResponse, TraceResponse,
};
use crate::pipeline::execute_run;
use crate::state::AppState;

/// Toutes les capacites exposees par `GET /api/plugins`, dans un ordre stable
/// (l'ordre de priorite intra-capacite vient d'`App::active_ids`).
const ALL_CAPABILITIES: &[Capability] = &[
    Capability::LlmProvider,
    Capability::CatalogSource,
    Capability::ModelRegistry,
    Capability::TaskClassifier,
    Capability::RouterPolicy,
    Capability::RouterFeedback,
    Capability::ContextEngine,
    Capability::CompactionModel,
    Capability::PromptCachePolicy,
    Capability::SkillStore,
    Capability::MemoryStore,
    Capability::MemorySearch,
    Capability::Tool,
    Capability::ToolSurface,
    Capability::SandboxBackend,
    Capability::ExecPolicy,
    Capability::PermissionPolicy,
    Capability::Hook,
    Capability::SubagentRunner,
    Capability::SchedulerJob,
    Capability::Channel,
    Capability::TelemetrySink,
    Capability::EventStore,
    Capability::ProvenanceSink,
    Capability::UiPanel,
];

/// Erreur d'API : statut + corps JSON versionne.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ApiErrorBody,
}

impl ApiError {
    fn bad_request(msg: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorBody::new(msg),
        }
    }

    fn from_domain(err: OmegaError) -> Self {
        let status = match &err {
            OmegaError::Io(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            status,
            body: ApiErrorBody::new(err.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

/// Construit le routeur Axum depuis un etat explicite.
///
/// Les chemins sont absolus (`/api/...`) : le routeur est montable tel quel
/// (binaire) ou imbrique derriere un prefixe.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/plugins", get(plugins))
        .route("/api/run", post(run))
        .route("/api/trace/:run_id", get(trace))
        .with_state(Arc::new(state))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse::ok())
}

async fn plugins(State(state): State<Arc<AppState>>) -> Json<Vec<PluginEntry>> {
    let mut entries = Vec::new();
    for cap in ALL_CAPABILITIES {
        for id in state.app.active_ids(*cap) {
            entries.push(PluginEntry::active(id, *cap));
        }
    }
    Json(entries)
}

async fn run(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApiRunRequest>,
) -> Result<Json<RunResponse>, ApiError> {
    let messages = req.to_messages().map_err(ApiError::bad_request)?;
    if let Some(id) = &req.run_id {
        if !omega_app::is_valid_run_id(id) {
            return Err(ApiError::bad_request(format!(
                "run_id invalide : {id:?} ([A-Za-z0-9_-]{{1,128}})"
            )));
        }
    }
    let resp = execute_run(&state, messages, req.run_id)
        .await
        .map_err(ApiError::from_domain)?;
    Ok(Json(resp))
}

async fn trace(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<Json<TraceResponse>, ApiError> {
    if !omega_app::is_valid_run_id(&run_id) {
        return Err(ApiError::bad_request(format!(
            "run_id invalide : {run_id:?} ([A-Za-z0-9_-]{{1,128}})"
        )));
    }
    let journal = state
        .store
        .read_run(&run_id)
        .map_err(ApiError::from_domain)?;
    // Run inconnu → journal vide → sequence vide + issue `unknown` (200, pas 404 :
    // l'UI affiche « run inconnu » au lieu d'une erreur).
    let summary = TraceReplayer::summarize(&run_id, journal.clone());
    let replayed = TraceReplayer::replay(&run_id, journal);
    Ok(Json(TraceResponse {
        run_id,
        replayed,
        summary,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::tests::test_state;

    fn state_arg(state: &AppState) -> State<Arc<AppState>> {
        State(Arc::new(state.clone()))
    }

    #[tokio::test]
    async fn health_handler_returns_ok() {
        let body = health().await.0;
        assert_eq!(body.status, "ok");
        assert_eq!(body.schema_version, omega_protocol::SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn plugins_handler_lists_wired_plugins() {
        let (state, _) = test_state("routes-plugins");
        let entries = plugins(state_arg(&state)).await.0;
        assert!(entries
            .iter()
            .any(|e| e.id == "stub" && e.capability == "llm-provider" && e.active));
        assert!(entries
            .iter()
            .any(|e| e.id == "default" && e.capability == "context-engine"));
        assert!(entries
            .iter()
            .any(|e| e.id == "stub-file" && e.capability == "event-store"));
        assert!(entries.iter().all(|e| e.active));
    }

    #[tokio::test]
    async fn run_handler_returns_decision_and_summary_without_network() {
        let (state, _) = test_state("routes-run");
        let req = ApiRunRequest {
            messages: vec![crate::dto::ApiMessage {
                role: "user".to_string(),
                content: "Liste les trois principes d'Omega en bullets.".to_string(),
            }],
            run_id: Some("route-1".to_string()),
        };
        let resp = run(state_arg(&state), Json(req)).await.unwrap().0;
        assert_eq!(resp.run_id, "route-1");
        assert_eq!(resp.status, omega_core::RunStatus::Ok);
        assert!(resp.decision.model_id.starts_with("ollama/"));
        assert_eq!(resp.summary.steps_completed, 2);
    }

    #[tokio::test]
    async fn run_handler_rejects_bad_request() {
        let (state, _) = test_state("routes-run-400");
        let req = ApiRunRequest {
            messages: vec![],
            run_id: None,
        };
        let err = run(state_arg(&state), Json(req)).await.unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn run_and_trace_reject_traversal_run_id() {
        // (traversal) `../` ne doit jamais atteindre le store : 400, pas de journal.
        let (state, _) = test_state("routes-run-traversal");
        let req = ApiRunRequest {
            messages: vec![crate::dto::ApiMessage {
                role: "user".to_string(),
                content: "bonjour".to_string(),
            }],
            run_id: Some("../evil".to_string()),
        };
        let err = run(state_arg(&state), Json(req)).await.unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        let err = trace(state_arg(&state), Path("../evil".to_string()))
            .await
            .unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn trace_handler_replays_run_journal() {
        let (state, _) = test_state("routes-trace");
        // D'abord un run (ecrit le journal), puis relecture via le handler.
        let req = ApiRunRequest {
            messages: vec![crate::dto::ApiMessage {
                role: "user".to_string(),
                content: "Bonjour, resume Omega en une phrase.".to_string(),
            }],
            run_id: Some("route-trace-1".to_string()),
        };
        let created = run(state_arg(&state), Json(req)).await.unwrap().0;
        assert_eq!(created.run_id, "route-trace-1");
        let resp = trace(state_arg(&state), Path("route-trace-1".to_string()))
            .await
            .unwrap()
            .0;
        assert_eq!(resp.run_id, "route-trace-1");
        assert_eq!(resp.replayed.steps.len(), 2);
        assert_eq!(
            resp.replayed.outcome,
            omega_otel::ReplayedOutcome::Completed
        );
        assert_eq!(resp.summary, {
            let journal = state.store.read_run("route-trace-1").unwrap();
            TraceReplayer::summarize("route-trace-1", journal)
        });
    }

    #[tokio::test]
    async fn trace_handler_returns_empty_for_unknown_run() {
        let (state, _) = test_state("routes-trace-unknown");
        let resp = trace(state_arg(&state), Path("nope".to_string()))
            .await
            .unwrap()
            .0;
        assert!(resp.replayed.steps.is_empty());
        assert_eq!(resp.replayed.outcome, omega_otel::ReplayedOutcome::Unknown);
        assert_eq!(resp.summary.total_events, 0);
    }

    #[test]
    fn router_builds_with_all_routes() {
        let (state, _) = test_state("routes-build");
        // Smoke test : le routeur se construit (les handlers sont testes
        // directement ci-dessus, sans reseau).
        let _ = create_router(state);
    }
}
