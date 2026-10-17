# Dissection — Qwen Code (`qwen-code`)

> Provenance : `[observed]` = lu dans le clone `--depth 1` du 2026-10-17 (version `0.25.2`,
> branche par defaut) ; `[stated]` = declare par le projet ; `[inferred]` = synthese pour Omega.
> Donnees : `omega/crates/omega-harness/data/apps/qwen-code.json` (23 fonctionnalites).
> Preuve REA : `docs/omega/research/rea/evidence/intake-qwen-code.summary.json`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Alibaba / Qwen `[stated]` |
| Licence | Apache-2.0 (`LICENSE` dans le clone) `[observed]` |
| Langage | TypeScript (monorepo pnpm) `[observed]` |
| Repo | https://github.com/QwenLM/qwen-code (28 528 etoiles au moment de l'intake) |
| Architecture | Fork historique de Gemini CLI, depuis tres diverge : noyau `packages/core` (boucle `Turn` evenementielle + `CoreToolScheduler`, registre d'outils unifie builtin/MCP/plugins, permissions a classifieur, memoire, skills, extensions + marketplace, sandbox OS, telemetrie OpenTelemetry) ; presentation `packages/cli` (TUI Ink, slash commands, non interactif, serve) ; versant desktop/live separe et SDK multi-langages. `[inferred]` |
| Limites REA | preuve `code: resource_constraint` — « the parent's result capacity was exhausted after retaining static facts for packages/live-host/src/preload/audio-engine.ts ; remaining sources were not analyzed » ; l'analyse machine s'est arretee tot (versant live), la dissection ci-dessous est donc faite par lecture directe du clone. |

## 2. Entrées

| Entree | Chemin `[observed]` |
|---|---|
| CLI | `packages/cli` (bin `qwen`) ;
non interactif `src/nonInteractiveCli.ts` |
| Serve / SDK | `packages/cli/src/serve`, `packages/managed-agent-server`, `packages/cli/src/acp-integration` |
| Config utilisateur | `~/.qwen/` : `settings.json`, `QWEN.md`, `extensions/`, `skills/`, `agents/`, `memory/` (`packages/core/src/config/storage.ts:46-649`) |
| Points d'extension | extensions marketplace/GitHub/zip (`packages/core/src/extension/`), hooks (`packages/core/src/hooks/`), sous-agents markdown, skills markdown, convertisseurs d'autres harnais (`claude-converter.ts`, `gemini-converter.ts`, `qoder-converter.ts`) |
| Others | desktop (`packages/desktop`), live (`packages/live-host`, `packages/qwen-live`), browser/cua (`packages/browser-use`, `packages/cua-driver`), SDK Java/Python/TS |

## 3. Inventaire fonctionnel

| # | Fonctionnalite | Domaine | Ce que ca fait | Ou (`file:line`) `[observed]` | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|---|
| 1 | Boucle `Turn` + `CoreToolScheduler` | execution | evenements + batching d'outils + confirmations fusionnees + budget par tour | `core/turn.ts:710`, `core/coreToolScheduler.ts:1751` | omega-core (natif) | planned | L |
| 2 | Registre d'outils + tool search differes | outils | registre unifie builtin/MCP/plugins + recherche lexicale | `tools/tool-registry.ts:234,1444`, `tools/tool-search.ts:598,718` | `ToolSurface` → `omega-context::ToolSurfaceIndex` | **adopted** | M |
| 3 | MCP : pool de transports + budgets | outils | cles stables, pool reutilisable, limites 1 Mo/10 s | `tools/mcp-client-manager.ts:387`, `mcp-transport-pool.ts:105`, `mcp-app-resource-limits.ts:12-16` | `ToolSurface` + `omega-mcp` | planned | M |
| 4 | Memoire QWEN.md + auto-memory + dream | memoire | contexte hierarchique + souvenirs + maintenance planifiee | `memory/writeContextFile.ts:68,125,265`, `recall.ts:21`, `dream.ts:17-184` | → `DefaultContextEngine` + nouveau `MemoryStore` | planned | L |
| 5 | Skills (activation conditionnelle + bundle) | contexte | markdown frontmatter, niveaux, activation par chemin persistante | `skills/skill-load.ts:22,119-140`, `skill-manager.ts:79`, `skill-manager.ts:572-613`, `skill-activation.ts:59-82` | `SkillStore` → `omega-context::SkillIndex` | **adopted** | S |
| 6 | Compaction auto calibree + microcompaction | contexte | deux couches, utils | `services/chatCompressionService.ts:58,91-97,124,443`, `microcompaction/microcompact.ts:14-15` | → `omega-context::ContextBudget` | planned | M |
| 7 | Budgets de surface (outils/rappels/exec) | contexte | budgets par outil, listing 8 000, `<system-reminder>` | `tools/tool-surface-budget.test.ts:67-140`, `core/environmentContext.ts:25-32` | → `TokenEstimator` + `ContextBudget` | planned | S |
| 8 | Generateurs de contenu pluggables + OAuth Qwen | routage | port unique ContentGenerator, adaptateurs, PKCE device-flow | `core/contentGenerator.ts:37-42,485-497`, `qwen/qwenOAuth2.ts:52-71` | `Provider` → `omega-providers` | **adopted** | M |
| 9 | Regles analytiques (danger + shell-semantics) | securite | interpreteur dangereux annule le read-only + AST shell | `permissions/dangerousRules.ts:125,233`, `shell-semantics.ts:1927` | `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy` | **adopted** | M |
| 10 | Mode AUTO : classifieur 2 etages | securite | LLM stage-1/stage-2 + fail-close + sanitization | `permissions/classifier.ts:135-303,326-328` | `TaskClassifier` + `PermissionPolicy` | planned | M |
| 11 | Approval modes + confirmation-bus unique | securite | DEFAULT/AUTO_EDITOR/YOLO + MessageBus indiffing | `config/approval-mode.ts:7-17`, `confirmation-bus/message-bus.ts:15` | `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | **adopted** | S |
| 12 | Sandboxes OS Landlock/BubbleWrap | execution | sandbox sans conteneur + backends local/docker/ssh | `sandbox/landlock-execution.ts:37,109-111`, `bwrap-execution.ts`, `services/*-execution-environment.ts` | `SandboxBackend` → `omega-sandbox` | **adopted** | M |
| 13 | Sous-agents frontmatter + backends | agents | `.md` + YAML, mineurs | `subagents/subagent-manager.ts:144-147,810-861`, `execution-backend.ts:12` | frontmatter → plugins/subagent-isolated | planned | M |
| 14 | Code Mode QuickJS | extension | execution code genere, hostCall batching | `code-mode/host.ts:66,273-279,446-456` | → `omega-sandbox::QuickJsBackend` | planned | L |
| 15 | Systeme de hooks (command/http/function) | extension | cgroups + ssrf + stopHookCap | `hooks/hookRegistry.ts:94`, `hookRunner.ts:652`, `hook-command-cgroup.ts`, `ssrfGuard.ts` | `Registre` (hook-bus) → `omega-plugin-api` | planned | M |
| 16 | Extensions marketplace + convertisseurs + network-policy | extension | install owner/repo + conversion Claude/Gemini/Qoder + HTTPS/DNS guard + zip-bomb | `extension/extensionManager.ts:501`, `marketplace.ts:112-141,419-421`, `network-policy.ts:111-141` | `Registre` → `omega-registry` | planned | M |
| 17 | Sessions + branch points + file history | execution | JSONL, writer-lease, edits reversible | `services/sessionService.ts:860`, `chatRecordingService.ts:142-160`, `branch-points.ts`, `fileHistoryService.ts` | `EventStore` + replay | planned | M |
| 18 | Config en couches + trust gate + safe-mode | configuration | settings global/projet, trust, QWEN.md safe | `config/storage.ts:46,210,644`, `config.ts:567,1308,5397` | `ConfigSource` → `omega-config` | **adopted** | S |
| 19 | Detection de boucle adaptative | execution | cut avant le seuil DashScope externe | `services/loopDetectionService.ts:329,690-785`, `core/stream-guards.ts` | garde deterministe (natif) | planned | S |
| 20 | Cron scheduler durable | automatisation | fichier + verrou + age-out one-shot | `services/cronScheduler.ts:248`, `cronTasksFile.ts` | omega-core scheduler | planned | M |
| 21 | Goal runtime (cartes + reducer + preuves) | agents | machine d'etats typée + provenance + verifier | `goals/goal-reducer.ts:105-112`, `goal-evidence.ts:117`, `goal-persistence.ts` | omega-core goal store | planned | M |
| 22 | Telemetrie OpenTelemetry full | observabilite | traces/metrics/logs GenAI + exporters fichier | `telemetry/sdk.ts:21-67`, `file-exporters.ts`, `sanitize.ts` | `TelemetrySink` → `omega-otel` | **adopted** | M |
| 23 | Stack live/desktop multimodale | ui | ECARTE : hors perimetre CLI Rust | packages/live-host | (ecarté) | rejected | L |

## 4. Traduction en plugins

- `ToolSurface → OCR ToolSurfaceIndex` (**adopted** #2) : check equitation directe, `scoreTool` est plus simple que notre BM25 + gate rarest-token ; on peut gagner la notion d'outils « deferres » sur la surface.
- `ConfigSource → omega-config` (**adopted** #18) : settings global/projet se mappent 1:1 sur nos couches + precedence ; ajouter un TrustGate d'entree de repertoire est clair (S).
- `PermissionPolicy → omega-sandbox::ApprovalQueue` (**adopted** #11) : l'enum d'approbation et le confirmation-bus se traduisent en file d'approbation existante ; le MessageBus est un bon modele si l'UI de cockpit doit fusionner plusieurs demandes.
- `ExecPolicy → omega-sandbox::RuleBasedExecPolicy` (**adopted** #9) : les regles `dangerous-interpreter` (`dangerousRules.ts:125: python/sh/node annule read-only`) et `extractShellOperations` enrichissent nos regles deterministes.
- `TelemetrySink → omega-otel::{Stdout,File}Sink` (**adopted** #22) : semconv GenAI (gen-ai-request/response + sanitize) est un enrichissement cible direct de notre sink fichier.

## 5. Consideres et retenir

- **Modes d'execution** : Code Mode via VM QuickJS (complet), notre preference differente (echange de perf pour la perte de surface), en mode Rust par `rquickjs` (M/L).
- **Memoire + skills** : QWEN.md hierarchie et auto-memory souvenirs (recal LLM-budgete). Le root `QWEN.md` global ~ context files est deja cible ; la partie souvenirs budgete un nouveau port `MemoryStore` (L).
- **Stack live/desktop** (audio, browser-use, computer-use, mobile) : rejete — c'est androids inopportun dans un harnais CLI Rust ; les outils `display-image` / `zoom-image` + `omni/disclosure.ts` couvrent deja l'utile multimodal dans notre grille.
- **Preuve REA** : la preuve `resource_constraint` a ceci — ce n'est pas la premiere fois (« The parent's result capacity was exhausted... ») ; c'est une limite de capacity la passage, pas un defaut du projet. Les points couverts par cette dissection proviennent de la lecture complete.

## 6. Sources

- Clone de reference : `/home/tayk0oo/omega-re/qwen-code` (v0.25.2, Apache-2.0 observe).
- Repo : https://github.com/QwenLM/qwen-code ; docs : https://qwenlm.github.io/qwen-code-docs/ (site `docs-site/` embarque).
- Preuve REA : `docs/omega/research/rea/evidence/intake-qwen-code.summary.json`.
