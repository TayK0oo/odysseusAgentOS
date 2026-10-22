//! Pipeline d'un run : classify → select → assemble/infer → resume.
//!
//! 100 % determine avant l'inference : le classifieur est heuristique (zero
//! LLM) et la selection est Pareto + score composite sur le candidat local.
//! Seule l'inference touche le reseau (Ollama local) — et les tests utilisent
//! un provider bouche (aucun reseau).

use omega_app::RunInput;
use omega_otel::TraceReplayer;
use omega_ports::EventStore;
use omega_providers::local_record;
use omega_router::selector::ParetoSelector;
use omega_router::{CompositeScoringPolicy, HeuristicTaskClassifier, TaskClassifier};
use omega_types::{
    Capabilities, DurableEvent, Message, ModelId, OmegaError, OmegaResult, RoutingRequest,
};

use crate::config::REQUIRED_CONTEXT;
use crate::dto::RunResponse;
use crate::state::AppState;

/// Candidat local (self-hosted, coût nul) : le record canonique W10
/// (`omega_providers::local_record`, contexte 32k). L'identifiant suit le
/// provider (`ollama/<modèle>`) pour que la décision réfère l'inférence réelle.
pub fn local_candidate(model: &str) -> omega_types::ModelRecord {
    local_record(model, Some(32768))
}

