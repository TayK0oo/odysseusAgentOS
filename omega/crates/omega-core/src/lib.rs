//! `omega-core` — **boucle d'agent agnostique** du provider et des extensions.
//!
//! Lecon n°3 du reverse-engineering : le noyau est une machine d'`Operation` ordonnees,
//! parametree par des traits (`Inference`, `EffectHandler`, `Emitter`), avec un event-bus
//! interne et une annulation cooperative. Il ne connait ni provider ni extension.
//!
//! L'annulation est verifiee **avant** chaque etape et **pendant** l'execution de
//! chaque etape (`tokio::select!` biased) pour court-circuiter les operations longues.
//!
//! Dependances autorisees : `omega-types` + `omega-ports` uniquement
//! (`OMEGA-BLUEPRINT.md` §2.2).

use async_trait::async_trait;
use omega_types::OmegaResult;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub use omega_types::Event;

/// Bus d'evenements interne au noyau.
pub trait EventBus: Send + Sync {
    fn publish(&self, event: Event);

    /// Journal des evenements publies (utile pour tests et replay).
    fn events(&self) -> Vec<Event>;
}

/// Bus en memoire qui conserve chaque evenement dans un journal.
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

struct CancellationInner {
    flag: AtomicBool,
    notify: Notify,
}

/// Token d'annulation **cooperative**. Verifie avant et pendant chaque etape.
#[derive(Clone)]
pub struct CancellationToken {
    inner: Arc<CancellationInner>,
}

impl CancellationToken {
    fn new(inner: Arc<CancellationInner>) -> Self {
        Self { inner }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.flag.load(Ordering::Relaxed)
    }

    /// Renvoie une erreur si l'annulation a ete demandee.
    pub fn check(&self) -> OmegaResult<()> {
        if self.is_cancelled() {
            Err(omega_types::OmegaError::Cancelled)
        } else {
            Ok(())
        }
    }

    /// Future qui se resout des que l'annulation est demandee.
    pub async fn cancelled(&self) {
        while !self.is_cancelled() {
            self.inner.notify.notified().await;
        }
    }
}

/// Poignee permettant d'annuler un run.
pub struct CancellationHandle {
    inner: Arc<CancellationInner>,
}

impl CancellationHandle {
    pub fn cancel(&self) {
        self.inner.flag.store(true, Ordering::Relaxed);
        self.inner.notify.notify_waiters();
    }
}

/// Cree une paire `(token, handle)` partageant le meme drapeau.
pub fn cancellation_pair() -> (CancellationToken, CancellationHandle) {
    let inner = Arc::new(CancellationInner {
        flag: AtomicBool::new(false),
        notify: Notify::new(),
    });
    (
        CancellationToken::new(inner.clone()),
        CancellationHandle { inner },
    )
}

/// Effet produit par une operation et applique par un `EffectHandler`.
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

/// Handler d'effets. Permet au noyau de rester agnostique des actions concretes.
#[async_trait]
pub trait EffectHandler: Send + Sync {
    async fn apply(&self, effect: &Effect) -> OmegaResult<()>;
}

/// Emetteur asynchrone d'evenements (frontiere live / UI).
#[async_trait]
pub trait Emitter: Send + Sync {
    async fn emit(&self, event: Event) -> OmegaResult<()>;
}

/// Interface d'inference. Le noyau recoit une implementation sans la connaitre.
#[async_trait]
pub trait Inference: Send + Sync {
    async fn infer(&self, input: serde_json::Value) -> OmegaResult<serde_json::Value>;
}

/// Etat partage pendant un run. Le noyau le passe a chaque operation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunState(pub serde_json::Value);

impl RunState {
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }
}

/// Contexte d'execution passe a chaque operation.
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

/// Operation elementaire de la boucle. L'ordre dans `AgentLoop` encode la priorite.
#[async_trait]
pub trait Operation: Send + Sync {
    fn name(&self) -> &str;

    /// Identifiant d'idempotence de l'operation (cle de reprise durable).
    ///
    /// Par defaut, c'est `name()`. Peut etre surcharge (ex. par `omega-app`) pour
    /// garantir l'unicite **par position** quand plusieurs operations partagent le
    /// meme nom : la reprise ne doit jamais sauter silencieusement une operation.
    fn operation_id(&self) -> String {
        self.name().to_string()
    }

    /// Si `true`, une erreur arrête le run avec `RunStatus::Failed`.
    fn is_critical(&self) -> bool {
        false
    }

    async fn run(&self, ctx: &mut RunContext<'_>) -> OmegaResult<()>;
}

/// Boucle d'agent minimale : execute une sequence d'operations ordonnees.
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

    /// Ajoute une operation a la fin de la sequence.
    pub fn push(mut self, op: impl Operation + 'static) -> Self {
        self.operations.push(Box::new(op));
        self
    }

    /// Ajoute une operation deja boxee a la fin de la sequence.
    ///
    /// Utile pour l'assemblage applicatif (`omega-app`) qui manipule des operations
    /// heterogenes derriere `Box<dyn Operation>` (plans de run, reprise).
    pub fn push_boxed(mut self, op: Box<dyn Operation>) -> Self {
        self.operations.push(op);
        self
    }

    /// Si `true`, la premiere operation en echec fait passer le run en `Failed`.
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Execute les operations dans l'ordre jusqu'a terminaison, annulation ou echec critique.
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
            // Annulation cooperative : verifiee **avant** chaque etape.
            if token.check().is_err() {
                bus.publish(Event::new("run.cancelled", json!({ "before": op.name() })));
                return RunStatus::Cancelled;
            }

