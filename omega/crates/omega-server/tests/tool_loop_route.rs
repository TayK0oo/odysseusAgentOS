//! B1 (P1-1) — `/api/run` doit piloter la **boucle d'outils** (chemin réel).
//!
//! Contrôle négatif du contrôle : ce test prouve que le chemin `/api/run`
//! exécute un outil réel, réinjecte l'observation, et que la réponse finale
//! contient la sortie de l'outil (le nom du fichier canari).
//!
//! Avant B1, `execute_run` enchaînait `assemble` + `infer` : un seul appel
//! LLM, aucune surface d'outils. Le canari n'apparaissait donc jamais.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use omega_ports::Provider;
use omega_types::{ChatRequest, ChatResponse, Message, Role, Usage};

/// Provider scripté : tour 1 = appel d'outil ReAct, tour 2 = réponse qui
/// REPREND le contenu de l'observation (le canari). C'est le comportement
/// d'un modèle qui lit l'observation avant de répondre.
struct ScriptedToolProvider {
    calls: AtomicUsize,
}

#[async_trait]
impl Provider for ScriptedToolProvider {
    fn id(&self) -> &str {
        "scripted-tool"
    }

    async fn complete(&self, req: ChatRequest) -> omega_types::OmegaResult<ChatResponse> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        if n == 0 {
            // Forme ReAct (ADR-0006) : le wire ne transporte pas encore
            // `tool_calls`, le repli ReAct est le chemin canonique.
            let content = "{\"tool\": \"ls\", \"arguments\": {\"path\": \".\"}}".to_string();
            Ok(ChatResponse {
                message: Message::new(Role::Assistant, content),
                usage: Usage::default(),
            })
        } else {
            let last = req
                .messages
                .last()
                .map(|m| m.content.clone())
                .unwrap_or_default();
            Ok(ChatResponse {
                message: Message::new(Role::Assistant, format!("Fichiers : {last}")),
                usage: Usage::default(),
            })
        }
    }
}

struct P(Arc<dyn Provider>);
impl omega_plugin_api::Contributor for P {
    fn name(&self) -> &str {
        "scripted-tool-provider"
    }
    fn contribute(
        &self,
        builder: &mut omega_registry::RegistryBuilder,
        _ctx: &omega_plugin_api::InstallContext,
    ) -> omega_types::OmegaResult<()> {
        builder
            .register_dyn(
                omega_registry::Manifest::new("ollama", omega_types::Capability::LlmProvider, "1"),
                self.0.clone(),
            )
            .map(|_| ())
    }
}

struct S(Arc<dyn omega_ports::EventStore>);
impl omega_plugin_api::Contributor for S {
    fn name(&self) -> &str {
        "file-store"
    }
    fn contribute(
        &self,
        builder: &mut omega_registry::RegistryBuilder,
        _ctx: &omega_plugin_api::InstallContext,
    ) -> omega_types::OmegaResult<()> {
        builder
            .register_dyn(
                omega_registry::Manifest::new("file", omega_types::Capability::EventStore, "1"),
                self.0.clone(),
            )
            .map(|_| ())
    }
}

/// État serveur dont le provider est scripté et dont la surface d'outils est
/// un FS local ancré sur `root`.
fn state_with_tool_provider(root: &std::path::Path) -> omega_server::AppState {
    let dir = std::env::temp_dir().join("omega-b1-events");
    let _ = std::fs::remove_dir_all(&dir);
    let store = Arc::new(omega_app::FileEventStore::new(dir));
    let app = omega_app::AppBuilder::new()
        .add_contributor(Box::new(P(Arc::new(ScriptedToolProvider {
            calls: AtomicUsize::new(0),
        }))))
        .add_contributor(Box::new(omega_context::ContextContributor::new()))
        .add_contributor(Box::new(S(
            Arc::clone(&store) as Arc<dyn omega_ports::EventStore>
        )))
        .build()
        .unwrap();
    omega_server::AppState {
        app: Arc::new(app),
        store,
        composition: Arc::new(omega_adapters::composition::compose_ported_crates()),
        config: omega_server::ServerConfig {
            ollama_model: "test-model".to_string(),
            tool_surface: Some(omega_server::ToolSurfaceKind::LocalFs {
                root: root.to_path_buf(),
            }),
            ..omega_server::ServerConfig::default()
        },
    }
}

