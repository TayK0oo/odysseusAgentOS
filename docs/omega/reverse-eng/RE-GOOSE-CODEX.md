# RE-GOOSE-CODEX — Reverse-engineering de deux harnais Rust (T3 + T4)

> **Workstream :** W5 — Reverse-engineering des harnais.
> **Cibles :** Goose (`github.com/block/goose`, Apache-2.0) et Codex CLI (`github.com/openai/codex`, Apache-2.0).
> **Pourquoi elles :** ce sont **les deux seuls harnais majeurs dont le cœur est en Rust** — exactement notre stack cible (Rust core / React-TS UI).
> **Provenance :** `[observed]` = lu dans le code cloné (refs `fichier:ligne`) · `[stated]` = README/docs amont · `[inferred]` = déduction de l'agent · `[todo]` = à vérifier.
> **Corpus local :** `/tmp/opencode/re/goose` @ `01fa5df` (2026-10-14) · `/tmp/opencode/re/codex` @ `1ffa95e` (2026-10-14).
> **Règle appliquée :** extraction de *patterns*, de *signatures de traits* et de *schémas* — aucune copie de code. Licences respectées (Apache-2.0 des deux côtés ; toute réutilisation de code devra conserver l'attribution).

---

## §1 Résumé exécutif

### 1.1 Goose (`block/goose`) — `[observed]`

**Ce que c'est.** Un agent self-hosted, multi-interface (CLI, desktop, API, ACP), dont le cœur métier est un **workspace Cargo tardivement découpé** : le monolithe initial a été fracturé en un crate de **types purs** (`goose-provider-types`), une **lib de boucle d'agent générique** (`goose-agent`), une **lib de compaction indépendante** (`goose-context-management`), et un **crate applicatif** (`goose`).

**Trois idées fortes à retenir :**

1. **Une boucle d'agent abstraite, testable, agnostique du provider** — `goose-agent` définit une machine à états à base de traits (`Operation`, `Inference`, `EffectHandler`, `SessionLoader`) où la conversation est un *paramètre* du générique, pas un type en dur (`crates/goose-agent/src/operation.rs:87`, `crates/goose-agent/src/machine.rs:55`). C'est **le modèle le plus proche de notre cœur Omega**.
2. **La compaction est un plugin décorateur** — `CompactingProvider` enveloppe n'importe quel `Arc<dyn Provider>` et rattrape `ContextLengthExceeded` pour re-tenter avec un historique résumé (`crates/goose-context-management/src/provider.rs:18,67`). Le *Context Revision* n'est donc pas câblé dans la boucle : c'est un adaptateur.
3. **Les providers sont déclaratifs** — ~57 fournisseurs sont des fichiers **JSON embarqués au build** (`include_dir!`), désérialisés à l'exécution (`crates/goose-providers/src/declarative.rs:10,19`, `crates/goose-providers/src/declarative/definitions/deepseek.json`). « Configuration > code » appliqué littéralement.

**Faiblesse observée :** l'applicatif `goose` (`crates/goose/src/`) reste massif et très couplé (extension manager, providers ACP pour Claude Code/Codex/Cursor, live voice, scheduler…) — le découpage s'est fait *autour* d'un noyau encore gros. `[inferred]`

### 1.2 Codex CLI (`openai/codex`) — `[observed]`

**Ce que c'est.** Un workspace Rust de **~200 crates** (`codex-rs/Cargo.toml:1`), découpé jusqu'à l'extrême : chaque capacité est un crate (sandbox, hooks, skills, memories, otel, execpolicy, model-provider, app-server…). C'est l'architecture « pieuvre » poussée à sa limite.

**Trois idées fortes à retenir :**

1. **Un registre d'extensions typé par traits « contributeurs »** — `codex-extension-api` expose ~15 traits fins (`McpServerContributor`, `ContextContributor`, `TurnLifecycleContributor`, `ToolContributor`, `ApprovalReviewContributor`, `TokenUsageContributor`…) et un `ExtensionRegistryBuilder`/`ExtensionRegistry` immuable après build (`crates/codex-rs/ext/extension-api/src/contributors.rs:84-437`, `registry.rs:21,154`). Chaque capacité (skills, memories, goal, guardian, queue…) est un crate `ext/*` qui s'installe via une fonction `install(...)`. **C'est le « everything is a plugin » le plus abouti du corpus.**
2. **Config TOML à couches avec provenance et précédence explicite** — `ConfigLayerSource` énumère 9 origines (packaged defaults, MDM, système, enterprise, user+profil, projet, flags de session…) et leur `precedence()` (`crates/codex-rs/config/src/config_layer_source.rs:5,31`). Les profils sont des unités nommées (`ConfigProfile`, `profile_toml.rs:22`). Merge récursif avec alias de clés (`merge.rs:62`).
3. **Télémétrie OTel exhaustive et *normalisée*** — un crate `otel` dédié, avec des **noms de métriques constants** (`codex.api_request`, `codex.turn.token_usage`, `codex.tool.call`, `codex.multi_agent.spawn.phase.duration_ms`, `codex.hooks.run`…) et des **cibles de tracing** séparant *log-only* de *trace-safe* (donc contrôle de la PII) (`crates/codex-rs/otel/src/metrics/names.rs`, `otel/src/targets.rs:1`).

**Faiblesse observée :** la granularité extrême (200 crates, `workspace.dependencies` de plusieurs centaines de lignes) crée un coût de navigation et de build élevé, et un couplage par configuration plus que par code. `[inferred]`

---

## §2 Layout du workspace Rust

### 2.1 Goose — 15 crates + UI `[observed]`

Racine `Cargo.toml` : `members = ["crates/*", "vendor/v8"]`, `resolver = "2"`, lints de workspace (`string_slice = "warn"`), et **`[profile.lean]`** (`opt-level="z"`, `lto="fat"`, `panic="abort"`, strip) pour des binaires compacts (`goose/Cargo.toml:1-7,127-140`).

| Crate | Responsabilité | Dépendances internes | Ref |
|---|---|---|---|
| `goose-provider-types` | **Types purs** : messages, conversation, modèle, erreurs, `Provider` trait, `MaybeSend/MaybeSync`, formats, canonical model registry, thinking, permissions. Zéro logique applicative. | *(aucune interne)* | `crates/goose-provider-types/src/lib.rs:1-20` |
| `goose-agent` | **Lib de boucle d'agent** (« the GDK's Agent Loop ») : machine à états, opérations, effets, emitter. | `goose-provider-types` | `crates/goose-agent/Cargo.toml` |
| `goose-context-management` | **Compaction autonome** : summarize, format, templates, modèle, décorateur provider. | `goose-provider-types` | `crates/goose-context-management/Cargo.toml` |
| `goose-providers` | Implémentations de providers (anthropic, openai, google, ollama, databricks…) + providers **déclaratifs** JSON. | types | `crates/goose-providers/src/lib.rs` |
| `goose-mcp` | Serveurs MCP *fournis* par goose (memory, autovisualiser, computer-controller, tutorial) + runner. | — | `crates/goose-mcp/src/lib.rs:1-40` |
| `goose` | **Crate applicatif** : agents, extension manager, ACP, config, recipes, scheduler, skills, plugins, hooks, session… | tous les précédents | `crates/goose/Cargo.toml:170-174` |
| `goose-cli` | Binaire CLI. | `goose` | — |
| `goose-sdk` / `goose-sdk-types` | **Wire types ACP** (`_goose/*` JSON-RPC) + bindings **uniffi** (Python/Kotlin). | types | `crates/goose-sdk/src/lib.rs:1-16` |
| `goose-download-manager`, `goose-local-inference`, `goose-roaming`, `goose-test`, `goose-test-support`, `goose-acp-macros` | Support (téléchargements, inférence locale candle/llama, roaming, helpers de test, macros ACP). | — | — |

**Sens des dépendances (propre) :** `provider-types` ← `goose-agent` ← `goose` ← `goose-cli`/UI. **Aucun cycle**, et les deux crates « cœur » (`goose-agent`, `goose-context-management`) ne connaissent que des **types**, jamais l'applicatif — ce qui les rend réutilisables et testables en isolation totale. `[observed]`

### 2.2 Codex — ~200 crates, découpage par capacité `[observed]`

