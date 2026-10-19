# Dissection — LangGraph (`langchain-ai/langgraph`)

> Reverse-engineering exhaustif, fonctionnalité par fonctionnalité.
> Données machine : `omega/crates/omega-harness/data/apps/langgraph.json` (schema_version=1).
> Preuve REA : `docs/omega/research/rea/evidence/intake-langgraph.summary.json`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | LangChain Inc. |
| Licence | **MIT** (vérifiée dans le clone : `LICENSE` — « Copyright (c) 2024 LangChain, Inc. ») |
| Langage | Python (également `sdk-js` pour le SDK TS) |
| Repo | https://github.com/langchain-ai/langgraph |
| Version observée | `langgraph` 1.2.16 (`libs/langgraph/pyproject.toml`) |
| Étoiles (REA) | 43 380 |

**Architecture.** LangGraph est un moteur d'exécution d'états de type Pregel : les
nœuds deviennent des *tâches*, l'état est découpé en *canaux versionnés* munis d'un
réducer ; chaque pas superpose une phase de lecture des canaux, de planification de
tâches (dont les tâches `Send` poussées dynamiquement), d'exécution parallèle et
d'écriture — avec un checkpoint persisté par pas (`libs/langgraph/langgraph/pregel/`).
Le noyau (`libs/langgraph`) est découpé de façon modulaire : persistance et mémoire
dans `libs/checkpoint` (contrat `BaseCheckpointSaver` + backends memory/sqlite/postgres
+ `BaseStore`), exécution préfabriquée dans `libs/prebuilt` (`ToolNode`, agent ReAct,
schémas human-in-the-loop), transport dans `libs/sdk-py`/`libs/sdk-js`, orchestration
de déploiement dans `libs/cli` (docker compose). Le tout est « runnable » via
l'interface `Runnable` de langchain-core.

**Lecture critique.** Le dépôt est la *bibliothèque* ; le serveur d'exécution complet
(`langgraph_api`, Studio, crons, auth multi-tenant) vit côté LangGraph Platform/LangSmith
et n'est **pas** dans ce dépôt seul (voir les limites de la preuve REA §6).

## 2. Entrées

