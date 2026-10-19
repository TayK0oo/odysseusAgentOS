# Dissection — OpenHands

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | OpenHands (All Hands AI) |
| Licence | MIT |
| Langage | Python + TypeScript |
| Repo UI | https://github.com/OpenHands/OpenHands (devenu **Agent Canvas**, `@openhands/agent-canvas` v1.27.1) |
| Repo backend | https://github.com/OpenHands/software-agent-sdk (SDK + Agent Server + tools + workspaces) |
| Architecture | SDK Python (tools, LLM, workspaces, ACP) + Agent Server FastAPI (REST `/api`, ~20 routers, isolation par conteneur Docker) + Canvas UI React/TS ; automations via paquet séparé `openhands-automation`. |

Constat important : le dépôt `OpenHands/OpenHands` ne contient plus le backend Python —
il n'est que le frontend (Agent Canvas). Le backend vit dans `software-agent-sdk`
(`openhands-sdk/`, `openhands-agent-server/`, `openhands-tools/`, `openhands-workspace/`,
`clients/typescript/`). Les `refs` ci-dessous préfixent `sdk/` (= software-agent-sdk)
et `canvas/` (= OpenHands/OpenHands).

## 2. Entrées

- **Binaires/CLI** : `agent-canvas` (`canvas/bin/agent-canvas.mjs`) ; `openhands-agent-server`
  (`sdk/.../agent_server/__main__.py:232-365`) ; stack locale `npm run dev` (agent-server +
  automation via `uvx` + ingress, cf. `canvas/docs/architecture.md`, `canvas/docs/SELF_HOSTING.md`).
- **Points d'extension** : plugins SDK (`sdk/.../sdk/plugin/`), hooks (`sdk/.../sdk/hooks/`),
  extensions Canvas (`sdk/.../agent_server/canvas_extensions/`, `canvas/src/extensions/`).
- **Config** : `canvas/config/defaults.json`, `.env.sample`, schémas de settings générés
  (`sdk/.../sdk/settings/model.py`), persistance fichier chiffrée
  (`sdk/.../agent_server/persistence/store.py`), `specs/*.md` (contrats front).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Agent Server REST multi-routers | exécution | Toutes les opérations agent derrière `/api` unique | `sdk/.../agent_server/api.py:539-589`, `server_details_router.py:113-170` | agent-server → AgentServer | planned | L |
| Cycle de vie des conversations | exécution | start/pause/interrupt/fork/navigate, registre, runs bornés | `sdk/.../conversation_router.py:158-835`, `conversation_registry.py:21` | conversation-store → ConversationStore | planned | L |
| Journal d'événements + streaming live/replay | mémoire | EventLog persistant, pub/sub, replay paginé vers sockets | `sdk/.../sdk/event/base.py:20`, `sdk/.../sdk/conversation/event_store.py:34`, `.../pub_sub.py:33`, `.../session_socket.py:277` | event-store → `omega-app::FileEventStore` | adopted | M |
| Registre d'outils + presets par modèle | outils | Catalogue typé Action/Observation, scellement, presets | `sdk/.../sdk/tool/registry.py:130`, `tool.py:347`, `defaults.py:27` | tool-surface → ToolSurface | planned | M |
| Outils d'exécution | outils | Terminal, fichiers, navigateur, tâches, workflows | `sdk/.../tools/terminal/definition.py:86-268`, `file_editor/`, `browser_use/` | tool → ToolExecutor | planned | L |
| Sous-agents et délégation | agents | delegate, registre de sous-agents, conversations enfants | `sdk/.../tools/delegate/definition.py:16`, `sdk/.../sdk/subagent/registry.py:85` | subagent-runner → SubagentRunner | planned | M |
| Workspaces multi-backends + conteneur/conversation | exécution | local/docker/apptainer/cloud/distant, éviction idle | `sdk/.../sdk/workspace/base.py:28`, `.../workspace/docker/workspace.py:53`, `.../docker_runtime/registry.py:55-352` | sandbox-backend → SandboxBackend | planned | L |
| Agents tiers via ACP | agents | Spawn CLI tierce en sous-processus JSON-RPC stdio | `sdk/.../sdk/agent/acp_agent.py:504`, `sdk/.../sdk/settings/acp_providers.py:176-552`, `canvas/docs/ACP_AGENTS.md` | agent-host → AgentHost | planned | M |
| Registre LLM + fallback + device-login | routage | Providers/modèles, retry, OAuth device, façade OpenAI | `sdk/.../sdk/llm/llm_registry.py:23`, `llm.py:252`, `.../llm_router.py:116-266`, `.../openai/router.py:121` | llm-provider → Provider | planned | M |
| Analyse de risque + confirmation | sécurité | Scoring multi-analyseurs, Always/Never/ConfirmRisky, secrets redactés | `sdk/.../sdk/security/risk.py:13`, `analyzer.py:29`, `confirmation_policy.py:9-43` | permission-policy → `omega-sandbox::ApprovalQueue` | adopted | M |
| Condensation LLM + mémoire budgétée | contexte | Rolling condenser, mémoire fichier tronquée, stuck-detector | `sdk/.../sdk/context/condenser/llm_summarizing_condenser.py:54`, `memory.py:56`, `stuck_detector.py:24` | context-engine → ContextEngine | planned | M |
| Skills chargeables | contexte | Skills user/projet/marketplace, injection bornée | `sdk/.../sdk/skills/skill.py:97-1493`, `trigger.py:13` | skill-store → `omega-context::SkillIndex` | adopted | M |
| Plugins + hooks + extensions Canvas | extension | Plugins installables, hooks d'événements, UI extensible à backend géré | `sdk/.../sdk/plugin/plugin.py:36`, `hooks/manager.py:22`, `.../plugins_router.py:261`, `.../canvas_extensions/manifest.py:204` | extension-host → ExtensionHost | planned | L |
| Automations planifiées + webhooks | automatisation | Backend dédié, tags de conversation, observabilité | `canvas/src/api/automation-service/automation-service.api.ts:60-737`, `agent-server-adapter.ts:558-572` | scheduler-job → Scheduler | planned | M |
| Settings/profils/secrets chiffrés | configuration | Couches + schémas générés + chiffrement au repos | `sdk/.../sdk/settings/model.py:113`, `.../persistence/store.py:294-916`, `cipher.py:22` | config-source → `omega-config` | adopted | S |
| Télémétrie à consentement + stats | observabilité | Sink PostHog/HTTP, kill-switch, tokens/coûts | `sdk/.../agent_server/telemetry/sink.py:42`, `service.py:56`, `policy.py:43`, `canvas/src/services/telemetry.ts:283` | telemetry-sink → `omega-otel::{Stdout,File}Sink` | adopted | S |
| Canvas UI | ui | Chat/terminal/navigateur/fichiers, multi-backends, lib embarquable | `canvas/docs/architecture.md`, `agent-server-adapter.ts:202-414`, `canvas-ui-client-tool.ts:9-60` | ui-shell → UiShell | planned | L |

