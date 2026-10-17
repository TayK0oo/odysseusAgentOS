# Dissection — AutoGPT (`Significant-Gravitas/AutoGPT`)

> Clone (LECTURE SEULE) : `/home/tayk0oo/omega-re/autogpt`, commit `06d623e` (`06d623e5e04cc96621130bb4bd118465c9e635d8`, 2026-10-16).
> Preuve machine REA : `docs/omega/research/rea/evidence/intake-autogpt.summary.json`
> (`evidence_id` `ev_7f447ff1df9072a3b2bcb2c62dfc12945475275fd123c19d2b177c08c562615b`,
> provider `rea-javascript-application`, confidence `derived`, 14 136 nœuds / 15 594 arêtes,
> `limitations_count` 6 294, digest sha256 `f49048b5…`). **Limite** : REA classe le dépôt en
> `nature: "other"` et `operation: analyze_javascript_application` alors que le code est du
> **Python** ; ses graphes portent donc sur le frontend et les fichiers de configuration, pas
> sur la sémantique Python. Toutes les preuves de cette dissection viennent des `file:line`
> du clone, pas de REA.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Significant Gravitas |
| Étoiles | 187 614 (REA, `intake-autogpt.summary.json`) |
| Licence | **`MIT AND PolyForm-Shield-1.0.0`** — GitHub renvoie `NOASSERTION` ; `LICENSE:1-4` tranche : *tout* `autogpt_platform/` est sous **PolyForm Shield 1.0.0** (`LICENSE:27`, non-compete, usage interne autorisé, revente d'un service concurrent interdite), *tout le reste* (dont `classic/`) sous **MIT** (`LICENSE:196-220`, Copyright (c) 2023 Toran Bruce Richards). Confirmé par le tableau du `README.md:188-193`. |
| Langage | Python (backend/plateforme) + TypeScript (frontend Next.js) + SQL/Prisma |
| Repo | https://github.com/Significant-Gravitas/AutoGPT |
| Architecture | Monorepo à deux étages. `classic/` (MIT) garde l'agent autonome **Forge** : boucle d'agent dont les composants sont des plugins à protocoles (`CommandProvider`/`MessageProvider`/`DirectiveProvider`/`AfterParse`) et les stratégies de raisonnement interchangeables (7). `autogpt_platform/` (Polyform Shield) est une plateforme **FastAPI multi-processus** qui représente un agent comme un **graphe orienté de blocs** (`Node`s + `Link`s) exécuté nœud par nœud via **RabbitMQ**, doublée d'un **copilote LLM** (registre d'outils/capacités BM25, gate d'approbation par effet, sandbox E2B) et d'un **frontend Next.js** ; l'état vit dans Postgres/Prisma + Redis. 413 fichiers de blocs (hors tests), 93 outils de copilote, 49 familles d'intégration. |

Le mot d'ordre maison est proche du nôtre : *un bloc = un plugin* (`Block` + UUID stable,
découverte par scan de paquet) et *un composant d'agent = un plugin à protocoles* (Forge).

## 2. Entrées

- **Binaires / points d'entrée** (un seul binaire qui lance 11 processus, `app.py:34-67`) :
  `AgentServer` (REST FastAPI `/api`, tags `v1`/`v2`), `WebsocketServer` (`api/ws_api.py:341`),
  `ExecutionManager` (consommateur RabbitMQ, `executor/manager.py:1588`), `Scheduler`
  (cron APScheduler, `executor/scheduler.py:26`), `DatabaseManager`, `BatchExecutor`
  (`executor/batch_executor.py`), `NotificationManager`, `PlatformLinkingManager`,
  `CoPilotChatBridge`, `CoPilotExecutor`.
- **CLI** : `backend/cli/main.py` (+ `chat.py`, `generate_openapi_json.py`, `store.py`).
- **Agent autonome historique** : `classic/original_autogpt/autogpt/__main__.py`,
  `app/agent_protocol_server.py` (API « agent protocol ») ; Forge dans `classic/forge/`.
- **UI** : `autogpt_platform/frontend` (Next.js, `src/app/(platform)` / `(no-navbar)` /
  `(public)`, client OpenAPI généré par `orval.config.ts`).
