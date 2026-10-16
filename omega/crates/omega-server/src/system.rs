//! Endpoints d'inspection du **système** (W7 inc4) : architecture, capacités,
//! catalogue de modèles, classifieur de tâche, journal des runs, politique de
//! sandbox, contexte.
//!
//! Règle : tout est **déterministe** (aucun LLM) sauf `/api/models` (lecture du
//! catalogue local Ollama) et `/api/run` (inférence, dans `pipeline`).

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use omega_ports::{CatalogSource, EventStore, ScoringPolicy};
use omega_providers::{local_record, OllamaCatalogSource};
use omega_router::{CompositeScoringPolicy, HeuristicTaskClassifier, TaskClassifier};
use omega_types::RoutingRequest;
use serde::{Deserialize, Serialize};

use crate::config::REQUIRED_CONTEXT;
use crate::routes::ApiError;
use crate::state::AppState;

/// Version du schéma wire (alignée sur `omega-protocol`).
pub use omega_protocol::SCHEMA_VERSION;

/* --- Architecture ---------------------------------------------------------- */

/// Un crate du workspace (miroir de `OMEGA-BLUEPRINT.md` §2.1, vérifié par
/// `checks/check_dep_graph.py`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub layer: u8,
    pub role: String,
    pub deps: Vec<String>,
}

fn crate_table() -> Vec<CrateInfo> {
    let c = |name: &str, layer: u8, role: &str, deps: &[&str]| CrateInfo {
        name: name.to_string(),
        layer,
        role: role.to_string(),
        deps: deps.iter().map(|d| (*d).to_string()).collect(),
    };
    vec![
        c(
            "omega-types",
            0,
            "Types purs (domaine, erreurs) — feuille",
            &[],
        ),
        c(
            "omega-ports",
            1,
            "Traits/ports (aucune implémentation)",
            &["omega-types"],
        ),
        c(
            "omega-config",
            1,
            "Config à couches + precedence",
            &["omega-types"],
        ),
        c(
            "omega-protocol",
            1,
            "Wire types versionnés (frontière UI)",
            &["omega-types"],
        ),
        c(
            "omega-core",
            2,
            "Boucle d'agent, event-bus, annulation",
            &["omega-types", "omega-ports"],
        ),
        c(
            "omega-registry",
            2,
            "Registre de capacités (slots, quarantaine)",
            &["omega-types", "omega-ports", "omega-config"],
        ),
        c(
            "omega-plugin-api",
            2,
            "Contributeurs, manifestes, installation",
            &["omega-types", "omega-registry"],
        ),
        c(
            "omega-context",
            3,
            "Context Engine, budget, tool search, skills",
            &[
                "omega-types",
                "omega-ports",
                "omega-registry",
                "omega-plugin-api",
            ],
        ),
        c(
            "omega-providers",
            3,
            "Adaptateurs (Ollama) + catalogue",
            &["omega-types", "omega-ports"],
        ),
        c(
            "omega-sandbox",
            3,
            "Politique d'exécution, backend local, approbations",
            &[
                "omega-types",
                "omega-ports",
                "omega-registry",
                "omega-plugin-api",
            ],
        ),
        c(
            "omega-otel",
            3,
            "Télémétrie, provenance, replay",
            &[
                "omega-types",
                "omega-ports",
                "omega-registry",
                "omega-plugin-api",
            ],
        ),
        c(
            "omega-router",
            4,
            "Classifieur, scoring composite, sélecteur Pareto",
            &["omega-types", "omega-ports", "omega-providers"],
        ),
        c(
            "omega-app",
            5,
            "Assemblage, exécution durable (journal + reprise)",
            &[
                "omega-types",
                "omega-ports",
                "omega-config",
                "omega-registry",
                "omega-plugin-api",
                "omega-core",
            ],
        ),
        c(
            "omega-server",
            6,
            "Adaptateur de bord HTTP + UI",
            &["omega-app", "omega-protocol"],
        ),
    ]
}

/// Capacité + régime + implémentations actives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub name: String,
    pub regime: String,
    pub active: Vec<String>,
}

