# Dissection — Goose (Block, Inc.)

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Block, Inc. (AAIF) |
| Licence | Apache-2.0 |
| Langage | **Rust** (workspace ~15 crates) + TypeScript (ui/) |
| Repo | https://github.com/block/goose |
| Version observee | 1.55.0 |
| Architecture | `goose-agent` (boucle d'agent pure : state machine + effets) sous `goose` (plateforme : extensions MCP, sessions, ACP, scheduler, permissions) ; `goose-providers` (~15 providers), `goose-cli`, `goose-sdk` (FFI), `ui/desktop` (Electron). Le serveur ACP local (`goose serve`) est le pivot : CLI, desktop et clients tiers s'y connectent. |

## 2. Entrees

| Entree | Chemin |
|---|---|
| CLI | `crates/goose-cli` (`session`, `recipe`, `schedule`, `configure`, `doctor`, `roam*`) |
| Serveur ACP | `crates/goose/src/acp/` (`server.rs`, `server_factory.rs`, `transport/`) |
| Client ACP / roaming | `crates/goose-cli/src/commands/roam_client.rs`, `ui/goose-acp*` |
| Desktop | `ui/desktop` (Electron + forge) |
| SDK embarquable | `crates/goose-sdk` (`bindings.rs`, `lib.rs`) |
| Gateway | `crates/goose/src/gateway/` (Telegram, headless, pairing) |
| Config | `crates/goose/src/config/` (`base.rs:120`, `extensions.rs`, `migrations.rs`, `declarative_providers.rs`) |
| Extensions | `crates/goose/src/agents/extension.rs:158` (`ExtensionConfig`), `extension_manager/` |
| Recettes | `crates/goose/src/recipe/mod.rs:43`, `goose-self-test.yaml` |
| Hooks | `crates/goose/src/hooks/mod.rs:54` |

## 3. Inventaire fonctionnel

| Fonctionnalite | Domaine | Ce que ca fait | Ou (file:line) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|
| Boucle d'agent a steps | agents | State machine de steps emettant des effets de conversation | `crates/goose-agent/src/machine.rs:55,121`, `operation.rs:226` | agent-runtime → AgentLoop (plugin) | planned | M |
| Subagents delegues bornes | agents | Delegation isolee avec budget, sans recursion | `crates/goose-agent/src/subagent.rs:165`, `subagent_handler.rs:29` | agent-runtime → SubagentHost (plugin) | planned | M |
| Compaction structuree | contexte | Resume des messages en schema lenient via le provider | `context_mgmt/mod.rs:87`, `structured.rs:14`, `summarize.rs:124` | context-engine → CompactionPolicy (plugin) | planned | M |
| Elagage algorithmique paires d'outils | contexte | Seuil proportionnel + protection du tour courant | `context_mgmt/mod.rs:383,393,461` | context-engine → CompactionPolicy (plugin) | planned | S |
| Extensions MCP declaratives | extension | stdio/SSE/streamable-http/builtin/platform en config | `agents/extension.rs:158,210,382` | extension-host → ExtensionHost (plugin) | planned | M |
| Gestionnaire d'extensions a leases | extension | Bail par session, enable/disable, proprietaire d'outil | `extension_manager/mod.rs:370,670`, `mcp_client.rs:167` | registry → Registre (plugin) | planned | M |
| Serveur ACP | execution | Agent servi en local via ACP HTTP/WebSocket | `acp/server.rs:313,2639`, `server_factory.rs:18` | agent-server → AcpServer (plugin) | planned | L |
| Client ACP + roaming | execution | Client interactif et delegation distante | `roam_client.rs:27,112` | agent-server → AcpClient (plugin) | planned | M |
| Multi-providers + epinglage | routage | ~15 providers, un epingle par session | `provider_manager.rs:18,59` | `omega-providers::OllamaProvider` | adopted | M |
| Recettes YAML | automatisation | Prompt + extensions + params versionnes | `recipe/mod.rs:43`, `recipes/recipe.rs:21` | skill-store → RecipeStore (plugin) | planned | M |
| Planificateur cron | automatisation | Jobs cron de recettes, pilotables par l'agent | `scheduler_trait.rs:9`, `scheduler/full.rs:108` | service interne | planned | M |
| Hooks d'evenements | extension | Pre/PostToolUse, SessionStart/End, shell, stop | `hooks/mod.rs:54,349,423` | agent-runtime → HookRegistry (plugin) | planned | S |
| Sessions persistantes | memoire | CRUD + nommage + recherche d'historique | `session_manager.rs:348,452`, `chat_history_search.rs:81` | `omega-app::FileEventStore` | adopted | M |
| Config en couches | configuration | Fichier + secrets OS + migrations | `config/base.rs:120,737` | `omega-config` | adopted | S |
| Permissions persistantes | securite | Allow/deny memorises + annotations read-only | `permission_store.rs:23,79`, `permission_inspector.rs:52` | `omega-sandbox::ApprovalQueue` | adopted | S |
| Juge LLM read-only | securite | Un modele tranche l'absence d'effet de bord | `permission_judge.rs:145` | permission-policy → PermissionPolicy (plugin) | planned | M |
| Telemetrie OTLP | observabilite | Export OTLP + propagation W3C + GenAI | `otel/otlp.rs:300,581`, `trace_context.rs:15` | `omega-otel::StdoutSink` | adopted | S |
| Docteur auto-diagnostic | observabilite | Verifie l'environnement, propose des correctifs | `doctor.rs:22`, `diagnostics.rs:297` | natif | planned | S |
| Desktop Electron | ui | Client desktop packagé multi-OS | `ui/desktop/forge.config.ts` | — (rejected) | rejected | L |

## 4. Traduction en plugins

- `go-agent-loop` : agent-runtime → AgentLoop → (nouveau) — la state machine a steps est le modele de boucle a reprendre ; `ConversationEffect` prefigure des effets typés.
- `go-subagents` : agent-runtime → SubagentHost → (nouveau) — delegation bornee (pas de recursion), reporting par notifications.
- `go-compaction` : context-engine → CompactionPolicy → (nouveau) — `StructuredSummary` lenient (intent, files, next_step) + preservation du dernier message user.
- `go-tool-pruning` : context-engine → CompactionPolicy → (nouveau) — `compute_tool_call_cutoff` + `tool_ids_to_summarize` : elagage proportionnel a la limite, tour courant protege.
- `go-ext-config` : extension-host → ExtensionHost → (nouveau) — `ExtensionConfig` multi-transports avec secrets via Config.
- `go-ext-manager` : registry → Registre → `omega-registry` — leases par session + `get_tool_owner`, avec quarantaine.
- `go-acp-server` : agent-server → AcpServer → (nouveau) — `goose serve` comme patron du serveur local multi-clients.
- `go-acp-client` : agent-server → AcpClient → (nouveau) — symetrique du serveur, roaming delegue.
- `go-providers` : llm-provider → Provider → `omega-providers::OllamaProvider` — epinglage par session (`ProviderManager::pinned`).
- `go-recipes` : skill-store → RecipeStore → (nouveau) — YAML `prompt + extensions + parameters + response schema`.
- `go-scheduler` : service interne cron → (nouveau) — `SchedulerTrait`, exposé au modele via `schedule_tool`.
- `go-hooks` : agent-runtime → HookRegistry → (nouveau) — 12 evenements, decisions d'interception.
- `go-sessions` : event-store → EventStore → `omega-app::FileEventStore` — persistance + nommage + recherche.
- `go-config` : config-source → ConfigSource → `omega-config` — couches + secrets + migrations.
- `go-permissions` : permission-policy → PermissionPolicy → `omega-sandbox::ApprovalQueue` — store avec expiration + annotations.
- `go-perm-judge` : permission-policy → PermissionPolicy → (nouveau, heuristique d'abord, LLM en option).
- `go-otel` : telemetry-sink → TelemetrySink → `omega-otel::StdoutSink` — OTLP + W3C + GenAI.
- `go-doctor` : natif — auto-diagnostic actionnable.

## 5. Ecarte

- **Desktop Electron (`go-desktop`, rejected)** : conteneur de packaging avec son cycle de release ; Omega sert son cockpit React via `omega-server` sur une seule origine. On reprend les vues (sessions, approbations), pas Electron.
- **Gateway Telegram/headless** : transport de notification/appairage hors scope du harnais (releve mais non repris en l'etat ; un connecteur messagerie reste possible plus tard comme service separe).
- **Distributions personnalisees (CUSTOM_DISTROS.md)** : guide de white-labelling (branding, preconfiguration), pas une capacite de code ; le besoin est couvert par config en couches + recettes.
- **« Context Revision » find-and-replace/ripgrep** : non observe dans le code. La compaction Goose = resume LLM (`compact_messages`, `StructuredSummary`) + elagage algorithmique (`compute_tool_call_cutoff`, `summarize_tool_call`). Aucune revision chirurgicale par ripgrep ; l'hypothese de la perspective est infirmee — documente tel quel.

## 6. Sources

- Repo : https://github.com/block/goose (clone `--depth 1`, version 1.55.0, repertoire `/home/tayk0oo/omega-re/goose`)
- Docs : `CUSTOM_DISTROS.md`, `docs/omega/reverse-eng/DISSECTION-SCHEMA.md`
- Fichiers cles : `crates/goose-agent/src/machine.rs`, `operation.rs`, `subagent.rs` ; `crates/goose/src/context_mgmt/mod.rs`, `agents/extension.rs`, `agents/extension_manager/mod.rs`, `agents/mcp_client.rs`, `acp/server.rs`, `acp/server_factory.rs`, `recipe/mod.rs`, `scheduler_trait.rs`, `scheduler/full.rs`, `hooks/mod.rs`, `session/session_manager.rs`, `config/base.rs`, `permission/`, `otel/`, `doctor.rs`, `gateway/manager.rs` ; `crates/goose-context-management/src/` ; `crates/goose-cli/src/commands/roam_client.rs`, `recipes/recipe.rs` ; `goose-self-test.yaml`.
