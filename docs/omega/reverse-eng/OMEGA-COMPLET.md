# OMEGA-COMPLET — le combine unifie

> Genere par `omega/scripts/omega_complet.py` (deterministe, regenerable). Fusionne **toutes** les dissections de harnais en un seul plan d'action.

## 1. Synthese

- harnais disseques : **17**
- fonctionnalites inventoriees : **288** (adoptees 59 · planifiees 221 · ecartees 8)
- capacites visees (regroupees) : **72**
- popularite : **mesuree** (API GitHub, somme des etoiles des harnais demandeurs)
- preuves REA (intake machine) disponibles pour : 0 harnais

### Ce que le consensus designe (>= 4 harnais demandeurs)

| Capacite | Harnais | Popularite (etoiles) | Cout min | Famille |
|---|---|---|---|---|
| `context-engine` | 13 | 744,090 | S | Contexte & memoire de travail |
| `subagent-runner` | 13 | 741,580 | S | Orchestration |
| `config-source` | 10 | 522,782 | S | Configuration & extension |
| `scheduler-job` | 8 | 366,798 | M | Orchestration |
| `tool-surface` | 8 | 291,918 | S | Outils & surface d'action |
| `sandbox-backend` | 8 | 262,534 | M | Execution & securite |
| `memory-store` | 7 | 155,705 | S | Contexte & memoire de travail |
| `tool` | 6 | 181,709 | S | Outils & surface d'action |
| `skill-store` | 6 | 123,938 | S | Contexte & memoire de travail |
| `registry` | 5 | 540,978 | S | Configuration & extension |
| `event-store` | 4 | 434,204 | M | Observabilite & tracabilite |
| `router-policy` | 4 | 84,339 | S | Routage & modeles |
| `hook-bus` | 4 | 4,453 | S | Orchestration |

## 2. Le combine, capacite par capacite

### Autres

#### `(automatisation)` — 1 harnais, cout M

- **harnais demandeurs** : goose
- **popularite cumulee** : 55,349 etoiles
- **exemplaire de reference** : `goose` — *Planificateur cron de recettes* (port `—`, cout M)
  - comment il le fait : Observe : SchedulerTrait (scheduler_trait.rs:9) : add/list/remove/pause/run_now/kill_running_job ; implementation full (scheduler/full.rs:108) avec schedule_recipe (full.rs:343) et sessions par job (full.rs:604) ; outil schedule_tool (agents/schedule_tool.rs:63) exposant la planification au modele. Cron + recettes.
  - preuves : `crates/goose/src/scheduler_trait.rs:9`, `crates/goose/src/scheduler/full.rs:108`, `crates/goose/src/agents/schedule_tool.rs:63`

#### `edit-apply` — 1 harnais, cout S

- **harnais demandeurs** : aider
- **popularite cumulee** : 49,587 etoiles
- **adaptateurs proposes** : `plugins/edit-search-replace`, `plugins/edit-udiff-patch`, `plugins/edit-wholefile`
- **exemplaire de reference** : `aider` — *Formats diff unifie et patch V4A* (port `EditApplier (apply_udiff/apply_patch)`, adaptateur `plugins/edit-udiff-patch`, cout M)
  - comment il le fait : UnifiedDiffCoder parse des diffs unifies standards ; PatchCoder implemente un mini-langage de patch V4A (ActionType/Chunk/PatchAction) avec parseur derive de apply_patch ; application atomique par fichier.
  - preuves : `aider/coders/udiff_coder.py:46-49`, `aider/coders/patch_coder.py:13-46`, `aider/coders/patch_coder.py:210-217`, `aider/coders/patch_coder.py:295-320`

#### `lint-check` — 1 harnais, cout S

- **harnais demandeurs** : aider
- **popularite cumulee** : 49,587 etoiles
- **adaptateurs proposes** : `plugins/lint-after-edit`
- **exemplaire de reference** : `aider` — *Linter multi-langage apres edition* (port `LintCheck (lint_file)`, adaptateur `plugins/lint-after-edit`, cout S)
  - comment il le fait : Classe Linter avec registre par langage (set_linter), lint() dispatchant vers py_compile/flake8 ou commandes externes ; lint_python_compile et basic_lint en repli sans dependance ; appels automatiques apres chaque edition.
  - preuves : `aider/linter.py:21-82`, `aider/linter.py:118-172`, `aider/linter.py:177-234`, `aider/commands.py:356-410`

#### `vcs-commit` — 1 harnais, cout S

- **harnais demandeurs** : aider
- **popularite cumulee** : 49,587 etoiles
- **adaptateurs proposes** : `plugins/git-auto-commit`
- **exemplaire de reference** : `aider` — *Git auto-commit + message genere* (port `VcsCommit (commit/message)`, adaptateur `plugins/git-auto-commit`, cout S)
  - comment il le fait : auto_commit appelle GitRepo.commit avec les fichiers edites ; le message est genere par le weak_model via get_commit_message ; attribution configurable (author vs committer, Co-authored-by, prefixe aider:) ; dirty_commit sauvegarde l'etat pre-edit.
  - preuves : `aider/coders/base_coder.py:2375-2400`, `aider/repo.py:131-270`, `aider/repo.py:326-375`, `aider/models.py:622-624`

#### `plan-store` — 1 harnais, cout M

- **harnais demandeurs** : plandex
- **popularite cumulee** : 15,750 etoiles
- **adaptateurs proposes** : `plugins/plan-branches`
- **exemplaire de reference** : `plandex` — *Arbre de versions du plan (branches par modèle)* (port `PlanStore (branch/fork/rewind)`, adaptateur `plugins/plan-branches`, cout M)
  - comment il le fait : Chaque plan possède des branches listables (api.Client.ListBranches, cmd/branches.go) ; l'état courant (CurrentPlanState : PlanResult, PlanApplies, ConvoMessageDescriptions) est versionné par apply et réversible via rewind (AnalyzeRewind, ApplyRewindChanges).
  - preuves : `app/cli/cmd/branches.go:20-40`, `app/shared/data_models.go:295-303`, `app/cli/lib/rewind.go:107-258`

#### `acp-agent` — 1 harnais, cout L

- **harnais demandeurs** : gptme
- **popularite cumulee** : 4,453 etoiles
- **adaptateurs proposes** : `plugins/acp-agent`
- **exemplaire de reference** : `gptme` — *Agent ACP (client + serveur, permissions mappées)* (port `AcpAgent (session/prompt/cancel)`, adaptateur `plugins/acp-agent`, cout L)
  - comment il le fait : GptmeAgent implemente le protocole ACP : initialize/new_session/prompt/cancel/load_session/authenticate (gptme/acp/agent.py:588-1732) ; run_chat_turn() pont entre le tour ACP et le chat gptme avec streaming de tokens (gptme/acp/agent.py:1305-1492) ; _request_tool_permission mappe les permissions d'outils ACP vers la confirmation gptme (gptme/acp/agent.py:376-563) ; client ACP pour piloter des agen
  - preuves : `gptme/acp/agent.py:254-300`, `gptme/acp/agent.py:588-869`, `gptme/acp/agent.py:1128-1492`, `gptme/acp/client.py:1-100`

#### `cost-guard` — 1 harnais, cout S