/// Info Ollama (URL + modèle courant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaInfo {
    pub base_url: String,
    pub model: String,
}

/// Compteurs du système.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCounts {
    pub crates: usize,
    pub capabilities: usize,
    pub capabilities_wired: usize,
}

/// `GET /api/system` : l'architecture complète + les capacités câblées.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResponse {
    pub schema_version: u32,
    pub crates: Vec<CrateInfo>,
    pub capabilities: Vec<CapabilityInfo>,
    pub counts: SystemCounts,
    pub ollama: OllamaInfo,
}

pub async fn system(State(state): State<Arc<AppState>>) -> Json<SystemResponse> {
    let capabilities: Vec<CapabilityInfo> = crate::routes::ALL_CAPABILITIES
        .iter()
        .map(|cap| CapabilityInfo {
            name: cap.as_str().to_string(),
            regime: match cap.regime() {
                omega_types::Regime::Slot => "slot".to_string(),
                omega_types::Regime::Ensemble => "ensemble".to_string(),
            },
            active: state.app.active_ids(*cap),
        })
        .collect();
    let wired = capabilities.iter().filter(|c| !c.active.is_empty()).count();
    Json(SystemResponse {
        schema_version: SCHEMA_VERSION,
        counts: SystemCounts {
            crates: 14,
            capabilities: capabilities.len(),
            capabilities_wired: wired,
        },
        crates: crate_table(),
        capabilities,
        ollama: OllamaInfo {
            base_url: state.config.ollama_base_url.clone(),
            model: state.model().to_string(),
        },
    })
}

/* --- Modèles (catalogue local) --------------------------------------------- */

/// `GET /api/models` : catalogue Ollama + record canonique local.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub schema_version: u32,
    /// Modèles tirés localement (`GET /api/tags`).
    pub installed: Vec<omega_types::RawModel>,
    /// Record de sélection du modèle courant (coût nul, tier self-hosted).
    pub record: omega_types::ModelRecord,
}

pub async fn models(State(state): State<Arc<AppState>>) -> Result<Json<ModelsResponse>, ApiError> {
    let source = OllamaCatalogSource::new(state.config.ollama_base_url.clone());
    let installed = source.fetch().await.map_err(ApiError::from_domain)?;
    Ok(Json(ModelsResponse {
        schema_version: SCHEMA_VERSION,
        installed,
        record: local_record(state.model(), Some(32768)),
    }))
}

/* --- Classifieur de tâche --------------------------------------------------- */

/// Corps de `POST /api/classify`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyRequest {
    pub text: String,
}

/// Résultat : type de tâche + poids du profil (déterministe, zéro LLM).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyResponse {
    pub schema_version: u32,
    pub task: omega_types::TaskType,
    pub task_code: String,
    pub weights: omega_types::Weights,
}

pub async fn classify(
    Json(req): Json<ClassifyRequest>,
) -> Result<Json<ClassifyResponse>, ApiError> {
    if req.text.trim().is_empty() {
        return Err(ApiError::bad_request("texte vide".to_string()));
    }
    let task = HeuristicTaskClassifier::new().classify(&RoutingRequest::new(&req.text));
    let weights =
        <CompositeScoringPolicy as ScoringPolicy>::weights(&CompositeScoringPolicy::new(), task);
    Ok(Json(ClassifyResponse {
        schema_version: SCHEMA_VERSION,
        task,
        task_code: task.code().to_string(),
        weights,
    }))
}

/* --- Journal des runs ------------------------------------------------------- */

/// `GET /api/runs` : liste les runs du journal (du plus récent au plus ancien).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunsResponse {
    pub schema_version: u32,
    pub runs: Vec<RunEntry>,
}

/// Entrée de journal de runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEntry {
    pub run_id: String,
    pub events: usize,
    pub steps_completed: usize,
    pub steps_failed: usize,
    pub outcome: omega_otel::ReplayedOutcome,
}

/// Nombre maximum de runs listés.
const RUNS_LIMIT: usize = 50;

