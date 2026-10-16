# Dissection — Open Interpreter

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | OpenInterpreter |
| Licence | Apache-2.0 |
| Langage | **Rust** (workspace `codex-rs/`, ~127 crates) |
| Repo | https://github.com/OpenInterpreter/open-interpreter |
| Architecture | Fork de Codex dont le delta est l'**émulation de harnais** : un `enum Harness` (16 variantes) + un routeur `wire_api × harness → StreamTransportRoute` façonne prompt, schémas d'outils, conversion de messages et post-traitement, **sans jamais shell-out vers les CLI externes** — les appels sont exécutés par le runtime Rust natif. |
| Positionnement | Agent de code optimisé pour modèles low-cost : choisir le harnais qui maximise le modèle donné (Kimi K3 → `kimi-code`, DeepSeek → `claude-code-bare`). |

Le delta maintenu vs upstream Codex est explicite (`docs/open-interpreter-delta.md`) : identité produit, catalogue de providers neutres, transport Chat Completions de première classe, transport Messages, harnais natifs.

## 2. Entrées

| Entrée | Fichier |
|---|---|
| `interpreter` CLI / TUI | `codex-rs/cli/src/main.rs`, `codex-rs/tui/` |
| `interpreter exec "..."` (one-shot scripts/CI, JSON, `-i image`) | `docs/exec.md` |
| `interpreter acp` (ACP over stdio pour éditeurs) | `codex-rs/cli/src/main.rs:1422,2791`, `codex-rs/acp-server/` |
| `interpreter mcp {add,list,get,remove,login,logout}` | `docs/mcp.md` |
| `interpreter resume --last` / picker / fork | `docs/sessions.md` |
| Config `~/.openinterpreter/config.toml` (`harness`, `harness_guidance`, `wire_api`, profils, MCP, hooks) | `codex-rs/config/src/config_toml.rs:166-170`, `docs/config.md` |
| Hooks `~/.openinterpreter/hooks.json`, `.openinterpreter/hooks.json`, inline, plugins | `docs/hooks.md`, `codex-rs/hooks/` |
| Skills `.agents/skills/` → `~/.agents/skills/` → embarqués → legacy `~/.openinterpreter/skills/` | `docs/skills.md`, `codex-rs/skills/` |
| Serveur MCP inverse (exposer OI comme outil) | `docs/mcp-server.md`, `codex-rs/mcp-server/` |
| app-server JSON-RPC + daemon | `codex-rs/app-server/` |

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (`fichier:ligne`) | Cible Oméga | Statut | Coût |
|---|---|---|---|---|---|---|
| Catalogue de harnais (`enum Harness`, 16 variantes) | routage | Registre fermé des surfaces réimplémentées + guidance par harnais | `codex-rs/tools/src/harness.rs:2-100`, `core/src/harness/guidance.rs:5-20` | harness-catalog → `omega-harness` | adopted | S |
| Routage strict `wire_api × harness` | routage | `StreamTransportRoute` + point d'intégration unique client (ajouter un harnais = un bras de route) | `core/src/harness/routing.rs:38-160`, `core/src/harness/request.rs:75,214` | router-policy → `plugins/harness-router` | planned | M |
| Inférence auto du harnais par famille | routage | kimi→kimi-code, deepseek→claude-code-bare, qwen→qwen-code, claude→claude-code, zcode dédié ; explicite gagne | `model-provider-info/src/lib.rs:216-285` | router-policy → `plugins/harness-autoselect` | planned | S |
| Émulation Claude Code (3 transports + bare) | outils | Surface Claude complète (tools, thinking, subagents, LSP, wakeups) sur Responses/Chat/Messages | `core/src/harness/claude_code.rs:83-320` (4870 l.) | harness-emulation → `plugins/harness-claude-code` | planned | L |
| Émulation Kimi Code (cache + thinking + 25 tools) | outils | `prompt_cache_key` session, `thinking keep:all`, 25 tools, skills natifs | `core/src/harness/kimi_code.rs:18-110`, `kimi_code_tools.json` | harness-emulation → `plugins/harness-kimi-code` | planned | M |
| Boucle SWE-agent sans tool schemas | exécution | Commande extraite du texte libre, exécutée (timeout 30 s), sortie en observation | `core/src/harness/swe_agent.rs:18-100,238-300,396-540` | harness-emulation → `plugins/harness-swe-agent` | planned | M |
| Émulation ZCode sur Messages | outils | Todos/plans/skills/subagents GLM, exige `wire_api=messages` | `core/src/harness/zcode.rs:1-80`, `zcode_tools.json` | harness-emulation → `plugins/harness-zcode` | planned | L |
| Surface minimale + harnais secondaires | outils | Prompt compact générique ; gabarits Qwen, DeepSeek-TUI, OpenCode, Pi... | `core/src/harness/minimal.rs:1-60`, `qwen_code.rs`, `deepseek_tui.rs`, `opencode.rs` | harness-emulation → `plugins/harness-minimal` | planned | S |
| Re-rendu des skills par harnais | contexte | `<skills_instructions>` parsé puis ré-exprimé au format natif de chaque harnais | `core/src/harness/session_skills.rs:1-100` | skill-store → `plugins/harness-skill-bridge` | planned | S |
| Skills `.agents/skills` paresseux | extension | Format partagé neutre-outil, metadata-first, priorités local > perso > embarqué | `docs/skills.md:36-80`, `skills/src/loading.rs:26,93-119` | skill-store → `omega-context::SkillIndex` | adopted | S |
| Serveurs MCP gérés | outils | stdio/HTTP, catalogue + cache + pagination, trusted access, login CLI | `codex-mcp/src/catalog.rs`, `connection_manager.rs`, `trusted_access.rs` | tool-source → `plugins/tool-mcp` | planned | M |
| ACP + MCP server inverse | extension | Pilotage depuis éditeur (permissions déléguées AllowOnce/Always) ; OI exposable comme outil | `acp-server/src/lib.rs:900-930`, `cli/src/main.rs:1422,2791` | agent-protocol → `plugins/acp-bridge` (service) | planned | M |
| Hooks du cycle de vie (10 phases) | automatisation | Scripts déterministes + couches + revue de trust, flag `features.hooks` | `hooks/src/events/mod.rs:1-10`, `hooks/src/types.rs:92-97` | lifecycle-hook → `plugins/lifecycle-hooks` | planned | M |
| Permissions + execpolicy | sécurité | Profils `:read-only/:workspace/:danger-full-access`, règles deny-prioritaire, `Decision::{Allow,Prompt,Forbidden}` | `docs/permissions.md`, `execpolicy/src/decision.rs:9-25`, `execpolicy/src/policy.rs:128-265` | exec-policy → `omega-sandbox::RuleBasedExecPolicy` | adopted | M |
| Sous-agents à rôles | agents | `/agent`, rôles default/worker/explorer, résumé seul remonte | `docs/subagents.md`, `agent-roles/src/lib.rs:1-8` | subagent-runner → `plugins/subagent-isolated` | planned | M |
| CLI + exec + TUI slash | ui | One-shot scriptable, TUI riche (`/model`, `/permissions`, `/compact`...), bascule harnais persistée | `docs/exec.md`, `docs/slash_commands.md`, `tui/src/config_update.rs:122-123` | ui-surface → `plugins/cli-tui-surface` | planned | M |
| Sessions + rollout + memories | mémoire | Resume/fork sous `~/.openinterpreter/`, journal JSONL parsable, memories opt-in par modèle | `docs/sessions.md`, `rollout/src/lib.rs:50-80`, `docs/memories.md` | event-store → `plugins/session-rollout` | planned | M |
| Computer-use GUI | exécution | **Absent** : aucune automatisation écran/souris ; shell + fichiers + patch uniquement | inventaire `codex-rs/` (aucun crate gui/automation) | — | rejected | S |

