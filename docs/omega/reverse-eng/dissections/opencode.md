# Dissection — OpenCode (`sst/opencode`)

> Clone : `/home/tayk0oo/omega-re/opencode`, commit `eeec76b`, version `2.0.30` (2026-10-16).
> Référence d'analyse : `docs/omega/reverse-eng/RE-OPENCODE-CLAUDECODE.md` (V2 `2.0.29`, commit `3943b84` — écarts mineurs constatés : version bump, mêmes contrats).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | sst |
| Licence | MIT (Copyright (c) 2025 opencode, `LICENSE`) |
| Langage | TypeScript / Bun (monorepo Bun + Turbo, `packages/*`, ~4 263 fichiers `.ts`) |
| Repo | https://github.com/sst/opencode |
| Architecture | Contrats `effect/Schema` (`packages/schema`) / runtime (`packages/core`) / transport HTTP (`packages/protocol` + `packages/server`) ; SDK plugin (`packages/plugin`, API Promise + Effect) ; noyau lui-même composé de plugins internes via les mêmes `transform`/`hook` que les tiers. |

## 2. Entrées

- **Binaires/CLI** : `packages/cli` (`opencode` CLI), `packages/server` (HTTP/OpenAPI), `packages/tui`, `packages/desktop`, `packages/app` (web).
- **Points d'extension** : `transform` (état rejouable, domaines `agent|command|integration|mcp|model|provider|reference|skill|tool|vcs|websearch`) + `hook` (opération live : `session.*`, `tool.execute.before/after`, `permission`, `event`, `storage`, `rpc`, `shell`, `worktree`, `plugin`) + `reload()` (`packages/plugin/src/README.md:5-76`).
- **Config** : `plugins: (string|{package,options})[]` (`packages/schema/src/config/plugin.ts:6-14`) ; discovery `.claude/.agents/.opencode` (`packages/core/src/config/discovery.ts:69-141`).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Registre de plugins (manifeste + capacités + supervision) | extension | Manifeste versionné + capacités server/tui/rpc + état failed non fatal, updatable via API | `packages/schema/src/plugin.ts:10-43`, `packages/plugin/src/host.ts:17-48`, `packages/protocol/src/groups/plugin.ts:10-49` | registry → `Registre` → `omega-registry::Registry` | adopted | M |
| Deux verbes : hook + transform + reload | extension | Surface d'API minimale : interception live vs contribution rejouable | `packages/plugin/src/README.md:5-76`, `packages/plugin/src/promise/registration.ts:1-21` | registry → `Registre` → `omega-plugin-api` | planned | M |
| Inscriptions scopées (overlay) | extension | Dernière inscription gagne ; fermer révèle la précédente | `specs/v2/tools.md:49-81`, `packages/core/src/plugin/host.ts` | registry → `Registre` (Scope) | planned | M |
| Fingerprint + cache d'échec + watch | extension | Découverte idempotente, reload sur changement | `packages/plugin/src/source.ts:6-66` | registry → `Registre` | planned | S |
| Agents déclaratifs | agents | Markdown + frontmatter + mode + permissions, compat `~/.claude/` | `packages/schema/src/agent.ts:23-53`, `packages/core/src/plugin/agent.ts:87-119` | agent-store → `AgentStore` | planned | S |
| Commandes markdown | agents | Template exécutant prompt/shell/subagent | `packages/schema/src/command.ts:1-18` | agent-store → `AgentStore` | planned | S |
| Skills : store + discovery + injection | contexte | directory/url/embedded, index.json, gardes anti-traversée, bloc `<skill_content>` | `packages/schema/src/skill.ts:27-66`, `packages/core/src/skill/discovery.ts:16-124` | skill-store → `SkillStore` → `omega-context::SkillIndex` | adopted | M |
| Sous-agents isolés parallèles | agents | Sessions enfants, background, reprise, modèle par invocation | `packages/core/src/tool/plugin/subagent.ts:19-90` | subagent-runner → `SubagentRunner` | planned | L |
| Plan mode | agents | Agent read-only + rappel replacé (préserve cache) | `packages/core/src/plugin/plan.ts:12-60` | agent-store → `AgentStore` | planned | S |
| Catalogue multi-fournisseurs rejouable | routage | Transforms config/models.dev/auth, activation, coûts cache | `packages/schema/src/provider.ts:12-60`, `packages/core/src/plugin/models-dev.ts:28-60` | llm-provider → `Provider` | planned | M |
| Prompt caching par préfixe | contexte | Breakpoints ≤ 4, affinité parent/fork/self | `packages/ai/src/cache-policy.ts:17-60`, `packages/core/src/session/affinity.ts:7-8` | context-engine → `PromptCachePolicy` | planned | M |
| Compaction structurée | contexte | Résumé typé + shrink steps + keep 15k, variante native | `packages/core/src/session/compaction.ts:95-233` | context-engine → `ContextBudget` | planned | M |
| Contrat d'outil triple | outils | output machine / content modèle / metadata UI, interop schémas | `packages/schema/src/tool.ts:14-99`, `packages/plugin/src/effect/tool.ts:21-59` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | adopted | M |
| Permissions allow/ask/deny + policy | sécurité | Rulesets ordonnés + assert avant exécution | `packages/schema/src/permission.ts:56-67`, `packages/schema/src/config/policy.ts:1-13` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | adopted | S |
| Event stream durable/éphémère | observabilité | Log versionné rejouable vs transitoire + manifeste | `packages/schema/src/event.ts:42-100` | event-store → `EventStore` → `FileEventStore`/`TraceReplayer` | planned | M |
| Serveur + protocole + surfaces | exécution | OpenAPI généré des schémas, TUI/desktop/web même origine | `packages/protocol/src/groups/plugin.ts:10-61`, `packages/server/src/api.ts` | service → `omega-server` | planned | M |
| Config discovery multi-dossiers | configuration | Couches `.claude/.agents/.opencode`, YAML tolérant | `packages/core/src/config/discovery.ts:69-141` | config-source → `ConfigSource` → `omega-config` | planned | S |
| Code Mode + LSP (écarté) | outils | Runtime JS in-process hors périmètre Rust | `packages/codemode/src/tool-schema.ts:41-142` | — | rejected | L |

