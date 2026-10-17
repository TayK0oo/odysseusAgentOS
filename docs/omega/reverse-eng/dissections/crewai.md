# Dissection — CrewAI (`crewAIInc/crewAI`)

> Clone : `/home/tayk0oo/omega-re/crewai`, commit `40bb2d3` (2026-10-16), version SCM.
> Preuve REA : `docs/omega/research/rea/evidence/intake-crewai.summary.json` (59 775 ★, analyse « derived », 26 nœuds / 28 arêtes, 5 limites) — la preuve confirme l'identité, la licence et le clone ; son graphe est trop pauvre pour l'inventaire fonctionnel, qui est 100 % issu de la lecture du clone.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | CrewAI Inc. |
| Licence | MIT (`LICENSE` : « Copyright (c) 2025 crewAI, Inc. » — vérifiée dans le clone) |
| Langage | Python (monorepo uv, paquets `lib/crewai`, `lib/crewai-core`, `lib/crewai-tools`, `lib/crewai-files`, `lib/cli`, `lib/devtools`) |
| Repo | https://github.com/crewAIInc/crewAI |
| Architecture | Framework d'orchestration multi-agents : **Crew** (Agents + Tasks, process séquentiel/hiérarchique), moteur d'agent lui-même implémenté comme **Flow** (automate ReAct étatisé), **Flows** (graphe `@start/@listen/@router` avec état persisté), mémoire unifiée analysée par LLM (LanceDB/Qdrant, scopes + poids composites), knowledge vectorielle ChromaDB, bus d'événements typé + hooks d'interception + tracing OpenTelemetry, couche LLM LiteLLM, intégrations MCP et A2A. |

## 2. Entrées

