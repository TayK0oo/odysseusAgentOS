# RE-OPENCLAW-HERMES — Reverse-engineering de deux harnais d'agent

> **Workstream :** W5 (T1 + T2) · **Date :** 2026-10-15 · **Branche :** `feat/agentos-omega`
> **Objet :** extraire les *ports, contrats et patterns* de **OpenClaw** et **Hermes Agent**
> afin de re-spécifier proprement les briques d'AgentOS Omega. **Aucune copie de code.**
>
> **Provenance (tag sur chaque affirmation) :**
> `[stated]` = déclaré par le projet (README/docs). `[observed]` = lu dans le code source (avec `file:line`).
> `[inferred]` = synthèse/analyse de l'auteur de cette fiche. `[todo]` = à vérifier.
>
> **Méthode :** clone superficiel (`git clone --depth 1`), inventaire, lecture du code (pas du README),
> extraction des signatures réelles, matrice feature × (valeur Omega, difficulté Rust, dépendances).
> Les deux dépôts sont sous licence **MIT** (`LICENSE`), ce qui autorise la citation de signatures
> à des fins d'interopérabilité et de documentation ; on n'extrait ici que des **patterns et idées**.

---

## 1. Résumé exécutif

### 1.1 OpenClaw — `github.com/openclaw/openclaw`

**Nature :** « Multi-channel AI gateway with extensible messaging integrations » (`package.json:12` `[observed]`).
Monorepo TypeScript/pnpm de ~55 883 fichiers (~939 Mo), **179 extensions** (`extensions/`), 20 packages
internes (`packages/`), 2 crates Rust (`crates/`) et une UI React (`ui/`). Version `2026.10.3`,
licence **MIT** (`LICENSE`) `[observed]`.

**Ce qui compte pour Omega (le plus abouti du modèle « everything is a plugin ») :**

1. **Context Engine enfichable** — un port `ContextEngine` avec un cycle de vie
   `bootstrap → ingest/ingestBatch → assemble → (compact) → afterTurn → maintain → dispose`, plus
   des hooks optionnels de sous-agent (`prepareSubagentSpawn` / `onSubagentEnded`). Le flag
   `info.ownsCompaction` dit si l'engine prend la compaction à son compte ou délègue au runtime
   (`src/context-engine/types.ts:191`) `[observed]`.
2. **Slots** — pour chaque capacité « exclusive » (`memory`, `contextEngine`) **une seule**
   implémentation est active ; le reste est résolu par config (`plugins.slots.*`)
   (`src/plugins/slots.ts:12-22,81-87`) `[observed]`.
3. **Policy-par-code** — le comportement est décrit dans des **schémas typés** (zod + `openclaw.plugin.json`),
   pas dans de la prose : `configSchema`, `activation`, `contracts` sont lus **sans exécuter le code du
   plugin** (`docs/plugins/manifest.md:48-60`, `src/plugins/manifest-types.ts:504,535`) `[observed]`.
4. **Hooks typés de cycle de vie** — `api.on("before_tool_call", …)` avec un catalogue d'événements
   (before/after agent run, before/after tool call, compaction, session, message) et une sémantique
   *Gate / Modify / Observe / Claim* (`docs/plugins/hooks/reference.md:126-254`) `[observed]`.
5. **Prompt-cache awareness** — l'engine reçoit des télémétries de cache (`ContextEnginePromptCacheInfo`,
   codes de rupture `historyRewrite|compaction|pruning|pruning|systemPrompt|tools|…`)
   (`src/context-engine/types.ts:242-286`) `[observed]`.

> Le port `ContextEngine` est **le meilleur candidat de référence pour le « Context Engine enfichable »
> d'Omega** : c'est un port fin, avec un cycle de vie explicite, un flag de propriété de la compaction,
> et une télémétrie de cache. `[inferred]`

### 1.2 Hermes Agent — `github.com/NousResearch/hermes-agent`

**Nature :** « The self-improving AI agent — creates skills from experience, improves them during use,
and runs anywhere » (`pyproject.toml:5`, `README.md:20`) `[stated]`. Python 3.14, licence **MIT**
(`LICENSE`) `[observed]`. Architecture *flat* (~200 modules `hermes_state_*.py`, `agent/`, `tools/`,
`gateway/`, `cron/`), avec une forte séparation `agent/` (orchestration) ↔ `tools/` (capacités) ↔
`hermes_state*.py` (persistance SQLite/FTS5).

**Ce qui compte pour Omega :**

1. **Tool Search** — trois outils passerelles `tool_search` / `tool_describe` / `tool_call` remplacent
   les schémas des outils différables. Recherche **BM25 + stemming Snowball + gate par le token le plus
   rare** (`tools/tool_search_catalog.py:118-208`) `[observed]`. C'est le pattern le plus transposable
   directement à Omega.
2. **Progressive disclosure des skills (3 niveaux)** — `skills_list` (nom+description) →
   `skill_view` (corps complet) → `references/ templates/ assets/ scripts/` (fichiers liés)
   (`tools/skills_tool.py:2-5`) `[observed]`.
3. **Mémoire bornée + snapshot gelé** — `MEMORY.md` / `USER.md`, entrées délimitées par `\n§\n`, budget
   en **caractères** (2200 / 1375 par défaut) ; le bloc injecté dans le system prompt est un **snapshot
   figé au chargement** pour préserver le prefix-cache, alors que l'état vivant continue d'évoluer
   (`tools/memory_tool_store.py:25,175-193,618-621`) `[observed]`.
4. **`session_search` FTS5** — un outil à *forme unique* (mode inféré des arguments : DISCOVERY /
   SCROLL / READ / BROWSE), recherche SQLite FTS5, dédup par *lineage*, et **démotion** des sources
   `cron` (recall blindness) (`tools/session_search_tool.py:2-9,21-56`) `[observed]`.
