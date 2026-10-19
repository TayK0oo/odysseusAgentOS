# Dissection — Zed

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Zed Industries |
| Licence | GPL-3.0 |
| Langage | Rust (moteur GPUI natif, ~270 crates) |
| Repo | https://github.com/zed-industries/zed |
| Architecture | Editeur natif avec agent integre : threads persistants (SQLite), boucle `run_turn` streaming annulable, agents externes via ACP v1/v2, serveurs MCP (ContextServer stdio/HTTP), skills SKILL.md a divulgation progressive, checkpoints git par message, isolation par worktree. |

Clone : `git clone --depth 1 --filter=blob:none https://github.com/zed-industries/zed zed` (reussi du premier coup, ~4386 fichiers racine).

## 2. Entrees

- **Binaire/editeur** : Zed (GPUI) + **Agent Panel** (`crates/agent_ui/src/agent_panel.rs:1116`).
- **Points d'extension** : serveurs d'agents externes ACP (`crates/agent_servers/src/` : `acp.rs`, `custom.rs`), serveurs MCP (`crates/context_server/src/`), skills `.agents/skills` (global `~/.agents/skills` + projet), outils natifs (`crates/agent/src/tools/` ~30 outils), regles `AGENTS.md` (`crates/agent_settings/src/user_agents_md.rs`).
- **Fichiers de config** : `AgentSettings` (`crates/agent_settings/src/agent_settings.rs:211` : profils, `auto_compact`, sandbox, permissions), prompts (`crates/agent/src/templates/system_prompt.hbs`, `crates/prompt_store/src/`).

## 3. Inventaire fonctionnel

| Fonctionnalite | Domaine | Ce que ca fait | Ou (file:line) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|
| Boucle agent streaming annulable | execution | Tours modele→outils, cancel watch, gestion refus/max-tokens | `crates/agent/src/thread.rs:2843-2920` | AgentRunner → `plugins/agent-loop` | planned | M |
| Profils d'agent | configuration | Toggle outil par outil (natifs + MCP) par profil | `crates/agent_settings/src/agent_profile.rs:29-152` | ConfigSource → `plugins/agent-profiles` | planned | S |
| Agents externes ACP v1/v2 | agents | Binaires/CLI tiers pilotés comme sessions, updates session+terminal | `crates/agent_servers/src/acp.rs:329-340` | AgentServer → `plugins/acp-bridge` | planned | M |
| Serveurs MCP stdio/HTTP | outils | Outils/contexte externes, OAuth, filtrage par profil | `crates/context_server/src/context_server.rs:28-115` | McpBridge → `plugins/mcp-bridge` | planned | M |
| Compaction automatique sur seuil | contexte | Seuil % / tokens utilisés / restants + télémétrie | `crates/agent/src/thread.rs:4583-4733` | ContextEngine → `omega-context::truncate_compact` | planned | M |
| Compaction manuelle résumée | contexte | Modèle dédié résume jusqu'au marqueur, remplace le contexte | `crates/agent/src/thread.rs:2683-2796` | ContextEngine → `omega-context::truncate_compact` | planned | M |
| New From Summary | memoire | Nouveau fil amorcé du résumé du précédent | `crates/agent_ui/src/agent_panel.rs:3490-3522` | ContextEngine → `plugins/thread-from-summary` | planned | S |
| @-mentions | contexte | Fichiers/sélections/diffs parsés et résolus en contexte | `crates/acp_thread/src/mention.rs:20-120` | MentionResolver → `plugins/mention-resolve` | planned | M |
| Isolation par worktree par fil | execution | Fil = git worktree dédié, archivable/restaurable | `crates/agent/src/thread.rs:843-865` | WorktreeIsolate → `plugins/worktree-isolate` | planned | L |
| Checkpoints git + restauration | memoire | Checkpoint par message, rewind fil + arbre | `crates/acp_thread/src/acp_thread.rs:6998-7030` | EventStore → `omega-app::FileEventStore` | adopted | M |
| Usage tokens visible | observabilite | Cumulé vs limite, warning 80 %, tooltip + bandeau | `crates/acp_thread/src/acp_thread.rs:3350-3390` | ContextEngine+Budget → `omega-context::ContextBudget` | adopted | S |
| Agent Panel | ui | Fils, sélecteurs modèle/profil, diffs, inline | `crates/agent_ui/src/agent_panel.rs:1116` | AgentPanel → `plugins/agent-panel` | planned | M |
| Registre multi-fournisseurs | routage | ~10 fournisseurs natifs, modèle sélectionné global | `crates/language_model/src/registry.rs:46-194` | Provider → `omega-providers::OllamaProvider` | adopted | M |
| Permissions + sandbox | securite | Règles codées en dur + settings, escalation, approbation | `crates/agent/src/tool_permissions.rs:21-253` | PermissionPolicy → `omega-sandbox::ApprovalQueue` | adopted | S |
| Skills SKILL.md progressifs | contexte | Catalogue → corps → ressources, outil skill + slash | `crates/agent/src/tools/skill_tool.rs:46-130` | SkillStore → `omega-context::SkillIndex` | adopted | S |
| Règles + migration skills | configuration | AGENTS.md injecté entier, migration vers skills | `crates/agent_settings/src/user_agents_md.rs:13-97` | ConfigSource → `plugins/rules-inject` | planned | S |
| Sous-agents via outils | agents | spawn/create_thread, threads enfants rattachés | `crates/agent/src/tools/spawn_agent_tool.rs:47-131` | SubagentRunner → `plugins/subagent-isolated` | planned | M |

