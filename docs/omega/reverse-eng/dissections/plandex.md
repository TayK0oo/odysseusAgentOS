# Dissection — Plandex

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | PlandexAI Inc. |
| Licence | MIT (`LICENSE`) — vérifiée dans le repo |
| Langage | Go (332 fichiers `*.go` sous `app/`, modules `cli` / `server` / `shared`) |
| Repo | https://github.com/plandex-ai/plandex (v2.2.1, `app/server/version.txt`) |
| Architecture | Agent de codage en deux phases **tell → build** : le serveur Go tient la boucle de planification conversationnelle puis la génération de diffs fichier par fichier ; la CLI Cobra orchestre ; les diffs sont calculés **hors arbre de travail** (répertoire temporaire + `git diff --no-index`) ; project map **tree-sitter** (~35 grammaires) ; ~30 fournisseurs via proxy LiteLLM ; ~2M tokens effectifs par chaînes de fallbacks. |

## 2. Entrées

- **Binaire CLI** : `app/cli/main.go`, ~50 sous-commandes Cobra dans `app/cli/cmd/` (`new`, `tell`, `build`, `apply`, `branches`, `rewind`, `context_show`, `set_model`, `ps`, `convo`…).
- **Serveur HTTP** : `app/server/main.go`, routes dans `app/server/routes/routes.go`, streaming SSE vers la CLI (`app/cli/stream/`, `app/cli/stream_tui/`).
- **Proxy LiteLLM embarqué** : `app/server/litellm_proxy.py` (même conteneur, `app/docker-compose.yml`).
- **Config partagée** : module `plandex-shared` (`app/shared/plan_config.go`, `plan_model_settings.go`, `ai_models_*.go`).
- **Plans** : répertoire `plans/` à la racine du repo de travail ; état courant `CurrentPlanState` (`app/shared/data_models.go:295`).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Boucle tell→build en deux phases | agents | Planification conversationnelle puis génération de diffs | `app/server/model/plan/tell_stream_main.go`, `app/server/model/plan/build_exec.go` | agent-loop → AgentLoop | planned | L |
| Arbre de versions du plan | memoire | Branches listables, applies horodatés, rewind | `app/cli/cmd/branches.go:20`, `app/shared/data_models.go:295`, `app/cli/lib/rewind.go:107` | plan-store → PlanStore | planned | M |
| Chargement à la demande par étape | contexte | Chaque étape n'hydrate que le fichier courant | `app/server/model/plan/build_load.go:17`, `app/server/model/plan/tell_load.go:20` | context-engine → ContextEngine | planned | M |
| Diff cumulatif en sandbox | execution | Diffs calculés en répertoire temporaire via git | `app/server/diff/diff.go:18`, `app/server/diff/diff.go:67` | sandbox-backend → SandboxBackend | planned | M |
| Project map tree-sitter | contexte | Carte définitions+signatures, ~35 langages, cache md5 | `app/server/syntax/file_map/map.go:34`, `app/server/syntax/parsers.go:42` | context-engine → ContextEngine | planned | L |
| Sept types de contexte + budgets | contexte | file/url/note/tree/piped/image/map, plafonds durs | `app/shared/data_models.go:85`, `app/cli/lib/context_load.go:26` | context-engine → ContextEngine | planned | M |
| Auto-chargement du contexte | contexte | Chargement parallèle avec garde-fous | `app/cli/lib/context_auto_load.go`, `app/cli/lib/context_update.go:305` | context-engine → ContextEngine | planned | S |
| Rôles de modèles par tâche | routage | 9 rôles (planner, coder, architect…), ~30 fournisseurs | `app/shared/ai_models_roles.go:3`, `app/shared/ai_models_providers.go` | task-classifier → TaskClassifier | planned | M |
| Fallbacks grands contextes | routage | Chaînes de fallbacks jusqu'à ~2M tokens | `app/shared/ai_models_large_context.go` | router-policy → Selector | planned | M |
| Comptabilité du caching | observabilite | Tokens cachés mesurés par modèle/rôle/plan | `app/server/model/model_request.go:201`, `app/server/model/model_request.go:218` | telemetry-sink → TelemetrySink | planned | S |
| Résumé de conversation | memoire | Summarizer au-delà du budget de fenêtre | `app/server/model/summarize.go:31`, `app/server/model/plan/tell_load.go:246` | context-engine → ContextEngine | planned | M |
| Apply + rewind | execution | Apply transactionnel, retour arrière horodaté | `app/cli/lib/apply.go:37`, `app/cli/lib/rewind.go:107` | sandbox-backend → SandboxBackend | planned | M |
| Edits structurés + validate-and-fix | execution | Ancrage syntaxique, 3 tentatives de correction | `app/server/syntax/structured_edits_tree_sitter.go:29`, `app/server/model/plan/build_validate_and_fix.go` | tool → ToolExecutor | planned | L |
| CLI + serveur + config partagée | configuration | Cobra + HTTP + types partagés | `app/cli/cmd/root.go`, `app/server/routes/routes.go` | config-source → ConfigSource | planned | M |
| Facturation cloud / RBAC SaaS | configuration | Crédits, orgs — écarté | `app/cli/cmd/billing.go`, `app/shared/rbac.go` | — | rejected | L |

