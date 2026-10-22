//! Contrat d'acceptation de la boucle d'outils (P1-1, plan §6 ; ADR-0006).
//!
//! Rouge d'abord : ce contrat ne compile pas tant que `omega_core::tool_loop`
//! n'existe pas, puis verrouille les invariants :
//! détection de tool-call (repli ReAct) -> exécution via le port `ToolSurface`
//! -> observation -> multi-tours ; outil inconnu = erreur structurée (jamais
//! de boycott ni de panic) ; budget N = jamais plus de N exécutions.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use omega_core::tool_loop::{ToolCall, ToolLoopOperation, ToolObservation, ToolSurface};
use omega_core::{
    cancellation_pair, Event, EventBus, InMemoryBus, Operation, RunContext, RunState,
};
use omega_ports::{EventStore, Provider};
use omega_types::{
    ChatRequest, ChatResponse, DurableEvent, Message, ModelId, OmegaError, OmegaResult, Role,
};

/// Provider scripté : défile un scénario de contenus assistant, puis échoue.
/// (Même principe que `ScriptedProvider` du banc P0-4 ; voix locale de test.)
struct ScriptedProvider {
    script: Mutex<std::collections::VecDeque<String>>,
    requests: Mutex<Vec<Vec<Message>>>,
}

impl ScriptedProvider {
    fn new(script: Vec<String>) -> Self {
        Self {
            script: Mutex::new(script.into()),
            requests: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Provider for ScriptedProvider {
    fn id(&self) -> &str {
        "scripted"
    }
    async fn complete(&self, req: ChatRequest) -> OmegaResult<ChatResponse> {
        self.requests.lock().unwrap().push(req.messages.clone());
        let next = self.script.lock().unwrap().pop_front();
        match next {
            Some(content) => Ok(ChatResponse {
                message: Message::new(Role::Assistant, content),
                usage: Default::default(),
            }),
            None => Err(OmegaError::Config("script épuisé".to_string())),
        }
    }
}

/// Store de trace réduit : la boucle doit append un DurableEvent par
/// exécution (port `EventStore` ; l'adaptateur réel d'A4 est plus loin).
struct TraceStore {
    events: Mutex<std::collections::VecDeque<DurableEvent>>,
}

#[async_trait]
impl EventStore for TraceStore {
    fn name(&self) -> &str {
        "trace-test"
    }
    fn append(&self, event: DurableEvent) -> OmegaResult<u64> {
        self.events.lock().unwrap().push_back(event.clone());
        Ok(event.seq)
    }
    fn read_run(&self, _run_id: &str) -> OmegaResult<Vec<DurableEvent>> {
        Ok(self.events.lock().unwrap().iter().cloned().collect())
    }
}

/// Surface de test : historise chaque appel, échec structuré pour `boom`.
struct EchoSurface {
    history: Mutex<Vec<String>>, // "call_id|tool"
}

impl EchoSurface {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            history: Mutex::new(Vec::new()),
        })
    }
}

#[async_trait]
impl ToolSurface for EchoSurface {
    async fn execute(&self, call: &ToolCall) -> OmegaResult<ToolObservation> {
        self.history
            .lock()
            .unwrap()
            .push(format!("{}|{}", call.call_id, call.name));
        if call.name == "boom" {
            return Ok(ToolObservation::error(
                call.call_id.clone(),
                "outil inconnu : boom",
            ));
        }
        Ok(ToolObservation::ok(
            call.call_id.clone(),
            serde_json::json!({ "echo": call.name }),
        ))
    }
}

fn op(
    provider: Arc<dyn Provider>,
    surface: Arc<dyn ToolSurface>,
    budget: usize,
) -> ToolLoopOperation {
    ToolLoopOperation {
        provider,
        model: ModelId::new("test-model"),
        surface,
        messages: vec![Message::new(Role::User, "travaille")],
        run_id: "run-contract".to_string(),
        trace: None,
        budget,
        policy_budget: None,
    }
}