## 4. Traduction en plugins

- `tool-surface → ToolSurface → omega-context::ToolSurfaceIndex` via skills+outils natifs (pattern catalogue minimal, **adopted** pour skills).
- `skill-store → SkillStore → omega-context::SkillIndex` (plugin, **adopted**) : `.agents/skills` global+projet, scan plat, enveloppe `render_skill_envelope`.
- `permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue` (plugin, **adopted**) : `HardcodedSecurityRules` + `decide_permission_*` + `ThreadSandbox`.
- `llm-provider → Provider → omega-providers::OllamaProvider` (plugin, **adopted**) : `LanguageModelRegistry` multi-fournisseurs derrière le port Provider.
- `context-engine → ContextEngine+ContextBudget → omega-context::ContextBudget` (plugin, **adopted**) : `TokenUsage::ratio` + seuils 80 % comme budget visible.
- `event-store → EventStore → omega-app::FileEventStore` (plugin, **adopted**) : checkpoints git par message = snapshots + `restore_checkpoint`.
- `agent-runner → AgentRunner → plugins/agent-loop` (plugin, planned) : `run_turn`/`RunningTurn`/cancel comme boucle annulable.
- `agent-server → AgentServer → plugins/acp-bridge` (plugin, planned) : ACP v1/v2 comme port d'agents externes.
- `mcp-bridge → McpBridge → plugins/mcp-bridge` (plugin, planned) : `ContextServer` stdio/HTTP + OAuth.
- `context-engine → ContextEngine → omega-context::truncate_compact` (plugin, planned) : auto-compact à seuils + compact manuel résumé.
- `mention-resolver → MentionResolver → plugins/mention-resolve` (plugin, planned) : `MentionUri` + `MentionSet`.
- `worktree-isolate → WorktreeIsolate → plugins/worktree-isolate` (plugin, planned) : `use_new_worktree` + archive/restauration.
- `subagent-runner → SubagentRunner → plugins/subagent-isolated` (plugin, planned) : `SpawnAgentTool`/`CreateThreadTool`.
- `config-source → ConfigSource → plugins/agent-profiles` + `plugins/rules-inject` + `plugins/thread-from-summary` (plugin, planned) : profils, AGENTS.md, New From Summary.
- `ui-panel → AgentPanel → plugins/agent-panel` (plugin, planned) : Agent Panel + diff + inline.

## 5. Écarté

Rien d'écarté : éditeur 100 % OSS (GPL-3.0) sans SaaS obligatoire ; même le cloud (`cloud_llm_client`) reste un fournisseur parmi d'autres derrière le registre. Seule réserve : `Agent Panel` UI (GPUI) n'est pas portable tel quel vers le cockpit React — on n'en reprend que le découpage (fils/sélecteurs/diffs), pas le rendu.

## 6. Sources

- Repo : https://github.com/zed-industries/zed (clone `--depth 1 --filter=blob:none`, ok du 1er essai)
- Docs : https://docs.zed.dev/ai/agent-panel, https://docs.zed.dev/ai/external-agents, https://agentskills.io/specification (via `crates/agent_skills/README.md`)
- Fichiers : `crates/agent/src/thread.rs`, `crates/acp_thread/src/acp_thread.rs`, `crates/acp_thread/src/mention.rs`, `crates/acp_thread/src/compaction.rs`, `crates/agent_servers/src/`, `crates/context_server/src/context_server.rs`, `crates/agent_skills/`, `crates/agent_ui/src/agent_panel.rs`, `crates/agent_ui/src/mention_set.rs`, `crates/agent_ui/src/thread_worktree_archive.rs`, `crates/agent_settings/src/`, `crates/language_model/src/registry.rs`