- **harnais demandeurs** : gptme
- **popularite cumulee** : 4,453 etoiles
- **adaptateurs proposes** : `plugins/cost-awareness`
- **exemplaire de reference** : `gptme` — *Token/cost/cache awareness (budgets + alertes)* (port `CostGuard (budget/warn/summary)`, adaptateur `plugins/cost-awareness`, cout S)
  - comment il le fait : add_token_budget/add_token_usage_warning injectent budgets et alertes dans le prompt (gptme/hooks/token_awareness.py:56-225) ; cost_warning_hook + session_end_cost_summary suivent le cout par session avec _calculate_llm_cost (gptme/hooks/cost_awareness.py:291-463, gptme/telemetry.py:362-442) ; CacheState/is_cache_likely_cold + cache_cold_warning_hook protegent le prefix-cache Anthropic (gptme/hook
  - preuves : `gptme/hooks/token_awareness.py:56-225`, `gptme/hooks/cost_awareness.py:291-463`, `gptme/hooks/cache_awareness.py:79-234`, `gptme/telemetry.py:362-442`

#### `app-server` — 1 harnais, cout M

- **harnais demandeurs** : codex
- **adaptateurs proposes** : `plugins/app-server-jsonrpc`
- **exemplaire de reference** : `codex` — *Frontiere Rust/TS generee (app-server)* (port `AppServer`, adaptateur `plugins/app-server-jsonrpc`, cout M)
  - comment il le fait : Crate app-server-protocol : types derives Serialize/Deserialize/JsonSchema/TS, generate_ts_with_options() / generate_json() pour les exports TypeScript, precomputed_exports + schema_fixtures en CI ; protos v2 (shared : SandboxMode...) ; transport app-server-transport + daemon.
  - preuves : `codex-rs/app-server-protocol/src/export.rs:123`, `codex-rs/app-server-protocol/src/export.rs:202`, `codex-rs/app-server-protocol/src/schema_fixtures.rs:1`

#### `cloud-runner` — 1 harnais, cout L

- **harnais demandeurs** : cursor
- **adaptateurs proposes** : `plugins/cloud-runner`
- **exemplaire de reference** : `cursor` — *Cloud Agents : VM isolees + environnements + artefacts* (port `CloudRunner`, adaptateur `plugins/cloud-runner`, cout L)
  - comment il le fait : [documente] VM isolee par agent : clone, dependances, secrets, reseau ; environnements via setup agentique, snapshot ou Dockerfile ; builds preparatoires ; declenchement iOS/web/desktop/Slack/GitHub/Linear/API ; artefacts + bureau distant ; anciennement Background Agents ; multi-repo possible.
  - preuves : `https://cursor.com/docs/cloud-agent.md`, `https://cursor.com/docs/subagents.md#cloud-subagents`, `docs/omega/research/HARNESS-LANDSCAPE.md`

#### `code-index` — 1 harnais, cout M

- **harnais demandeurs** : cursor
- **adaptateurs proposes** : `plugins/code-index-local`
- **exemplaire de reference** : `cursor` — *Index du codebase 100 % local (privacy)* (port `CodeIndex`, adaptateur `plugins/code-index-local`, cout M)
  - comment il le fait : [documente] Index construit et interroge sur la machine ; ni chemins ni embeddings du code envoyes pour l'indexation. Seul le contenu des fichiers ouverts par l'Agent peut rejoindre la requete modele (cf. Data Use). [infere] Structure d'index incrementale surveillant le filesystem.
  - preuves : `https://cursor.com/docs/context/codebase-indexing`, `docs/omega/research/HARNESS-LANDSCAPE.md`

#### `code-review` — 1 harnais, cout L

- **harnais demandeurs** : devin
- **adaptateurs proposes** : `plugins/review-assistant`
- **exemplaire de reference** : `devin` — *Devin Review (revue de PR augmentee + auto-review + auto-fix)* (port `Reviewer (diff/bugs/security/auto-fix)`, adaptateur `plugins/review-assistant`, cout L)
  - comment il le fait : Documente : diffs organises logiquement, detection copie/deplacement, bug catcher (severe/non-severe), scan securite (CWE, critique/warning, non desactivable), chat codebase-aware, edition de code depuis le chat, actions merge/close/draft/auto-merge, stacked PRs (diff par couche, merge atomique), auto-review (declencheurs, modes, self-enrollment, limites de depense ACU par PR), auto-fix (bots allo
  - preuves : `https://docs.devin.ai/work-with-devin/devin-review.md`, `https://docs.devin.ai/work-with-devin/stacked-prs.md`

#### `code-search` — 1 harnais, cout M

- **harnais demandeurs** : cursor
- **adaptateurs proposes** : `plugins/grep-local`
- **exemplaire de reference** : `cursor` — *Instant Grep : moteur de recherche exact local* (port `CodeSearch`, adaptateur `plugins/grep-local`, cout M)
  - comment il le fait : [documente] Moteur maison, regex + word-boundary, presente comme plus rapide que ripgrep sur gros codebases ; tourne sans configuration ; l'Agent construit des motifs (import.*PaymentService) et fouille les references. Index construit et requete localement.
  - preuves : `https://cursor.com/docs/context/codebase-indexing`, `docs/omega/research/HARNESS-LANDSCAPE.md`

#### `control-ui` — 1 harnais, cout M

- **harnais demandeurs** : openclaw
- **adaptateurs proposes** : `plugins/control-ui (descripteurs + session actions)`
- **exemplaire de reference** : `openclaw` — *Control UI + TUI + RPC pilotées par plugins* (port `ControlUi (describe + session actions/extensions)`, adaptateur `plugins/control-ui (descripteurs + session actions)`, cout M)
  - comment il le fait : Descripteurs de control UI et widgets de board declares par les plugins (controlUi/dashboard dans le manifeste, valides contre les enregistrements runtime), actions et extensions de session enregistrables, UI React (ui/src) + TUI embarquee (src/tui/commands.ts) + routes HTTP RPC (registerHttpRoute, registerGatewayMethod).
  - preuves : `RE-OPENCLAW-HERMES.md`, `src/plugins/plugin-api.types.ts:116-167`, `src/plugins/manifest-types.ts:520-535`, `src/tui/commands.ts:1`

#### `inline-complete` — 1 harnais, cout M

- **harnais demandeurs** : cursor
- **adaptateurs proposes** : `plugins/inline-complete`
- **exemplaire de reference** : `cursor` — *Tab : autocompletion multi-lignes + sauts predictifs* (port `InlineComplete`, adaptateur `plugins/inline-complete`, cout M)
  - comment il le fait : [documente] Suggestion grisee acceptee par Tab (mot-a-mot via Cmd/Ctrl+Fleche), editions multi-lignes avec imports manquants, sauts predictifs (jump-in-file) et portails inter-fichiers ; reglages par extension ; [infere] modele dedie cote serveur, desactive par l'indicateur Tab. Les rules n'affectent pas Tab.
  - preuves : `https://cursor.com/docs/tab/overview`, `https://cursor.com/docs/rules (FAQ : rules sans effet sur Tab)`, `docs/omega/research/HARNESS-LANDSCAPE.md`

#### `playbook-store` — 1 harnais, cout S

- **harnais demandeurs** : devin
- **adaptateurs proposes** : `plugins/playbook-store`
- **exemplaire de reference** : `devin` — *Playbooks (prompts reutilisables + macros + versions)* (port `PlaybookStore (list/attach/version)`, adaptateur `plugins/playbook-store`, cout S)
  - comment il le fait : Documente : playbook = prompt reutilisable partageable (Overview, Procedure imperative MECE, Specifications, Advice, Forbidden Actions, Required from User), macros !nom, historique de versions avec revert, scopes organisation/entreprise/systeme, attachement par macro dans le prompt d'une session ; Devin peut generer/ameliorer des playbooks depuis des sessions reussies/ratees via Advanced Capabilit
  - preuves : `https://docs.devin.ai/product-guides/creating-playbooks.md`, `https://docs.devin.ai/product-guides/using-playbooks.md`, `https://docs.devin.ai/work-with-devin/advanced-capabilities.md`

#### `task-handoff` — 1 harnais, cout M

- **harnais demandeurs** : devin
- **adaptateurs proposes** : `plugins/handoff-bridge`
- **exemplaire de reference** : `devin` — *API + CLI + /handoff bidirectionnel + ACP/SSH + Outposts* (port `Handoff (package/launch/reattach)`, adaptateur `plugins/handoff-bridge`, cout M)
  - comment il le fait : Documente : /handoff du CLI embarque contexte conversation + branche + diff non commite (100 Ko) vers une session cloud sur VM fraiche, et inversement recupere la branche PR cloud en local ; plugin open-source devin-handoff (repo+branche+git diff) pour Claude Code/Codex/Cursor ; SSH (devin ssh/forward/scp), subagents CLI, ACP (JetBrains/Zed/Xcode), sandbox CLI, routing Adaptive/Fusion ; API v1/v2/
  - preuves : `https://docs.devin.ai/work-with-devin/devin-handoff.md`, `https://docs.devin.ai/cli/handoff.md`, `https://docs.devin.ai/cli/index.md`, `https://github.com/club-cog/devin-handoff`

#### `task-planner` — 1 harnais, cout M

- **harnais demandeurs** : windsurf
- **adaptateurs proposes** : `plugins/task-planner-cascade`
- **exemplaire de reference** : `windsurf` — *Plan continu + Todo list (agent de planification en tâche de fond)* (port `TaskPlanner (refine/track)`, adaptateur `plugins/task-planner-cascade`, cout M)
  - comment il le fait : [documenté] Un agent de planification spécialisé raffine le plan long terme en arrière-plan pendant que le modèle sélectionné exécute les actions court terme ; la Todo list est affichée dans la conversation et modifiable en langage naturel ; le plan s'auto-actualise à l'arrivée d'une Memory. Aucun détail d'implémentation interne n'est publié (binaire fermé).
  - preuves : `https://docs.devin.ai/desktop/cascade/cascade.md`

#### `workflow-engine` — 1 harnais, cout L

- **harnais demandeurs** : devin
- **adaptateurs proposes** : `plugins/workflow-dag`
- **exemplaire de reference** : `devin` — *Dynamic Workflows (orchestration deterministe + reprise)* (port `WorkflowEngine (register/agent/pipeline/replay)`, adaptateur `plugins/workflow-dag`, cout L)
  - comment il le fait : Documente : Devin ecrit un script Python ordinaire utilisant les primitives register_workflow/agent/pipeline/parallel/log ; chaque appel agent() est cle par un hash (prompt, schema, reglages) et rejoue son resultat enregistre a la reprise ; les agents tournent par defaut sur VM separee (echange via branches git) ou en VM partagee ; budget max 7 jours. Infere : orchestrateur deterministe + magasin 
  - preuves : `https://docs.devin.ai/work-with-devin/dynamic-workflows.md`, `https://docs.devin.ai/product-guides/skills.md`

#### `workflow-runner` — 1 harnais, cout S

- **harnais demandeurs** : windsurf
- **adaptateurs proposes** : `plugins/workflow-runner`
- **exemplaire de reference** : `windsurf` — *Workflows markdown invocables en /slash (manuels uniquement)* (port `WorkflowRunner (discover/invoke)`, adaptateur `plugins/workflow-runner`, cout S)
  - comment il le fait : [documenté] Markdown dans .devin/workflows/ (préféré, héritage .windsurf/) + globaux ~/.codeium/windsurf/global_workflows/ + système Entreprise + built-in (/plan) ; invocation manuelle exclusive via /nom (jamais auto-invoqué, à l'inverse des Skills) ; composables (un workflow en appelle d'autres) ; précédence système > workspace > global > built-in ; 12 000 car. par fichier.
  - preuves : `https://docs.devin.ai/desktop/cascade/workflows.md`

### Configuration & extension

#### `config-source` — 10 harnais, cout S

- **harnais demandeurs** : aider, claude-code, codex, cursor, hermes, opencode, plandex, rea, windsurf, zed
- **popularite cumulee** : 522,782 etoiles
- **adaptateurs proposes** : `omega-config`, `omega-config (agents-md discovery)`, `omega-config (couches + precedence + policy managée)`, `omega-config (couches + precedence)`, `omega-config (profils)`, `omega-config (rules layers)`, `plugins/agent-profiles`, `plugins/rules-inject`, `plugins/rules-scoped`
- **exemplaire de reference** : `plandex` — *CLI Cobra + serveur HTTP + config partagée* (port `ConfigSource (layers)`, adaptateur `omega-config`, cout M)
  - comment il le fait : ~50 sous-commandes Cobra (app/cli/cmd : new, tell, build, apply, branches, rewind, context_show, set_model…) + serveur HTTP (app/server/routes/routes.go, main.go) + configuration partagée plandex-shared (plan_config.go, plan_model_settings.go) ; hooks serveur (DidSendModelRequest) pour la télémétrie.
  - preuves : `app/cli/cmd/root.go`, `app/server/routes/routes.go`, `app/server/main.go`, `app/shared/plan_config.go`

#### `registry` — 5 harnais, cout S

- **harnais demandeurs** : aider, claude-code, cline, goose, opencode
- **popularite cumulee** : 540,978 etoiles
- **adaptateurs proposes** : `omega-plugin-api (Transform rejouable + Hook live + reload par domaine)`, `omega-registry`, `omega-registry (Scope : fermeture = revelation de la couche precedente)`, `omega-registry (empreinte source + reload)`, `omega-registry (manifeste + marketplace resolver + scopes + namespacing)`, `omega-registry (slots modeles)`
- **exemplaire de reference** : `aider` — *Registre de modeles + routage fort/faible* (port `Registre (model slots)`, adaptateur `omega-registry (slots modeles)`, cout M)
  - comment il le fait : ModelSettings (edit_format, use_repo_map, cache_control, accepts_settings) resolus par ModelInfoManager (cache JSON + litellm.get_model_info + OpenRouter) avec heuristiques par nom de modele ; weak_model pour les taches cheap, editor_model pour l'application ; token_counter et completion_cost via litellm.
  - preuves : `aider/models.py:128-160`, `aider/models.py:161-280`, `aider/models.py:329-370`, `aider/models.py:437-494`

#### `extension-host` — 2 harnais, cout M

- **harnais demandeurs** : goose, openhands
- **popularite cumulee** : 146,428 etoiles
- **adaptateurs proposes** : `plugins/extension-host`
- **exemplaire de reference** : `openhands` — *Plugins + hooks + extensions Canvas* (port `ExtensionHost (install/enable/hook)`, adaptateur `plugins/extension-host`, cout L)
  - comment il le fait : SDK : Plugin (manifeste, sources, fetch strict, installation) + HookManager/HookExecutor (evenements, decisions) ; serveur : routers plugins (fetch/install/enable/marketplace) et canvas_extensions (manifeste, backend manage, bridge, bundle) ; UI : chargement de modules d'extension.
  - preuves : `sdk/openhands-sdk/openhands/sdk/plugin/plugin.py:36`, `sdk/openhands-sdk/openhands/sdk/plugin/types.py:242`, `sdk/openhands-sdk/openhands/sdk/hooks/manager.py:22`, `sdk/openhands-sdk/openhands/sdk/hooks/types.py:9-35`

#### `plugin-manifest` — 1 harnais, cout M

- **harnais demandeurs** : openclaw
- **adaptateurs proposes** : `plugins/plugin-manifest (serde + JSON Schema)`
- **exemplaire de reference** : `openclaw` — *Manifeste declaratif + policy-par-code* (port `PluginManifest (validate + inspect sans executer)`, adaptateur `plugins/plugin-manifest (serde + JSON Schema)`, cout M)
  - comment il le fait : Chaque plugin embarque openclaw.plugin.json : configSchema (JSON Schema), activation/setup/doctorContract lisibles sans executer le code du plugin, champ contracts (snapshot statique des capacites revendiquees), entree scellee par definePluginEntry ; l'activation-planner decide du chargement sur les seules metadonnees.
  - preuves : `RE-OPENCLAW-HERMES.md`, `src/plugins/manifest-types.ts:429,503-535`, `src/plugin-sdk/plugin-entry.ts:244-260`, `docs/plugins/manifest.md:48-60`

### Contexte & memoire de travail

#### `context-engine` — 13 harnais, cout S

- **harnais demandeurs** : aider, claude-code, cline, codex, goose, gptme, hermes, openclaw, opencode, openhands, plandex, windsurf, zed
- **popularite cumulee** : 744,090 etoiles
- **adaptateurs proposes** : `omega-context (ContextBudget estimate/compact + resume structure)`, `omega-context (ContextBudget estimate/compact + résumé structuré + réhydratation ciblée)`, `omega-context::DefaultContextEngine`, `omega-context::FastRetriever`, `omega-context::PromptCacheTelemetry`, `omega-context::truncate_compact`, `omega-context::truncate_compact (partiel)`, `plugins/agents-md-merge`, `plugins/context-autoload`, `plugins/context-compress`, `plugins/context-condenser`, `plugins/context-per-step`, `plugins/convo-summarize`, `plugins/project-map`, `plugins/prompt-cache`, `plugins/prompt-cache-policy (breakpoints ≤ 4, ordre tools→system→messages, TTL 5m/1h, affinité parent/fork/self)`, `plugins/prompt-cache-policy (breakpoints<=4, TTL 5m/1h)`, `plugins/repomap-pagerank`, `plugins/thread-from-summary`
- **exemplaire de reference** : `aider` — *Repo map tree-sitter + PageRank sous budget* (port `ContextEngine (rank/compact)`, adaptateur `plugins/repomap-pagerank`, cout L)
  - comment il le fait : Extraction tree-sitter (requetes .scm par langage, get_tags_raw) en Tags def/ref ; graphe MultiDiGraph referencer->definer pondere (mention x10, snake/camel>=8 chars x10, prive x0.1, >5 definers x0.1, fichier en chat x50, sqrt des refs) ; nx.pagerank avec personnalisation vers fichiers du chat ; rank distribue aux definitions, rendu arborescent sous budget map_tokens, cache SQLite des tags.
  - preuves : `aider/repomap.py:19-29`, `aider/repomap.py:42-103`, `aider/repomap.py:217-233`, `aider/repomap.py:279-328`

#### `memory-store` — 7 harnais, cout S

- **harnais demandeurs** : claude-code, codex, devin, gptme, hermes, openclaw, windsurf
- **popularite cumulee** : 155,705 etoiles
- **adaptateurs proposes** : `plugins/memory-bounded`, `plugins/memory-bounded (CLAUDE.md layers + MEMORY.md 25KB + audit)`, `plugins/memory-fts`, `plugins/memory-layered`, `plugins/memory-local`
- **exemplaire de reference** : `gptme` — *Mémoire en couches partagée avec Claude Code* (port `MemoryStore (recall/add/replace/patch/remove)`, adaptateur `plugins/memory-layered`, cout M)
  - comment il le fait : resolve_roots() ordonne les couches : explicit > project (<workspace>/memory) > cc (~/.claude/projects/<hash>/memory, toujours present) > agent > user (gptme/memory/roots.py:57-97) ; get_cc_memory_dir() recalcule le hash de projet Claude Code (gptme/dirs.py:205-295) ; MemoryStore.save/supersede/audit avec ecriture atomique et verrou (gptme/memory/store.py:244-478) ; default_write_root() choisit la
  - preuves : `gptme/memory/roots.py:1-97`, `gptme/dirs.py:205-295`, `gptme/memory/store.py:244-478`, `gptme/memory/match.py:1-60`

#### `skill-store` — 6 harnais, cout S

- **harnais demandeurs** : devin, goose, hermes, open-interpreter, rea, windsurf
- **popularite cumulee** : 123,938 etoiles
- **adaptateurs proposes** : `omega-context::SkillIndex`, `plugins/harness-skill-bridge`, `plugins/skill-evolve`
- **exemplaire de reference** : `devin` — *Skills SKILL.md + Knowledge a triggers (+ migration)* (port `SkillStore (discover/load)`, adaptateur `omega-context::SkillIndex`, cout S)
  - comment il le fait : Documente : SKILL.md au standard Agent Skills decouverts dans 6 chemins (.agents/.devin/.github/.claude/.cognition/.windsurf), sources indexees + disque (re-scan au clone), frontmatter (name, description, allowed-tools, argument-hint, triggers), invocation auto ou @skills:nom + substitution $ARGUMENTS/$1..$9, une seule skill active a la fois, suggestion auto de skills par Devin avec bouton Create 
  - preuves : `https://docs.devin.ai/product-guides/skills.md`, `https://docs.devin.ai/onboard-devin/knowledge-onboarding.md`, `https://docs.devin.ai/product-guides/plugins.md`, `https://agentskills.io/specification`

#### `conversation-store` — 1 harnais, cout L

- **harnais demandeurs** : openhands
- **popularite cumulee** : 91,079 etoiles
- **adaptateurs proposes** : `plugins/conversation-lifecycle`
- **exemplaire de reference** : `openhands` — *Cycle de vie des conversations* (port `ConversationStore (start/pause/fork/navigate)`, adaptateur `plugins/conversation-lifecycle`, cout L)
  - comment il le fait : Router FastAPI (search/count/get/start/pause/interrupt/delete/run) + registre ConversationRegistry abstrait ; implementations LocalConversation (persistee, fork/navigate/condense/goal) et RemoteConversation ; RunSlot/semaphore limite les runs concurrents.
  - preuves : `sdk/openhands-agent-server/openhands/agent_server/conversation_router.py:158-835`, `sdk/openhands-agent-server/openhands/agent_server/conversation_registry.py:21`, `sdk/openhands-sdk/openhands/sdk/conversation/impl/local_conversation.py:180-974`, `sdk/openhands-agent-server/openhands/agent_server/event_service.py:104-179`

#### `memory-search` — 1 harnais, cout M

- **harnais demandeurs** : hermes
- **adaptateurs proposes** : `plugins/memory-fts`
- **exemplaire de reference** : `hermes` — *Recherche de sessions FTS5* (port `MemorySearch (query)`, adaptateur `plugins/memory-fts`, cout M)
  - comment il le fait : Table FTS5 alimentée en écriture ; requêtes lexicales.
  - preuves : `RE-OPENCLAW-HERMES.md`

### Execution & securite

#### `sandbox-backend` — 8 harnais, cout M

- **harnais demandeurs** : claude-code, devin, gptme, hermes, openhands, plandex, rea, windsurf
- **popularite cumulee** : 262,534 etoiles
- **adaptateurs proposes** : `omega-sandbox::LocalSandboxBackend`, `plugins/diff-sandbox`, `plugins/sandbox-docker`, `plugins/sandbox-docker-wasmtime`, `plugins/sandbox-rea-capture`, `plugins/sandbox-worktree (git worktree + vérification cwd/commande + cleanup)`, `plugins/sandbox-{docker,ssh,modal,…}`, `plugins/worktree-isolated`
- **exemplaire de reference** : `gptme` — *Sandbox poly-backend (firejail/bwrap/docker/wasmtime)* (port `SandboxBackend (execute/fetch_file/cleanup)`, adaptateur `plugins/sandbox-docker-wasmtime`, cout L)
  - comment il le fait : SandboxConfig choisit le backend via GPTME_SANDBOX (firejail|bwrap|docker|wasmtime|none) (gptme/sandbox.py:91-146) ; wrap_shell_cmd/_firejail_cmd/_bwrap_cmd confinent le shell (gptme/sandbox.py:167-403) ; sandbox_exec_python execute le code dans docker (gptme/sandbox.py:404-516) ; sandbox_exec_wasmtime execute CPython WASI via wasmtime-py (gptme/sandbox.py:517-653) ; le tool python bascule automat
  - preuves : `gptme/sandbox.py:91-146`, `gptme/sandbox.py:167-403`, `gptme/sandbox.py:404-516`, `gptme/sandbox.py:517-653`

#### `exec-policy` — 3 harnais, cout S

- **harnais demandeurs** : aider, openclaw, rea
- **popularite cumulee** : 49,587 etoiles
- **adaptateurs proposes** : `omega-sandbox::RuleBasedExecPolicy`, `plugins/reflect-lint-test`
- **exemplaire de reference** : `openclaw` — *Audit de securite extensible + scans* (port `ExecPolicy (allow/deny + audit collectors)`, adaptateur `omega-sandbox::RuleBasedExecPolicy`, cout M)
  - comment il le fait : Collecteurs d'audit de securite enregistrables (securityAuditCollectors dans definePluginEntry), audits dedies (config/symlinks/perms, surface d'exec, sandbox host, cross-agent session access, filesystem), scan des secrets et profils d'auth, magasins d'audit ; politique reseau (net-policy) et contrats de packages plugin (plugin-package-contract).
  - preuves : `RE-OPENCLAW-HERMES.md`, `src/plugin-sdk/plugin-entry.ts:244-260`, `src/security/audit-exec-surface.test.ts:1`, `src/security/audit-config-symlink.test.ts:1`

#### `permission-policy` — 2 harnais, cout M

- **harnais demandeurs** : devin, goose
- **popularite cumulee** : 55,349 etoiles
- **adaptateurs proposes** : `omega-sandbox::ApprovalQueue`
- **exemplaire de reference** : `devin` — *Security profiles + secrets + OIDC + guardrails* (port `PermissionPolicy + SandboxBackend`, adaptateur `omega-sandbox::ApprovalQueue`, cout M)
  - comment il le fait : Documente : profils nommaux (network allowlist hostnames/CIDR, allowlist MCP, git read-only/full, retrait token gh, Devin MCP read-only) lies en hierarchie (entreprise > organisation > automations > automation > session, intersection en mode mandatory, resolution au boot/wake) ; secrets (organisation/perso/repo/session, cookies, TOTP) ; OIDC workload identity (AWS/GCP/Vault/JFrog/Databricks) ; AI 
  - preuves : `https://docs.devin.ai/product-guides/security-profiles.md`, `https://docs.devin.ai/product-guides/secrets.md`, `https://docs.devin.ai/product-guides/oidc.md`, `https://docs.devin.ai/enterprise/features/ai-guardrails.md`

#### `checkpoint-store` — 2 harnais, cout M

- **harnais demandeurs** : cursor, windsurf
- **adaptateurs proposes** : `plugins/checkpoint-local`, `plugins/checkpoint-store`
- **exemplaire de reference** : `cursor` — *Checkpoints : snapshots locaux + revert* (port `CheckpointStore`, adaptateur `plugins/checkpoint-local`, cout M)
  - comment il le fait : [documente] Snapshots automatiques avant modifications significatives ; clic sur un checkpoint = apercu puis restauration des fichiers ; restauration = fichiers seuls (messages conserves). Stockes localement, hors Git.
  - preuves : `https://cursor.com/docs/agent/overview`, `docs/omega/research/HARNESS-LANDSCAPE.md`

#### `(execution)` — 1 harnais, cout M

- **harnais demandeurs** : opencode
- **popularite cumulee** : 214,363 etoiles
- **adaptateurs proposes** : `omega-server (cockpit + /api/health|plugins|run|trace)`
- **exemplaire de reference** : `opencode` — *Serveur + protocole HTTP/OpenAPI + surfaces TUI/desktop* (port `—`, adaptateur `omega-server (cockpit + /api/health|plugins|run|trace)`, cout M)
  - comment il le fait : packages/protocol : groupes agent/command/config/credential/event/session/model/provider/skill/mcp/plugin/... exposes en HttpApiEndpoint (plugin.list GET /api/plugin, plugin.check, plugin.update concurrent + notify reload) ; server assemble HttpApi ; TUI/desktop/app consomment la meme API ; plugin declare ses surfaces via Features.
  - preuves : `packages/protocol/src/groups/plugin.ts:10-61`, `packages/server/src/api.ts`, `packages/schema/src/plugin.ts:24-30`, `specs/v2/README.md:30-42`

#### `worktree-isolate` — 1 harnais, cout L

- **harnais demandeurs** : zed
- **popularite cumulee** : 91,830 etoiles
- **adaptateurs proposes** : `plugins/worktree-isolate`
- **exemplaire de reference** : `zed` — *Isolation par worktree par fil* (port `WorktreeIsolate (spawn/archive/restore)`, adaptateur `plugins/worktree-isolate`, cout L)
  - comment il le fait : ThreadOptions.use_new_worktree + worktree_name + base ref (thread.rs:843-850) : le fil nait dans un workspace git worktree dedie ; build_root_plan / workspaces_for_archive (thread_worktree_archive.rs:112,928) et restore_archive_checkpoint (thread_worktree_archive.rs:778) archivent et restaurent l'etat du worktree.
  - preuves : `crates/agent/src/thread.rs:843-865`, `crates/agent_ui/src/thread_worktree_archive.rs:112-130`, `crates/agent_ui/src/thread_worktree_archive.rs:778-789`

#### `worktree-manager` — 1 harnais, cout M

- **harnais demandeurs** : cursor
- **adaptateurs proposes** : `plugins/worktree-git`
- **exemplaire de reference** : `cursor` — *Isolation des sous-agents par worktree Git* (port `WorktreeManager`, adaptateur `plugins/worktree-git`, cout M)
  - comment il le fait : [documente] Chaque sous-agent isole recoit sa copie : worktree Git + branche sur la meme machine, ou VM + clone dedies dans le cloud ; le parent fusionne. Worktrees desactives en multi-root et pour Cloud Agents multi-root.
  - preuves : `https://cursor.com/docs/subagents.md#isolated-project-copies`, `https://cursor.com/docs/configuration/worktrees.md`, `https://cursor.com/docs/context/codebase-indexing`, `docs/omega/research/HARNESS-LANDSCAPE.md`

### Observabilite & tracabilite

#### `event-store` — 4 harnais, cout M

- **harnais demandeurs** : claude-code, devin, open-interpreter, opencode
- **popularite cumulee** : 434,204 etoiles
- **adaptateurs proposes** : `omega-app::FileEventStore + omega-otel::TraceReplayer`, `plugins/hook-bus (matcher + if + décision exit-code/JSON + async)`, `plugins/session-persistent`, `plugins/session-rollout`
- **exemplaire de reference** : `open-interpreter` — *Sessions persistantes + journal rejouable + memories* (port `EventStore`, adaptateur `plugins/session-rollout`, cout M)
  - comment il le fait : Conversations stockees sous ~/.openinterpreter/ : resume --last / picker / fork / rename (docs/sessions.md) ; journal JSONL par session lu via decode_rollout_line / parse_rollout_line (rollout/src/lib.rs:50-80) avec maintenance et compression ; memories opt-in (features.memories) avec extraction/consolidation par modele dedie (memories.extract_model / consolidation_model) et injection des souvenir
  - preuves : `docs/sessions.md:1-60`, `docs/memories.md:1-40`, `codex-rs/rollout/src/lib.rs:50-80`, `codex-rs/rollout/src/maintenance.rs`

#### `telemetry-sink` — 3 harnais, cout S

- **harnais demandeurs** : aider, devin, plandex
- **popularite cumulee** : 65,337 etoiles
- **adaptateurs proposes** : `omega-otel::StdoutSink`, `plugins/eval-harness`
- **exemplaire de reference** : `devin` — *Session Insights + metering ACU + preuves video* (port `TelemetrySink + TraceReplayer`, adaptateur `omega-otel::StdoutSink`, cout S)
  - comment il le fait : Documente : Session Insights (metriques ACU/user-messages/taille XS-XL/categorie, timeline d'incidents rouge/jaune/vert, prompt ameliore avec diffs, action items machine/repo, usage knowledge utile vs trompeuse, generation auto pour sessions L/XL, API de generation) ; ACU (facturation, limites org/utilisateur, coach d'efficacite, analytics perso) ; testing & recordings video comme preuve ; Session
  - preuves : `https://docs.devin.ai/product-guides/session-insights.md`, `https://docs.devin.ai/admin/billing/usage.md`, `https://docs.devin.ai/work-with-devin/testing-and-recordings.md`, `https://docs.devin.ai/work-with-devin/security-swarm.md`

#### `ui-panel` — 1 harnais, cout M

- **harnais demandeurs** : zed
- **popularite cumulee** : 91,830 etoiles
- **adaptateurs proposes** : `plugins/agent-panel`
- **exemplaire de reference** : `zed` — *Agent Panel (fils, selecteurs, diff)* (port `AgentPanel (threads/selectors)`, adaptateur `plugins/agent-panel`, cout M)
  - comment il le fait : AgentPanel (agent_panel.rs:1116) : new_thread (agent_panel.rs:1753), new_native_agent_thread_from_summary (agent_panel.rs:3490), threads_archive_view, model/profile selectors (model_selector.rs, profile_selector.rs), AgentDiffPane (agent_diff.rs:41) pour la revue des modifications, inline_assistant pour l'edition inline.
  - preuves : `crates/agent_ui/src/agent_panel.rs:1116`, `crates/agent_ui/src/agent_panel.rs:1753`, `crates/agent_ui/src/agent_diff.rs:41-83`, `https://docs.zed.dev/ai/agent-panel`

#### `ui-shell` — 1 harnais, cout L

- **harnais demandeurs** : openhands
- **popularite cumulee** : 91,079 etoiles
- **adaptateurs proposes** : `plugins/canvas-ui`
- **exemplaire de reference** : `openhands` — *Canvas UI (chat, terminal, navigateur, fichiers)* (port `UiShell (views/actions)`, adaptateur `plugins/canvas-ui`, cout L)
  - comment il le fait : React 19 + Router 7 + Zustand + React Query : vues conversation/chat/terminal/navigateur/fichiers/reglages/automations ; adaptateur traduisant les actions UI en REST (toAppConversation, suffixes systeme runtime_services) ; entrypoints librairie (build:lib) ; mocks MSW ; client-tools agent->UI (canvas-ui, request-secret, automation-form).
  - preuves : `canvas/docs/architecture.md`, `canvas/src/api/agent-server-adapter.ts:202-414`, `canvas/src/api/conversation-service/agent-server-conversation-service.api.ts:431-1258`, `canvas/src/api/event-service/event-service.api.ts:157`

#### `ui-surface` — 1 harnais, cout M

- **harnais demandeurs** : open-interpreter
- **popularite cumulee** : 68,589 etoiles
- **adaptateurs proposes** : `plugins/cli-tui-surface`
- **exemplaire de reference** : `open-interpreter` — *CLI + exec non-interactif + TUI a slash commands* (port `UiSurface (exec/interact)`, adaptateur `plugins/cli-tui-surface`, cout M)
  - comment il le fait : interpreter exec 'tache' (stdin -, git diff | ..., -i image, sortie JSON) pour scripts/CI (docs/exec.md) ; TUI plein ecran avec slash commands adaptees au mode (/model, /fast, /permissions, /approvals, /new, /resume, /fork, /compact, /agent, /init, /review, /mcp, /skills, /memories, /hooks...) documentees dans docs/slash_commands.md ; bascule de harnais persistee via config_update (tui/src/app/con
  - preuves : `docs/exec.md:1-50`, `docs/slash_commands.md:1-80`, `codex-rs/tui/src/app/config_persistence.rs`, `codex-rs/tui/src/config_update.rs:122-123`

#### `(observabilite)` — 1 harnais, cout S

- **harnais demandeurs** : goose
- **popularite cumulee** : 55,349 etoiles
- **exemplaire de reference** : `goose` — *Docteur auto-diagnostic* (port `—`, cout S)
  - comment il le fait : Observe : doctor run(session) (doctor.rs:22) diagnostique config/providers/extensions et rend un Message ; generate_diagnostics (session/diagnostics.rs:297). Auto-diagnostic actionnable, pas juste des logs.
  - preuves : `crates/goose/src/doctor.rs:22`, `crates/goose/src/session/diagnostics.rs:297`

#### `provenance` — 1 harnais, cout M

- **harnais demandeurs** : rea
- **adaptateurs proposes** : `plugins/tool-rea-evidence`
- **exemplaire de reference** : `rea` — *Contrat d'évidence canonique (preuves + limites)* (port `Provenance`, adaptateur `plugins/tool-rea-evidence`, cout M)
  - comment il le fait : Schéma zod evidenceSchema (superRefine) + parseEvidence/immutableEvidence ; text+structuredContent ; raw_result vs normalized_result ; budgets transport 10 MiB, export streaming, vues paginées inspect_analysis_view ; erreurs canoniques isError + error-contract.schema.json.
  - preuves : `src/domain/evidence.ts:37-291`, `docs/mcp-contracts.md:9-150`, `docs/error-contract.schema.json:1-60`

### Orchestration

#### `subagent-runner` — 13 harnais, cout S

- **harnais demandeurs** : aider, claude-code, cline, codex, cursor, devin, gptme, hermes, open-interpreter, openclaw, opencode, openhands, zed
- **popularite cumulee** : 741,580 etoiles
- **adaptateurs proposes** : `omega-server (run non-interactif + SDK surface)`, `plugins/architect-loop`, `plugins/subagent-explore`, `plugins/subagent-isolated`, `plugins/subagent-isolated (background + notify) + plugins/agent-team`, `plugins/subagent-isolated (spawn/resume, isolation worktree)`, `plugins/subagent-isolated (spawn/resume, isolation worktree, modèle/effort/TTL par invocation)`
- **exemplaire de reference** : `openclaw` — *Sous-agents isoles + node host Rust* (port `SubagentRunner (spawn/rollback/onEnded + modes)`, adaptateur `plugins/subagent-isolated`, cout M)
  - comment il le fait : Hooks prepareSubagentSpawn (preparation du contexte enfant, modes isolated|fork, rollback annulable) et onSubagentEnded (raisons deleted/completed/swept/released) ; cles de session et spawn >= contexte (agent-runtime-session-spawn-context) ; runtime worker embarque (embedded-agent.runtime) et node host Rust (duplex, reconnect, handshake sidecar, lifecycle).
  - preuves : `RE-OPENCLAW-HERMES.md`, `src/context-engine/types.ts:206-238,522-536`, `src/gateway/agent-runtime-session-spawn-context.ts:1`, `crates/openclaw-node-host/src/sidecar_protocol.rs`

#### `scheduler-job` — 8 harnais, cout M

- **harnais demandeurs** : aider, claude-code, cline, devin, gptme, hermes, openclaw, openhands
- **popularite cumulee** : 366,798 etoiles
- **adaptateurs proposes** : `plugins/file-watcher`, `plugins/scheduler-cron`, `plugins/scheduler-cron (local + cloud-runner + skill verify/simplify pre-commit)`, `plugins/scheduler-cron + plugins/channel-*`, `plugins/scheduler-cron + plugins/channel-dispatch`, `plugins/scheduler-service`
- **exemplaire de reference** : `openhands` — *Automations planifiees + webhooks (backend dedie)* (port `Scheduler (schedule/trigger)`, adaptateur `plugins/scheduler-cron`, cout M)
  - comment il le fait : Backend dedie openhands-automation (paquet PyPI versionne en sync avec le front) servant /api/automation (dont webhooks) ; le front tague les conversations (automationtrigger/id/name/run_id) et propage le contexte d'observabilite ; stack locale via uvx + ingress.
  - preuves : `canvas/src/api/automation-service/automation-service.api.ts:60-737`, `canvas/src/api/agent-server-adapter.ts:558-572`, `sdk/openhands-sdk/openhands/sdk/automation/__init__.py`, `canvas/scripts/check-sdk-version-sync.mjs:63-82`

#### `hook-bus` — 4 harnais, cout S

- **harnais demandeurs** : cursor, gptme, openclaw, windsurf
- **popularite cumulee** : 4,453 etoiles
- **adaptateurs proposes** : `plugins/hook-bus`, `plugins/hook-bus (bus async + budgets + ordre deterministe)`, `plugins/hooks`
- **exemplaire de reference** : `gptme` — *Bus de hooks typés (~20 types, sync+async)* (port `HookBus (register/trigger)`, adaptateur `plugins/hook-bus`, cout M)
  - comment il le fait : HookType enumere ~20 types en notation pointee (step.pre/post, tool.execute.pre/post, session.start/end, generation.*, loop.continue, tool.confirm, elicit...) (gptme/hooks/types.py:66-140) ; HookRegistry.register/trigger avec filtrage de kwargs, StopPropagation et hooks async (gptme/hooks/registry.py:109-329) ; ~25 modules de hooks concrets (confirm, guardrails, token/cost/cache awareness...) ; le
  - preuves : `gptme/hooks/types.py:66-140`, `gptme/hooks/registry.py:109-329`, `gptme/hooks/registry.py:434-607`, `gptme/plugins/registry.py:144-165`

#### `agent-server` — 3 harnais, cout M

- **harnais demandeurs** : goose, openhands, zed
- **popularite cumulee** : 238,258 etoiles
- **adaptateurs proposes** : `plugins/acp-bridge`, `plugins/agent-server-rest`
- **exemplaire de reference** : `zed` — *Agents externes via ACP (v1/v2)* (port `AgentServer (spawn/session)`, adaptateur `plugins/acp-bridge`, cout M)
  - comment il le fait : AgentServerDelegate (agent_servers.rs:31) + AcpSession (acp.rs:329, v2_terminal_client_builder acp.rs:340) : l'editeur pilote des agents externes (CLI ou binaires) qui parlent ACP v1 (v1.rs) / v2 (v2.rs) via transport (transport.rs) ; les dossiers acp/ (runtime, session_updates, terminal_updates) adaptent les mises a jour de session et de terminal.
  - preuves : `crates/agent_servers/src/agent_servers.rs:31`, `crates/agent_servers/src/acp.rs:329-340`, `https://docs.zed.dev/ai/external-agents`

#### `agent-store` — 2 harnais, cout S

- **harnais demandeurs** : claude-code, opencode
- **popularite cumulee** : 365,615 etoiles
- **adaptateurs proposes** : `plugins/agent-declarative (frontmatter YAML + Ruleset)`, `plugins/command-md`, `plugins/command-md (template + $args + argument-hint + overrides)`, `plugins/plan-mode`, `plugins/plan-mode (deny Write/Edit + délégation Plan + rappel)`
- **exemplaire de reference** : `opencode` — *Agents declaratifs (schema + frontmatter + permissions)* (port `AgentStore (discover/load)`, adaptateur `plugins/agent-declarative (frontmatter YAML + Ruleset)`, cout S)
  - comment il le fait : Schema Agent.Info : {id, name, model?, request, system?, description?, mode: subagent|primary|all, hidden, color?, steps?, permissions} ; constructeur statique default(id) posant allow * sauf read *.env -> ask ; plugins internes general/explore aux permissions restreintes ; agent build par defaut.
  - preuves : `packages/schema/src/agent.ts:23-53`, `packages/core/src/plugin/agent.ts:87-119`, `packages/core/src/agent.ts:13`, `packages/core/src/config/markdown.ts:1-40`

#### `agent-runtime` — 2 harnais, cout S

- **harnais demandeurs** : cline, goose
- **popularite cumulee** : 125,776 etoiles
- **adaptateurs proposes** : `plugins/agent-loop`
- **exemplaire de reference** : `goose` — *Subagents delegues bornes* (port `SubagentHost`, cout M)
  - comment il le fait : Observe : SubagentOperation<S> (subagent.rs:165) avec SubagentOutcome/SubagentUpdate (subagent.rs:24,32) ; SubagentPromptContext (subagent_handler.rs:29) construit le prompt delegue ; outil d'execution parallele avec notifications (subagent_execution_tool/notification_events.rs:41). Delegation bornee : pas de nested delegation (goose-self-test.yaml).
  - preuves : `crates/goose-agent/src/subagent.rs:165`, `crates/goose/src/agents/subagent_handler.rs:29`, `crates/goose/src/agents/subagent_execution_tool/notification_events.rs:41`

#### `agent-runner` — 2 harnais, cout M

- **harnais demandeurs** : cursor, zed
- **popularite cumulee** : 91,830 etoiles
- **adaptateurs proposes** : `plugins/agent-loop`
- **exemplaire de reference** : `zed` — *Boucle agent run_turn streaming annulable* (port `AgentRunner (run_turn/cancel)`, adaptateur `plugins/agent-loop`, cout M)
  - comment il le fait : Thread::run_turn (thread.rs:2843) vide le message partiel, annule le tour en vol, fige le resume, capture les outils actives puis delegue a run_turn_internal (thread.rs:2921) qui streame la completion ; RunningTurn (thread.rs:5090) porte prompt_id, event_stream, outils et canal d'annulation watch. Les evenements (arret/refus/max-tokens/erreur) sont emis via ThreadEventStream.
  - preuves : `crates/agent/src/thread.rs:2843-2920`, `crates/agent/src/thread.rs:5090-5130`

#### `channel` — 2 harnais, cout M

- **harnais demandeurs** : cline, openclaw
- **popularite cumulee** : 70,427 etoiles
- **adaptateurs proposes** : `plugins/channel-*`
- **exemplaire de reference** : `openclaw` — *Gateway multi-canal a canaux enregistrables* (port `Channel (register + config + garde-fous DM)`, adaptateur `plugins/channel-*`, cout M)
  - comment il le fait : Enregistrement declare (registerChannel + catalogue de canaux embarques + metadonnees de config), resolution de comptes et garde-fous DM (direct-dm-guard-policy, allow-from, allowlists), methodes gateway et politique d'acces gateway enregistrables par plugin ; 180 extensions dont telegram, discord, slack, signal, whatsapp.
  - preuves : `RE-OPENCLAW-HERMES.md`, `src/plugins/plugin-api.types.ts:257-283`, `src/channels/direct-dm-guard-policy.ts:1`, `src/channels/allow-from.ts:1`

#### `hook` — 2 harnais, cout M

- **harnais demandeurs** : cline, codex
- **popularite cumulee** : 70,427 etoiles
- **adaptateurs proposes** : `plugins/hook-command`, `plugins/hook-runner`
- **exemplaire de reference** : `codex` — *Hooks declaratifs types et abortables* (port `Hook`, adaptateur `plugins/hook-command`, cout M)
  - comment il le fait : HookResult { Success, FailedContinue(err), FailedAbort(err) } avec should_abort_operation() ; Hook = { name, HookFn } ; HookPayload { session_id, cwd, triggered_at, hook_event } ; 12 HookEventName (PreToolUse, PermissionRequest, PostToolUse, Pre/PostCompact, SessionStart/End, UserPromptSubmit, SubagentStart/Stop, Stop, Interrupt), 4 HookHandlerType (Command, McpTool, Prompt, Agent), modes Sync/Asy
  - preuves : `codex-rs/hooks/src/types.rs:15`, `codex-rs/hooks/src/types.rs:27`, `codex-rs/hooks/src/types.rs:39`, `codex-rs/hooks/src/types.rs:64`

#### `agent-loop` — 2 harnais, cout M

- **harnais demandeurs** : gptme, plandex
- **popularite cumulee** : 20,203 etoiles
- **adaptateurs proposes** : `plugins/agent-loop`, `plugins/agent-tell-build`
- **exemplaire de reference** : `gptme` — *Boucle agent ReAct step/turn + overflow recovery* (port `AgentLoop (chat/step/recover)`, adaptateur `plugins/agent-loop`, cout M)
  - comment il le fait : chat() orchestre la session (gptme/chat.py:212-397) ; _run_chat_loop() itere les tours (gptme/chat.py:397-586) ; step() = une generation LLM + execution d'outils (gptme/chat.py:1011-1145) ; _reply_with_overflow_recovery() regenere en reduisant le contexte si depassement (gptme/chat.py:948-1010) ; _run_post_tool_compaction() compacte apres outil si besoin (gptme/chat.py:798-824).
  - preuves : `gptme/chat.py:212-397`, `gptme/chat.py:397-586`, `gptme/chat.py:948-1010`, `gptme/chat.py:1011-1145`

#### `agent-host` — 1 harnais, cout M

- **harnais demandeurs** : openhands
- **popularite cumulee** : 91,079 etoiles
- **adaptateurs proposes** : `plugins/agent-acp`
- **exemplaire de reference** : `openhands` — *Agents tiers via ACP (Claude Code, Codex, Gemini, Pi, OpenCode)* (port `AgentHost (spawn/relay)`, adaptateur `plugins/agent-acp`, cout M)
  - comment il le fait : L'Agent Server engendre la CLI de l'agent tiers en sous-processus et relaie chaque tour en JSON-RPC sur stdio ; registre ACP_PROVIDERS (commandes npx, cles, logins OAuth detectes) ; decouverte de modeles via acp_router ; choix stocke par backend (agent_kind, acp_*).
  - preuves : `sdk/openhands-sdk/openhands/sdk/agent/acp_agent.py:504-637`, `sdk/openhands-sdk/openhands/sdk/settings/acp_providers.py:176-552`, `sdk/openhands-agent-server/openhands/agent_server/acp_router.py:150-175`, `canvas/docs/ACP_AGENTS.md`

#### `headless-run` — 1 harnais, cout M

- **harnais demandeurs** : cline
- **popularite cumulee** : 70,427 etoiles
- **adaptateurs proposes** : `plugins/headless-cli`
- **exemplaire de reference** : `cline` — *CLI headless/CI + TUI* (port `HeadlessRun (run/stream)`, adaptateur `plugins/headless-cli`, cout M)
  - comment il le fait : run-agent.ts:432 orchestre un run non-interactif (prompt, politiques d'outils, evenements session, format de sortie) ; run-interactive.ts/run-zen.ts pour le TUI, session-events.ts pour le streaming, program.ts definissant les commandes (run, chat, schedule, hook, mcp, skill, plugin, kanban, history, doctor) ; e2e couverts par cli.e2e.test.ts.
  - preuves : `apps/cli/src/runtime/run-agent.ts:1-60`, `apps/cli/src/commands/program.ts:1-60`, `apps/cli/src/main.ts:1-40`

#### `agent-protocol` — 1 harnais, cout M

- **harnais demandeurs** : open-interpreter
- **popularite cumulee** : 68,589 etoiles
- **adaptateurs proposes** : `plugins/acp-bridge`
- **exemplaire de reference** : `open-interpreter` — *Agent Client Protocol + MCP server inverse* (port `AcpServer (serve/request_permission)`, adaptateur `plugins/acp-bridge`, cout M)
  - comment il le fait : interpreter acp (cli/src/main.rs:1422,2791) parle ACP sur stdin/stdout pour les editeurs compatibles ; meme home/config/auth/providers/MCP/skills/sandbox/approvals que le TUI ; permissions remontees au client via RequestPermission (acp-server/src/lib.rs:915-927) avec options AllowOnce/AllowAlways/RejectOnce ; pendant modulaire : mcp-server embarque inversement exposant Open Interpreter comme outil
  - preuves : `docs/acp.md:1-50`, `codex-rs/acp-server/src/lib.rs:900-930`, `codex-rs/cli/src/main.rs:1422,2791`, `docs/mcp-server.md`

#### `lifecycle-hook` — 1 harnais, cout M

- **harnais demandeurs** : open-interpreter
- **popularite cumulee** : 68,589 etoiles
- **adaptateurs proposes** : `plugins/lifecycle-hooks`
- **exemplaire de reference** : `open-interpreter` — *Hooks deterministes du cycle de vie agent* (port `HookEngine (fire/register)`, adaptateur `plugins/lifecycle-hooks`, cout M)
  - comment il le fait : 10 phases : session_start, user_prompt_submit, pre_tool_use, post_tool_use, permission_request, stop, session_end, compact, interrupt (+ AfterAgent typé, types.rs:92) ; decouverte a cote des couches de config (~/.openinterpreter/hooks.json, .openinterpreter/hooks.json, inline config.toml, plugins) sans remplacement par precedence ; commandes non gerees soumises a revue de confiance (trust enregist
  - preuves : `docs/hooks.md:1-80`, `codex-rs/hooks/src/events/mod.rs:1-10`, `codex-rs/hooks/src/types.rs:92-97`, `codex-rs/hooks/src/registry.rs`

#### `agent-mode` — 1 harnais, cout S

- **harnais demandeurs** : aider
- **popularite cumulee** : 49,587 etoiles
- **adaptateurs proposes** : `plugins/readonly-modes`
- **exemplaire de reference** : `aider` — *Modes lecture seule ask / context* (port `AgentMode (ask/context/code)`, adaptateur `plugins/readonly-modes`, cout S)
  - comment il le fait : AskCoder repond sans jamais ecrire ; ContextCoder identifie les fichiers a editer, les ajoute au chat et demande une reformulation (reflected_message) au lieu d'editer ; commandes /ask /context /architect basculent de Coder.
  - preuves : `aider/coders/ask_coder.py:5-9`, `aider/coders/context_coder.py:5-53`, `aider/commands.py:1182-1198`

#### `agent-control` — 1 harnais, cout S

- **harnais demandeurs** : cursor
- **adaptateurs proposes** : `plugins/agent-steer`
- **exemplaire de reference** : `cursor` — *File de messages + pilotage a chaud + side chats* (port `AgentControl`, adaptateur `plugins/agent-steer`, cout S)
  - comment il le fait : [documente] Entree = mise en file (traitee apres la tache) ; Cmd+Entree = envoi immediat ; pilotage a chaud delivre au prochain appel d'outil sans couper le travail en cours ; side chats /side avec contexte parent masque ; /goal pour objectif persistant ; recherche de conversations indexee localement.
  - preuves : `https://cursor.com/docs/agent/overview`, `https://cursor.com/docs/agent/agents-window.md`

#### `agent-modes` — 1 harnais, cout S

- **harnais demandeurs** : windsurf
- **adaptateurs proposes** : `plugins/agent-modes`
- **exemplaire de reference** : `windsurf` — *Modes Code / Plan / Ask (+ Write/Chat)* (port `AgentModePolicy (permissions par mode)`, adaptateur `plugins/agent-modes`, cout S)
  - comment il le fait : [documenté] Sélecteur sous la zone de saisie (raccourci Cmd/Ctrl+.) ; Code = tous outils, Plan = exploration + questions à choix multiples + plan écrit dans un fichier Markdown externe persistant (~/.windsurf/plans ou ~/.devin/plans, réutilisable par @-mention), Ask = lecture seule (outils de recherche uniquement) ; mots-clés megaplan/ultraplan/masterplan pour planification renforcée.
  - preuves : `https://docs.devin.ai/desktop/cascade/modes.md`

#### `channel-connector` — 1 harnais, cout M

- **harnais demandeurs** : devin
- **adaptateurs proposes** : `plugins/channel-slack + plugins/tracker-linear + plugins/vcs-github`
- **exemplaire de reference** : `devin` — *Integrations Slack/Teams/GitHub/Linear/Jira (+ Data/PagerDuty)* (port `Channel / Tracker / Vcs`, adaptateur `plugins/channel-slack + plugins/tracker-linear + plugins/vcs-github`, cout M)
  - comment il le fait : Documente : Slack (sessions depuis tout canal, threads synchronises, code channel dedie), Teams (@Devin AI + mots-cles inline), GitHub (PR, commentaires de review, acces repo), GitLab (MR, pipelines), Bitbucket, Linear (tickets -> PR via labels/@mentions), Jira (agent Rovo ou service user OAuth), Databricks/MongoDB/PagerDuty/Pylon, PR templates decouverts automatiquement. Infere : adaptateurs OAut
  - preuves : `https://docs.devin.ai/integrations/slack.md`, `https://docs.devin.ai/integrations/microsoft-teams.md`, `https://docs.devin.ai/integrations/gh.md`, `https://docs.devin.ai/integrations/linear.md`

### Outils & surface d'action

#### `tool-surface` — 8 harnais, cout S

- **harnais demandeurs** : aider, claude-code, cursor, devin, openclaw, openhands, rea, windsurf
- **popularite cumulee** : 291,918 etoiles
- **adaptateurs proposes** : `omega-context::ToolSurfaceIndex`, `plugins/agent-tools`, `plugins/mcp-client (stdio/http/sse/ws + trust + elicitation)`, `plugins/mcp-rea`, `plugins/skill-rea`, `plugins/slash-commands`, `plugins/tool-devin-workspace`, `plugins/tool-registry`
- **exemplaire de reference** : `aider` — *Systeme de commandes slash terminal* (port `ToolSurface (commands)`, adaptateur `plugins/slash-commands`, cout M)
  - comment il le fait : Commands (commands.py, ~1700 lignes) expose ~35 cmd_* : /add /drop /read-only /map /tokens /commit /undo /diff /test /lint /run /git /voice /model /tokens ; auto-completion prompt_toolkit et detection de mentions de fichiers/URLs dans l'entree.
  - preuves : `aider/commands.py:87-138`, `aider/commands.py:799-967`, `aider/commands.py:993-1064`, `aider/commands.py:1418-1432`

#### `tool` — 6 harnais, cout S

- **harnais demandeurs** : cline, gptme, hermes, openhands, plandex, rea
- **popularite cumulee** : 181,709 etoiles
- **adaptateurs proposes** : `plugins/structured-edits`, `plugins/tool-rea-android`, `plugins/tool-rea-captures`, `plugins/tool-rea-crash`, `plugins/tool-rea-dotnet`, `plugins/tool-rea-evm`, `plugins/tool-rea-firmware`, `plugins/tool-rea-functions`, `plugins/tool-rea-js`, `plugins/tool-rea-native`, `plugins/tool-rea-sqlite`, `plugins/tool-rea-wasm`, `plugins/tool-rpc`, `plugins/tool-sdk`, `plugins/tool-shell`, `plugins/tools-exec`
- **exemplaire de reference** : `rea` — *Routeur multi-providers natifs (Hopper/Ghidra/IDA/Rizin/objdump/Cutter/GDB)* (port `ToolExecutor`, adaptateur `plugins/tool-rea-native`, cout L)
  - comment il le fait : AnalysisProviderRegistry + SessionProviderRouter : une liaison immuable par cible, sélection déterministe sans fallback ; bridge Python Hopper (bridge/hopper_bridge.py), bundle Java Ghidra (bridge/ghidra/), adaptation du MCP upstream ida-pro-mcp, profils Rizin/objdump/GDB, Cutter. Sessions persistantes limitées à la vie du process MCP.
  - preuves : `src/ghidra/GhidraProvider.ts:1-40`, `src/hopper/HopperProvider.ts:1-40`, `src/ida/IdaProvider.ts:39-60`, `src/objdump/ObjdumpCommand.ts:1-40`

#### `mcp-bridge` — 2 harnais, cout M

- **harnais demandeurs** : cline, zed
- **popularite cumulee** : 162,257 etoiles
- **adaptateurs proposes** : `plugins/mcp-bridge`
- **exemplaire de reference** : `cline` — *Pont MCP (manager + cache + politiques)* (port `McpBridge (register/call)`, adaptateur `plugins/mcp-bridge`, cout M)
  - comment il le fait : InMemoryMcpManager (manager.ts:43) avec registre de serveurs, cache d'outils TTL 5s et verrous par operation ; client.ts (transport), config-loader.ts, policies.ts (garde-fous), remote-proxy.ts, oauth.ts, name-transform.ts ; pont vers outils natifs dans extensions/mcp/tools.ts avec resultPolicy cache-oversized ; installation via services/mcp-install.ts et CLI `mcp`.
  - preuves : `sdk/packages/core/src/extensions/mcp/manager.ts:43-80`, `sdk/packages/core/src/extensions/mcp/tools.ts:1-60`, `sdk/packages/shared/src/mcp.ts:1-51`

#### `mcp-client` — 2 harnais, cout M

- **harnais demandeurs** : cursor, gptme
- **popularite cumulee** : 4,453 etoiles
- **adaptateurs proposes** : `plugins/mcp-client`
- **exemplaire de reference** : `gptme` — *Client MCP (stdio+HTTP, tools/resources/prompts)* (port `McpClient (connect/call/list)`, adaptateur `plugins/mcp-client`, cout L)
  - comment il le fait : MCPClient gere les connexions stdio + HTTP avec callbacks roots et elicitation (gptme/mcp/client.py:60-310) ; expose call_tool/list_resources/read_resource/list_prompts/get_prompt et gestion dynamique des roots (gptme/mcp/client.py:351-593) ; create_mcp_tools() convertit les outils MCP distants en ToolSpec locaux avec retry (gptme/tools/mcp_adapter.py:290-523) ; MCPRegistry cherche dans les regist
  - preuves : `gptme/mcp/client.py:60-310`, `gptme/mcp/client.py:351-593`, `gptme/tools/mcp_adapter.py:290-523`, `gptme/mcp/registry.py:50-214`

#### `mention-resolver` — 1 harnais, cout M

- **harnais demandeurs** : zed
- **popularite cumulee** : 91,830 etoiles
- **adaptateurs proposes** : `plugins/mention-resolve`
- **exemplaire de reference** : `zed` — *@-mentions de fichiers/selections/diffs* (port `MentionResolver (parse/resolve)`, adaptateur `plugins/mention-resolve`, cout M)
  - comment il le fait : MentionUri (mention.rs:20) avec parse (mention.rs:80), parse_hyperlink (mention.rs:292), abs_path/name/tooltip (mention.rs:315-398) ; MentionSet (mention_set.rs:60) attache les mentions aux creases de l'editeur (insert_mention mention_set.rs:113, confirm_mention_for_file/selection/git_diff) et les resout en contenu de contexte.
  - preuves : `crates/acp_thread/src/mention.rs:20-120`, `crates/agent_ui/src/mention_set.rs:45-130`, `https://docs.zed.dev/ai/agent-panel#mentions`

#### `harness-emulation` — 1 harnais, cout S

- **harnais demandeurs** : open-interpreter
- **popularite cumulee** : 68,589 etoiles
- **adaptateurs proposes** : `plugins/harness-claude-code`, `plugins/harness-kimi-code`, `plugins/harness-minimal`, `plugins/harness-swe-agent`, `plugins/harness-zcode`
- **exemplaire de reference** : `open-interpreter` — *Surface minimale + harnais secondaires (qwen, opencode, pi...)* (port `HarnessEmulation (build_request/handle_output)`, adaptateur `plugins/harness-minimal`, cout S)
  - comment il le fait : minimal.rs (745 lignes, build_request analogue aux autres bras de request.rs) : system prompt compact d'agent logiciel generique + mapping direct des outils fonctionnels en tool list Chat Completions, sans surface specifique au provider ; autres bras du meme point d'integration : deepseek_tui (876 lignes, metadonnees de tour + instructions projet generees), qwen_code (echange synthetique date/OS/c
  - preuves : `codex-rs/core/src/harness/minimal.rs:1-60`, `codex-rs/core/src/harness/qwen_code.rs:1-60`, `codex-rs/core/src/harness/deepseek_tui.rs:1-60`, `codex-rs/core/src/harness/opencode.rs:1-60`

#### `tool-source` — 1 harnais, cout M

- **harnais demandeurs** : open-interpreter
- **popularite cumulee** : 68,589 etoiles
- **adaptateurs proposes** : `plugins/tool-mcp`
- **exemplaire de reference** : `open-interpreter` — *Serveurs MCP stdio/HTTP geres en CLI* (port `ToolSource (list/call)`, adaptateur `plugins/tool-mcp`, cout M)
  - comment il le fait : Serveurs MCP stdio (command/args/env) ou HTTP (url + bearer_token_env_var), geres par interpreter mcp {add,list,get,remove,login,logout} ; connection_manager + catalog + tool_catalog_cache avec pagination dans codex-mcp ; trusted_access et elicitation pour les acces sensibles ; les outils MCP rejoignent le catalogue d'outils du runtime comme les outils natifs.
  - preuves : `docs/mcp.md:1-60`, `codex-rs/codex-mcp/src/catalog.rs`, `codex-rs/codex-mcp/src/connection_manager.rs`, `codex-rs/codex-mcp/src/trusted_access.rs`

#### `mcp-server` — 1 harnais, cout M

- **harnais demandeurs** : gptme
- **popularite cumulee** : 4,453 etoiles
- **adaptateurs proposes** : `plugins/mcp-server`
- **exemplaire de reference** : `gptme` — *Serveur MCP (expose les outils gptme)* (port `McpServer (serve/list/call)`, adaptateur `plugins/mcp-server`, cout M)
  - comment il le fait : GptmeMCPServer expose les outils gptme via le protocole MCP (_toolspec_to_mcp_tool convertit ToolSpec -> types.Tool) (gptme/mcp/server.py:65-153) avec session shell partagee (gptme/mcp/server.py:136-149) ; `gptme mcp serve` demarre le serveur (gptme/cli/cmd_mcp_serve.py:1-72) ; dans l'autre sens, a2a_api.py et skills_api.py exposent agents/skills en HTTP (gptme/server/a2a_api.py, gptme/server/skil
  - preuves : `gptme/mcp/server.py:65-153`, `gptme/cli/cmd_mcp_serve.py:1-72`, `gptme/server/skills_api.py:1-161`, `gptme/server/a2a_api.py:1-120`

### Reverse-engineering

#### `harness-catalog` — 1 harnais, cout S

- **harnais demandeurs** : rea
- **adaptateurs proposes** : `plugins/tool-rea-cli`
- **exemplaire de reference** : `rea` — *CLI one-shot miroir du MCP* (port `Registre`, adaptateur `plugins/tool-rea-cli`, cout S)
  - comment il le fait : ~50 modules src/cli/*Commands.ts enregistrés dans src/cli.ts sur le même moteur application que MCP ; chaque invocation = un process séparé (persister via fichiers) ; sorties --json + vues (summary/page) pour gros evidences ; client-configuration et vérification documentés.
  - preuves : `src/cli.ts:1-60`, `docs/cli.md:1-60`, `docs/client-configuration.md:1-40`

### Routage & modeles

#### `router-policy` — 4 harnais, cout S

- **harnais demandeurs** : cursor, open-interpreter, plandex, windsurf
- **popularite cumulee** : 84,339 etoiles
- **adaptateurs proposes** : `omega-router::ArenaSelector`, `plugins/harness-autoselect`, `plugins/harness-router`, `plugins/model-router`, `plugins/router-large-ctx`
- **exemplaire de reference** : `open-interpreter` — *Routage strict wire_api x harness* (port `Selector`, adaptateur `plugins/harness-router`, cout M)
  - comment il le fait : resolve_stream_transport_route (routing.rs:56) croise WireApi {Responses, Chat, Messages} x Harness -> StreamTransportRoute (routing.rs:38 : ResponsesApi, ChatCompletionsCompat, ChatHarness(..), MessagesHarness(..), ClaudeCodeResponses(..), ClaudeCodeChat(..)) ; le fil Messages exige un transport natif harnais et rejette Native avec un message d'erreur explicite ; le point d'integration unique cli
  - preuves : `codex-rs/core/src/harness/routing.rs:38-160`, `codex-rs/core/src/harness/request.rs:43-120,214-260`, `codex-rs/model-provider-info/src/lib.rs:111-130`, `docs/harness.md:62-90`

#### `llm-provider` — 2 harnais, cout M

- **harnais demandeurs** : opencode, openhands
- **popularite cumulee** : 305,442 etoiles
- **adaptateurs proposes** : `omega-providers (catalogue par transforms + scoring communautaire)`, `plugins/llm-registry`
- **exemplaire de reference** : `openhands` — *Registre LLM multi-providers + fallback + device-login* (port `Provider (complete/stream)`, adaptateur `plugins/llm-registry`, cout M)
  - comment il le fait : LLMRegistry (evenements de registre) + modele LLM pydantic avec mixins retry/outils non-natifs + FallbackStrategy ; router list_models/list_verified/device-login OAuth + passerelle OpenAI-compatible (chat/completions, responses) ; suivi de solde cote UI.
  - preuves : `sdk/openhands-sdk/openhands/sdk/llm/llm_registry.py:23`, `sdk/openhands-sdk/openhands/sdk/llm/llm.py:252`, `sdk/openhands-sdk/openhands/sdk/llm/fallback_strategy.py:45`, `sdk/openhands-agent-server/openhands/agent_server/llm_router.py:116-266`

#### `task-classifier` — 1 harnais, cout M

- **harnais demandeurs** : plandex
- **popularite cumulee** : 15,750 etoiles
- **adaptateurs proposes** : `omega-router::HeuristicTaskClassifier`
- **exemplaire de reference** : `plandex` — *Rôles de modèles spécialisés par tâche* (port `TaskClassifier (role)`, adaptateur `omega-router::HeuristicTaskClassifier`, cout M)
  - comment il le fait : Neuf rôles énumérés (ai_models_roles.go:3-12 : planner, coder, architect, summarizer, builder, whole-file-builder, names, commit-messages, auto-continue) ; chaque rôle a sa config modèle propre (ai_models_roles.go, ai_models_config.go) et le proxy LiteLLM route vers ~30 fournisseurs (OpenAI, Anthropic, Google, DeepSeek, OpenRouter…).
  - preuves : `app/shared/ai_models_roles.go:3-12`, `app/shared/ai_models_providers.go`, `app/server/litellm_proxy.py`

## 3. Feuille de route par phase

### Phase 1 — socle (26 capacites)

- `context-engine` — 13 harnais · 744,090 etoiles
- `subagent-runner` — 13 harnais · 741,580 etoiles
- `config-source` — 10 harnais · 522,782 etoiles
- `tool-surface` — 8 harnais · 291,918 etoiles
- `memory-store` — 7 harnais · 155,705 etoiles
- `tool` — 6 harnais · 181,709 etoiles
- `skill-store` — 6 harnais · 123,938 etoiles
- `registry` — 5 harnais · 540,978 etoiles
- `router-policy` — 4 harnais · 84,339 etoiles
- `hook-bus` — 4 harnais · 4,453 etoiles
- `telemetry-sink` — 3 harnais · 65,337 etoiles
- `exec-policy` — 3 harnais · 49,587 etoiles
- `agent-store` — 2 harnais · 365,615 etoiles
- `agent-runtime` — 2 harnais · 125,776 etoiles
- `harness-emulation` — 1 harnais · 68,589 etoiles
- `(observabilite)` — 1 harnais · 55,349 etoiles
- `agent-mode` — 1 harnais · 49,587 etoiles
- `edit-apply` — 1 harnais · 49,587 etoiles
- `lint-check` — 1 harnais · 49,587 etoiles
- `vcs-commit` — 1 harnais · 49,587 etoiles
- `cost-guard` — 1 harnais · 4,453 etoiles
- `agent-control` — 1 harnais
- `agent-modes` — 1 harnais
- `harness-catalog` — 1 harnais
- `playbook-store` — 1 harnais
- `workflow-runner` — 1 harnais

### Phase 2 — structure (39 capacites)

- `scheduler-job` — 8 harnais · 366,798 etoiles
- `sandbox-backend` — 8 harnais · 262,534 etoiles
- `event-store` — 4 harnais · 434,204 etoiles
- `agent-server` — 3 harnais · 238,258 etoiles
- `llm-provider` — 2 harnais · 305,442 etoiles
- `mcp-bridge` — 2 harnais · 162,257 etoiles
- `extension-host` — 2 harnais · 146,428 etoiles
- `agent-runner` — 2 harnais · 91,830 etoiles
- `channel` — 2 harnais · 70,427 etoiles
- `hook` — 2 harnais · 70,427 etoiles
- `permission-policy` — 2 harnais · 55,349 etoiles
- `agent-loop` — 2 harnais · 20,203 etoiles
- `mcp-client` — 2 harnais · 4,453 etoiles
- `checkpoint-store` — 2 harnais
- `(execution)` — 1 harnais · 214,363 etoiles
- `mention-resolver` — 1 harnais · 91,830 etoiles
- `ui-panel` — 1 harnais · 91,830 etoiles
- `agent-host` — 1 harnais · 91,079 etoiles
- `headless-run` — 1 harnais · 70,427 etoiles
- `agent-protocol` — 1 harnais · 68,589 etoiles
- `lifecycle-hook` — 1 harnais · 68,589 etoiles
- `tool-source` — 1 harnais · 68,589 etoiles
- `ui-surface` — 1 harnais · 68,589 etoiles
- `(automatisation)` — 1 harnais · 55,349 etoiles
- `plan-store` — 1 harnais · 15,750 etoiles
- `task-classifier` — 1 harnais · 15,750 etoiles
- `mcp-server` — 1 harnais · 4,453 etoiles
- `app-server` — 1 harnais
- `channel-connector` — 1 harnais
- `code-index` — 1 harnais
- `code-search` — 1 harnais
- `control-ui` — 1 harnais
- `inline-complete` — 1 harnais
- `memory-search` — 1 harnais
- `plugin-manifest` — 1 harnais
- `provenance` — 1 harnais
- `task-handoff` — 1 harnais
- `task-planner` — 1 harnais
- `worktree-manager` — 1 harnais

### Phase 3 — lourd (7 capacites)

- `worktree-isolate` — 1 harnais · 91,830 etoiles
- `conversation-store` — 1 harnais · 91,079 etoiles
- `ui-shell` — 1 harnais · 91,079 etoiles
- `acp-agent` — 1 harnais · 4,453 etoiles
- `cloud-runner` — 1 harnais
- `code-review` — 1 harnais
- `workflow-engine` — 1 harnais

## 4. Non-objectifs (ecartes, avec raison)

| Harnais | Fonctionnalite | Pourquoi ecartee |
|---|---|---|
| `claude-code` | Mods JS in-process (écarté : runtime JS) | ECARTE : documenté (https://code.claude.com/docs/en/plugins/mods/overview, liens depuis https://code.claude.com/docs/en/hooks observés par webfetch) : les mods sont des hooks JS in-process pouvant des |
| `cline` | Cline Hub cloud (SaaS operateur) | Hub SaaS proprietaire (sdk/packages/core/src/hub, cloud/, remote/) : authentification WorkOS device-auth, sessions cloud distantes (CloudSessionController), facturation/quotas, connecteurs geres par d |
| `cursor` | Telemetrie cloud proprietaire | [documente] Télémétrie d'usage vers les serveurs Anysphere avec Privacy Mode / retention configurable (entreprise) ; [infere] pipeline closed-source, facturation a l'usage indexee dessus. Non repris : |
| `goose` | App desktop Electron | Ecarte : ui/desktop est une app Electron (forge.config.ts, electron-builder) separee du workspace Rust, avec son cycle de release propre. Omega vise cockpit React servi par omega-server sur une seule  |
| `open-interpreter` | Computer-use GUI : absent (constate) | Inventaire exhaustif de codex-rs/ (~127 crates) : aucun crate d'automatisation GUI (pas de controle souris/clavier, capture assiste, ni protocole d'accessibilite) et aucune section computer-use dans d |
| `opencode` | Code Mode + LSP/formatter (ecarte : runtime JS) | LSP/formatter declaratifs en config (command + extensions) : adaptateurs lancant des serveurs externes ; Code Mode : le modele ecrit du code appelant les outils, schemas JSON rendus en types TypeScrip |
| `plandex` | Facturation cloud et RBAC SaaS | Facturation cloud (cmd/billing.go, crédits, rachats auto), invitations et RBAC org (rbac.go, invite_helpers.go) n'ont de sens que dans l'offre SaaS Plandex ; non transférables à un harnais self-hosted |
| `windsurf` | Catalogue de modèles à crédits propriétaires | [documenté] Catalogue de dizaines de modèles (SWE, Claude, GPT, Gemini, Grok, ...) chacun avec coûts entrée/sortie/cache et multiplicateur de crédits (0.25x à 320x) ; routage Adaptive (sélection auto  |

## 5. Provenance

- dissections : `omega/crates/omega-harness/data/apps/*.json` (schema `DISSECTION-SCHEMA.md`)
- popularite : `omega/crates/omega-harness/data/harnesses_census.json` (API GitHub)
- preuves machine : `docs/omega/research/rea/evidence/` (REA, avec limites declarees)
- ce document ne declare « adopte » que ce qui existe reellement dans le code