pub async fn runs(State(state): State<Arc<AppState>>) -> Json<RunsResponse> {
    let dir = state.store.dir();
    let mut entries: Vec<(std::time::SystemTime, String)> = Vec::new();
    if let Ok(read) = std::fs::read_dir(dir) {
        for e in read.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("jsonl") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let modified = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            entries.push((modified, stem.to_string()));
        }
    }
    entries.sort_by_key(|e| std::cmp::Reverse(e.0));
    entries.truncate(RUNS_LIMIT);

    let runs = entries
        .into_iter()
        .filter_map(|(_, run_id)| {
            let journal = state.store.read_run(&run_id).ok()?;
            let s = omega_otel::TraceReplayer::summarize(&run_id, journal);
            Some(RunEntry {
                run_id,
                events: s.total_events,
                steps_completed: s.steps_completed,
                steps_failed: s.steps_failed,
                outcome: s.outcome,
            })
        })
        .collect();

    Json(RunsResponse {
        schema_version: SCHEMA_VERSION,
        runs,
    })
}

/* --- Politique de sandbox --------------------------------------------------- */

/// Règle exposée par `GET /api/policy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub prefix: String,
    /// `allow` | `prompt` | `forbidden`.
    pub decision: String,
    pub reason: Option<String>,
}

/// `GET /api/policy` : politique de référence du module `omega-sandbox`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub schema_version: u32,
    pub strict: bool,
    pub rules: Vec<PolicyRule>,
    pub default: String,
    /// Note d'honnêteté : ce serveur n'exécute aucune commande.
    pub note: String,
}

/// Politique de référence (deny-by-default + règles courantes).
///
/// Ce n'est **pas** ce que le serveur exécute (il n'exécute rien) : c'est la
/// politique du module `omega-sandbox`, exposée pour inspection/évaluation.
pub fn reference_policy() -> omega_sandbox::RuleBasedExecPolicy {
    use omega_types::ExecDecision;
    omega_sandbox::RuleBasedExecPolicy::new("omega-reference")
        .rule("echo", ExecDecision::Allow)
        .rule("ls", ExecDecision::Allow)
        .rule("cat", ExecDecision::Allow)
        .rule("pwd", ExecDecision::Allow)
        .rule("git status", ExecDecision::Allow)
        .rule("cargo test", ExecDecision::Allow)
        .rule("ollama", ExecDecision::Allow)
        .rule(
            "sudo",
            ExecDecision::Prompt("élévation de privilèges".to_string()),
        )
        .rule("rm", ExecDecision::Prompt("suppression".to_string()))
        .rule("docker", ExecDecision::Prompt("conteneur".to_string()))
        .rule("curl", ExecDecision::Prompt("réseau sortant".to_string()))
}

fn decision_str(d: &omega_types::ExecDecision) -> (String, Option<String>) {
    match d {
        omega_types::ExecDecision::Allow => ("allow".to_string(), None),
        omega_types::ExecDecision::Prompt(r) => ("prompt".to_string(), Some(r.clone())),
        omega_types::ExecDecision::Forbidden(r) => ("forbidden".to_string(), Some(r.clone())),
    }
}

pub async fn policy() -> Json<PolicyResponse> {
    let p = reference_policy();
    let rules: Vec<PolicyRule> = p
        .rules()
        .iter()
        .map(|r| {
            let (d, reason) = decision_str(&r.decision);
            PolicyRule {
                prefix: r.prefix.clone(),
                decision: d,
                reason,
            }
        })
        .collect();
    let (default, _) = decision_str(&p.default_decision());
    Json(PolicyResponse {
        schema_version: SCHEMA_VERSION,
        strict: p.is_strict(),
        rules,
        default,
        note:
            "Politique de référence du module omega-sandbox (ce serveur n'exécute aucune commande)."
                .to_string(),
    })
}

/// Corps de `POST /api/policy/evaluate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateRequest {
    pub command: String,
}

