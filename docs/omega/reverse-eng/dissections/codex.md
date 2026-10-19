# Dissection — Codex CLI (`openai/codex`)

> Corpus : `/home/tayk0oo/omega-re/codex` @ `e790ce2` (clone depth-1, 2026-10-16).
> Provenance : `[observed]` = lu dans le code clone · `[stated]` = README/docs amont · `[inferred]` = deduction.
> Fiche existante : `docs/omega/reverse-eng/RE-GOOSE-CODEX.md` (T3+T4, corpus precedent @ `1ffa95e`).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | OpenAI |
| Licence | Apache-2.0 |
| Langage | **Rust** (workspace `codex-rs`, ~127 crates) + TypeScript (CLI legacy, SDK, UI) |
| Repo | https://github.com/openai/codex |
| Architecture | Un crate par capacite autour d'un noyau session (`core`, `thread-store`, `rollout`) ; extensibilite par ~15 traits contributeurs dans un registre immuable ; frontiere Rust/TS generee par `ts-rs`. |

## 2. Entrees

| Entree | Code | Config / extension |
|---|---|---|
| `codex` CLI | `codex-rs/cli/src/` | `~/.codex/config.toml`, profils nommes, `~/.codex/<nom>.config.toml` (profile-v2) |
| `codex` TUI | `codex-rs/tui/src/` | — |
| app-server JSON-RPC (TUI/IDE) | `codex-rs/app-server/`, protocole `codex-rs/app-server-protocol/` | transport `app-server-transport`, daemon `app-server-daemon` |
| exec-server distant | `codex-rs/exec-server/`, `exec-server-protocol` | admission locale + RPC |
| Points d'extension | `codex-rs/ext/*` (16 crates : skills, memories, goal, guardian-v2, guardian-reviewer, queue, mcp, web-search, image-generation, git-attribution, history-notes, items, agent-message-board, connectors, extension-api, + `core-plugins`) | `pub fn install(builder)` par extension |
| Hooks declaratifs | `codex-rs/hooks/` (types + `engine/` par handler) | matchers d'evenements, `HookSource`, `HookTrustStatus` |
| Skills | `codex-rs/skills/` (loading, interface, selection, mentions) | `SKILL.md`, snapshots, `~/.agents/skills` + projet |
| Memoires | `codex-rs/memories/{read,write}` | modeles preferes par usage |
| Sous-agents / roles | `codex-rs/agent-roles/` (TOML `deny_unknown_fields`) | `description`, `config_file`, `nickname_candidates` |
| Instructions projet | `codex-rs/core/src/agents_md.rs` | `AGENTS.md` remontant + `AGENTS.override.md` |

## 3. Inventaire fonctionnel

