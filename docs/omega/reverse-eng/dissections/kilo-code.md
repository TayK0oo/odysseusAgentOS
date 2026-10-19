# Dissection — Kilo Code

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Kilo Org |
| Licence | MIT (verifie dans le clone, `LICENSE:1-3` — « Copyright (c) 2026 Kilo Code / (c) 2025 opencode ») |
| Langage | TypeScript (Bun 1.4.2 + Turbo, `package.json:6`) ; Kotlin pour le plugin JetBrains |
| Etoiles | 27 627 |
| Repo | https://github.com/Kilo-Org/kilocode |
| Clone local (lecture seule) | `/home/tayk0oo/omega-re/kilo-code` |
| Architecture | Monorepo `packages/*` fusionnant deux lignages : le runtime d'agent **opencode v2** (`packages/core`, `packages/schema`, `packages/opencode`, `packages/protocol`, `packages/server` — contrats `effect/Schema`, events durables, serveur `HttpApi`, TUI) et l'heritage **Cline/Roo** (`packages/kilo-vscode` — extension VS Code, Agent Manager a git worktrees, modes/groupes, approbations, checkpoints). Autour, des briques propres a Kilo : `kilo-memory` (memoire projet), `kilo-indexing` (RAG codebase tree-sitter + LanceDB/Qdrant), `kilo-sandbox` (bubblewrap/seatbelt + proxy reseau), `kilo-gateway` (auth device + provider OpenRouter + FIM), `kilo-telemetry`, `kilo-i18n` (20 langues), surfaces `kilo-ui` / `kilo-web-ui` / `kilo-console` / `kilo-jetbrains`. |

## 2. Entrees

- **Binaires / CLI** (`packages/opencode/src/cli/cmd`) : `run` (+ `run-stdin`, transport `run/stream.ts`), `serve`, `attach`, `remote`, `acp`, `pr` (checkout/link/unlink/status), `github` (install/run), `mcp`, `plug`, `agent`, `session`, `import`, `export`, `stats`, `models`, `providers`, `upgrade`, `uninstall`, `tui`. Script racine : `KILO_CLIENT=cli bun run --cwd packages/opencode src/index.ts` (`package.json:8`).
- **Extension VS Code** : `packages/kilo-vscode` (esbuild + `webview-ui`), point d'entree `src/extension.ts`, pont unique `src/KiloProvider.ts`.
- **Serveur HTTP** : `packages/opencode/src/server`, API generee par groupes (`server/routes/instance/httpapi/groups/*`), SDK genere dans `packages/sdk` / `packages/sdk-next`, codegen `packages/httpapi-codegen`, rejeu de requetes `packages/http-recorder`.
- **Editeurs tiers** : client **ACP** (`packages/opencode/src/acp`) pour Zed/JetBrains ; plugin JetBrains natif (`packages/kilo-jetbrains`, Kotlin `backend`/`frontend`) ; console web `packages/kilo-console` (React/Vite).
- **Points d'extension** : Plugins v2 (`PluginV2.Service.add/remove/wait`, `packages/core/src/plugin.ts:1-60`) avec host par domaine (`packages/core/src/plugin/host.ts`) ; CLI `plug` (`cli/cmd/plug.ts:1-40`) ; skills (`packages/core/src/skill.ts`) ; Marketplace (`packages/opencode/src/kilocode/marketplace/`) ; MCP (`packages/core/src/config/mcp.ts`) ; agents declaratifs (`packages/opencode/src/kilocode/agent/builder.ts`).
- **Fichiers de config** : `kilo.json` / `kilo.jsonc` / `opencode.json(c)` ; `AGENTS.md`, `CLAUDE.md`, `CONTEXT.md` (`session/instruction.ts:63-90`) ; `.kilo/rules`, `.kilocode/rules`, `.kilo/rules-{mode}` (`rules-migrator.ts:11-90`) ; `.kilo/command/*.md`, `.kilo/agent/*.md` ; `.kilo/agent-manager.json` (etat des worktrees) ; `.kilo/` est la cible recommandee, `.kilocode/` et `.opencode/` sont explicitement deconseilles pour les nouveaux fichiers (`kilocode/system-prompt.ts:29`).

