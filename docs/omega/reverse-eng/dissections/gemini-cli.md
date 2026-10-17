# Dissection — Gemini CLI (`google-gemini/gemini-cli`)

> Clone : `/home/tayk0oo/omega-re/gemini-cli`, commit `ed96fae` (2026-10-16), version `0.66.0-nightly.20261013.g204db94ae`.
> Preuve machine : `docs/omega/research/rea/evidence/intake-gemini-cli.summary.json` (provider `rea-javascript-application`, opération `analyze_javascript_application`, confiance `derived`, 9 177 nœuds / 13 543 arêtes de graphe, **3 666 limitations déclarées**).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Google (Google DeepMind / Gemini) |
| Licence | Apache-2.0 (`LICENSE` du clone : « Apache License Version 2.0 ») |
| Langage | TypeScript (monorepo npm `packages/*`, CLI en `react-ink`) |
| Repo | https://github.com/google-gemini/gemini-cli |
| Architecture | Noyau agent `packages/core` (boucle, outils, MCP, politique, sandbox, télémétrie, routage, skills, mémoire, hooks, scheduler) ; CLI `packages/cli` (TUI ink, settings, extensions, ACP, commandes) ; serveur A2A dédié `packages/a2a-server` ; SDK `packages/sdk` ; compagnon IDE VS Code `packages/vscode-ide-companion` exposant **un serveur MCP** ; sécurité en couches : `PolicyEngine` (allow/deny/ask) + modes d'approbation + sandbox OS (bubblewrap/seatbelt/AppContainer) + dossier de confiance + keychain. |

**Lecture REA / limites.** L'analyse REA confirme l'entrée (repo, TypeScript, Apache-2.0, `cloned: true`, statut `analyzed`) et fournit un graphe dense (9 177 nœuds), mais déclare **3 666 limitations** : l'analyse JS est *derived* (dispatch dynamique, `require` conditionnels, génération de code par templates). Chaque `refs` de cette dissection a donc été **relu manuellement dans le clone** ; les lignes citées sont celles du commit `ed96fae`. Le digest REA (`c8a81e8a…`) atteste la version analysée.

## 2. Entrées

