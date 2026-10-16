# Dissection — Cursor (Anysphere)

> **Méthode :** dissection par DOCUMENTATION (logiciel propriétaire, pas de clone).
> `[documenté]` = lu dans la doc officielle (URL en §6) · `[inféré]` = déduction signalée comme telle dans chaque `how` du JSON.
> Données : `omega/crates/omega-harness/data/apps/cursor.json` (17 fonctionnalités).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Anysphere |
| Licence | Propriétaire (`oss=false`) |
| Langage | TypeScript (fork VS Code) |
| Repo | Fermé (pas de repo public) |
| Architecture | Éditeur + Agent orchestrateur (instructions + outils + modèle, réglages per-modèle) ; indexation et recherche locales (Instant Grep) ; sous-agents à contexte isolé ; MCP multi-transports ; exécution locale et cloud (VM dédiées) |

## 2. Entrées

- **Binaires/CLI :** éditeur Cursor, Agent (panneau latéral `Cmd+I`), CLI (`/goal`, `/in-cloud`, `/autopilot`), Cloud Agents (`cursor.com/agents`), app iOS.
- **Points d'extension :** `.cursor/rules/*.mdc`, `AGENTS.md` imbriqués, `.cursor/agents/*.md` (+ compat `.claude/`, `.codex/`), `.cursor/mcp.json`, `.cursor/hooks.json`, marketplace + team marketplace (`.cursor-plugin/marketplace.json`).
- **Fichiers de config :** `.cursor/rules/`, `.cursor/mcp.json` / `~/.cursor/mcp.json`, `.cursor/hooks.json`, `~/.cursor/agents/`, dashboard team (rules, MCP, modèles).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|
| Tab : autocompletion + sauts prédictifs | outils | Suggestion à la frappe multi-lignes, imports, jump-in-file, portails inter-fichiers | inline-complete → InlineComplete | planned | M |
| Instant Grep | contexte | Recherche exacte maison > ripgrep, auto-déclenchée par l'Agent | code-search → CodeSearch | planned | M |
| Index codebase 100 % local | contexte | Index incrémental sans upload chemins/embeddings | code-index → CodeIndex | planned | M |
| Agent (instructions+outils+modèle) | agents | Boucle autonome, tuning per-modèle, appels illimités | agent-runner → AgentRunner | planned | L |
| Surface d'outils de l'Agent | outils | Fichiers, shell, navigateur, web, images, questions | tool-surface → ToolSurface | planned | M |
| Checkpoints | execution | Snapshots locaux + revert fichiers seuls | checkpoint-store → CheckpointStore | planned | M |
| File + pilotage à chaud + side chats | agents | Queue, steer, `/side`, `/goal` | agent-control → AgentControl | planned | S |
| Project Rules `.mdc` | configuration | 4 types, frontmatter, précédence Team>Project>User | config-source → ConfigSource | planned | S |
| AGENTS.md imbriqués | configuration | Markdown cumulé parent→enfant | config-source → ConfigSource | planned | S |
| Sous-agents custom + builtin | agents | Frontmatter, fg/bg, imbrication ≤2, reprise par ID | subagent-runner → SubagentRunner | planned | M |
| Explore subagent | contexte | Fan-out recherche, seul résumé remonte | subagent-runner → SubagentRunner | planned | S |
| MCP 3 transports + allowlist | extension | Tools/Prompts/Resources/Roots/Elicitation/Apps, OAuth, sandbox réseau | mcp-client → McpClient | planned | M |
| Multi-modèles + Router Auto | routage | Pool hebdo + tiers au prix API, Cost/Balance/Intelligence | router-policy → ScoringPolicy | planned | M |
| Cloud Agents | execution | VM isolées, env setup/snapshot/Dockerfile, artefacts, RDP | cloud-runner → CloudRunner | planned | L |
| Worktrees par sous-agent | execution | Worktree Git + branche, fusion parent | worktree-manager → WorktreeManager | planned | M |
| Hooks de cycle de vie | automatisation | `.cursor/hooks.json`, cloud après phase read-only | hook-bus → HookBus | planned | S |
| Télémétrie cloud propriétaire | observabilite | Mesure centralisée couplée facturation | — | **rejected** | S |

Répartition : 17 fonctionnalités — 16 planned, 1 rejected, 0 adopted (honnêteté : rien n'existe encore dans le code Omega). Coûts : S=6, M=9, L=2.

## 4. Traduction en plugins

- `inline-complete → InlineComplete → plugins/inline-complete` (plugin, M) — Tab.
- `code-search → CodeSearch → plugins/grep-local` (plugin, M) — Instant Grep : index inversé + regex, sans LLM (règle §3.2).
- `code-index → CodeIndex → plugins/code-index-local` (plugin, M) — index incrémental local, garantie no-upload.
- `agent-runner → AgentRunner → plugins/agent-loop` (plugin, L) — boucle instructions+outils+modèle, tuning per-modèle.
- `tool-surface → ToolSurface → plugins/agent-tools` (plugin, M) — surface fixe fichiers/shell/browser/web.
- `checkpoint-store → CheckpointStore → plugins/checkpoint-local` (plugin, M) — snapshots + revert fichiers seuls.
- `agent-control → AgentControl → plugins/agent-steer` (plugin, S) — file, steer, side chats.
- `config-source → ConfigSource → plugins/rules-scoped` (plugin, S) — `.mdc` + `AGENTS.md` imbriqués, précédence explicite.
- `subagent-runner → SubagentRunner → plugins/subagent-isolated` (plugin, M) — fg/bg, readonly, reprise par ID ; variante explore (S).
- `mcp-client → McpClient → plugins/mcp-client` (plugin, M) — stdio/SSE/HTTP, OAuth, allowlist, sandbox réseau.
- `router-policy → ScoringPolicy → plugins/model-router` (plugin, M) — pool + tiers, modes coût/vitesse/reflexion.
- `cloud-runner → CloudRunner → plugins/cloud-runner` (plugin, L) — VM, env, artefacts (candidat microservice).
- `worktree-manager → WorktreeManager → plugins/worktree-git` (plugin, M) — isolation par worktree (gix).
- `hook-bus → HookBus → plugins/hooks` (plugin, S) — bus d'événements + sous-processus.

## 5. Écarté

- **Télémétrie cloud propriétaire** (`cu-telemetry-cloud`, rejected) : mesure centralisée chez Anysphere couplée à la facturation, pipeline closed-source. Omega mesure en local via `omega-otel` (`Stdout`/`File` sinks), sans exfiltration — reprendre l'idée (mesurer l'usage par modèle) sans le vecteur (cloud éditeur).

## 6. Sources

- https://cursor.com/docs/context/codebase-indexing (Instant Grep, Explore, privacy, multi-root)
- https://cursor.com/docs/agent/overview (Agent, outils, checkpoints, file, side chats, /goal)
- https://cursor.com/docs/rules (rules .mdc, Team Rules, AGENTS.md)
- https://cursor.com/docs/subagents (custom/builtin, fg/bg, worktrees, cloud, reprise)
- https://cursor.com/docs/tab/overview (Tab)
- https://cursor.com/docs/context/mcp (MCP transports, OAuth, allowlist)
- https://cursor.com/docs/cloud-agent (Cloud Agents, env, artefacts)
- https://cursor.com/docs/models-and-pricing (modèles, pools, Router)
- https://cursor.com/docs/hooks.md + https://cursor.com/docs/configuration/worktrees.md (cités, non re-fetchés — chemins documentés depuis subagents/cloud)
- docs/omega/research/HARNESS-LANDSCAPE.md §2.3 (fiche Cursor du paysage)
