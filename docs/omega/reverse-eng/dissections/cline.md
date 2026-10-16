# Dissection — Cline

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Cline |
| Licence | Apache-2.0 |
| Langage | TypeScript (Bun/Node ≥ 22, workspaces) |
| Repo | https://github.com/cline/cline |
| Architecture | Monorepo SDK-first : runtime d'agent stateless (`sdk/packages/agents`), noyau `@cline/core` (outils, MCP, checkpoints git, cron, equipes), contrats `@cline/shared`, LLM `@cline/llms`, hotes CLI headless/TUI (`apps/cli`), extension VS Code (`apps/vscode`), Hub cloud. |

## 2. Entrees

- **Binaires/CLI** : `apps/cli` — `run` (headless/CI), `chat` (TUI), `schedule`, `connect`/`connect-via-hub`, `hook`, `mcp`, `skill`, `plugin`, `kanban`, `history`, `doctor` (`apps/cli/src/commands/program.ts`, `main.ts`).
- **Points d'extension** : `createTool` (`@cline/shared`), `Agent` (`@cline/sdk`), plugins (`plugin-loader.ts`), hooks (10 evenements), connecteurs (6 plateformes), skills `SKILL.md`.
- **Fichiers de config** : `.clinerules`, `AGENTS.md`, repertoires `rules/`/`skills/`/`workflows`, frontmatter YAML, `apps/cli/skills-lock.json`.

## 3. Inventaire fonctionnel

| Fonctionnalite | Domaine | Ce que ca fait | Ou (file:line) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|
| Boucle agent avec recuperation bornee | execution | Tours modele→outils, retries backoff, relance concision, echec explicite overflow | `sdk/packages/agents/src/agent-runtime.ts:58-115` | AgentRunner → `plugins/agent-loop` | planned | M |
| Boite a outils par defaut (9 outils) | outils | Shell, fichiers, recherche, web, patch, editeur, skills, dialogue | `sdk/packages/core/src/extensions/tools/definitions.ts:915` | ToolSurface → `omega-context::ToolSurfaceIndex` | adopted | M |
| SDK createTool | extension | Fabrique d'outils typee, validation stricte, timeout/retries/lifecycle | `sdk/packages/shared/src/tools/create.ts:108` | Tool → `plugins/tool-sdk` | planned | S |
| Hooks de cycle de vie (10 evenements) | extension | Interception start/tool/prompt/compact/shutdown via fichiers ou sous-processus | `sdk/packages/shared/src/hooks/events.ts:58` | HookRunner → `plugins/hook-runner` | planned | M |
| Modes Plan/Act/Yolo + garde | execution | Plan bloque les mutations shell avec consigne de plan approuvable | `sdk/packages/core/src/extensions/tools/command-guard.ts:514` | ExecPolicy → `omega-sandbox::RuleBasedExecPolicy` | adopted | S |
| Approbation par outil | securite | Politiques par outil, --yolo normalise, garde allow/deny par acteur | `sdk/packages/shared/src/agent.ts:589` | PermissionPolicy → `omega-sandbox::ApprovalQueue` | adopted | S |
| Pont MCP | outils | Manager en memoire, cache TTL, OAuth, proxy, politiques | `sdk/packages/core/src/extensions/mcp/manager.ts:43` | McpBridge → `plugins/mcp-bridge` | planned | M |
| Regles + skills | contexte | `.clinerules`/`AGENTS.md` et `SKILL.md` decouvrables, outil `skills` | `sdk/packages/core/src/extensions/tools/definitions.ts:761` | SkillStore → `omega-context::SkillIndex` | adopted | S |
| Checkpoints git | memoire | Stash auto avant mutation, diff, restauration, nettoyage | `sdk/packages/core/src/hooks/checkpoint-hooks.ts:11` | EventStore → `omega-app::FileEventStore` | adopted | M |
| Equipes multi-agents | agents | Spawn coordonne d'agents persistants (heartbeat, reprise) | `sdk/packages/core/src/session/team/team-session-coordinator.ts:15` | SubagentRunner → `plugins/subagent-isolated` | planned | L |
| Passerelle multi-fournisseurs | routage | Gateway, erreurs retryables, routage par modele, catalogue | `sdk/packages/llms/src/providers.ts:1` | Provider → `omega-providers::OllamaProvider` | adopted | M |
| Compaction basique + agentique | contexte | Troncature ou resume, declenchee sur compteur reel, hook pre_compact | `sdk/packages/core/src/extensions/context/compaction.ts:1` | ContextEngine → `omega-context::truncate_compact` | planned | M |
| Budget de contexte + cache oversized | contexte | Pagination via `cline://cache`, LRU 16 MiB, consignes d'overflow | `sdk/packages/agents/src/agent-runtime.ts:100` | ContextEngine+Budget → `omega-context::ContextBudget` | adopted | S |
| Runs planifies + agenda | automatisation | Cron SQLite + taches agenda reconciliables + CLI `schedule` | `sdk/packages/core/src/cron/runner/cron-runner.ts:194` | Scheduler → `plugins/scheduler-cron` | planned | L |
| Connecteurs messagerie (6) | execution | Telegram/Slack/Discord/WhatsApp/gchat/Linear → sessions RPC | `sdk/packages/shared/src/connectors/platforms.ts:6` | Channel → `plugins/channel-*` | planned | L |
| CLI headless/CI + TUI | execution | Run non-interactif partageant le runtime du TUI | `apps/cli/src/runtime/run-agent.ts:1` | HeadlessRun → `plugins/headless-cli` | planned | M |
| Registre de plugins + marketplace | extension | Chargement avec diagnostics, sandbox, marketplace | `sdk/packages/core/src/extensions/plugin/plugin-loader.ts:107` | Registre → `omega-registry` | planned | M |
| Hub cloud SaaS | configuration | Auth, sessions distantes, dashboard operateur | `sdk/packages/core/src/cloud/controller.ts:1` | — | rejected | L |