5. **7 backends d'exécution** derrière une ABC `BaseEnvironment` : `local, docker, singularity, modal,
   daytona, vercel_sandbox, ssh` (+ plugin), avec un modèle *spawn-per-call* et un snapshot de session
   (`tools/environments/base.py:225,610`, `tools/terminal_tool_backends.py:54`) `[observed]`.
6. **cron + gateway messagerie** — scheduler sans démon, job schema déclaratif
   (`cron/job_definition.py:13-18`), et un **registre de plateformes** où un plugin s'auto-enregistre
   (`gateway/platform_registry.py:43,349`) `[observed]`.
7. **Boucle d'auto-amélioration** — `background_review` (fork après chaque tour : « faut-il sauver/
   patcher un skill ou une mémoire ? ») + `curator` (maintenance d'inactivité)
   (`agent/background_review.py:1-5`, `agent/curator.py:1-6`) `[observed]`.

---

## 2. Carte d'architecture

### 2.1 OpenClaw

| Zone | Chemin | Rôle | Refs |
|---|---|---|---|
| Entrée CLI | `openclaw.mjs`, `src/entry.ts`, `src/index.ts` | bootstrap, respawn, compile-cache | `package.json:29` `[observed]` |
| Gateway | `src/gateway/` | multi-canal (messagerie), HTTP RPC | `src/` `[observed]` |
| Runner agent | `src/agents/embedded-agent-runner/` | boucle de run, compaction, guards de contexte | `run/compaction-runtime.ts`, `run/attempt-prompt-phase.ts:137` `[observed]` |
| **Context engine** | `src/context-engine/` | port + registre + slots + legacy | `types.ts`, `registry.ts`, `registry-contract.ts`, `legacy.ts` `[observed]` |
| Plugins | `src/plugins/` | registre, slots, manifest, hooks, capability | `slots.ts`, `plugin-api.types.ts`, `registry-contribution-types.ts` `[observed]` |
| Memory host | `packages/memory-host-sdk/src/` | contrat mémoire enfichable (embeddings, FTS, recall) | `host/openclaw-runtime-memory.ts` `[observed]` |
| Provider/model | `src/routing/`, `src/model-catalog/`, `packages/model-catalog-core/` | routage + catalogue de modèles | `src/routing/resolve-route.ts:269` `[observed]` |
| Hooks | `src/hooks/`, `docs/plugins/hooks*.md` | hooks typés + hooks internes | `docs/plugins/hooks.md` `[observed]` |
| Extensions | `extensions/` (179) | providers, canaux, capabilities | `extensions/anthropic`, `clawrouter`, … `[observed]` |
| Rust | `crates/openclaw-gateway-client`, `crates/openclaw-node-host` | client gateway + node host en Rust | `crates/Cargo.toml:2` `[observed]` |
| UI | `ui/`, `src/web/`, `src/tui/` | React/TS + TUI | `ui/src/` `[observed]` |

**Cycle de vie du Context Engine** (orchestration runtime) :
`bootstrapHarnessContextEngine` (`src/agents/harness/context-engine-lifecycle.ts:111,137`) →
`contextEngine.assemble(...)` (`:239`) → `contextEngine.afterTurn(...)` (`:442`) ;
`contextEngine.compact(...)` (`src/agents/embedded-agent-runner/run/compaction-runtime.ts:208`) ;
`contextEngine.maintain(...)` (`src/agents/embedded-agent-runner/context-engine-maintenance-execution.ts:113`) ;
ingest/assemble/afterTurn aussi appelés depuis `tool-result-context-guard.ts:232,256,287` `[observed]`.

### 2.2 Hermes Agent

| Zone | Chemin | Rôle | Refs |
|---|---|---|---|
| Entrée CLI | `cli.py`, `run_agent.py`, `hermes_cli/` | bootstrap, commandes | `pyproject.toml` `[observed]` |
| Agent | `agent/` (~200 modules) | orchestration, prompt, auto-amélioration | `agent/prompt_builder.py`, `agent/curator.py` `[observed]` |
| Outils | `tools/` | capacités + registre + tool search | `tools/registry.py:462`, `tools/tool_search.py` `[observed]` |
| **Tool Search** | `tools/tool_search*.py` | BM25 + passerelles | `tool_search.py`, `tool_search_catalog.py`, `tool_search_validation.py` `[observed]` |
| Env d'exécution | `tools/environments/` | 7 backends derrière `BaseEnvironment` | `base.py:225`, `local.py`, `docker.py`, `ssh.py`, `singularity.py`, `modal.py`, `managed_modal.py`, `daytona.py`, `vercel_sandbox.py` `[observed]` |
| État/persistance | `hermes_state*.py` | SQLite, FTS5, migrations, WAL | `hermes_state_schema.py:977` `[observed]` |
| Cron | `cron/` | scheduler sans démon, job schema | `cron/job_definition.py:13`, `cron/scheduler.py` `[observed]` |
| Gateway messagerie | `gateway/`, `gateway/platforms/` | Telegram/Signal/WhatsApp/Weixin/Yuanbao/QQ/Webhook/API | `gateway/platform_registry.py:43` `[observed]` |
| Skills | `skills/` (bundled) + `~/.hermes/skills/` | SKILL.md + resources | `tools/skills_tool.py:67-68` `[observed]` |
| Providers | `providers/`, `plugins/` | backends de modèles | `providers/base.py` `[observed]` |

---

## 3. Ports & contrats extraits (partie centrale)

> Toutes les signatures ci-dessous sont **citées** (MIT) à des fins d'interopérabilité. Elles
> constituent le gabarit des traits Rust / interfaces TS à re-spécifier dans Omega.

### 3.1 OpenClaw

#### 3.1.1 Port `ContextEngine` — le cœur du modèle enfichable

Fichier : `src/context-engine/types.ts:355-542` `[observed]`. Méthodes **requises** : `ingest`,
`assemble`, `compact` (vérifié par `describeResolvedContextEngineContractError`,
`src/context-engine/registry-contract.ts:53-57`). Tout le reste est optionnel.

```ts
interface ContextEngine {
  readonly info: ContextEngineInfo;

  bootstrap?(p: { sessionId; sessionKey?; sessionTarget?; sessionFile;
                  runtimeSettings?; runtimeContext? }): Promise<BootstrapResult>;

  maintain?(p: { sessionId; sessionKey?; sessionTarget?; sessionFile;
                 runtimeSettings?; runtimeContext?; abortSignal? })
           : Promise<ContextEngineMaintenanceResult>;

  ingest(p: { sessionId; sessionKey?; message: AgentMessage; isHeartbeat? })
        : Promise<IngestResult>;                       // { ingested: boolean }

  ingestBatch?(p: { sessionId; sessionKey?; messages: AgentMessage[]; isHeartbeat? })
              : Promise<IngestBatchResult>;            // { ingestedCount: number }

  afterTurn?(p: { sessionId; sessionKey?; sessionTarget?; sessionFile;
                  messages: AgentMessage[]; prePromptMessageCount: number;
                  autoCompactionSummary?; isHeartbeat?; tokenBudget?;
                  runtimeSettings?; runtimeContext? }): Promise<void>;

  commitTurn?(p: { advancementKey: string; admission; terminal; messages;
                   sessionId; sessionKey?; sessionTarget?; … })
             : Promise<{ status: "committed" | "duplicate" }>;

  assemble(p: { sessionId; sessionKey?; messages: AgentMessage[]; tokenBudget?;
                availableTools?: Set<string>; citationsMode?; model?; prompt?;
                runtimeSettings?; runtimeContext? }): Promise<AssembleResult>;

  compact(p: { sessionId; sessionKey: string; agentId?; sessionTarget?; tokenBudget?;
               force?; currentTokenCount?; compactionTarget?: "budget" | "threshold";
               customInstructions?; runtimeSettings?; runtimeContext?; abortSignal? })
          : Promise<CompactResult>;

  prepareSubagentSpawn?(p: { parentSessionKey; childSessionKey;
                             contextMode?: "isolated" | "fork"; … })
                        : Promise<SubagentSpawnPreparation | undefined>;  // { rollback() }
  onSubagentEnded?(p: { childSessionKey; reason: SubagentEndReason }): Promise<void>;
  dispose?(): Promise<void>;
}
```

**Résultats (contrats de données)** `[observed]` :
- `AssembleResult = { messages: AgentMessage[]; estimatedTokens: number;
  promptAuthority?: "assembled" | "preassembly_may_overflow"; systemPromptAddition?: string;
  contextProjection?: { mode: "per_turn" | "thread_bootstrap"; epoch?; fingerprint? } }`
  (`types.ts:8-46`).
- `CompactResult = { ok: boolean; compacted: boolean; reason?; result?: { summary?;
  firstKeptEntryId?; tokensBefore; tokensAfter?; details?; sessionId?; sessionTarget? } }`
  (`types.ts:110-134`).
- `IngestResult = { ingested: boolean }` (`types.ts:156-159`).
- `BootstrapResult = { bootstrapped: boolean; importedMessages?; reason? }` (`types.ts:166-173`).

**Astuce de contrat importante** : la projection du *contexte* sur les backends à thread persistant
(`thread_bootstrap` + `epoch`) permet d'injecter le contexte **une fois par époque** au lieu de le
re-projeter à chaque tour — un gain de cache `[observed]` (`types.ts:28-46`).

#### 3.1.2 `ContextEngineInfo` + flag `ownsCompaction`

Fichier : `src/context-engine/types.ts:175-204` `[observed]`.

```ts
type ContextEngineInfo = {
  id: string; name: string; version?: string;
  acceptedHostParams?: string[];
  transcriptSemantics?: { currentTurnFence?: "before-current-turn-entry-v1";
                          turnAdvancementIdempotency?: "atomic-idempotent-v1" };
  ownsCompaction?: boolean;                 // true => l'engine gère la compaction
  turnMaintenanceMode?: "foreground" | "background";
  hostRequirements?: Partial<Record<ContextEngineOperation, ContextEngineHostRequirements>>;
};
type ContextEngineOperation = "agent-run" | "manual-compact" | "subagent-spawn";
type ContextEngineHostCapability = "bootstrap" | "assemble-before-prompt" | "after-turn"
  | "maintain" | "compact" | "runtime-llm-complete" | "thread-bootstrap-projection";
