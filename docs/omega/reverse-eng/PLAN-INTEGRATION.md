# PLAN-INTEGRATION — ce que les harnais convergent à demander

> **Généré** par `omega/scripts/integration_plan.py` (déterministe, zéro LLM) à partir
> des dissections embarquées : **35 applications**, **745 fonctionnalités**.
> **Lecture :** une *capacité* réclamée par plusieurs harnais est un signal
> structurel — c'est elle qu'il faut implémenter en premier (port + plugins).

| Chiffre | Valeur |
|---|---|
| Applications disséquées | 35 |
| Fonctionnalités | 745 |
| Adoptées (déjà dans Omega) | 167 |
| Planifiées (à faire) | 556 |
| Écartées (assumées) | 22 |

## Priorités par consensus

| # | Capacité visée | Harnais | Coût min | Exemples (harnais → adaptateur) |
|---|---|---|---|---|
| 1 | `context-engine` | 28 (agentgpt, aider, autogpt, claude-code…) | S | AgentGPT → `omega-context::DefaultContextEngine` ; AgentGPT → `omega-context::DefaultContextEngine` ; AgentGPT → `omega-context::DefaultContextEngine` |
| 2 | `subagent-runner` | 23 (aider, autogpt, claude-code, cline…) | S | Aider → `plugins/architect-loop` ; AutoGPT → `omega-app::SubagentRunner (a creer : spawn/resume + budget)` ; AutoGPT → `omega-app::ExpertRoster (a creer : roster + routines + credentials)` |
| 3 | `tool-surface` | 21 (agentgpt, aider, autogpt, claude-code…) | S | AgentGPT → `omega-context::ToolSurfaceIndex` ; AgentGPT → `omega-context::ToolSurfaceIndex` ; AgentGPT → `omega-context::ToolSurfaceIndex` |
| 4 | `config-source` | 19 (agentgpt, aider, autogpt, claude-code…) | S | AgentGPT → `omega-config (auth providers)` ; AgentGPT → `omega-config (oauth credentials)` ; AgentGPT → `omega-config (feature flags)` |
| 5 | `registry` | 16 (aider, autogpt, claude-code, cline…) | S | Aider → `omega-registry (slots modeles)` ; AutoGPT → `omega-registry::Registry (Slot/Ensemble + quarantaine)` ; AutoGPT → `omega-registry::Registry (Slot/Ensemble + quarantaine)` |
| 6 | `event-store` | 14 (agentgpt, claude-code, crewai, crush…) | S | AgentGPT → `omega-app::FileEventStore` ; AgentGPT → `omega-app::FileEventStore` ; Claude Code → `plugins/hook-bus (matcher + if + décision exit-code/JSON + async)` |
| 7 | `memory-store` | 13 (claude-code, codex, crewai, deep-agents…) | S | Claude Code → `plugins/memory-bounded (CLAUDE.md layers + MEMORY.md 25KB + audit)` ; Codex CLI → `plugins/memory-local` ; CrewAI → `plugins/memory-unified (analyse LLM + score composite + backend vectoriel)` |
| 8 | `agent-store` | 12 (agentgpt, autogpt, claude-code, crewai…) | S | AgentGPT → `plugins/agent-autonome (objectif → tâches → exécution → synthèse)` ; AgentGPT → `plugins/agent-autonome (function calling)` ; AutoGPT → `plugins/reasoning-strategies (a creer)` |
| 9 | `exec-policy` | 12 (aider, autogpt, continue, crewai…) | S | Aider → `plugins/reflect-lint-test` ; AutoGPT → `omega-sandbox::RuleBasedExecPolicy` ; Continue → `omega-sandbox::RuleBasedExecPolicy (mode plan = deny-by-default hors lecture)` |
| 10 | `(execution)` | 11 (autogpt, continue, crewai, crush…) | S | AutoGPT → `omega-app (orchestrateur de graphe, a extraire du noyau)` ; AutoGPT → `omega-app (file de travail + verrou distribue)` ; AutoGPT → `omega-server (processus unique : API + cockpit)` |
| 11 | `sandbox-backend` | 11 (claude-code, devin, dsh-desktop, gemini-cli…) | M | Claude Code → `plugins/sandbox-worktree (git worktree + vérification cwd/commande + cleanup)` ; Devin → `plugins/sandbox-docker` ; DSH Desktop → `omega-sandbox::LocalSandboxBackend` |
| 12 | `telemetry-sink` | 9 (aider, crewai, devin, dsh-desktop…) | S | Aider → `plugins/eval-harness` ; CrewAI → `omega-otel::{Stdout,File}Sink + omega-otel::Provenance` ; Devin → `omega-otel::StdoutSink` |
| 13 | `permission-policy` | 9 (autogpt, deepseek-harness, devin, dsh-desktop…) | S | AutoGPT → `omega-sandbox::ApprovalQueue` ; DeepSeek Harness → `omega-sandbox::ApprovalQueue` ; Devin → `omega-sandbox::ApprovalQueue` |
| 14 | `skill-store` | 9 (continue, devin, dsh-web, goose…) | S | Continue → `omega-context::SkillIndex (etendu aux blocs de prompt reutilisables)` ; Devin → `omega-context::SkillIndex` ; DSH Web → `omega-context::SkillIndex` |
| 15 | `(ui)` | 9 (agentgpt, autogpt, continue, crush…) | M | AgentGPT → `—` ; AutoGPT → `omega-server (cockpit React/TS servi sur la meme origine)` ; Continue → `omega-cli (binaire TUI sur omega-app + omega-server)` |
| 16 | `scheduler-job` | 9 (aider, claude-code, cline, devin…) | M | Aider → `plugins/file-watcher` ; Claude Code → `plugins/scheduler-cron (local + cloud-runner + skill verify/simplify pre-commit)` ; Cline → `plugins/scheduler-cron` |
| 17 | `tool` | 7 (cline, deepseek-harness, gptme, hermes…) | S | Cline → `plugins/tool-sdk` ; DeepSeek Harness → `omega-sandbox + omega-plugin-api (planned)` ; gptme → `plugins/tool-shell` |
| 18 | `(automatisation)` | 7 (autogpt, continue, deepseek-harness, goose…) | M | AutoGPT → `omega-app (planificateur cron + index de jobs)` ; AutoGPT → `omega-server (ingress webhook) + omega-app (mapping payload -> nœud)` ; Continue → `omega-sandbox::LocalSandboxBackend (worktrees) + omega-server (API de revue)` |
| 19 | `llm-provider` | 6 (continue, crewai, opencode, openhands…) | S | Continue → `omega-providers::OllamaProvider (catalogue multi-fournisseurs a etendre)` ; CrewAI → `omega-providers::OllamaProvider` ; OpenCode → `omega-providers (catalogue par transforms + scoring communautaire)` |
| 20 | `router-policy` | 5 (autogpt, cursor, open-interpreter, plandex…) | S | AutoGPT → `omega-router::CompositeScoringPolicy + TierFallbackPolicy` ; Cursor → `plugins/model-router` ; Open Interpreter → `plugins/harness-router` |
| 21 | `hook` | 5 (cline, codex, deep-agents, deepseek-harness…) | M | Cline → `plugins/hook-runner` ; Codex CLI → `plugins/hook-command` ; Deep Agents → `—` |
| 22 | `channel` | 5 (cline, deep-agents, gemini-cli, langgraph…) | M | Cline → `plugins/channel-*` ; Deep Agents → `—` ; Gemini CLI → `plugins/voice-capture (provider de transcription derriere un port)` |
| 23 | `hook-bus` | 4 (cursor, gptme, openclaw, windsurf) | S | Cursor → `plugins/hooks` ; gptme → `plugins/hook-bus` ; OpenClaw → `plugins/hook-bus (bus async + budgets + ordre deterministe)` |
| 24 | `task-classifier` | 4 (gemini-cli, metagpt, plandex, qwen-code) | S | Gemini CLI → `omega-router::HeuristicTaskClassifier (variante heuristique existante)` ; MetaGPT → `plugins/planner-review + omega-context (contexte structure compact)` ; MetaGPT → `omega-router::HeuristicTaskClassifier + plugins/quick-path` |
| 25 | `agent-loop` | 4 (gpt-engineer, gptme, langgraph, plandex) | S | GPT Engineer → `plugins/agent-loop (pipeline d'etapes explicite, sans boucle cachee)` ; gptme → `plugins/agent-loop` ; LangGraph → `plugins/agent-loop (boucle plan/act sur le routage Omega)` |
| 26 | `agent-server` | 4 (goose, langgraph, openhands, zed) | M | Goose → `—` ; Goose → `—` ; LangGraph → `omega-server (client HTTP interne du cockpit)` |
| 27 | `agent-runtime` | 3 (cline, goose, void) | S | Cline → `plugins/agent-loop` ; Goose → `—` ; Goose → `—` |
| 28 | `workflow-engine` | 3 (crewai, devin, langgraph) | S | CrewAI → `plugins/flow-runtime (DAG etatise + persistance + human-in-the-loop)` ; Devin → `plugins/workflow-dag` ; LangGraph → `plugins/workflow-graph (graphe d'etats + canaux, au-dessus de omega-app::App)` |
| 29 | `provenance` | 3 (crewai, dsh-web, rea) | S | CrewAI → `omega-otel::Provenance` ; DSH Web → `omega-otel::Provenance` ; REA (Reverse Engineer Anything) → `plugins/tool-rea-evidence` |
| 30 | `ui-panel` | 3 (deep-agents, langgraph, zed) | S | Deep Agents → `—` ; LangGraph → `omega-ui (panneaux du cockpit, canal de messages a ajouter)` ; Zed → `plugins/agent-panel` |

