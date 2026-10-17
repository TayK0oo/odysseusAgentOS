# PLAN-INTEGRATION — ce que les harnais convergent à demander

> **Généré** par `omega/scripts/integration_plan.py` (déterministe, zéro LLM) à partir
> des dissections embarquées : **33 applications**, **709 fonctionnalités**.
> **Lecture :** une *capacité* réclamée par plusieurs harnais est un signal
> structurel — c'est elle qu'il faut implémenter en premier (port + plugins).

| Chiffre | Valeur |
|---|---|
| Applications disséquées | 33 |
| Fonctionnalités | 709 |
| Adoptées (déjà dans Omega) | 152 |
| Planifiées (à faire) | 538 |
| Écartées (assumées) | 19 |

## Priorités par consensus

| # | Capacité visée | Harnais | Coût min | Exemples (harnais → adaptateur) |
|---|---|---|---|---|
| 1 | `context-engine` | 28 (agentgpt, aider, autogpt, claude-code…) | S | AgentGPT → `omega-context::DefaultContextEngine` ; AgentGPT → `omega-context::DefaultContextEngine` ; AgentGPT → `omega-context::DefaultContextEngine` |
| 2 | `subagent-runner` | 21 (aider, autogpt, claude-code, cline…) | S | Aider → `plugins/architect-loop` ; AutoGPT → `omega-app::SubagentRunner (a creer : spawn/resume + budget)` ; AutoGPT → `omega-app::ExpertRoster (a creer : roster + routines + credentials)` |
| 3 | `tool-surface` | 20 (agentgpt, aider, autogpt, claude-code…) | S | AgentGPT → `omega-context::ToolSurfaceIndex` ; AgentGPT → `omega-context::ToolSurfaceIndex` ; AgentGPT → `omega-context::ToolSurfaceIndex` |
| 4 | `config-source` | 19 (agentgpt, aider, autogpt, claude-code…) | S | AgentGPT → `omega-config (auth providers)` ; AgentGPT → `omega-config (oauth credentials)` ; AgentGPT → `omega-config (feature flags)` |
| 5 | `registry` | 15 (aider, autogpt, claude-code, cline…) | S | Aider → `omega-registry (slots modeles)` ; AutoGPT → `omega-registry::Registry (Slot/Ensemble + quarantaine)` ; AutoGPT → `omega-registry::Registry (Slot/Ensemble + quarantaine)` |
| 6 | `event-store` | 13 (agentgpt, claude-code, crewai, crush…) | S | AgentGPT → `omega-app::FileEventStore` ; AgentGPT → `omega-app::FileEventStore` ; Claude Code → `plugins/hook-bus (matcher + if + décision exit-code/JSON + async)` |
| 7 | `agent-store` | 12 (agentgpt, autogpt, claude-code, crewai…) | S | AgentGPT → `plugins/agent-autonome (objectif → tâches → exécution → synthèse)` ; AgentGPT → `plugins/agent-autonome (function calling)` ; AutoGPT → `plugins/reasoning-strategies (a creer)` |
| 8 | `exec-policy` | 12 (aider, autogpt, continue, crewai…) | S | Aider → `plugins/reflect-lint-test` ; AutoGPT → `omega-sandbox::RuleBasedExecPolicy` ; Continue → `omega-sandbox::RuleBasedExecPolicy (mode plan = deny-by-default hors lecture)` |
| 9 | `memory-store` | 12 (claude-code, codex, crewai, devin…) | S | Claude Code → `plugins/memory-bounded (CLAUDE.md layers + MEMORY.md 25KB + audit)` ; Codex CLI → `plugins/memory-local` ; CrewAI → `plugins/memory-unified (analyse LLM + score composite + backend vectoriel)` |
| 10 | `sandbox-backend` | 11 (claude-code, devin, dsh-desktop, gemini-cli…) | M | Claude Code → `plugins/sandbox-worktree (git worktree + vérification cwd/commande + cleanup)` ; Devin → `plugins/sandbox-docker` ; DSH Desktop → `omega-sandbox::LocalSandboxBackend` |
| 11 | `(execution)` | 10 (autogpt, continue, crewai, crush…) | S | AutoGPT → `omega-app (orchestrateur de graphe, a extraire du noyau)` ; AutoGPT → `omega-app (file de travail + verrou distribue)` ; AutoGPT → `omega-server (processus unique : API + cockpit)` |
| 12 | `telemetry-sink` | 9 (aider, crewai, devin, dsh-desktop…) | S | Aider → `plugins/eval-harness` ; CrewAI → `omega-otel::{Stdout,File}Sink + omega-otel::Provenance` ; Devin → `omega-otel::StdoutSink` |
| 13 | `(ui)` | 9 (agentgpt, autogpt, continue, crush…) | M | AgentGPT → `—` ; AutoGPT → `omega-server (cockpit React/TS servi sur la meme origine)` ; Continue → `omega-cli (binaire TUI sur omega-app + omega-server)` |
| 14 | `scheduler-job` | 9 (aider, claude-code, cline, devin…) | M | Aider → `plugins/file-watcher` ; Claude Code → `plugins/scheduler-cron (local + cloud-runner + skill verify/simplify pre-commit)` ; Cline → `plugins/scheduler-cron` |
| 15 | `permission-policy` | 8 (autogpt, deepseek-harness, devin, dsh-desktop…) | S | AutoGPT → `omega-sandbox::ApprovalQueue` ; DeepSeek Harness → `omega-sandbox::ApprovalQueue` ; Devin → `omega-sandbox::ApprovalQueue` |
| 16 | `skill-store` | 8 (continue, devin, goose, hermes…) | S | Continue → `omega-context::SkillIndex (etendu aux blocs de prompt reutilisables)` ; Devin → `omega-context::SkillIndex` ; Goose → `—` |
| 17 | `tool` | 7 (cline, deepseek-harness, gptme, hermes…) | S | Cline → `plugins/tool-sdk` ; DeepSeek Harness → `omega-sandbox + omega-plugin-api (planned)` ; gptme → `plugins/tool-shell` |
| 18 | `(automatisation)` | 7 (autogpt, continue, deepseek-harness, goose…) | M | AutoGPT → `omega-app (planificateur cron + index de jobs)` ; AutoGPT → `omega-server (ingress webhook) + omega-app (mapping payload -> nœud)` ; Continue → `omega-sandbox::LocalSandboxBackend (worktrees) + omega-server (API de revue)` |
| 19 | `llm-provider` | 6 (continue, crewai, opencode, openhands…) | S | Continue → `omega-providers::OllamaProvider (catalogue multi-fournisseurs a etendre)` ; CrewAI → `omega-providers::OllamaProvider` ; OpenCode → `omega-providers (catalogue par transforms + scoring communautaire)` |
| 20 | `router-policy` | 5 (autogpt, cursor, open-interpreter, plandex…) | S | AutoGPT → `omega-router::CompositeScoringPolicy + TierFallbackPolicy` ; Cursor → `plugins/model-router` ; Open Interpreter → `plugins/harness-router` |
| 21 | `hook-bus` | 4 (cursor, gptme, openclaw, windsurf) | S | Cursor → `plugins/hooks` ; gptme → `plugins/hook-bus` ; OpenClaw → `plugins/hook-bus (bus async + budgets + ordre deterministe)` |
| 22 | `task-classifier` | 4 (gemini-cli, metagpt, plandex, qwen-code) | S | Gemini CLI → `omega-router::HeuristicTaskClassifier (variante heuristique existante)` ; MetaGPT → `plugins/planner-review + omega-context (contexte structure compact)` ; MetaGPT → `omega-router::HeuristicTaskClassifier + plugins/quick-path` |
| 23 | `agent-loop` | 4 (gpt-engineer, gptme, langgraph, plandex) | S | GPT Engineer → `plugins/agent-loop (pipeline d'etapes explicite, sans boucle cachee)` ; gptme → `plugins/agent-loop` ; LangGraph → `plugins/agent-loop (boucle plan/act sur le routage Omega)` |
| 24 | `hook` | 4 (cline, codex, deepseek-harness, gemini-cli) | M | Cline → `plugins/hook-runner` ; Codex CLI → `plugins/hook-command` ; DeepSeek Harness → `omega-plugin-api (Hook) + omega-sandbox (planned)` |
| 25 | `channel` | 4 (cline, gemini-cli, langgraph, openclaw) | M | Cline → `plugins/channel-*` ; Gemini CLI → `plugins/voice-capture (provider de transcription derriere un port)` ; LangGraph → `plugins/workflow-graph::channels (BinaryOperatorAggregate + Topic)` |
| 26 | `agent-server` | 4 (goose, langgraph, openhands, zed) | M | Goose → `—` ; Goose → `—` ; LangGraph → `omega-server (client HTTP interne du cockpit)` |
| 27 | `agent-runtime` | 3 (cline, goose, void) | S | Cline → `plugins/agent-loop` ; Goose → `—` ; Goose → `—` |
| 28 | `workflow-engine` | 3 (crewai, devin, langgraph) | S | CrewAI → `plugins/flow-runtime (DAG etatise + persistance + human-in-the-loop)` ; Devin → `plugins/workflow-dag` ; LangGraph → `plugins/workflow-graph (graphe d'etats + canaux, au-dessus de omega-app::App)` |
| 29 | `mcp-bridge` | 3 (cline, crewai, zed) | M | Cline → `plugins/mcp-bridge` ; CrewAI → `plugins/mcp-client (transports stdio/http/sse + filtres)` ; Zed → `plugins/mcp-bridge` |
| 30 | `(extension)` | 3 (continue, kilo-code, tabby) | M | Continue → `omega-server (surface IDE externe, meme origine que le cockpit)` ; Continue → `omega-server (surface IDE externe, protocole unique au lieu de messages dupliques)` ; Kilo Code → `omega-server (surface editeur, a evaluer)` |