async fn drive(op: &mut ToolLoopOperation) -> (Vec<Event>, OmegaResult<()>, RunState) {
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

#[tokio::test]
async fn tool_call_execute_observation_et_multitours() {
    // (P1-1) détection -> exécution -> observation -> multi-tours ; le même
    // outil appelé deux fois reçoit des observations cohérentes (call_id dédié).
    let surface = EchoSurface::new();
    let provider = Arc::new(ScriptedProvider::new(vec![
        "{\"tool\":\"read_file\",\"arguments\":{\"path\":\"a.txt\"}}".to_string(),
        "{\"tool\":\"read_file\",\"arguments\":{\"path\":\"a.txt\"}}".to_string(),
        "Fait.".to_string(),
    ]));
    let mut op = op(provider.clone(), surface.clone(), 8);
    let (events, status, _state) = drive(&mut op).await;
    assert!(status.is_ok());
    assert_eq!(surface.history.lock().unwrap().len(), 2);
    assert_eq!(
        *surface.history.lock().unwrap(),
        vec![
            "call-1|read_file".to_string(),
            "call-2|read_file".to_string()
        ]
    );

    // Le modèle reçoit l'observation de chaque call (role Tool) au tour suivant.
    let requests = provider.requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    let tool_msgs_round2: Vec<&Message> = requests[1]
        .iter()
        .filter(|m| m.role == Role::Tool)
        .collect();
    assert_eq!(tool_msgs_round2.len(), 1);
    assert!(tool_msgs_round2[0].content.contains("call-1"));
    assert!(tool_msgs_round2[0].content.contains("read_file"));
    let obs_round3: Vec<&Message> = requests[2]
        .iter()
        .filter(|m| m.role == Role::Tool)
        .collect();
    assert!(obs_round3.len() == 2); // observations du call-1 puis du call-2

    // Trace : un event completed par call, avec run_id, call_id, origine du contenu.
    let completed: Vec<&Event> = events
        .iter()
        .filter(|e| e.name == "tool.call.completed")
        .collect();
    assert_eq!(completed.len(), 2);
    assert_eq!(completed[0].payload["call_id"], "call-1");
    assert_eq!(completed[1].payload["call_id"], "call-2");
    assert_eq!(completed[0].payload["origin"], "react");
    assert_eq!(completed[1].payload["origin"], "react");
    assert!(completed
        .iter()
        .all(|e| e.payload["run_id"] == "run-contract"));
}

#[tokio::test]
async fn tool_inconnu_est_erreur_structuree_sans_boycott() {
    // (Cas limite §6) outil inventé par le modèle : observation ok=false,
    // la boucle continue (pas de crash, pas de refus global).
    let surface = EchoSurface::new();
    let provider = Arc::new(ScriptedProvider::new(vec![
        "{\"tool\":\"boom\",\"arguments\":{}}".to_string(),
        "Fait.".to_string(),
    ]));
    let mut op = op(provider, surface.clone(), 8);
    let (events, status, _state) = drive(&mut op).await;
    assert!(status.is_ok());
    assert_eq!(surface.history.lock().unwrap().len(), 1);
    let completed: Vec<&Event> = events
        .iter()
        .filter(|e| e.name == "tool.call.completed")
        .collect();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].payload["ok"], false);
    assert_eq!(
        completed[0].payload["content"]["error"],
        "outil inconnu : boom"
    );
}

#[tokio::test]
async fn budget_1_ne_laisse_qu_une_execution() {
    // (§7-A2, cas scalaire) budget N=1 : le modèle demande 3 appels,
    // un seul s'exécute, puis l'arrêt est propre et tracé.
    let surface = EchoSurface::new();
    let provider = Arc::new(ScriptedProvider::new(vec![
        "{\"tool\":\"read_file\",\"arguments\":{}}".to_string(),
        "{\"tool\":\"read_file\",\"arguments\":{}}".to_string(),
        "{\"tool\":\"read_file\",\"arguments\":{}}".to_string(),
        "Fait.".to_string(),
    ]));
    let mut op = op(provider, surface.clone(), 1);
    let (events, status, state) = drive(&mut op).await;
    assert!(status.is_ok());
    assert_eq!(surface.history.lock().unwrap().len(), 1);
    assert_eq!(state.0["tool_executions"], 1);
    assert_eq!(state.0["status"], "budget-exhausted");
    assert!(events.iter().any(|e| e.name == "tool.budget.exhausted"));
}

#[tokio::test]
async fn trace_durable_par_exécution_run_id_call_id_origine() {
    // (P1-4, minimum via port EventStore) après un run avec 1 appel d'outil,
    // le journal durable contient exactement 1 event `tool.call.completed`
    // portant run_id, call_id et l'origine du contenu (ReAct ici) — la base
    // du rejeu de traces d'or (A4).
    let surface = EchoSurface::new();
    let trace = std::sync::Arc::new(TraceStore {
        events: Mutex::new(std::collections::VecDeque::new()),
    });
    let provider = Arc::new(ScriptedProvider::new(vec![
        "{\"tool\":\"read_file\",\"arguments\":{}}".to_string(),
        "Fait.".to_string(),
    ]));
    let mut op = op(provider, surface.clone(), 8);
    op.trace = Some(trace.clone());
    let (_events, status, _state) = drive(&mut op).await;
    assert!(status.is_ok());
    let journal = trace.read_run("run-contract").unwrap();
    assert_eq!(journal.len(), 1);
    let e = &journal[0];
    assert_eq!(e.name, "tool.call.completed");
    assert_eq!(e.run_id, "run-contract");
    assert_eq!(e.payload["call_id"], "call-1");
    assert_eq!(e.payload["origin"], "react");
    assert_eq!(e.payload["ok"], true);
}

#[tokio::test]
async fn input_illisible_ne_panique_pas() {
    // (Cas limite §6) sortie modèle illisible (texte parasite, JSON cassé) :
    // aucune détection, aucune exécution, fin propre.
    let surface = EchoSurface::new();
    let provider = Arc::new(ScriptedProvider::new(vec![
        "voici du texte {pas du json".to_string(),
        "Fait, désolé.".to_string(),
    ]));
    let mut op = op(provider, surface.clone(), 8);
    let (_events, status, state) = drive(&mut op).await;
    assert!(status.is_ok());
    assert_eq!(surface.history.lock().unwrap().len(), 0);
    assert_eq!(state.0["status"], "done");
}
