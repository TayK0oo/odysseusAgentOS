# Dissection — gptme (Erik Bjareh / contributors, MIT, Python)

> Repo : https://github.com/gptme/gptme — version observée : 0.35.1.
> Perspective : Plugins + Skills + Lessons + Hooks ; MCP dans les deux sens ; ACP ; mémoire inter-harnais ; agents autonomes ; sandbox Docker/Wasmtime ; compression de contexte ; token awareness.
> Clone local : `/home/tayk0oo/omega-re/gptme` (depth 1).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Erik Bjareh Seri + contributeurs |
| Licence | MIT (OSS : oui) |
| Langage | Python (~typage strict, `py.typed`) |
| Repo | https://github.com/gptme/gptme |
| Architecture | Harnais ReAct (boucle step/turn dans `gptme/chat.py`) : ~60 outils `ToolSpec` auto-dérivés, bus de hooks typés (~20 types), leçons/skills à matching hybride, client + serveur MCP, agent ACP, sandbox poly-backend, mémoire fichier en couches, compression adaptative, services systemd/launchd. |

## 2. Entrées

| Entrée | Fichier |
|---|---|
| CLI `gptme` (chat, service, skills, mcp, memory, hooks...) | `gptme/cli/main.py:63-244` |
| Serveur `gptme-server` (API v2 + WebUI + skills/A2A/tasks APIs) | `gptme/server/app.py:1-339`, `gptme/server/api_v2.py` |
| Agent ACP `gptme-acp` | `gptme/acp/__main__.py:1-255`, `gptme/acp/agent.py:254-300` |
| Serveur MCP `gptme mcp serve` | `gptme/cli/cmd_mcp_serve.py:1-72` |
| Services autonomes `gptme service` (systemd/launchd) | `gptme/agent/service.py:53-180` |
| Points d'extension | `gptme/plugins/` (entrypoints), `gptme/lessons/skills/`, `gptme/hooks/` |
| Config | `gptme.toml`, `~/.config/gptme/`, `gptme/config/core.py:44-268` |

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Boucle ReAct step/turn + overflow recovery | agents | Tours/étapes, régénération réduite si dépassement, compaction post-outil | `gptme/chat.py:212-397,948-1011` | agent-loop → AgentLoop → plugins/agent-loop | planned | M |
| Leçons/skills matching hybride + auto-include budgété | contexte | Index, matcher hybride, gardes, injection sous budget tokens | `gptme/lessons/index.py:82-283`, `selector.py:130-257` | skill-store → SkillStore → omega-context::SkillIndex | adopted | S |
| Bus de hooks typés (~20 types, sync+async) | extension | Registre central autour de la boucle, extensible par plugins | `gptme/hooks/types.py:66-140`, `registry.py:109-329` | hook-bus → HookBus → plugins/hook-bus | planned | M |
| Surface d'outils typés auto-dérivés (~60) | outils | ToolSpec dérivés des callables, hydration à la demande | `gptme/tools/base.py:544-679` | tool-surface → ToolSurface → omega-context::ToolSurfaceIndex | adopted | M |
| Shell persistant + jobs background (tmux) | outils | Sessions persistantes, cwd suivi, nettoyage process | `gptme/tools/shell.py:645-830` | tool → ToolExecutor → plugins/tool-shell | planned | M |
| Client MCP (stdio+HTTP) | outils | Outils/ressources/prompts distants → outils locaux | `gptme/mcp/client.py:60-310`, `tools/mcp_adapter.py:290-523` | mcp-client → McpClient → plugins/mcp-client | planned | L |
| Serveur MCP (expose gptme) | outils | Outils/agents gptme vers autres harnais (MCP + A2A) | `gptme/mcp/server.py:65-153` | mcp-server → McpServer → plugins/mcp-server | planned | M |
| Agent ACP (client + serveur) | agents | Sessions, streaming, permissions mappées | `gptme/acp/agent.py:588-1492` | acp-agent → AcpAgent → plugins/acp-agent | planned | L |
| Sandbox poly-backend | securite | firejail/bwrap (shell), docker/wasmtime (python) | `gptme/sandbox.py:91-653` | sandbox-backend → SandboxBackend → plugins/sandbox-docker-wasmtime | planned | L |
| Mémoire en couches partagée Claude Code | memoire | project/cc/agent/user, lecture CC native, écriture atomique | `gptme/memory/roots.py:1-97`, `store.py:244-478` | memory-store → MemoryStore → plugins/memory-layered | planned | M |
| Compression contexte adaptative | contexte | Extractif scoré, autocompact par seuils, strip reasoning | `gptme/context/adaptive_compressor.py:259-438` | context-engine → ContextEngine → plugins/context-compress | planned | L |
| Token/cost/cache awareness | contexte | Budgets, alertes coûts, protection prefix-cache | `gptme/hooks/token_awareness.py:56-225`, `cost_awareness.py:291-463` | cost-guard → CostGuard → plugins/cost-awareness | planned | S |
| Agents autonomes systemd/launchd | automatisation | Services + timers, file de prompts, watcher | `gptme/agent/service.py:53-180` | scheduler-job → Scheduler → plugins/scheduler-service | planned | M |
| Sous-agents isolés parallèles | agents | Contexte séparé, batch borné, reprise | `gptme/tools/subagent/api.py:1-300` | subagent-runner → SubagentRunner → plugins/subagent-isolated | planned | M |
| Confirmation + guardrails | securite | Confirm pré-exécution, anti-secrets, anti-injection | `gptme/hooks/guardrails.py:224-333` | permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue | adopted | S |
| Télémétrie OTel + JSONL + checkpoints | observabilite | Traces, logs conversations, checkpoints git, métriques | `gptme/telemetry.py:64-350`, `checkpoint.py:240-429` | telemetry-sink → TelemetrySink → omega-otel::StdoutSink | adopted | S |

