# Dissection — Refact.ai (smallcloudai, AGPL-3.0, Rust)

> Source : clone `--depth 1` de https://github.com/smallcloudai/refact dans `/home/tayk0oo/omega-re/refact-ai`.
> Donnees : `omega/crates/omega-harness/data/apps/refact-ai.json` (10 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | smallcloudai |
| Langage | Rust |
| Repo | https://github.com/smallcloudai/refact |
| Architecture | Moteur d'agent Rust (refact-agent/engine) : crates refact-* (core/chat-api/agentic/caps/context/history), tools en tool_*.rs, scratchpads + vecdb + knowledge_graph + memories frontmatter, subchat/background agents, LSP + git worktrees, privacy blocklist, plugins IDE vscode/intellij. |

## 2. Inventaire fonctionnel

### `rfc-engine-rust` — Moteur agent complet en Rust (refact-agent/engine) : 16 crates + src module

- **domaine** : execution · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : adopted
- **what** : Un moteur d'agent en Rust : crates refact-core/chat-api/agentic/caps/context/history et un src/ : scratchpads, tools, providers, vecdb, knowledge_graph, lsp, git, worktrees.
- **how** : refact-agent/engine/ (Cargo workspace, crates/ contient refact-agentic, refact-ast, refact-chat-api, refact-core...) + engine/src/lib.rs. Omega : transposition directe au workspace omega de 16 crates ; decoupage crates homologue.
- **adaptateur Omega** : omega-core (engine agent en Rust)
- **preuves** : `refact-agent/engine/src/lib.rs` · `refact-agent/engine/crates`

### `rfc-tools` — Catalogue tool_* par schema : lifecycle/steer/diff/ask/background agents

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : M · **statut** : planned
- **what** : Un catalogue de tools par fichiers tool_*.rs : activate_skill, agent_lifecycle/steer/diff/pulse, buddy (draft/issue), ast_definition, background_agents, ask_questions...
- **how** : engine/src/tools/ (tool_agent_lifecycle.rs, tool_agent_steer.rs, tool_background_agents.rs, tool_ask_questions.rs...) + mod.rs (registre). Omega : ToolSurfaceIndex couvert.
- **adaptateur Omega** : omega-context::ToolSurfaceIndex (catalogue tool_*)
- **preuves** : `refact-agent/engine/src/tools`

### `rfc-scratchpads` — Scratchpads : session de contexte structuree par template + postprocessing

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : M · **statut** : planned
- **what** : Une scratchpad = une session de contexte structree : template par tache, multimodalite, postprocessing de la reponse.
- **how** : engine/src/scratchpads/ (mod.rs, multimodality.rs, scratchpad_utils.rs) + scratchpad_abstract.rs + postprocessing/. Omega : ContextEngine (template + postprocess).
- **adaptateur Omega** : omega-context (scratchpad structuree)
- **preuves** : `refact-agent/engine/src/scratchpad_abstract.rs` · `refact-agent/engine/src/scratchpads`

### `rfc-vecdb` — vecdb : index d'embeddings local + indexing incremental

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : M · **statut** : planned
- **what** : Vector DB local du workspace : embeddings de snippets re-indexes incrementalement, config yaml, token counting.
- **how** : engine/src/vecdb/ + indexing_utils.rs. Omega : omega-context (recherche semantique locale).
- **adaptateur Omega** : omega-context (vecdb local)
- **preuves** : `refact-agent/engine/src/vecdb` · `refact-agent/engine/src/indexing_utils.rs`

### `rfc-memories` — Memories en frontmatter (faits + prefs) avec dedupe/normalisation tags

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : M · **statut** : planned
- **what** : Systeme de memoires persistantes : MemoRecord en markdown + frontmatter KnowledgeFrontmatter, dedupe des preferences, normalisation des tags (max_tags).
- **how** : engine/src/memories.rs : MemoRecord (:39), create_frontmatter (:133), normalize_memory_tags (:261), normalize_preference_text_for_dedupe (:767). Omega : FileMemoryStore ; retenu : normalisation/dedupe des tags.
- **adaptateur Omega** : omega-app::FileMemoryStore (frontmatter + dedupe prefs)
- **preuves** : `refact-agent/engine/src/memories.rs:39-133` · `refact-agent/engine/src/memories.rs:261`

### `rfc-kg` — Knowledge graph de concepts de code relie aux memories

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : L · **statut** : planned
- **what** : Un KG des concepts du depot (kg_structs) alimente les memories via KnowledgeFrontmatter.
- **how** : engine/src/knowledge_graph/ + knowledge_index.rs + knowledge/. Omega : structure Graphify-like ; garder le meme futur port.
- **adaptateur Omega** : omega-app (knowledge graph + index)
- **preuves** : `refact-agent/engine/src/knowledge_graph` · `refact-agent/engine/src/knowledge_index.rs`

### `rfc-subagents` — Sous-agents : subchat + background agents + phases

- **domaine** : agents · **capacite** : subagent-runner · **port** : SubagentRunner · **cout** : M · **statut** : planned
- **what** : Les sous-agents tournent en sessions subchat separees, et tool_background_agents les relance en async ; subagent_phases structure les phases.
- **how** : engine/src/subchat.rs + tools/tool_background_agents.rs + tools/subagent_phases.rs. Omega : LocalSubagentRunner couvert.
- **adaptateur Omega** : omega-app::LocalSubagentRunner (subchat async)
- **preuves** : `refact-agent/engine/src/subchat.rs` · `refact-agent/engine/src/tools/tool_background_agents.rs`

### `rfc-lsp-worktrees` — Integration LSP + git worktrees par agent

- **domaine** : automatisation · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : M · **statut** : planned
- **what** : Le moteur parle LSP au workspace et isole chaque agent dans un worktree git.
- **how** : engine/src/lsp.rs + engine/src/{git/,worktrees/}. Omega : omega-sandbox::LocalSandboxBackend (worktrees).
- **adaptateur Omega** : omega-sandbox (worktrees + LSP)
- **preuves** : `refact-agent/engine/src/lsp.rs` · `refact-agent/engine/src/worktrees`

### `rfc-privacy` — Privacy : blocklist de fichiers sensibles, traitement local vs cloud

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : Filtrage du contexte : files_blocklist (fichiers jamais envoyes), privacy.rs oriente local/cmloud par regle.
- **how** : engine/src/privacy.rs + engine/src/files_blocklist.rs. Omega : ExecPolicy (blocklist deja couverte).
- **adaptateur Omega** : omega-core::ExecPolicy (blocklist + mode privacy)
- **preuves** : `refact-agent/engine/src/privacy.rs` · `refact-agent/engine/src/files_blocklist.rs`

### `rfc-stats` — Stats telemetry agregee, visible en dashboard self-hosted

- **domaine** : observabilite · **capacite** : telemetry-sink · **port** : TelemetrySink · **cout** : S · **statut** : planned
- **what** : Module stats des usages (runs, tokens) expose via http.rs.
- **how** : engine/src/stats/ + engine/src/http.rs. Omega : omega-otel + EventStore.
- **adaptateur Omega** : omega-otel (dashboard usage)
- **preuves** : `refact-agent/engine/src/stats`

