# Dissection — Devin (Cognition)

> Dissection par **documentation** (logiciel proprietaire, code ferme). Tout fait ci-dessous est [documente] sauf mention [infere]. Sources : docs.devin.ai (llms.txt + pages), repo ouvert devin-handoff.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Cognition |
| Licence | Proprietaire |
| Langage | (ferme) |
| Repo | Code ferme ; plugin ouvert : https://github.com/club-cog/devin-handoff |
| Architecture | Sessions cloud longue duree sur VM dediees bootees d'un snapshot d'environnement (blueprint -> build -> snapshot) ; outils Shell/IDE/Browser prenables en main ; orchestration d'enfants paralleles ; pilotage webapp/Slack/Teams/API/CLI/MCP. |
| Docs | https://docs.devin.ai (index : https://docs.devin.ai/llms.txt) |

## 2. Entrees

| Entree | Nature |
|---|---|
| webapp `app.devin.ai` (sessions, review, automations, playbooks, environment) | UI web |
| Devin CLI (`devin`, `/handoff`, `/cloud`, `ssh`, `forward`) | binaire local |
| Devin API v1/v2/v3 + Devin MCP server | API/MCP |
| Plugin `devin-handoff` (Claude Code, Codex, Cursor, scripts) | plugin ouvert |
| Integrations Slack/Teams/GitHub/GitLab/Bitbucket/Linear/Jira/Databricks/MongoDB/PagerDuty/Pylon | webhooks/OAuth |
| Automations (triggers Slack/GitHub/Linear/Jira/schedule RRULE/webhook) | event-driven |
| Fichiers : `SKILL.md` (6 chemins), `AGENTS.md`/`REVIEW.md`/`CLAUDE.md`, `.devin/blueprint.yaml`, `.devin/wiki.json` | config repo |

## 3. Inventaire fonctionnel