```

**Règle du flag** (`[observed]`, commentaire `types.ts:184-190`) : quand `ownsCompaction = true`,
l'hôte vérifie **chaque** requête sortante (y compris la première) à partir de l'usage mesuré du run
courant ; `estimatedTokens` de l'engine **ne contrôle jamais** l'admission de la requête. Le
runtime lit le flag pour décider s'il doit compacter lui-même
(`attempt-prompt-phase.ts:137`, `compaction-runtime.ts:562`).

**Projection filtrée des paramètres hôte** (`src/context-engine/registry-contract.ts:4-25`) : l'hôte
ne transmet que les paramètres listés dans `acceptedHostParams` (allowlist), avec une exception pour
`abortSignal` sur `compact` `[observed]`.

#### 3.1.3 Télémétrie prompt-cache (stabilité de préfixe)

Fichier : `src/context-engine/types.ts:242-286` `[observed]`.

```ts
type ContextEnginePromptCacheRetention = "none" | "short" | "long" | "in_memory" | "24h";
type ChangeCode = "historyRewrite" | "compaction" | "pruning" | "runtimeContextCarrier"
  | "aggregateToolResultTruncation" | "cacheRetention" | "model" | "streamStrategy"
  | "systemPrompt" | "systemPromptSuffix" | "tools" | "transport";
type ContextEnginePromptCacheInfo = {
  retention?; lastCallUsage?; observation?: { broke: boolean; previousCacheRead?;
  cacheRead?; changes?: { code: ChangeCode; detail: string }[] }; lastCacheTouchAt?; expiresAt?;
};
```

Le runtime fournit à l'engine un **diagnostic de rupture de cache** typé : c'est ce qui permet à un
engine « cache-aware » de savoir *pourquoi* le préfixe a changé (réécriture d'historique, compaction,
changement d'outils…) et d'éviter les ruptures évitables `[observed]`.

#### 3.1.4 `ContextEngineRuntimeSettings` — enveloppe d'exécution (schéma versionné)

Fichier : `src/context-engine/types.ts:78-108`, construction
`src/context-engine/runtime-settings.ts:29-91` `[observed]`.

```ts
type ContextEngineRuntimeSettings = {
  schemaVersion: 1;
  runtime: { host: "openclaw"; mode: "normal"|"fallback"|"degraded";
             harnessId; runtimeId };
  model: { requested; resolved; provider; family };
  contextEngineSelection: { selectedId: string|null; source: "configured"|"default"|"unknown" };
  executionHost: { id; label };
  limits: { promptTokenBudget: number|null; maxOutputTokens: number|null };
  diagnostics: { fallbackReason; degradedReason };   // codes: provider_timeout,
                                                     // provider_unavailable, rate_limited,
                                                     // context_overflow, runtime_unavailable, unknown
};
```

Pattern : **chaque frontière a un schéma versionné** (`schemaVersion: 1`) — exactement la règle
« zéro-friction frontière » d'Omega (RULES §2.5) `[inferred]`.

#### 3.1.5 Slots — une seule implémentation active par capacité

Fichiers : `src/plugins/slots.ts`, `src/plugins/slot-selection.ts`, `src/plugins/plugin-kind.types.ts` `[observed]`.

```ts
type PluginKind = "memory" | "context-engine";                       // plugin-kind.types.ts:2
type PluginSlotKey = keyof PluginSlotsConfig;                        // "memory" | "contextEngine"

const SLOT_BY_KIND = { memory: "memory", "context-engine": "contextEngine" };   // slots.ts:12-15
const DEFAULT_SLOT_BY_KEY = { memory: "memory-core", contextEngine: "legacy" }; // slots.ts:17-20

type SlotSelection =
  | { kind: "default"; pluginId: string }
  | { kind: "off" }
  | { kind: "pinned"; pluginId: string };

function resolveSlotSelection(slotKey, value): SlotSelection;        // slots.ts:81-87
function applyExclusiveSlotSelection({ config, selectedId, selectedKind }): OpenClawConfig; // slots.ts:110
```

Règles clés (commentaires `slots.ts:71-80`) `[observed]` :
- Un slot **non renseigné** = l'*owner implicite* (jamais « le plugin qui se trouve être activé »).
- `none` désactive le slot ; vide = non défini.
- Sélectionner un plugin qui déclare un `kind` **déplace** tous les slots de ce kind vers lui
  (exclusivité).
- Le choix se fait en inspectant le **manifeste** (métadonnées), pas l'inventaire du gateway en cours
  (`slot-selection.ts:15-16`).

Enumération effective : `resolveEffectiveContextEngineId(config, entries)`
(`src/context-engine/registry-selection.ts`) retombe sur `legacy` si l'engine sélectionné n'est pas
enregistré `[observed]`.

#### 3.1.6 Contrat mémoire enfichable (`MemoryPluginCapability` / `MemoryPluginRuntime`)

Fichiers : `src/plugins/registry-contribution-types.ts:271-376`, ré-export
`packages/memory-host-sdk/src/runtime-core.ts:20-40`, enregistrement
`src/plugins/plugin-api.types.ts:498` `[observed]`.

```ts
type MemoryPluginCapability = {
  promptBuilder?: MemoryPromptSectionBuilder;
  flushPlanResolver?: MemoryFlushPlanResolver;
  providerFlushPlanResolver?: MemoryProviderFlushPlanResolver;   // préféré
  runtime?: MemoryPluginRuntime;
  providerRuntime?: MemoryProviderRuntime;                       // préféré
  publicArtifacts?: MemoryPluginPublicArtifactsProvider;
  recallToolNames?: readonly string[];
  deterministicRecallToolName?: string;
  supportsPrivateTranscriptRecall?: boolean;
};

type MemoryPluginRuntime = {
  searchForCli?(p): Promise<MemoryCliSearchResult>;
  getMemorySearchManager(p: { cfg; agentId; purpose?; inspectSources? })
    : Promise<{ manager: RegisteredMemorySearchManager | null; debug?; error? }>;
  resolveMemoryBackendConfig(p: { cfg; agentId }): { backend: "builtin" };
  authorizeSearchHits?(p: { cfg; agentId; requesterSessionKey?; sandboxed; hits;
                            trustedAgentScope?; conversationRecall? })
    : Promise<MemorySearchResult[]>;
  supportsWorkspaceMemoryReadSources?: true;
  classifyWorkspaceMemoryPaths?(p): Promise<Array<{ relativePath; originClass }>>;
  prepareReload?(change): { drain(): Promise<…>; resume(): void };
  closeMemorySearchManager?(p): Promise<void>;
  closeAllMemorySearchManagers?(): Promise<void>;
};

type MemoryPluginCapabilityRegistration = {
  pluginId: string; capability: MemoryPluginCapability;
  memorySlotSelected?: boolean;   // seul l'owner du slot peut déplacer les champs à la résolution
};
```

Pattern remarquable : les providers de mémoire peuvent **délimiter / autoriser** les hits de
recherche (`authorizeSearchHits`) et **classifier l'origine** des fichiers (`MemoryOriginClass`), ce
qui permet une politique de confiance par source `[observed]`.

#### 3.1.7 API plugin (surface d'enregistrement)

Fichier : `src/plugins/plugin-api.types.ts` `[observed]`. Plus de ~60 méthodes `register*`, typées :

```
registerTool, registerHook, registerHttpRoute, registerChannel, registerGatewayMethod,
registerGatewayAccessPolicy, registerSessionCatalog, registerCli, registerNodeHostCommand,
registerNodeInvokePolicy, registerSecurityAuditCollector, registerService, registerProvider,
registerWorkerProvider, registerStorageProvider, registerModelCatalogProvider,
registerEmbeddingProvider, registerSpeechProvider, registerRealtimeTranscriptionProvider,
registerRealtimeVoiceProvider, registerMediaUnderstandingProvider,
registerTranscriptSourceProvider, registerImageGenerationProvider, registerVideoGenerationProvider,
registerMusicGenerationProvider, registerWebFetchProvider, registerMemoryCapability, …
registerAgentEventSubscription, registerRuntimeLifecycle, registerSessionSchedulerJob,
registerSessionAction, registerSessionExtension, registerControlUiDescriptor, …
```

L'entrée plugin est scellée par `definePluginEntry({ id, name, description, kind, configSchema,
reload, nodeHostCommands, securityAuditCollectors, register })` (`src/plugin-sdk/plugin-entry.ts:244`)
`[observed]`.

#### 3.1.8 Hooks typés de cycle de vie

Fichier : `docs/plugins/hooks/reference.md:126-254` `[observed]`. Catalogue (sémantique
Gate / Modify / Observe / Claim) :

| Hook | Effet | Budget / timeout |
|---|---|---|
| `before_model_resolve` | Modifier (provider/modèle/fallback) | — |
| `before_prompt_build` | Modifier (ajouter contexte, restreindre les outils du tour) | 15 s |
| `before_agent_run` | Gate (inspecter avant soumission) | 15 s |
| `before_agent_reply` | Claim (court-circuiter/afficher une réponse synthétique) | — |
| `before_agent_finalize` | Modifier (demander un tour de plus) | 15 s |
| `before_tool_call` | Modifier / Gate (réécrire params, bloquer, exiger approbation) | 15 s |
| `after_tool_call` | Observe (résultat, erreur, durée) | — |
| `before_message_write` | Sync modify / gate | (sync) |
| `before_dispatch` | Claim (traiter un message entrant) | — |
| `before_compaction` / `after_compaction` | Observe | 30 s |
| `session_start` / `session_end` | Observe | 30 s (drain 2 s au shutdown) |
| `skill_changed`, `agent_end`, `tool_result_persist` | Observe | — |

Deux systèmes de hooks coexistent : **hooks typés** (`api.on(name, handler)`) et **hooks internes**
(`HOOK.md`, événements `command:new`, `agent:bootstrap`) `[stated]` (`docs/plugins/hooks.md:24-32`).

### 3.2 Hermes Agent

#### 3.2.1 Port d'exécution `BaseEnvironment` (7 backends)

Fichiers : `tools/environments/base.py:225`, `tools/terminal_tool_backends.py:54` `[observed]`.

```python
_BUILTIN_BACKENDS = "local, docker, singularity, modal, daytona, vercel_sandbox, ssh"