- **Points d'extension** :
  1. **Blocs** — déposer un fichier sous `backend/blocks/` suffit ; `load_all_blocks()`
     (`blocks/__init__.py:13`) scanne, importe, valide et enregistre. SDK dédié
     (`backend/sdk/__init__.py`, « `from backend.sdk import *` ») + décorateurs d'auto-registration.
  2. **Outils de copilote** — `TOOL_REGISTRY` (`copilot/tools/__init__.py:111`) + `TOOL_GROUPS`
     (`:253`) ; divulgation progressive via `find_capability`/`describe_capability`/`run_capability`.
  3. **Serveurs MCP** — catalogue (`integrations/mcp_catalog.py`) + client dual-era
     (`blocks/mcp/client.py`), outils fondus dans l'index de capacités.
  4. **Skills** — dossiers `workspace://skills/{slug}/SKILL.md` écrits par le modèle lui-même.
  5. **Experts** — roster YAML publié avec le catalogue de skills (`api/features/experts/roster.py`).
  6. **Forge (classic)** — composants à protocoles (`forge/agent/protocols.py:12`) et stratégies
     de prompting (`agents/prompt_strategies/base.py:47`).
- **Config** : `backend/util/settings.py` (`Config`/`Settings` + secrets), `copilot/config.py`
  (`ChatConfig`, `TransportProfile`), flags par utilisateur (`util/feature_flag`), catalogue de
  modèles versionné (`data/llm_registry`), `autogpt_platform/docker-compose*.yml` (self-host).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Registre de blocs auto-découvert (bloc = plugin) | extension | Scan du paquet, UUID obligatoire, collisions refusées, cache 1 h | `backend/blocks/__init__.py:13-90`, `blocks/_base.py:585` | registry → `Registre` → `omega-registry::Registry` | planned | M |
