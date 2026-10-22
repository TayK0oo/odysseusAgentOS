//! Propriété §7-A2 (plan-maître) : budget monotone de la boucle d'outils.
//!
//! Pour toute suite aléatoire de k appels (mélanges : outils valides, outil
//! inconnu, arguments variés — unicode, longs, vides) sous un budget N :
//! le nombre d'exécutions d'outils consommées est EXACTEMENT min(k, N),
//! donc jamais N+1. Budget de la BOUCLE (tours/exécutions posé par
//! l'appelant) — la politique A3 n'entre pas ici (voir header de
//! `tool_loop.rs`).

use async_trait::async_trait;
use omega_core::tool_loop::{ToolCall, ToolLoopOperation, ToolObservation, ToolSurface};
use omega_core::{
    cancellation_pair, Event, EventBus, InMemoryBus, Operation, RunContext, RunState,
};
use omega_ports::Provider;
use omega_types::{ChatRequest, ChatResponse, Message, ModelId, OmegaError, OmegaResult, Role};
use proptest::prelude::*;
use serde_json::json;
use std::sync::{Arc, Mutex};

/// Provider scripté déterministe : le script est fabriqué par réflexion du
/// seed (LCG), chaque tour émet un tool-call (la boucle ne peut pas sortir
/// avant épuisement du script) ; le dernier tour clôt par "Fait.".
struct ScriptedProvider {
    script: Mutex<std::collections::VecDeque<String>>,
}

impl ScriptedProvider {
    fn script_from_seed(seed: u64, k: usize) -> Vec<String> {
        let mut state = seed | 1;
        let mut lcg = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        (0..k)
            .map(|_| {
                let pick = lcg() % 3;
                let arg = match lcg() % 4 {
                    0 => json!({}),
                    1 => json!({"path": "f.txt"}),
                    2 => {
                        // Unicode (précomposé + décomposition NFD invisible dans
                        // les arguments, la jail filtre à l'exécution, pas ici).
                        json!({"path": "café-cafe\u{0301}.txt", "n": lcg() % 50})
                    }
                    _ => {
                        // Longueur variable (jusqu'à ~1200 chars) : sous la garde
                        // de longueur de la jail.
                        let n = (lcg() % 1200) as usize;
                        json!({"long": "x".repeat(n)})
                    }
                };
                let tool = match pick {
                    0 => "read_file",
                    1 => "write_file",
                    _ => "boom", // outil inconnu : trace ok=false, jamais crash
                };
                format!(
                    "{{\"tool\":\"{}\",\"arguments\":{}}}",
                    tool,
                    serde_json::to_string(&arg).unwrap()
                )
            })
            .chain(std::iter::once("Fait.".to_string()))
            .collect()
    }

    fn new(seed: u64, k: usize) -> Self {
        Self {
            script: Mutex::new(Self::script_from_seed(seed, k).into()),
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

/// Surface qui ne fait qu'historiser (l'exécution effective n'importe pas ici).
struct CountingSurface {
    executed: Mutex<usize>,
}

#[async_trait]
impl ToolSurface for CountingSurface {
    async fn execute(&self, call: &ToolCall) -> OmegaResult<ToolObservation> {
        *self.executed.lock().unwrap() += 1;
        Ok(ToolObservation::ok(call.call_id.clone(), json!({})))
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn budget_n_borne_exactement_min_k_n(seed in any::<u64>(), n in 1usize..=32, k in 1usize..=32) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
        let provider = Arc::new(ScriptedProvider::new(seed, k));
        let surface = Arc::new(CountingSurface { executed: Mutex::new(0) });
        let op = ToolLoopOperation {
            provider,
            model: ModelId::new("prop-model"),
            surface: surface.clone(),
            messages: vec![Message::new(Role::User, "travaille")],
            budget: n,
            policy_budget: None,
            run_id: "run-prop".to_string(),
            trace: None,
            store: None,
        };
        let bus = InMemoryBus::new();
        let (token, _handle) = cancellation_pair();
        let mut state = RunState::default();
        let op = op;
        let mut ctx = RunContext { bus: &bus, token: &token, state: &mut state };
        let status = op.run(&mut ctx).await;
        assert!(status.is_ok(), "la boucle doit s'arrêter proprement sous budget : {:?}", status);
        let executed = *surface.executed.lock().unwrap();
        assert_eq!(executed, std::cmp::min(k, n),
            "exécutions consommées != min(k, N) : budget de boucle violé");
        // Tracé au juste prix : un event bus `completed` par exécution.
        let events = bus.events();
        let completed: Vec<&Event> = events.iter()
            .filter(|e| e.name == "tool.call.completed").collect();
        assert_eq!(completed.len(), executed);
        })
    }
}
