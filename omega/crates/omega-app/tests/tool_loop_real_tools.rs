//! Contrat d'intégration P1-1 : la boucle `ToolLoopOperation` branchée aux
//! **vrais outils filesystem** (`FsSurface`, jail de `omega-tools`) dans
//! l'assemblage applicatif — T1 écriture puis T0 lecture canari dans un
//! tmpdir, observation réinjectée au modèle au tour suivant, trace par
//! exécution (`run_id`, `call_id`, origine), interruption utilisateur.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use omega_core::tool_loop::ToolLoopOperation;
use omega_core::{
    cancellation_pair, Event, EventBus, InMemoryBus, Operation, RunContext, RunState,
};
use omega_ports::Provider;
use omega_tools::surface::FsSurface;
use omega_tools::ToolCtx;
use omega_types::{ChatRequest, ChatResponse, Message, ModelId, OmegaError, OmegaResult, Role};

/// Provider scripté : défile les contenus assistant, puis échoue.
struct ScriptedProvider {
    script: Mutex<std::collections::VecDeque<String>>,
}

impl ScriptedProvider {
    fn new(script: Vec<&str>) -> Self {
        Self {
            script: Mutex::new(script.into_iter().map(str::to_string).collect()),
        }
    }
}

#[async_trait]
impl Provider for ScriptedProvider {
    fn id(&self) -> &str {
        "scripted"
    }
    async fn complete(&self, _req: ChatRequest) -> OmegaResult<ChatResponse> {
        match self.script.lock().unwrap().pop_front() {
            Some(content) => Ok(ChatResponse {
                message: Message::new(Role::Assistant, content),
                usage: Default::default(),
            }),
            None => Err(OmegaError::Config("script épuisé".to_string())),
        }
    }
}

async fn drive(op: &ToolLoopOperation) -> (Vec<Event>, OmegaResult<()>, RunState) {
    let bus = InMemoryBus::new();
    let (token, _handle) = cancellation_pair();
    let mut state = RunState::default();
    let mut ctx = RunContext {
        bus: &bus,
        token: &token,
        state: &mut state,
    };
    let status = op.run(&mut ctx).await;
    (bus.events(), status, state)
}

fn fs(tmp: &str) -> (std::path::PathBuf, Arc<FsSurface>) {
    let root = std::env::temp_dir().join(format!("omega-integ-{}", tmp));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let ctx = ToolCtx::new(&root);
    (ctx.root.clone(), Arc::new(FsSurface::new(ctx)))
}

fn op(
    provider: Arc<dyn Provider>,
    surface: Arc<FsSurface>,
    messages: Vec<Message>,
) -> ToolLoopOperation {
    ToolLoopOperation {
        provider,
        model: ModelId::new("test-model"),
        surface,
        messages,
        budget: 8,
        policy_budget: None,
        run_id: "run-integration".to_string(),
        trace: None,
    }
}

#[tokio::test]
async fn t1_write_puis_t0_read_canari_en_deux_tours() {
    // Le modèle écrit (T1, canari encadré par la jail) puis relit (T0) ce
    // qu'il vient d'écrire ; l'observation est réinjectée au tour suivant, le
    // 3e tour clôt proprement ("Fait." sans tool-call).
    let (root, surface) = fs("canary");
    let provider = Arc::new(ScriptedProvider::new(vec![
        "{\"tool\":\"write_file\",\"arguments\":{\"path\":\"canari.txt\",\"content\":\"code canari 42\"}}",
        "{\"tool\":\"read_file\",\"arguments\":{\"path\":\"canari.txt\"}}",
        "Fait.",
    ]));
    let messages = vec![Message::new(Role::User, "ecris puis relis le canari")];
    let (events, status, state) = drive(&op(provider, surface.clone(), messages)).await;
    assert!(status.is_ok(), "status d'échec : {:?}", status);

    // Exécution réelle : le canari est sur disque, d'exactement une écriture T1.
    let on_disk = std::fs::read_to_string(root.join("canari.txt")).unwrap();
    assert_eq!(on_disk, "code canari 42");

    // Deux exécutions (write puis read), tracées avec run_id/call_id/origine.
    assert_eq!(state.0["tool_executions"], 2);
    assert_eq!(state.0["status"], "done");
    let completed: Vec<&Event> = events
        .iter()
        .filter(|e| e.name == "tool.call.completed")
        .collect();
    assert_eq!(completed.len(), 2);
    assert_eq!(completed[0].payload["tool"], "write_file");
    assert_eq!(completed[1].payload["tool"], "read_file");
    assert!(completed
        .iter()
        .all(|e| e.payload["run_id"] == "run-integration"
            && e.payload["call_id"].is_string()
            && e.payload["origin"] == "react"));

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn interruption_arrete_la_boucle_avant_execution() {
    // (Cas limite plan §6) annulation utilisateur : le token interrompt la
    // boucle avant le 1er tour — aucune exécution, aucun fichier touché.
    let (root, surface) = fs("cancel");
    let provider = Arc::new(ScriptedProvider::new(vec![
        "{\"tool\":\"write_file\",\"arguments\":{\"path\":\"jamais.txt\",\"content\":\"x\"}}",
    ]));
    let bus = InMemoryBus::new();
    let (token, handle) = cancellation_pair();
    handle.cancel();
    let mut state = RunState::default();
    let messages = vec![Message::new(Role::User, "travaille")];
    let mut ctx = RunContext {
        bus: &bus,
        token: &token,
        state: &mut state,
    };
    let loop_op = ToolLoopOperation {
        provider,
        model: ModelId::new("test-model"),
        surface: surface.clone(),
        messages,
        budget: 8,
        policy_budget: None,
        run_id: "run-cancel".to_string(),
        trace: None,
    };
    let status = loop_op.run(&mut ctx).await;
    assert!(status.is_err());
    assert!(!root.join("jamais.txt").exists());
    let _ = std::fs::remove_dir_all(&root);
}
