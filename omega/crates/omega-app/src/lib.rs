//! `omega-app` — **assemblage applicatif** et **exécution durable**.
//!
//! Responsabilités :
//! - composer un [`Registry`] depuis des [`Contributor`]s (`omega-plugin-api::compose`)
//!   avec un [`InstallContext`] (config effective issue d'`omega-config`) ;
//! - résoudre les ports **depuis le registre** (jamais codés en dur) : `Provider`
//!   (capacité `llm-provider`), `ContextEngine` (`context-engine`), `EventStore`
//!   (`event-store`) ;
//! - câbler `omega-core::AgentLoop` avec ces ports et exposer `run(...) -> RunOutcome`
//!   qui **émet les événements durables vers l'`EventStore`** ;
//! - offrir la **reprise idempotente** : un run interrompu reprend sans ré-exécuter les
//!   opérations déjà complétées (identifiant d'opération = `Operation::operation_id()`,
//!   rendu **unique par position** dans le plan).
//!
//! Garantie d'idempotence : une opération est considérée « complétée » si et seulement
//! si le journal durable contient un événement `step.completed` portant son identifiant
//! (`DurableEvent::operation_id`). Une opération dont l'écriture durable n'a pas abouti
//! sera au pire ré-exécutée, jamais silencieusement perdue.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use async_trait::async_trait;
use omega_core::{
    cancellation_pair, AgentLoop, Event, EventBus, Operation, RunContext, RunState, RunStatus,
};
use omega_plugin_api::{compose, Contributor, InstallContext};
use omega_ports::{ContextEngine, EventStore, Provider, SubagentRunner};
use omega_registry::Registry;
use omega_types::{
    Capability, ChatRequest, DurableEvent, Message, ModelId, OmegaError, OmegaResult,
};
use tokio::process::Command as TokioCommand;

pub use runtime_ports::{FileMemoryStore, LocalSubagentRunner, RuntimePortsContributor, SimpleHookRegistry};
mod runtime_ports;
mod store;

pub use store::{is_valid_run_id, FileEventStore, InMemoryEventStore};

/// Application assemblee : registre gele (quarantaine possible a chaud) + config effective.
///
/// `Send + Sync` : utilisable derriere `Arc` dans un runtime async.
pub struct App {
    registry: RwLock<Registry>,
    config: serde_json::Map<String, serde_json::Value>,
}

impl App {
    /// Identifiants actifs (non quarantaines) d'une capacite, par ordre de priorite.
    pub fn active_ids(&self, cap: Capability) -> Vec<String> {
        self.registry
            .read()
            .expect("registre empoisonné")
            .active_ids(cap)
    }

    /// Config effective (fusionnee) disponible aux plugins a l'installation.
    pub fn config(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.config
    }

    /// Vue d'inspection du registre : capacite -> entrees (isolation par plugin).
    pub fn describe(&self) -> Vec<(Capability, Vec<omega_registry::EntryInfo>)> {
        self.registry
            .read()
            .expect("registre empoisonné")
            .describe()
    }

    /// Resout le `Provider` actif (capacite `llm-provider`), quarantaine respectee.
    pub fn provider(&self) -> OmegaResult<Arc<dyn Provider>> {
        self.registry
            .read()
            .expect("registre empoisonné")
            .resolve_dyn::<dyn Provider>(Capability::LlmProvider)
    }