| Type | Entrée |
|---|---|
| Binaire / CLI | `langgraph` (`libs/cli/langgraph_cli/cli.py`) — commandes `up`, `dev`, `build`, `validate`, `new`, `prepare` |
| API Python | `StateGraph` → `compile()` → `invoke/ainvoke/stream/astream`, `get_state`, `update_state`, `get_state_history` (`libs/langgraph/langgraph/pregel/main.py`) |
| API fonctionnelle | `@entrypoint`, `@task` (`libs/langgraph/langgraph/func/__init__.py:110,262`) |
| Client réseau | `RemoteGraph` (`pregel/remote.py:118`), `langgraph_sdk.Client` (`libs/sdk-py`) |
| Points d'extension | `BaseCheckpointSaver` (checkpoint), `BaseStore` (store), `BaseCache` (cache), `StreamTransformer` (stream), `GraphCallbackHandler` (callbacks), `ToolNode`/`Runtime` (outils) |
| Config | `langgraph.json` (validateur `libs/cli/langgraph_cli/config.py`), env `LANGGRAPH_STRICT_MSGPACK`, `LANGSMITH_API_KEY`, `LANGGRAPH_CLOUD_LICENSE_KEY` |

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Graphe d'états déclaratif | agents | `StateGraph` nœuds/arêtes/conditionnelles → `CompiledStateGraph` | `graph/state.py:131,667,928,982,1032,1177,1404` | `WorkflowEngine` | planned | L |
| Canaux + reducers | execution | Fusion déterministe des mises à jour concurrentes | `channels/base.py:70,90`, `channels/binop.py:66`, `channels/topic.py:23`, `graph/message.py:64` | `GraphChannel` | planned | M |
| Arêtes conditionnelles + `Send` | routage | Route dynamique + map-reduce parallèle | `graph/state.py:982`, `graph/_branch.py:83`, `types.py:738` | `WorkflowEngine (route/Send)` | planned | M |
| Checkpointing | memoire | État persisté par `(thread_id, ns, id)` + backends | `checkpoint/base/__init__.py:199…344`, `memory/__init__.py:34`, `sqlite/__init__.py:49`, `postgres/base.py:547` | `CheckpointStore` | planned | L |
| Exécution durable | execution | Modes `sync\|async\|exit`, reprise sans ré-exécution | `types.py:98`, `pregel/_loop.py:190,515,1163` | `EventStore` → `omega-app::{File,InMemory}EventStore` | **adopted** | M |
| HITL `interrupt()` | securite | Pause reprendable au cœur d'un nœud | `types.py:833,903`, `errors.py:103` | `HumanGate` | planned | L |
| Schémas d'approbation | securite | accept/ignore/edit/response | `prebuilt/interrupt.py:51,87` | `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | **adopted** | S |
| Streaming multi-modes | observabilite | values/updates/messages/custom/… + bus v2/v3 | `types.py:131`, `pregel/main.py:2709,3704`, `stream/_types.py:44`, `stream/_mux.py:26` | `StreamSink` | planned | M |
| Callbacks de vie de graphe | observabilite | `on_interrupt` / `on_resume` | `callbacks.py:87,99,219` | `TelemetrySink` → `omega-otel::{Stdout,File}Sink` | **adopted** | S |
| Sous-graphes | agents | Composition + namespace de checkpoint | `pregel/main.py:1144`, `pregel/_algo.py:614,623` | `SubagentRunner` | planned | M |
| Store mémoire longue | memoire | Namespaces, recherche, TTL, index vectoriel | `store/base/__init__.py:708,784,866,966,545,578`, `store/memory/__init__.py:136` | `MemoryStore` | planned | L |
| Valeurs gérées | contexte | `IsLastStep` / `RemainingSteps` injectés | `managed/base.py:18`, `managed/is_last_step.py:8` | `ContextEngine` | planned | S |
| Retry + timeout par nœud | execution | Backoff/jitter + `run_timeout`/`idle_timeout` | `types.py:427,461`, `pregel/_retry.py:573,660` | `RetryPolicy` | planned | M |
| Nœuds gestionnaires d'erreur | execution | Route d'erreur locale ou globale | `graph/state.py:99,331,867,1292` | `WorkflowEngine (on_error)` | planned | S |
| `ToolNode` | outils | Exécution d'outils + injection + erreurs | `prebuilt/tool_node.py:630,391,1697,1787,1863` | `ToolSurface` | planned | M |
| Fabrique d'agent ReAct | agents | `create_react_agent` (dépréciée → `create_agent`) | `prebuilt/chat_agent_executor.py:278,309,313` | `AgentLoop` | planned | S |
| API fonctionnelle | agents | `@entrypoint`/`@task`, `previous`, `.final` | `func/__init__.py:110,262,476` | `AgentLoop` | planned | M |
| Inspection d'état | observabilite | `get_state` / `get_state_history` / `update_state` | `pregel/main.py:1464,1514,2617` | `CheckpointStore (history/fork)` | planned | M |
| Rejeu read-only | observabilite | Replay du journal sans ré-exécution | `omega-otel/src/replay.rs:64,67` | `EventStore` → `omega-otel::TraceReplayer` | **adopted** | S |
| `RemoteGraph` | execution | Client d'un déploiement distant | `pregel/remote.py:118,322,724` | `RemoteGraph` | planned | M |
| CLI de déploiement | automatisation | up/dev/build/validate + docker compose | `cli/cli.py:276,419,763,871` | serveur Omega | planned | M |
| SDK de contrôle | execution | threads/runs/assistants/crons + auth | `sdk-py/.../schema.py:335,364,385`, `auth/types.py:374` | `omega-protocol` | planned | M |
| Sérialisation sûre | securite | Allow-list msgpack stricte + chiffrement | `serde/jsonplus.py:104,150`, `serde/_msgpack.py:12`, `serde/encrypted.py:8` | `ConfigSource` (allow-list) | planned | M |
| Messages UI | ui | `push_ui_message` / reducer de suppression | `graph/ui.py:61,133,165` | `UiPanel` | planned | S |

