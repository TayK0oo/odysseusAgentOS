//! Concurrence (P2) — deux `/api/run` parallèles sur le même event_dir
//! n'écrasent aucune mise à jour et laissent l'effet (append.txt) présent
//! exactement une fois par run.

use std::sync::Arc;

use async_trait::async_trait;
use omega_ports::Provider;
use omega_server::execute_run;
use omega_types::{ChatRequest, ChatResponse, Message, Role, Usage};

struct AppendProvider;

#[async_trait]
impl Provider for AppendProvider {
    fn id(&self) -> &str {
        "append-provider"
    }
    async fn complete(&self, req: ChatRequest) -> omega_types::OmegaResult<ChatResponse> {
        let last = req
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        if last.contains("appended") || last.contains("canary-line") && last.contains("bytes") {
            return Ok(ChatResponse {
                message: Message::new(Role::Assistant, last),
                usage: Usage::default(),
            });
        }
        // Heuristique : si le dernier message est une observation d'outil, on l'echo.
        if req.messages.len() > 2 {
            return Ok(ChatResponse {
                message: Message::new(Role::Assistant, last),
                usage: Usage::default(),
            });
        }
        Ok(ChatResponse {
            message: Message::new(
                Role::Assistant,
                "{\"tool\": \"append_file\", \"arguments\": {\"path\": \"append.txt\", \"content\": \"canary-line\"}}",
            ),
            usage: Usage::default(),
        })
    }
}

#[tokio::test]
async fn deux_runs_paralleles_ne_perdent_pas_de_mise_a_jour() {
    let root = std::env::temp_dir().join("omega-conc-root");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let dir = std::env::temp_dir().join("omega-conc-events");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let store = Arc::new(omega_app::FileEventStore::new(dir.clone()));
    let app = omega_app::AppBuilder::new()
        .add_contributor(Box::new(P(Arc::new(AppendProvider))))
        .add_contributor(Box::new(omega_context::ContextContributor::new()))
        .add_contributor(Box::new(S(
            Arc::clone(&store) as Arc<dyn omega_ports::EventStore>
        )))
        .build()
        .unwrap();
    let state = Arc::new(omega_server::AppState {
        app: Arc::new(app),
        store: store.clone(),
        composition: Arc::new(omega_adapters::composition::compose_ported_crates()),
        config: omega_server::ServerConfig {
            ollama_model: "test-model".to_string(),
            tool_surface: Some(omega_server::ToolSurfaceKind::LocalFs { root: root.clone() }),
            ..omega_server::ServerConfig::default()
        },
    });

    // Il faut les contributors P et S comme dans les tests existants.
    // (définis ci-dessous via use? on les redéfinit localement)
    let messages = vec![Message::new(Role::User, "Ajoute une ligne")];

    let s1 = state.clone();
    let s2 = state.clone();
    let m1 = messages.clone();
    let m2 = messages.clone();

    let (r1, r2) = tokio::join!(
        execute_run(&s1, m1, Some("conc-1".into()), false),
        execute_run(&s2, m2, Some("conc-2".into()), false),
    );
    assert!(r1.is_ok(), "run 1 doit reussir");
    assert!(r2.is_ok(), "run 2 doit reussir");

    // L'effet : append.txt contient 2 lignes (une par run), aucune perdue.
    let append_path = root.join("append.txt");
    assert!(append_path.exists(), "append.txt doit exister");
    let content = std::fs::read_to_string(&append_path).unwrap();
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 2, "2 lignes attendues, trouve {}", lines.len());

    // Chaque journal contient son tool.call.completed.
    for rid in ["conc-1", "conc-2"] {
        let journal = omega_ports::EventStore::read_run(&*store, rid).unwrap();
        let n = journal
            .iter()
            .filter(|e| e.name == "tool.call.completed")
            .count();
        assert_eq!(n, 1, "run {rid} : un seul tool.call.completed, trouve {n}");
    }

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// Helpers contributors (mêmes que tool_loop_route.rs)
struct P(Arc<dyn Provider>);
impl omega_plugin_api::Contributor for P {
    fn name(&self) -> &str {
        "append-provider-contrib"
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
        "file-store-contrib"
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