| Contrat de bloc schéma-first | outils | Input/output pydantic, JSON Schema dérivé, champs credentials validés à l'import | `blocks/_base.py:217-360`, `:585-830` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | **adopted** | M |
| Effet + coût déclarés par bloc | sécurité | `BlockEffect`, `BlockType`, `BlockCost` (6 types, table de tokens) | `blocks/_base.py:68-95`, `:122-205` | exec-policy → `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy` | planned | M |
| **Index de capacités BM25 + divulgation progressive** | outils | BM25 sur outils+blocs+catalogue MCP, noyau « eager » restreint, fusion d'implémentations | `copilot/capabilities/index.py:1-135`, `registry.py:35-110`, `sources/static_tools.py:25-58` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | **adopted** | L |
| Registre d'outils du copilote (groupes, disponibilité, activité) | outils | `TOOL_REGISTRY`/`TOOL_GROUPS`, `is_available`, `activity_event`, `gate_subject` | `copilot/tools/__init__.py:111-311`, `tools/base.py:319-430` | registry → `Registre` → `omega-registry::Registry` | planned | M |
| Permissions outils+blocs, restrictives par héritage | sécurité | Listes blanches/noires, UUID partiels, `merged_with_parent` | `copilot/permissions.py:1-80`, `:397-500` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | **adopted** | M |
| **Gate d'approbation par effet** | sécurité | `Effect`/`PARKABLE`/`Verdict`, pire effet du graphe, cartes d'approbation | `copilot/gate/policy.py:1-120`, `gate/effects.py:1-60` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | planned | L |
| Backend de sandbox interchangeable (E2B / bubblewrap) | sécurité | Sandbox E2B persistante par propriétaire, repli bwrap sans réseau | `copilot/tools/e2b_sandbox.py:1-45`, `tools/bash_exec.py:240-295` | sandbox-backend → `SandboxBackend` → `omega-sandbox::LocalSandboxBackend` | **adopted** | L |
| Human-in-the-loop : bloc d'attente + file de revues | sécurité | `HumanInTheLoopBlock`, `/pending`, `/execution/…`, `/session/…`, `POST /action` | `blocks/human_in_the_loop.py:22-160`, `api/features/graph_executions/review/routes.py:76-200` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | **adopted** | M |
| Moteur d'exécution par nœud sur DAG | exécution | Propagation des sorties, enfilement des nœuds complets, timeout 30 min | `executor/manager.py:223-560`, `:724-1010`, `data/graph.py:99-160` | service → `omega-app` (orchestrateur de graphe) | planned | L |
| Exécution distribuée (RabbitMQ + verrous Redis) | exécution | Files/exchanges dédiés, ack/reject + requeue, file d'annulation, `ClusterLock` | `executor/manager.py:1588-1870`, `data/rabbitmq.py`, `executor/cluster_lock.py` | service → `omega-app` (file + verrou) | planned | M |
| Runtime multi-processus (11 services) | exécution | `run_processes` démarre N−1 en fond, arrêt en ordre inverse | `app.py:14-67`, `util/process.py` | service → `omega-server` | planned | M |
| Planification cron/date | automatisation | APScheduler (`CronTrigger`/`DateTrigger`), fuseaux, événements manqués, index de jobs | `executor/scheduler.py:1-45`, `executor/schedule_index.py` | service → `omega-app` (cron) | planned | M |
| Déclencheurs webhook par bloc | automatisation | `BlockManualWebhookConfig`/`BlockWebhookConfig`, manager générique | `blocks/_base.py:528-585`, `blocks/generic_webhook/triggers.py:23-80` | service → `omega-server` (ingress) + `omega-app` | planned | M |
| Mémoire temporelle en graphe (Graphiti/FalkorDB) | mémoire | Faits/procédures/règles, partition par utilisateur/expert, forget bi-temporel | `copilot/graphiti/client.py:83-120`, `memory_model.py`, `falkordb_driver.py:110`, `api/features/memory/routes.py` | context-engine → `ContextEngine` → `omega-context::DefaultContextEngine` | planned | L |
| Consolidation nocturne (« dream pass ») + ratification | mémoire | Lot Anthropic, reprise, ratification avant application, marqueurs cron | `copilot/dream/nightly_batch.py:54-120`, `ratification.py`, `executor/batch_executor.py` | service → `omega-app` (job nocturne) | planned | L |
| Skills auto-distillées (SKILL.md + frontmatter) | contexte | Dossier + `SKILL.md`, corps à la demande, bloc `<available_skills>`, import/export zip scanné | `copilot/tools/skills.py:1-40`, `:301-370`, `api/features/skills/routes.py:1-50` | skill-store → `SkillStore` → `omega-context::SkillIndex` | **adopted** | M |
| Client MCP bi-ère (Streamable HTTP + SSE + OAuth) | outils | Révision stateless `2026-07-28` + repli `initialize`, cache d'ère, catalogues validés | `blocks/mcp/client.py:1-35`, `oauth.py`, `integrations/mcp_catalog.py:1-40` | tool-surface → `ToolSurface` → `omega-mcp` | planned | M |
| Marketplace d'agents et de skills | extension | Recherche hybride BM25+vecteurs+popularité, soumissions, versions, scan antivirus | `api/features/store/routes.py:1-40`, `store/hybrid_search.py:1-40`, `store/skill_catalog.py` | registry → `Registre` → `omega-registry::Registry` | planned | L |
| Sous-agents (agent dans l'agent, garde de récursion, résumés) | agents | `AgentExecutorBlock`, `SubAgentRecursionError`, `run_sub_session`/`delegate_to_expert` | `blocks/agent.py:29-160`, `blocks/autopilot.py:57-120`, `copilot/tools/run_agent.py` | subagent-runner → `SubagentRunner` → `omega-app::SubagentRunner` | planned | L |
| Experts embauchables (roster, routines, credentials, computer) | agents | Roster validé, routines planifiées, grants de credentials, sandbox persistante | `api/features/experts/routes.py:167-260`, `roster.py:1-30`, `spend_approval.py`, `tools/hire_expert.py:40` | subagent-runner → `SubagentRunner` → `omega-app::ExpertRoster` | planned | L |
| Transports multi-fournisseurs (4 transports) | routage | Table figée `TransportProfile`, repli de clés, compat. de slugs, Ollama local | `copilot/config.py:35-70`, `:50-200`, `transport_routing.py` | llm-provider → `Provider` → `omega-providers::OllamaProvider` | **adopted** | M |
| Routage mode×tier avec chaîne de repli + kill-switch | routage | LaunchDarkly → catalogue → env, refus des slugs désactivés, HIDDEN servi | `copilot/model_router.py:1-60`, `provider_tiers.py:1-40`, `engine.py:1-40` | router-policy → `ScoringPolicy/Selector/FallbackPolicy` → `omega-router::CompositeScoringPolicy + TierFallbackPolicy` | planned | M |
| Traces et métriques multi-canaux | observabilité | Transaction Sentry par nœud, trace partagée, Prometheus cardinalité contrôlée, Langfuse | `executor/node_tracing.py:1-30`, `monitoring/instrumentation.py:1-40`, `copilot/service.py:151-160` | telemetry-sink → `TelemetrySink` → `omega-otel::{File,Stdout}Sink` + `Provenance` | **adopted** | M |
| Coût déclaré, métrage post-vol, facturation crédits | observabilité | `BlockCost` (6 types), `charge_node_usage`, `failure_reason` collant, ledger | `blocks/_base.py:122-205`, `executor/manager.py:1151-1200`, `executor/cost_tracking.py`, `billing.py` | service → `omega-otel::Provenance` + ledger | planned | L |
| Frontend Next.js (builder, copilote, store, approbations) | ui | Routes `(platform)`/`(no-navbar)`, client OpenAPI, fixtures de cartes générées | `frontend/orval.config.ts`, `frontend/src/app`, `gate/card_fixture_test.py:20-35` | service → `omega-server` cockpit | planned | L |
| Diffusion temps réel (WebSocket + registre de flux) | observabilité | Abonnements par canal, heartbeat, reprise après déconnexion, file de tours | `api/ws_api.py:99-260`, `copilot/stream_registry.py`, `turn_queue.py` | service → `omega-server` (SSE/WS) | planned | M |
| Forge : composants d'agent comme plugins à protocoles | extension | `CommandProvider`/`MessageProvider`/`DirectiveProvider`/`AfterParse` + pipeline d'endpoints | `classic/forge/forge/agent/protocols.py:12-40`, `agent/base.py`, `components/` | registry → `Registre` → `omega-registry::Registry` | planned | L |
| Stratégies de raisonnement interchangeables (7) | agents | `one_shot`, `rewoo`, `plan_execute`, `reflexion`, `tree_of_thoughts`, `lats`, `multi_agent_debate` | `classic/…/agents/prompt_strategies/base.py:44-70`, `lats.py`, `rewoo.py` | agent-store → `AgentStore` → `plugins/reasoning-strategies` | planned | L |
| Bloc orchestrateur (LLM + outils = blocs) | outils | Outils choisis par le modèle, ciblage d'un nœud « sink » du graphe hôte | `blocks/orchestrator.py:58-100`, `:426-600`, `:1100-1210` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | planned | L |
| Configuration par couches (settings + env + flags) | configuration | `Settings`/`ChatConfig` pydantic, flags par utilisateur, catalogue versionné | `util/settings.py:1-60`, `copilot/config.py:191-230`, `util/feature_flag` | config-source → `ConfigSource` → `omega-config` | planned | S |

Comptage (vérifié sur le JSON) : **31 fonctionnalités** — adoption honnête **8**
(`adopted`), 23 `planned`, 0 `rejected` ; coûts : 1 `S`, 16 `M`, 14 `L` ; intégrations :
22 `plugin`, 9 `service` ; domaines : outils 5, sécurité 5, extension 3, exécution 3,
agents 3, observabilité 3, automatisation 2, mémoire 2, routage 2, contexte 1, ui 1,
configuration 1.

## 4. Traduction en plugins

Triplet `capacité → port → adaptateur`, avec le mode d'intégration :

| Fonctionnalité (JSON) | Capacité → Port | Adaptateur | Mode |
|---|---|---|---|
| `ag-block-contract` | tool-surface → `ToolSurface` | `omega-context::ToolSurfaceIndex` | plugin, **adopted** |
| `ag-capability-index-bm25` | tool-surface → `ToolSurface` | `omega-context::ToolSurfaceIndex` | plugin, **adopted** |
| `ag-orchestrator-block` | tool-surface → `ToolSurface` | `omega-context::ToolSurfaceIndex` + boucle d'outils du noyau | plugin, planned |
| `ag-mcp-client` | tool-surface → `ToolSurface` | `omega-mcp` (crate existant à étendre) | plugin, planned |
| `ag-permissions-tools-blocks` | permission-policy → `PermissionPolicy` | `omega-sandbox::ApprovalQueue` | plugin, **adopted** |
| `ag-gate-effect-approval` | permission-policy → `PermissionPolicy` | `omega-sandbox::ApprovalQueue` | plugin, planned |
| `ag-hitl-review-queue` | permission-policy → `PermissionPolicy` | `omega-sandbox::ApprovalQueue` | plugin, **adopted** |
| `ag-block-effect-cost` (effet) | exec-policy → `ExecPolicy` | `omega-sandbox::RuleBasedExecPolicy` | plugin, planned |
| `ag-sandbox-exec-backend` | sandbox-backend → `SandboxBackend` | `omega-sandbox::LocalSandboxBackend` (E2B = adaptateur distant à ajouter) | plugin, **adopted** |
| `ag-block-registry` | registry → `Registre` | `omega-registry::Registry` (Slot/Ensemble + quarantaine) | plugin, planned |
| `ag-tool-registry-groups` | registry → `Registre` | `omega-registry::Registry` | plugin, planned |
| `ag-store-marketplace` | registry → `Registre` | `omega-registry::Registry` | plugin, planned |
| `ag-forge-components` | registry → `Registre` | `omega-registry::Registry` | plugin, planned |
| `ag-skills-registry` | skill-store → `SkillStore` | `omega-context::SkillIndex` | plugin, **adopted** |
| `ag-graphiti-memory` | context-engine → `ContextEngine` | `omega-context::DefaultContextEngine` | plugin, planned |
| `ag-prompt-strategies` | agent-store → `AgentStore` | `plugins/reasoning-strategies` | plugin, planned |
| `ag-subagent-runners` | subagent-runner → `SubagentRunner` | `omega-app::SubagentRunner` | plugin, planned |
| `ag-experts-roster` | subagent-runner → `SubagentRunner` | `omega-app::ExpertRoster` | plugin, planned |
| `ag-llm-providers` | llm-provider → `Provider` | `omega-providers::OllamaProvider` | plugin, **adopted** |
| `ag-model-router-tiers` | router-policy → `ScoringPolicy`/`Selector`/`FallbackPolicy` | `omega-router::CompositeScoringPolicy + TierFallbackPolicy` | plugin, planned |
| `ag-observability-traces` | telemetry-sink → `TelemetrySink` | `omega-otel::{File,Stdout}Sink` + `omega-otel::Provenance` | plugin, **adopted** |
| `ag-config-layers` | config-source → `ConfigSource` | `omega-config` (couches + precedence) | plugin, planned |
| `ag-exec-dag-engine` | — | `omega-app` (orchestrateur de graphe) | service, planned |
| `ag-distributed-queue` | — | `omega-app` (file + verrou distribué) | service, planned |
| `ag-multi-process-runtime` | — | `omega-server` | service, planned |
| `ag-scheduler-cron` | — | `omega-app` (cron + index de jobs) | service, planned |
| `ag-webhook-triggers` | — | `omega-server` (ingress) + `omega-app` | service, planned |
| `ag-dream-consolidation` | — | `omega-app` (job nocturne + ratification) | service, planned |
| `ag-cost-tracking-billing` | — | `omega-otel::Provenance` + ledger de crédits | service, planned |
| `ag-frontend-ui` | — | `omega-server` (cockpit React/TS même origine) | service, planned |
| `ag-streaming-ws` | — | `omega-server` (SSE/WS + rejeu de trace) | service, planned |

Les 8 `adopted` le sont **parce que l'adaptateur existe déjà** dans notre code (vérifié :
`omega-context::ToolSurfaceIndex` `tool_surface.rs:133`, `omega-context::SkillIndex`
`skills.rs:66`, `omega-context::DefaultContextEngine` `engine.rs:42`,
`omega-sandbox::ApprovalQueue` `approvals.rs:56`, `omega-sandbox::LocalSandboxBackend`
`backend.rs:19`, `omega-sandbox::RuleBasedExecPolicy` `policy.rs:119`,
`omega-providers::OllamaProvider` `ollama.rs:73`, `omega-otel::{Stdout,File}Sink`
`sinks.rs:16,41`, `omega-otel::Provenance` `provenance.rs:31`,
`omega-router::{CompositeScoringPolicy,TierFallbackPolicy,ParetoSelector}`
`scoring.rs:8`/`fallback.rs:58`/`selector.rs:9`, `omega-registry::Registry` `lib.rs:85`).

