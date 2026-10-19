# Dissection — Continue (`continuedev/continue`)

> Clone : `/home/tayk0oo/omega-re/continue`, branche `main`, commit `5522c6f` (« docs: remove Sign in link (login flow retired) », 2026-10-17), version noyau `core@1.1.0`.
> Preuve REA : `docs/omega/research/rea/evidence/intake-continue.summary.json` (graphe 11 622 noeuds / 14 639 arêtes, 2 215 limitations — voir §7).
> Corpus observé : ~1 429 fichiers `.ts`, 345 `.tsx`, 86 `.kt` (hors `node_modules`).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Continue Dev Inc. |
| Licence | Apache-2.0 (`LICENSE` : « Apache License Version 2.0 ») |
| Langage | TypeScript (monorepo npm) + extension JetBrains en Kotlin + crate Rust historique `sync/` |
| Repo | https://github.com/continuedev/continue |
| Architecture | Noyau `core/` partagé exécuté en process séparé (`binary/src/index.ts` : `IpcMessenger`/`TcpMessenger`) et piloté par des surfaces IPC : extension VS Code, extension JetBrains (Kotlin), CLI `cn` (TUI React), GUI React `gui/` ; le protocole webview↔noyau est **dupliqué** VS Code/IntelliJ (`constants/MessageTypes.kt`, imposé par `core/rules.md`) ; paquets partagés `packages/config-yaml` (assistant YAML + blocs), `openai-adapters`, `continue-sdk`, `terminal-security`. |

## 2. Entrées

- **Binaires/CLI** : `extensions/cli` (`cn` : `chat`, `checks`, `ls`, `review`, `serve`, `init` — `extensions/cli/src/index.ts:6-17`), `binary/` (noyau détaché IPC/TCP, `binary/src/index.ts`).
- **Extensions IDE** : VS Code (`extensions/vscode/src/extension.ts:16`), JetBrains (`ContinuePluginStartupActivity.kt:86-88`), GUI React embarquée (`gui/src/main.tsx:13-15`), GitHub Action de revue (`actions/README.md`).
- **Points d'extension** : hooks CLI compatibles Claude Code (`extensions/cli/src/hooks/types.ts:10-45`), overrides d'outils par config (`core/tools/applyToolOverrides.ts`), policies d'outils par outil/groupe (`gui/src/pages/config/sections/ToolsSection.tsx:466`), providers de contexte (`core/context/providers/index.ts:43-72`).
- **Fichiers de config** : `.continue/config.yaml` + blocs `.continue/*.yaml` (global + workspace, `core/config/yaml/loadYaml.ts:54-70`), `.continue/rules/`, `.continue/prompts/`, `.continuerules`, `AGENTS.md`/`AGENT.md`/`CLAUDE.md` (`core/config/markdown/loadMarkdownRules.ts:10`), `.continueignore` (`core/indexing/continueignore.ts:6-11`).

## 3. Inventaire fonctionnel

| # | Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|---|
| 1 | Extension VS Code (webview GUI + autocomplete inline) | extension | Sidebar GUI + inline completion branchée sur l'éditeur | `extensions/vscode/src/extension.ts:16`, `ContinueGUIWebviewViewProvider.ts:10` | service → `omega-server` (surface IDE) | planned | M |
| 2 | Extension JetBrains (Kotlin + JCEF) | extension | StartupActivity, listeners VFS/éditeurs, même GUI en JCEF | `ContinuePluginStartupActivity.kt:86-88`, `constants/MessageTypes.kt` | service → `omega-server` | planned | L |
| 3 | CLI/TUI `cn` (chat, serve, review, ls, init) | ui | CLI npm avec TUI React, sessions persistées, auto-compaction, services | `extensions/cli/src/index.ts:6-17`, `src/compaction.ts:19-266` | service → `omega-cli` (à créer) | planned | L |
| 4 | Modes chat/agent/plan/background | agents | Chat sans outils, plan lecture seule (outils readonly), agent outillé | `core/index.d.ts:495`, `ModeSelect.tsx:37-42`, `selectActiveTools.ts:16-32`, `getBaseSystemMessage.ts:15-27` | exec-policy → `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy` | planned | M |
| 5 | Sous-agents nommés (outil `Subagent`) | agents | Délégation à un sous-agent nommé, exécuteur dédié | `extensions/cli/src/subagent/index.ts:3-23`, `executor.ts:58` | subagent-runner → `SubagentRunner` (port déjà présent) | planned | L |
| 6 | Autocomplétion tab (FIM, snippets, prefilter, streaming) | ui | Ghost-text FIM, snippets code+diff, debounce, garde fichiers sensibles | `core/autocomplete/CompletionProvider.ts:33-200`, `snippets/getAllSnippets.ts:169` | service autocomplete (`omega-providers` + `omega-context`) | planned | L |
| 7 | Next Edit (prédiction de zone éditable) | ui | Prédit la prochaine zone à modifier (±5 lignes, diff d'historique, prefetch) | `core/nextEdit/NextEditProvider.ts:57`, `DocumentHistoryTracker.ts:8` | service (mêmes ports que l'autocomplétion) | planned | M |
| 8 | Édition inline : quick edit + diff stream + lazy apply | ui | Diff de lignes streamé, lazy apply avec repli déterministe | `core/edit/streamDiffLines.ts:77`, `lazy/streamLazyApply.ts:14`, `lazy/deterministic.ts:129` | native → `omega-diff` (script déterministe) | planned | M |
| 9 | Indexation de codebase (embeddings + full-text + snippets + cache SQLite) | contexte | Index incrémental : vecteurs LanceDB, plein texte, fragments tree-sitter, `.continueignore` | `core/indexing/CodebaseIndexer.ts:48`, `LanceDbIndex.ts:41-203`, `refreshIndex.ts:21-546` | context-engine → `ContextEngine` → `omega-context::DefaultContextEngine` | planned | L |
| 10 | Retrieval RAG (rerank / no-rerank) + repo map | contexte | Pipeline de retrieval avec reranker LLM optionnel + repo map | `core/context/retrieval/retrieval.ts:11-73`, `pipelines/RerankerRetrievalPipeline.ts:12` | context-engine → `ContextEngine` | planned | M |
| 11 | Providers de contexte `@` (30 providers) | contexte | Sources de contexte standardisées avec sous-menus paginés | `core/context/providers/index.ts:43-72`, `loadContextProviders.ts:22` | context-engine → `ContextEngine` (contributor) | planned | M |
| 12 | Indexation de documentation web (`@docs`) | contexte | Crawl → articles → chunks vectorisés, retrieval ciblé | `core/indexing/docs/DocsService.ts:167`, `article.ts:123-182` | context-engine → `ContextEngine` | planned | M |
| 13 | Auto-compaction (seuil 80 % + résumé LLM structuré) | contexte | Résumé 6 sections + anti-boucle ; Omega n'a qu'une compaction déterministe | `core/util/conversationCompaction.ts:19`, `extensions/cli/src/compaction.ts:19-266` | context-engine → `ContextEngine` (`omega-context::compaction` existant) | planned | M |
| 14 | Skills markdown + outil `read_skill` | contexte | Skills listés au modèle, lus par un outil dédié | `core/config/markdown/loadMarkdownSkills.ts:45`, `tools/definitions/readSkill.ts:6-19` | skill-store → `SkillStore` → `omega-context::SkillIndex` | **adopted** | S |
| 15 | Rules (`.continue/rules`, `AGENTS.md`, globs/regex/alwaysApply) | configuration | Règles injectées au system prompt, application ciblée par fichier | `core/index.d.ts:1921-1931`, `llm/rules/getSystemMessageWithRules.ts:205-332` | context-engine → `ContextEngine` | planned | M |
| 16 | Prompt files `.prompt` → slash commands | agents | Fichiers prompt réutilisables (preamble YAML + `<system>`) | `core/promptFiles/parsePromptFile.ts:5-26`, `slash/promptFileSlashCommand.ts:4` | skill-store → `SkillStore` (étendu) | planned | S |
| 17 | Config en couches (`.continue/` global + workspace + profils) | configuration | Fusion de couches + sélection de profil/assistant | `core/config/yaml/loadYaml.ts:54-70`, `ConfigHandler.ts:31-139` | config-source → `ConfigSource` → `omega-config` | planned | S |
| 18 | Assistant YAML + blocs composables (`uses:`/`with:`/`override:`) | configuration | Blocs models/context/data/mcpServers/rules/prompts/docs, dédupliqués et déroulés | `packages/config-yaml/src/schemas/index.ts:81-164`, `load/getBlockType.ts:3-10`, `load/unroll.ts` | config-source → `ConfigSource` | planned | M |
| 19 | Hub distant `owner/package` (registre cloud) | configuration | **Écarté** : résolution de blocs cloud + secrets d'orga supprimée de l'OSS | `registryClient.ts:29`, `extensions/cli/src/hubLoader.ts:1-7,59` | — (port `CatalogSource` local suffit) | rejected | L |
| 20 | Pont client MCP (initialize, tools/list, call) | outils | Serveurs MCP stdio/npx exposés en outils derrière `Capability::ToolSurface` | `omega/crates/omega-mcp/src/lib.rs:57-259,391-564`, `omega-server/src/state.rs:104` | tool-surface → `ToolSurface` → `omega-mcp::McpToolSurface` | **adopted** | S |
| 21 | MCP multi-serveurs + OAuth SSE + `@mcp` | outils | Plusieurs serveurs, OAuth avec stockage par serveur, ressources en contexte | `core/context/mcp/MCPManagerSingleton.ts:6`, `MCPConnection.ts:122-151`, `MCPOauth.ts:76-274` | tool-surface → `ToolSurface` → `omega-mcp` (extension) | planned | M |
| 22 | Catalogue d'outils built-in + overrides + groupes | outils | Outils adaptables (modèle recommandé, remote, experimental), policies par outil | `core/tools/index.ts:6-53`, `applyToolOverrides.ts` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | **adopted** | M |
| 23 | Politiques d'outils + garde hors workspace | sécurité | allow/deny par outil, `allowedWithPermission` hors workspace, YAML + precedence (CLI) | `core/tools/policies/fileAccess.ts:11-27`, `extensions/cli/src/permissions/precedenceResolver.ts:33` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | **adopted** | S |
| 24 | Liste de verrou : secrets jamais lus/édités/indexés | sécurité | `.env`, `*.secret`, clés cloud interdits (erreur explicite) | `core/indexing/ignore.ts:211-258`, `CompletionProvider.ts:170` | exec-policy → `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy` | planned | S |
| 25 | Hooks compatibles Claude Code (18 événements) | extension | PreToolUse/Stop/WorktreeCreate... exécutés en shell/HTTP/prompt/agent | `extensions/cli/src/hooks/types.ts:5-45`, `HookService.ts:32` | registry → `Registre` (hooks, port `Hook`) | planned | M |
| 26 | Registre multi-fournisseurs (60+) + autodetect de template + rôles | routage | Classes LLM résolues depuis la config, templates devinés, rôles embeddings/rerank/apply | `core/llm/llms/index.ts:73-190`, `autodetect.ts:313-471`, `packages/openai-adapters/src/apis` | llm-provider → `Provider` → `omega-providers::OllamaProvider` | planned | L |
| 27 | Télémétrie OpenTelemetry (OTLP + console) | observabilité | Métriques CLI exportées OTLP/console, intervalle et en-têtes configurables | `extensions/cli/src/telemetry/telemetryService.ts:4-149`, `headerUtils.ts:1` | telemetry-sink → `TelemetrySink` → `omega-otel::{Stdout,File}Sink` | **adopted** | S |
| 28 | `cn serve` : agent en HTTP + prompt initial + reprise | exécution | Serveur express, file de messages, snapshot git, reprise de session | `extensions/cli/src/commands/serve.ts:4-70`, `serve.helpers.ts` | service → `omega-server` | planned | M |
| 29 | Revue de PR : workers + worktrees isolés + GitHub Action | automatisation | Une revue par worker forké dans un worktree git, rapport/patch, action `@continue-review` | `commands/review.ts:22-60`, `review/worktree.ts:15-100`, `actions/README.md` | service → `omega-sandbox::LocalSandboxBackend` + `omega-server` | planned | M |