- **API Python** : `Crew(...).kickoff()` (`crew.py:995`), `Flow(...).kickoff()` (`flow/flow.py:33`), `LiteAgent` (`lite_agent.py:193`).
- **CLI** : `lib/cli/src/crewai_cli` (`create_crew.py`, `create_flow.py`, `crew_run_tui.py`, `deploy/`, `evaluate_crew.py`, `checkpoint_cli.py`).
- **Points d'extension** : hooks `InterceptionPoint` (`hooks/dispatch.py:40-61`), bus d'événements `@on(TypeEvent)` (`events/event_bus.py:245`), skills `SKILL.md` (`skills/loader.py:48`), serveurs MCP (`mcp/config.py:12-117`), serveur A2A (`a2a/wrapper.py:97`).
- **Config** : `config/agents.yaml` + `config/tasks.yaml` via `@CrewBase` (`project/crew_base.py:154-156`), `MemoryConfig` (`memory/types.py:135`), `flow_config.py`, `security_config.py`.

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Agent = role/goal/backstory + délégation | agents | Identité d'agent en triplet injecté au prompt + `allow_delegation`, retries, timeout | `agent/core.py:213-331`, `agents/agent_builder/base_agent.py:186-278` | agent-store → `AgentStore` | planned | S |
| AgentExecutor : ReAct étatisé = Flow | agents | Boucle pense/agit/répond routée par états ; `max_iter=25` → force la final answer | `agents/agent_executor.py:187-262`, `:1600-1610`, `:2411-2418` | orchestrator → `AgentRunner` | planned | M |
| Délégation par outils (delegate/ask) | agents | Le manager délègue via `DelegateWorkTool` / `AskQuestionTool` ciblant un pair par son rôle | `tools/agent_tools/agent_tools.py:16-33`, `delegate_work_tool.py:16` | subagent-runner → `SubagentRunner` | planned | M |
| Crew séquentiel vs hiérarchique (manager) | exécution | Deux process ; manager auto-instancié (`manager_llm`/`manager_agent`) attribue et agrège | `crew.py:1522-1571`, `:725-741`, `process.py:4-10` | orchestrator → `Orchestration` | planned | L |
| Tasks : contexte, parallèle, sorties typées | exécution | `context=[Task]`, `async_execution`, `output_json/pydantic`, `output_file`, `human_input`, callback | `task.py:120-233`, `crews/utils.py:140-175` | orchestrator → `Orchestration` | planned | M |
| ConditionalTask (gating) | exécution | Fonction `condition(output_précédent)` décide du skip | `tasks/conditional_task.py:14-39` | orchestrator → gating | planned | S |
| Planning préalable (`CrewPlanner`) | exécution | Plan pas-à-pas par tache injecté en contexte ; `reasoning` déprécié → `planning_config` | `utilities/planning_handler.py:37-58`, `crew.py:347-358` | planner → `Planner` | planned | M |
| train / test / replay / kickoff_for_each | exécution | Évaluation, métriques, rejeu depuis une tache stockée, batch | `crew.py:943`, `:2101-2186`, `:2299`, `:1099` | event-store (replay) → `TraceReplayer` | planned | M |
| Guardrails de sortie + retries | sécurité | Validation fonction/LLM/hallucination avant la tache suivante, plafond de retries | `task.py:252-393`, `utilities/guardrail.py:60-123`, `tasks/llm_guardrail.py:39-49` | output-guardrail → `OutputGuardrail` | planned | M |
| Mémoire unifiée analysée par LLM | mémoire | Save LLM-analysé (scope/catégories/importance), score composite, consolidation, LanceDB/Qdrant | `memory/unified_memory.py:76-254`, `memory_scope.py:39-229` | memory-store → `MemoryStore` | planned | L |
| Recall adaptatif + analyse | mémoire | Profondeur de recherche variable (`RecallFlow`) + classement d'importance | `memory/recall_flow.py:3-58`, `memory/analyze.py:37-293` | memory-store → `MemoryStore` | planned | M |
| Knowledge sources + ChromaDB | contexte | Sources typees (txt/csv/json/pdf/excel/string/docling) chunkées, indexées, interrogeables | `knowledge/knowledge.py:88-116`, `storage/knowledge_storage.py:22-47` | context-engine → `ContextEngine` | planned | M |
| BaseTool + `@tool` + cache d'appel | outils | Schéma auto-de la signature, `ToolUsage` déduplique via cache `tool\|input` + `cache_function` | `tools/base_tool.py:103-701`, `tools/tool_usage.py:84-390` | tool-surface → `ToolSurface` | planned | M |
| Politique d'échec d'outil | outils | `ToolFailure` typé + `retryable` + émission d'événement + abort propre | `tools/tool_failure.py:60-176` | exec-policy → `ExecPolicy` | planned | S |
| Intégrations MCP + filtres | outils | Serveurs MCP stdio/HTTP/SSE, cache de liste, filtres contextuels | `mcp/config.py:12-117`, `mcp/filters.py:17`, `tools/mcp_native_tool.py:13` | mcp-bridge → `McpBridge` | planned | M |
| Flows : DAG + persistance + feedback humain | automatisation | `@start/@listen/@router`, état SQLite persisté, pause/reprise `HumanFeedback` | `flow/flow.py:33`, `flow/dsl/*`, `flow/persistence/base.py:48-97`, `flow/human_feedback.py:150-194` | workflow-engine → `FlowRuntime` | planned | L |
| Bus d'événements typé + listeners | observabilité | Chaque étape émet un événement typé, consommé par `@on` ; moteur séparé du rendu | `events/event_bus.py:245-572`, `events/types/*` | event-store (+ TelemetrySink) | planned | M |
| Hooks d'interception + abort | extension | Points pre/post model/tool/step, `HookAborted`, portée globale ou scoping contextvar | `hooks/dispatch.py:40-160` | registry → `Registre` (register_hook) | planned | M |
| Tracing OTEL + éphémère local | observabilité | Spans parent/enfant synchronisées, uploads non consentis gardés locaux | `telemetry/telemetry.py:30-33`, `telemetry/tracing/context.py:19-67`, `events/listeners/tracing/trace_listener.py:930-944` | telemetry-sink → `omega-otel` | planned | M |
| Couche LLM unifiée LiteLLM | routage | Lazy-load LiteLLM, streaming avec tool-calls, RPM limité | `llm.py:86-132`, `:258`, `:768-949`, `utilities/rpm_controller.py:12-44` | llm-provider → `Provider` | planned | M |
| Skills `SKILL.md` | extension | Frontmatter YAML, scan, registry + cache, niveaux d'activation (résumé vs complet) | `skills/parser.py:30-39`, `skills/loader.py:48-194`, `registry.py:134-154` | skill-store → `SkillStore` → `SkillIndex` | **adopted** | S |
| Crews YAML (`@CrewBase`) | configuration | `agents.yaml`/`tasks.yaml` séparés du code, `load_yaml` injectable | `project/crew_base.py:154-156`, `:343-392` | config-source → `ConfigSource` | planned | S |
| Protocole A2A (agent card HTTP) | exécution | Exposition d'un agent/crew comme serveur A2A avec carte et tasks asynchrones | `a2a/wrapper.py:97-262` | service → `omega-server` | planned | M |
| LiteAgent (run simple) | agents | Agent autonome sans Crew, garde de fenêtre de contexte, événements dédiés | `lite_agent.py:193-501`, `:805` | agent-store → `AgentRunner` | planned | M |
| Adaptateurs LangGraph / OpenAI Agents | extension | **Écarté** : interop avec des framework Python externes, hors périmètre Rust | `agents/agent_adapters/*` | — | rejected | L |
| Fingerprint déterministe d'agent | sécurité | UUID stable dérivé du seed (identity/provenance pour les traces) | `security/fingerprint.py:17-91` | provenance → `omega-otel::Provenance` | planned | S |