## 4. Traduction en plugins

- `agent-loop → AgentLoop → plugins/agent-tell-build` (plugin) : séparer planification et exécution en deux boucles de streaming, comme `tell_stream_main` / `build_exec`.
- `plan-store → PlanStore → plugins/plan-branches` (plugin) : branches + applies horodatés + rewind, calqué sur `CurrentPlanState` / `AnalyzeRewind`.
- `context-engine → ContextEngine → plugins/context-per-step` (plugin) : hydration par étape (`loadPendingBuilds` / `loadBuildFile`), verrou read après le tour 0.
- `sandbox-backend → SandboxBackend → plugins/diff-sandbox` (plugin) : calcul de diffs en répertoire temporaire (`GetDiffs` / `GetDiffReplacements`).
- `context-engine → ContextEngine → plugins/project-map` (plugin) : project map tree-sitter avec cache par hash, servie au rôle architecte.
- `context-engine → ContextEngine → omega-context::DefaultContextEngine` (plugin) : types de contexte + budgets + auto-chargement.
- `task-classifier → TaskClassifier → omega-router::HeuristicTaskClassifier` (plugin) : rôles de modèles par tâche.
- `router-policy → Selector → plugins/router-large-ctx` (plugin) : fallbacks grands contextes avec padding configurable.
- `telemetry-sink → TelemetrySink → omega-otel::StdoutSink` (plugin) : comptabilité cached/input/output tokens par appel.
- `context-engine → ContextEngine → plugins/convo-summarize` (plugin) : résumé par rôle summarizer au-delà du budget.
- `sandbox-backend → SandboxBackend → omega-sandbox::LocalSandboxBackend` (plugin) : apply transactionnel + rewind.
- `tool → ToolExecutor → plugins/structured-edits` (plugin) : edits ancrés tree-sitter + boucle validate-and-fix (3 essais).
- `config-source → ConfigSource → omega-config` (plugin) : couches CLI / serveur / types partagés.

## 5. Écarté

- **Facturation cloud et RBAC SaaS** (`pl-cloud-billing`, rejected) : crédits, rachats auto, invitations d'organisation — propre au modèle économique hébergé de Plandex, sans équivalent dans un harnais self-hosted modulaire.

## 6. Sources

- Repo : https://github.com/plandex-ai/plandex (clone `--depth 1`, v2.2.1)
- Chemins : `app/cli/` (CLI, ~50 commandes), `app/server/model/plan/` (boucles tell/build), `app/server/diff/diff.go`, `app/server/syntax/` (tree-sitter), `app/server/db/context_helpers_map.go` (cache des cartes), `app/shared/` (types, rôles, budgets)