| Fonctionnalite | Domaine | Ce que ca fait | Ou (file:line) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|
| Registre d'extensions a contributeurs | extension | ~15 traits fins, builder puis registre immuable, `install()` par crate `ext/*` | `ext/extension-api/src/contributors.rs:84`, `ext/extension-api/src/registry.rs:21` | registry → `omega-registry` | adopted | L |
| Config TOML a couches + provenance | configuration | 9 sources, `precedence() -> i16`, merge recursif avec alias | `config/src/config_layer_source.rs:5`, `config/src/config_layer_source.rs:31`, `config/src/merge.rs:57` | config-source → `omega-config` | adopted | M |
| Profils nommes | configuration | Unites (modele, provider, approbations, sandbox) validees strictement, schema genere | `config/src/profile_toml.rs:24`, `config-schema/src/main.rs:1` | config-source → `omega-config` | adopted | S |
| Hooks declaratifs abortables | automatisation | 12 evenements, 4 handlers, sync/async, thread/turn, `HookResult` abortable | `hooks/src/types.rs:15`, `protocol/src/protocol.rs:1693` | hook → `plugins/hook-command` | planned | M |
| Contrat d'outil spec/handle | outils | `spec` vs `handle`, exposition Direct/Deferred, `search_info` | `tools/src/tool_executor.rs:51`, `tools/src/tool_executor.rs:106` | tool-surface → `omega-context::ToolSurfaceIndex` | adopted | M |
| Sandbox multi-OS typee | execution | `SandboxPolicy` + `WritableRoot` (.git protege), Seatbelt/Landlock+bwrap/MXC | `protocol/src/protocol.rs:1183`, `sandboxing/src/lib.rs:19` | sandbox-backend → `omega-sandbox::LocalSandboxBackend` | adopted | L |
| Approbations granulaires | securite | `AskForApproval` par categorie, chaine `decide()`, porte `TurnStartAdmission` | `protocol/src/protocol.rs:1097`, `ext/extension-api/src/contributors.rs:413` | permission-policy → `omega-sandbox::ApprovalQueue` | adopted | M |
| Execpolicy (DSL) | securite | Prefixes + reseau → Allow/Prompt/Forbidden, deterministe, amendable | `execpolicy/src/decision.rs:9`, `execpolicy/src/rule.rs:111`, `execpolicy/src/policy.rs:28` | exec-policy → `omega-sandbox::RuleBasedExecPolicy` | adopted | M |
| OTel normalisee | observabilite | Constantes `codex.<domaine>.<objet>`, OTLP + stdout | `otel/src/metrics/names.rs:3`, `otel/src/metrics/names.rs:63` | telemetry-sink → `omega-otel::{StdoutSink,FileSink}` | adopted | S |
| Gating PII log-only/trace-safe | observabilite | Seules les cibles trace-safe sont exportees | `otel/src/targets.rs:1` | — → `omega-otel::Provenance` | adopted | S |
| Rollout rejouable | observabilite | `RolloutRecorder` + traces par phase + thread-store | `rollout/src/recorder.rs:86`, `rollout-trace/src/compaction.rs:35` | event-store → `omega-otel::TraceReplayer` | adopted | M |
| Compaction budgetee | contexte | Resume par modele, budget tokens, hooks Pre/PostCompact | `core/src/compact.rs:85`, `core/src/compact.rs:155` | context-engine → `omega-context::DefaultContextEngine` | adopted | M |
| AGENTS.md remontant + trust | contexte | Concatenation racine→cwd, override local, court-circuit si non fiable | `core/src/agents_md.rs:43` | context-engine → `plugins/agents-md-merge` | planned | S |
| Memoires locales | memoire | Lecture citee / ecriture gardee, modeles dedies extraction | `ext/memories/src/extension.rs:163` | memory-store → `plugins/memory-local` | planned | M |
| Port provider + fabrique | routage | `ModelProvider`, `create_model_provider`, catalogue | `model-provider/src/provider.rs:108`, `model-provider/src/provider.rs:336` | llm-provider → `omega-providers::OllamaProvider` | adopted | M |
| Sous-agents par roles | agents | Roles TOML stricts, cycle SubagentStart/Stop trace | `agent-roles/src/agent_role_config.rs:10` | subagent-runner → `plugins/subagent-isolated` | planned | M |
| Skills snapshottees | extension | Cache par snapshot, interface declaree, suivi d'invocation | `skills/src/loading.rs:24`, `skills/src/interface.rs:15` | skill-store → `omega-context::SkillIndex` | adopted | S |
| Flags runtime versionnes | configuration | `Stage` + registre `FEATURES`, de couple des features Cargo | `features/src/lib.rs:48` | config-source → `omega-config` | planned | S |
| Protocole app-server genere | ui | Types TS generes (`ts-rs`), fixtures de schema en CI | `app-server-protocol/src/export.rs:123`, `app-server-protocol/src/schema_fixtures.rs:1` | app-server → `plugins/app-server-jsonrpc` | planned | M |

## 4. Traduction en plugins