**Total : 24 fonctionnalités** — 4 `adopted`, 20 `planned`, 0 `rejected`.

## 4. Traduction en plugins

### Déjà chez nous (`adopted` — conformes à la table d'honnêteté du schéma)

| Fonctionnalité | Capacité | Port | Adaptateur existant | Preuve |
|---|---|---|---|---|
| Exécution durable / reprise | `event-store` | `EventStore` | `omega-app::{File,InMemory}EventStore` | `omega-app/src/lib.rs:125` (contrat `resume`), tests `crash_after_k_steps_resumes_without_reexecuting`, `resume_is_deterministic`, `resume_rejects_changed_plan` (`:788,864,911`) |
| Schémas d'approbation | `permission-policy` | `PermissionPolicy` | `omega-sandbox::ApprovalQueue` | `omega-sandbox/src/approvals.rs:56` (`request/approve/deny`), consommé par `LocalSandboxBackend` (`backend.rs:4,35`) |
| Rejeu read-only | `event-store` | `EventStore` | `omega-otel::TraceReplayer` | `omega-otel/src/replay.rs:64-105` — trie par `seq`, issue déterministe, détection de conflit ; exposé par `GET /api/trace/:run_id` |
| Callbacks de vie (flux) | `telemetry-sink` | `TelemetrySink` | `omega-otel::{Stdout,File}Sink` | `omega-otel/src/sinks.rs:16,41` |

### À porter (`planned`) — triplets capacité → port → adaptateur