### Cibles prioritaires (les plus fortes pour Omega)

1. **`ag-capability-index-bm25` + `ag-block-contract`** — l'index BM25 unique (outils + blocs +
   catalogue MCP) avec noyau « eager » restreint (`static_tools.py:25`) est le pendant exact de
   `omega-context::ToolSurfaceIndex` ; il fournit la recette de la *divulgation progressive* qui
   manque à notre surface actuelle (recherche lexicale + `find/describe/run_capability` +
   `RETIRED_TOOLS` pour déprécier sans casser).
2. **`ag-gate-effect-approval`** — classifier chaque action par **effet** (`read`/`shell`/
   `platform`/`external`) et n'exiger l'accord humain que sur les effets parkables, en remontant
   le *pire* effet d'un graphe et de ses sous-graphes, avec un classifieur qui ne peut
   qu'**ajouter** une question. Complète `omega-sandbox::ApprovalQueue` sans la remplacer.
3. **`ag-cost-tracking-billing`** — le triptyque coût déclaré (`BlockCost`, 6 types dont
   `TOKENS` avec carte de taux), métrage post-vol (`charge_node_usage`) et raison d'échec
   *collante* (`InsufficientBalanceError` ne se fait pas écraser par une erreur tardive) est le
   modèle le plus mûr pour notre pilier « économie & traçabilité ».