| Fonctionnalite | Domaine | Ce que ca fait | Ou (doc) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|
| Sessions longues + workspace persistant + takeover | execution | Sessions VM longue duree, pause/reprise, prise en main IDE/shell, side chats read-only | session-tools, blueprints, first-run | event-store → EventStore → plugins/session-persistent | planned | M |
| Sessions managees en parallele | agents | Coordinateur + enfants isoles (VM), messages, reveil auto, suivi ACU | advanced-capabilities | subagent-runner → SubagentRunner → plugins/subagent-isolated | planned | M |
| Dynamic Workflows | automatisation | Script Python deterministe, fan-out/pipeline, replay par hash, reprise 7 j | dynamic-workflows | workflow-engine → WorkflowEngine → plugins/workflow-dag | planned | L |
| Shell/IDE/Browser + Computer Use | outils | Historique copiable, IDE prenable en main, browser interactif, desktop complet | session-tools, computer-use, browser-auth | tool-surface → ToolSurface → plugins/tool-devin-workspace | planned | M |
| Integrations chat/trackers/VCS | extension | Slack/Teams/GitHub/Linear/Jira -> sessions et PR | integrations/* | channel-connector → Channel/Tracker/Vcs → plugins/channel-* | planned | M |
| Devin Review + auto-review + auto-fix | outils | Diffs intelligents, bugs/securite CWE, chat contextuel, stacked PRs, auto-fix | devin-review, stacked-prs | code-review → Reviewer → plugins/review-assistant | planned | L |
| Skills SKILL.md + Knowledge | contexte | Standard Agent Skills, 6 chemins, @skills:, Knowledge migree vers plugins | skills, knowledge-onboarding, plugins | skill-store → SkillStore → omega-context::SkillIndex | planned | S |
| Memory drive + Dreaming | memoire | Depot Git personnel, apprentissage auto, consolidation de fond | memory (+ AgentMemoryRepo) | memory-store → MemoryStore → plugins/memory-bounded | planned | M |
| Playbooks + macros + versions | automatisation | Prompts reutilisables versionnes, generables depuis sessions | creating/using-playbooks | playbook-store → PlaybookStore → plugins/playbook-store | planned | S |
| Automations + preflight + schedules + webhooks | automatisation | Triggers multi-sources, filtre LLM/script, file bornee, budgets ACU | automations, auto-triage, oncall | scheduler-job → Scheduler/Trigger → plugins/scheduler-cron | planned | L |
| API + CLI + /handoff + ACP/SSH + Outposts | execution | Handoff local<->cloud, API/MCP, workers self-hosted | devin-handoff, cli/handoff, outposts | task-handoff → Handoff → plugins/handoff-bridge | planned | M |
| Blueprints -> builds -> snapshots | execution | Envs declaratifs versionnes en snapshots VM, 3 niveaux, monorepos | blueprints, blueprint-reference | sandbox-backend → SandboxBackend → plugins/sandbox-docker | planned | L |
| Security profiles + secrets + OIDC + guardrails | securite | Profils hierarchiques mandatory, secrets, OIDC, RBAC/SSO | security-profiles, secrets, oidc | permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue | planned | M |
| Session Insights + metering ACU + preuves video | observabilite | Timeline, prompt ameliore, tailles XS-XL, recordings | session-insights, billing/usage | telemetry-sink → TelemetrySink → omega-otel::StdoutSink | planned | S |

## 4. Traduction en plugins

- `dv-long-session` : event-store → `EventStore (persist/resume)` → `plugins/session-persistent` (plugin, planned, M).
- `dv-managed-parallel` : subagent-runner → `SubagentRunner` → `plugins/subagent-isolated` (plugin, planned, M).
- `dv-dynamic-workflows` : workflow-engine → `WorkflowEngine` → `plugins/workflow-dag` (plugin, planned, L).
- `dv-session-tools` : tool-surface → `ToolSurface` → `plugins/tool-devin-workspace` (plugin, planned, M).
- `dv-chat-tracker-integrations` : channel-connector → `Channel/Tracker/Vcs` → `plugins/channel-*` (plugin, planned, M).
- `dv-review` : code-review → `Reviewer` → `plugins/review-assistant` (plugin, planned, L).
- `dv-skills-knowledge` : skill-store → `SkillStore` → `omega-context::SkillIndex` (plugin, planned, S).
- `dv-memory-dreaming` : memory-store → `MemoryStore` → `plugins/memory-bounded` (plugin, planned, M).
- `dv-playbooks` : playbook-store → `PlaybookStore` → `plugins/playbook-store` (plugin, planned, S).
- `dv-automations` : scheduler-job → `Scheduler/Trigger` → `plugins/scheduler-cron` (plugin, planned, L).
- `dv-api-cli-handoff` : task-handoff → `Handoff` → `plugins/handoff-bridge` (plugin, planned, M).
- `dv-env-blueprints` : sandbox-backend → `SandboxBackend` → `plugins/sandbox-docker` (plugin, planned, L).
- `dv-security-profiles` : permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` (plugin, planned, M).
- `dv-insights-metering` : telemetry-sink → `TelemetrySink` → `omega-otel::StdoutSink` (plugin, planned, S).

## 5. Ecarte

- Rien d'ecarte : tout est repris en `planned`. Seule reserve : le scan securite/bug catcher de Devin Review repose sur des modeles/moteurs fermes non documentes — l'adaptateur Omega devra brancher nos propres analyseurs (lint, secrets, CWE) plutot que repliquer un moteur inconnu.

## 6. Sources

- https://docs.devin.ai / https://docs.devin.ai/llms.txt
- https://docs.devin.ai/get-started/devin-intro.md, first-run.md
- https://docs.devin.ai/work-with-devin/devin-session-tools.md, advanced-capabilities.md, dynamic-workflows.md, devin-handoff.md, devin-review.md, stacked-prs.md, computer-use.md, browser-auth.md, testing-and-recordings.md, mcp.md, devin-mcp.md
- https://docs.devin.ai/product-guides/memory.md, skills.md, plugins.md, creating-playbooks.md, using-playbooks.md, automations.md, auto-triage.md, devin-oncall.md, scheduled-sessions.md, session-insights.md, secrets.md, security-profiles.md, oidc.md
- https://docs.devin.ai/onboard-devin/knowledge-onboarding.md, agents-md.md, environment.md, environment/blueprints.md, environment/blueprint-reference.md
- https://docs.devin.ai/integrations/overview.md, slack.md, microsoft-teams.md, gh.md, linear.md, jira.md
- https://docs.devin.ai/cli/index.md, cli/handoff.md, cloud/outposts/overview.md
- https://github.com/club-cog/devin-handoff · https://github.com/AgentMemoryRepo/agentmemoryrepo · https://agentskills.io/specification
- Contexte Omega : docs/omega/research/HARNESS-LANDSCAPE.md (§1/§2 Devin), omega/crates/omega-harness/data/apps/hermes.json (exemple de forme).