- **`workflow-engine` → `WorkflowEngine (build/compile/run)` → `plugins/workflow-graph`** :
  le cœur (graphe + canaux + arêtes conditionnelles + gestionnaires d'erreur). Rust `L`.
- **`checkpoint-store` → `CheckpointStore (put/get_tuple/list)` → `plugins/checkpoint-store`** :
  journal indexé par `thread_id`, sur le contrat `EventStore` déjà existant. Rust `L`.
- **`hitl-gate` → `HumanGate (interrupt/resume)` → `plugins/hitl-gate`** : pause repriseable
  par thread, au-dessus de `omega-app::EventStore`. Rust `L`.
- **`memory-store` → `MemoryStore (recall/remember + namespaces)` → `omega-context::NamespaceStore`** :
  le port `MemoryStore` existe (`omega-ports/src/lib.rs:35`) mais sans adaptateur
  namespace/TTL/index — LangGraph donne le modèle de contrat. Rust `L`.
- **`stream-surface` → `StreamSink` → `omega-server`** : SSE du cockpit sur `POST /api/run`.
- **`retry-policy` → `RetryPolicy (attempt/backoff)` → `plugins/retry-policy`** : politiques
  par tache, distinctes du routeur de modèles. Rust `M`.
- **`subagent-runner` → `SubagentRunner (spawn/resume)` → `plugins/subagent-graph`** :
  sous-graphes isolés avec namespace (le port existe, `omega-ports/src/lib.rs:172`).
- **`tool-surface` → `ToolSurface` → `omega-context::ToolSurfaceIndex` + `ToolExecutor`** :
  l'index existe (`omega-context/src/tool_surface.rs:133`), il manque l'exécution en nœud.
- **`agent-loop` → `AgentLoop (plan/act)` → `plugins/agent-loop`** : boucle agent compilée.
- **`context-engine` → `ContextEngine` → `omega-context::DefaultContextEngine`** :
  métacompteurs de run (`IsLastStep`/`RemainingSteps`) injectés à l'assemblage.
- **`ui-panel` → `UiPanel` → `omega-ui`** : canal de messages poussés par un nœud.
- **Natif (sérialisation sûre)** : porter l'allow-list stricte sur `FileEventStore`
  (interdire tout type hors liste à la lecture du journal) — `omega-types`.
- **Service (transport)** : `RemoteGraph`/SDK/CLI → `omega-protocol` + `omega-server` +
  `omega/scripts/run-local.sh`, déjà alignés sur l'origine unique.

## 5. Écarté (et réserves)

Rien n'a été marqué `rejected` dans le JSON : chaque fonctionnalité retenue a une cible
crédible. En revanche, trois **réserves d'architecture** doivent accompagner le portage :

1. **Dépendance à langchain-core** — `Pregel` hérite de `Runnable`, les nœuds passent par
   `coerce_to_runnable`, les modèles par `BaseChatModel`. Repris en Rust, ce sont les
   contrats `omega-ports::{Provider,ToolExecutor}` qui font foi ; la notion `Runnable`
   n'est pas réimplémentée (coût évité : L).
2. **Le vrai serveur n'est pas dans ce dépôt** — `langgraph dev` importe `langgraph_api`
   (non présent : `libs/cli/langgraph_cli/cli.py:800` lève `ImportError` sans le paquet
   `[inmem]`), et les crons/Studio/auth multi-tenant sont côté LangSmith. La partie
   « déploiement » documentée ici est donc la *CLI + SDK*, pas la plateforme.
3. **API en transition** — `create_react_agent` est dépréciée au profit de
   `langchain.agents.create_agent` (middleware), et les schémas HITL de `prebuilt`
   migrent vers `langchain.agents.interrupt` (`prebuilt/interrupt.py:9,31`). Ne pas
   figer les signatures dépréciées dans nos ports.

## 6. Sources

**Clone (lecture seule) :** `/home/tayk0oo/omega-re/langgraph` (branche `main`, `langgraph` 1.2.16).

- Noyau : `libs/langgraph/langgraph/graph/state.py`, `pregel/main.py`, `pregel/_loop.py`,
  `pregel/_algo.py`, `pregel/_retry.py`, `pregel/remote.py`, `types.py`, `errors.py`,
  `callbacks.py`, `runtime.py`, `func/__init__.py`, `managed/`, `channels/`, `stream/`, `graph/ui.py`.
- Persistance : `libs/checkpoint/langgraph/checkpoint/{base,memory,serde}/`,
  `libs/checkpoint/langgraph/store/{base,memory}/`, `libs/checkpoint-sqlite/`,
  `libs/checkpoint-postgres/`, `libs/checkpoint-conformance/`.
- Exécution préfabriquée : `libs/prebuilt/langgraph/prebuilt/{tool_node,chat_agent_executor,interrupt}.py`.
- Transport/déploiement : `libs/sdk-py/langgraph_sdk/`, `libs/cli/langgraph_cli/`.
- Licence : `LICENSE` (MIT, LangChain Inc. 2024) ; version : `libs/langgraph/pyproject.toml`.

**Docs officielles** (index `docs/llms.txt` du clone) :
https://docs.langchain.com/oss/python/langgraph/{graph-api,durable-execution,interrupts,checkpointers,streaming,memory,stores,use-subgraphs,functional-api,local-server}.md

**Côté Omega (cibles) :** `omega/crates/omega-app/src/{lib,store}.rs`,
`omega/crates/omega-otel/src/{sinks,replay,provenance}.rs`,
`omega/crates/omega-sandbox/src/{approvals,backend}.rs`,
`omega/crates/omega-ports/src/lib.rs`, `omega/crates/omega-context/src/{tool_surface,skills}.rs`.

**Preuve REA :** `docs/omega/research/rea/evidence/intake-langgraph.summary.json`
(provider `rea-javascript-application`, confiance `derived`, 89 nœuds / 109 arêtes détectés,
`limitations_count: 6`). **Limite :** cette preuve est un comptage structurel automatisé
d'un dépôt Python analysé par un extracteur JavaScript — elle corrobore l'existence d'un
graphe (nœuds/arêtes cohérents avec `StateGraph`) mais **ne valide aucune** des
fonctionnalités listées ici ; toutes les refs `file:ligne` proviennent du clone direct.