- `registry → Registre → omega-registry` (registre contributeurs, **plugin**, adopted) : le modele Builder-mutable-puis-immuable + `to_builder()` (cloner-etendre) est la reference directe pour « everything is a plugin ».
- `config-source → ConfigSource → omega-config` (couches + profils + flags, **plugin**, adopted/planned) : la `precedence() -> i16` testable et le `deny_unknown_fields` sont deja repris ; restent les profils nommes avances et les flags versionnes.
- `hook → Hook → plugins/hook-command` (**plugin**, planned) : reprendre `HookResult::{Success, FailedContinue, FailedAbort}` tel quel — il distingue continuer vs tout arreter, ce qui manque a notre port `Hook`.
- `tool-surface → ToolSurface → omega-context::ToolSurfaceIndex` (**plugin**, adopted) : l'idee `exposure Direct/Deferred` + `search_info` conforte l'hydratation a la demande.
- `sandbox-backend → SandboxBackend → omega-sandbox::LocalSandboxBackend` (**plugin**, adopted) : reprendre le *type* `SandboxPolicy` (politique typee), pas les backends OS (cout L/XL).
- `permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue` (**plugin**, adopted) : la granularite par categorie + la chaine `decide()` sont le patron a suivre.
- `exec-policy → ExecPolicy → omega-sandbox::RuleBasedExecPolicy` (**plugin**, adopted) : le deterministe (prefixes + reseau, `Allow/Prompt/Forbidden`) incarne « le deterministe n'est jamais une tache d'agent ».
- `telemetry-sink → TelemetrySink → omega-otel::{StdoutSink,FileSink}` (**plugin**, adopted) + `Provenance` (**native**, adopted) : nommage `codex.<domaine>.<objet>` et gating log-only/trace-safe a copier.
- `event-store → EventStore → omega-otel::TraceReplayer` (**plugin**, adopted) : rollout + traces par phase rejouables sans re-executer.
- `context-engine → ContextEngine → omega-context::DefaultContextEngine` (**plugin**, adopted/planned) : compaction budgetee reprise ; `AGENTS.md` remontant + trust a porter (`plugins/agents-md-merge`).
- `memory-store → MemoryStore → plugins/memory-local` (**plugin**, planned) : lecture/ecriture gardee avec modeles dedies.
- `llm-provider → Provider → omega-providers::OllamaProvider` (**plugin**, adopted) : `map_api_error` (traduction d'erreur vers le noyau) et modeles preferes par usage.
- `subagent-runner → SubagentRunner → plugins/subagent-isolated` (**plugin**, planned) : roles TOML stricts + cycle trace.
- `skill-store → SkillStore → omega-context::SkillIndex` (**plugin**, adopted) : snapshots + interface declaree.
- `app-server → AppServer → plugins/app-server-jsonrpc` (**service**, planned) : frontiere generee `ts-rs` + fixtures.

## 5. Ecarte

- **Couches MDM / Enterprise / managed-legacy** (`config_layer_source.rs:5`) : specifiques au parc Apple/entreprise d'OpenAI, sans equivalent dans notre cible self-hosted. La mecanique (precedence) est reprise, pas ces sources.
- **Backends OS bruts** (Seatbelt, Landlock/seccomp, MXC, `windows-sandbox-rs`) : cout XL pour un gain nul a notre echelle — on confine via backend local + execpolicy, pas via le noyau OS.
- **Cloud/chatgpt/lmstudio providers proprietaires** (`chatgpt/`, `codex-client`, `responses-api-proxy`) : couples au backend OpenAI ; seul le *port* `ModelProvider` est repris.
- **Granularite extreme du workspace** (~127 crates, `workspace.dependencies` geant) : anti-pattern a notre stade — Goose (15 crates) est le bon grain, Codex le bon principe a terme (cf. RE-GOOSE-CODEX §2.3).

## 6. Sources

- Repo : https://github.com/openai/codex — corpus `/home/tayk0oo/omega-re/codex` @ `e790ce2`.
- Fiche precedente : `docs/omega/reverse-eng/RE-GOOSE-CODEX.md`.
- Donnees : `omega/crates/omega-harness/data/apps/codex.json` (19 fonctionnalites : 13 adopted, 6 planned).
- Chemins cles : `codex-rs/ext/extension-api/src/{contributors.rs,registry.rs}`, `codex-rs/config/src/{config_layer_source.rs,merge.rs,profile_toml.rs}`, `codex-rs/hooks/src/types.rs`, `codex-rs/tools/src/tool_executor.rs`, `codex-rs/execpolicy/src/{decision.rs,rule.rs,policy.rs}`, `codex-rs/otel/src/{targets.rs,metrics/names.rs}`, `codex-rs/protocol/src/{protocol.rs,config_types.rs}`, `codex-rs/core/src/{agents_md.rs,compact.rs}`, `codex-rs/{skills,memories,agent-roles,features,app-server-protocol,rollout,thread-store,sandboxing}/`.