## 4. Traduction en plugins

Le morceau à plus forte valeur pour Oméga est le **couple routage + façonnage** :

- `harness-catalog → Registre → omega-harness` (adopted) : le catalogue des surfaces émulables devient donnée du registre, interrogeable par le routeur.
- `router-policy → Selector → plugins/harness-router` + `plugins/harness-autoselect` : `resolve_stream_transport_route` (matrice stricte wire×harness avec erreurs explicites) et `default_harness_for_provider_model` (inférence par famille) sont directement transposables — c'est le chaînon manquant de notre routage actuel (choix du *harnais*, pas seulement du modèle).
- `harness-emulation → HarnessEmulation (build_request/handle_output) → plugins/harness-*` : un port unique (façonner la requête + post-traiter le stream), N adaptateurs (claude-code L, kimi-code M, swe-agent M, zcode L, minimal S). La règle d'or à copier : **un seul point d'intégration côté client** (`build_chat_harness_request` + `apply_chat_harness_postprocess`), chaque harnais n'ajoutant qu'un bras de route.
- `skill-store → SkillStore → plugins/harness-skill-bridge` : le pont `<skills_instructions>` → format natif garantit que les skills survivent au changement de harnais.
- Le reste suit les ports existants : `ToolSource` (MCP), `AcpServer` (service ACP), `HookEngine` (10 phases), `ExecPolicy`/`ApprovalQueue` (permissions, adopted car adaptateurs existants), `SubagentRunner`, `UiSurface`, `EventStore` (sessions/rollout → rejeu de traces).