## Consensus fort (≥ 4 harnais réclament la même capacité)

- **`context-engine`** — 28 harnais (agentgpt, aider, autogpt, claude-code, cline, codex, continue, crewai, crush, deepseek-harness, gemini-cli, goose, gpt-engineer, gpt-pilot, gptme, hermes, kilo-code, langgraph, metagpt, openclaw, opencode, openhands, plandex, qwen-code, tabby, void, windsurf, zed) — coût min **S**
- **`subagent-runner`** — 23 harnais (aider, autogpt, claude-code, cline, codex, continue, crewai, crush, cursor, deep-agents, deepseek-harness, devin, dsh-web, gemini-cli, gptme, hermes, kilo-code, langgraph, open-interpreter, openclaw, opencode, openhands, zed) — coût min **S**
- **`tool-surface`** — 21 harnais (agentgpt, aider, autogpt, claude-code, continue, crewai, crush, cursor, deep-agents, deepseek-harness, devin, gemini-cli, kilo-code, langgraph, metagpt, openclaw, openhands, qwen-code, rea, void, windsurf) — coût min **S**
- **`config-source`** — 19 harnais (agentgpt, aider, autogpt, claude-code, codex, continue, crewai, cursor, dsh-desktop, hermes, kilo-code, langgraph, opencode, plandex, rea, tabby, void, windsurf, zed) — coût min **S**
- **`registry`** — 16 harnais (aider, autogpt, claude-code, cline, continue, crewai, deepseek-harness, dsh-desktop, dsh-web, gemini-cli, goose, kilo-code, metagpt, opencode, qwen-code, void) — coût min **S**
- **`event-store`** — 14 harnais (agentgpt, claude-code, crewai, crush, deepseek-harness, devin, dsh-desktop, dsh-web, gemini-cli, metagpt, open-interpreter, opencode, qwen-code, void) — coût min **S**
- **`memory-store`** — 13 harnais (claude-code, codex, crewai, deep-agents, devin, gemini-cli, gpt-engineer, gptme, hermes, kilo-code, langgraph, openclaw, windsurf) — coût min **S**
- **`agent-store`** — 12 harnais (agentgpt, autogpt, claude-code, crewai, crush, deepseek-harness, gpt-pilot, kilo-code, metagpt, opencode, qwen-code, tabby) — coût min **S**
- **`exec-policy`** — 12 harnais (aider, autogpt, continue, crewai, crush, deepseek-harness, dsh-desktop, gemini-cli, gpt-pilot, openclaw, rea, void) — coût min **S**
- **`(execution)`** — 11 harnais (autogpt, continue, crewai, crush, deepseek-harness, dsh-web, gemini-cli, kilo-code, langgraph, opencode, qwen-code) — coût min **S**
- **`sandbox-backend`** — 11 harnais (claude-code, devin, dsh-desktop, gemini-cli, gptme, hermes, openhands, plandex, rea, void, windsurf) — coût min **M**
- **`telemetry-sink`** — 9 harnais (aider, crewai, devin, dsh-desktop, gpt-pilot, metagpt, plandex, tabby, void) — coût min **S**
- **`permission-policy`** — 9 harnais (autogpt, deepseek-harness, devin, dsh-desktop, dsh-web, gemini-cli, goose, metagpt, tabby) — coût min **S**
- **`skill-store`** — 9 harnais (continue, devin, dsh-web, goose, hermes, metagpt, open-interpreter, rea, windsurf) — coût min **S**
- **`(ui)`** — 9 harnais (agentgpt, autogpt, continue, crush, deepseek-harness, gemini-cli, gpt-engineer, gpt-pilot, tabby) — coût min **M**
- **`scheduler-job`** — 9 harnais (aider, claude-code, cline, devin, gptme, hermes, kilo-code, openclaw, openhands) — coût min **M**
- **`tool`** — 7 harnais (cline, deepseek-harness, gptme, hermes, openhands, plandex, rea) — coût min **S**
- **`(automatisation)`** — 7 harnais (autogpt, continue, deepseek-harness, goose, langgraph, qwen-code, tabby) — coût min **M**
- **`llm-provider`** — 6 harnais (continue, crewai, opencode, openhands, tabby, void) — coût min **S**
- **`router-policy`** — 5 harnais (autogpt, cursor, open-interpreter, plandex, windsurf) — coût min **S**
- **`hook`** — 5 harnais (cline, codex, deep-agents, deepseek-harness, gemini-cli) — coût min **M**
- **`channel`** — 5 harnais (cline, deep-agents, gemini-cli, langgraph, openclaw) — coût min **M**
- **`hook-bus`** — 4 harnais (cursor, gptme, openclaw, windsurf) — coût min **S**
- **`task-classifier`** — 4 harnais (gemini-cli, metagpt, plandex, qwen-code) — coût min **S**
- **`agent-loop`** — 4 harnais (gpt-engineer, gptme, langgraph, plandex) — coût min **S**
- **`agent-server`** — 4 harnais (goose, langgraph, openhands, zed) — coût min **M**