## 4. Traduction en plugins

- `oc-plugin-registry` : registry → `Registre` → `omega-registry::Registry` (plugin, adopted).
- `oc-transform-hook` : registry → `Registre` (register_transform/register_hook/reload) → `omega-plugin-api` (plugin, planned).
- `oc-scoped-overlay` : registry → `Registre` (Scope) → `omega-registry` scopes (plugin, planned).
- `oc-source-watch` : registry → `Registre` → `omega-registry` empreinte+reload (plugin, planned).
- `oc-agent-schema` / `oc-command-md` / `oc-plan-mode` : agent-store → `AgentStore` → `plugins/agent-declarative|command-md|plan-mode` (plugin, planned).
- `oc-skill-store` : skill-store → `SkillStore` → `omega-context::SkillIndex` (plugin, adopted).
- `oc-subagents` : subagent-runner → `SubagentRunner` → `plugins/subagent-isolated` (plugin, planned).
- `oc-provider-catalog` : llm-provider → `Provider` → `omega-providers` (plugin, planned).
- `oc-cache-policy` / `oc-compaction` : context-engine → `PromptCachePolicy`/`ContextBudget` → `omega-context` (plugin, planned).
- `oc-tool-contract` : tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` (plugin, adopted).
- `oc-permissions` : permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` (plugin, adopted).
- `oc-event-stream` : event-store → `EventStore` → `omega-app::FileEventStore` + `omega-otel::TraceReplayer` (plugin, planned).
- `oc-server-protocol` : service → `omega-server` cockpit + API (service, planned).
- `oc-config-layers` : config-source → `ConfigSource` → `omega-config` (plugin, planned).

## 5. Écarté

- **`oc-codemode-lsp` (rejected)** : Code Mode (modèle écrivant du TS appelant les outils, `tool-schema.ts` MAX_RENDER_DEPTH=8) et adaptateurs LSP/formatter reposent sur un runtime JS/TS in-process, hors périmètre du cœur Rust. Remplacement prévu : WASM sandboxé si besoin ; LSP reste un plugin-adaptateur externe derrière un port.

## 6. Sources

- Clone local `/home/tayk0oo/omega-re/opencode` (commit `eeec76b`, `2.0.30`, MIT) : fichiers cités en `refs` du JSON (`packages/schema/src/*.ts`, `packages/plugin/src/*`, `packages/core/src/plugin/*`, `packages/core/src/session/*`, `packages/ai/src/cache-policy.ts`, `packages/protocol/src/groups/plugin.ts`, `specs/v2/*.md`).
- Analyse amont : `docs/omega/reverse-eng/RE-OPENCODE-CLAUDECODE.md` (§2, §4–§6).
- Ports cibles : `RE-OPENCODE-CLAUDECODE.md` §4 (`PluginRegistry`, `ContextEngine`, `ModelRegistry`, `PromptCachePolicy`, `ContextBudget`, `ToolRegistry`, `ToolSurface`, `SkillStore`, `SubagentRunner`, `PermissionPolicy`, `TelemetryExporter`).