## 4. Traduction en plugins

- `gm-lessons-skills` : skill-store → SkillStore → `omega-context::SkillIndex` (plugin, adopted — l'index existe ; ajouter matching hybride + auto-include budgété).
- `gm-tool-surface` : tool-surface → ToolSurface → `omega-context::ToolSurfaceIndex` (plugin, adopted — ajouter dérivation auto des specs).
- `gm-confirm-guardrails` : permission-policy → PermissionPolicy → `omega-sandbox::ApprovalQueue` (plugin, adopted — ajouter guardrails anti-secrets + anti-injection).
- `gm-observability` : telemetry-sink → TelemetrySink → `omega-otel::StdoutSink` (plugin, adopted — ajouter checkpoints git).
- `gm-react-loop` : agent-loop → AgentLoop → `plugins/agent-loop` (plugin, planned).
- `gm-hooks-registry` : hook-bus → HookBus → `plugins/hook-bus` (plugin, planned — le morceau le plus structural : ~20 types d'événements).
- `gm-shell-tmux` : tool → ToolExecutor → `plugins/tool-shell` (plugin, planned).
- `gm-mcp-client` : mcp-client → McpClient → `plugins/mcp-client` (plugin, planned).
- `gm-mcp-serve` : mcp-server → McpServer → `plugins/mcp-server` (plugin, planned).
- `gm-acp-agent` : acp-agent → AcpAgent → `plugins/acp-agent` (plugin, planned).
- `gm-sandbox-poly` : sandbox-backend → SandboxBackend → `plugins/sandbox-docker-wasmtime` (plugin, planned — Omega n'a que le backend local).
- `gm-shared-memory` : memory-store → MemoryStore → `plugins/memory-layered` (plugin, planned — la couche `cc` est la killer-feature inter-harnais).
- `gm-context-compress` : context-engine → ContextEngine → `plugins/context-compress` (plugin, planned).
- `gm-token-cost-awareness` : cost-guard → CostGuard → `plugins/cost-awareness` (plugin, planned).
- `gm-autonomous-service` : scheduler-job → Scheduler → `plugins/scheduler-service` (plugin, planned).
- `gm-subagents` : subagent-runner → SubagentRunner → `plugins/subagent-isolated` (plugin, planned).

## 5. Écarté

- Rien de rejeté : tout est repris (4 adopted, 12 planned). Seule réserve : le backend `wasmtime` (CPython WASI) est coûteux à porter en Rust — viser d'abord `plugins/sandbox-docker`, wasmtime en second.
- Profils d'agents (`gptme/profiles.py:47-103`) non isolés en fonctionnalité propre : couverts par `gm-confirm-guardrails` (restriction d'outils) + config ; à réévaluer si les profils deviennent un besoin.

## 6. Sources

- https://github.com/gptme/gptme
- Clone : `/home/tayk0oo/omega-re/gptme`
- Fichiers : `gptme/chat.py`, `gptme/hooks/{types,registry,token_awareness,cost_awareness,cache_awareness,guardrails,confirm,knowledge_inject}.py`, `gptme/lessons/{index,matcher,hybrid_matcher,selector,auto_include}.py`, `gptme/tools/{base,shell,python,mcp_adapter}.py`, `gptme/tools/subagent/`, `gptme/mcp/{client,server,registry}.py`, `gptme/acp/agent.py`, `gptme/sandbox.py`, `gptme/memory/{roots,store}.py`, `gptme/dirs.py`, `gptme/context/{compress,adaptive_compressor}.py`, `gptme/agent/service.py`, `gptme/telemetry.py`, `gptme/checkpoint.py`, `gptme/logmanager/manager.py`
