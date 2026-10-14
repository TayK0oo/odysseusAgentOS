//! `omega-core` — **boucle d'agent agnostique** du provider et des extensions.
//!
//! Leçon n°3 du reverse-engineering : le noyau est une machine d'`Operation` ordonnées,
//! paramétrée par des traits (`Inference`, `EffectHandler`, `Emitter`), avec un event-bus
//! interne et une annulation coopérative. Il ne connaît ni provider ni extension.
//!
//! Dépendances autorisées : `omega-types` + `omega-ports` uniquement
//! (`OMEGA-BLUEPRINT.md` §2.2).

use async_trait::async_trait;
use omega_types::OmegaResult;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Événement émis par le noyau ou par une opération.
///
/// Les événements `durable` sont destinés au journal append-only ; les autres sont
/// éphémères (UI live, métriques).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub payload: serde_json::Value,
    pub durable: bool,
}

impl Event {
    pub fn new(name: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            payload,
            durable: true,
        }
    }

    pub fn ephemeral(name: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            payload,
            durable: false,
        }
    }
}

/// Bus d'événements interne au noyau.
pub trait EventBus: Send + Sync {
    fn publish(&self, event: Event);

    /// Journal des événements publiés (utile pour tests et replay).
    fn events(&self) -> Vec<Event>;
}

/// Bus en mémoire qui conserve chaque événement dans un journal.
#[derive(Default)]
pub struct InMemoryBus {
    journal: Mutex<Vec<Event>>,
}

impl InMemoryBus {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventBus for InMemoryBus {
    fn publish(&self, event: Event) {
        if let Ok(mut journal) = self.journal.lock() {
            journal.push(event);
        }
    }

    fn events(&self) -> Vec<Event> {
        self.journal.lock().map(|j| j.clone()).unwrap_or_default()
    }
}

/// Token d'annulation **coopérative**. Vérifié avant chaque étape de la boucle.
#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new(inner: Arc<AtomicBool>) -> Self {
        Self { inner }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::Relaxed)
    }

    /// Renvoie une erreur si l'annulation a été demandée.
    pub fn check(&self) -> OmegaResult<()> {
        if self.is_cancelled() {
            Err(omega_types::OmegaError::PluginFailed(
                "kernel".to_string(),
                "annulation demandee".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

/// Poignée permettant d'annuler un run.
pub struct CancellationHandle {
    inner: Arc<AtomicBool>,
}

impl CancellationHandle {
    pub fn cancel(&self) {
        self.inner.store(true, Ordering::Relaxed);
    }
}

/// Crée une paire `(token, handle)` partageant le même drapeau.
pub fn cancellation_pair() -> (CancellationToken, CancellationHandle) {
    let inner = Arc::new(AtomicBool::new(false));
    (
        CancellationToken::new(inner.clone()),
        CancellationHandle { inner },
    )
}

/// Effet produit par une opération et appliqué par un `EffectHandler`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    pub name: String,
    pub payload: serde_json::Value,
}

impl Effect {
    pub fn new(name: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            payload,
        }
    }
}

/// Handler d'effets. Permet au noyau de rester agnostique des actions concrètes.
#[async_trait]
pub trait EffectHandler: Send + Sync {
    async fn apply(&self, effect: &Effect) -> OmegaResult<()>;
}

/// Émetteur asynchche d'événements (frontière live / UI).
#[async_trait]
pub trait Emitter: Send + Sync {
    async fn emit(&self, event: Event) -> OmegaResult<()>;
}

/// Interface d'inférence. Le noyau reçoit une implémentation sans la connaître.
#[async_trait]
pub trait Inference: Send + Sync {
    async fn infer(&self, input: serde_json::Value) -> OmegaResult<serde_json::Value>;
}

/// État partagé pendant un run. Le noyau le passe à chaque opération.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunState(pub serde_json::Value);

impl RunState {
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }
}

/// Contexte d'exécution passé à chaque opération.
pub struct RunContext<'a> {
    pub bus: &'a dyn EventBus,
    pub token: &'a CancellationToken,
    pub state: &'a mut RunState,
}

impl<'a> RunContext<'a> {
    pub fn publish(&self, event: Event) {
        self.bus.publish(event);
    }