## 3. Inventaire fonctionnel

| Fonctionnalite | Domaine | Ce que ca fait | Ou (file:line) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|
| System Context Registry + Context Epoch | contexte | Sources de contexte typees rendues en baseline immuable par epoch, changements via message systeme chronologique | `CONTEXT.md:1-40`, `packages/core/src/session/context-epoch.ts` | ContextEngine → `omega-context::DefaultContextEngine` | adopted | M |
| Compaction + historique durable / promotion de prompt | contexte | Compaction, promotion durable de l'input admis, drain de session, sortie d'outil bornee | `packages/schema/src/kilocode/session-drain.ts:1-14`, `packages/core/src/session/projector.ts:400-480` | ContextBudget → `omega-context::ContextBudget` | planned | M |
| Index semantique du codebase | contexte | Tree-sitter + embeddings multi-fournisseurs + LanceDB/Qdrant + watcher + overlay worktree | `packages/kilo-indexing/src/indexing/manager.ts:33-70`, `indexing/constants/index.ts:1-60` | ContextEngine (retrieval) → `omega-context` | planned | L |
| Skills (sources locale/URL/embedded, permission) | contexte | Divulgation progressive + garde-fous anti-traversee + filtrage par permission | `packages/core/src/skill.ts:1-40`, `packages/core/src/skill/discovery.ts:1-60` | SkillStore → `omega-context::SkillIndex` | adopted | M |
| Chaine d'instructions + migration des regles | configuration | AGENTS.md/CLAUDE.md/CONTEXT.md multicouche, import `.kilocode/rules` | `packages/opencode/src/session/instruction.ts:63-90`, `kilocode/rules-migrator.ts:11-90` | ConfigSource → `omega-config` | planned | S |
| Memoire projet (capture selective + consolidation) | memoire | 3 sources, topics par regle, throttling, quotas, redaction de secrets | `packages/kilo-memory/src/schema.ts:9-82`, `capture/redact.ts:2-42` | MemoryStore → `plugins/memory-store` | planned | L |
| Inject d'index memoire + outils recall/save | memoire | Bloc ```kilo-memory-v1 context_not_instruction, 4 modes de recall, asks permissionnes | `packages/kilo-memory/src/recall/budget.ts:1-40`, `tool.ts:15-55` | MemoryStore (index/inject) → `plugins/memory-store` | planned | M |
| Surface d'outils builtins + registre scoped | outils | 12 outils shipped composes par couches, registre dynamique separe, sortie bornee | `packages/core/src/tool/builtins.ts:1-45`, `application-tools.ts:1-55` | ToolSurface → `omega-context::ToolSurfaceIndex` | adopted | M |
| Navigateur pilote (Playwright CDP) + politique d'egress | outils | Navigation/clic/capture, devtools, reseau, streaming ; HTTPS public ou localhost uniquement | `services/browser-automation/browser-broker.ts:246-320`, `browser-policy.ts:1-31` | ToolSurface (browser tool) → `plugins/browser-cdp` | planned | L |
| Pont MCP local/remote + OAuth + warmup | outils | Serveurs locaux/distants, timeouts distincts, OAuth, prechauffage | `packages/core/src/config/mcp.ts:1-60`, `mcp-oauth.ts:1-30` | ToolSurface (MCP bridge) → `plugins/mcp-bridge` | planned | M |
| Autocompletion inline FIM + chat-autocomplete | outils | Codestral/Inception/passerelle Kilo, modele dedie | `packages/kilo-gateway/src/fim.ts:1-40` | CompletionProvider → `plugins/autocomplete-fim` | planned | M |
| Regles allow/ask/deny + groupes de modes | securite | Ruleset ordonne, mapping groupes→permissions, `fileRegex`, deny par defaut | `packages/schema/src/permission.ts:25-65`, `kilocode/modes-migrator.ts:38-90` | PermissionPolicy → `omega-sandbox::ApprovalQueue` | adopted | S |
| Provenance de la decision d'approbation | securite | Marqueur runtime `source` (agent/global/project/yolo/session/manual/default) sur la regle gagnante | `kilocode/permission/provenance.ts:1-40` | Provenance → `omega-otel::Provenance` | adopted | S |
| Bac a sable processus + reseau proxy egress | securite | bwrap/seatbelt, profil FS et reseau, proxy TLS SNI, destinations validees | `kilo-sandbox/src/backend.ts:1-49`, `profile.ts:1-34`, `proxy.ts:13-90` | SandboxBackend → `omega-sandbox::LocalSandboxBackend` | adopted | M |
| Profils d'execution (HITL/autonomous) + bash allow-list + garde de cout | execution | Presets qui ecrivent permissions et affichage, auto-approve, alerte de cout non bloquante | `shared/work-style-presets.ts:1-90`, `kilo-provider/auto-approve.ts:1-25` | ExecPolicy → `omega-sandbox::RuleBasedExecPolicy` | adopted | S |
| Sous-agents via l'outil `task` | agents | Ruleset derive (heritage deny + external_directory, deny task/todowrite par defaut) | `packages/opencode/src/agent/subagent-permissions.ts:1-30` | SubagentRunner → `plugins/subagent-isolated` | planned | M |
| Agent manager : un git worktree par session/groupe | agents | Etat persistant decouple, sections/pins, scripts de setup, suivi git/PR, cycle de vie | `agent-manager/WorktreeStateManager.ts:1-70`, `orchestration-domain.ts:1-60` | WorkspaceManager → `plugins/agent-manager-worktree` | planned | L |
| Agents declaratifs + modes d'organisation | agents | Markdown (`scope`, `mode`, `tools`, `permission`, `steps`) + surcharge cloud | `kilocode/agent/builder.ts:1-60`, `kilo-gateway/src/api/modes.ts:1-80` | AgentStore → `plugins/agent-declarative` | planned | S |
| Tableau de sessions (messagerie inter-sessions) | agents | Board SQL par session racine, messages/reponses, pagination par revision | `kilocode/board/store.ts:1-30`, `kilo-provider/session-board.ts:1-40` | BoardStore → `plugins/session-board` | planned | M |
| Sessions planifiees (wakeups) + drain | automatisation | `session.wakeup` avec `pending`, statut `scheduled` + `scheduledAt` | `packages/schema/src/kilocode/wakeup-event.ts:1-20` | Scheduler → `plugins/scheduler-wakeup` | planned | M |
| Retour arriere par snapshots git (checkpoints) | memoire | Snapshots content-addresses, plan par message, diff/restauration par fichier | `packages/core/src/snapshot.ts:35-90`, `session/revert.ts:31-100` | SnapshotStore → journal + restauration | planned | M |
| CLI headless / serve / remote / pr / github | execution | Un runtime partage entre CLI, extension et serveur | `cli/cmd/plug.ts:1-40`, `cli/cmd/pr.ts:1-50` | omega-app (CLI) + omega-server | planned | M |
| Serveur HttpApi par groupes + SDK + surfaces | execution | 21+ groupes d'API, SDK genere, TUI/console/web/JetBrains | `server/routes/instance/httpapi/groups/session.ts` | omega-server | planned | L |
| Agent Client Protocol (editeurs tiers) | extension | Sessions, modes, modeles, permissions, usage via ACP | `packages/opencode/src/acp/service.ts:1-60` | omega-server (surface editeur) | planned | M |
| Marketplace MCP/skills/agents/plugins | extension | Installation validee (deja-installe, evasion de chemin bloquee, compagnons) | `kilocode/marketplace/installer.ts:1-240` | Registre (install/remove) → `plugins/marketplace` | planned | M |
| Plugins v2 (add/remove/wait + host par domaine) | extension | Services Effect ajoutes/retires, echec isole, contributions par registre | `packages/core/src/plugin.ts:1-60` | Registre → `omega-registry` | adopted | M |
| Catalogue de telemetrie produit | observabilite | ~24 evenements (CLI, session, LLM, outils, index, MCP, auth, erreurs) + opt-out | `packages/kilo-telemetry/src/events.ts:1-51` | TelemetrySink → `omega-otel::{Stdout,File}Sink` | adopted | S |
| Events durables vs ephemères + manifeste | observabilite | `durable{version, aggregate}`, union typee, projection en base | `packages/schema/src/event.ts:1-60`, `durable-event-manifest.ts:1-20` | EventStore → `omega-app::FileEventStore` | adopted | M |
| Comptabilite par run (tokens/cout/issue) | observabilite | Agregation par run d'agent, snapshots d'usage par fournisseur | `kilocode/agent-usage/index.ts:1-45`, `kilocode/provider-usage.ts:1-45` | CostLedger → sinks + omega-router | planned | S |
| Catalogue multi-fournisseurs rejouable | routage | Catalogue mutable par transforms, couts cache, 500+ modeles, usage reel | `packages/core/src/catalog.ts:1-55`, `plugin/models-dev.ts` | Provider → `omega-providers::OllamaProvider` | adopted | M |
| Passerelle Kilo (compte, solde, modes, sessions cloud) | routage | **Ecarte** : backend SaaS operateur | `kilo-gateway/src/auth/device-auth.ts:30-98`, `provider.ts:1-78` | — | rejected | L |

