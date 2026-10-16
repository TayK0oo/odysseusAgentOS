# Dissection — Windsurf / Cascade (Cognition)

> Dissection par DOCUMENTATION (logiciel propriétaire, binaire fermé).
> Date : 2026-10-16 · Sources : docs officielles Devin Desktop (ex-Windsurf), anciennement docs.windsurf.com.
> Tout fait est tagué `[documenté]` (lu dans la doc, URL citée) ou `[inféré]` (déduction).
> La doc Windsurf a migré vers `docs.devin.ai` après le rachat par Cognition ; les chemins
> `.windsurf/` restent supportés en héritage de `.devin/` `[documenté]`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Nom | Windsurf / Cascade |
| Éditeur | Cognition (ex-Codeium) |
| Licence | Propriétaire |
| OSS | Non (`oss=false`) |
| Langage | Rust/TypeScript (inféré — non publié ; l'IDE est un fork VS Code, le CLI/agent Devin Local a des composants Rust `[inféré]`) |
| Repo | Aucun (fermé) |
| Architecture | IDE agentique dont le panneau Cascade orchestre un agent principal + un agent de planification en tâche de fond + des sous-agents de retrieval (Fast Context) ; personnalisation par fichiers markdown découverts (`.devin/`, héritage `.windsurf/`) ; extension par MCP + hooks shell ; isolation des tâches parallèles par git worktrees `[documenté pour le comportement, inféré pour l'intérieur]` |

## 2. Entrées

| Entrée | Nature |
|---|---|
| Devin Desktop (ex-Windsurf Editor), panneau Cascade (`Cmd/Ctrl+L`) | UI principale, modes Code/Plan/Ask |
| Plugins JetBrains / VS Code / Visual Studio / Vim / Jupyter (Cascade Write/Chat) | Extension éditeurs |
| Devin CLI (`devin`, `/handoff`, `/cloud`) — agent Devin Local | CLI, successeur de Cascade pour les nouveaux onglets |
| `mcp_config.json` (`~/.config/devin/`) | Déclaration des serveurs MCP |
| `.devin/rules/*.md`, `.devin/workflows/*.md`, `.devin/skills/*/SKILL.md`, `.devin/hooks.json` (héritage `.windsurf/…`) | Personnalisation versionnable |
| `~/.codeium/windsurf/memories/`, `~/.windsurf/plans`, `~/.windsurf/worktrees/` | État local (mémoires, plans, worktrees) |
| Dashboard Teams/Enterprise (MCP allowlist, hooks cloud, analytics API) | Gouvernance SaaS |

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Source (doc) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Plan continu + Todo list (planificateur en tâche de fond) | agents | Plan long terme raffiné en continu, Todo suivie en conversation | cascade.md | task-planner → TaskPlanner → plugins/task-planner-cascade | planned | M |
| Modes Code / Plan / Ask | agents | 3 modes (tout-outils / plan externe / lecture seule), plan en Markdown persistant | modes.md | agent-modes → AgentModePolicy → plugins/agent-modes | planned | S |
| Fast Context (SWE-grep) | contexte | Sous-agent de retrieval parallèle, 20x plus rapide, outils restreints | fast-context.md | context-engine → ContextEngine → omega-context::FastRetriever | planned | M |
| Moteur RAG + pinning (M-Query) | contexte | Index codebase, contexte par défaut, pinning, .devinignore | overview.md, devin-ignore.md | context-engine → ContextEngine → omega-context::DefaultContextEngine | planned | M |
| Memories auto-générées | memoire | Faits retenus auto, scopés workspace, rappel pertinent | memories.md | memory-store → MemoryStore → plugins/memory-bounded | planned | S |
| Rules à 4 triggers | configuration | always_on / glob / model_decision / manual, 3 niveaux (global/workspace/système) | memories.md | config-source → ConfigSource → omega-config | planned | S |
| AGENTS.md scopés | configuration | Instructions scopées par emplacement, sans frontmatter | agents-md.md | config-source → ConfigSource → omega-config | planned | S |
| Workflows /slash manuels | automatisation | Runbooks markdown, invocation manuelle exclusive, composables | workflows.md | workflow-runner → WorkflowRunner → plugins/workflow-runner | planned | S |
| Skills SKILL.md + progressive disclosure | extension | Dossiers avec supports, nom+description puis corps à l'usage | skills.md, agentskills.io | skill-store → SkillStore → omega-context::SkillIndex | planned | S |
| MCP natif + allowlist regex | outils | stdio/HTTP/SSE+OAuth, plafond 100 outils, gouvernance Entreprise | mcp.md | tool-surface → ToolSurface → omega-context::ToolSurfaceIndex | planned | M |
| Hooks 12 événements (exit 2 bloque) | execution | Shell pre/post avec contexte JSON, audit transcript JSONL | hooks.md | hook-bus → HookBus → plugins/hook-bus | planned | M |
| Checkpoints nommés + revert | execution | Snapshots nommés, retour au pas choisi (irréversible) | cascade.md | checkpoint-store → CheckpointStore → plugins/checkpoint-store | planned | M |
| Sessions en git worktrees | execution | Worktree isolé par session, Merge, hook de setup, nettoyage auto | worktrees.md | sandbox-backend → SandboxBackend → plugins/worktree-isolated | planned | M |
| Awareness temps réel + @-conversations | contexte | Sélection auto-incluse, "Continue", rappel par résumé ciblé | cascade.md | context-engine → ContextEngine → omega-context::DefaultContextEngine | planned | M |
| Arena mode aveugle | routage | Fan-out multi-modèles isolés, convergence sur le meilleur | arena.md | router-policy → ScoringPolicy/Selector → omega-router::ArenaSelector | planned | M |
| Catalogue à crédits propriétaires | routage | Multiplicateurs 0.25x–320x, Adaptive, Fusion | models.md, adaptive.md | — (rejeté : monnaie SaaS) | rejected | M |

Fonctionnalités mineures relevées mais non modélisées (faute de profondeur suffisante ou redondantes) :
messages en file (queued), saisie vocale, auto-fix du linter (gratuit si faute de Cascade),
Explain & Fix, web search `@web`/`@docs` + lecture d'URL locale, app deploys Netlify,
partage de conversation (Teams), analytics API (Cascade/lignes/consommation), Command `Cmd+I`.
La plupart sont des variantes UI des capacités ci-dessus `[documenté]`.

## 4. Traduction en plugins

| Capacité | Port | Adaptateur | Intégration | Source Windsurf |
|---|---|---|---|---|
| task-planner | TaskPlanner (refine/track) | plugins/task-planner-cascade | plugin | plan continu + Todo |
| agent-modes | AgentModePolicy | plugins/agent-modes | plugin | modes Code/Plan/Ask |
| context-engine | ContextEngine (retrieve) | omega-context::FastRetriever | plugin | Fast Context SWE-grep |
| context-engine | ContextEngine (ingest/assemble) | omega-context::DefaultContextEngine | plugin | RAG M-Query + pinning |
| context-engine | ContextEngine (awareness/recall) | omega-context::DefaultContextEngine | plugin | awareness + @-mentions |
| memory-store | MemoryStore | plugins/memory-bounded | plugin | memories workspace |
| config-source | ConfigSource (layers + activation) | omega-config (rules layers) | plugin | rules 4 triggers |
| config-source | ConfigSource (discovery + scoping) | omega-config (agents-md) | plugin | AGENTS.md |
| workflow-runner | WorkflowRunner | plugins/workflow-runner | plugin | workflows /slash |
| skill-store | SkillStore (discover/load) | omega-context::SkillIndex | plugin | skills progressive disclosure |
| tool-surface | ToolSurface (index + hydrate) | omega-context::ToolSurfaceIndex | plugin | MCP + allowlist |
| hook-bus | HookBus (pre/post + policy) | plugins/hook-bus | plugin | hooks 12 événements |
| checkpoint-store | CheckpointStore | plugins/checkpoint-store | plugin | checkpoints nommés |
| sandbox-backend | SandboxBackend (isolate/merge/cleanup) | plugins/worktree-isolated | plugin | worktrees |
| router-policy | ScoringPolicy/Selector | omega-router::ArenaSelector | plugin | arena mode |

Les 5 plus fortes (transférabilité × originalité) :
1. **Rules à 4 triggers** — le meilleur système de chargement conditionnel observé
   (`model_decision` = description seule en prompt, corps à la demande) ; patron direct
   pour borner le coût en contexte d'Omega.
2. **Fast Context** — retrieval délégué à un sous-agent rapide aux outils restreints ;
   incarne « le coût de calcul va au système » (P10 §4 HARNESS-LANDSCAPE).
3. **Planificateur en tâche de fond + Todo** — séparation planification/exécution
   sans changer de mode, auto-actualisé par les memories.
4. **Hooks 12 événements + transcript JSONL** — gouvernance déterministe la plus
   complète observée (blocage exit 2, audit des règles déclenchées).
5. **Arena aveugle sur worktrees** — évaluation comparative sans biais de nom,
   convergence explicite ; patron pour le router Pareto d'Omega.

## 5. Écarté

- **Catalogue de modèles à crédits propriétaires** (`rejected`) : la mécanique de
  facturation (multiplicateurs par modèle/effort, quotas, Adaptive/Fusion comme
  routeurs fermés) suppose un opérateur SaaS ; incompatible avec l'exigence
  self-hosted et anti-lock-in d'Omega. On reprend l'idée (routage par capacité,
  comparaison aveugle) via `omega-router`, pas la monnaie.
- **Télémétrie/analytics SaaS** (dashboard, analytics API) : non modélisée comme
  fonctionnalité ; le besoin est couvert par `omega-otel` (TelemetrySink).
- **Lecture du binaire** : aucune ; tout est `[documenté]`, l'intérieur (M-Query,
  SWE-grep, orchestrateur) reste `[inféré]`.