/// Résultat d'évaluation (aucune exécution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateResponse {
    pub schema_version: u32,
    pub command: String,
    /// `allow` | `prompt` | `forbidden`.
    pub decision: String,
    pub reason: Option<String>,
    /// La commande compose-t-elle (métacaractères hors quotes) ?
    pub compound: bool,
    /// Contient-elle une substitution (`$(...)`, backticks, `<(...)`) ?
    pub substitution: bool,
    /// Décision effective après la garde anti-composition du backend.
    pub effective: String,
}

pub async fn evaluate(
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, ApiError> {
    if req.command.trim().is_empty() {
        return Err(ApiError::bad_request("commande vide".to_string()));
    }
    let p = reference_policy();
    let (decision, reason) = decision_str(&p.decide(&req.command));
    let compound = omega_sandbox::has_unquoted_metachars(&req.command);
    let substitution = omega_sandbox::has_any_substitution(&req.command);
    // Même garde que `LocalSandboxBackend::run` : Allow + métacaractères -> Prompt.
    let effective = if decision == "allow" && compound {
        "prompt".to_string()
    } else {
        decision.clone()
    };
    Ok(Json(EvaluateResponse {
        schema_version: SCHEMA_VERSION,
        command: req.command,
        decision,
        reason,
        compound,
        substitution,
        effective,
    }))
}

/* --- Contexte --------------------------------------------------------------- */

/// `GET /api/context` : paramètres d'économie de contexte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResponse {
    pub schema_version: u32,
    pub estimator: String,
    pub max_tokens: usize,
    pub required_context: u32,
    pub sample: Vec<ContextSample>,
}

/// Échantillon d'estimation (texte -> tokens).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSample {
    pub chars: usize,
    pub tokens: usize,
}