`codex-rs/Cargo.toml:1` liste les membres ; `[workspace.dependencies]` mappe **toutes** les dépendances internes par chemin (une centaine d'entrées `codex-* = { path = ... }`). Familles observées :

| Famille | Crates représentatifs | Rôle | Ref |
|---|---|---|---|
| **Noyau session** | `core`, `core-api`, `thread-store`, `state`, `history`, `message-history`, `rollout`, `rollout-trace`, `context-fragments` | boucle de tour, threads, historique immuable, fragments de contexte | `codex-rs/Cargo.toml:47-52,152-170` |
| **Extensibilité** | `ext/extension-api`, `ext/*` (16 crates), `plugin`, `utils/plugins`, `core-plugins`, `core-plugin-common` | registre + extensions (skills, memories, goal, guardian, queue, mcp, web-search, image-generation…) | `ext/extension-api/src/lib.rs` |
| **Outils & exécution** | `tools`, `shell-command`, `apply-patch`, `file-search`, `file-watcher`, `file-system`, `exec`, `exec-server`, `exec-server-protocol` | registre d'outils, exécution locale/remote | `tools/src/tool_executor.rs:106` |
| **Sécurité** | `sandboxing`, `linux-sandbox`, `windows-sandbox-rs`, `mxc-sandbox`, `bwrap`, `network-proxy`, `execpolicy`, `process-hardening`, `user-verification` | confinement OS + politique d'exécution | `sandboxing/src/lib.rs:19-63` |
| **LLM** | `model-provider`, `model-provider-info`, `models-manager`, `codex-api`, `codex-client`, `http-client`, `chatgpt`, `lmstudio`, `ollama`, `responses-api-proxy` | port provider + surfaces | `model-provider/src/provider.rs:108` |
| **Config** | `config`, `config-schema`, `features` | TOML à couches, profils, schéma JSON, flags de feature | `config/src/lib.rs:1-49` |
| **Observabilité** | `otel`, `otel-trace-websocket`, `analytics`, `diagnostics` | métriques/logs/traces, télémétrie | `otel/src/metrics/names.rs` |
| **Protocole / frontière** | `protocol`, `app-server`, `app-server-protocol`, `app-server-transport`, `app-server-daemon`, `code-mode-protocol`, `exec-server-protocol` | types de protocole + serveurs, exports **ts-rs** | `app-server-protocol/src/export.rs:2468` |
| **UI & agents** | `tui`, `cli`, `agent-roles`, `agent-identity`, `agent-graph-store`, `collaboration-mode-templates`, `skills`, `hooks`, `memories/{read,write}` | TUI, déclaration de rôles/sous-agents, skills, hooks, mémoire | `agent-roles/src/lib.rs:1-14` |
| **Utils** | ~35 crates `utils/*` (`absolute-path`, `cancellation`, `redacted-string`, `fuzzy-match`, `template`, `output-truncation`, `approval-presets`, `rustls-provider`…) | primitives transverses, chacune isolée | `codex-rs/Cargo.toml:119-176` |

**Sens des dépendances :** très majoritairement `utils/*` → `protocol`/`config` → `core` → `ext/*` → `tui`/`cli`. Le découpage est si fin que `protocol` (types) est un hub : la frontière Rust↔TS est matérialisée par ce crate + `app-server-protocol` (exports TypeScript générés). `[observed]`

### 2.3 Leçon de layout pour Omega `[inferred]`

- **Goose est le bon *grain* pour Omega** dans un premier temps : 1 crate de types, 1 crate de boucle, 1 crate de contexte, N crates de plugins, 1 crate applicatif, 1 crate CLI, 1 crate UI-bindings.
- **Codex est le bon *principe* à terme** : un crate par capacité de premier plan (sandbox, execpolicy, otel, skills, hooks) dès qu'une capacité a sa propre frontière de crash/perf.
- **Anti-pattern à éviter :** laisser `core`/`goose` devenir un monolithe « tout-en-un » (les deux le font partiellement).

---

## §3 Ports & traits extraits (partie centrale)

### 3.1 Goose — la boucle d'agent (`goose-agent`) `[observed]`

Le cœur le plus réutilisable du corpus. La boucle est un `StateMachine<'a, S, E>` générique sur la **session `S`** et l'**effet `E`**, paramétré par des `Step`/`Operation` enregistrés.

```rust
// crates/goose-agent/src/operation.rs:87
pub trait Operation<S, E: MaybeSend + 'static = ConversationEffect>: MaybeSend + MaybeSync {
    fn name(&self) -> &'static str;                                   // identité de l'étape
    fn set_message_meta(&self, message: &mut Message, key: &str, value: serde_json::Value) { .. }
    fn message_meta<'a>(&self, message: &'a Message, key: &str) -> Option<&'a serde_json::Value> { .. }
    async fn finalize_cancellation(&self, _session: &S, _conversation: &Conversation, _emit: &Emitter) -> Vec<E> { Vec::new() }
    async fn run_command(&self, _cmd: &SlashCommand<'_>, _session: &S, _conversation: &Conversation, _emit: &Emitter) -> Result<OperationResult<E>> { not_applicable() }
    async fn inference_tools(&self, _session: &S) -> Result<Vec<Tool>> { Ok(Vec::new()) }          // outils apportés par l'étape
    async fn prompt_parts(&self, _session: &S, _conversation: &Conversation) -> Result<Vec<(String,String)>> { Ok(Vec::new()) }
    async fn moim_parts(&self, _session: &S, _conversation: &Conversation) -> Result<Vec<String>> { Ok(Vec::new()) }
    async fn run(&self, _session: &S, _conversation: &Conversation, _emit: &Emitter) -> Result<OperationResult<E>> { not_applicable() }
}

// crates/goose-agent/src/operation.rs:156
pub trait Inference<S, E: MaybeSend + 'static = ConversationEffect>: Operation<S, E> {
    fn applies(&self, conversation: &Conversation) -> bool;                                   // le modèle serait-il appelé ?
    async fn prepare_session(&self, _session: &S) -> Result<Option<S>> { Ok(None) }            // transformation avant inférence (ex. compaction)
    async fn infer(&self, session: &S, conversation: &Conversation, input: InferenceInput, emit: &Emitter) -> Result<OperationResult<E>>;
}

// crates/goose-agent/src/machine.rs:18,25,31,35
pub trait MachineSession: MaybeSend + MaybeSync { fn id(&self) -> &str; fn conversation(&self) -> Option<&Conversation>; }
pub trait SessionLoader<S>:  MaybeSend + MaybeSync { async fn load(&self, session_id: &str) -> Result<S>; }
pub trait EffectHandler<S, E>: MaybeSend + MaybeSync { async fn apply_effects(&self, session: &S, effects: &mut [E], emit: &Emitter) -> Result<()>; }
pub trait EffectUsage<E>:  MaybeSend + MaybeSync { fn usage(&self, _effect: &E) -> Option<Usage> { None } }

// crates/goose-agent/src/subagent.rs:39
pub trait SubagentRunner<S>: MaybeSend + MaybeSync { /* lance un sous-agent sur une sous-tâche */ }
// crates/goose-agent/src/tool.rs:125
pub trait ToolProvider<S>:  MaybeSend + MaybeSync { /* expose des outils à l'inférence */ }
// crates/goose-agent/src/inference.rs:34,71
pub trait InferenceRequestPreparer<S>: MaybeSend + MaybeSync { .. }
pub trait InferenceEffect: From<Message> + MaybeSend + 'static { .. }
```

**Mécanique d'exécution** (`machine.rs:121-226`) : `step()` parcourt les étapes dans l'ordre, s'arrête à la première `Applied`, agrège les outils/prompts de **toutes** les opérations dans `InferenceInput`, et honore un `CancellationToken` via `tokio::select!{ biased; … }`. `run()` boucle `load → step → apply` jusqu'à `Yielded | Cancelled`, puis `finalize()` **répare la conversation après annulation** (messages d'outil sans réponse → réponses d'erreur synthétiques, `machine.rs:69-92,228-263`).

**Effets et émission :**

```rust
// crates/goose-agent/src/operation.rs:224
pub enum ConversationEffect {
    AppendMessage(Message),
    ReplaceConversation(Conversation),
    PatchToolRequestMeta { tool_call_id: String, patch: serde_json::Value },
    SetMessageVisibility { message_id: String, user_visible: bool, agent_visible: bool },
}
// crates/goose-agent/src/operation.rs:220
pub trait MachineEffect { fn ensure_message_ids(&mut self); }
// crates/goose-agent/src/operation.rs:267
pub struct Emitter { tx: mpsc::UnboundedSender<AgentEvent>, cancel: CancellationToken }
impl Emitter { pub fn emit(&self, e: AgentEvent); pub fn message(&self, m: Message) -> Message; pub fn cancel_token(&self) -> &CancellationToken; }
```

**Notes de conception remarquables :**

- `RunStatus { Continuing, Yielded, Cancelled }` + `OperationResult { NotApplicable, Applied(StepResult) }` : l'ordre des étapes encode la priorité (hooks avant inférence, etc.) (`operation.rs:174-190`).
- Le `target_arch = "wasm32"` change `async_trait(?Send)` et les alias `OperationFuture`/`OperationStream` — la même boucle compile pour le navigateur (`operation.rs:15-25`, `machine.rs:23-24`). **Pattern à reprendre** pour un cœur partagé Rust↔WASM.
- `set_message_meta`/`message_meta` : chaque opération **note dans le message ce qu'elle a fait**, afin qu'un pipeline reconstruit depuis la conversation persistée aboutisse à la même décision (`operation.rs:90-100`). Idée forte pour la *traçabilité* Omega.

### 3.2 Goose — le port Provider et son registre

```rust
// crates/goose-provider-types/src/base.rs:520
pub trait Provider: MaybeSend + MaybeSync {
    fn get_name(&self) -> &str;
    fn provider_session_id(&self) -> Option<String> { None }
    async fn resume(&self, _session_id: &str) -> Result<(), ProviderError> { Ok(()) }

    /// Méthode primaire : tout provider DOIT l'implémenter.
    async fn stream(&self, model_config: &ModelConfig, system: &str, messages: &[Message], tools: &[Tool])
        -> Result<MessageStream, ProviderError>;

    /// Implémentation par défaut : collecte le stream.
    async fn complete(..) -> Result<(Message, ProviderUsage), ProviderError> { collect_stream(self.stream(..).await?).await }

    async fn get_context_limit(&self, model: &str, override_limit: Option<usize>) -> usize { .. }
    fn retry_config(&self) -> RetryConfig { RetryConfig::default() }
    async fn fetch_supported_models(&self) -> Result<Vec<String>, ProviderError> { Ok(vec![]) }
    async fn fetch_recommended_models(&self, toolshim: bool) -> Result<Vec<String>, ProviderError> { .. }
    async fn map_to_canonical_model(&self, provider_model: &str) -> Result<Option<String>, ProviderError> { .. }
    fn manages_own_context(&self) -> bool { false }          // cf. wrappers CLI (Claude Code…)
    fn supports_builtin_tools(&self) -> bool { !self.manages_own_context() }
    async fn configure_oauth(&self) -> Result<(), ProviderError> { .. }
    async fn update_mode(&self, _session_id: &str, _mode: GooseMode) -> Result<(), ProviderError> { .. }
    fn thinking_effort_support(&self) -> ThinkingEffortSupport { ThinkingEffortSupport::Unspecified }
    fn permission_routing(&self) -> PermissionRouting { PermissionRouting::Noop }
    async fn handle_permission_confirmation(&self, _id: &str, _c: &PermissionConfirmation) -> bool { false }
    // + set_thinking_effort, apply_model_selection, refresh_credentials, subscribe_thinking_effort_support…
}

// crates/goose-provider-types/src/base.rs:334
pub trait ProviderDescriptor { fn metadata() -> ProviderMetadata; }   // métadonnées statiques (nom, modèles, setup, coûts)

// crates/goose/src/providers/base.rs:31
pub trait ProviderDef: ProviderDescriptor + Send + Sync {
    type Provider: Provider + 'static;
    const SESSION_BOUND: bool = false;         // instance liée à une session (pas partageable)
    const RUNS_OWN_TOOL_LOOP: bool = false;    // le provider gère lui-même la boucle d'outils (harness externe)
    fn from_env(extensions: Vec<ExtensionConfig>, tls: Option<TlsConfig>) -> BoxFuture<'static, Result<Self::Provider>>;
    fn from_env_with_working_dir(..) -> BoxFuture<..> { Self::from_env(..) }
    fn from_env_with_default_model(..) -> BoxFuture<..> { Self::from_env(..) }
}
```

**Registre runtime à constructeurs (`Arc<dyn Fn(..) -> BoxFuture<..>>`)** — pas de découverte par `inventory`, mais une table `HashMap<String, ProviderEntry>` peuplée par `init` :

```rust
// crates/goose/src/providers/provider_registry.rs:15
pub type ProviderConstructor = Arc<dyn Fn(Vec<ExtensionConfig>, Option<PathBuf>, Option<TlsConfig>, bool)
                                        -> BoxFuture<'static, Result<Arc<dyn Provider>>> + Send + Sync>;
pub struct ProviderEntry { metadata: ProviderMetadata, constructor: ProviderConstructor,
                           provider_type: ProviderType, session_bound: bool, runs_own_tool_loop: bool, .. }
pub struct ProviderRegistry { entries: HashMap<String, ProviderEntry>, tls_config: Option<TlsConfig> }
// ProviderType = { Preferred, Builtin, Declarative, Custom }  (provider_registry.rs / base.rs:20)
```

**Providers déclaratifs** (le plus intéressant pour Omega) :

```rust
// crates/goose-providers/src/declarative.rs:10
pub static FIXED_PROVIDERS: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/declarative/definitions");
// macros.rs:1 — expose_declarative_provider! génère create(tls, key_resolver) -> Box<dyn Provider>
// definitions/deepseek.json : { name, engine:"openai", base_url, api_key_env, models[], headers, timeout_seconds, setup{…} }
// KeyResolver (trait à erreur associée) : la résolution du secret est injectée, pas codée en dur.
```

### 3.3 Goose — Extensions MCP

```rust
// crates/goose/src/agents/mcp_client.rs:76
pub trait McpClientTrait: Send + Sync {
    async fn list_tools(&self, session_id: &str, cursor: Option<String>, cancel: CancellationToken) -> Result<ListToolsResult, ..>;
    async fn call_tool(&self, ..) -> Result<CallToolResult, ..>;
    fn get_info(&self) -> Option<&InitializeResult>;
    async fn get_instructions(&self, _session_id: &str, _working_dir: &Path) -> Option<String> { .. }
    async fn list_resources(..) / read_resource(..) / list_prompts(..) / get_prompt(..);
    async fn subscribe(&self) -> mpsc::Receiver<ServerNotification>;
    async fn get_moim(&self, _session_id: &str, _tools: &[Tool]) -> Option<String> { .. }   // "Model-Only Instructions/Memory"
}
```

Trois implémentations de transport dans `extension_manager/` : `stdio.rs`, `streamable_http.rs`, `builtin.rs` ; `ExtensionConfig` est une enum sérialisable (`ExtensionConfig::{Builtin, Platform, Stdio, StreamableHttp, …}`, `extension.rs:158`) avec `resolve(config)` qui substitue les `${VAR}` d'environnement (`extension.rs:382,495`). Le manager tient des **slots par scope** (working_dir + container) avec cache de tools versionné (`tools_version: AtomicU64`) invalidé sur `tools/list_changed` (`extension_manager/mod.rs:130-148,272-298`).

Les extensions dites **plateforme** (tool interne, pas un process MCP) sont enregistrées dans une `Lazy<HashMap<&str, PlatformExtensionDef>>` (`platform_extensions/mod.rs:31`) — précédent utile pour nos plugins *in-process*.

### 3.4 Goose — Context management (Context Revision)

Conçu comme une **lib indépendante**, pas comme du code de la boucle :

```rust
// crates/goose-context-management/src/lib.rs:35
pub trait CompactionInput { fn messages(&self) -> Vec<Message>; fn templates(&self) -> Templates { default } }
// crates/goose-context-management/src/lib.rs:44
pub trait CompactionOutput { fn set_summary(&mut self, summary: Message); fn set_usage(&mut self, usage: ProviderUsage); }
// crates/goose-context-management/src/lib.rs:49 — API trait-based, le caller lit/écrit SA représentation
pub async fn compact<I: CompactionInput + ?Sized, O: CompactionOutput + ?Sized>(
    model: &dyn CompactionModel, estimator: Option<&dyn TokenEstimator>, input: &I, output: &mut O) -> Result<()>;

// crates/goose-context-management/src/model.rs:13,23
pub trait CompactionModel: Send + Sync { .. }        // abstraction "appeler un modèle pour résumer"
pub trait TokenEstimator: Send + Sync { .. }
// crates/goose-context-management/src/provider.rs:18 — DÉCORATEUR
pub struct CompactingProvider { inner: Arc<dyn Provider>, templates: Templates }
//   stream(): Err(ContextLengthExceeded) => résume => retente une fois  (provider.rs:62-75)
```

Côté applicatif, `goose` règle **quand** compacter : `auto_compact_threshold(context_limit)` combine un seuil relatif (`DEFAULT_COMPACTION_THRESHOLD = 0.8`, `lib.rs:32`) et un plafond absolu de tokens (`DEFAULT_AUTO_COMPACT_TOKEN_LIMIT = 225_000`, `context_mgmt/mod.rs:20,35`) ; la compaction par paires d'outils est *opt-in* (`GOOSE_TOOL_PAIR_SUMMARIZATION`, `context_mgmt/mod.rs:49`). La suppression algorithmique des tool-responses obsolètes se fait **du milieu vers l'extérieur** (`summarize.rs:36`).

### 3.5 Goose — autres ports

```rust
// crates/goose/src/scheduler_trait.rs:8
#[async_trait] pub trait SchedulerTrait: Send + Sync {
    async fn add_scheduled_job(&self, job: ScheduledJob, copy_recipe: bool) -> Result<(), SchedulerError>;
    async fn schedule_recipe(&self, path: PathBuf, cron: Option<String>) -> Result<(), SchedulerError>;
    async fn list_scheduled_jobs(&self) -> Vec<ScheduledJob>;
    async fn run_now(&self, id: &str) -> Result<String, SchedulerError>;
    async fn kill_running_job(&self, id: &str) -> Result<(), SchedulerError>;
    // pause/unpause/update/remove/sessions/get_running_job_info…
}
```

- **Hooks** : `HookEvent { PreToolUse, PreToolUseResult, PostToolUse, PostToolUseFailure, SessionStart, SessionEnd, UserPromptSubmit, BeforeReadFile, AfterFileEdit, BeforeShellExecution, AfterShellExecution, Stop }` (`hooks/mod.rs:54`) + `HookManager` (`hooks/mod.rs:417`) ; format `hooks/hooks.json` par plugin, `matcher` regex, actions `type:"command"` avec stdin JSON, timeout par défaut 30 s (`hooks/mod.rs:1-28`).
- **Skills** : `SkillFrontmatter { name, description, metadata }` (`skills/mod.rs:33`), découverte `~/.agents/skills` + projet (`skills/mod.rs:47,53,357,609`), injection via un **client MCP interne** (`skills/client.rs`, `SkillsClient`).
- **Sous-agents** : `TaskConfig { provider, model_config, parent_session_id, parent_working_dir, extensions, max_turns }`, `DEFAULT_SUBAGENT_MAX_TURNS = 25` (`subagent_task_config.rs:10,13`).
- **SDK multi-langage** : `goose-sdk` = wire types ACP + **uniffi** (cdylib/staticlib) exposant providers déclaratifs et streaming à Python/Kotlin (`goose-sdk/src/lib.rs:1-16`).

### 3.6 Codex — le registre d'extensions (le « plugin system ») `[observed]`

**~15 traits contributeurs, fins et spécialisés** — chacun répond à *une* question du cycle de vie. Tous retournent un future boxé `ExtensionFuture<'a, T>` et ont des **implémentations par défaut vides**, donc un plugin ne surcharge que ce qu'il utilise (principe ISP).

| Trait | Rôle | Signature clé | Ref |
|---|---|---|---|
| `McpServerContributor<C>` | fournir des serveurs MCP runtime + déclarer des plugins | `id() -> &'static str`, `contribute(ctx) -> Vec<McpServerContribution>`, `selected_plugins(ctx, cfg)` | `contributors.rs:84` |
| `ContextContributor` | injecter des fragments de prompt | `contribute_thread_context`, `contribute_turn_context`, `contribute_world_state` | `:112` |
| `ThreadLifecycleContributor<C>` | hooks de cycle de vie thread | `on_thread_start/ready/resume/idle/stop` | `:155` |
| `TurnLifecycleContributor` | hooks de cycle de vie tour | `turn_start_phase`, `on_turn_start/stop/abort/error`, `on_item_completed` | `:207` |
| `TurnInputContributor` | ajouter de l'entrée modèle locale au tour | `contribute(input, metrics, stores) -> Vec<Box<dyn ContextualUserFragment>>` | `:273` |
| `ConfigContributor<C>` | réagir à un changement de config effective | `on_config_changed(stores, prev, new)` | `:290` |
| `TokenUsageContributor` | observer la consommation de tokens | `on_token_usage(stores, usage)` | `:307` |
| `SkillInvocationContributor` | observer les invocations de skills | `requires_host_skill_discovery()`, `on_skill_invocation(input)` | `:327` |
| `ToolContributor` | exposer des outils natifs | `tools(stores) -> Vec<Arc<dyn ToolExecutor<_>>>`, `tools_for_step` | `:348` |
| `ToolLifecycleContributor` | observer le cycle de vie outil | `on_tool_dispatch/start/finish/timing`, `on_command_start`, `on_mcp_tool_result` | `:372` |
| `ApprovalReviewContributor` | **décider** une approbation (chaîne de responsabilité) | `decide(input) -> Option<ApprovalDecision>` | `:413` |
| `TurnItemContributor` | post-traiter un item de tour (mutation autorisée) | `contribute(thread_store, turn_store, item) -> Result<(),String>` | `:428` |
| `ModelRequestContributor` / `ModelResponseInterceptor` | intercepter requête/réponse modèle | — | `model_request.rs:33,40` |
| `ThreadInstructionsProvider` / `UserInstructionsProvider` | fournir les instructions système/projet | — | `user_instructions.rs:44,67` |
| `TurnStartAdmission` | **porte** d'admission d'un nouveau tour | `admit_turn_start() -> Option<Box<dyn Send>>` | `turn_admission.rs:10` |
| `ToolPolicy`, `SessionIsolation`, `IsolatedSessionExtensions`, `ExtensionEventSink`, `ExtensionMetrics`, `ResponseItemInjector` | politique d'outils, isolation de session, émission d'événements, métriques, injection d'items | — | `lib.rs:14-24` |

**Le registre lui-même :**

```rust
// ext/extension-api/src/registry.rs:21 — mutable pendant l'installation
pub struct ExtensionRegistryBuilder<C: Sync> { /* une Vec<Arc<dyn Trait>> par catégorie */ }
impl ExtensionRegistryBuilder<C> {
    pub fn approval_review_contributor(&mut self, c: Arc<dyn ApprovalReviewContributor>);
    pub fn thread_lifecycle_contributor(&mut self, c: Arc<dyn ThreadLifecycleContributor<C>>);
    pub fn tool_contributor(&mut self, c: Arc<dyn ToolContributor>);
    // … une méthode par trait
    pub fn build(self) -> ExtensionRegistry<C>;
}
// ext/extension-api/src/registry.rs:154 — immuable après build
pub struct ExtensionRegistry<C: Sync> { /* accesseurs &[Arc<dyn Trait>] */ }
impl ExtensionRegistry<C> {
    pub async fn decide_approval(&self, input: &ApprovalDecisionInput<'_>) -> Option<ApprovalDecision> {
        for c in &self.approval_review_contributors { if let Some(d) = c.decide(input).await { return Some(d); } } None
    }                                            // <= chaîne de responsabilité "premier qui réclame gagne"
}
pub fn empty_extension_registry<C: Sync>() -> Arc<ExtensionRegistry<C>>;   // hosts sans plugins
```

Chaque extension (`ext/skills`, `ext/memories`, `ext/goal`, `ext/guardian-v2`, `ext/queue`, `ext/mcp`…) expose une fonction **`pub fn install(...) -> …`** qui reçoit le builder et enregistre ses contributeurs (`ext/skills/src/lib.rs:24`, `ext/memories/src/lib.rs:9`, `ext/goal/src/lib.rs:22`).

**Deux mécanismes de rétro-compatibilité très soignés** `[observed]` :
- `to_builder()` sur le registre immuable permet à un host de *cloner puis étendre* la composition sans la muter (`registry.rs:174`).
- Les défauts préservent l'ancien comportement : `RequiresHostSkillDiscovery` par défaut `true`, donc l'absence de contributeur ⇒ découverte legacy (`registry.rs:238-244`, `contributors.rs:331`).

### 3.7 Codex — outils et provider modèle

```rust
// codex-rs/tools/src/tool_executor.rs:106 — contrat runtime d'un outil
pub trait ToolExecutor<Invocation>: Send + Sync {
    fn tool_name(&self) -> ToolName;
    fn spec(&self) -> ToolSpec;                                    // schéma model-visible
    fn exposure(&self) -> ToolExposure { ToolExposure::Direct }    // Direct | Deferred | CodeModeOnly | Direct/DeferredModelOnly | Hidden (tool_executor.rs:51)
    fn search_info(&self) -> Option<ToolSearchInfo>;               // métadonnées pour la recherche d'outils
    fn supports_parallel_tool_calls(&self) -> bool { false }
    fn handle<'a>(&'a self, invocation: Invocation) -> ToolExecutorFuture<'a> where Invocation: 'a;
}

// codex-rs/model-provider/src/provider.rs:108 — port LLM
pub trait ModelProvider: fmt::Debug + Send + Sync {
    fn info(&self) -> &ModelProviderInfo;
    fn capabilities(&self) -> ProviderCapabilities { .. }
    fn auth_manager(&self) -> Option<Arc<AuthManager>>;
    fn map_api_error(&self, error: ApiError) -> CodexErr;
    fn responses_api_provider<'a>(..) -> ModelProviderFuture<'a, ..>;
    fn models_manager(self: Arc<Self>) -> Arc<dyn ModelsManager>;   // catalogue de modèles
    // préférences par usage : approval_review_preferred_model(), memory_extraction/consolidation_preferred_model()
}
pub fn create_model_provider(...) -> ..;                            // fabrique
```

Remarquer la **séparation `spec` (schéma) / `handle` (exécution)** explicitement documentée : « hosts can layer routing, hooks, telemetry… without reopening the spec/runtime split » (`tool_executor.rs:100-104`).

### 3.8 Codex — hooks (2 moteurs, spec déclarative)

```rust
// codex-rs/hooks/src/types.rs:15
pub enum HookResult { Success, FailedContinue(Box<dyn Error…>), FailedAbort(Box<dyn Error…>) }
impl HookResult { pub fn should_abort_operation(&self) -> bool; }     // FailedAbort ⇒ interrompt l'opération
pub struct Hook { pub name: String, pub func: HookFn }               // HookFn = Arc<dyn Fn(&HookPayload) -> BoxFuture<HookResult>>
pub struct HookPayload { session_id: ThreadId, cwd: AbsolutePathBuf, client: Option<String>, triggered_at: DateTime<Utc>, hook_event: HookEvent }

// codex-rs/protocol/src/protocol.rs:1691 — événements normalisés
pub enum HookEventName { PreToolUse, PermissionRequest, PostToolUse, PreCompact, PostCompact,
                         SessionStart, SessionEnd, UserPromptSubmit, SubagentStart, SubagentStop, Stop, Interrupt }
pub enum HookHandlerType { Command, McpTool, Prompt, Agent }         // :1708
pub enum HookExecutionMode { Sync, Async }                          // :1717
pub enum HookScope { Thread, Turn }                                 // :1724
pub enum HookSource { System, User, Project, Mdm, SessionFlags, Plugin, CloudRequirements, .. } // :1731
pub enum HookTrustStatus { Managed, Untrusted, Trusted }            // :1748
```

Implémentation dans `hooks/src/engine/` : `discovery.rs`, `matcher.rs` (matching des matchers d'événements), `command_runner.rs`, `mcp_runner.rs`, `output_parser.rs`, `schema_loader.rs` ; un runner par `HookHandlerType`. Le déclaratif vit dans `hooks/src/declarations.rs`. **Résultat typé et abortable** : bien plus riche que le `type:"command"` de Goose.

### 3.9 Codex — config, features, instructions

```rust
// codex-rs/config/src/config_layer_source.rs:5,31 — provenance + précédence
pub enum ConfigLayerSource { PackagedDefaults, Mdm, System, EnterpriseManaged, User{profile}, Project, SessionFlags, LegacyManagedConfigTomlFromFile, LegacyManagedConfigTomlFromMdm }
impl ConfigLayerSource { pub fn precedence(&self) -> i16 { /* -10 … 50 */ } }
pub struct ConfigLayer { pub name: ConfigLayerSource, pub version: String, pub config: JsonValue, pub disabled_reason: Option<String> }  // :72
pub fn merge_toml_values(base: &mut TomlValue, overlay: &TomlValue);   // config/src/merge.rs:62 (alias de clés + merge récursif)

// codex-rs/config/src/profile_toml.rs:22
pub struct ConfigProfile { model, model_provider, approval_policy, sandbox_mode, model_reasoning_effort, tools, .. }
// #[schemars(deny_unknown_fields)] : un champ inconnu dans un profil = erreur.

// codex-rs/features/src/lib.rs:48 — cycle de vie d'un flag
pub enum Stage { UnderDevelopment, Experimental{name, menu_description, announcement}, Stable, Deprecated, Removed }
pub const FEATURES: &[FeatureSpec];                                    // :1019 registre des features
```

**Découverte `AGENTS.md` remontante** (documentée en tête de fichier) :

```
// codex-rs/core/src/agents_md.rs:1-18 — algorithme
// 1. Racine projet = remontée depuis le cwd jusqu'à un marqueur (`project_root_markers`, défaut `.git`),
//    via codex_file_system::find_nearest_ancestor_with_markers (agents_md.rs:33,215).
// 2. Concaténer TOUS les AGENTS.md de la racine jusqu'au cwd inclus, dans cet ordre.
// 3. On ne remonte jamais au-delà de la racine projet. Liste de marqueurs vide ⇒ remontée désactivée.
pub const DEFAULT_AGENTS_MD_FILENAME: &str = "AGENTS.md";              // :43
pub const LOCAL_AGENTS_MD_FILENAME: &str = "AGENTS.override.md";       // :45 (override local)
pub(crate) async fn load_project_instructions(..) -> io::Result<Option<LoadedAgentsMd>>;  // :58
pub struct LoadedAgentsMd { .. }                                       // :301
```

Autres points clés de la config : `project_doc_max_bytes` plafonne le total, les dossiers non-fair `active_project.is_untrusted()` court-circuitent la lecture (`agents_md.rs:64-68`), et un projet non-trusté n'expose que les instructions utilisateur.

---

## §4 Patterns Rust réutilisables

### 4.1 Config typée + schéma + refus des inconnues `[observed]`
- **serde + schemars** partout, `#[schemars(deny_unknown_fields)]`, et un crate dédié `config-schema` (avec `main.rs`) qui **génère le JSON Schema** `config.schema.json` pour l'éditeur (`config-schema/src/main.rs`, `config/src/schema.rs:6-20`).
- **Erreurs de config typées** avec localisation : `ConfigError`, `diagnostics.rs`, `strict_config.rs` (avertissements sur champs ignorés), `ErrorLocation { TextPosition, TextRange }` dans execpolicy.
- **Priorité explicite** : la précédence est une méthode *testable* (`precedence() -> i16`), pas un `if` enfoui.
- Goose applique la même philosophie en plus simple : YAML + `Config::global()`, secrets via `get_secret`, providers déclaratifs JSON.

→ **Pour Omega :** un crate `omega-config` : `serde` + `schemars` + `deny_unknown_fields`, une enum `ConfigLayerSource` avec `precedence()`, merge récursif TOML, générateur de schéma publié pour l'UI.

### 4.2 Gestion d'erreurs `[observed]`
- **`thiserror` pour les erreurs de domaine** (`ProviderError`, `SkillParseError`, `ConfigError`, `ExtensionError`) ; **`anyhow::Result` pour le câblage applicatif**.
- **Erreurs porteuses de contexte** : `ProviderError::ContextLengthExceeded(..)` est *matchée* par le décorateur de compaction (`provider.rs:67`) — une erreur est un **signal exploitable**, pas seulement un message.
- **Erreurs typées côté frontière** : `CodexErrorDetails`, `agent_error.rs`, `ApiError -> CodexErr` via `map_api_error` (le provider traduit son erreur vers l'erreur du noyau).
- `HookResult::{FailedContinue, FailedAbort}` distingue « continuer la chaîne » de « tout arrêter » — pattern à reprendre pour nos pipelines de plugins.

→ **Pour Omega :** une enum d'erreurs par port (`thiserror`), jamais de `String` d'erreur ; distinguer *récupérable / fatale* ; les erreurs du noyau connues des décorateurs (ex. `ContextOverflow`).

### 4.3 Async runtime `[observed]`
- **Tokio multi-thread**, `tokio::sync` (mpsc unbounded pour les événements, `Mutex`/`RwLock`), `CancellationToken` (tokio-util) pour annulation coopérative.
- Annulation explicite et répétable : `tokio::select! { biased; result = fut => …, _ = cancel.cancelled() => None }` (`machine.rs:159`), extension trait `OrCancelExt::or_cancel(&token)` pour toute `Future` (`async-utils/src/lib.rs:16`).
- **`async_trait` avec variante `?Send` pour wasm** : un même crate compile natif + navigateur (`operation.rs:15-25`).
- Futures retournés en `BoxFuture`/`ExtensionFuture<'a, T> = Pin<Box<dyn Future + Send + 'a>>` pour effacer les types d'impl.
- Threads de travail à **pile explicite** (`THREAD_STACK_SIZE_BYTES = 16 MiB`, `async-utils/src/lib.rs:8`).

→ **Pour Omega :** une seule « règle d'annulation » (token propagé partout, jamais de `abort` brutal), `BoxFuture` aux frontières de traits, tests compatibles wasm si on veut un cœur navigateur.

### 4.4 Injection de dépendances `[observed]`
- **Trois niveaux, du plus fort au plus faible :**
  1. **Registre runtime à constructeurs** (Goose providers) : `Arc<dyn Fn(..) -> BoxFuture<Result<Arc<dyn Provider>>>>`, table `HashMap` → testable et remplaçable à chaud.
  2. **Builder + registre typé** (Codex extensions) : contribuer par trait avant `build()`, immuable ensuite ; `to_builder()` pour cloner-étendre.
  3. **Fabrique de haut niveau** (`create_model_provider`, `empty_extension_registry`, `ExtensionManager::new`).
- **Aucun service locator.** Les dépendances arrivent par le constructeur (`TaskConfig`, `ExtensionManagerCapabilities`, `AnyClient`…). `[observed]`
- **Capacités optionnelles par `Option<Arc<dyn Trait>>`** (ex. `Option<Arc<dyn ExtensionMetrics>>` dans `TurnInputContributor::contribute`, `contributors.rs:279`) — un plugin déclare ce dont il a besoin.

→ **Pour Omega :** `OmegaRegistry<T: PortTrait>` générique, enregistrement par `register(id, Arc<dyn Port>)`, résolution par *capacité* (jamais par marque), et un `PortContext` injecté.

### 4.5 Feature flags `[observed]`
- **Features Cargo** pour le *build* (Goose : `otel`, `local-inference`, `cuda/vulkan/mlx`, `rustls-tls`/`native-tls`, `scheduler`, `code-mode`, `platform-apps` ; `compile_error!` si `live-websocket` sans TLS — `goose-providers/src/lib.rs:1-5`).
- **Flags runtime** pour le *produit* (Codex `codex-features` : `Stage` + `FEATURES` + legacy keys) — découplés du build, avec cycle de vie et message de dépréciation.
- **`[profile.lean]`** pour un binaire compact.

→ **Pour Omega :** les deux niveaux (Cargo pour les backends, un registre runtime `FeatureSpec` avec `Stage` pour les capacités produit) ; un flag retiré a un chemin de compatibilité explicite.

### 4.6 Tests `[observed]`
- Tests **dans le même fichier** (`#[cfg(test)] mod tests`) ou fichiers `*_tests.rs` à côté — Codex les référence explicitement dans les `mod` (`config/src/lib.rs`, `otel/src/*_tests.rs`).
- Helpers de test partagés dans des crates dédiés (`goose-test-support`, `codex-test-binary-support`, `test_support.rs`) et `dev-dependencies` (`wiremock`, `insta`, `test-case`, `serial_test`, `pretty_assertions`).
- **Tests de contrat d'erreur** : `add_tools_to_inference_input` rejette les noms d'outil dupliqués entre opérations (`machine.rs:309-332`) — le noyau se teste *sans réseau*.
- Snapshots (insta) pour les formats sérialisés stables (`hook_payload_serializes_stable_wire_shape`, `hooks/src/types.rs:114`).

### 4.7 Frontière Rust ↔ TS/UI `[observed]`
- **Goose :** wire types ACP dans `goose-sdk-types` (source unique pour les méthodes `_goose/*`), SDK multi-langage **uniffi** (`goose-sdk/src/lib.rs`), serveur ACP HTTP/WebSocket pour UI custom (`CUSTOM_DISTROS.md`) ; schémas d'outils normalisés (`tool_schema_normalize.rs`).
- **Codex :** `protocol` + `app-server-protocol` génèrent les **types TypeScript via `ts-rs`** (`export.rs:2468`, `precomputed_exports.rs`) avec **fixtures de schéma** (`schema_fixtures.rs`). La frontière est *générée*, pas maintenue à la main.
- Les deux posent `#[serde(rename_all = "snake_case"|"kebab-case")]` et, côté Codex, `#[derive(JsonSchema, TS)]` sur les mêmes types.

→ **Pour Omega :** un crate `omega-protocol` source unique, dérive `Serialize/Deserialize/JsonSchema/TS`, export TS automatique + fixtures de schéma en CI.

---

## §5 Features détaillées (valeur Omega · portage)

Légende difficulté : **S** ≤ 1 j · **M** 2-5 j · **L** 1-3 sem. · **XL** > 3 sem. (estimation pour un portage *original*, pas un copier-coller).

### 5.1 Goose — Boucle d'agent abstraite `goose-agent` ⭐ `[observed]`
- **Ce que ça fait :** machine à états pilotée par une liste ordonnée d'`Operation`/`Inference` ; l'ordre encode la priorité ; effets différés et appliqués par un `EffectHandler` ; réparation de la conversation après annulation ; identifiants de message auto-générés ; compilation wasm.
- **Implémentation :** traits §3.1 ; `Emitter` = canal mpsc + token d'annulation ; `InferenceInput` agrège outils/prompt/moim de *toutes* les étapes.
- **Refs :** `crates/goose-agent/src/{machine.rs:55,121,211,228 ; operation.rs:87,156,220,224,267}`.
- **Valeur Omega :** ★★★ — c'est **le squelette de notre cœur d'orchestration** ; répond directement à P7 (Plan/Act) et P10 (hooks) via l'ordre des étapes.
- **Portage :** **M** (le code est petit ; l'effort est dans l'adaptation du type `Conversation`).

### 5.2 Goose — Context Revision / compaction enfichable ⭐ `[observed]`
- **Ce que ça fait :** résumé par un (petit) modèle, seuil double (relatif 0,8 + plafond 225 k), suppression algorithmique du milieu vers l'extérieur, décorateur qui rattrape l'overflow et retente, templates de continuation (conversation vs boucle d'outils vs manuel).
- **Implémentation :** `CompactionInput/Output` + `compact()` + `CompactingProvider` + `summarize()` ; `Templates` (minijinja) personnalisables ; `StructuredSummary`/`FileActivity` pour un format stable.
- **Refs :** `crates/goose-context-management/src/{lib.rs:32,35,44,49 ; provider.rs:18,67 ; model.rs:13,23 ; summarize.rs:36,77,124 ; structured.rs}`, `crates/goose/src/context_mgmt/mod.rs:20,35,49`.
- **Valeur Omega :** ★★★ — P4 (compaction + stabilité de préfixe) ; le décorateur évite de polluer le noyau.
- **Portage :** **M**.

### 5.3 Goose — Providers déclaratifs JSON ⭐ `[observed]`
- **Ce que ça fait :** ajouter un fournisseur OpenAI-compatible **sans écrire de Rust** (nom, base_url, `api_key_env`, modèles, coûts, headers, setup).
- **Implémentation :** `include_dir!` + `expose_declarative_provider!` → module par provider avec `create(tls, key_resolver)` ; `KeyResolver` injecté.
- **Refs :** `crates/goose-providers/src/declarative.rs:10,19`, `declarative/macros.rs:1`, `definitions/deepseek.json`, `crates/goose/src/providers/custom_provider_config.rs:5-20`.
- **Valeur Omega :** ★★★ — cœur du pilier « routing multi-fournisseurs » (P6) ; économie (ajouter un provider openrouter/scaleway = config).
- **Portage :** **S-M**.

### 5.4 Goose — Extensions MCP + extensions plateforme `[observed]`
- **Ce que ça fait :** config d'extension (stdio / streamable-http / builtin / platform) avec substitution `${VAR}`, cache de tools versionné, instructions par extension, MOIM, gestion enable/disable par session, préfixage des outils (`ext__tool`).
- **Implémentation :** `McpClientTrait` (3 transports), `ExtensionManager` avec slots scopés, `PlatformExtensionDef` pour les outils in-process.
- **Refs :** `crates/goose/src/agents/mcp_client.rs:76` ; `extension_manager/mod.rs:130,272,370` ; `platform_extensions/mod.rs:31` ; `extension.rs:158,382,495`.
- **Valeur Omega :** ★★★ — P1/P3 (context engine, tool search) : le préfixage + cache versionné + filtrage par session sont directement réutilisables.
- **Portage :** **M-L**.

### 5.5 Codex — Registre d'extensions « contributeurs » ⭐⭐ `[observed]`
- **Ce que ça fait :** ~15 points d'extension fins sur tout le cycle de vie (threads, tours, outils, approbations, tokens, skills, config, contexte, MCP) ; builder → registre immuable ; chaînes de responsabilité (`decide_approval`) ; rétro-compatibilité par défauts.
- **Implémentation :** traits §3.6, chacun avec défauts vides ; `ExtensionRegistryBuilder`/`ExtensionRegistry` ; chaque crate `ext/*` fournit `install(builder)`.
- **Refs :** `codex-rs/ext/extension-api/src/contributors.rs:84-437`, `registry.rs:21,154,174,247,296`.
- **Valeur Omega :** ★★★ — **le modèle de référence pour « everything is a plugin »** ; inspiration directe pour nos ports (P10 hooks, P5 sous-agents, P6 routing via `ConfigContributor`, observabilité via `TokenUsageContributor`).
- **Portage :** **L** (beaucoup de traits ; à *réduire* à notre périmètre, pas à cloner).

### 5.6 Codex — Config TOML à couches + profils ⭐ `[observed]`
- **Ce que ça fait :** 9 sources de config ordonnées par précédence, profils nommés, merge récursif avec alias de clés, schéma JSON généré, avertissements sur champs ignorés, config « managed » entreprise/MDM, trust de projet.
- **Implémentation :** `ConfigLayerSource::precedence()`, `ConfigLayer`, `merge_toml_values`, `ConfigProfile`, `schema.rs`, `strict_config.rs`, `project_trust.rs`, loader par OS (`loader/{local,macos,windows,project_discovery}.rs`).
- **Refs :** `codex-rs/config/src/config_layer_source.rs:5,31,72` ; `merge.rs:62` ; `profile_toml.rs:22` ; `schema.rs:6` ; `loader/mod.rs:1-40`.
- **Valeur Omega :** ★★★ — P6 + « configuration > code » (RULES §1.4) ; le concept de **provenance de couche** est précieux pour la traçabilité.
- **Portage :** **M** (adapter au besoin : on n'a probablement pas MDM).

### 5.7 Codex — Sandbox multi-OS + politiques d'approbation granulaires ⭐⭐ `[observed]`
- **Ce que ça fait :** `SandboxMode`/`SandboxPolicy` (read-only, workspace-write, danger-full-access, external-sandbox) avec `writable_roots`, network on/off, exclusions TMPDIR ; backends **Seatbelt (macOS) / Landlock+seccomp (Linux) / RestrictedToken & MXC (Windows)** ; `WritableRoot` protège `.git`, `.git/hooks`, `.codex` ; `SandboxManager::transform()` réécrit l'argv pour l'injection (bwrap, seatbelt-exec…).
- **Approbations :** `AskForApproval { UnlessTrusted, OnRequest, Granular(GranularApprovalConfig), Never }` où `Granular` distingue `sandbox_approval`, `rules`, `skill_approval`, `request_permissions`, `mcp_elicitations` (`protocol.rs:1095,1121`).
- **Refs :** `protocol/src/protocol.rs:1095,1121,1178,1181,1237` ; `sandboxing/src/lib.rs:19-63` ; `sandboxing/src/manager.rs:36,44,50,313,355` ; `linux-sandbox/`, `windows-sandbox-rs/`, `mxc-sandbox/`.
- **Valeur Omega :** ★★★ — P9 (exécution sandboxée multi-backend) + sécurité des agents longs. Le **modèle de politique typée** vaut plus que le code OS.
- **Portage :** **XL** côté backend, **M** côté *types de politique* (réutiliser Landlock/bubblewrap via crates existants).

### 5.8 Codex — Execpolicy (DSL de règles d'exécution) ⭐ `[observed]`
- **Ce que ça fait :** un petit langage de règles (préfixes de commande, protocoles réseau) qui décide `Allow`/`Prompt`/`Forbidden` **avant** sandbox/approbation ; amendable dynamiquement (`blocking_append_allow_prefix_rule`).
- **Implémentation :** `PolicyParser`, `Policy::evaluate`, `Rule`/`PrefixRule`/`NetworkRule`, `Decision`, `RuleMatch`.
- **Refs :** `codex-rs/execpolicy/src/lib.rs:1-33`, `decision.rs:9`, `rule.rs:16,40,64,111,118,149`, `policy.rs`, `amend.rs`.
- **Valeur Omega :** ★★ — déterministe, local, rapide : **exactement le principe « le déterministe n'est pas une tâche d'agent »** (RULES §3.2). Complète le sandbox.
- **Portage :** **M** (portage du *concept* : règles préfixes + réseau, évaluées en Rust).

### 5.9 Codex — OpenTelemetry exhaustif `[observed]`
- **Ce que ça fait :** métriques nommées et stables (`codex.api_request`, `codex.tool.call`, `codex.turn.token_usage`, `codex.turn.ttft.duration_ms`, `codex.turn.cost_microusd`, `codex.multi_agent.spawn.phase.duration_ms`, `codex.hooks.run`, `codex.thread.skills.*`…), logs/traces avec **cibles log-only vs trace-safe** (contrôle PII), compteurs/histogrammes avec buckets dédiés, exporteur OTLP + stdout.
- **Refs :** `otel/src/metrics/names.rs:3-90`, `otel/src/targets.rs:1-8`, `otel/src/events/shared.rs:1-70`, `otel/src/events/session_telemetry.rs:752,1195,382,395,275`, `otel/src/metrics/{client,buffered,runtime_metrics}.rs`, `goose/src/otel/otlp.rs:300` (Goose a une variante plus simple : OTLP + propagation de trace-context via MCP meta, `otel/trace_context.rs:15,26`).
- **Valeur Omega :** ★★★ — pilier « économie & traçabilité ». Le nommage `codex.<domaine>.<objet>` et la séparation log/trace sont à copier.
- **Portage :** **M**.

### 5.10 Codex — Hooks déclaratifs typés ⭐ `[observed]`
- **Ce que ça fait :** 12 événements (Pre/PostToolUse, Pre/PostCompact, Session Start/End, UserPromptSubmit, Subagent Start/Stop, Stop, Interrupt, PermissionRequest), 4 types de handler (command, mcp_tool, prompt, agent), modes sync/async, scopes thread/turn, provenance de la source du hook, statut de confiance.
- **Refs :** `protocol/src/protocol.rs:1691-1752`, `hooks/src/types.rs:15,33,39,64,92`, `hooks/src/engine/*`, `hooks/src/declarations.rs`.
- **Valeur Omega :** ★★★ — P10. Notre `HookManager` devrait renvoyer un **résultat typé** (continue/abort) et tracer `hook_id`, `event`, `duration`.
- **Portage :** **M**.

### 5.11 Codex — Skills (progressive disclosure) `[observed]`
- **Ce que ça fait :** `SKILL.md` avec frontmatter YAML (`name`, `description`, `metadata.short-description`), découverte multi-source, catalogue + **prompt de catalogue** + sélection dynamique, invocation explicite et **implicite** (`detect_implicit_skill_invocation`), alias/host-roots, fichiers support, politique (`allow_implicit_invocation`, `products`), dépendances déclarées (outils/transport/command/url/oauth), télémétrie de troncature de description.
- **Refs :** `skills/src/parser.rs:35`, `skills/src/metadata.rs:28`, `ext/skills/src/{lib.rs:1-40 ; catalog.rs ; dynamic_skill_selector.rs ; invocation.rs ; world_state.rs ; telemetry.rs}`.
- **Valeur Omega :** ★★ — P2. La sélection dynamique (ne charger que quelques skills, tronquer les descriptions, mesurer) est la partie qui compte.
- **Portage :** **M**.

### 5.12 Codex — Découverte `AGENTS.md` remontante ⭐ `[observed]`
- **Ce que ça fait :** racine projet par marqueurs (`.git` par défaut, configurable, vide ⇒ off), concaténation racine→cwd, override local `AGENTS.override.md`, plafond d'octets, respect du **trust** du projet, marche par environnement/executor (fonctionne en sandbox remote).
- **Refs :** `core/src/agents_md.rs:1-18,43,45,58,215,301`.
- **Valeur Omega :** ★★★ — implémentation déjà éprouvée du « contexte projet » ; à porter quasiment telle quelle dans notre Context Engine.
- **Portage :** **S**.

### 5.13 Codex — Sous-agents déclarés (roles) + boucle d'agents `[observed]`
- **Ce que ça fait :** rôles définis par fichier TOML (`name`, `description`, `nickname_candidates`, et un `ConfigToml` embarqué = **la config du sous-agent**) ; découverte/discovery ; les multi-agents ont leurs propres instructions (`prompts/src/multi_agent_instructions.rs`), leur télémétrie de spawn, leur registre (`agent-graph-store`, `agent-identity`, `agent-message-board-client`).
- **Refs :** `agent-roles/src/{lib.rs ; agent_role_config.rs:11,27,31 ; discovery.rs ; loader.rs}`, `core/src/tools/handlers/multi_agents_spec.rs:738`, `otel/src/metrics/names.rs:6-8`.
- **Valeur Omega :** ★★★ — P5 (sous-agents isolés parallèles). Le fait que **chaque rôle porte sa propre config** est le bon design.
- **Portage :** **M**.

### 5.14 Codex — Tool executor & exposure (tool search / code mode) `[observed]`
- **Ce que ça fait :** un outil = `{ name, spec, exposure, search_info, supports_parallel, handle() }` ; `ToolExposure` distingue direct/différé/code-mode-only/hidden → base d'un **chargement progressif des outils** (P3 tool search) et d'un mode « exécuter du code ».
- **Refs :** `tools/src/tool_executor.rs:51,106`, `tools/src/{tool_discovery.rs,code_mode.rs,dynamic_tool.rs,responses_api.rs}`.
- **Valeur Omega :** ★★★ — P3 : `search_info()` + `exposure()` = exactement l'infra d'un tool-search à la Hermes/OpenClaw.
- **Portage :** **M**.

### 5.15 Codex — Memories locales `[observed]`
- **Ce que ça fait :** dossier `memories/` local, outils MCP-ish (`add_ad_hoc_note`, `list`, `read`, `search`), budget de tokens (read 20 k, search 200 résultats), pipeline d'écriture en 2 phases avec sandbox, citations, métriques ; instructions développeur bornées (~2,5 k tokens).
- **Refs :** `memories/read/src/lib.rs`, `memories/write/src/{phase1.rs,phase2.rs,storage.rs,guard.rs}`, `ext/memories/src/lib.rs:9-24`.
- **Valeur Omega :** ★★ — mémoire bornée (à comparer à T2 Hermes/OpenClaw avant de porter).
- **Portage :** **M**.

### 5.16 Codex — App-server / protocole & exports TS `[observed]`
- **Ce que ça fait :** un crate `protocol` en source unique des types, `app-server` (JSON-RPC) + `app-server-protocol` qui **exportent les types TypeScript** (`ts-rs`) et maintiennent des **fixtures de schéma** ; transports multiples (stdio-to-uds, websocket, tcp-tunnel).
- **Refs :** `app-server-protocol/src/export.rs:2468`, `app-server-protocol/src/{rpc.rs,schema_fixtures.rs,protocol/*}`, `app-server-*`.
- **Valeur Omega :** ★★ — « zéro-friction frontière » (RULES §2.5) : schéma typé versionné + types TS générés.
- **Portage :** **M**.

### 5.17 Goose — Distributions personnalisées `[stated]`
- **Ce que ça fait :** « white labelling » documenté : préconfigurer provider/modèle, bundler des extensions, prompts/UI custom, tout via `config.yaml`, `init-config.yaml`, env, recipes — ou via `goose serve` (ACP) pour une UI tierce.
- **Refs :** `CUSTOM_DISTROS.md:1-60`.
- **Valeur Omega :** ★★ — confirme qu'une **UI interchangeable par ACP** + config déclarative permet de marquer/brander sans fork.
- **Portage :** **S** (décision d'architecture, pas de code).

### 5.18 Codex — Feature staging & cycle de vie `[observed]`
- **Ce que ça fait :** `Stage { UnderDevelopment, Experimental{…}, Stable, Deprecated, Removed }` + registre `FEATURES` + clés legacy + schéma de features restreint.
- **Refs :** `features/src/lib.rs:48,1019`, `features/src/legacy.rs`, `config/src/schema.rs:41`.
- **Valeur Omega :** ★★ — discipline de cycle de vie pour nos plugins/capacités.
- **Portage :** **S**.

---

## §6 Ce qu'Omega reprend / améliore / écarte

### 6.1 À reprendre (directement ou quasi)
| # | Élément | Source | Notre port |
|---|---|---|---|
| R1 | Boucle `Operation`/`Inference` + `EffectHandler` + `Emitter` | Goose `goose-agent` | `omega-core` : `Operation`, `Inference`, `Effect`, `Emitter`, `RunStatus` |
| R2 | Compaction = lib indépendante + décorateur provider | Goose | `omega-context` : `CompactionInput/Output`, `CompactingProvider` |
| R3 | Providers déclaratifs JSON + `KeyResolver` injecté | Goose | `omega-providers` : `definitions/*.json` |
| R4 | Registre runtime à constructeurs + `ProviderType` | Goose | `omega-registry` : `Arc<dyn Fn(..) -> BoxFuture<Result<Arc<dyn Provider>>>>` |
| R5 | Contributeurs typés par cycle de vie + builder→registre immuable | Codex | `omega-plugin-api` : ~8-10 traits (pas 15) |
| R6 | Config TOML à couches + profils + `precedence()` | Codex | `omega-config` |
| R7 | Politique de sandbox/approbation **typée** | Codex | `omega-sandbox` (types) + backends |
| R8 | Découverte `AGENTS.md` remontante + trust + plafond | Codex | `omega-context` |
| R9 | OTel nommé + cibles log-only/trace-safe | Codex | `omega-otel` |
| R10 | Hooks typés abortables multi-source | Codex | `omega-hooks` |
| R11 | Rôles de sous-agents porteurs de leur config | Codex | `omega-subagents` |
| R12 | `ToolExecutor{spec,exposure,search_info,handle}` | Codex | `omega-tools` |
| R13 | Types protocole générés vers TS + fixtures | Codex | `omega-protocol` / `omega-ui` |

### 6.2 À améliorer
| # | Sujet | Limite observée | Notre amélioration |
|---|---|---|---|
| A1 | Monolithe applicatif | `goose` reste gros ; `core` Codex hub trop chargé | Découpage strict : `core` ne dépend que de `types` + `ports` |
| A2 | Granularité extrême | ~200 crates ⇒ coût de build/navigation | Démarrer à ~10-12 crates ; fracturer **quand une frontière de crash/perf le justifie** |
| A3 | Registre par `HashMap` + closure | Goose : pas de découverte compile-time, ordre non garanti | Registre ordonné et **déterministe** (IndexMap + priorité explicite) |
| A4 | Erreurs parfois `anyhow` au fond | perte de typage aux frontières de plugin | Erreur de port typée obligatoire ; `anyhow` réservé au câblage binaire |
| A5 | Traçabilité partielle | pas de provenance de *couche de config* exposée uniformément | Exposer `layer_source` + version sur toute décision (RULES §4.1) |
| A6 | Providers codés pour certains harnais externes | Goose a `claude_code.rs`, `codex_acp.rs`, `gemini_cli.rs`… couplés | Ces harnais = **plugins** `harness-*`, jamais dans le noyau |
| A7 | Compaction non mesurée côté qualité | Goose mesure des tokens, pas la « perte d'info » | Ajouter un score de rétention (déterministe) avant/après |

### 6.3 À écarter (pour Omega)
- ❌ Les **9 couches de config enterprise/MDM** de Codex : hors périmètre perso ; garder 3-4 couches (defaults, user, projet, session).
- ❌ Le **marketplace de plugins distant** de Codex (`core-plugins/marketplace*`, `remote*`) : complexité inutile à court terme.
- ❌ Les **backends Windows** (RestrictedToken, MXC, WindowsSandboxService) : Linux-first ; garder Landlock/seccomp + bubblewrap.
- ❌ Le **V8/deno code-mode** complet : intéressant mais lourd (vendor/v8, v8-poc) ; différer P3 « exécuter du code ».
- ❌ Le **binding uniffi multi-langage** de Goose (Python/Kotlin) : viser ACP + JSON-RPC d'abord.
- ❌ La **live voice** (websocket voix) : hors mission.

---

## §7 Sources

- Goose — dépôt `https://github.com/block/goose` (Apache-2.0), clone superficiel `/tmp/opencode/re/goose` @ commit `01fa5dfe9ee1964988b237aaf9bcdd738b720299` (2026-10-14).
- Codex — dépôt `https://github.com/openai/codex` (Apache-2.0), clone superficiel `/tmp/opencode/re/codex` @ commit `1ffa95ee8a02ec68f6ccd758998bcb7d4b32bcdb` (2026-10-14).
- Fichiers de référence clés cités : voir les `file:line` en §2-§5 (chemins relatifs à la racine de chaque clone).
- Doc amont Goose : `goose/CUSTOM_DISTROS.md`.
- Contrat de production : `docs/omega/workstreams/W5-reverse-engineering.md` (étapes 1-5), `docs/omega/rules/RULES-OMEGA.md`.
- Provenance : `[observed]` pour tout ce qui porte un `file:line` ; `[stated]` pour `CUSTOM_DISTROS.md` et les README ; `[inferred]` pour les jugements de valeur/portage ; `[todo]` : vérifier la version publiée des crates au moment du portage (les deux dépôts bougent vite).

---

## Résumé final (demandé)

### Structure de workspace idéale pour Omega

**Principe :** le grain de Goose, la discipline de Codex. ~12 crates, dépendances strictement descendantes, zéro cycle.

```
omega/
  omega-types/           # types purs : Message, Conversation, ModelConfig, Usage, erreurs de domaine
  omega-ports/           # UNIQUEMENT des traits (aucune implémentation) : Provider, Tool, ContextEngine,
                         #   Hook, Sandbox, Storage, Router, Telemetry, Subagent — ports fins (ISP)
  omega-core/            # boucle d'agent (Operation/Inference/Effect/Emitter) — dépend de types+ports
  omega-context/         # compaction/revision enfichable (CompactionInput/Output + décorateur)
  omega-config/          # config TOML à couches + profils + schéma JSON (precedence() explicite)
  omega-registry/        # registres ordonnés et déterministes de plugins (par capacité, pas par marque)
  omega-plugin-api/      # contrats des contributeurs (builder → registre immuable), ~8-10 traits max
  omega-otel/            # télémétrie nommée + cibles log-only/trace-safe
  omega-protocol/        # wire types ACP/JSON-RPC, source unique → exports TS (ts-rs) + fixtures
  omega-sandbox/         # types de politique + backends Linux (landlock/seccomp, bwrap)
  omega-providers/       # providers déclaratifs (definitions/*.json) + adaptateurs par API
  plugins/               # un crate par capacité : harness-*, skills, hooks, memories, subagents, tools…
  omega-app/             # assemble : enregistre les plugins, expose le daemon
  omega-cli/             # binaire CLI
  omega-ui-bindings/     # frontière Rust↔React/TS (générée)
```

**Règles de dépendance :** `types ← ports ← core` ; `plugins → ports` (jamais l'inverse) ; `app → tout` ; `cli/ui-bindings → app/protocol`. Un port = un trait + un test d'acceptation avant tout code (RULES §5.4).

**Décisions structurantes tirées du RE :**
1. La **boucle** ne connaît ni provider ni extension : elle orchestre des `Operation` et émet des `Effect` (Goose).
2. La **compaction** est un décorateur de `Provider`, pas du code dans la boucle (Goose).
3. Les **plugins** s'enregistrent par traits fins de cycle de vie via un builder immuable en sortie (Codex).
4. La **config** est à couches avec provenance et `precedence()` testable (Codex).
5. La **frontière TS** est générée depuis les types Rust (Codex) ; les wire types ont une source unique (Goose).

### Top features à porter (par ordre de valeur/difficulté)
1. **Boucle d'agent abstraite** (Goose) — M — squelette du noyau.
2. **Context Revision enfichable + décorateur** (Goose) — M — P4.
3. **Registre d'extensions à contributeurs** (Codex, rationalisé) — L — « everything is a plugin ».
4. **Config TOML à couches + profils + schéma** (Codex) — M — pilier déclaratif.
5. **Providers déclaratifs JSON** (Goose) — S-M — routing multi-fournisseurs.
6. **Politique de sandbox/approbation typée** (Codex) — M (types) / XL (backends) — P9.
7. **Hooks typés abortables** (Codex) — M — P10.
8. **Découverte `AGENTS.md` remontante** (Codex) — S — contexte projet.
9. **Sous-agents déclarés (rôles porteurs de config)** (Codex) — M — P5.
10. **OTel nommé + log/trace-safe** (Codex) — M — économie & traçabilité.