## 4. Traduction en plugins

- `crw-agent-identity` / `crw-lite-agent` : agent-store → `AgentStore`/`AgentRunner` → `plugins/agent-identity`, `plugins/lite-agent` (plugin).
- `crw-agent-executor-flow` / `crw-crew-process` / `crw-task-graph` / `crw-conditional-task` : orchestrator → `Orchestration` → `plugins/crew-process` (le cœur de CrewAI : nos `Registre` slots + `EventStore` rejouable couvrent la charpente, il manque le runtime d'orchestration lui-même).
- `crw-delegation-tools` : subagent-runner → `SubagentRunner` → `plugins/delegation-tools` (pair avec `crw-agent-executor-flow`).
- `crw-crew-planning` : planner → `Planner` → `plugins/crew-planner` (plan injecté en contexte, coûte un appel LLM).
- `crw-train-replay` : event-store (replay) → `EventStore` → `omega-otel::TraceReplayer` (notre rejeu existe pour les traces ; étendre à la re-exécution incrémentale d'une tâche).
- `crw-guardrails` : output-guardrail → `OutputGuardrail` → `plugins/guardrails` (chaîne valider→corriger→rejeter, plafond de retries — équivalent structurel de notre `ExecPolicy` appliqué à la sortie).
- `crw-memory-unified` / `crw-memory-recall` : memory-store → `MemoryStore` → `plugins/memory-unified` (nouveau port ; s'appuie sur l'index vectoriel du `ToolSurfaceIndex`/`SkillIndex` existant).
- `crw-knowledge` : context-engine → `ContextEngine` → `plugins/knowledge-sources` (sources typées alimentant notre budget de contexte).
- `crw-tool-contract` / `crw-tool-failure` : tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` + exec-policy → `omega-sandbox::RuleBasedExecPolicy`.
- `crw-mcp` : mcp-bridge → `McpBridge` → `plugins/mcp-client` (transports stdio/http/sse + filtres).
- `crw-flows` : workflow-engine → `FlowRuntime` → `plugins/flow-runtime` (DAG étatisé + persistance + human-in-the-loop, alimenté par `EventStore`).
- `crw-event-bus` / `crw-hooks` : event-store → `omega-app::FileEventStore` + `omega-otel::TraceReplayer` ; registry → `Registre` (`register_hook`).
- `crw-otel-tracing` / `crw-security-fingerprint` : telemetry-sink → `omega-otel::{Stdout,File}Sink` + provenance → `omega-otel::Provenance`.
- `crw-llm-layer` : llm-provider → `Provider` → `omega-providers::OllamaProvider` (on garde notre registre multi-fournisseurs ; le lazy-load LiteLLM est leur mécanisme, pas le nôtre).
- `crw-skills` : skill-store → `SkillStore` → `omega-context::SkillIndex` — **adopté**, c'est notre capacité existante.
- `crw-yaml-config` : config-source → `ConfigSource` → `omega-config` (couches + precedence).
- `crw-a2a` : service → `omega-server` (endpoint agent + agent card).

## 5. Écarté

- **Adaptateurs LangGraph / OpenAI Agents** (`agents/agent_adapters/`) : ils enrobent des runtime Python tiers ; sans équivalent dans un cœur Rust, seul le *pattern* adaptateur (BaseAgentAdapter / BaseToolAdapter / BaseConverterAdapter) est retenu comme étude.
- **Lazy-load LiteLLM et ses side-effects dotenv** (`llm.py:86-132`) : contournement d'un problème Python ; notre `Provider` est un trait Rust, pas concerné.
- **CrewAI Plus / telemetry d'adoption** (`plus_api.py`, consentement de trace upload) : service SaaS de l'éditeur, hors périmètre « everything is a plugin » self-hosted ; seul le mécanisme éphémère local (`telemetry/tracing/ephemeral.py:1`) est retenu.

## 6. Sources

- Clone : `/home/tayk0oo/omega-re/crewai` (commit `40bb2d3`, 2026-10-16, lecture seule).
- Preuve REA : `docs/omega/research/rea/evidence/intake-crewai.summary.json` (license MIT confirmée, clone ok, 5 limites non détaillées — graphe de 26 nœuds insuffisant pour l'inventaire, corroboration limitée à l'identité/licence).
- https://github.com/crewAIInc/crewAI (upstream, LICENSE MIT).
- Tous les `file:line` cités renvoient au clone ci-dessus.