pub async fn context(State(state): State<Arc<AppState>>) -> Json<ContextResponse> {
    let est = omega_context::TokenEstimator::new();
    let big = "x".repeat(400);
    let huge = "y".repeat(4000);
    let texts: [&str; 4] = ["", "bonjour", big.as_str(), huge.as_str()];
    let sample = texts
        .iter()
        .map(|t| ContextSample {
            chars: t.chars().count(),
            tokens: est.estimate(t),
        })
        .collect();
    Json(ContextResponse {
        schema_version: SCHEMA_VERSION,
        estimator: "ceil(chars / 4) — déterministe, sans LLM".to_string(),
        max_tokens: state.config.max_tokens,
        required_context: REQUIRED_CONTEXT,
        sample,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::tests::test_state;

    fn state() -> State<Arc<AppState>> {
        let (s, _) = test_state("sys");
        State(Arc::new(s))
    }

    #[tokio::test]
    async fn system_lists_all_capabilities_and_crates() {
        let body = system(state()).await.0;
        assert_eq!(body.crates.len(), 14);
        assert_eq!(body.capabilities.len(), 26);
        // Trois capacités câblées dans l'état de test.
        assert_eq!(body.counts.capabilities_wired, 4);
        let llm = body
            .capabilities
            .iter()
            .find(|c| c.name == "llm-provider")
            .unwrap();
        assert_eq!(llm.regime, "ensemble");
        assert_eq!(llm.active, vec!["stub".to_string()]);
        let ctx = body
            .capabilities
            .iter()
            .find(|c| c.name == "context-engine")
            .unwrap();
        assert_eq!(ctx.regime, "slot");
    }

    #[tokio::test]
    async fn classify_is_deterministic_and_rejects_empty() {
        let ok = classify(Json(ClassifyRequest {
            text: "Résume ce document en trois points".to_string(),
        }))
        .await
        .unwrap()
        .0;
        assert_eq!(ok.task_code, "T2");
        assert!(matches!(ok.task, omega_types::TaskType::Summarization));
        assert!(ok.weights.chat > 0.0);

        let err = classify(Json(ClassifyRequest {
            text: "   ".to_string(),
        }))
        .await
        .unwrap_err();
        assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn policy_exposes_rules_and_deny_default() {
        let body = policy().await.0;
        assert!(!body.rules.is_empty());
        assert_eq!(body.default, "forbidden");
        assert!(!body.strict);
        assert!(body
            .rules
            .iter()
            .any(|r| r.prefix == "sudo" && r.decision == "prompt"));
    }

    #[tokio::test]
    async fn evaluate_flags_composition_and_escalates() {
        // `echo hi; rm -rf /` : règle `echo` -> allow, mais composition -> prompt.
        let r = evaluate(Json(EvaluateRequest {
            command: "echo hi; rm -rf /".to_string(),
        }))
        .await
        .unwrap()
        .0;
        assert_eq!(r.decision, "allow");
        assert!(r.compound);
        assert_eq!(r.effective, "prompt");

        // `echo hi` simple : allow effectif.
        let r = evaluate(Json(EvaluateRequest {
            command: "echo hi".to_string(),
        }))
        .await
        .unwrap()
        .0;
        assert_eq!(r.effective, "allow");
        assert!(!r.compound);
        assert!(!r.substitution);

        // `curl` est en prompt ; substitution détectée.
        let r = evaluate(Json(EvaluateRequest {
            command: "curl $(cat secret)".to_string(),
        }))
        .await
        .unwrap()
        .0;
        assert_eq!(r.decision, "prompt");
        assert!(r.substitution);

        // Vide -> 400.
        let err = evaluate(Json(EvaluateRequest {
            command: "".to_string(),
        }))
        .await
        .unwrap_err();
        assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn runs_lists_executed_runs() {
        let (s, _) = test_state("sys-runs");
        let st = State(Arc::new(s.clone()));
        // Un run via le pipeline (stub, sans réseau).
        crate::pipeline::execute_run(
            &s,
            vec![omega_types::Message::new(
                omega_types::Role::User,
                "bonjour",
            )],
            Some("run-liste-1".to_string()),
        )
        .await
        .unwrap();
        let body = runs(st).await.0;
        assert!(body.runs.iter().any(|r| r.run_id == "run-liste-1"));
    }

    #[tokio::test]
    async fn context_estimates_samples() {
        let body = context(state()).await.0;
        assert_eq!(body.sample.len(), 4);
        assert_eq!(body.sample[0].tokens, 0);
        assert_eq!(body.sample[1].chars, 7);
        assert!(body.estimator.contains("ceil"));
    }

    #[tokio::test]
    async fn harness_catalog_is_exposed_and_isolable() {
        let all = harnesses().await.0;
        assert!(all.harnesses.len() >= 12);
        // Les harnais demandés par la vision sont présents.
        for id in ["cursor", "claude-code", "codex", "hermes"] {
            assert!(all.by_id(id).is_some(), "{id}");
        }
        // Isolation d'un harnais précis.
        let h = harness(axum::extract::Path("claude-code".to_string()))
            .await
            .unwrap()
            .0;
        assert_eq!(h.name, "Claude Code");
        assert!(h
            .principles
            .iter()
            .any(|p| p.status == omega_harness::PrincipleStatus::Adopted));
        // Inconnu -> 400.
        let err = harness(axum::extract::Path("nope".to_string()))
            .await
            .unwrap_err();
        assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn overview_aggregates_dashboard_in_one_call() {
        let (st, _) = test_state("sys-overview");
        let body = overview(State(Arc::new(st.clone()))).await.0;
        assert_eq!(body.status, "ok");
        assert_eq!(body.counts.capabilities, 26);
        assert_eq!(body.crate_count, 14);
        assert_eq!(body.model_record.cost_input_per_1m, 0.0);
        assert!(body.harness.harnesses >= 12);
        assert!(body.harness.adopted >= 10);
        let _ = st;
    }

    #[tokio::test]
    async fn registry_isolates_entries_with_kind_and_quarantine() {
        let (st, _) = test_state("sys-registry");
        let body = registry(State(Arc::new(st))).await.0;
        assert_eq!(body.capabilities.len(), 26);
        let llm = body
            .capabilities
            .iter()
            .find(|c| c.name == "llm-provider")
            .unwrap();
        assert_eq!(llm.entries.len(), 1);
        assert_eq!(llm.entries[0].id, "stub");
        assert_eq!(llm.entries[0].kind, "trait-object");
        assert!(!llm.entries[0].quarantined);
    }

    #[tokio::test]
    async fn config_and_routes_are_self_describing() {
        let (st, _) = test_state("sys-config");
        let c = config(State(Arc::new(st.clone()))).await.0;
        assert_eq!(c.ollama_model, "test-model");
        assert_eq!(c.max_tokens, 8000);
        let r = routes().await.0;
        assert!(r.routes.iter().any(|x| x.path == "/api/overview"));
        assert!(r
            .routes
            .iter()
            .any(|x| x.path == "/api/runs/:run_id/events"));
        let _ = st;
    }

    #[tokio::test]
    async fn run_events_expose_raw_journal_for_traceability() {
        let (st, _) = test_state("sys-events");
        crate::pipeline::execute_run(
            &st,
            vec![omega_types::Message::new(
                omega_types::Role::User,
                "bonjour",
            )],
            Some("trace-raw-1".to_string()),
        )
        .await
        .unwrap();
        let body = run_events(
            State(Arc::new(st)),
            axum::extract::Path("trace-raw-1".to_string()),
        )
        .await
        .unwrap()
        .0;
        assert!(body.events.iter().any(|e| e.name == "plan.defined"));
        assert!(body.events.iter().any(|e| e.name == "step.completed"));
        assert!(body.events.iter().any(|e| e.name == "infer.completed"));
        // Sequences monotones.
        let seqs: Vec<u64> = body.events.iter().map(|e| e.seq).collect();
        let mut sorted = seqs.clone();
        sorted.sort_unstable();
        assert_eq!(seqs, sorted);

        // run_id invalide -> 400.
        let err = run_events(
            State(Arc::new(test_state("sys-events-4").0)),
            axum::extract::Path("../x".to_string()),
        )
        .await
        .unwrap_err();
        assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}

/* --- Catalogue de harnais (reverse-engineering) ----------------------------- */

/// Résumé du catalogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessCounts {
    pub harnesses: usize,
    pub principles: usize,
    pub adopted: usize,
    pub planned: usize,
    pub rejected: usize,
}

fn harness_counts() -> HarnessCounts {
    let c = omega_harness::Catalog::embedded();
    let (a, p, r) = c.counts();
    HarnessCounts {
        harnesses: c.harnesses.len(),
        principles: a + p + r,
        adopted: a,
        planned: p,
        rejected: r,
    }
}

/// `GET /api/harnesses` : le catalogue complet des harnais observés.
pub async fn harnesses() -> Json<omega_harness::Catalog> {
    Json(omega_harness::Catalog::embedded())
}

/// `GET /api/harnesses/:id` : un harnais précis.
pub async fn harness(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<omega_harness::Harness>, ApiError> {
    omega_harness::Catalog::embedded()
        .by_id(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::bad_request(format!("harnais inconnu : {id}")))
}

/* --- Configuration ---------------------------------------------------------- */

/// `GET /api/config` : configuration effective (sans secret).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub schema_version: u32,
    pub port: u16,
    pub ollama_url: String,
    pub ollama_model: String,
    pub event_dir: String,
    pub max_tokens: usize,
    pub ui_dir: Option<String>,
    pub ui_served: bool,
}

pub async fn config(State(state): State<Arc<AppState>>) -> Json<ConfigResponse> {
    let c = &state.config;
    Json(ConfigResponse {
        schema_version: SCHEMA_VERSION,
        port: c.port,
        ollama_url: c.ollama_base_url.clone(),
        ollama_model: c.ollama_model.clone(),
        event_dir: c.event_dir.display().to_string(),
        max_tokens: c.max_tokens,
        ui_dir: c.ui_dir.as_ref().map(|d| d.display().to_string()),
        ui_served: c.ui_dir.as_ref().map(|d| d.is_dir()).unwrap_or(false),
    })
}

/* --- Registre (isolation des plugins) --------------------------------------- */

/// Entree de registre exposée.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntryDto {
    pub id: String,
    pub version: String,
    pub priority: i32,
    pub quarantined: bool,
    /// `typed` | `trait-object`.
    pub kind: String,
}

/// Capacité + ses entrées.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryCapabilityDto {
    pub name: String,
    pub regime: String,
    pub entries: Vec<RegistryEntryDto>,
}

/// `GET /api/registry` : le registre complet (manifests, priorité, quarantaine).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryResponse {
    pub schema_version: u32,
    pub capabilities: Vec<RegistryCapabilityDto>,
}

pub async fn registry(State(state): State<Arc<AppState>>) -> Json<RegistryResponse> {
    let described = state.app.describe();
    let capabilities = crate::routes::ALL_CAPABILITIES
        .iter()
        .map(|cap| {
            let entries = described
                .iter()
                .find(|(c, _)| c == cap)
                .map(|(_, v)| {
                    v.iter()
                        .map(|e| RegistryEntryDto {
                            id: e.id.clone(),
                            version: e.version.clone(),
                            priority: e.priority,
                            quarantined: e.quarantined,
                            kind: match e.kind {
                                omega_registry::EntryKind::Typed => "typed".to_string(),
                                omega_registry::EntryKind::TraitObject => {
                                    "trait-object".to_string()
                                }
                            },
                        })
                        .collect()
                })
                .unwrap_or_default();
            RegistryCapabilityDto {
                name: cap.as_str().to_string(),
                regime: match cap.regime() {
                    omega_types::Regime::Slot => "slot".to_string(),
                    omega_types::Regime::Ensemble => "ensemble".to_string(),
                },
                entries,
            }
        })
        .collect();
    Json(RegistryResponse {
        schema_version: SCHEMA_VERSION,
        capabilities,
    })
}

/* --- Surface d'API ---------------------------------------------------------- */

/// Un endpoint de l'API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub method: String,
    pub path: String,
    pub description: String,
}

/// `GET /api/routes` : surface d'API auto-descriptive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutesResponse {
    pub schema_version: u32,
    pub routes: Vec<RouteInfo>,
}