## 5. Écarté

- **Computer-use GUI** (`rejected`) : vérifié absent par inventaire (`codex-rs/` sans crate gui/automation, aucune section computer-use dans `docs/`, aucune sous-commande dédiée). L'exécution reste shell/fichiers/patch sandboxés — rien à reprendre sur cet axe.
- **Transports propriétaires** : le détail des providers hébergés générés (~10 IDs : openrouter, groq, poe...) n'est pas repris tel quel ; seul le motif `wire_api` + catalogue déclaratif intéresse `omega-providers`.
- **Branding/produit** (`FORK_BRANDING.md`, installeurs, `product-info`) : spécifique à la distribution, sans valeur plugin.

## 6. Sources

- Repo : https://github.com/OpenInterpreter/open-interpreter (clone `--depth 1` du 2026-10-16 dans `/home/tayk0oo/omega-re/open-interpreter`)
- Docs : `docs/harness.md` (matrice IDs/routes), `docs/providers.md` (wire APIs + défauts par famille), `docs/skills.md`, `docs/mcp.md`, `docs/acp.md`, `docs/hooks.md`, `docs/permissions.md`, `docs/sandbox.md`, `docs/subagents.md`, `docs/exec.md`, `docs/sessions.md`, `docs/memories.md`, `docs/slash_commands.md`, `docs/open-interpreter-delta.md`, `FORK_BRANDING.md`
- Code : `codex-rs/tools/src/harness.rs`, `codex-rs/core/src/harness/{mod,request,routing,guidance,session_skills,claude_code kimi_code,kimi_cli,swe_agent,mini_swe_agent,zcode,minimal,qwen_code,deepseek_tui,opencode,pi,little_coder,terminus_2}.rs`, `codex-rs/model-provider-info/src/lib.rs:111-130,216-285`, `codex-rs/chat-wire-compat/`, `codex-rs/skills/src/loading.rs`, `codex-rs/codex-mcp/`, `codex-rs/acp-server/src/lib.rs`, `codex-rs/hooks/`, `codex-rs/execpolicy/`, `codex-rs/agent-roles/`, `codex-rs/rollout/src/lib.rs`, `codex-rs/config/src/config_toml.rs:166-170`