- **Binaires/CLI** : `gemini` (`packages/cli/package.json` → `bin.gemini`, bundle `dist/index.js`), binaire séparé `gemini-cli-a2a-server` (`packages/a2a-server/package.json`).
- **Modes** : interactif (TUI ink), `--headless`/`-p` non-interactif (`packages/cli/src/nonInteractiveCli.ts`), `--acp` (Agent Client Protocol, stdio ndjson), serveur A2A.
- **Groupes de commandes** : `gemini extensions` (`packages/cli/src/commands/extensions/`), `gemini mcp add|remove|list|enable|disable` (`packages/cli/src/commands/mcp.ts:16-29`), `gemini hooks`, `gemini skills`.
- **Points d'extension** : hooks de cycle de vie (`docs/hooks/reference.md`), extensions installables (`gemini-extension.json`), agents markdown (`.gemini/agents/*.md`), skills (`SKILL.md`), commandes TOML (`.gemini/commands/**/*.toml`), thèmes custom, serveurs MCP.
- **Configuration** : `settings.json` en 5 scopes (`packages/cli/src/config/settings.ts:131-139` : SystemDefaults/System/User/Workspace/Session), politiques TOML (`packages/core/src/policy/policies/*.toml`), `GEMINI.md` (contexte mémoire), variables d'env (`GEMINI_CLI_SYSTEM_SETTINGS_PATH`).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Boucle agent ouverte + scheduler | execution | `while(true)` LLM→outil→LLM découpée en Turn, tâches planifiées | `agent/agent-session.ts:18-208`, `core/turn.ts:253`, `scheduler/scheduler.ts:139` | native → `AgentLoop`/`App::run` | planned | L |
| Registre d'outils + déclarations dynamiques | outils | Tous les outils connus/actifs, déclarations par famille de modèle | `tools/tool-registry.ts:247`, `tools/definitions/resolver.ts:17` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | adopted | M |
| Shell (PTY, arrière-plan) | outils | Shell interactif + jobs persistants (lire/relister) | `tools/shell.ts:1190`, `tools/shellBackgroundTools.ts:71-249`, `services/shellExecutionService.ts` | tool-surface + sandbox-backend | planned | M |
| Outils web (Grounding + fetch) | outils | `web_search` API Gemini, `web_fetch` avec normalisation/limites | `tools/web-search.ts:207`, `tools/web-fetch.ts:153-899` | tool-surface → `plugins/web-grounding` | planned | S |
| Client MCP (spawn, discovery, call) | extension | stdio/SSE/HTTP, `tools/list` → outils préfixés au modèle | `tools/mcp-client.ts:153-1842`, `mcp-client-manager.ts:37-375` | tool-surface → `omega-mcp::McpToolSurface` | adopted | M |
| MCP étendu + exposition serveur | extension | OAuth MCP, resources/prompts, serveur MCP côté CLI (IDE) | `mcp/oauth-provider.ts`, `mcp-client.ts:1505-1648`, `vscode-ide-companion/src/ide-server.ts:14` | tool-surface → `omega-mcp` (à étendre) | planned | M |
| Sous-agents (registre, délégation, @mention) | agents | Agents spécialisés en `.md`, outil de délégation, routage forcé `@nom` | `agents/registry.ts:50`, `agents/agent-tool.ts:43-217`, `docs/core/subagents.md` | subagent-runner → `SubagentRunner` | planned | L |
| Agents distants A2A + serveur | agents | Délégation hors-processus (HTTP JSON-RPC, tâches persistées) | `agents/a2a-client-manager.ts:52`, `packages/a2a-server/src/http/server.ts` | service → `omega-server` + `plugins/a2a` | planned | L |
| Skills : découverte + index + actifs | contexte | `SKILL.md` glob, frontmatter validé, vues et assets | `skills/skillManager.ts:17-104`, `skills/skillLoader.ts:154-189` | skill-store → `SkillStore` → `omega-context::SkillIndex` | adopted | M |
| Outil `activate_skill` | contexte | Disclosure progressive : charge le dossier de ressources + arborescence | `tools/activate-skill.ts:36-160` | tool-surface → `plugins/skill-activation` | planned | S |
| Mémoire `GEMINI.md` hiérarchique + JIT | mémoire | Remontée parent/dossier, dédoublonnage (dev,ino), JIT sous-répertoires | `utils/memoryDiscovery.ts:54-610`, `tools/memoryTool.ts`, `config/memory.ts:15` | memory-store → `MemoryStore` | planned | S/M |
| Imports de mémoire `@fichier` (memport) | mémoire | Modularisation des `GEMINI.md`, anti-cycle, confinement | `docs/reference/memport.md`, `utils/memoryImportProcessor.ts` | memory-store → `MemoryStore (resolve)` | planned | S |
| Auto Memory (extraction) | mémoire | Mine les transcripts, propose patches mémoire + `SKILL.md`, inbox à approuver | `services/memoryService.ts:84-1133`, `agents/skill-extraction-agent.ts`, `docs/cli/auto-memory.md` | memory-store → `plugins/auto-memory` | planned | L |
| Compression de contexte | contexte | Summarisation LLM des tours anciens, collapse des function responses, masking et distillation | `context/chatCompressionService.ts:45-716`, `toolOutputMaskingService.ts:24-125`, `toolDistillationService.ts:39`, `contextCompressionService.ts:16-510` | context-engine → `ContextEngine`/`CompactionModel` | planned | M |
| File de confirmation outil (ask_user) | sécurité | Toute action à risque passe par confirmation ; outcomes `ProceedAlways[+AndSave]` | `confirmation-bus/message-bus.ts:15`, `tools/tools.ts:245-256`, `policy/policy-engine.ts:242-301` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | adopted | S |
| Modes d'approbation (4) | sécurité | `default/autoEdit/yolo/plan`, ordonnés par permissivité, commutables en session | `policy/types.ts:48-65`, `config/config.ts:1036-1349` | permission-policy → `PermissionPolicy` (mode persisté) | planned | S |
| Moteur de politiques TOML | sécurité | allow/deny/ask par outil/sous-agent, intégrité, politiques par workspace | `policy/policy-engine.ts:242-439`, `policy/policies/*.toml`, `docs/reference/policy-engine.md:297` | exec-policy → `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy` | planned | M |
| Sandbox OS (bwrap/seatbelt) | sécurité | Isolation réelle des commandes et écritures FS hors projet, par OS | `sandbox/linux/LinuxSandboxManager.ts:144`, `bwrapArgsBuilder.ts:33`, `macos/seatbeltArgsBuilder.ts`, `services/sandboxManager.ts` | sandbox-backend → `LocalSandboxBackend` (à étendre) | planned | M |
| Dossier de confiance + keychain | sécurité | Consentement avant exécution des artefacts d'un dossier, verrou settings workspace, secrets en keychain | `services/FolderTrustDiscoveryService.ts:29-62`, `trustedFolders.ts:37-44`, `settings.ts:315-494`, `keychainService.ts:27-63` | permission-policy → `plugins/folder-trust` | planned | S |
| Checkpointing + `/restore` + `/rewind` | execution | Git ombre, restauration par checkpoint, rewind conversation+fichiers | `services/gitService.ts:55-223`, `utils/checkpointUtils.ts:48-164`, `commands/restore.ts:11`, `docs/cli/{checkpointing,rewind}.md` | event-store → `EventStore` → `FileEventStore`/`TraceReplayer` | planned | L |
| Hooks de cycle de vie | extension | 11 événements (Before/After Tool/Model/Agent, PreCompress, Session*) via commandes shell | `hooks/hookSystem.ts:151-216`, `docs/hooks/reference.md:21-287` | hook → `Hook` (port présent, adaptateur absent) | planned | M |
| Extensions installables | extension | Paquets GitHub/locaux (MCP, commandes, thèmes, skills…), consentement, intégrité (hash) | `cli/src/config/extension-manager.ts:106-181`, `core/src/config/storage.ts:473-477`, `extensions/integrity.ts:224` | registry → `Registre` → `omega-registry` | planned | M |
| Settings en couches (5 scopes) | configuration | Précédence, env assaini, validation schema, gestionnaire de fichiers | `cli/src/config/settings.ts:131-1059`, `settingsSchema.ts`, `services/environmentSanitization.ts` | config-source → `ConfigSource` → `omega-config` | adopted | S |
| Commandes slash TOML | agents | `/nom` depuis `~/.gemini`, projet, extensions ; rechargeables | `cli/src/services/FileCommandLoader.ts:64-304`, `ui/commands/commandsCommand.ts:38-137` | registry → `Registre` + `plugins/command-toml` | planned | S |
| TUI ink + 19 thèmes + `/theme` | ui | Interface terminal react, thèmes builtin dark/light + thèmes d'extensions, no-color, accessibilité | `cli/package.json:52`, `ui/themes/theme-manager.ts:55-663`, `ui/themes/builtin/{dark,light}/`, `docs/cli/themes.md` | native → `omega-server` (surface web) + TUI dédiée | planned | M |
| Enregistrement durable des sessions | observabilité | Transcripts par projet (`~/.gemini/tmp/<hash>/chats`), `/chat resume`, `/export`, retention | `services/chatRecordingService.ts:670`, `docs/cli/session-management.md`, `settings.ts:186` | event-store → `FileEventStore` + `TraceReplayer` | adopted | M |
| Télémétrie OTel (local/GCP) | observabilité | Traces/metrics/logs, cible fichier ou OTLP/GCP, `logPrompts`, sanitization | `docs/reference/configuration.md:2544-2575`, `telemetry/file-exporters.ts:60-90`, `gcp-exporters.ts`, `sanitize.ts` | telemetry-sink → `omega-otel::{Stdout,File}Sink` | adopted | S |
| Détection de boucles | observabilité | Boucles d'outils / contenu streame / tours LLM détectées et coupées | `services/loopDetectionService.ts:123-157`, `core/turn.ts:66` | loop-guard → `LoopGuard` (nouveau port) | planned | M |
| Chaîne de stratégies de routage | routage | override → fallback → approvalMode → classifiers → composite, décision tracée | `routing/modelRouterService.ts:30-128`, `routing/routingStrategy.ts:21-49`, `strategies/*` | router-policy → `ScoringPolicy`/`Selector`/`FallbackPolicy` → `omega-router::Composite`/`Pareto`/`TierFallback` | adopted | M |
| Bascule sur défaillance de modèle | routage | Classification d'erreurs, politiques retry/consentement, repli silencieux pour les utilitaires | `availability/modelAvailabilityService.ts:37-48`, `modelPolicy.ts:44`, `errorClassification.ts`, `docs/cli/model-routing.md` | router-policy → `FallbackPolicy` → `omega-router::TierFallbackPolicy` | adopted | M |
| Routage par classificateur local (Gemma) | routage | Sélection par modèle entraîné (LiteRT) + seuil, repli numérique | `strategies/gemmaClassifierStrategy.ts`, `strategies/numericalClassifierStrategy.ts`, `core/localLiteRtLmClient.ts` | task-classifier → `TaskClassifier` → `omega-router::HeuristicTaskClassifier` | planned | M |
| Plan Mode | execution | Environnement lecture seule + outils enter/exit + `plan.toml` + modèle Pro forcé | `policy/types.ts:48-65`, `tools/enter-plan-mode.ts:29`, `strategies/approvalModeStrategy.ts:60`, `docs/cli/plan-mode.md` | exec-policy → `RuleBasedExecPolicy` (profil lecture seule) | planned | S |
| ACP (IDE, JSON-RPC stdio) | execution | Sessions multiples, permissions côté client, FS borné au workspace, streaming | `cli/src/acp/acpSessionManager.ts:34-382`, `acpRpcDispatcher.ts:21`, `acpFileSystemService.ts:15`, `docs/cli/acp-mode.md` | service → `omega-server` + `plugins/acp` | planned | M |
| Isolation git worktree | execution | Worktree isolé (commit de base capturé), nettoyage si non modifié | `services/worktreeService.ts:24-69`, `settings.ts:782`, `docs/cli/git-worktrees.md` | sandbox-backend → `plugins/worktree-isolation` | planned | M |
| Entrée vocale | ui | Enregistrement micro + transcription Live API ou Whisper local | `voice/audioRecorder.ts:22`, `transcriptionFactory.ts`, `whisperTranscriptionProvider.ts` | channel → `Channel (audio in)` | planned | L |