pub async fn routes() -> Json<RoutesResponse> {
    let r = |m: &str, p: &str, d: &str| RouteInfo {
        method: m.to_string(),
        path: p.to_string(),
        description: d.to_string(),
    };
    Json(RoutesResponse {
        schema_version: SCHEMA_VERSION,
        routes: vec![
            r("GET", "/api/health", "Sonde de vie (schema_version)"),
            r(
                "GET",
                "/api/overview",
                "Vue d'ensemble (dashboard, 1 requête)",
            ),
            r("GET", "/api/system", "Architecture (crates) + capacités"),
            r(
                "GET",
                "/api/registry",
                "Registre : manifests, priorité, quarantaine",
            ),
            r(
                "GET",
                "/api/config",
                "Configuration effective (sans secret)",
            ),
            r("GET", "/api/routes", "Cette surface d'API"),
            r("GET", "/api/plugins", "Capacités câblées (liste plate)"),
            r(
                "GET",
                "/api/models",
                "Catalogue Ollama + record de sélection",
            ),
            r(
                "POST",
                "/api/classify",
                "Classifieur de tâche (déterministe)",
            ),
            r(
                "POST",
                "/api/run",
                "Run complet (classify→select→assemble→infer)",
            ),
            r("GET", "/api/runs", "Journal des runs (résumés rejoués)"),
            r(
                "GET",
                "/api/runs/:run_id/events",
                "Événements bruts d'un run (traçabilité)",
            ),
            r("GET", "/api/trace/:run_id", "Séquence rejouée + réponse"),
            r("GET", "/api/policy", "Politique d'exécution de référence"),
            r(
                "POST",
                "/api/policy/evaluate",
                "Évaluer une commande (sans exécuter)",
            ),
            r("GET", "/api/context", "Paramètres d'économie de contexte"),
            r(
                "GET",
                "/api/harnesses",
                "Catalogue de reverse-engineering des harnais (principes)",
            ),
            r(
                "GET",
                "/api/harnesses/:id",
                "Un harnais précis (principes détaillés)",
            ),
        ],
    })
}