## 4. Traduction en plugins

- `oh-event-sourcing` : `event-store → EventStore → omega-app::FileEventStore` (plugin, adopted).
- `oh-security-gates` : `permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue` (plugin, adopted).
- `oh-skills-marketplace` : `skill-store → SkillStore → omega-context::SkillIndex` (plugin, adopted).
- `oh-settings-secrets` : `config-source → ConfigSource → omega-config` (couches + précédence) (plugin, adopted).
- `oh-observability` : `telemetry-sink → TelemetrySink → omega-otel::{Stdout,File}Sink` (plugin, adopted).
- Les 12 autres : `plugins/agent-server-rest`, `plugins/conversation-lifecycle`, `plugins/tool-registry`,
  `plugins/tools-exec`, `plugins/subagent-isolated`, `plugins/sandbox-docker`, `plugins/agent-acp`,
  `plugins/llm-registry`, `plugins/context-condenser`, `plugins/extension-host`,
  `plugins/scheduler-cron`, `plugins/canvas-ui` (tous planned).

## 5. Écarté

- **Electron desktop** (`canvas/electron/`) : coquille de distribution, rien à reprendre dans le harnais.
- **Passerelle OpenAI-compatible** (`.../openai/router.py`) : notée dans `oh-llm-registry`, non reprise
  telle quelle (Omega expose son propre protocole ; compat OpenAI = plugin futur éventuel).
- **Logins OAuth propriétaires** (Keychain macOS, `~/.claude/`, `~/.codex/`) : mécanisme d'auth
  spécifique aux CLIs tierces, non portable ; seul le pattern « registry de providers + résolution
  de secrets » est retenu (`oh-acp-agents`, `oh-settings-secrets`).

## 6. Sources

- https://github.com/OpenHands/OpenHands (MIT) — `docs/architecture.md`, `docs/ACP_AGENTS.md`,
  `docs/SELF_HOSTING.md`, `src/api/*`, `src/services/telemetry.ts`, `src/extensions/`, `config/defaults.json`.
- https://github.com/OpenHands/software-agent-sdk — `openhands-sdk/openhands/sdk/{tool,agent,llm,event,conversation,workspace,skills,subagent,hooks,plugin,security,mcp,settings,secret,automation}/`,
  `openhands-agent-server/openhands/agent_server/*_router.py`, `openhands-tools/`, `openhands-workspace/`.
- Clones d'inventaire (non livrés) : `/home/tayk0oo/omega-re/openhands` (commit `d9ecec2`),
  `/home/tayk0oo/omega-re/sdk`.