31 fonctionnalites, 11 domaines, 11 `adopted` / 19 `planned` / 1 `rejected`.

## 4. Traduction en plugins

Chaque ligne est le triplet `capacite → port → adaptateur` plus le mode d'integration.

**Adopte (le port et l'adaptateur existent deja chez nous — voir table d'honnetete du schema) :**

- `context-engine → ContextEngine → omega-context::DefaultContextEngine` (plugin) : le vocabulaire *Context Source / Epoch / Snapshot* de `CONTEXT.md` correspond exactement a ce que fait `DefaultContextEngine` (baseline + mises a jour) ; ce qui manque est le *renderer de retrait* et la notion d'epoch comme frontiere de cache.
- `skill-store → SkillStore → omega-context::SkillIndex` (plugin) : `SkillIndex` fait deja decouverte + index ; on lui ajoute les trois sources (`directory`/`url`/`embedded`) et `isSafeRelativePath` comme garde-fou de traversee.
- `tool-surface → ToolSurface → omega-context::ToolSurfaceIndex` (plugin) : la liste builtin de Kilo est un `Layer` compose (pas un tableau en dur) ; on reprend la separation *builtins statiques* / *registre dynamique MCP+plugins* et la limite de taille de sortie avec debordement fichier.
- `permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue` (plugin) : `Rule{action, resource, effect}` ordonnee + `Reply ∈ {once, always, reject}` s'ajoutent tels quels ; le mapping *groupes de mode → permissions* et le `deny` explicite des permissions absentes sont des regles pures, sans etat.
- `provenance → Provenance → omega-otel::Provenance` (plugin) : le marqueur runtime-only `source` sur la regle gagnante est l'equivalent exact d'un tag de provenance non persiste — a brancher directement.
- `sandbox-backend → SandboxBackend → omega-sandbox::LocalSandboxBackend` (plugin) : `Profile{filesystem, network{allow|deny|proxy}, environment}` est le contrat d'entree de nos backends ; on ajoute le **proxy egress avec inspection TLS ClientHello** et la verification de destination (`isPublicAddress`) comme politiques.
- `exec-policy → ExecPolicy → omega-sandbox::RuleBasedExecPolicy` (plugin) : les presets *human-in-the-loop / autonomous* sont une table de regles ; la allow-list bash (lecture seule) et le `"*>*": "ask"` se transcrivent en regles, et l'alerte de cout est une politique non bloquante.
- `registry → Registre → omega-registry` (plugin) : `PluginV2.Service.add/remove/wait` + echec isole + host par domaine = exactement le regime *Slot/Ensemble + quarantaine*.
- `telemetry-sink → TelemetrySink → omega-otel::{Stdout,File}Sink` (plugin) : le catalogue d'evenements ferme (24 types) fournit la taxonomie a implementer sur nos sinks.
- `event-store → EventStore → omega-app::FileEventStore` (plugin) : `durable{version, aggregate}` + manifeste + projection = journal d'agregat versionne, deja en place (rejeu via `omega-otel::TraceReplayer`).
- `llm-provider → Provider → omega-providers::OllamaProvider` (plugin) : le `Catalog` rejouable (transforms, couts cache, `PolicyActions=["provider.use"]`) alimente notre registre fournisseurs.