#[tokio::test]
async fn api_run_execute_un_outil_reel_et_reinjecte_l_observation() {
    // Canari sur le disque : invisible pour le modèle sauf par l'observation.
    let root = std::env::temp_dir().join("omega-b1-root");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let canary = "CANARY-b1-9f3a.txt";
    std::fs::write(root.join(canary), b"contenu canari").unwrap();

    let state = state_with_tool_provider(&root);
    let messages = vec![Message::new(Role::User, "Liste les fichiers")];

    let resp = omega_server::execute_run(&state, messages, Some("b1-run".into()))
        .await
        .expect("le run doit reussir");

    // 1. La réponse contient la sortie RÉELLE de l'outil : le nom du canari
    //    n'est pas dans le prompt, il ne peut venir que de `ls`.
    let answer = resp.answer.clone().unwrap_or_default();
    assert!(
        answer.contains(canary),
        "la reponse doit contenir le canari execute par l'outil ; reponse = {answer:?}"
    );

    // 2. Le journal contient les evenements de la boucle d'outils.
    let journal = omega_ports::EventStore::read_run(&*state.store, "b1-run").unwrap();
    let names: Vec<&str> = journal.iter().map(|e| e.name.as_str()).collect();
    assert!(
        names.contains(&"tool.call.completed"),
        "le journal doit contenir tool.call.completed ; evenements = {names:?}"
    );

    // Un seul evenement par execution d'outil (le bus est la source unique :
    // le champ `trace` de la boucle ne doit pas doubler l'ecriture).
    let n = journal
        .iter()
        .filter(|e| e.name == "tool.call.completed")
        .count();
    assert_eq!(
        n, 1,
        "un seul tool.call.completed par execution ; trouve {n}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

// --- Contrôles négatifs exigés -------------------------------------------

/// Un tool-call INCONNU ne doit pas paniquer : erreur structurée, run qui
/// se termine proprement, aucune trace de panic.
#[tokio::test]
async fn outil_inconnu_ne_panique_pas() {
    struct UnknownToolProvider;
    #[async_trait]
    impl Provider for UnknownToolProvider {
        fn id(&self) -> &str {
            "unknown-tool"
        }
        async fn complete(&self, _req: ChatRequest) -> omega_types::OmegaResult<ChatResponse> {
            Ok(ChatResponse {
                message: Message::new(
                    Role::Assistant,
                    "{\"tool\": \"outil_qui_nexiste_pas\", \"arguments\": {}}",
                ),
                usage: Usage::default(),
            })
        }
    }

    let root = std::env::temp_dir().join("omega-b1-root-unknown");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let dir = std::env::temp_dir().join("omega-b1-events-unknown");
    let _ = std::fs::remove_dir_all(&dir);
    let store = Arc::new(omega_app::FileEventStore::new(dir));
    let app = omega_app::AppBuilder::new()
        .add_contributor(Box::new(P(Arc::new(UnknownToolProvider))))
        .add_contributor(Box::new(omega_context::ContextContributor::new()))
        .add_contributor(Box::new(S(
            Arc::clone(&store) as Arc<dyn omega_ports::EventStore>
        )))
        .build()
        .unwrap();
    let state = omega_server::AppState {
        app: Arc::new(app),
        store,
        composition: Arc::new(omega_adapters::composition::compose_ported_crates()),
        config: omega_server::ServerConfig {
            ollama_model: "test-model".to_string(),
            tool_surface: Some(omega_server::ToolSurfaceKind::LocalFs { root: root.clone() }),
            ..omega_server::ServerConfig::default()
        },
    };

    let messages = vec![Message::new(Role::User, "appelle un outil inconnu")];
    // Pas de panic : le run se termine (Ok ou Err typee), jamais d'unwind.
    let _ = omega_server::execute_run(&state, messages, Some("b1-unknown".into())).await;

    let _ = std::fs::remove_dir_all(&root);
}

/// Un budget de 0 n'exécute AUCUN outil : le budget est monotone et strict.
#[tokio::test]
async fn budget_zero_n_execute_aucun_outil() {
    let root = std::env::temp_dir().join("omega-b1-root-budget");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let canary = "CANARY-budget.txt";
    std::fs::write(root.join(canary), b"x").unwrap();

    let state = state_with_tool_provider(&root);
    let messages = vec![Message::new(Role::User, "Liste les fichiers")];

    let resp = omega_server::execute_run_with_budget(&state, messages, Some("b1-budget".into()), 0)
        .await
        .expect("le run doit reussir");

    let answer = resp.answer.clone().unwrap_or_default();
    assert!(
        !answer.contains(canary),
        "budget 0 : aucun outil ne doit s'executer ; reponse = {answer:?}"
    );

    let journal = omega_ports::EventStore::read_run(&*state.store, "b1-budget").unwrap();
    let names: Vec<&str> = journal.iter().map(|e| e.name.as_str()).collect();
    assert!(
        !names.contains(&"tool.call.completed"),
        "budget 0 : aucun tool.call.completed ; evenements = {names:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}
