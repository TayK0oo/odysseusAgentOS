# PLAN-INTEGRATION — ce que les harnais convergent à demander

> **Généré** par `omega/scripts/integration_plan.py` (déterministe, zéro LLM) à partir
> des dissections embarquées : **16 applications**, **269 fonctionnalités**.
> **Lecture :** une *capacité* réclamée par plusieurs harnais est un signal
> structurel — c'est elle qu'il faut implémenter en premier (port + plugins).

| Chiffre | Valeur |
|---|---|
| Applications disséquées | 16 |
| Fonctionnalités | 269 |
| Adoptées (déjà dans Omega) | 59 |
| Planifiées (à faire) | 202 |
| Écartées (assumées) | 8 |

## Priorités par consensus

| # | Capacité visée | Harnais | Coût min | Exemples (harnais → adaptateur) |
|---|---|---|---|---|
| 1 | `context-engine` | 13 (aider, claude-code, cline, codex…) | S | Aider → `plugins/repomap-pagerank` ; Aider → `plugins/prompt-cache` ; Aider → `omega-context::truncate_compact (partiel)` |
| 2 | `subagent-runner` | 13 (aider, claude-code, cline, codex…) | S | Aider → `plugins/architect-loop` ; Claude Code → `plugins/subagent-isolated (spawn/resume, isolation worktree, modèle/effort/TTL par invocation)` ; Claude Code → `plugins/subagent-isolated (background + notify) + plugins/agent-team` |
| 3 | `config-source` | 9 (aider, claude-code, codex, cursor…) | S | Aider → `omega-config (couches + precedence)` ; Claude Code → `omega-config (couches + precedence + policy managée)` ; Codex CLI → `omega-config` |
| 4 | `scheduler-job` | 8 (aider, claude-code, cline, devin…) | M | Aider → `plugins/file-watcher` ; Claude Code → `plugins/scheduler-cron (local + cloud-runner + skill verify/simplify pre-commit)` ; Cline → `plugins/scheduler-cron` |
| 5 | `memory-store` | 7 (claude-code, codex, devin, gptme…) | S | Claude Code → `plugins/memory-bounded (CLAUDE.md layers + MEMORY.md 25KB + audit)` ; Codex CLI → `plugins/memory-local` ; Devin → `plugins/memory-bounded` |
| 6 | `tool-surface` | 7 (aider, claude-code, cursor, devin…) | M | Aider → `plugins/slash-commands` ; Claude Code → `plugins/mcp-client (stdio/http/sse/ws + trust + elicitation)` ; Cursor → `plugins/agent-tools` |
| 7 | `sandbox-backend` | 7 (claude-code, devin, gptme, hermes…) | M | Claude Code → `plugins/sandbox-worktree (git worktree + vérification cwd/commande + cleanup)` ; Devin → `plugins/sandbox-docker` ; gptme → `plugins/sandbox-docker-wasmtime` |
| 8 | `registry` | 5 (aider, claude-code, cline, goose…) | S | Aider → `omega-registry (slots modeles)` ; Claude Code → `omega-registry (manifeste + marketplace resolver + scopes + namespacing)` ; Cline → `omega-registry` |
| 9 | `tool` | 5 (cline, gptme, hermes, openhands…) | S | Cline → `plugins/tool-sdk` ; gptme → `plugins/tool-shell` ; Hermes Agent → `plugins/tool-rpc` |
| 10 | `skill-store` | 5 (devin, goose, hermes, open-interpreter…) | S | Devin → `omega-context::SkillIndex` ; Goose → `—` ; Hermes Agent → `plugins/skill-evolve` |
| 11 | `router-policy` | 4 (cursor, open-interpreter, plandex, windsurf) | S | Cursor → `plugins/model-router` ; Open Interpreter → `plugins/harness-router` ; Open Interpreter → `plugins/harness-autoselect` |
| 12 | `hook-bus` | 4 (cursor, gptme, openclaw, windsurf) | S | Cursor → `plugins/hooks` ; gptme → `plugins/hook-bus` ; OpenClaw → `plugins/hook-bus (bus async + budgets + ordre deterministe)` |
| 13 | `event-store` | 4 (claude-code, devin, open-interpreter, opencode) | M | Claude Code → `plugins/hook-bus (matcher + if + décision exit-code/JSON + async)` ; Devin → `plugins/session-persistent` ; Open Interpreter → `plugins/session-rollout` |
| 14 | `telemetry-sink` | 3 (aider, devin, plandex) | S | Aider → `plugins/eval-harness` ; Devin → `omega-otel::StdoutSink` ; Plandex → `omega-otel::StdoutSink` |
| 15 | `agent-server` | 3 (goose, openhands, zed) | M | Goose → `—` ; Goose → `—` ; OpenHands → `plugins/agent-server-rest` |
| 16 | `agent-store` | 2 (claude-code, opencode) | S | Claude Code → `plugins/command-md (template + $args + argument-hint + overrides)` ; Claude Code → `plugins/plan-mode (deny Write/Edit + délégation Plan + rappel)` ; OpenCode → `plugins/agent-declarative (frontmatter YAML + Ruleset)` |
| 17 | `agent-runtime` | 2 (cline, goose) | S | Cline → `plugins/agent-loop` ; Goose → `—` ; Goose → `—` |
| 18 | `exec-policy` | 2 (aider, openclaw) | M | Aider → `plugins/reflect-lint-test` ; OpenClaw → `omega-sandbox::RuleBasedExecPolicy` |
| 19 | `hook` | 2 (cline, codex) | M | Cline → `plugins/hook-runner` ; Codex CLI → `plugins/hook-command` |
| 20 | `mcp-bridge` | 2 (cline, zed) | M | Cline → `plugins/mcp-bridge` ; Zed → `plugins/mcp-bridge` |
| 21 | `channel` | 2 (cline, openclaw) | M | Cline → `plugins/channel-*` ; OpenClaw → `plugins/channel-*` |
| 22 | `agent-runner` | 2 (cursor, zed) | M | Cursor → `plugins/agent-loop` ; Zed → `plugins/agent-loop` |
| 23 | `checkpoint-store` | 2 (cursor, windsurf) | M | Cursor → `plugins/checkpoint-local` ; Windsurf / Cascade → `plugins/checkpoint-store` |
| 24 | `mcp-client` | 2 (cursor, gptme) | M | Cursor → `plugins/mcp-client` ; gptme → `plugins/mcp-client` |
| 25 | `permission-policy` | 2 (devin, goose) | M | Devin → `omega-sandbox::ApprovalQueue` ; Goose → `—` |
| 26 | `extension-host` | 2 (goose, openhands) | M | Goose → `—` ; OpenHands → `plugins/extension-host` |
| 27 | `agent-loop` | 2 (gptme, plandex) | M | gptme → `plugins/agent-loop` ; Plandex → `plugins/agent-tell-build` |
| 28 | `llm-provider` | 2 (opencode, openhands) | M | OpenCode → `omega-providers (catalogue par transforms + scoring communautaire)` ; OpenHands → `plugins/llm-registry` |
| 29 | `edit-apply` | 1 (aider) | S | Aider → `plugins/edit-search-replace` ; Aider → `plugins/edit-wholefile` ; Aider → `plugins/edit-udiff-patch` |
| 30 | `agent-mode` | 1 (aider) | S | Aider → `plugins/readonly-modes` |

