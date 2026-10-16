# Dissection — Claude Code (Anthropic, propriétaire)

> Dissection **par documentation publique** (aucun clone, aucun code copié, contenu du system prompt fuité non reproduit).
> Référence d'analyse : `docs/omega/reverse-eng/RE-OPENCODE-CLAUDECODE.md` (§3, §5–§6).
> Docs officielles vérifiées le 2026-10-16 (`code.claude.com/docs`, `platform.claude.com/docs`) ; corpus local `docs/omega/research/loucash/`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Anthropic |
| Licence | Propriétaire (RE par docs uniquement) |
| Langage | TypeScript |
| Repo | — (pas de dépôt public ; docs : `https://code.claude.com/docs`) |
| Architecture | CLI agentique : boucle d'agent sur fichiers déclaratifs (CLAUDE.md, skills, agents, hooks, MCP) ; politiques de contexte matures (compaction structurée, prompt caching par préfixe, tool search, progressive disclosure) ; télémétrie OpenTelemetry complète. |

## 2. Entrées

- **Binaires/surfaces** : `claude` CLI (interactif, headless `-p`, `--agent`), extensions VS Code / JetBrains, Desktop app (sessions locales, Cowork, tâches planifiées), cloud sessions (`claude.ai/code`), Agent SDK (TS/Python).
- **Points d'extension** : skills (`SKILL.md` + frontmatter), sous-agents (`*.md` + frontmatter), hooks (`settings.json` / `hooks/hooks.json` / frontmatter, 5 types de handlers), serveurs MCP (`.mcp.json` / `mcpServers` inline), plugins (dossier + `marketplace.json`), routines planifiées.
- **Config** : couches `managed > cli > env > user > project > local` (`docs/en/settings`) ; policy managée (`allowManagedHooksOnly`, allowlists MCP, `availableModels`).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (doc/ancre) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Compaction structurée + rechargement ciblé | contexte | Résumé structuré remplaçant le verbatim + réhydratation ciblée (≤ 5 fichiers, skills ≤ 5 000 tok), garde anti-thrash | `docs/en/context-window`, `docs/en/how-claude-code-works` | context-engine → `ContextBudget` | planned | M |
| Prompt caching par préfixe | contexte | ≤ 4 breakpoints tools→system→messages, TTL 5m/1h, métriques cache | `platform…/prompt-caching`, `docs/en/prompt-caching` | context-engine → `PromptCachePolicy` | planned | M |
| Tool search MCP (schémas différés) | outils | Index noms+descriptions, hydratation via ToolSearch, alwaysLoad | `docs/en/mcp` (§ scale-with-mcp-tool-search) | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | adopted | M |
| Skills à progressive disclosure | contexte | Standard Agent Skills, descriptions ≤ 1 536 car., corps à l'usage, globs/hook/tools scopés | `docs/en/skills` | skill-store → `SkillStore` → `omega-context::SkillIndex` | adopted | S |
| Commandes slash fusionnées | extension | `/name` déclaratives + `$args`, hints, overrides, précédence | `docs/en/skills`, `docs/en/commands` | agent-store → `AgentStore` | planned | S |
| Sous-agents isolés à frontmatter riche | agents | Contexte isolé, résumé remonté, model/effort/mémoire/worktree, built-ins Explore/Plan | `docs/en/sub-agents` | subagent-runner → `SubagentRunner` | planned | L |
| Arrière-plan + équipes d'agents | exécution | Background + notify, agent teams, messaging inter-sessions | `docs/en/agent-view` | subagent-runner → `SubagentRunner` | planned | M |
| Plugins + marketplace + scopes | extension | Dossier-unité versionné, catalogue, user/project/local, namespacing, coût mesuré | `docs/en/plugins/overview`, `docs/en/plugins/components` | registry → `Registre` | planned | M |
| Hooks de cycle de vie | automatisation | ~30 événements, 5 handlers, matcher+if, décisions exit-code/JSON | `docs/en/hooks` | event-store → `EventStore` (+ dispatch) | planned | M |
| Permissions + modes | sécurité | allow/ask/deny par outil, 6 modes (plan, auto), trust workspace | `docs/en/permissions`, `docs/en/permission-modes` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | adopted | S |
| Export OpenTelemetry | observabilité | Métriques/logs/traces beta, spans `gen_ai.*`, gates, destination verrouillée | `docs/en/monitoring-usage` | telemetry-sink → `TelemetrySink` → `omega-otel::{Stdout,File}Sink` + `Provenance` | adopted | M |
| Mémoire hiérarchique | mémoire | CLAUDE.md managé→local + auto-memory bornée + audit vers skills | `docs/en/memory` | memory-store → `MemoryStore` | planned | S |
| Isolation worktree + garde-fous | exécution | Worktrees isolés, vérif cwd/commandes, cleanup auto | `docs/en/worktrees`, `docs/en/sub-agents` | sandbox-backend → `SandboxBackend` | planned | L |
| Tâches planifiées + routines | automatisation | Cron local vs routines cloud, verify/simplify pré-commit | `docs/en/routines`, `docs/en/scheduled-tasks` | scheduler-job → `Scheduler` | planned | M |
| Plan mode | agents | Session read-only + recherche déléguée au sous-agent Plan | `docs/en/permission-modes` | agent-store → `AgentStore` | planned | S |
| Client MCP multi-transport | outils | stdio/http/sse/ws, serveurs scopés, elicitation, contrôles managed | `docs/en/mcp` | tool-surface → `ToolSurface` | planned | M |
| Settings en couches + policy | configuration | Fusion managed→local, secrets interdits en repo, runs contraints | `docs/en/settings`, `docs/en/settings-reference` | config-source → `ConfigSource` → `omega-config` | planned | S |
| Headless + SDK + CI | exécution | `-p`, Agent SDK, GitHub Actions/Setup, bare/safe mode | `docs/en/headless`, `docs/en/github-actions` | subagent-runner → `SubagentRunner` → `omega-server` | planned | M |
| Mods JS in-process (écarté) | extension | Runtime JS embarqué hors périmètre Rust | `docs/en/hooks` (mods) | — | rejected | L |