## 4. Traduction en plugins

- **`gc-tool-registry`** : tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` (plugin, **adopted**) — même contrat « specs indexées + hydratation » ; les déclarations par famille de modèle deviennent des variantes de `ToolSpec`.
- **`gc-mcp-client`** : tool-surface → `ToolSurface` → `omega-mcp::McpToolSurface` (plugin, **adopted**) — `McpContributor` s'insère déjà dans le `Registre` comme n'importe quel contributeur.
- **`gc-mcp-extended-server`** : extension d'`omega-mcp` (OAuth → `TokenStore` port, resources/prompts, puis `McpServerSpec` → vrai serveur pour exposer *nos* outils) (plugin, planned).
- **`gc-skills`** : skill-store → `SkillStore` → `omega-context::SkillIndex` (plugin, **adopted**) ; **`gc-skill-activation`** ajoute l'outil `activate_skill` sur la même surface (planned).
- **`gc-approval-ask`** : permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` (plugin, **adopted**) — l'outcome « toujours allow » Omega = persistance de règles dans le ruleset (`gc-approval-modes`, planned).
- **`gc-settings`** : config-source → `ConfigSource` → `omega-config` (plugin, **adopted**) — les 5 scopes Gemini = 5 `ConfigLayerSource` ordonnés par `precedence()`.
- **`gc-session-record`** : event-store → `EventStore` → `omega-app::FileEventStore` + `omega-otel::TraceReplayer` (plugin, **adopted**) — le replay à froid existe ; le rangement « par projet » (hash de cwd) et la reprise conversationnelle UI restent à faire.
- **`gc-telemetry`** : telemetry-sink → `TelemetrySink` → `omega-otel::{StdoutSink,FileSink}` (plugin, **adopted**) — les cibles GCP/OTLP de Gemini = nouveaux sinks derrière le même port.
- **`gc-router-chain`** / **`gc-model-fallback`** : router-policy → `ScoringPolicy`/`Selector`/`FallbackPolicy` → `omega-router::{CompositeScoringPolicy,ParetoSelector,TierFallbackPolicy}` (plugin, **adopted**) — `CircuitState` couvre déjà la coupure sur défaillance.
- **`gc-hooks`** : hook → `Hook` (port `omega-ports:154`, **sans adaptateur**) → `omega-plugin-api` : déclencheur typé + exécution déléguée (planned).
- **`gc-subagents`** / **`gc-remote-a2a`** : subagent-runner → `SubagentRunner` (port sans adaptateur) → `plugins/subagent-delegation` ; A2A en `service` derrière `omega-server` (planned, L).
- **`gc-memory`** / **`gc-memory-imports`** / **`gc-auto-memory`** : memory-store → `MemoryStore` (port sans adaptateur) → `plugins/memory-hierarchical`, `plugins/memory-imports`, `plugins/auto-memory` (planned).
- **`gc-compaction`** : context-engine → `ContextEngine` + `CompactionModel` → `omega-context` (compaction par marqueur existante, summarisation LLM à ajouter comme modèle de compaction) (planned).
- **`gc-policy-toml`** / **`gc-plan-mode`** / **`gc-sandbox`** / **`gc-worktree`** : exec-policy / sandbox-backend → `omega-sandbox` (planned) — étendre `RuleBasedExecPolicy` au niveau *outil*, ajouter isolation namespace/FS (`bwrap`/`seatbelt`), worktree comme mode d'isolation.
- **`gc-extensions`** / **`gc-commands`** : registry → `Registre` → `omega-registry` + installateur (planned) — les extensions Gemini = « paquet de contributeurs » installés dans le registre.
- **`gc-agent-loop`** / **`gc-tui-themes`** : intégration `native` (cœur du harnais) — boucle LLM→outil ouverte au-delà du plan durable ; surface TUI en complément du cockpit web.
- **`gc-acp`** : `service` — JSON-RPC stdio exposant les mêmes routes qu'`omega-server`.
- **`gc-voice`** : channel → `Channel (audio in)` (planned, L).