## Consensus fort (≥ 4 harnais réclament la même capacité)

- **`context-engine`** — 13 harnais (aider, claude-code, cline, codex, goose, gptme, hermes, openclaw, opencode, openhands, plandex, windsurf, zed) — coût min **S**
- **`subagent-runner`** — 13 harnais (aider, claude-code, cline, codex, cursor, devin, gptme, hermes, open-interpreter, openclaw, opencode, openhands, zed) — coût min **S**
- **`config-source`** — 9 harnais (aider, claude-code, codex, cursor, hermes, opencode, plandex, windsurf, zed) — coût min **S**
- **`scheduler-job`** — 8 harnais (aider, claude-code, cline, devin, gptme, hermes, openclaw, openhands) — coût min **M**
- **`memory-store`** — 7 harnais (claude-code, codex, devin, gptme, hermes, openclaw, windsurf) — coût min **S**
- **`tool-surface`** — 7 harnais (aider, claude-code, cursor, devin, openclaw, openhands, windsurf) — coût min **M**
- **`sandbox-backend`** — 7 harnais (claude-code, devin, gptme, hermes, openhands, plandex, windsurf) — coût min **M**
- **`registry`** — 5 harnais (aider, claude-code, cline, goose, opencode) — coût min **S**
- **`tool`** — 5 harnais (cline, gptme, hermes, openhands, plandex) — coût min **S**
- **`skill-store`** — 5 harnais (devin, goose, hermes, open-interpreter, windsurf) — coût min **S**
- **`router-policy`** — 4 harnais (cursor, open-interpreter, plandex, windsurf) — coût min **S**
- **`hook-bus`** — 4 harnais (cursor, gptme, openclaw, windsurf) — coût min **S**
- **`event-store`** — 4 harnais (claude-code, devin, open-interpreter, opencode) — coût min **M**

## Quick wins (coût S, le plus réclamé)

- `context-engine` — 13 harnais (Aider, Aider, Aider, Claude Code)
- `subagent-runner` — 13 harnais (Aider, Claude Code, Claude Code, Claude Code)
- `config-source` — 9 harnais (Aider, Claude Code, Codex CLI, Cursor)
- `memory-store` — 7 harnais (Claude Code, Codex CLI, Devin, gptme)
- `registry` — 5 harnais (Aider, Claude Code, Cline, Goose)
- `tool` — 5 harnais (Cline, gptme, Hermes Agent, OpenHands)
- `skill-store` — 5 harnais (Devin, Goose, Hermes Agent, Open Interpreter)
- `router-policy` — 4 harnais (Cursor, Open Interpreter, Open Interpreter, Plandex)
- `hook-bus` — 4 harnais (Cursor, gptme, OpenClaw, Windsurf / Cascade)
- `telemetry-sink` — 3 harnais (Aider, Devin, Plandex)
- `agent-store` — 2 harnais (Claude Code, Claude Code, OpenCode, OpenCode)
- `agent-runtime` — 2 harnais (Cline, Goose, Goose, Goose)
- `edit-apply` — 1 harnais (Aider, Aider, Aider)
- `agent-mode` — 1 harnais (Aider)
- `vcs-commit` — 1 harnais (Aider, Aider)

## Écarté (et pourquoi c'est assumé)

- `configuration` — 2 fonctionnalité(s) écartée(s)
- `extension` — 1 fonctionnalité(s) écartée(s)
- `observabilite` — 1 fonctionnalité(s) écartée(s)
- `ui` — 1 fonctionnalité(s) écartée(s)
- `sandbox-backend` — 1 fonctionnalité(s) écartée(s)
- `outils` — 1 fonctionnalité(s) écartée(s)
- `routage` — 1 fonctionnalité(s) écartée(s)

---

_Régénérer : `python3 omega/scripts/integration_plan.py`_