Domaines couverts : `extension`, `ui`, `agents`, `contexte`, `configuration`, `outils`, `securite`, `routage`, `observabilite`, `execution`, `automatisation` — 29 fonctionnalités, dont 5 `adopted`, 1 `rejected`, 23 `planned`.

## 4. Traduction en plugins

- `ct-skills-read` : skill-store → `SkillStore` → `omega-context::SkillIndex` (plugin, **adopted** — `list/view/asset` couvrent déjà le contrat).
- `ct-mcp-bridge` : tool-surface → `ToolSurface` → `omega-mcp::McpToolSurface` (plugin, **adopted** — pont déjà enregistré comme `Capability::ToolSurface` dans `omega-server` ; hors périmètre : OAuth et multi-serveurs, voir `ct-mcp-host`).
- `ct-tools-catalog` : tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` (plugin, **adopted** — index + hydratation existent ; à étendre aux *overrides* de catalogue).
- `ct-tool-permissions` : permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` (plugin, **adopted** — file d'approbations et règles existent ; le matching « outil + arguments » du CLI reste à porter).
- `ct-telemetry-otel` : telemetry-sink → `TelemetrySink` → `omega-otel::{Stdout,File}Sink` (plugin, **adopted**).
- `ct-modes-chat-agent-plan` : exec-policy → `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy` (plugin, planned) — le mode plan se traduit en deny-by-default hors lecture, sans nouveau port.
- `ct-security-ignores` : exec-policy → `ExecPolicy` + `LocalSandboxBackend` (plugin, planned) — règles deny sur motifs de chemins (secrets) à ajouter au DSL de préfixes.
- `ct-index-embeddings` / `ct-rag-retrieval` / `ct-context-providers` / `ct-docs-indexing` / `ct-rules` / `ct-compaction-llm` : context-engine → `ContextEngine` → `omega-context` (plugin, planned) — un seul port absorbe indexation, retrieval, providers, règles et budget de contexte.
- `ct-prompt-files` : skill-store → `SkillStore` → `omega-context::SkillIndex` étendu aux blocs de prompt (plugin, planned).
- `ct-config-layers` / `ct-config-hub-blocks` : config-source → `ConfigSource` → `omega-config` (plugin, planned) — couches/precedence déjà présentes, déroulé de blocs `uses:` à écrire.
- `ct-hooks-claude-code` : registry → `Registre` (registre de hooks de cycle de vie sur le port `Hook`, plugin, planned).
- `ct-subagents` : subagent-runner → `SubagentRunner` (plugin, planned) — le port existe déjà (`omega-ports`), il manque l'adaptateur.
- `ct-llm-providers` : llm-provider → `Provider` → `omega-providers` (plugin, planned) — extension du catalogue au-delà d'Ollama + rôles embeddings/rerank.
- Surfaces (`ct-extension-vscode`, `ct-extension-jetbrains`, `ct-cli-tui-cn`, `ct-serve-bg-agents`, `ct-review-automation`, `ct-autocomplete-fim`, `ct-next-edit`) : `service`, sans capacité/port forcé, car ce sont des **clients** du noyau derrière `omega-server`/`omega-protocol`.
- `ct-inline-edit-diff` : `native` — le diff streamé et le repli déterministe sont des **scripts** (règle « le déterministe n'est jamais une tâche d'agent »), à héberger dans un module `omega-diff`.

## 5. Écarté

- **`ct-hub-cloud` (rejected)** : la résolution de blocs `uses: owner/package` depuis le registre cloud de Continue et la synchronisation de secrets d'organisation (`/ide/sync-secrets`) dépendent du service de l'éditeur — et ont d'ailleurs été **retirées du dépôt OSS** (`registryClient.ts:29`, `hubLoader.ts:59`). Reprendre ce mécanisme contredirait le pilier « jamais dépendre d'un fournisseur unique » : le port `CatalogSource` (auto-hebergé) suffit.
- Non repris faute de valeur pour Omega : providers propriétaires tierces (Greptile, Google, Jira, GitLab MR, Discord) — à traiter comme adaptateurs externes derrière le port `ContextEngine`, pas comme fonctionnalités du cœur ; télémétrie « continue.dev » de plateforme (l'OTel standard est déjà couvert) ; métriques d'usage `schemas/data/*` (autocomplete/nextEdit/quickEdit outcomes) — utile pour le scoring de modèles, à rattacher plus tard à `omega-router`/`CatalogSource` si le besoin se confirme.

## 6. Sources

- Clone local `/home/tayk0oo/omega-re/continue` (commit `5522c6f`, `core@1.1.0`, Apache-2.0) : tous les chemins `file:line` du tableau et du JSON `refs`.
- Documentation officielle citée dans le code : https://docs.continue.dev/features/tab-autocomplete, https://docs.continue.dev/features/prompt-files, https://docs.continue.dev/customization/context-providers#built-in-context-providers, https://docs.continue.dev/walkthroughs/codebase-embeddings, https://docs.continue.dev/customize/model-roles/embeddings, https://docs.continue.dev/cli/quickstart, https://docs.continue.dev/hub/agents/overview, https://docs.continue.dev/telemetry.
- Côté Omega : `omega/crates/omega-ports/src/lib.rs`, `omega-context/src/{tool_surface,skills,compaction}.rs`, `omega-sandbox/src/{policy,approvals}.rs`, `omega-otel/src/sinks.rs`, `omega-mcp/src/lib.rs`, `omega-config/src/loader.rs`, `omega-server/src/{state,routes}.rs`.
- Sorties jumelles : `omega/crates/omega-harness/data/apps/continue.json` (29 fonctionnalités) + ce récit.

## 7. Corroboration REA (et limites)

- Preuve : `docs/omega/research/rea/evidence/intake-continue.summary.json` — provider `rea-javascript-application`, opération `analyze_javascript_application`, `status: analyzed`, `cloned: true`, confidence **derived**, digest `sha256:80a46dd0…`, graphe de dépendances 11 622 noeuds / 14 639 arêtes.
- Ce que la preuve **corrobore** : la nature monorepo JS/TS du dépôt et son échelle (~14 300 fichiers sources TS/TSX/Kotlin observés localement, cohérent avec un graphe de cette taille) ; l'absence de compileur/VM exotique (tout le noyau est TS, seule la bibliothèque native `@lancedb` et le crate `sync/` sortent du JS).
- Ce que la preuve **ne couvre pas** (limites assumées, `limitations_count: 2215`) : le résumé REA ne livre **aucune** fonctionnalité nommée, ni fichier:ligne — c'est un digest de graphe, pas un inventaire fonctionnel. Toutes les lignes du §3 proviennent donc d'une lecture directe du clone (`[observed]`), la preuve REA servant uniquement de contrôle de cohérence d'échelle et de nature. Les 2 215 « limitations » ne sont pas détaillées dans le résumé : impossible de s'y appuyer pour écarter quoi que ce soit.