class BaseEnvironment(ABC):
    def __init__(self, cwd: str, timeout: int, env: dict | None = None,
                 persist_env: bool = True): ...

    @abstractmethod
    def cleanup(self): ...

    def init_session(self): ...                     # capture du snapshot de session (env/fonctions/alias)

    def _run_bash(self, ...) -> ...: ...
    def fetch_file(self, remote_path: str, local_dest: Path, *, max_bytes: int) -> None: ...
    def fetch_realpath(self, remote_path: str) -> str | None: ...

    def execute(self, command: str, cwd: str = "", *, timeout: int | None = None,
                stdin_data: str | None = None, rewrite_compound_background: bool = True,
                bounded_capture: bool = False,
                yield_handler: Callable[[ProcessHandle, str], dict] | None = None) -> dict:
        """Retourne {"output": str, "returncode": int}. Voir base.py:610-623."""
```

Modèle (docstring `base.py:1-7`) `[observed]` : **spawn-per-call** — chaque commande lance un
`bash -c` neuf ; un snapshot de session (env/fonctions/alias) est capturé une fois et re-sourcé avant
chaque commande ; le CWD persiste via des marqueurs stdout (distant) ou un fichier temp (local).
La sortie est bornée par `tool_output.max_bytes` quand `bounded_capture=True`.

Implémentations : `local.py`, `docker.py`, `singularity.py`, `modal.py`, `managed_modal.py`,
`daytona.py`, `vercel_sandbox.py`, `ssh.py` (`tools/environments/`) `[observed]`. La sélection se fait
par `TERMINAL_ENV` (`terminal_tool._create_environment`) et des backends **plugin** peuvent s'ajouter
(`terminal_tool_backends.py:234,368`) `[observed]`.

#### 3.2.2 Tool Search — configuration + passerelles

Fichier : `tools/tool_search.py` `[observed]`.

```python
@dataclass(frozen=True)
class ToolSearchConfig:
    enabled: str                      # "auto" | "on" | "off"
    threshold_pct: float              # 0..100
    search_default_limit: int
    max_search_limit: int             # clamp 1..50
    listing: str = "auto"             # "auto"/"on"/"off"
    listing_max_tokens: int = 4000    # budget = min(ceci, threshold_pct% du contexte)
    defer_tools: Optional[frozenset] = None   # None = set curé ; [] = defer rien

    @classmethod
    def from_raw(cls, raw) -> "ToolSearchConfig": ...   # clamp + fallback sûrs

TOOL_SEARCH_NAME, TOOL_DESCRIBE_NAME, TOOL_CALL_NAME = "tool_search", "tool_describe", "tool_call"
BRIDGE_TOOL_NAMES = frozenset({...}); CHARS_PER_TOKEN = 4.0
```

Contrat des trois passerelles (`tools/tool_search.py:262-319`) `[observed]` :

- `tool_search(queries: string[], limit?: int)` → noms + descriptions des outils correspondants
  (1 requête par capacité ; recherche parallèle ; `_MAX_QUERIES_PER_CALL = 7`).
- `tool_describe(names: string[])` → schémas JSON complets (batch ; `_MAX_DESCRIBE_NAMES_PER_CALL = 10`).
- `tool_call(calls: {name, arguments}[])` → invocation batch, dispatch via `handle_function_call`.

Décision d'activation (`tools/tool_search.py:196-205`) `[observed]` :
`"off"` n'active jamais ; `"on"`/`"auto"` activent dès qu'un outil différable existe (ou qu'un compte
connecteur est autorisé). **Le budget de listing ne contrôle PAS l'activation** — il borne seulement ce
que le manifeste embarqué peut consommer (`full` → `names` → `bare`).

Fonctions de contrat : `assemble_tool_defs(tool_defs, *, context_length, config) -> AssemblyResult`
(`:335`), `classify_tools(tool_defs) -> (visible, deferrable)` (`:169`),
`is_deferrable_tool_name(name, defer_tools)` (`:149`), `resolve_underlying_call(args)` (`:556`),
`scoped_deferrable_names(tool_defs)` (`:532`, isolation du scope de session).

#### 3.2.3 Recherche BM25 + gate par token le plus rare

Fichier : `tools/tool_search_catalog.py` `[observed]`.

```python
@dataclass
class CatalogEntry:
    name: str; description: str; schema: dict; source: str      # "mcp" | "plugin" | "other"
    source_name: str; _tokens: list[str]                        # pré-tokenisé pour BM25

def _stem(token: str) -> str            # Snowball "english", mémoïsé (lru_cache 16384)
def _tokenize(text: str) -> list[str]   # [a-z0-9]+, stemmés (index ET requête)

def _bm25_score(query_tokens, doc_tokens, doc_lengths, avg_dl, doc_freq, n_docs,
                k1=1.5, b=0.75) -> float: ...                      # BM25 standard inliné

def _gate_token(query_tokens, doc_freq, n_docs) -> str: ...      # token de plus haut IDF

MIN_QUERY_TERM_COVERAGE = 0.5
MIN_ANSWERABLE_TERMS_FOR_COVERAGE = 4

def search_catalog(catalog, query, limit=5, *, corpus_stats=None) -> list[CatalogEntry]:
    """Top-limit par BM25 (match exact du nom en premier). Admission par le token le plus
    rare (<gate>) ET couverture suffisante des termes répondables (requêtes longues)."""
```

Trois idées fortes `[observed]` :
1. **Gate par le token le plus rare** — un document doit contenir le token de plus haut IDF de la
   requête (le mot qui *nomme l'intention*), sinon il est écarté, quel que soit son score additif
   (`_gate_token`, `:145-153`). Empêche « send gmail email » de retourner 5 outils qui ne partagent
   que `email`.
2. **Couverture minimale** pour les requêtes longues (≥ 4 termes répondables → ≥ 50 % de
   couverture) — évite les longues chasses où chaque mot existe *quelque part* mais aucun outil n'en
   porte plus d'un (`:156-173`).
3. **Listing byte-stable et dégradant par serveur** — l'ordre est trié déterministe pour que le
   préfixe de requête reste cachable ; les plus gros serveurs sont résumés en premier
   (`build_catalog_listing_with_form`, `:260-324`) `[observed]`.

Validation : `tools/tool_search_validation.py` (`normalize_tool_call_entries`,
`not_deferrable_error`, `validate_deferred_call_args`) `[observed]`.

#### 3.2.4 Registre d'outils (`ToolEntry` / `ToolRegistry`)

Fichier : `tools/registry.py` `[observed]`.

```python
@dataclass(eq=False, slots=True)
class ToolEntry:                      # registry.py:216-232
    name: str
    toolset: str
    schema: dict
    handler: Callable
    check_fn: Optional[Callable]      # disponibilité (imports/deps/capacités)
    requires_env: list
    is_async: bool
    description: str
    emoji: str
    max_result_size_chars: int | float | None = None
    dynamic_schema_overrides: Optional[Callable] = None   # merge à chaque get_definitions()
    execution_timeout: float | None = None