    pub fn check_cancellation(&self) -> OmegaResult<()> {
        self.token.check()
    }
}

/// Statut final d'un run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatus {
    Ok,
    Failed,
    Cancelled,
}

/// Opération élémentaire de la boucle. L'ordre dans `AgentLoop` encode la priorité.
#[async_trait]
pub trait Operation: Send + Sync {
    fn name(&self) -> &str;

    /// Si `true`, une erreur arrête le run avec `RunStatus::Failed`.
    fn is_critical(&self) -> bool {
        false
    }

    async fn run(&self, ctx: &mut RunContext<'_>) -> OmegaResult<()>;
}

/// Boucle d'agent minimale : exécute une séquence d'opérations ordonnées.
pub struct AgentLoop {
    operations: Vec<Box<dyn Operation>>,
    fail_fast: bool,
}

impl Default for AgentLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentLoop {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            fail_fast: false,
        }
    }

    /// Ajoute une opération à la fin de la séquence.
    pub fn push(mut self, op: impl Operation + 'static) -> Self {
        self.operations.push(Box::new(op));
        self
    }

    /// Si `true`, la première opération en échec fait passer le run en `Failed`.
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Exécute les opérations dans l'ordre jusqu'à terminaison, annulation ou échec critique.
    pub async fn run(
        &self,
        bus: &dyn EventBus,
        token: &CancellationToken,
        initial_state: RunState,
    ) -> RunStatus {
        let mut state = initial_state;
        bus.publish(Event::new(
            "run.started",
            json!({ "operations": self.operations.len() }),
        ));

        for op in &self.operations {
            // Annulation coopérative : vérifiée **avant** chaque étape.
            if token.check().is_err() {
                bus.publish(Event::new("run.cancelled", json!({ "before": op.name() })));
                return RunStatus::Cancelled;
            }

            bus.publish(Event::new(
                "step.started",
                json!({ "operation": op.name() }),
            ));

            let mut ctx = RunContext {
                bus,
                token,
                state: &mut state,
            };

            match op.run(&mut ctx).await {
                Ok(()) => {
                    bus.publish(Event::new(
                        "step.completed",
                        json!({ "operation": op.name() }),
                    ));
                }
                Err(e) => {
                    let payload = json!({
                        "operation": op.name(),
                        "error": format!("{e:?}"),
                        "critical": op.is_critical() || self.fail_fast,
                    });
                    bus.publish(Event::new("step.failed", payload));

                    if op.is_critical() || self.fail_fast {
                        bus.publish(Event::new("run.failed", json!({ "operation": op.name() })));
                        return RunStatus::Failed;
                    }
                    // Sinon : isolation — on continue avec les opérations suivantes.
                }
            }
        }

        bus.publish(Event::new("run.completed", json!(null)));
        RunStatus::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PushOp {
        name: &'static str,
        target: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Operation for PushOp {
        fn name(&self) -> &str {
            self.name
        }

        async fn run(&self, _ctx: &mut RunContext<'_>) -> OmegaResult<()> {
            self.target.lock().unwrap().push(self.name.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn operations_run_in_order() {
        // (i) l'ordre des opérations est respecté.
        let trace = Arc::new(Mutex::new(Vec::new()));
        let bus = InMemoryBus::new();
        let (token, _handle) = cancellation_pair();

        let trace2 = trace.clone();
        let loop_ = AgentLoop::new()
            .push(PushOp {
                name: "a",
                target: trace.clone(),
            })
            .push(PushOp {
                name: "b",
                target: trace.clone(),
            })
            .push(PushOp {
                name: "c",
                target: trace.clone(),
            });

        let status = loop_.run(&bus, &token, RunState::default()).await;
        assert_eq!(status, RunStatus::Ok);
        assert_eq!(
            *trace2.lock().unwrap(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[tokio::test]
    async fn cancellation_stops_before_next_step() {
        // (ii) annulation coopérative → Cancelled et aucune op suivante ne s'exécute.
        let trace = Arc::new(Mutex::new(Vec::new()));
        let bus = InMemoryBus::new();
        let (token, handle) = cancellation_pair();

        struct CancelAfterFirst {
            trace: Arc<Mutex<Vec<String>>>,
            handle: CancellationHandle,
        }

        #[async_trait]
        impl Operation for CancelAfterFirst {
            fn name(&self) -> &str {
                "cancel-trigger"
            }

            async fn run(&self, _ctx: &mut RunContext<'_>) -> OmegaResult<()> {
                self.trace.lock().unwrap().push("first".to_string());
                self.handle.cancel();
                Ok(())
            }
        }

        let loop_ = AgentLoop::new()
            .push(CancelAfterFirst {
                trace: trace.clone(),
                handle,
            })
            .push(PushOp {
                name: "second",
                target: trace.clone(),
            });

        let status = loop_.run(&bus, &token, RunState::default()).await;
        assert_eq!(status, RunStatus::Cancelled);
        assert_eq!(*trace.lock().unwrap(), vec!["first".to_string()]);

        // L'événement d'annulation a été journalisé.
        let events = bus.events();
        let names: Vec<_> = events.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"run.cancelled"));
    }

    #[tokio::test]
    async fn bus_journals_every_step() {
        // (iii) l'event-bus journalise chaque étape.
        let bus = InMemoryBus::new();
        let (token, _handle) = cancellation_pair();
        let loop_ = AgentLoop::new()
            .push(PushOp {
                name: "x",
                target: Arc::new(Mutex::new(Vec::new())),
            })
            .push(PushOp {
                name: "y",
                target: Arc::new(Mutex::new(Vec::new())),
            });

        let status = loop_.run(&bus, &token, RunState::default()).await;
        assert_eq!(status, RunStatus::Ok);

        let events = bus.events();
        let names: Vec<_> = events.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"run.started"));
        assert!(names.contains(&"step.started"));
        assert!(names.contains(&"step.completed"));
        assert!(names.contains(&"run.completed"));
    }

    struct FailOp(&'static str, bool);

    #[async_trait]
    impl Operation for FailOp {
        fn name(&self) -> &str {
            self.0
        }

        fn is_critical(&self) -> bool {
            self.1
        }

        async fn run(&self, _ctx: &mut RunContext<'_>) -> OmegaResult<()> {
            Err(omega_types::OmegaError::PluginFailed(
                self.0.to_string(),
                "boom".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn isolated_failure_continues() {
        // (iv) une opération en échec non critique n'arrête pas le noyau.
        let trace = Arc::new(Mutex::new(Vec::new()));
        let bus = InMemoryBus::new();
        let (token, _handle) = cancellation_pair();

        let loop_ = AgentLoop::new().push(FailOp("fail", false)).push(PushOp {
            name: "after",
            target: trace.clone(),
        });

        let status = loop_.run(&bus, &token, RunState::default()).await;
        assert_eq!(status, RunStatus::Ok);
        assert_eq!(*trace.lock().unwrap(), vec!["after".to_string()]);

        let events = bus.events();
        let names: Vec<_> = events.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"step.failed"));
    }

    #[tokio::test]
    async fn critical_failure_produces_failed() {
        // (iv) une opération critique produit Failed proprement.
        let bus = InMemoryBus::new();
        let (token, _handle) = cancellation_pair();

        let loop_ = AgentLoop::new()
            .push(PushOp {
                name: "before",
                target: Arc::new(Mutex::new(Vec::new())),
            })
            .push(FailOp("fatal", true));

        let status = loop_.run(&bus, &token, RunState::default()).await;
        assert_eq!(status, RunStatus::Failed);

        let events = bus.events();
        let names: Vec<_> = events.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"run.failed"));
    }

    #[test]
    fn cancellation_pair_works() {
        let (token, handle) = cancellation_pair();
        assert!(!token.is_cancelled());
        handle.cancel();
        assert!(token.is_cancelled());
        assert!(token.check().is_err());
    }

    #[test]
    fn dependency_graph_guard_passes() {
        // (f) le garde-fou de graphe de dépendances est exécuté et passe.
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let script = std::path::Path::new(&manifest_dir).join("../../scripts/check_dep_graph.py");
        let output = std::process::Command::new("python3")
            .arg(&script)
            .current_dir(&manifest_dir)
            .output()
            .expect("le script check_dep_graph.py doit etre executable");

        if !output.status.success() {
            panic!(
                "garde-fou graphe en echec :\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