## Consensus fort (≥ 4 harnais réclament la même capacité)

- **`context-engine`** — 28 harnais (agentgpt, aider, autogpt, claude-code, cline, codex, continue, crewai, crush, deepseek-harness, gemini-cli, goose, gpt-engineer, gpt-pilot, gptme, hermes, kilo-code, langgraph, metagpt, openclaw, opencode, openhands, plandex, qwen-code, tabby, void, windsurf, zed) — coût min **S**
- **`subagent-runner`** — 21 harnais (aider, autogpt, claude-code, cline, codex, continue, crewai, crush, cursor, deepseek-harness, devin, gemini-cli, gptme, hermes, kilo-code, langgraph, open-interpreter, openclaw, opencode, openhands, zed) — coût min **S**
- **`tool-surface`** — 20 harnais (agentgpt, aider, autogpt, claude-code, continue, crewai, crush, cursor, deepseek-harness, devin, gemini-cli, kilo-code, langgraph, metagpt, openclaw, openhands, qwen-code, rea, void, windsurf) — coût min **S**
- **`config-source`** — 19 harnais (agentgpt, aider, autogpt, claude-code, codex, continue, crewai, cursor, dsh-desktop, hermes, kilo-code, langgraph, opencode, plandex, rea, tabby, void, windsurf, zed) — coût min **S**
- **`registry`** — 15 harnais (aider, autogpt, claude-code, cline, continue, crewai, deepseek-harness, dsh-desktop, gemini-cli, goose, kilo-code, metagpt, opencode, qwen-code, void) — coût min **S**
- **`event-store`** — 13 harnais (agentgpt, claude-code, crewai, crush, deepseek-harness, devin, dsh-desktop, gemini-cli, metagpt, open-interpreter, opencode, qwen-code, void) — coût min **S**
- **`agent-store`** — 12 harnais (agentgpt, autogpt, claude-code, crewai, crush, deepseek-harness, gpt-pilot, kilo-code, metagpt, opencode, qwen-code, tabby) — coût min **S**
- **`exec-policy`** — 12 harnais (aider, autogpt, continue, crewai, crush, deepseek-harness, dsh-desktop, gemini-cli, gpt-pilot, openclaw, rea, void) — coût min **S**
- **`memory-store`** — 12 harnais (claude-code, codex, crewai, devin, gemini-cli, gpt-engineer, gptme, hermes, kilo-code, langgraph, openclaw, windsurf) — coût min **S**
- **`sandbox-backend`** — 11 harnais (claude-code, devin, dsh-desktop, gemini-cli, gptme, hermes, openhands, plandex, rea, void, windsurf) — coût min **M**
- **`(execution)`** — 10 harnais (autogpt, continue, crewai, crush, deepseek-harness, gemini-cli, kilo-code, langgraph, opencode, qwen-code) — coût min **S**
- **`telemetry-sink`** — 9 harnais (aider, crewai, devin, dsh-desktop, gpt-pilot, metagpt, plandex, tabby, void) — coût min **S**
- **`(ui)`** — 9 harnais (agentgpt, autogpt, continue, crush, deepseek-harness, gemini-cli, gpt-engineer, gpt-pilot, tabby) — coût min **M**
- **`scheduler-job`** — 9 harnais (aider, claude-code, cline, devin, gptme, hermes, kilo-code, openclaw, openhands) — coût min **M**
- **`permission-policy`** — 8 harnais (autogpt, deepseek-harness, devin, dsh-desktop, gemini-cli, goose, metagpt, tabby) — coût min **S**
- **`skill-store`** — 8 harnais (continue, devin, goose, hermes, metagpt, open-interpreter, rea, windsurf) — coût min **S**
- **`tool`** — 7 harnais (cline, deepseek-harness, gptme, hermes, openhands, plandex, rea) — coût min **S**
- **`(automatisation)`** — 7 harnais (autogpt, continue, deepseek-harness, goose, langgraph, qwen-code, tabby) — coût min **M**
- **`llm-provider`** — 6 harnais (continue, crewai, opencode, openhands, tabby, void) — coût min **S**
- **`router-policy`** — 5 harnais (autogpt, cursor, open-interpreter, plandex, windsurf) — coût min **S**
- **`hook-bus`** — 4 harnais (cursor, gptme, openclaw, windsurf) — coût min **S**
- **`task-classifier`** — 4 harnais (gemini-cli, metagpt, plandex, qwen-code) — coût min **S**
- **`agent-loop`** — 4 harnais (gpt-engineer, gptme, langgraph, plandex) — coût min **S**
- **`hook`** — 4 harnais (cline, codex, deepseek-harness, gemini-cli) — coût min **M**
- **`channel`** — 4 harnais (cline, gemini-cli, langgraph, openclaw) — coût min **M**
- **`agent-server`** — 4 harnais (goose, langgraph, openhands, zed) — coût min **M**