class ToolRegistry:                   # registry.py:462
    def register(...)                 # :709
    def get_definitions(self, tool_names: set[str], quiet: bool = False) -> list[dict]: ...  # :886
```

`_PluginOverridePolicy` (`:235-246`) : un module plugin ne peut **pas** écraser un outil natif sans
opt-in opérateur (`allow_tool_override`). Les `check_fn` sont mémoïsées avec scope et invalidation
(`:298-460`). `_HERMES_CORE_TOOLS` (`toolsets.py:11`) liste les outils qui ne se diffèrent jamais.

#### 3.2.5 Mémoire bornée + snapshot gelé

Fichier : `tools/memory_tool_store.py` `[observed]`.

```python
MEMORY_BLOCK_HEADERS = {"memory": "MEMORY (your personal notes)",
                        "user": "USER PROFILE (who the user is)"}
ENTRY_DELIMITER = "\n§\n"

class MemoryStore:                    # :175
    def __init__(self, memory_char_limit=2200, user_char_limit=1375, *,
                 memory_enabled=True, user_profile_enabled=True):
        self.memory_entries: list[str] = []
        self.user_entries: list[str] = []
        self._system_prompt_snapshot: dict[str, str] = {"memory": "", "user": ""}  # gelé au load

    def load_from_disk(self):         # :221  -> remplit le snapshot (assaini, threat-scanné)
    def add(self, target, content) -> dict: ...            # :369
    def replace(self, target, old_text, new_content, matched_entry=None) -> dict: ...  # :392
    def patch(self, target, old_text, new_text, matched_entry=None) -> dict: ...       # :406
    def remove(self, target, old_text, matched_entry=None) -> dict: ...                # :441
    def apply_batch(self, target, operations) -> dict: ...                             # :539 (all-or-nothing)
    def format_for_system_prompt(self, target) -> Optional[str]:                       # :618
        """Snapshot gelé au chargement (PAS l'état vivant) — préserve le prefix-cache."""
```

Patterns remarquables `[observed]` :
- **Budget en caractères** (indépendant du modèle), `handle` d'usage `x/y chars` dans le bloc prompt.
- **Snapshot vs état vivant** : le system prompt voit le snapshot figé ; les écritures n'y touchent
  pas → le préfixe reste cachable (`:618-621`).
- **Consolidation transactionnelle** : `apply_batch` est *all-or-nothing* et vérifie le budget sur le
  résultat final, donc une seule opération peut *libérer et ajouter* (`:539-616`).
- **Garde-fous** : détection de *drift* externe (texte libre non délimité → sauvegarde `.bak` et refus
  d'écrire, `:685-700`) ; refus d'un read échoué (`:63-69`) ; scan d'injection sur chaque entrée
  (`_scan_memory_content`, `:28-31`) ; plafond de 3 échecs de consolidation par tour (`:184,204-219`).
- **Patch vs Replace** : `replace` écrase l'entrée entière, `patch` remplace un *span* unique et
  rejette un span ambigu (`:406-417`).

#### 3.2.6 `session_search` FTS5

Fichier : `tools/session_search_tool.py` `[observed]`.

```python
def session_search(query: str = "", role_filter: str | None = None, limit: int = 3,
                   db=None, ...) -> str:
    """Forme unique, mode inféré des args :
       DISCOVERY (query) | SCROLL (session_id + around_message_id) |
       READ (session_id) | BROWSE (aucun arg). Aucun appel LLM."""

_HIDDEN_SESSION_SOURCES = ("kanban", "subagent", "tool")     # hors recherche/navigation
_DEMOTED_SESSION_SOURCES = ("cron",)                          # cherchable mais démotionné
_READ_MAX_CONTENT = 2000        # cap par message en lecture
_DISCOVER_SCAN_LIMIT = 300      # lignes FTS scannées avant dédup lineage
```

Side SQL : schéma FTS5 avec probe de disponibilité et tokenizer trigram
(`hermes_state_schema.py:212-220,692-695,977-1003`) `[observed]`. Détails d'implémentation :
`_resolve_lineage` (`:123`) déduplique par *root de lineage* (compression-splits vs délégations),
`_parse_iso_bound` (`:127-140`) accepte bornes ISO **et** relatives (`7d`/`24h`/`2w`).

#### 3.2.7 Cron — job schema

Fichier : `cron/job_definition.py:13-18` `[observed]`.

```python
JOB_DEFINITION_FIELDS = frozenset({
  "name", "prompt", "skills", "skill", "model", "provider", "base_url",
  "script", "no_agent", "monitor_script", "monitor_url", "context_from",
  "schedule", "schedule_display", "deliver", "origin", "enabled_toolsets",
  "workdir", "attach_to_session", "reasoning_effort", "failure_deliver"})
```

Séparation nette entre champs **authored** (`create_job`) et état **scheduler** (`next_run_at`,
`state`, compteurs) ; l'import d'un store étranger préserve l'état local (`merge_job_definition`,
`:21-45`). Le scheduler est **sans démon** : `scheduler.py`, `scheduler_tick.py`, `occurrences.py`
`[observed]`. Livraison cron → canal via `deliver=<plateforme>` et `cron_deliver_env_var`
(voir 3.2.8).

#### 3.2.8 Registre de plateformes (gateway messagerie)

Fichier : `gateway/platform_registry.py:43-60,349,435` `[observed]`.

```python
@dataclass
class PlatformEntry:
    name: str                                   # identifiant config.yaml (ex "irc")
    label: str
    adapter_factory: Callable[[Any], Any]       # PlatformConfig -> adapter
    check_fn: Callable[[], bool]                # sonde PASSIVE (sans side-effect)
    validate_config: Optional[Callable[[Any], bool]] = None
    ...                                          # hooks optionnels (env_enablement_fn, …)

class PlatformRegistry:                          # (instance)
    def register(self, entry: PlatformEntry, *, scope: Optional[str] = None) -> None: ...  # :349
    def registered_names(self) -> set[str]: ...                                            # :412
    def create_adapter(self, name: str, config: Any) -> Optional[Any]: ...                 # :435
```

Le path **plugin** (`~/.hermes/plugins/<x>/plugin.yaml` + `adapter.py`) s'enregistre via
`ctx.register_platform()` — **zéro modification du core** (`gateway/platforms/ADDING_A_PLATFORM.md:1-20`)
`[stated]`. Hooks optionnels : `env_enablement_fn`, `apply_yaml_config_fn`, `is_connected`,
`cron_deliver_env_var`, `standalone_sender_fn` `[stated]`. Adaptateurs présents : `signal.py`,
`whatsapp_cloud.py`, `weixin.py`, `yuanbao.py`, `qqbot/`, `webhook.py`, `bluebubbles.py`,
`msgraph_webhook.py`, `tcp_site.py`, `api_server.py` `[observed]`.

#### 3.2.9 Boucle d'auto-amélioration

Fichiers : `agent/background_review.py`, `agent/curator.py`, `agent/learning_graph.py`,
`agent/learning_mutations.py` `[observed]`.

```python
# background_review.py — après CHAQUE tour
def prepare_background_review_run(agent) -> Optional[_BackgroundReviewRun]: ...   # :75
def spawn_background_review_thread(...): ...                                     # :1310
# Le fork hérite du runtime vivant (provider/model/credentials/system prompt caché)
# => même prefix-cache ; écrire va directement vers memory + skill stores.

