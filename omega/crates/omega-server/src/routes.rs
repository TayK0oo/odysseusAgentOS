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
use tower_http::services::{ServeDir, ServeFile};

use crate::dto::{
    ApiErrorBody, ApiRunRequest, HealthResponse, PluginEntry, RunResponse, TraceResponse,
};
use crate::pipeline::execute_run;
use crate::state::AppState;

/// Toutes les capacites exposees par `GET /api/plugins`, dans un ordre stable
/// (l'ordre de priorite intra-capacite vient d'`App::active_ids`).
pub(crate) const ALL_CAPABILITIES: &[Capability] = &[
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
    Capability::HarnessCatalog,
];

/// Erreur d'API : statut + corps JSON versionne.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ApiErrorBody,
}

impl ApiError {
    pub(crate) fn bad_request(msg: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorBody::new(msg),
        }
    }

    pub(crate) fn from_domain(err: OmegaError) -> Self {
        let status = match &err {
            OmegaError::Io(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            status,
            body: ApiErrorBody::new(err.to_string()),
        }
    }

    /// Statut HTTP (inspection/tests).
    #[cfg(test)]
    pub(crate) fn status(&self) -> StatusCode {
        self.status
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
    api_router(state)
}

/// Routeur API + service statique de l'UI buildee (`omega/ui/dist`).
///
/// Une origine unique : le cockpit React est servi sur `/` et parle a `/api`
/// (proxy inutile). `ui_dir` absent/introuvable → API seule (l'UI de dev reste
/// utilisable via `vite`).
pub fn create_router_with_ui(state: AppState, ui_dir: Option<std::path::PathBuf>) -> Router {
    let router = api_router(state);
    match ui_dir {
        Some(dir) if dir.is_dir() => {
            let index = dir.join("index.html");
            router.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(index)))
        }
        _ => router,
    }
}

fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/plugins", get(plugins))
        .route("/api/run", post(run))
        .route("/api/trace/:run_id", get(trace))
        // W7 inc4 : inspection du système (architecture, modèles, classifieur,
        // runs, politique, contexte).
        .route("/api/system", get(crate::system::system))
        .route("/api/overview", get(crate::system::overview))
        .route("/api/registry", get(crate::system::registry))
        .route("/api/config", get(crate::system::config))
        .route("/api/routes", get(crate::system::routes))
        .route("/api/models", get(crate::system::models))
        .route("/api/classify", post(crate::system::classify))
        .route("/api/runs", get(crate::system::runs))
        .route("/api/runs/:run_id/events", get(crate::system::run_events))
        .route("/api/policy", get(crate::system::policy))
        .route("/api/policy/evaluate", post(crate::system::evaluate))
        .route("/api/context", get(crate::system::context))
        .route("/api/harnesses", get(crate::system::harnesses))
        .route("/api/harnesses/:id", get(crate::system::harness))
        .route("/api/dissections", get(crate::system::dissections))
        .route("/api/dissections/:id", get(crate::system::dissection))
        // W5/W7 : pont MCP REA (outil de reverse-engineering) — instantané
        // embarqué + catalogue live + appel d'outil.
        .route("/api/rea", get(crate::rea::rea))
        .route("/api/rea/tools", get(crate::rea::rea_tools))
        .route("/api/rea/call", post(crate::rea::rea_call))
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
    let replayed = TraceReplayer::replay(&run_id, journal.clone());
    Ok(Json(TraceResponse {
        run_id,
        replayed,
        summary,
        answer: crate::pipeline::last_answer(&journal),
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

    // (P1-2) Snapshot nominal : les 8 adaptateurs portes doivent etre visibles
    // via GET /api/plugins, avec leur capacite exacte (liste, pas un compte).
    // Compare a la liste lue depuis `omega-adapters::composition` lui-meme.
    const ADAPTERS_PORTES: &[(&str, &str)] = &[
        ("agentstore/default", "agent-store"),
        ("config/default", "config-source"),
        ("loop/default", "agent-loop"),
        ("sandbox/default", "sandbox-backend"),
        ("skills/default", "skill-store"),
        ("ui/main", "ui-surface"),
        ("ui/sidebar", "ui-surface"),
        ("workflow/default", "workflow-engine"),
    ];

    #[tokio::test]
    async fn plugins_handler_lists_all_registered_adapters() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let (state, _) = test_state("routes-plugins-p12");
        let app = create_router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/plugins")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let entries: Vec<PluginEntry> =
            serde_json::from_slice(&body).expect("reponse /api/plugins parseable en JSON");
        for (id, capability) in ADAPTERS_PORTES {
            let entry = entries
                .iter()
                .find(|e| e.id == *id)
                .unwrap_or_else(|| panic!("adaptateur manquant de /api/plugins : {id}"));
            assert_eq!(
                entry.capability, *capability,
                "capacite de l'adaptateur {id}"
            );
        }
        // Doublon interdit : la liste nominative est exacte.
        let mut ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "ids dupliques dans /api/plugins");
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

    #[tokio::test]
    async fn ui_dir_is_served_and_api_still_works() {
        // (W7 inc3) UI buildee servie sur `/`, API intacte sur `/api`.
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let dir = std::env::temp_dir().join(format!("omega-ui-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><title>Omega Cockpit</title>",
        )
        .unwrap();

        let (state, _) = test_state("routes-ui");
        let app = create_router_with_ui(state, Some(dir.clone()));

        let res = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Omega Cockpit"));

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn missing_ui_dir_leaves_api_only() {
        // (W7 inc3) `ui_dir` absent -> API seule, pas d'erreur de construction.
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;
        let (state, _) = test_state("routes-noui");
        let app = create_router_with_ui(state, Some("/n-existe-pas-omega".into()));
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