## 6. Sources

- Cascade Overview (Code/Chat, plan+Todo, checkpoints, awareness, @-mentions, worktrees) :
  https://docs.devin.ai/desktop/cascade/cascade.md (ex-https://docs.windsurf.com/windsurf/cascade/cascade)
- Modes : https://docs.devin.ai/desktop/cascade/modes.md
- Memories & Rules : https://docs.devin.ai/desktop/cascade/memories.md
- AGENTS.md : https://docs.devin.ai/desktop/cascade/agents-md.md
- Workflows : https://docs.devin.ai/desktop/cascade/workflows.md
- Skills : https://docs.devin.ai/desktop/cascade/skills.md
- MCP : https://docs.devin.ai/desktop/cascade/mcp.md
- Hooks : https://docs.devin.ai/desktop/cascade/hooks.md
- Worktrees : https://docs.devin.ai/desktop/cascade/worktrees.md
- Arena : https://docs.devin.ai/desktop/cascade/arena.md
- Web/Docs search : https://docs.devin.ai/desktop/cascade/web-search.md
- Contexte RAG : https://docs.devin.ai/desktop/context-awareness/overview.md
- Fast Context : https://docs.devin.ai/desktop/context-awareness/fast-context.md
- Modèles + Adaptive : https://docs.devin.ai/desktop/models.md · https://docs.devin.ai/desktop/adaptive.md
- Standard skills : https://agentskills.io/specification
- Contexte paysage : docs/omega/research/HARNESS-LANDSCAPE.md (§1 Windsurf/Cascade, §4 P7)