    /// Resout le `ContextEngine` actif (capacite `context-engine`), quarantaine respectee.
    pub fn context_engine(&self) -> OmegaResult<Arc<dyn ContextEngine>> {
        self.registry
            .read()
            .expect("registre empoisonné")
            .resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine)
    }

    /// Resout l'`EventStore` actif (capacite `event-store`), quarantaine respectee.
    pub fn event_store(&self) -> OmegaResult<Arc<dyn EventStore>> {
        self.registry
            .read()
            .expect("registre empoisonné")
            .resolve_dyn::<dyn EventStore>(Capability::EventStore)
    }

    /// Met un plugin en quarantaine (scopee par capacite). Les resolutions suivantes
    /// replient sur l'implementation suivante, jamais sur la quarantainee.
    pub fn quarantine(&self, cap: Capability, id: &str) {
        self.registry
            .write()
            .expect("registre empoisonné")
            .quarantine(cap, id);
    }

    /// Operation canonique : assemble le contexte via le `ContextEngine` resolu.
    pub fn assemble_operation(&self, messages: Vec<Message>) -> OmegaResult<Box<dyn Operation>> {
        Ok(Box::new(AssembleOperation {
            engine: self.context_engine()?,
            messages,
        }))
    }

    /// Operation canonique : infere via le `Provider` resolu (etat assemble ou repli).
    pub fn infer_operation(
        &self,
        model: ModelId,
        fallback_messages: Vec<Message>,
    ) -> OmegaResult<Box<dyn Operation>> {
        Ok(Box::new(InferOperation {
            provider: self.provider()?,
            model,
            fallback_messages,
        }))
    }

    /// Execute un plan d'operations, avec journalisation durable et reprise idempotente.
    ///
    /// - `resume = true` : reconstruit l'ensemble des operations deja completes depuis le
    ///   journal du run et ne re-execute que le reste (run coupe au step k -> reprend au
    ///   step k+1).
    /// - Les evenements durables emis par la boucle (`step.completed`, `run.completed`...)
    ///   sont appends **synchroniquement** a l'`EventStore`, avant l'etape suivante.
    pub async fn run(&self, input: RunInput) -> OmegaResult<RunOutcome> {
        let event_store = self.event_store()?;
        let run_id = input.run_id.unwrap_or_else(generate_run_id);

        // Identifiants d'idempotence du plan courant (uniques par position).
        let all_ids: Vec<String> = input
            .operations
            .iter()
            .enumerate()
            .map(|(index, op)| format!("{}#{}", op.name(), index))
            .collect();

        // Lecture unique du journal : opérations complétées + plan enregistré.
        let journal = event_store.read_run(&run_id)?;
        let completed: HashSet<String> = journal
            .iter()
            .filter(|e| e.name == "step.completed")
            .filter_map(|e| e.operation_id.clone())
            .collect();
        let recorded_plan: Option<Vec<String>> = journal
            .iter()
            .rev()
            .filter(|e| e.name == "plan.defined")
            .find_map(|e| {
                e.payload
                    .get("operations")?
                    .as_array()?
                    .iter()
                    .map(|v| v.as_str().map(str::to_string))
                    .collect::<Option<Vec<String>>>()
            });

        if input.resume {
            // Garde anti-dérive (N1) : reprendre un plan différent sauterait des
            // opérations au contenu différent. Refus typé, jamais de saut silencieux.
            // (Journaux antérieurs sans `plan.defined` : admis sans contrôle.)
            if let Some(recorded) = recorded_plan {
                if recorded != all_ids {
                    return Err(OmegaError::Config(format!(
                        "reprise impossible : le plan a changé (run {run_id})"
                    )));
                }
            }
        } else if recorded_plan.is_none() {
            // Premier run : fige le plan dans le journal (référence des reprises).
            event_store.append(DurableEvent::new(
                &run_id,
                "plan.defined",
                serde_json::json!({"operations": all_ids}),
            ))?;
        }

        let mut skipped = 0usize;
        let mut pending: Vec<Box<dyn Operation>> = Vec::with_capacity(input.operations.len());
        for (index, op) in input.operations.into_iter().enumerate() {
            // Identifiant d'idempotence **unique par position** : deux operations de meme
            // `name()` (ex. deux « infer ») sont distinctes et ne sont jamais confondues
            // a la reprise (pas de saut silencieux).
            let id = format!("{}#{}", op.name(), index);
            if input.resume && completed.contains(&id) {
                skipped += 1;
            } else {
                pending.push(Box::new(TaggedOperation { id, inner: op }));
            }
        }
        let executed = pending.len();

        let mut agent_loop = AgentLoop::new();
        for op in pending {
            agent_loop = agent_loop.push_boxed(op);
        }

        let bus = StoreBackedBus::new(event_store, run_id.clone());
        let (token, _handle) = cancellation_pair();
        let status = agent_loop.run(&bus, &token, RunState::default()).await;

        if let Some(err) = bus.take_error() {
            return Err(err);
        }

        Ok(RunOutcome {
            run_id,
            status,
            skipped_operations: skipped,
            executed_operations: executed,
        })
    }
}

/// Plan d'execution d'un run.
pub struct RunInput {
    /// Identifiant de run (genere si absent). Explicite recommande pour la reprise.
    pub run_id: Option<String>,
    /// Si vrai, reprend depuis le journal durable (idempotence par identifiant d'operation).
    pub resume: bool,
    /// Operations ordonnees (l'ordre encode la priorite). L'identifiant d'idempotence
    /// est rendu **unique par position** (`name#index`) : deux operations de meme nom
    /// sont distinctes et chacune s'execute exactement une fois.
    pub operations: Vec<Box<dyn Operation>>,
}

impl RunInput {
    pub fn new(operations: Vec<Box<dyn Operation>>) -> Self {
        Self {
            run_id: None,
            resume: false,
            operations,
        }
    }

    pub fn with_run_id(mut self, id: impl Into<String>) -> Self {
        self.run_id = Some(id.into());
        self
    }

    /// Active la reprise depuis le journal durable.
    pub fn resume(mut self) -> Self {
        self.resume = true;
        self
    }
}

/// Resultat d'un run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub run_id: String,
    pub status: RunStatus,
    /// Operations deja completes, sautees (reprise).
    pub skipped_operations: usize,
    /// Operations executees dans ce run.
    pub executed_operations: usize,
}

/// Constructeur d'`App` : accumulate des contributeurs et une config effective.
#[derive(Default)]
pub struct AppBuilder {
    contributors: Vec<Box<dyn Contributor>>,
    ctx: InstallContext,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un contributeur (plugin qui enregistre des capacites au build).
    pub fn add_contributor(mut self, contributor: Box<dyn Contributor>) -> Self {
        self.contributors.push(contributor);
        self
    }

    /// Opt-in : ajoute les adaptateurs runtime (MemoryStore fichier JSONL,
    /// Hook en memoire, SubagentRunner local tokio).
    pub fn with_runtime_ports(self, memory_path: std::path::PathBuf) -> OmegaResult<Self> {
        Ok(self.add_contributor(Box::new(RuntimePortsContributor::new(memory_path)?)))
    }

    /// Installe le contexte (config effective + metadonnees) passe aux contributeurs.
    pub fn with_install_context(mut self, ctx: InstallContext) -> Self {
        self.ctx = ctx;
        self
    }

    /// Construit le contexte d'installation depuis une `LayeredConfig` fusionnee.
    pub fn with_config(mut self, config: omega_config::LayeredConfig) -> Self {
        self.ctx = InstallContext {
            config: config.merged(),
        };
        self
    }