/* --- Événements bruts d'un run (traçabilité) -------------------------------- */

/// `GET /api/runs/:run_id/events` : journal brut (chaque événement durable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEventsResponse {
    pub schema_version: u32,
    pub run_id: String,
    pub events: Vec<omega_types::DurableEvent>,
}

pub async fn run_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(run_id): axum::extract::Path<String>,
) -> Result<Json<RunEventsResponse>, ApiError> {
    if !omega_app::is_valid_run_id(&run_id) {
        return Err(ApiError::bad_request(format!(
            "run_id invalide : {run_id:?}"
        )));
    }
    let events = state
        .store
        .read_run(&run_id)
        .map_err(ApiError::from_domain)?;
    Ok(Json(RunEventsResponse {
        schema_version: SCHEMA_VERSION,
        run_id,
        events,
    }))
}

/* --- Vue d'ensemble (dashboard) --------------------------------------------- */

/// `GET /api/overview` : tout ce qu'affiche le dashboard, en une requête.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewResponse {
    pub schema_version: u32,
    pub status: String,
    pub config: ConfigResponse,
    pub counts: SystemCounts,
    pub capabilities: Vec<CapabilityInfo>,
    pub crate_count: usize,
    pub recent_runs: Vec<RunEntry>,
    pub model_record: omega_types::ModelRecord,
    pub harness: HarnessCounts,
}