# curator.py — déclenché par INACTIVITÉ (pas de démon cron)
def maybe_run_curator(*, idle_for_seconds=None, on_summary=None) -> Optional[dict]: ...  # :1198
DEFAULT_INTERVAL_HOURS, DEFAULT_MIN_IDLE_HOURS = 24*7, 2
DEFAULT_STALE_AFTER_DAYS, DEFAULT_ARCHIVE_AFTER_DAYS = 14, 30
def apply_automatic_transitions(now=None) -> dict[str, int]: ...                 # :209
```

Invariants `[observed]` (docstring `curator.py:1-6`) : seuls les skills gérés par le curator sont
touchés ; **jamais de suppression, seulement archivage** (récupérable via `hermes curator restore`) ;
les skills *pinned* échappent à toute transition automatique ; le fork utilise le client auxiliaire et
ne touche pas le prompt-cache de la session principale.

---

## 4. Features détaillées + matrice

**Échelle valeur Omega :** `0` = rien · `1` = utile · `2` = important · `3` = fondamental.
**Difficulté portage Rust :** `S` ≤ 1 j · `M` 1–3 j · `L` > 3 j / équipe.

| # | Feature | Harnais | Valeur | Rust | Dépendances |
|---|---|---|---|---|---|
| F-OC-1 | Context Engine enfichable (`ingest/assemble/compact/afterTurn`) + `ownsCompaction` | OpenClaw | **3** | **L** | runtime de run, transcript store |
| F-OC-2 | Slots (une impl exclusive par capacité) | OpenClaw | **3** | **S** | config typée |
| F-OC-3 | Manifeste déclaratif + policy-par-code (schema zod, `contracts`, activation sans exécution) | OpenClaw | **3** | **M** | serde/JSON-Schema |
| F-OC-4 | Hooks typés de cycle de vie (Gate/Modify/Observe/Claim + budgets) | OpenClaw | **3** | **M** | bus d'événements |
| F-OC-5 | Télémétrie prompt-cache + codes de rupture | OpenClaw | **3** | **M** | provider usage hooks |
| F-OC-6 | Hooks de sous-agents (`prepareSubagentSpawn` rollback, `contextMode isolated\|fork`) | OpenClaw | **3** | **M** | sous-agents, sessions |
| F-OC-7 | Tool Search natif (`tool_search`/`tool_describe`/`dispatch_action`, directory scalable) | OpenClaw | 2 | M | catalogue d'outils |
| F-OC-8 | Capability registry (~60 méthodes `register*`, 179 extensions) | OpenClaw | 2 | L | chargement plugin, sandbox |
| F-OC-9 | Cœur Rust `openclaw-gateway-client` / `openclaw-node-host` | OpenClaw | 2 | S | tokio, serde |
| F-H-1 | Tool Search BM25 + stemming + rarest-token gate + coverage + listing budgeté | Hermes | **3** | **M** | stemmer Snowball |
| F-H-2 | Progressive disclosure des skills (3 niveaux) | Hermes | **3** | **S** | frontmatter YAML |
| F-H-3 | Mémoire bornée + snapshot gelé (prefix-cache) | Hermes | **3** | **S/M** | FS, hash |
| F-H-4 | `session_search` FTS5 (forme unique, dédup lineage, démotion source) | Hermes | 2 | **M** | SQLite/FTS5 |
| F-H-5 | 7 backends d'exécution derrière `BaseEnvironment` | Hermes | **3** | **L** | docker/ssh/sandbox SDKs |
| F-H-6 | Cron + gateway messagerie (registre de plateformes, path plugin) | Hermes | 2 | M | scheduler, adaptateurs |
| F-H-7 | Boucle d'auto-amélioration (background review + curator) | Hermes | 2 | M/L | fork agent, scoring |
| F-H-8 | Plugin override policy (`allow_tool_override`, identité) | Hermes | 2 | **S** | registre |

### 4.1 F-OC-1 — Context Engine enfichable

- **Ce que ça fait.** Remplace toute la gestion de contexte (ingestion, assemblage sous budget,
  compaction, maintenance post-tour) par un port unique ; l'engine peut posséder la compaction.
- **Implémentation.** Port (`types.ts:355`), validation de forme (`registry-contract.ts:27-62`),
  registre + fallback `legacy` (`registry.ts`, `legacy.ts`), orchestration
  (`context-engine-lifecycle.ts:111,239,442`). `LegacyContextEngine` enveloppe le comportement
  pré-plugin (ingest no-op, assemble pass-through, compact délégué) pour **100 % compat**
  (`legacy.ts:12-38`) `[observed]`.
- **Détails portables.** Allowlist de paramètres (`acceptedHostParams`), projection
  `per_turn|thread_bootstrap` avec `epoch`, `hostRequirements` par opération, `turnMaintenanceMode`
  foreground/background, rollback de spawn de sous-agent.
- **Valeur Omega 3.** C'est exactement le pilier P1 et le credo « everything is a plugin ».
- **Difficulté Rust L.** Le port est fin, mais l'orchestration (budgets, compaction, projection de
  thread, garde-fous) est la partie coûteuse.
- **Test d'acceptation (esquisse).** Un engine de test qui possède la compaction et renvoie un
  `AssembleResult` borné doit (a) être chargé par le registre via slot, (b) recevoir
  `bootstrap→ingest→assemble→afterTurn`, (c) bloquer la compaction native quand
  `ownsCompaction=true`, (d) ne recevoir que les champs listés dans `acceptedHostParams`.

### 4.2 F-OC-2 — Slots

- **Ce que ça fait.** Garantit qu'une capacité exclusive (mémoire, context engine) n'a **qu'une**
  implémentation active, résolue par config, avec un owner implicite par défaut.
- **Implémentation.** `slots.ts:12-22,55-107`, `slot-selection.ts:9-56`, résolution effective
  `registry-selection.ts` `[observed]`.
- **Valeur Omega 3.** Répond directement à RULES §1.5 (capability-first) et à l'objectif
  « une seule implémentation active par capacité ».
- **Difficulté Rust S.** Une map `kind → slot`, une résolution par config, une exclusivité.

### 4.3 F-OC-3 — Manifeste déclaratif + policy-par-code

- **Ce que ça fait.** Le comportement d'un plugin est décrit dans `openclaw.plugin.json`
  (`configSchema`, `activation`, `contracts`) **sans exécuter son code** ; validation stricte via zod.
- **Implémentation.** `docs/plugins/manifest.md:48-60`, types `manifest-types.ts:504,535`,
  `definePluginEntry(… configSchema …)` `[observed]`. Le champ `contracts` est un *snapshot statique
  de propriété de capacités* (auth, embeddings, speech, génération média, web fetch/search, outils).
- **Valeur Omega 3.** Traduit « Configuration > code » (RULES §1.4) et « zéro-friction frontière »
  (§2.5) : un manifeste validé par JSON Schema = contrat avant chargement.
- **Difficulté Rust M.** serde + JSON Schema ; l'astuce « lire les métadonnées sans exécuter le
  code » exige une séparation nette manifest ↔ runtime.

### 4.4 F-OC-4 — Hooks typés

- **Ce que ça fait.** Points d'extension déclarés (`api.on`), avec budget de temps et sémantique
  explicite (bloquer / modifier / observer / réclamer).
- **Implémentation.** Catalogue `docs/plugins/hooks/reference.md:126-254`, runner typé + `api.on`
  (`plugin-api.types.ts:241`) `[observed]`.
- **Valeur Omega 3.** Pilier P10 ; indispensable pour une politique d'approbation et pour
  l'interception d'outils.
- **Difficulté Rust M.** Bus d'événements async + budgets + ordre déterministe.

### 4.5 F-OC-5 — Prompt-cache awareness

- **Ce que ça fait.** Donne à l'engine un diagnostic *typé* des ruptures de cache et l'usage réel du
  dernier appel.
- **Implémentation.** `types.ts:242-286` (`ContextEnginePromptCacheInfo`), codes de rupture
  (`historyRewrite|compaction|pruning|systemPrompt|tools|…`) `[observed]`.
- **Valeur Omega 3.** Fait partie du pilier P4 (compaction + stabilité de préfixe) et de la mentalité
  économique (mesurer, pas supposer).
- **Difficulté Rust M.** Dépend de la capacité des adaptateurs de providers à remonter l'usage cache.

### 4.6 F-OC-6 — Hooks de sous-agents

- **Ce que ça fait.** `prepareSubagentSpawn` permet de préparer l'état du contexte pour un enfant et de
  renvoyer un `rollback()` ; `contextMode: "isolated" | "fork"` ; `onSubagentEnded(reason)`.
- **Implémentation.** `types.ts:206-238,522-536` ; raisons de fin
  `"deleted"|"completed"|"swept"|"released"` `[observed]`.
- **Valeur Omega 3.** Pilier P5 (sous-agents isolés parallèles) + gestion de cycle de vie propre.
- **Difficulté Rust M.** Proche de F-OC-1 mais localisé.

### 4.7 F-H-1 — Tool Search BM25 (la transposition la plus directe)

- **Ce que ça fait.** Remplace les schémas des outils différables par 3 passerelles ; recherche
  lexicale avec gate par le token le plus rare.
- **Implémentation.** `tools/tool_search.py`, `tools/tool_search_catalog.py` (BM25 `:118-131`, gate
  `:145-153`, coverage `:156-173`, listing `:260-324`) `[observed]`.
- **Valeur Omega 3.** Pilier P3. Directement transposable : un index BM25 borné (< 500 outils),
  un stemmer, un gate d'admission.
- **Difficulté Rust M.** BM25 + stemmer (`rust-stemmers` cripple) + budget de listing byte-stable ;
  aucun ML.
- **Test d'acceptation (esquisse).** Sur un catalogue de ~200 outils, « send gmail email » ne doit
  retourner que des outils contenant un token de la famille `gmail`, jamais un outil qui ne partage
  que `email` ; une requête dont le token rare n'existe nulle part retourne un groupe vide avec hint.

### 4.8 F-H-2 — Progressive disclosure des skills (3 niveaux)

- **Ce que ça fait.** Niveau 1 : `skills_list` = nom + description. Niveau 2 : `skill_view` = corps +
  fichiers liés. Niveau 3 : `references/`, `templates/`, `assets/`, `scripts/`.
- **Implémentation.** docstring `tools/skills_tool.py:2-5` ; cache de découverte par signature
  mtime/TTL 30 s (`:34-39`) ; frontmatter YAML préprocessé (`agent/skill_preprocessing.py:142`) ;
  filtres de pertinence offer-time (plateforme/environnement/apps) (`:139-142`) `[observed]`.
- **Valeur Omega 3.** Pilier P2 + RULES « le déterministe au système » (listing = scan FS, pas LLM).
- **Difficulté Rust S.** Walk FS + parse frontmatter + render.

### 4.9 F-H-3 — Mémoire bornée + snapshot gelé

- **Ce que ça fait.** Une mémoire curatoriale bornée en caractères, persistée en fichiers, dont le
  bloc injecté est gelé pour stabiliser le préfixe.
- **Implémentation.** `tools/memory_tool_store.py` (voir §3.2.5) `[observed]`.
- **Valeur Omega 3.** Pilier P4 + économie (budget en caractères, pas en tokens).
- **Difficulté Rust S/M.** Le cœur est simple ; la difficulté vient des garde-fous (drift, lock,
  scan d'injection, batch atomique) — à porter tels quels.

### 4.10 F-H-4 — `session_search` FTS5

- **Ce que ça fait.** Rappel de conversations passées sans appel LLM, via SQLite FTS5.
- **Implémentation.** `tools/session_search_tool.py` (voir §3.2.6), schéma FTS5
  (`hermes_state_schema.py`) `[observed]`.
- **Valeur Omega 2.** Utile (mémoire de sessions) ; FTS5 est disponible en Rust via `rusqlite`.
- **Difficulté Rust M.** Les quatre formes + dédup lineage + démotion de source.

### 4.11 F-H-5 — 7 backends d'exécution

- **Ce que ça fait.** Une ABC unique, plusieurs contextes d'exécution, sélection par
  `TERMINAL_ENV`, backends plugin possibles.
- **Implémentation.** `tools/environments/base.py:225,610`, `tools/terminal_tool_backends.py:54,234,368` `[observed]`.
- **Valeur Omega 3.** Pilier P9 (sandbox multi-backend) et « self-hosted compris en profondeur ».
- **Difficulté Rust L.** Docker/SSH/Singularity/Modal/Daytona/Vercel = autant d'adaptateurs lourds ;
  le *port* est simple, les implémentations coûteuses.

### 4.12 F-H-6 — Cron + gateway

- **Ce que ça fait.** Automatisations planifiées livrées vers n'importe quelle plateforme de
  messagerie, avec un path plugin à zéro modification du core.
- **Implémentation.** `cron/job_definition.py:13-18`, `gateway/platform_registry.py:43-60,349,435`,
  `gateway/platforms/ADDING_A_PLATFORM.md:1-20` `[observed]`/`[stated]`.
- **Valeur Omega 2.** Utile (agents planifiés, notifications) ; non central au cœur d'Omega.
- **Difficulté Rust M.** Scheduler sans démon + registre d'adaptateurs.

### 4.13 F-H-7 — Boucle d'auto-amélioration

- **Ce que ça fait.** Après chaque tour, un fork évalue « faut-il sauver/patcher un skill ou une
  mémoire ? » ; en inactivité, le curator archive/consolide/répare.
- **Implémentation.** `agent/background_review.py:75,1310`, `agent/curator.py:1-6,209,1198` `[observed]`.
- **Valeur Omega 2.** Intéressant pour le pilier adaptatif, mais introduit du coût LLM
  (conflit potentiel avec la règle « le coût va au système »). À porter **désactivé par défaut**.
- **Difficulté Rust M/L.** Fork isolé + scoring + garde-fous de coût.

### 4.14 F-H-8 — Plugin override policy

- **Ce que ça fait.** Un plugin ne peut pas écraser un outil natif sans opt-in opérateur.
- **Implémentation.** `_PluginOverridePolicy` + `_OVERRIDE_DENIED_MSG` (`tools/registry.py:235-246`) `[observed]`.
- **Valeur Omega 2.** Sécurité de la supply-chain des plugins (le pyproject de Hermes cite
  explicitement la worm « Mini Shai-Hulud » et les pins exacts — `pyproject.toml:24-31` `[stated]`).
- **Difficulté Rust S.**

---

## 5. Ce qu'Omega reprend / améliore / écarte

### 5.1 À reprendre tel quel (patterns)

| Pattern | Source | Justification |
|---|---|---|
| Port `ContextEngine` + `ownsCompaction` | OpenClaw F-OC-1 | Contrat fin, cycle de vie explicite, propriété de compaction déclarée. |
| Slots (exclusivité par capacité) | OpenClaw F-OC-2 | « Un contrat, N implémentations, une active » — RULES §1. |
| Manifeste validé par schéma, lu sans exécuter le code | OpenClaw F-OC-3 | Configuration > code ; contrat avant chargement. |
| Hooks typés Gate/Modify/Observe/Claim + budgets | OpenClaw F-OC-4 | Politique d'approbation et interception d'outils. |
| Télémétrie prompt-cache typée | OpenClaw F-OC-5 | Rendre la stabilité de préfixe **mesurable**. |
| Tool Search BM25 + rarest-token gate + coverage | Hermes F-H-1 | Réduit la surface de tokens sans perdre le recall ; déterministe, pas de LLM. |
| Progressive disclosure skills 3 niveaux | Hermes F-H-2 | Pilier P2, coût système (scan FS). |
| Mémoire bornée + snapshot gelé | Hermes F-H-3 | Préserve le prefix-cache ; budgets en caractères. |
| Schémas de frontières versionnés (`schemaVersion`) | OpenClaw §3.1.4 / Hermes §3.2.1 | RULES §2.5. |

### 5.2 À améliorer pour Omega

1. **Rust-first / zéro-copie.** Les deux harnais sont mono-langage (TS ou Python). Omega doit
   exprimer ces ports en **traits Rust** avec bindings TS pour l'UI (RULES §1.2) : c'est une
   amélioration, pas une reprise. `[inferred]`
2. **Lexique unifié.** OpenClaw dit « context engine / slot / capability » ; Hermes dit « tool search
   / backend / plateforme ». Omega doit unifier sous **capacité + port + adaptateur** (RULES §1.5).
   `[inferred]`
3. **Routage multi-modèles intégré.** OpenClaw a un catalogue de modèles (`packages/model-catalog-core/`)
   et Hermes un provider-agnosticisme ; Omega en fait un **registre noté par la communauté** avec
   routage par capacité (pilier P6) — au-delà des deux. `[inferred]`
4. **Coût mesuré, pas supposé.** Hermes ajoute un fork LLM après **chaque** tour (`background_review`) :
   excellent pour l'adaptatif, mais contraire à « le coût va au système ». Omega le porte
   **opt-in** avec plafond de budget. `[inferred]`
5. **Provenance & traçabilité.** Aucun des deux n'impose un tag de provenance sur ses faits ;
   Omega le fait (RULES §4.1). On garde la télémétrie d'OpenClaw comme *moyen* de preuve. `[inferred]`

### 5.3 À écarter (ou fortement encadrer)

| Écarté | Source | Raison |
|---|---|---|
| La surface `register*` complète (~60 méthodes) | OpenClaw F-OC-8 | Trop large pour Omega ; on garde un **cœur** (tools, hooks, provider, channel, memory) et on extensibilise le reste par contrat. `[inferred]` |
| Le fork LLM automatique après chaque tour | Hermes F-H-7 | Coût LLM non plafonné par défaut ; contraire à RULES §3.1-3.3. À reprendre en opt-in. |
| Un `PluginKind` à 2 valeurs seulement | OpenClaw §3.1.5 | Omega a besoin de plus de capacités (`routing`, `memory`, `search`, `sandbox`…) dans le registre. `[inferred]` |
| Le monolithe Python « flat » de Hermes | Hermes §2.2 | Omega vise hexagonal + microservices ciblés (RULES §2.1,2.3) ; on garde les *idées*, pas la structure. `[inferred]` |

### 5.4 Ce que cela change pour les prochaines briques d'Omega

- Écrire d'abord le **trait `ContextEngine`** (port) + un `LegacyContextEngine` (compat) + le test
  d'acceptation de F-OC-1, avant toute implémentation (RULES §4.4).
- Les **slots** et le **manifeste validé** sont des prérequis au chargement de plugin : à spécifier en
  même temps (blocage mutuel sinon).
- Le **Tool Search BM25** est le chantier le plus rentable *et* le plus isolé (F-H-1, Rust M) : bon
  premier candidat pour prouver la chaîne « port → contrat → test ».

---

## 6. Sources

### 6.1 Corpus local (clone superficiel `--depth 1`, `/tmp/opencode/re/`)

- **OpenClaw** : commit `86cdc5a0` (`test(plugins): complete MCP Events repository inventories`),
  version `2026.10.3`, licence MIT (`LICENSE:1`).
- **Hermes Agent** : licence MIT (`LICENSE:1`), `pyproject.toml:5` `version = "0.0.0"`.

### 6.2 OpenClaw — fichiers cités

| Fichier | Lignes | Contenu |
|---|---|---|
| `src/context-engine/types.ts` | 8-46, 110-204, 206-238, 242-286, 299-347, 355-542 | Port, résultats, info, prompt-cache, runtime context |
| `src/context-engine/registry-contract.ts` | 4-25, 27-62 | Allowlist de params + validation de forme |
| `src/context-engine/runtime-settings.ts` | 29-91 | Enveloppe d'exécution versionnée |
| `src/context-engine/registry-selection.ts` | 1-28 | Résolution effective / fallback |
| `src/context-engine/legacy.ts` | 12-38 | Engine legacy (compat 100 %) |
| `src/context-engine/registry.ts` | — | Registre, métadonnées, ressources |
| `src/plugins/slots.ts` | 12-22, 55-107, 110-154 | Kinds → slots, résolution, exclusivité |
| `src/plugins/slot-selection.ts` | 9-56 | Application par plugin |
| `src/plugins/plugin-kind.types.ts` | 1-3 | `PluginKind` |
| `src/plugins/registry-contribution-types.ts` | 271-376 | `MemoryPluginRuntime` / `MemoryPluginCapability` |
| `src/plugins/plugin-api.types.ts` | 87-498 | Surface `register*` |
| `src/plugins/manifest-types.ts` | 504, 535, 441 | `activation`, `contracts`, `kind` |
| `src/plugin-sdk/plugin-entry.ts` | 244-275 | `definePluginEntry` |
| `src/agents/harness/context-engine-lifecycle.ts` | 111, 137, 239, 442 | bootstrap/assemble/afterTurn |
| `src/agents/embedded-agent-runner/run/compaction-runtime.ts` | 208, 562 | compact + check `ownsCompaction` |
| `src/agents/embedded-agent-runner/run/attempt-prompt-phase.ts` | 137 | Propagation de `ownsCompaction` |
| `src/agents/embedded-agent-runner/context-engine-maintenance-execution.ts` | 113 | `maintain` |
| `src/agents/embedded-agent-runner/tool-result-context-guard.ts` | 232, 256, 287 | afterTurn/ingest/assemble |
| `packages/memory-host-sdk/src/runtime-core.ts` | 1-40 | Ré-export du contrat mémoire |
| `docs/plugins/manifest.md` | 35, 48-60, 259 | Manifeste + `contracts` |
| `docs/plugins/hooks.md` | 24-32 | Deux systèmes de hooks |
| `docs/plugins/hooks/reference.md` | 126-254 | Catalogue + sémantique + budgets |
| `docs/tools/tool-search.md` | 17-40, 91-94, 160-187 | Tool Search natif (3 contrôles) |
| `crates/Cargo.toml` | 2 | Crates Rust |
| `package.json` | 12, 29 | Description, binaires |

### 6.3 Hermes Agent — fichiers cités

| Fichier | Lignes | Contenu |
|---|---|---|
| `tools/tool_search.py` | 1-7, 41-84, 169-208, 262-319, 335-370, 430-585 | Config, classification, passerelles, dispatch |
| `tools/tool_search_catalog.py` | 18-35, 42-55, 118-208, 260-324 | BM25, gate, coverage, listing |
| `tools/tool_search_validation.py` | — | Validation des appels passerelles |
| `tools/registry.py` | 216-246, 462, 709, 886 | `ToolEntry`, override policy, registre |
| `tools/skills_tool.py` | 2-5, 34-44, 139-142 | Progressive disclosure, cache |
| `agent/skill_preprocessing.py` | 24-142 | Frontmatter, préprocess |
| `tools/memory_tool_store.py` | 25, 175-193, 221, 369-616, 618-621, 685-700 | `MemoryStore` + snapshot + garde-fous |
| `tools/memory_tool.py` | — | Cycle de vie, dirs mémoire |
| `tools/session_search_tool.py` | 1-56, 123-140, 601 | 4 formes, lineage, démotion |
| `hermes_state_schema.py` | 212-220, 692-695, 977-1007 | FTS5, probe, migrations |
| `tools/environments/base.py` | 1-7, 225, 610-623 | `BaseEnvironment`, `execute` |
| `tools/environments/__init__.py` | 1-5 | Liste des backends |
| `tools/terminal_tool_backends.py` | 54, 234, 368 | `_BUILTIN_BACKENDS`, sélection |
| `cron/job_definition.py` | 13-45 | Schema de job + merge |
| `gateway/platform_registry.py` | 43-60, 349, 412, 435 | `PlatformEntry` + registre |
| `gateway/platforms/ADDING_A_PLATFORM.md` | 1-50 | Path plugin, hooks |
| `agent/background_review.py` | 1-5, 75, 1310 | Boucle post-tour |
| `agent/curator.py` | 1-6, 209, 1198 | Curator / inactivité |
| `agent/learning_mutations.py` | 1-14 | Édition/suppression nœuds |
| `toolsets.py` | 11-71, 247 | `_HERMES_CORE_TOOLS`, surfaces |
| `pyproject.toml` | 5, 24-31 | Description, politique de pins |
| `README.md` | 20-40 | Positionnement, 7 backends, plateformes |

### 6.4 URLs

- OpenClaw — <https://github.com/openclaw/openclaw> (MIT, © 2026 OpenClaw Foundation).
- Hermes Agent — <https://github.com/NousResearch/hermes-agent> (MIT, © 2025 Nous Research).
- Doc Hermes — <https://hermes-agent.nousresearch.com/docs/>.
- Doc OpenClaw plugin SDK — <https://github.com/openclaw/openclaw/tree/main/docs/plugins>.
- Standard skills — <https://agentskills.io>.

---

> **Statut :** 🟢 fiche T1+T2 produite. Prochaine étape W5 : `RE-GOOSE-CODEX-OPENCODE.md` (T3+T4+T5)
> puis consolidation `FEATURE-MATRIX.md` et `OMEGA-BLUEPRINT.md` (`docs/omega/workstreams/W5-reverse-engineering.md:55-63`).
> **Rappel :** aucun code d'Omega n'est écrit tant que le port, le contrat et le test d'acceptation ne
> sont pas figés (RULES §4.4).