## 4. Traduction en plugins

- `tool-surface → ToolSurface → omega-context::ToolSurfaceIndex` (plugin, **adopted**) : les 9 outils natifs Cline deviennent des contributions au catalogue ; le `resultPolicy: cache-oversized` inspire la pagination du budget.
- `skill-store → SkillStore → omega-context::SkillIndex` (plugin, **adopted**) : `.clinerules`/`AGENTS.md`/`SKILL.md` + outil `skills` = divulgation progressive deja en place.
- `exec-policy → ExecPolicy → omega-sandbox::RuleBasedExecPolicy` (plugin, **adopted**) : Plan/Act/Yolo comme politiques declaratives ; la blacklist plan-mode est une regle de politique.
- `permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue` (plugin, **adopted**) : `ToolPolicy` par outil + garde acteur des connecteurs.
- `llm-provider → Provider → omega-providers::OllamaProvider` (plugin, **adopted**) : la gateway Cline (erreurs retryables, routage par modele) derriere le port Provider.
- `context-engine → ContextEngine+ContextBudget → omega-context::ContextBudget` (plugin, **adopted**) : token-awareness + cache oversized.
- `event-store → EventStore → omega-app::FileEventStore` (plugin, **adopted**) : checkpoints git = snapshots persistants + restauration.
- `agent-runtime → AgentRunner → plugins/agent-loop` (plugin, planned) : boucle bornee avec garde-fous anti-divergence.
- `tool → Tool → plugins/tool-sdk` (plugin, planned) : `createTool` comme API auteur (zod→schema strict, lifecycle, executionMode).
- `hook → HookRunner → plugins/hook-runner` (plugin, planned) : les 10 evenements + `HookControl` (cancel/review/override).
- `mcp-bridge → McpBridge → plugins/mcp-bridge` (plugin, planned) : manager + cache TTL + politiques.
- `subagent-runner → SubagentRunner → plugins/subagent-isolated` (plugin, planned) : coordinateur d'equipe + spawn tools.
- `scheduler-job → Scheduler → plugins/scheduler-cron` (plugin, planned) : cron + agenda + reconciliateur fichier.
- `channel → Channel → plugins/channel-*` (plugin, planned) : 6 connecteurs, un adaptateur par plateforme.
- `headless-run → HeadlessRun → plugins/headless-cli` (plugin, planned) : run CI non-interactif.
- `context-engine → ContextEngine → omega-context::truncate_compact` (plugin, planned) : compaction agentique au-dela de la troncature.
- `registry → Registre → omega-registry` (plugin, planned) : plugin-loader + sandbox en regime Slot/Ensemble.

## 5. Ecarte

- **Cline Hub cloud (rejected)** : backend SaaS operateur (auth WorkOS, sessions distantes, feature flags distants, facturation) — dependance operateur incompatible avec l'exigence self-hosted d'Omega. Les briques client (controller, reconciliation de transcript) restent une source d'inspiration pour un `hub` auto-heberge, mais le service lui-meme n'est pas repris.

## 6. Sources

- https://github.com/cline/cline (clone depth-1, `/home/tayk0oo/omega-re/cline`)
- `sdk/packages/agents/src/agent-runtime.ts`, `sdk/packages/agents/src/index.ts`
- `sdk/packages/shared/src/tools/create.ts`, `sdk/packages/shared/src/agent.ts`, `sdk/packages/shared/src/hooks/events.ts`, `sdk/packages/shared/src/hooks/contracts.ts`
- `sdk/packages/shared/src/connectors/platforms.ts`, `sdk/packages/shared/src/storage/paths.ts`, `sdk/packages/shared/src/tasks.ts`, `sdk/packages/shared/src/mcp.ts`
- `sdk/packages/core/src/extensions/tools/definitions.ts`, `command-guard.ts`, `sdk/packages/core/src/extensions/mcp/manager.ts`
- `sdk/packages/core/src/hooks/checkpoint-hooks.ts`, `sdk/packages/core/src/session/checkpoint-restore.ts`, `checkpoint-diff.ts`
- `sdk/packages/core/src/session/team/team-session-coordinator.ts`, `sdk/packages/core/src/extensions/tools/team/spawn-agent-tool.ts`
- `sdk/packages/core/src/cron/schedule/scheduler.ts`, `sdk/packages/core/src/cron/runner/cron-runner.ts`, `sdk/packages/core/src/tasks/agenda-task-manager.ts`
- `sdk/packages/core/src/extensions/context/compaction.ts`, `sdk/packages/core/src/extensions/plugin/plugin-loader.ts`
- `apps/cli/src/runtime/run-agent.ts`, `apps/cli/src/commands/program.ts`, `apps/cli/src/connectors/`