/// Dernière réponse de modèle lue dans le journal (`infer.completed`).
///
/// La sortie du modèle est un événement durable (W8) : l'UI la restitue **sans
/// ré-exécuter** l'inférence. `None` si le run n'a pas d'inférence aboutie.
pub fn last_answer(journal: &[DurableEvent]) -> Option<String> {
    journal
        .iter()
        .rev()
        .find(|e| e.name == "infer.completed")
        .and_then(|e| e.payload.get("content"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

/// Execute un run complet et retourne la reponse d'API.
///
/// - `run_id = None` : l'`App` genere l'identifiant (compteur `run-N`) ;
/// - le resume provient du journal durable (preuve, pas d'affirmation).
pub async fn execute_run(
    state: &AppState,
    messages: Vec<Message>,
    run_id: Option<String>,
) -> OmegaResult<RunResponse> {
    execute_run_with_budget(state, messages, run_id, state.config.tool_budget).await
}

/// Execute un run avec un budget d'executions d'outils explicite.
///
/// La boucle d'outils (P1-1) remplace l'inference unique quand une surface
/// d'outils est configuree : `assemble` puis `tool_loop` (detection ->
/// execution -> observation -> re-injection -> multi-tours). Sans surface,
/// le comportement d'avant est conserve (`assemble` + `infer`).
pub async fn execute_run_with_budget(
    state: &AppState,
    messages: Vec<Message>,
    run_id: Option<String>,
    tool_budget: usize,
) -> OmegaResult<RunResponse> {
    if messages.is_empty() {
        return Err(OmegaError::Config("aucun message fourni".to_string()));
    }
    // 1. Classification deterministe (zero LLM) sur le texte concatene.
    let text = messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let task = HeuristicTaskClassifier::new().classify(&RoutingRequest::new(&text));

    // 2. Sélection Pareto sur le seul candidat local (cout nul garanti).
    // `select_with_local_preference` : sans effet à un seul candidat, mais
    // contrat homogène avec les flux multi-candidats (W10).
    let selector = ParetoSelector::new(CompositeScoringPolicy::new());
    let candidates = [local_candidate(state.model())];
    let decision = selector
        .select_with_local_preference(task, Capabilities::default(), REQUIRED_CONTEXT, &candidates)
        .map_err(|e| OmegaError::Config(format!("selection : {e:?}")))?;

    // 3. Assemble (ContextEngine du registre) + execution.
    let app = &state.app;
    let mut ops: Vec<Box<dyn omega_core::Operation>> =
        vec![app.assemble_operation(messages.clone())?];

    // Identifiant du run : la boucle d'outils ecrit sa trace SOUS cet
    // identifiant, il faut donc le connaître AVANT de construire l'operation.
    let effective_run_id = run_id.clone().unwrap_or_else(omega_app::next_run_id);

    match state.tool_surface() {
        Some(surface) => {
            // Boucle d'outils : le provider est resolu par le registre, la
            // surface vient de la configuration (slot explicite — la surface
            // MCP du registre ne peut pas servir le cas local, cf.
            // `config::ToolSurfaceKind`).
            let provider = app.provider()?;
            // Budget de politique (P1-3) : plafonds portants sur le budget
            // d'executions d'outils. La politique calcule le cumul monotone ;
            // la boucle ne connait que le port.
            let policy_budget: Option<std::sync::Arc<dyn omega_ports::RunBudget>> = {
                let limits = omega_policy::budget::Limits {
                    max_steps: tool_budget as u64,
                    ..omega_policy::budget::Limits::default()
                };
                Some(std::sync::Arc::new(omega_policy::PolicyRunBudget::new(
                    std::sync::Arc::new(omega_policy::budget::Budget::new(limits)),
                )))
            };
            ops.push(Box::new(omega_core::tool_loop::ToolLoopOperation {
                provider,
                model: ModelId::new(decision.model_id.clone()),
                surface,
                messages,
                budget: tool_budget,
                policy_budget,
                run_id: effective_run_id.clone(),
                trace: None,
            }));
        }
        None => {
            ops.push(app.infer_operation(ModelId::new(decision.model_id.clone()), messages)?);
        }
    }

    let input = RunInput::new(ops).with_run_id(effective_run_id);
    let outcome = app.run(input).await?;

    // 4. Preuve par le journal : resume rejoue (deterministe) + reponse du modele.
    let journal = state.store.read_run(&outcome.run_id)?;
    let answer = last_answer(&journal);
    let summary = TraceReplayer::summarize(&outcome.run_id, journal);

    Ok(RunResponse {
        run_id: outcome.run_id,
        status: outcome.status,
        decision,
        summary,
        answer,
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use async_trait::async_trait;
    use omega_app::AppBuilder;
    use omega_context::ContextContributor;
    use omega_plugin_api::{Contributor, InstallContext};
    use omega_registry::{Manifest, RegistryBuilder};
    use omega_types::Capability;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Provider bouche : repond sans reseau (tests du pipeline + handlers).
    pub struct StubProvider {
        calls: AtomicUsize,
    }

    impl StubProvider {
        pub fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl omega_ports::Provider for StubProvider {
        fn id(&self) -> &str {
            "stub"
        }

        async fn complete(
            &self,
            req: omega_types::ChatRequest,
        ) -> OmegaResult<omega_types::ChatResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(omega_types::ChatResponse {
                message: Message::new(
                    omega_types::Role::Assistant,
                    format!(
                        "echo: {}",
                        req.messages
                            .last()
                            .map(|m| m.content.as_str())
                            .unwrap_or("")
                    ),
                ),
                usage: Default::default(),
            })
        }
    }

    struct StubProviderContributor {
        provider: Arc<dyn omega_ports::Provider>,
    }

    impl Contributor for StubProviderContributor {
        fn name(&self) -> &str {
            "stub-provider"
        }

        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder
                .register_dyn(
                    Manifest::new("stub", Capability::LlmProvider, "1"),
                    self.provider.clone(),
                )
                .map(|_| ())
        }
    }

    struct StubStoreContributor {
        store: Arc<dyn EventStore>,
    }

    impl Contributor for StubStoreContributor {
        fn name(&self) -> &str {
            "stub-store"
        }

        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder
                .register_dyn(
                    Manifest::new("stub-file", Capability::EventStore, "1"),
                    self.store.clone(),
                )
                .map(|_| ())
        }
    }

    /// Etat de test : stub (zero reseau) + vrai ContextEngine + vrai FileEventStore.
    pub fn test_state(tag: &str) -> (AppState, Arc<omega_app::FileEventStore>) {
        let dir = std::env::temp_dir().join(format!("omega-server-test-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Arc::new(omega_app::FileEventStore::new(dir));
        let app = AppBuilder::new()
            .add_contributor(Box::new(StubProviderContributor {
                provider: Arc::new(StubProvider::new()),
            }))
            .add_contributor(Box::new(ContextContributor::new()))
            .add_contributor(Box::new(StubStoreContributor {
                store: store.clone(),
            }))
            // Meme cablage que la production : catalogue de harnais inclus.
            .add_contributor(Box::new(omega_harness::HarnessCatalogContributor::new()))
            .build()
            .unwrap();
        (
            AppState {
                app: Arc::new(app),
                store: store.clone(),
                composition: Arc::new(omega_adapters::composition::compose_ported_crates()),
                config: crate::config::ServerConfig {
                    ollama_model: "test-model".to_string(),
                    ..crate::config::ServerConfig::default()
                },
            },
            store,
        )
    }

    #[test]
    fn local_candidate_is_self_hosted_free() {
        use omega_types::ModelTier;
        let c = local_candidate("qwen2.5-coder:7b");
        assert_eq!(c.id, "ollama/qwen2.5-coder:7b");
        assert_eq!(c.tier, ModelTier::SelfHosted);
        assert_eq!(c.cost_input_per_1m, 0.0);
    }

    #[tokio::test]
    async fn pipeline_runs_without_network_and_summarizes() {
        let (state, _) = test_state("pipeline");
        let messages = vec![Message::new(
            omega_types::Role::User,
            "Resume ce texte en trois points : Omega est un harnais.",
        )];
        let resp = execute_run(&state, messages, Some("pipe-1".to_string()))
            .await
            .unwrap();
        assert_eq!(resp.run_id, "pipe-1");
        assert_eq!(resp.status, omega_core::RunStatus::Ok);
        assert_eq!(resp.decision.model_id, "ollama/test-model");
        // Preuve par le journal : 2 etapes completees, issue completed.
        assert_eq!(resp.summary.steps_completed, 2);
        assert_eq!(resp.summary.steps_failed, 0);
        assert_eq!(resp.summary.outcome, omega_otel::ReplayedOutcome::Completed);
    }

    #[tokio::test]
    async fn pipeline_rejects_empty_messages() {
        let (state, _) = test_state("pipeline-empty");
        let err = execute_run(&state, vec![], None).await.unwrap_err();
        assert!(matches!(err, OmegaError::Config(_)));
    }

    #[tokio::test]
    async fn pipeline_exposes_model_answer_from_journal() {
        // La reponse du modele vient du journal (`infer.completed`), pas d'un
        // etat cache : l'UI la restitue sans re-executer l'inference (W8).
        let (state, _) = test_state("pipeline-answer");
        let messages = vec![Message::new(omega_types::Role::User, "bonjour")];
        let resp = execute_run(&state, messages, Some("ans-1".to_string()))
            .await
            .unwrap();
        let answer = resp.answer.expect("reponse presente dans le journal");
        assert!(answer.starts_with("echo:"), "{answer}");
        // Le journal contient bien l'evenement dedie.
        let journal = state.store.read_run("ans-1").unwrap();
        assert!(journal.iter().any(|e| e.name == "infer.completed"));
    }
}