**Planifie (capacite nouvelle, pas encore de port/adaptateur) :**

- `memory-store → MemoryStore → plugins/memory-store` : capture selective, consolidation **par regles** (jamais par LLM), quotas, redaction. Aucun equivalent chez nous : c'est le trou le plus net, et la brique la plus rentable.
- `completion-provider → CompletionProvider → plugins/autocomplete-fim` : FIM multi-destinations (passerelle / Codestral / Inception).
- `subagent-runner → SubagentRunner → plugins/subagent-isolated` : la *derivation de ruleset* par sous-agent (heritage des seuls deny + external_directory, recursion opt-in) est une politique a copier, pas un moteur.
- `workspace-manager → WorkspaceManager → plugins/agent-manager-worktree` : worktree par session/groupe, sections/pins, scripts de setup, cycle de vie, suivi PR.
- `agent-store → AgentStore → plugins/agent-declarative` : agents/modes en markdown, fusion d'un sur-ensemble distants.
- `session-board → BoardStore → plugins/session-board` : journal inter-sessions ancre sur une session racine.
- `scheduler-job → Scheduler → plugins/scheduler-wakeup` : echeance + `pending` + statut `scheduled`.
- `snapshot-store → SnapshotStore` : snapshots content-addresses + `stage/clear/commit`, avec les etats d'indisponibilite (`snapshots-disabled`, `not-a-git-repo`) a conserver tels quels dans le contrat.
- `cost-accounting → CostLedger` : agregation par run (tokens dont cache read/write, issue) — pilier *economie* d'Omega.
- `registry → Registre → plugins/marketplace` : installation validee d'MCP/skills/agents/plugins.
- `tool-surface → ToolSurface → plugins/mcp-bridge` et `plugins/browser-cdp` : deux fournisseurs d'outils a brancher sur le meme port.