    /// Compose le registre, gele, et valide la presence des capacites requises
    /// (`llm-provider` + `context-engine`) avec une erreur typee si absente.
    pub fn build(self) -> OmegaResult<App> {
        let refs: Vec<&dyn Contributor> = self.contributors.iter().map(|b| b.as_ref()).collect();
        let registry = compose(&refs, &self.ctx)?;
        let app = App {
            registry: RwLock::new(registry),
            config: self.ctx.config,
        };
        // Validation anticipee : erreur claire si capacite absente (NoImplementation)
        // ou si le type enregistre ne correspond pas au port (TypeMismatch).
        app.provider()?;
        app.context_engine()?;
        Ok(app)
    }
}

static RUN_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_run_id() -> String {
    format!("run-{}", RUN_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Pont entre l'event-bus du noyau et l'`EventStore` durable.
///
/// A chaque `publish` d'un evenement durable, l'evenement est append **synchroniquement**
/// au store : l'ordre et la durabilite sont garantis avant l'etape suivante. La premiere
/// erreur d'append est retenue puis remontee en fin de run.
struct StoreBackedBus {
    store: Arc<dyn EventStore>,
    run_id: String,
    journal: Mutex<Vec<Event>>,
    error: Mutex<Option<OmegaError>>,
}

impl StoreBackedBus {
    fn new(store: Arc<dyn EventStore>, run_id: String) -> Self {
        Self {
            store,
            run_id,
            journal: Mutex::new(Vec::new()),
            error: Mutex::new(None),
        }
    }

    fn take_error(&self) -> Option<OmegaError> {
        self.error.lock().expect("journal empoisonné").take()
    }
}

impl EventBus for StoreBackedBus {
    fn publish(&self, event: Event) {
        self.journal
            .lock()
            .expect("journal empoisonné")
            .push(event.clone());
        if !event.durable {
            return;
        }
        let durable = to_durable_event(&self.run_id, &event);
        if let Err(err) = self.store.append(durable) {
            let mut guard = self.error.lock().expect("journal empoisonné");
            if guard.is_none() {
                *guard = Some(err);
            }
        }
    }

    fn events(&self) -> Vec<Event> {
        self.journal.lock().expect("journal empoisonné").clone()
    }
}

/// Convertit un evenement du noyau en evenement durable, en extrayant l'identifiant
/// d'operation des evenements d'etape (`step.completed` / `step.failed`).
fn to_durable_event(run_id: &str, event: &Event) -> DurableEvent {
    let operation_id = match event.name.as_str() {
        "step.completed" | "step.failed" => event
            .payload
            .get("operation")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        _ => None,
    };
    DurableEvent::new(run_id, event.name.clone(), event.payload.clone())
        .with_operation_id(operation_id)
}

/// Enveloppe une operation avec un identifiant d'idempotence **unique par position**.
///
/// Deux operations de meme `name()` (ex. deux « infer ») recoivent des ids distincts
/// (`name#0`, `name#1`...) : la reprise ne saute plus silencieusement une operation
/// homonyme. `name()` et `operation_id()` renvoient cet id unique ; `run` est delegue
/// a l'operation interne (y compris `is_critical`).
struct TaggedOperation {
    id: String,
    inner: Box<dyn Operation>,
}

#[async_trait]
impl Operation for TaggedOperation {
    fn name(&self) -> &str {
        &self.id
    }

    fn is_critical(&self) -> bool {
        self.inner.is_critical()
    }

    async fn run(&self, ctx: &mut RunContext<'_>) -> OmegaResult<()> {
        self.inner.run(ctx).await
    }
}

/// Operation canonique : assemble le contexte via le `ContextEngine` resolu.
struct AssembleOperation {
    engine: Arc<dyn ContextEngine>,
    messages: Vec<Message>,
}

#[async_trait]
impl Operation for AssembleOperation {
    fn name(&self) -> &str {
        "assemble"
    }

    async fn run(&self, ctx: &mut RunContext<'_>) -> OmegaResult<()> {
        let assembled = self.engine.assemble(self.messages.clone()).await?;
        ctx.state.0 = serde_json::to_value(assembled)
            .map_err(|e| OmegaError::Config(format!("sérialisation contexte : {e}")))?;
        Ok(())
    }
}

/// Operation canonique : infere via le `Provider` resolu (etat assemble ou repli).
struct InferOperation {
    provider: Arc<dyn Provider>,
    model: ModelId,
    fallback_messages: Vec<Message>,
}

#[async_trait]
impl Operation for InferOperation {
    fn name(&self) -> &str {
        "infer"
    }

    async fn run(&self, ctx: &mut RunContext<'_>) -> OmegaResult<()> {
        let messages: Vec<Message> = serde_json::from_value(ctx.state.0.clone())
            .unwrap_or_else(|_| self.fallback_messages.clone());
        let response = self
            .provider
            .complete(ChatRequest {
                model: self.model.clone(),
                messages,
            })
            .await?;
        // Réponse journalisée (event-sourced) : la sortie du modèle vit dans le
        // journal durable, pas seulement dans l'état — l'UI la relit sans
        // ré-exécuter l'inférence (traçabilité, W8).
        ctx.publish(Event::new(
            "infer.completed",
            serde_json::json!({
                "model": self.model.0.clone(),
                "role": format!("{:?}", response.message.role).to_lowercase(),
                "content": response.message.content.clone(),
                "input_tokens": response.usage.input_tokens,
                "output_tokens": response.usage.output_tokens,
            }),
        ));
        ctx.state.0 = serde_json::to_value(response)
            .map_err(|e| OmegaError::Config(format!("sérialisation réponse : {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{AppBuilder, InMemoryEventStore, RunInput};
    use async_trait::async_trait;
    use omega_core::{Operation, RunContext, RunStatus};
    use omega_plugin_api::{Contributor, InstallContext};
    use omega_ports::{ContextEngine, EventStore, Hook, MemoryStore, Provider, SubagentRunner};
    use omega_registry::{Manifest, RegistryBuilder};
    use omega_types::{
        Capability, ChatRequest, ChatResponse, DurableEvent, Message, ModelId, OmegaError,
        OmegaResult, Role,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering};
    use std::sync::{Arc, Mutex};

    // --- Stubs (Provider / ContextEngine) -----------------------------------

    struct StubProvider {
        id: &'static str,
        calls: AtomicUsize,
    }

    impl StubProvider {
        fn new(id: &'static str) -> Self {
            Self {
                id,
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl Provider for StubProvider {
        fn id(&self) -> &str {
            self.id
        }

        async fn complete(&self, _req: ChatRequest) -> OmegaResult<ChatResponse> {
            self.calls.fetch_add(1, AtomicOrdering::SeqCst);
            Ok(ChatResponse {
                message: Message::new(Role::Assistant, "stub"),
                usage: Default::default(),
            })
        }
    }

    struct StubEngine {
        name: &'static str,
        calls: AtomicUsize,
    }

    impl StubEngine {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl ContextEngine for StubEngine {
        fn name(&self) -> &str {
            self.name
        }

        fn owns_compaction(&self) -> bool {
            false
        }

        async fn assemble(&self, messages: Vec<Message>) -> OmegaResult<Vec<Message>> {
            self.calls.fetch_add(1, AtomicOrdering::SeqCst);
            Ok(messages)
        }
    }

    // --- Contributeurs de test ----------------------------------------------

    struct ProviderContributor {
        manifest: Manifest,
        provider: Arc<dyn Provider>,
    }

    impl Contributor for ProviderContributor {
        fn name(&self) -> &str {
            &self.manifest.id
        }

        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder.register_dyn(self.manifest.clone(), self.provider.clone()).map(|_| ())
        }
    }

    struct EngineContributor {
        manifest: Manifest,
        engine: Arc<dyn ContextEngine>,
    }

    impl Contributor for EngineContributor {
        fn name(&self) -> &str {
            &self.manifest.id
        }

        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder.register_dyn(self.manifest.clone(), self.engine.clone()).map(|_| ())
        }
    }

    struct StoreContributor {
        store: Arc<dyn EventStore>,
    }

    impl Contributor for StoreContributor {
        fn name(&self) -> &str {
            "store"
        }

        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder.register_dyn(
                Manifest::new("store", Capability::EventStore, "1"),
                self.store.clone(),
            )
            .map(|_| ())
        }
    }

    // --- Adaptateurs des ports orphelins (NOYAU Phase B) ---------------------

    /// MemoryStore simple : stockage en mémoire + fichier JSONL pour persistance.
    struct FileMemoryStore {
        path: std::path::PathBuf,
        facts: Mutex<Vec<String>>,
    }

    impl FileMemoryStore {
        fn new(path: impl Into<std::path::PathBuf>) -> OmegaResult<Self> {
            let path = path.into();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| OmegaError::Io(format!("création répertoire : {e}")))?;
            }
            let mut facts = Vec::new();
            if path.is_file() {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| OmegaError::Io(format!("lecture fichier : {e}")))?;
                for line in content.lines() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(s) = v.as_str() {
                            facts.push(s.to_string());
                        }
                    }
                }
            }
            Ok(Self {
                path,
                facts: Mutex::new(facts),
            })
        }
    }

    #[async_trait]
    impl MemoryStore for FileMemoryStore {
        fn name(&self) -> &str {
            "file-memory"
        }
        async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>> {
            let facts = self.facts.lock().unwrap();
            let results: Vec<String> = facts
                .iter()
                .filter(|f| f.contains(query))
                .take(limit)
                .cloned()
                .collect();
            Ok(results)
        }
        async fn remember(&self, fact: &str) -> OmegaResult<()> {
            self.facts.lock().unwrap().push(fact.to_string());
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
                .map_err(|e| OmegaError::Io(format!("ouverture fichier : {e}")))?;
            use std::io::Write;
            writeln!(file, "{}", serde_json::to_string(fact).unwrap())
                .map_err(|e| OmegaError::Io(format!("écriture fichier : {e}")))?;
            Ok(())
        }
    }

    struct MemoryContributor {
        store: Arc<dyn MemoryStore>,
    }

    impl Contributor for MemoryContributor {
        fn name(&self) -> &str {
            "file-memory"
        }
        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder
                .register_dyn(
                    Manifest::new("file", Capability::MemoryStore, "1"),
                    self.store.clone(),
                )
                .map(|_| ())
        }
    }

    /// Hook simple : registre en mémoire des hooks.
    struct SimpleHookRegistry {
        hooks: Mutex<Vec<Box<dyn Hook>>>,
    }

    impl SimpleHookRegistry {
        fn new() -> Self {
            Self {
                hooks: Mutex::new(Vec::new()),
            }
        }
        fn add(&self, hook: Box<dyn Hook>) {
            self.hooks.lock().unwrap().push(hook);
        }
    }

    #[async_trait]
    impl Hook for SimpleHookRegistry {
        fn name(&self) -> &str {
            "hook-registry"
        }
    }

    struct HookContributor {
        registry: Arc<SimpleHookRegistry>,
    }

    impl Contributor for HookContributor {
        fn name(&self) -> &str {
            "hook-registry"
        }
        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder
                .register_dyn(
                    Manifest::new("registry", Capability::Hook, "1"),
                    self.registry.clone(),
                )
                .map(|_| ())
        }
    }

    /// SubagentRunner local : spawn via tokio.
    struct LocalSubagentRunner {
        running: Mutex<Vec<tokio::task::JoinHandle<OmegaResult<String>>>>,
    }

    impl LocalSubagentRunner {
        fn new() -> Self {
            Self {
                running: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl SubagentRunner for LocalSubagentRunner {
        fn name(&self) -> &str {
            "local-subagents"
        }
        async fn spawn(&self, spec: &str) -> OmegaResult<String> {
            let spec = spec.to_string();
            let handle = tokio::spawn(async move {
                // Phase 2 : exécution locale simple (simule un sous-agent)
                // En Phase D, ce sera remplacé par un provider externe (Claude Code/Codex/ACP)
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                Ok(format!("subagent:{}", spec))
            });
            self.running.lock().unwrap().push(handle);
            Ok("spawned".to_string())
        }
    }

    struct SubagentContributor {
        runner: Arc<dyn SubagentRunner>,
    }

    impl Contributor for SubagentContributor {
        fn name(&self) -> &str {
            "local-subagents"
        }
        fn contribute(
            &self,
            builder: &mut RegistryBuilder,
            _ctx: &InstallContext,
        ) -> OmegaResult<()> {
            builder
                .register_dyn(
                    Manifest::new("local", Capability::SubagentRunner, "1"),
                    self.runner.clone(),
                )
                .map(|_| ())
        }
    }

    // --- Operations de test (trace d'achevement, crash une fois) -------------

    struct CountingOp {
        id: &'static str,
        trace: Arc<Mutex<Vec<&'static str>>>,
    }

    #[async_trait]
    impl Operation for CountingOp {
        fn name(&self) -> &str {
            self.id
        }

        async fn run(&self, _ctx: &mut RunContext<'_>) -> OmegaResult<()> {
            self.trace.lock().unwrap().push(self.id);
            Ok(())
        }
    }

    /// Operation critique qui echoue une fois (crash simule) puis reussit a la reprise.
    /// Elle ne marque l'achevement (trace) que lorsqu'elle termine reellement.
    struct CrashOnce {
        id: &'static str,
        trace: Arc<Mutex<Vec<&'static str>>>,
        crashed: Arc<AtomicBool>,
    }

    #[async_trait]
    impl Operation for CrashOnce {
        fn name(&self) -> &str {
            self.id
        }

        fn is_critical(&self) -> bool {
            true
        }

        async fn run(&self, _ctx: &mut RunContext<'_>) -> OmegaResult<()> {
            if self.crashed.swap(true, AtomicOrdering::SeqCst) {
                self.trace.lock().unwrap().push(self.id);
                Ok(())
            } else {
                Err(OmegaError::PluginFailed(
                    self.id.to_string(),
                    "crash simulé".to_string(),
                ))
            }
        }
    }

    fn crash_plan(
        trace: Arc<Mutex<Vec<&'static str>>>,
        crashed: Arc<AtomicBool>,
    ) -> Vec<Box<dyn Operation>> {
        vec![
            Box::new(CountingOp {
                id: "op1",
                trace: trace.clone(),
            }),
            Box::new(CountingOp {
                id: "op2",
                trace: trace.clone(),
            }),
            Box::new(CrashOnce {
                id: "op3",
                trace: trace.clone(),
                crashed,
            }),
            Box::new(CountingOp {
                id: "op4",
                trace: trace.clone(),
            }),
            Box::new(CountingOp {
                id: "op5",
                trace: trace.clone(),
            }),
        ]
    }

    fn base_app(store: Arc<dyn EventStore>) -> crate::App {
        AppBuilder::new()
            .add_contributor(Box::new(ProviderContributor {
                manifest: Manifest::new("p", Capability::LlmProvider, "1"),
                provider: Arc::new(StubProvider::new("p")),
            }))
            .add_contributor(Box::new(EngineContributor {
                manifest: Manifest::new("e", Capability::ContextEngine, "1"),
                engine: Arc::new(StubEngine::new("e")),
            }))
            .add_contributor(Box::new(StoreContributor { store }))
            .build()
            .unwrap()
    }

    // --- Tests d'acceptation -------------------------------------------------

    #[test]
    fn composes_registry_from_two_contributors_and_resolves_ports() {
        // (a) 2 contributeurs + manifestes -> resolution provider / context-engine.
        let app = AppBuilder::new()
            .add_contributor(Box::new(ProviderContributor {
                manifest: Manifest::new("stub-provider", Capability::LlmProvider, "1"),
                provider: Arc::new(StubProvider::new("stub-provider")),
            }))
            .add_contributor(Box::new(EngineContributor {
                manifest: Manifest::new("stub-engine", Capability::ContextEngine, "1"),
                engine: Arc::new(StubEngine::new("stub-engine")),
            }))
            .build()
            .unwrap();

        assert_eq!(app.provider().unwrap().id(), "stub-provider");
        assert_eq!(app.context_engine().unwrap().name(), "stub-engine");
        assert_eq!(
            app.active_ids(Capability::LlmProvider),
            vec!["stub-provider".to_string()]
        );
        assert_eq!(
            app.active_ids(Capability::ContextEngine),
            vec!["stub-engine".to_string()]
        );
    }

    #[tokio::test]
    async fn run_writes_durable_journal_with_stubs() {
        // (b) run complet avec stubs -> journal ecrit + cablage effectif des ports.
        let provider = Arc::new(StubProvider::new("p"));
        let engine = Arc::new(StubEngine::new("e"));
        let store = Arc::new(InMemoryEventStore::new());

        let app = AppBuilder::new()
            .add_contributor(Box::new(ProviderContributor {
                manifest: Manifest::new("p", Capability::LlmProvider, "1"),
                provider: provider.clone(),
            }))
            .add_contributor(Box::new(EngineContributor {
                manifest: Manifest::new("e", Capability::ContextEngine, "1"),
                engine: engine.clone(),
            }))
            .add_contributor(Box::new(StoreContributor {
                store: store.clone(),
            }))
            .build()
            .unwrap();

        let messages = vec![Message::new(Role::User, "bonjour")];
        let ops: Vec<Box<dyn Operation>> = vec![
            app.assemble_operation(messages.clone()).unwrap(),
            app.infer_operation(ModelId::new("m"), messages).unwrap(),
        ];

        let outcome = app
            .run(RunInput::new(ops).with_run_id("run-b"))
            .await
            .unwrap();

        assert_eq!(outcome.status, RunStatus::Ok);
        assert_eq!(outcome.executed_operations, 2);
        assert_eq!(outcome.skipped_operations, 0);

        // Les deux stubs ont ete appeles exactement une fois (cablage effectif).
        assert_eq!(engine.calls.load(AtomicOrdering::SeqCst), 1);
        assert_eq!(provider.calls.load(AtomicOrdering::SeqCst), 1);

        // Journal durable ecrit, avec marqueurs d'achevement par identifiant d'operation.
        let journal = store.read_run("run-b").unwrap();
        let names: Vec<&str> = journal.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"run.started"));
        assert!(names.contains(&"step.completed"));
        assert!(names.contains(&"run.completed"));

        let completed: Vec<&str> = journal
            .iter()
            .filter(|e| e.name == "step.completed")
            .filter_map(|e| e.operation_id.as_deref())
            .collect();
        assert_eq!(completed, vec!["assemble#0", "infer#1"]);
    }

    #[tokio::test]
    async fn crash_after_k_steps_resumes_without_reexecuting() {
        // (c) crash simule apres 2 etapes -> reprise : op1/op2 NON re-executes, fin Ok.
        let store = Arc::new(InMemoryEventStore::new());
        let app = base_app(store.clone());

        let trace = Arc::new(Mutex::new(Vec::new()));
        let crashed = Arc::new(AtomicBool::new(false));

        // Premier run : op1, op2 completes ; op3 critique -> Failed.
        let first = app
            .run(RunInput::new(crash_plan(trace.clone(), crashed.clone())).with_run_id("run-c"))
            .await
            .unwrap();
        assert_eq!(first.status, RunStatus::Failed);

        // Reprise : saute op1/op2, termine op3/op4/op5 -> Ok.
        let second = app
            .run(
                RunInput::new(crash_plan(trace.clone(), crashed.clone()))
                    .with_run_id("run-c")
                    .resume(),
            )
            .await
            .unwrap();
        assert_eq!(second.status, RunStatus::Ok);
        assert_eq!(second.skipped_operations, 2);
        assert_eq!(second.executed_operations, 3);

        // Chaque operation achevee exactement une fois (op1/op2 non re-executees).
        assert_eq!(
            *trace.lock().unwrap(),
            vec!["op1", "op2", "op3", "op4", "op5"]
        );
    }

    #[test]
    fn missing_capability_is_typed_error() {
        // (d) provider absent -> NoImplementation(LlmProvider) des le build.
        let build_res = AppBuilder::new()
            .add_contributor(Box::new(EngineContributor {
                manifest: Manifest::new("e", Capability::ContextEngine, "1"),
                engine: Arc::new(StubEngine::new("e")),
            }))
            .build();
        let err = match build_res {
            Ok(_) => panic!("build devait echouer : provider absent"),
            Err(e) => e,
        };
        assert!(matches!(
            err,
            OmegaError::NoImplementation(Capability::LlmProvider)
        ));

        // provider + context-engine presents mais event-store absent : build ok, run KO.
        let app = AppBuilder::new()
            .add_contributor(Box::new(ProviderContributor {
                manifest: Manifest::new("p", Capability::LlmProvider, "1"),
                provider: Arc::new(StubProvider::new("p")),
            }))
            .add_contributor(Box::new(EngineContributor {
                manifest: Manifest::new("e", Capability::ContextEngine, "1"),
                engine: Arc::new(StubEngine::new("e")),
            }))
            .build()
            .unwrap();
        let err = match app.event_store() {
            Ok(_) => panic!("event-store absent : attendu NoImplementation"),
            Err(e) => e,
        };
        assert!(matches!(
            err,
            OmegaError::NoImplementation(Capability::EventStore)
        ));
    }

    #[tokio::test]
    async fn resume_is_deterministic() {
        // (e) la reprise est deterministe : reprise repetee -> meme resultat, sans re-exec.
        let store = Arc::new(InMemoryEventStore::new());
        let app = base_app(store.clone());

        let trace = Arc::new(Mutex::new(Vec::new()));
        let plan = || -> Vec<Box<dyn Operation>> {
            vec![
                Box::new(CountingOp {
                    id: "a",
                    trace: trace.clone(),
                }),
                Box::new(CountingOp {
                    id: "b",
                    trace: trace.clone(),
                }),
            ]
        };

        let first = app
            .run(RunInput::new(plan()).with_run_id("run-e"))
            .await
            .unwrap();
        assert_eq!(first.status, RunStatus::Ok);
        assert_eq!(first.executed_operations, 2);

        let second = app
            .run(RunInput::new(plan()).with_run_id("run-e").resume())
            .await
            .unwrap();
        assert_eq!(second.status, RunStatus::Ok);
        assert_eq!(second.skipped_operations, 2);
        assert_eq!(second.executed_operations, 0);

        let third = app
            .run(RunInput::new(plan()).with_run_id("run-e").resume())
            .await
            .unwrap();
        assert_eq!(third.status, RunStatus::Ok);
        assert_eq!(third.skipped_operations, 2);
        assert_eq!(third.executed_operations, 0);

        // Chaque operation executee exactement une fois au total.
        assert_eq!(*trace.lock().unwrap(), vec!["a", "b"]);
    }

    #[tokio::test]
    async fn resume_rejects_changed_plan() {
        // (N1) run [a, b] complet, puis reprise avec un plan différent [a, c] :
        // refus typé, jamais de saut silencieux de `c`.
        let store = Arc::new(InMemoryEventStore::new());
        let app = base_app(store.clone());
        let trace = Arc::new(Mutex::new(Vec::new()));
        let plan_ab = || -> Vec<Box<dyn Operation>> {
            vec![
                Box::new(CountingOp {
                    id: "a",
                    trace: trace.clone(),
                }),
                Box::new(CountingOp {
                    id: "b",
                    trace: trace.clone(),
                }),
            ]
        };
        let first = app
            .run(RunInput::new(plan_ab()).with_run_id("run-plan"))
            .await
            .unwrap();
        assert_eq!(first.status, RunStatus::Ok);

        let plan_ac = || -> Vec<Box<dyn Operation>> {
            vec![
                Box::new(CountingOp {
                    id: "a",
                    trace: trace.clone(),
                }),
                Box::new(CountingOp {
                    id: "c",
                    trace: trace.clone(),
                }),
            ]
        };
        let err = app
            .run(RunInput::new(plan_ac()).with_run_id("run-plan").resume())
            .await
            .unwrap_err();
        assert!(matches!(err, OmegaError::Config(_)));
        // `c` ne s'est jamais exécutée, `a`/`b` une fois chacun.
        assert_eq!(*trace.lock().unwrap(), vec!["a", "b"]);
    }

    #[tokio::test]
    async fn resume_without_recorded_plan_stays_allowed() {
        // (N1) journaux antérieurs sans `plan.defined` : reprise admise sans contrôle.
        let store = Arc::new(InMemoryEventStore::new());
        store
            .append(
                DurableEvent::new("legacy", "step.completed", serde_json::json!(null))
                    .with_operation_id(Some("a#0".to_string())),
            )
            .unwrap();
        let app = base_app(store.clone());
        let trace = Arc::new(Mutex::new(Vec::new()));
        let out = app
            .run(
                RunInput::new(vec![Box::new(CountingOp {
                    id: "a",
                    trace: trace.clone(),
                })])
                .with_run_id("legacy")
                .resume(),
            )
            .await
            .unwrap();
        assert_eq!(out.status, RunStatus::Ok);
        assert_eq!(out.skipped_operations, 1);
        assert!(trace.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn duplicate_operation_names_each_run_once_and_resume() {
        // (B1) deux operations de meme `name()` -> chacune s'execute exactement une fois ;
        // la reprise ne re-execute ni ne saute silencieusement aucune des deux.
        let store = Arc::new(InMemoryEventStore::new());
        let app = base_app(store.clone());

        let trace = Arc::new(Mutex::new(Vec::new()));
        let plan = || -> Vec<Box<dyn Operation>> {
            vec![
                Box::new(CountingOp {
                    id: "dup",
                    trace: trace.clone(),
                }),
                Box::new(CountingOp {
                    id: "dup",
                    trace: trace.clone(),
                }),
            ]
        };

        let first = app
            .run(RunInput::new(plan()).with_run_id("run-dup"))
            .await
            .unwrap();
        assert_eq!(first.status, RunStatus::Ok);
        assert_eq!(first.executed_operations, 2);
        assert_eq!(first.skipped_operations, 0);

        let second = app
            .run(RunInput::new(plan()).with_run_id("run-dup").resume())
            .await
            .unwrap();
        assert_eq!(second.status, RunStatus::Ok);
        assert_eq!(second.skipped_operations, 2);
        assert_eq!(second.executed_operations, 0);

        // Les deux operations homonymes ont chacune ete executee exactement une fois.
        assert_eq!(*trace.lock().unwrap(), vec!["dup", "dup"]);
    }

    #[test]
    fn respects_quarantine() {
        // La resolution passe par le registre : quarantaine -> repli sur la suivante.
        let app = AppBuilder::new()
            .add_contributor(Box::new(ProviderContributor {
                manifest: Manifest::new("a", Capability::LlmProvider, "1").with_priority(10),
                provider: Arc::new(StubProvider::new("a")),
            }))
            .add_contributor(Box::new(ProviderContributor {
                manifest: Manifest::new("b", Capability::LlmProvider, "1").with_priority(5),
                provider: Arc::new(StubProvider::new("b")),
            }))
            .add_contributor(Box::new(EngineContributor {
                manifest: Manifest::new("e", Capability::ContextEngine, "1"),
                engine: Arc::new(StubEngine::new("e")),
            }))
            .build()
            .unwrap();

        assert_eq!(app.provider().unwrap().id(), "a");

        app.quarantine(Capability::LlmProvider, "a");
        assert_eq!(app.provider().unwrap().id(), "b");
        assert_eq!(
            app.active_ids(Capability::LlmProvider),
            vec!["b".to_string()]
        );
    }
}

// --- Provider externe de sous-agents (NOYAU Phase D) --------------------------

use std::process::Stdio;

/// Configuration pour un provider externe de sous-agents.
#[derive(Debug, Clone)]
pub struct ExternalSubagentConfig {
    /// Commande de base (ex. "claude", "codex", "acp")
    pub command: String,
    /// Arguments fixes (ex. ["--print", "--model", "sonnet"])
    pub args: Vec<String>,
    /// Variables d'environnement à définir
    pub env: Vec<(String, String)>,
    /// Timeout par défaut
    pub timeout: std::time::Duration,
    /// Répertoire de travail
    pub working_dir: Option<std::path::PathBuf>,
}

impl Default for ExternalSubagentConfig {
    fn default() -> Self {
        Self {
            command: "claude".into(),
            args: vec!["--print".into()],
            env: vec![],
            timeout: std::time::Duration::from_secs(300),
            working_dir: None,
        }
    }
}

/// Runner de sous-agents via processus externe (Claude Code, Codex, ACP).
///
/// Fail-closed : si le binaire n'est pas trouvé ou échoue, l'erreur est propagée
/// sans fallback silencieux (règle n°5 du noyau).
pub struct ExternalSubagentRunner {
    config: ExternalSubagentConfig,
}

impl ExternalSubagentRunner {
    pub fn new(config: ExternalSubagentConfig) -> Self {
        Self { config }
    }

    /// Vérifie si le binaire est disponible (fail-closed).
    pub fn check_available(&self) -> OmegaResult<()> {
        let mut cmd = TokioCommand::new("which");
        cmd.arg(&self.config.command);
        let output = std::process::Command::new("which")
            .arg(&self.config.command)
            .output()
            .map_err(|e| OmegaError::Io(format!("which {}: {}", self.config.command, e)))?;
        if !output.status.success() {
            return Err(OmegaError::ForbiddenCommand {
                command: self.config.command.clone(),
                reason: format!("binaire '{}' introuvable dans PATH", self.config.command),
            });
        }
        Ok(())
    }

    async fn run_external(&self, prompt: &str) -> OmegaResult<String> {
        // Construire la commande complète
        let mut cmd = TokioCommand::new(&self.config.command);
        cmd.args(&self.config.args);
        cmd.arg(prompt);

        if let Some(dir) = &self.config.working_dir {
            cmd.current_dir(dir);
        }

        for (k, v) in &self.config.env {
            cmd.env(k, v);
        }

        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Lancer avec timeout
        let child = cmd
            .spawn()
            .map_err(|e| OmegaError::Io(format!("spawn {}: {}", self.config.command, e)))?;

        let output = tokio::time::timeout(self.config.timeout, child.wait_with_output()).await;

        match output {
            Err(_) => Err(OmegaError::CommandTimeout {
                command: self.config.command.clone(),
                timeout_ms: self.config.timeout.as_millis() as u64,
            }),
            Ok(Err(e)) => Err(OmegaError::Io(format!("wait {}: {}", self.config.command, e))),
            Ok(Ok(out)) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return Err(OmegaError::PluginFailed(
                        self.config.command.clone(),
                        format!("exit {}: {}", out.status.code().unwrap_or(-1), stderr),
                    ));
                }
                Ok(String::from_utf8_lossy(&out.stdout).into_owned())
            }
        }
    }
}

#[async_trait]
impl SubagentRunner for ExternalSubagentRunner {
    fn name(&self) -> &str {
        "external-subagent"
    }

    async fn spawn(&self, spec: &str) -> OmegaResult<String> {
        // Vérification fail-closed au premier spawn
        self.check_available()?;
        
        // Construire le prompt selon le format attendu par le provider
        // Pour Claude Code: le spec est passé directement
        // Pour Codex: format JSON
        let prompt = if self.config.command == "codex" {
            serde_json::json!({
                "prompt": spec,
                "mode": "subagent"
            }).to_string()
        } else {
            spec.to_string()
        };

        let result = self.run_external(&prompt).await?;
        Ok(result)
    }
}

/// Contributeur pour enregistrer un ExternalSubagentRunner.
pub struct ExternalSubagentContributor {
    config: ExternalSubagentConfig,
}

impl ExternalSubagentContributor {
    pub fn new(config: ExternalSubagentConfig) -> Self {
        Self { config }
    }

    /// Crée un contributeur pour Claude Code (nécessite `claude` dans PATH).
    pub fn claude_code() -> Self {
        Self::new(ExternalSubagentConfig {
            command: "claude".into(),
            args: vec!["--print".into()],
            ..Default::default()
        })
    }

    /// Crée un contributeur pour Codex (nécessite `codex` dans PATH).
    pub fn codex() -> Self {
        Self::new(ExternalSubagentConfig {
            command: "codex".into(),
            args: vec!["exec".into(), "--json".into()],
            ..Default::default()
        })
    }
}

impl omega_plugin_api::Contributor for ExternalSubagentContributor {
    fn name(&self) -> &str {
        // Stocker le nom pour le retourner par référence
        // Note: on leak la string volontairement (nom court, durée de vie du process)
        Box::leak(format!("external-subagent-{}", self.config.command).into_boxed_str())
    }

    fn contribute(
        &self,
        builder: &mut omega_registry::RegistryBuilder,
        _ctx: &omega_plugin_api::InstallContext,
    ) -> OmegaResult<()> {
        let runner = Arc::new(ExternalSubagentRunner::new(self.config.clone()));
        // Fail-closed: si le binaire n'est pas dispo, l'enregistrement échoue
        runner.check_available()?;
        builder
            .register_dyn(
                omega_registry::Manifest::new(
                    self.config.command.clone(),
                    omega_types::Capability::SubagentRunner,
                    "1",
                ),
                runner,
            )
            .map(|_| ())
    }
}

#[cfg(test)]
mod external_subagent_tests {
    use super::*;

    #[test]
    fn external_runner_check_available_fails_gracefully() {
        let runner = ExternalSubagentRunner::new(ExternalSubagentConfig {
            command: "omega-nonexistent-binary-xyz-123".into(),
            ..Default::default()
        });
        let err = runner.check_available().unwrap_err();
        assert!(matches!(err, OmegaError::ForbiddenCommand { .. }));
    }
}
