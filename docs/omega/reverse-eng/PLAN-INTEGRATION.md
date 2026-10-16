# PLAN-INTEGRATION — ce que les harnais convergent à demander

> **Généré** par `omega/scripts/integration_plan.py` (déterministe, zéro LLM) à partir
> des dissections embarquées : **12 applications**, **203 fonctionnalités**.
> **Lecture :** une *capacité* réclamée par plusieurs harnais est un signal
> structurel — c'est elle qu'il faut implémenter en premier (port + plugins).

| Chiffre | Valeur |
|---|---|
| Applications disséquées | 12 |
| Fonctionnalités | 203 |
| Adoptées (déjà dans Omega) | 55 |
| Planifiées (à faire) | 143 |
| Écartées (assumées) | 5 |

## Priorités par consensus

| # | Capacité visée | Harnais | Coût min | Exemples (harnais → adaptateur) |
|---|---|---|---|---|
| 1 | `context-engine` | 11 (aider, cline, codex, goose…) | S | Aider → `plugins/repomap-pagerank` ; Aider → `plugins/prompt-cache` ; Aider → `omega-context::truncate_compact (partiel)` |
| 2 | `subagent-runner` | 10 (aider, cline, codex, gptme…) | M | Aider → `plugins/architect-loop` ; Cline → `plugins/subagent-isolated` ; Codex CLI → `plugins/subagent-isolated` |
| 3 | `config-source` | 6 (aider, codex, hermes, opencode…) | S | Aider → `omega-config (couches + precedence)` ; Codex CLI → `omega-config` ; Hermes Agent → `omega-config (profils)` |
| 4 | `scheduler-job` | 6 (aider, cline, gptme, hermes…) | M | Aider → `plugins/file-watcher` ; Cline → `plugins/scheduler-cron` ; gptme → `plugins/scheduler-service` |
| 5 | `tool` | 5 (cline, gptme, hermes, openhands…) | S | Cline → `plugins/tool-sdk` ; gptme → `plugins/tool-shell` ; Hermes Agent → `plugins/tool-rpc` |
| 6 | `registry` | 4 (aider, cline, goose, opencode) | S | Aider → `omega-registry (slots modeles)` ; Cline → `omega-registry` ; Goose → `—` |
| 7 | `memory-store` | 4 (codex, gptme, hermes, openclaw) | M | Codex CLI → `plugins/memory-local` ; gptme → `plugins/memory-layered` ; Hermes Agent → `plugins/memory-bounded` |
| 8 | `sandbox-backend` | 4 (gptme, hermes, openhands, plandex) | M | gptme → `plugins/sandbox-docker-wasmtime` ; Hermes Agent → `plugins/sandbox-{docker,ssh,modal,…}` ; OpenHands → `plugins/sandbox-docker` |
| 9 | `skill-store` | 3 (goose, hermes, open-interpreter) | S | Goose → `—` ; Hermes Agent → `plugins/skill-evolve` ; Open Interpreter → `plugins/harness-skill-bridge` |
| 10 | `tool-surface` | 3 (aider, openclaw, openhands) | M | Aider → `plugins/slash-commands` ; OpenClaw → `omega-context::ToolSurfaceIndex` ; OpenHands → `plugins/tool-registry` |
| 11 | `agent-server` | 3 (goose, openhands, zed) | M | Goose → `—` ; Goose → `—` ; OpenHands → `plugins/agent-server-rest` |
| 12 | `telemetry-sink` | 2 (aider, plandex) | S | Aider → `plugins/eval-harness` ; Plandex → `omega-otel::StdoutSink` |
| 13 | `agent-runtime` | 2 (cline, goose) | S | Cline → `plugins/agent-loop` ; Goose → `—` ; Goose → `—` |
| 14 | `router-policy` | 2 (open-interpreter, plandex) | S | Open Interpreter → `plugins/harness-router` ; Open Interpreter → `plugins/harness-autoselect` ; Plandex → `plugins/router-large-ctx` |
| 15 | `exec-policy` | 2 (aider, openclaw) | M | Aider → `plugins/reflect-lint-test` ; OpenClaw → `omega-sandbox::RuleBasedExecPolicy` |
| 16 | `hook` | 2 (cline, codex) | M | Cline → `plugins/hook-runner` ; Codex CLI → `plugins/hook-command` |
| 17 | `mcp-bridge` | 2 (cline, zed) | M | Cline → `plugins/mcp-bridge` ; Zed → `plugins/mcp-bridge` |
| 18 | `channel` | 2 (cline, openclaw) | M | Cline → `plugins/channel-*` ; OpenClaw → `plugins/channel-*` |
| 19 | `extension-host` | 2 (goose, openhands) | M | Goose → `—` ; OpenHands → `plugins/extension-host` |
| 20 | `agent-loop` | 2 (gptme, plandex) | M | gptme → `plugins/agent-loop` ; Plandex → `plugins/agent-tell-build` |
| 21 | `hook-bus` | 2 (gptme, openclaw) | M | gptme → `plugins/hook-bus` ; OpenClaw → `plugins/hook-bus (bus async + budgets + ordre deterministe)` |
| 22 | `event-store` | 2 (open-interpreter, opencode) | M | Open Interpreter → `plugins/session-rollout` ; OpenCode → `omega-app::FileEventStore + omega-otel::TraceReplayer` |
| 23 | `llm-provider` | 2 (opencode, openhands) | M | OpenCode → `omega-providers (catalogue par transforms + scoring communautaire)` ; OpenHands → `plugins/llm-registry` |
| 24 | `edit-apply` | 1 (aider) | S | Aider → `plugins/edit-search-replace` ; Aider → `plugins/edit-wholefile` ; Aider → `plugins/edit-udiff-patch` |
| 25 | `agent-mode` | 1 (aider) | S | Aider → `plugins/readonly-modes` |
| 26 | `vcs-commit` | 1 (aider) | S | Aider → `plugins/git-auto-commit` ; Aider → `plugins/git-auto-commit` |
| 27 | `lint-check` | 1 (aider) | S | Aider → `plugins/lint-after-edit` |
| 28 | `(observabilite)` | 1 (goose) | S | Goose → `—` |
| 29 | `cost-guard` | 1 (gptme) | S | gptme → `plugins/cost-awareness` |
| 30 | `harness-emulation` | 1 (open-interpreter) | S | Open Interpreter → `plugins/harness-claude-code` ; Open Interpreter → `plugins/harness-kimi-code` ; Open Interpreter → `plugins/harness-swe-agent` |