**Services (hors noyau, derriere un port/une origine) :** `omega-app` (CLI headless : `run`, `serve`, `attach`, `remote`, `pr`, `github`, `plug`), `omega-server` (HttpApi par groupes + SDK genere + surfaces), et le client **ACP** pour les editeurs non-VS Code. A noter : le serveur Kilo expose 21+ groupes d'API generes depuis les schemas — c'est le modele a suivre pour `/api/health|plugins|run|trace` (une route par groupe, SDK genere, jamais d'API ecrite a la main).

**Modes d'integration utilises :** `plugin` 27, `service` 3, `rejected` 1.

## 5. Ecarte

- **Passerelle Kilo / cloud (`rejected`)** : `kilo-gateway` est un backend **SaaS operateur** (kilo.ai) : auth par device code, provider OpenRouter re-emballe, solde/profil, modes d'organisation, import de sessions cloud, lien PR, WebSocket distant (`kilo-sessions/kilo-sessions.ts:1-50`). Incompatible avec l'exigence self-hosted et sans fournisseur unique. Les *briques client* restent reutilisables et sont deja couvertes ailleurs : validation par schema d'un export de session (→ notre journal durable), device-auth generique (→ notre boucle d'auth locale), synchronisation de statut (→ `TelemetrySink`).
- **Modes d'organisation imposes par le cloud** : le *format* (`groups` avec `fileRegex`) est repris, mais **pas** la source de verite distante : chez nous les modes sont des agents locaux declaratifs (`plugins/agent-declarative`), sans aller-retour reseau obligatoire.
- **`kilo-local-recall` (transcript local) et `kilo_memory_*` en tant qu'outils nommes** : on reprend la capacite, pas le nommage ni le couplage a la memoire projet — l'outil devient une contribution au `ToolSurface`.
- **Runtime JS/TS in-process** la ou il n'a pas d'equivalent Rust : l'orchestration d'indexation, les embedders et Playwright restent des **services** hors noyau ; seuls les contrats (port + schema d'entree/sortie) sont portes dans Omega.
- **Non repris (hors perimetre fonctionnel)** : `kilo-jetbrains` (plugin Kotlin natif), `kilo-console` / `kilo-web-ui` (surfaces proprietaires), `kilo-i18n` (20 langues — utile mais non prioritaire), `sqd`/`anaconda-desktop`, `speech-to-text`, `image-generation`, `cmp`-style actions GitHub.

## 6. Sources

- Code (clone lecture seule, `/home/tayk0oo/omega-re/kilo-code`) :
  - `LICENSE:1-3`, `README.md:24,49,121,144`, `package.json:6-8`, `CONTEXT.md:1-40`
  - `packages/schema/src/` : `event.ts`, `durable-event-manifest.ts`, `permission.ts`, `plugin.ts`, `revert.ts`, `skill.ts`, `kilocode/wakeup-event.ts`, `kilocode/session-drain.ts`
  - `packages/core/src/` : `catalog.ts`, `plugin.ts`, `plugin/host.ts`, `snapshot.ts`, `skill.ts`, `skill/discovery.ts`, `session/revert.ts`, `session/projector.ts`, `session/message-updater.ts`, `config/mcp.ts`, `tool/builtins.ts`, `tool/application-tools.ts`, `kilocode/agent-usage/index.ts`, `kilocode/provider-usage.ts`, `kilocode/cost/max-cost-nudge.ts`
  - `packages/opencode/src/` : `session/instruction.ts`, `agent/subagent-permissions.ts`, `kilocode/system-prompt.ts`, `kilocode/modes-migrator.ts`, `kilocode/rules-migrator.ts`, `kilocode/permission/provenance.ts`, `kilocode/board/store.ts`, `kilocode/agent/builder.ts`, `kilocode/marketplace/installer.ts`, `kilo-sessions/kilo-sessions.ts`, `acp/service.ts`, `cli/cmd/*`
  - `packages/kilo-memory/src/` : `schema.ts`, `tool.ts`, `memory.ts`, `capture/redact.ts`, `capture/operations.ts`, `recall/budget.ts`, `recall/indexer.ts`
  - `packages/kilo-sandbox/src/` : `backend.ts`, `profile.ts`, `network.ts`, `proxy.ts`, `destination.ts`, `bubblewrap.ts`, `seatbelt.ts`
  - `packages/kilo-indexing/src/indexing/` : `manager.ts`, `constants/index.ts`, `embedders/*`, `vector-store/*`, `processors/file-watcher.ts`, `worktree-overlay.ts`
  - `packages/kilo-gateway/src/` : `fim.ts`, `provider.ts`, `api/modes.ts`, `auth/device-auth.ts`, `cloud-sessions.ts`
  - `packages/kilo-vscode/src/` : `KiloProvider.ts`, `shared/work-style-presets.ts`, `kilo-provider/auto-approve.ts`, `kilo-provider/session-board.ts`, `agent-manager/WorktreeStateManager.ts`, `agent-manager/SetupScriptRunner.ts`, `agent-manager/orchestration-domain.ts`, `services/browser-automation/*`, `services/telemetry/types.ts`
  - `packages/kilo-telemetry/src/events.ts`
- Doc officielle : https://kilo.ai/docs · https://kilo.ai/docs/code-with-ai/agents/using-agents · https://kilo.ai/cli · https://github.com/Kilo-Org/kilocode
- Preuve machine REA : `docs/omega/research/rea/evidence/intake-kilo-code.summary.json` — `provider: rea-javascript-application`, `operation: analyze_javascript_application`, `confidence: derived`, `digest.sha256: 8b13d032…`, graphe de 24 843 nœuds / 26 397 aretes, **11 687 limitations**. **Limites de cette preuve** : le resume ne contient qu'un digest, des compteurs de graphe et un compte de limitations — aucun fait par fonctionnalite, aucune ligne source. Elle corrobore donc seulement l'identite (id, depot, langage, licence MIT, nature JS), le volume du depot et le fait que l'analyse est partielle ; **toutes** les affirmations de cette dissection viennent de la lecture directe du clone (`fichier:ligne`) et non de la preuve REA.