4. **`ag-subagent-runners` + `ag-experts-roster`** — `AgentExecutorBlock` (agent dans l'agent),
   garde de récursion, permissions filles intersectées, sandbox persistante par expert : un
   `SubagentRunner` crédible, à câbler sur `omega-app`.
5. **`ag-prompt-strategies` (Forge)** — 7 stratégies de raisonnement dans des fichiers
   autonomes partageant une infra commune (`base.py`), donc un vrai patron de *policy de boucle*
   interchangeable pour `omega-core`.

## 5. Écarté

Aucune fonctionnalité marquée `rejected` : le sous-ensemble repris ci-dessus couvre le meilleur
de l'app sans dépendre de son héritage Python.

Restent **hors périmètre immédiat**, avec le motif :

- **`ag-exec-dag-engine`, `ag-distributed-queue`, `ag-multi-process-runtime`** — c'est la
  plateforme (DAG + RabbitMQ + 11 processus). Omega reste un harnais mono-tour avec un
  exécuteur local ; reprendre le DAG suppose de choisir d'abord un port d'orchestration
  (service, pas plugin) et un budget Rust réaliste (`L`).
- **`ag-store-marketplace`, `ag-frontend-ui`, `ag-streaming-ws`** — gamme *produit* (monétisation,
  billing Stripe, avatars, analytics PostHog, multi-tenant Postgres/Prisma). Utile comme
  référence d'ergonomie ; coût `L` pour un gain nul sur le cœur. Le point réutilisable est
  la *génération* du client OpenAPI (`orval.config.ts`) et la génération de fixtures côté
  serveur (`gate/card_fixture_test.py`) pour empêcher la dérive UI/serveur — déjà notre
  approche dans `omega-server` (une origine, schéma unique).
- **Licence** : tout ce qui vient de `autogpt_platform/` est sous **PolyForm Shield 1.0.0**
  (clause *non-compete*) ; nous ne recopions aucun code, seulement des **idées d'architecture**
  re-décrites en Rust, et les fichiers `classic/` cités (Forge, stratégies) sont sous **MIT**.
  C'est une contrainte à garder en tête si une reprise plus littérale était envisagée.

## 6. Sources

- Clone local `/home/tayk0oo/omega-re/autogpt` (commit `06d623e`, 2026-10-16), lecture seule.
  Fichiers cités en `refs` du JSON : `autogpt_platform/backend/backend/{blocks,executor,copilot,data,api,monitoring,integrations,util,sdk}/…`,
  `autogpt_platform/frontend/orval.config.ts`, `classic/forge/forge/agent/protocols.py`,
  `classic/original_autogpt/autogpt/agents/prompt_strategies/*`.
- Preuve machine : `docs/omega/research/rea/evidence/intake-autogpt.summary.json`
  (`ev_7f447ff1…`, `confidence: derived`, `limitations_count: 6294`).
- Licence : `LICENSE:1-4`, `LICENSE:27-191` (PolyForm Shield 1.0.0), `LICENSE:196-220` (MIT) ;
  `README.md:188-193` ; GitHub renvoie `NOASSERTION`.
- Docs officielles : https://docs.agpt.co/ (`docs/platform/*` du clone : `data-flow-and-execution.md`,
  `agent-blocks.md`, `block-sdk-guide.md`, `autopilot.md`, `credits-and-billing.md`,
  `copilot-local-llm.md`, `integrating/*`) ; MCP : https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http
  (référence citée dans `blocks/mcp/client.py:13-17`).