## Consensus fort (≥ 4 harnais réclament la même capacité)

- **`context-engine`** — 11 harnais (aider, cline, codex, goose, gptme, hermes, openclaw, opencode, openhands, plandex, zed) — coût min **S**
- **`subagent-runner`** — 10 harnais (aider, cline, codex, gptme, hermes, open-interpreter, openclaw, opencode, openhands, zed) — coût min **M**
- **`config-source`** — 6 harnais (aider, codex, hermes, opencode, plandex, zed) — coût min **S**
- **`scheduler-job`** — 6 harnais (aider, cline, gptme, hermes, openclaw, openhands) — coût min **M**
- **`tool`** — 5 harnais (cline, gptme, hermes, openhands, plandex) — coût min **S**
- **`registry`** — 4 harnais (aider, cline, goose, opencode) — coût min **S**
- **`memory-store`** — 4 harnais (codex, gptme, hermes, openclaw) — coût min **M**
- **`sandbox-backend`** — 4 harnais (gptme, hermes, openhands, plandex) — coût min **M**

## Quick wins (coût S, le plus réclamé)

- `context-engine` — 11 harnais (Aider, Aider, Aider, Cline)
- `config-source` — 6 harnais (Aider, Codex CLI, Hermes Agent, OpenCode)
- `tool` — 5 harnais (Cline, gptme, Hermes Agent, OpenHands)
- `registry` — 4 harnais (Aider, Cline, Goose, OpenCode)
- `skill-store` — 3 harnais (Goose, Hermes Agent, Open Interpreter)
- `telemetry-sink` — 2 harnais (Aider, Plandex)
- `agent-runtime` — 2 harnais (Cline, Goose, Goose, Goose)
- `router-policy` — 2 harnais (Open Interpreter, Open Interpreter, Plandex)
- `edit-apply` — 1 harnais (Aider, Aider, Aider)
- `agent-mode` — 1 harnais (Aider)
- `vcs-commit` — 1 harnais (Aider, Aider)
- `lint-check` — 1 harnais (Aider)
- `(observabilite)` — 1 harnais (Goose)
- `cost-guard` — 1 harnais (gptme)
- `harness-emulation` — 1 harnais (Open Interpreter, Open Interpreter, Open Interpreter, Open Interpreter)

## Écarté (et pourquoi c'est assumé)

- `configuration` — 2 fonctionnalité(s) écartée(s)
- `ui` — 1 fonctionnalité(s) écartée(s)
- `sandbox-backend` — 1 fonctionnalité(s) écartée(s)
- `outils` — 1 fonctionnalité(s) écartée(s)

---

_Régénérer : `python3 omega/scripts/integration_plan.py`_
