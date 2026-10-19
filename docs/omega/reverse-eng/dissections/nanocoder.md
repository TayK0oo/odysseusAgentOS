# Dissection — Nanocoder (Nanotech-NanoCode, Apache-2.0, TypeScript)

> Source : clone `--depth 1` de https://github.com/Nanotech-NanoCode/Nanocoder dans `/home/tayk0oo/omega-re/nanocoder`.
> Donnees : `omega/crates/omega-harness/data/apps/nanocoder.json` (10 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Nanotech-NanoCode |
| Langage | TypeScript |
| Repo | https://github.com/Nanotech-NanoCode/Nanocoder |
| Architecture | CLI agent coding local-first (Ink/React) : tool-registry + profiles, tools LSP, client MCP multi-transports, memoire semantique + summarizer, sous-agents markdown, bash failure capsule + approval policy, stats locales, mode daemon + SDK, repo-map, extensions declaratives skills/custom-tools. |

## 2. Inventaire fonctionnel

### `nano-tool-registry` — ToolManager/ToolRegistry : registry de tools avec profils et conventions de sortie

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : M · **statut** : planned
- **what** : Un registre de tools structure : tool-registry (modules), tool-manager (instances), tool-profiles (selecteurs par contexte), conventions de sorties contractuelles.
- **how** : source/tools/tool-registry.ts + tool-manager.ts + tool-profiles.ts (profiles spec : tool-profiles.spec.ts). Omega : ToolSurface + ToolExecutor ; retenu : les 'tool profiles' qui activent des sous-ensembles de tools selon contexte.
- **adaptateur Omega** : omega-context::ToolSurfaceIndex (profiles de tools)
- **preuves** : `source/tools/tool-registry.ts` · `source/tools/tool-profiles.ts`

### `nano-lsp-tools` — Tools LSP natives : diagnostics + format document

- **domaine** : automatisation · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : planned
- **what** : Deux tools LSP integres : lsp_get_diagnostics (erreurs du language server) et lsp_format_document, utilises comme bouffee revert de l'agent.
- **how** : source/tools/lsp-get-diagnostics.tsx + lsp-format-document.tsx + source/lsp/. Omega : EventStore (diagnostics) + tool format ; le tool diagnostics est un signal de verification post-edit.
- **adaptateur Omega** : omega-app (lsp diagnostics comme verification)
- **preuves** : `source/tools/lsp-get-diagnostics.tsx` · `source/tools/lsp-format-document.tsx`

### `nano-mcp` — Client MCP : transports multiples (stdio/http/sse) via factory

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : S · **statut** : planned
- **what** : Un client MCP generique avec transport-factory (stdio, http, sse) pour brancher tout serveur MCP externe.
- **how** : source/mcp/mcp-client.ts + transport-factory.ts. Omega : couvert (MCP natifs) ; retenu : la factory de transports.
- **adaptateur Omega** : omega-app (mcp client + transports)
- **preuves** : `source/mcp/mcp-client.ts` · `source/mcp/transport-factory.ts`

### `nano-semantic-memory` — SemanticMemoryManager : memoire semantique persistante dedisquee + summarizer

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : M · **statut** : planned
- **what** : Une memoire semantique : SemanticMemoryManager (index recherchee), project-context, proposal-store, summarizer-service (compaction).
- **how** : source/memory/semantic-memory-manager.ts:297 (class) + project-context.ts + summarizer-service.ts. Omega : MemoryStore + ContextEngine ; retenu : la memoire semantique requetable.
- **adaptateur Omega** : omega-app::FileMemoryStore (+ semantic index)
- **preuves** : `source/memory/semantic-memory-manager.ts:297` · `source/memory/summarizer-service.ts`

### `nano-subagents` — Sous-agents built-in avec executor et parsing markdown d'assignation

- **domaine** : agents · **capacite** : subagent-runner · **port** : SubagentRunner · **cout** : S · **statut** : planned
- **what** : Un systeme de sous-agents : built-in (definis en markdown), subagent-executor, parser d'assignation, messages d'echec structurés.
- **how** : source/subagents/{built-in/,subagent-executor.ts,markdown-parser.ts,failure-message.ts}. Omega : LocalSubagentRunner couvert.
- **adaptateur Omega** : omega-app::LocalSubagentRunner (markdown agents)
- **preuves** : `source/subagents/subagent-executor.ts` · `source/subagents/built-in`

### `nano-bash-capsule` — Bash failure capsule : encapsulation d'erreur + approval policy

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : Les commandes shell passent une 'failure capsule' (sortie d'erreur structuree) + approval-policy + validation de chemins + guard command-injection.
- **how** : source/tools/bash-failure-capsule.ts + approval-policy.ts + file-tools-path-validation + source/security/command-injection. Omega : ExecPolicy couvert ; retenu : la capsule de sortie structuree.
- **adaptateur Omega** : omega-core::ExecPolicy (+ capsule de sortie)
- **preuves** : `source/tools/bash-failure-capsule.ts` · `source/tools/approval-policy.ts`

### `nano-stats` — Stats locales : aggregation usage par date, charts

- **domaine** : observabilite · **capacite** : telemetry-sink · **port** : TelemetrySink · **cout** : S · **statut** : planned
- **what** : Un module stats : record/aggregators/chart par date (runs, tokens, models).
- **how** : source/stats/{record.ts,aggregators.ts,chart.ts}. Omega : TelemetrySink.
- **adaptateur Omega** : omega-otel (aggregation locale)
- **preuves** : `source/stats/aggregators.ts`

### `nano-daemon-sdk` — Mode daemon + SDK : API programme local pour integrations

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Le CLI peut tourner en daemon (API locale) et un SDK headless expose le run d'agent aux programmes.
- **how** : source/daemon/ + source/sdk/. Omega : omega-server (API) couvert.
- **adaptateur Omega** : omega-server (mode daemon + SDK)
- **preuves** : `source/daemon` · `source/sdk`

### `nano-repo-map` — Repo map : cartographie du depot pour le contexte

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : S · **statut** : planned
- **what** : Un module repo-map : generating cartographie du depot (structure/fichiers) injecte au prompt.
- **how** : source/repo-map/. Omega : ContextEngine (repo map deja note chez other harnesses).
- **adaptateur Omega** : omega-context (repo map)
- **preuves** : `source/repo-map`

### `nano-skills-tasks` — Skills + custom tools + tasks : extension par fichiers markdown/TOML

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Extension utilisateur par dossiers declaratives : skills/, custom-tools/, custom-commands/ (markdown/TOML charges au boot).
- **how** : source/skills/ + source/custom-tools/ + source/custom-commands/. Omega : chargement declaratif deja couvert (agents/skills md).
- **adaptateur Omega** : omega-app (skills + custom tools declaratifs)
- **preuves** : `source/skills` · `source/custom-tools`