## Quick wins (coût S, le plus réclamé)

- `context-engine` — 28 harnais (AgentGPT, AgentGPT, AgentGPT, Aider)
- `subagent-runner` — 23 harnais (Aider, AutoGPT, AutoGPT, Claude Code)
- `tool-surface` — 21 harnais (AgentGPT, AgentGPT, AgentGPT, AgentGPT)
- `config-source` — 19 harnais (AgentGPT, AgentGPT, AgentGPT, AgentGPT)
- `registry` — 16 harnais (Aider, AutoGPT, AutoGPT, AutoGPT)
- `event-store` — 14 harnais (AgentGPT, AgentGPT, Claude Code, CrewAI)
- `memory-store` — 13 harnais (Claude Code, Codex CLI, CrewAI, CrewAI)
- `agent-store` — 12 harnais (AgentGPT, AgentGPT, AutoGPT, Claude Code)
- `exec-policy` — 12 harnais (Aider, AutoGPT, Continue, Continue)
- `(execution)` — 11 harnais (AutoGPT, AutoGPT, AutoGPT, Continue)
- `telemetry-sink` — 9 harnais (Aider, CrewAI, Devin, DSH Desktop)
- `permission-policy` — 9 harnais (AutoGPT, DeepSeek Harness, Devin, DSH Desktop)
- `skill-store` — 9 harnais (Continue, Devin, DSH Web, Goose)
- `tool` — 7 harnais (Cline, DeepSeek Harness, gptme, Hermes Agent)
- `llm-provider` — 6 harnais (Continue, CrewAI, OpenCode, OpenHands)

## Écarté (et pourquoi c'est assumé)

- `extension` — 5 fonctionnalité(s) écartée(s)
- `configuration` — 3 fonctionnalité(s) écartée(s)
- `observabilite` — 3 fonctionnalité(s) écartée(s)
- `ui` — 3 fonctionnalité(s) écartée(s)
- `routage` — 3 fonctionnalité(s) écartée(s)
- `outils` — 2 fonctionnalité(s) écartée(s)
- `securite` — 1 fonctionnalité(s) écartée(s)
- `execution` — 1 fonctionnalité(s) écartée(s)
- `sandbox-backend` — 1 fonctionnalité(s) écartée(s)

---

_Régénérer : `python3 omega/scripts/integration_plan.py`_