pub async fn overview(State(state): State<Arc<AppState>>) -> Json<OverviewResponse> {
    let capabilities: Vec<CapabilityInfo> = crate::routes::ALL_CAPABILITIES
        .iter()
        .map(|cap| CapabilityInfo {
            name: cap.as_str().to_string(),
            regime: match cap.regime() {
                omega_types::Regime::Slot => "slot".to_string(),
                omega_types::Regime::Ensemble => "ensemble".to_string(),
            },
            active: state.app.active_ids(*cap),
        })
        .collect();
    let wired = capabilities.iter().filter(|c| !c.active.is_empty()).count();
    let c = &state.config;
    let config = ConfigResponse {
        schema_version: SCHEMA_VERSION,
        port: c.port,
        ollama_url: c.ollama_base_url.clone(),
        ollama_model: c.ollama_model.clone(),
        event_dir: c.event_dir.display().to_string(),
        max_tokens: c.max_tokens,
        ui_dir: c.ui_dir.as_ref().map(|d| d.display().to_string()),
        ui_served: c.ui_dir.as_ref().map(|d| d.is_dir()).unwrap_or(false),
    };
    Json(OverviewResponse {
        schema_version: SCHEMA_VERSION,
        status: "ok".to_string(),
        config,
        counts: SystemCounts {
            crates: 14,
            capabilities: capabilities.len(),
            capabilities_wired: wired,
        },
        capabilities,
        crate_count: 14,
        recent_runs: collect_runs(&state, 8),
        model_record: local_record(state.model(), Some(32768)),
        harness: harness_counts(),
    })
}

/// Collecte les `limit` runs les plus récents (résumés rejoués).
fn collect_runs(state: &AppState, limit: usize) -> Vec<RunEntry> {
    let dir = state.store.dir();
    let mut entries: Vec<(std::time::SystemTime, String)> = Vec::new();
    if let Ok(read) = std::fs::read_dir(dir) {
        for e in read.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("jsonl") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let modified = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            entries.push((modified, stem.to_string()));
        }
    }
    entries.sort_by_key(|e| std::cmp::Reverse(e.0));
    entries.truncate(limit);
    entries
        .into_iter()
        .filter_map(|(_, run_id)| {
            let journal = state.store.read_run(&run_id).ok()?;
            let s = omega_otel::TraceReplayer::summarize(&run_id, journal);
            Some(RunEntry {
                run_id,
                events: s.total_events,
                steps_completed: s.steps_completed,
                steps_failed: s.steps_failed,
                outcome: s.outcome,
            })
        })
        .collect()
}