## 5. Écarté

Aucune fonctionnalité majeure n'est `rejected` : le périmètre Gemini est en forte adhérence avec nos piliers (tout ce qui est listé est soit déjà adopté, soit planifié). Deux réserves à ne pas confondre avec de l'adoption :

- **La sandbox Gemini n'est pas notre `LocalSandboxBackend`** : elle isole réellement (namespaces Linux, seatbelt macOS) alors que notre backend fait `sh -c` + timeout + politique. C'est pour cela que `gc-sandbox` reste `planned` malgré un port existant.
- **Le routage par classificateur Gemma** (`gc-classifier`) repose sur un modèle entraîné embarqué (LiteRT) : nous gardons `HeuristicTaskClassifier` (déterministe) — le « déterministe n'est jamais une tâche d'agent » s'applique ici, mais l'apprentissage du scoring relèverait d'un adaptateur hors cœur.

## 6. Sources

- Clone local `/home/tayk0oo/omega-re/gemini-cli` (commit `ed96fae`, `0.66.0-nightly.20261013`, Apache-2.0 vérifié sur `LICENSE`) : fichiers cités en `refs` du JSON.
- Documentation officielle du clone : `docs/reference/{configuration,commands,policy-engine,tools}.md`, `docs/cli/{checkpointing,rewind,auto-memory,model-routing,plan-mode,session-management,sandbox,themes,skills,git-worktrees,acp-mode}.md`, `docs/core/{subagents,remote-agents}.md`, `docs/extensions/*.md`, `docs/hooks/reference.md`.
- Preuve machine REA : `docs/omega/research/rea/evidence/intake-gemini-cli.summary.json` (digest `c8a81e8a…`, 9 177 nœuds / 13 543 arêtes, 3 666 limitations, confiance `derived`).
- Ports cibles : `omega/crates/omega-ports/src/lib.rs` (traits), `omega/crates/omega-types/src/lib.rs:242-270` (enum `Capability`), `omega/crates/omega-{mcp,context,sandbox,router,otel,app,config}/src`.