## 4. Traduction en plugins

- `cc-tool-search` : tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` (plugin, adopted).
- `cc-skills` : skill-store → `SkillStore` → `omega-context::SkillIndex` (plugin, adopted).
- `cc-permissions` : permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` (plugin, adopted).
- `cc-otel` : telemetry-sink → `TelemetrySink` → `omega-otel::{Stdout,File}Sink` + `Provenance` (plugin, adopted).
- `cc-compaction` / `cc-prompt-cache` : context-engine → `ContextBudget` / `PromptCachePolicy` → `omega-context` (plugin, planned).
- `cc-subagents` / `cc-background` / `cc-ci-headless` : subagent-runner → `SubagentRunner` → `plugins/subagent-isolated`, `plugins/agent-team`, `omega-server` (plugin, planned).
- `cc-plugins` : registry → `Registre` → `omega-registry` + résolveur marketplace (plugin, planned).
- `cc-hooks` : event-store → `EventStore` → `plugins/hook-bus` (plugin, planned).
- `cc-commands` / `cc-plan-mode` : agent-store → `AgentStore` → `plugins/command-md`, `plugins/plan-mode` (plugin, planned).
- `cc-memory` : memory-store → `MemoryStore` → `plugins/memory-bounded` (plugin, planned).
- `cc-worktrees` : sandbox-backend → `SandboxBackend` → `plugins/sandbox-worktree` (plugin, planned).
- `cc-routines` : scheduler-job → `Scheduler` → `plugins/scheduler-cron` (plugin, planned).
- `cc-mcp` : tool-surface → `ToolSurface` → `plugins/mcp-client` (plugin, planned).
- `cc-settings` : config-source → `ConfigSource` → `omega-config` (plugin, planned).

## 5. Écarté

- **`cc-mods-js` (rejected)** : hooks JavaScript in-process avec accès UI ; runtime JS embarqué hors périmètre du cœur Rust (même décision que `oc-codemode-lsp` pour OpenCode). Les 4 autres types de hooks (command/http/mcp_tool/prompt/agent) couvrent le besoin ; remplacement prévu : WASM sandboxé si besoin.

## 6. Sources

- Docs officielles (vérifiées 200 le 2026-10-16) : `sub-agents`, `skills`, `commands`, `hooks`, `plugins/overview`, `plugins/components`, `monitoring-usage`, `mcp`, `context-window`, `how-claude-code-works`, `prompt-caching` (×2), `memory`, `permissions`, `permission-modes`, `worktrees`, `routines`, `scheduled-tasks`, `agent-view`, `settings`, `settings-reference`, `headless`, `github-actions` (toutes sous `https://code.claude.com/docs/en/`, sauf prompt-caching originel sous `https://platform.claude.com/docs/en/build-with-claude/`).
- Pages inexistantes constatées (404, non citées) : `docs/en/checkpoints`, `docs/en/rewind`, `docs/en/sandbox`.
- Fiche locale : `docs/omega/reverse-eng/RE-OPENCODE-CLAUDECODE.md` (§3.1–§3.9, §5 F1–F20, §6.3).
- Corpus local (`docs/omega/research/loucash/videos.json`, observé) : `7659512794085166358` (vidéo #30, fuite du system prompt — non reproduite), `7664901404997192982` (5 skills favoris), `7686601511652150560` (un plugin = N skills).