            bus.publish(Event::new(
                "step.started",
                json!({ "operation": op.operation_id() }),
            ));

            let mut ctx = RunContext {
                bus,
                token,
                state: &mut state,
            };

            let fut = op.run(&mut ctx);
            tokio::pin!(fut);

            // Annulation cooperative : verifiee **pendant** chaque etape.
            let result = tokio::select! {
                biased;
                _ = token.cancelled() => {
                    bus.publish(Event::new("run.cancelled", json!({ "during": op.name() })));
                    return RunStatus::Cancelled;
                }
                r = fut => r,
            };

            match result {
                Ok(()) => {
                    bus.publish(Event::new(
                        "step.completed",
                        json!({ "operation": op.operation_id() }),
                    ));
                }
                Err(omega_types::OmegaError::Cancelled) => {
                    bus.publish(Event::new("run.cancelled", json!({ "during": op.name() })));
                    return RunStatus::Cancelled;
                }
                Err(e) => {
                    let payload = json!({
                        "operation": op.operation_id(),
                        "error": format!("{e:?}"),
                        "critical": op.is_critical() || self.fail_fast,
                    });
                    bus.publish(Event::new("step.failed", payload));

                    if op.is_critical() || self.fail_fast {
                        bus.publish(Event::new("run.failed", json!({ "operation": op.name() })));
                        return RunStatus::Failed;
                    }
                    // Sinon : isolation — on continue avec les operations suivantes.
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
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

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

    struct SleepyOp {
        completed: Arc<AtomicBool>,
    }

    #[async_trait]
    impl Operation for SleepyOp {
        fn name(&self) -> &str {
            "sleepy"
        }

        async fn run(&self, _ctx: &mut RunContext<'_>) -> OmegaResult<()> {
            tokio::time::sleep(Duration::from_secs(60)).await;
            self.completed.store(true, Ordering::Relaxed);
            Ok(())
        }
    }

    #[tokio::test]
    async fn operations_run_in_order() {
        // (i) l'ordre des operations est respecte.
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
        // (ii) annulation cooperative -> Cancelled et aucune op suivante ne s'execute.
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

        // L'evenement d'annulation a ete journalise.
        let events = bus.events();
        let names: Vec<_> = events.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"run.cancelled"));
    }

    #[tokio::test]
    async fn cancellation_interrupts_long_operation() {
        // (§4.6) operation longue annulee en cours par le noyau.
        let bus = InMemoryBus::new();
        let (token, handle) = cancellation_pair();
        let completed = Arc::new(AtomicBool::new(false));

        let loop_ = AgentLoop::new().push(SleepyOp {
            completed: completed.clone(),
        });

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            handle.cancel();
        });

        let status = loop_.run(&bus, &token, RunState::default()).await;
        assert_eq!(status, RunStatus::Cancelled);
        assert!(!completed.load(Ordering::Relaxed));

        let events = bus.events();
        let names: Vec<_> = events.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"run.cancelled"));
    }

    #[tokio::test]
    async fn bus_journals_every_step() {
        // (iii) l'event-bus journalise chaque etape.
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
        // (iv) une operation en echec non critique n'arrete pas le noyau.
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
        // (iv) une operation critique produit Failed proprement.
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

    struct IdOverrideOp(&'static str);

    #[async_trait]
    impl Operation for IdOverrideOp {
        fn name(&self) -> &str {
            self.0
        }

        fn operation_id(&self) -> String {
            format!("{}#0", self.0)
        }

        async fn run(&self, _ctx: &mut RunContext<'_>) -> OmegaResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn step_events_carry_operation_id_not_name() {
        // (B1) `step.started`/`step.completed` portent `operation_id` (pas `name`) :
        // la cle d'idempotence peut etre rendue unique sans changer `name()`.
        let bus = InMemoryBus::new();
        let (token, _handle) = cancellation_pair();
        let loop_ = AgentLoop::new().push(IdOverrideOp("dup"));
        let status = loop_.run(&bus, &token, RunState::default()).await;
        assert_eq!(status, RunStatus::Ok);

        let events = bus.events();
        let started = events.iter().find(|e| e.name == "step.started").unwrap();
        let completed = events.iter().find(|e| e.name == "step.completed").unwrap();
        assert_eq!(started.payload["operation"], "dup#0");
        assert_eq!(completed.payload["operation"], "dup#0");
    }

    #[test]
    fn cancellation_pair_works() {
        let (token, handle) = cancellation_pair();
        assert!(!token.is_cancelled());
        handle.cancel();
        assert!(token.is_cancelled());
        let err = token.check().unwrap_err();
        assert!(matches!(err, omega_types::OmegaError::Cancelled));
    }

    #[test]
    fn dependency_graph_guard_self_test_passes() {
        // (f/B2) le garde-fou de graphe detecte les cas invalides (--self-test).
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let script = std::path::Path::new(&manifest_dir).join("../../scripts/check_dep_graph.py");
        let output = std::process::Command::new("python3")
            .arg(&script)
            .arg("--self-test")
            .current_dir(&manifest_dir)
            .output()
            .expect("le script check_dep_graph.py doit etre executable");

        if !output.status.success() {
            panic!(
                "auto-test du garde-fou en echec :\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