## Quick wins (coût S, le plus réclamé)

- `context-engine` — 28 harnais (AgentGPT, AgentGPT, AgentGPT, Aider)
- `subagent-runner` — 21 harnais (Aider, AutoGPT, AutoGPT, Claude Code)
- `tool-surface` — 20 harnais (AgentGPT, AgentGPT, AgentGPT, AgentGPT)
- `config-source` — 19 harnais (AgentGPT, AgentGPT, AgentGPT, AgentGPT)
- `registry` — 15 harnais (Aider, AutoGPT, AutoGPT, AutoGPT)
- `event-store` — 13 harnais (AgentGPT, AgentGPT, Claude Code, CrewAI)
- `agent-store` — 12 harnais (AgentGPT, AgentGPT, AutoGPT, Claude Code)
- `exec-policy` — 12 harnais (Aider, AutoGPT, Continue, Continue)
- `memory-store` — 12 harnais (Claude Code, Codex CLI, CrewAI, CrewAI)
- `(execution)` — 10 harnais (AutoGPT, AutoGPT, AutoGPT, Continue)
- `telemetry-sink` — 9 harnais (Aider, CrewAI, Devin, DSH Desktop)
- `permission-policy` — 8 harnais (AutoGPT, DeepSeek Harness, Devin, DSH Desktop)
- `skill-store` — 8 harnais (Continue, Devin, Goose, Hermes Agent)
- `tool` — 7 harnais (Cline, DeepSeek Harness, gptme, Hermes Agent)
- `llm-provider` — 6 harnais (Continue, CrewAI, OpenCode, OpenHands)

## Écarté (et pourquoi c'est assumé)

- `extension` — 5 fonctionnalité(s) écartée(s)
- `configuration` — 3 fonctionnalité(s) écartée(s)
- `routage` — 3 fonctionnalité(s) écartée(s)
- `observabilite` — 2 fonctionnalité(s) écartée(s)
- `outils` — 2 fonctionnalité(s) écartée(s)
- `ui` — 2 fonctionnalité(s) écartée(s)
- `securite` — 1 fonctionnalité(s) écartée(s)
- `sandbox-backend` — 1 fonctionnalité(s) écartée(s)

---

_Régénérer : `python3 omega/scripts/integration_plan.py`_
