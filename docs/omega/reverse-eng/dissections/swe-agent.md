# Dissection — SWE-agent (Princeton NLP, MIT, Python)

> Source : clone `--depth 1` de https://github.com/princeton-nlp/SWE-agent dans `/home/tayk0oo/omega-re/swe-agent`.
> Donnees : `omega/crates/omega-harness/data/apps/swe-agent.json` (13 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Princeton NLP |
| Langage | Python |
| Repo | https://github.com/princeton-nlp/SWE-agent |
| Architecture | Harnais de resolution d'issues GitHub : pydantic BaseModel tout au centre (config YAML partiels), SWEEnv bash persistant + outils installes comme bundles de scripts (windowed editing, diff_state), chaine de history processors, tool parsing double (function-calling ou backticks), reviewer + retry loop, harness de run batch/replay/comparison (SWE-bench). |

## 2. Inventaire fonctionnel

### `swea-template-config` — Templates Jinja versionnes par config et valides au chargement

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Les templates de prompt (system/instance/next_step) sont des champs de config versionnes, valides pour la syntaxe Jinja des le chargement, changenant par pack de config.
- **how** : TemplateConfig (sweagent/agent/agents.py:60) model_post_init valide la syntaxe Jinja des templates (agents.py:128) et genere des warnings (agents.py:136) ; les packs de config sont des YAML partiels (config/default.yaml). Omega : omega-config couvre la config_cv cs le remplacement {{variable}} par Jinja du model_prompt: le pattern de validation systématique du template est une preuve.
- **adaptateur Omega** : omega-config (validation du template de config)
- **preuves** : `sweagent/agent/agents.py:60-136` · `config/default.yaml`

### `swea-tool-handler` — ToolHandler : install/reset des outils dans la sandbox, avec registry de bundles

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : M · **statut** : planned
- **what** : Les outils sont des bundles installes dans l'env (script de bash), avec une commande d'etat qui expose l'etat du systeme de fichiers/env a l'agent.
- **how** : ToolHandler (sweagent/tools/tools.py:227) a from_config/install/reset (tools.py:252-256) ; Bundle (sweagent/tools/bundle.py:17) empaquette commands + state_command (bundle.py:44) ; tools/registry est un bundle standard installable (tools/registry/install.sh). Omega : le plugin-api couvre les plugins ; le pattern bundle commande bash+state_command est un port chaud d'outre.
- **adaptateur Omega** : omega-app (tool bundles installes dans la sandbox)
- **preuves** : `sweagent/tools/tools.py:227-256` · `sweagent/tools/bundle.py:17-52`

### `swea-toolfilter` — ToolFilterConfig : le model chois/dispose les outils avec regex, per-sandbox

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : Un sous-ensemble des outils peut etre expose au model selon un filtre par regex ; utile pour des politiques prevenus.
- **how** : ToolFilterConfig (sweagent/tools/tools.py:29) sur ToolConfig (tools.py:75) : filters regex sur les commandes visibles. Omega : ExecPolicy + ToolSurface (allowlist declarative) ; l'ecart : filtre dynamique per-run.
- **adaptateur Omega** : omega-ports::ExecPolicy (filter regex per-run)
- **preuves** : `sweagent/tools/tools.py:29-75`

### `swea-history-processor` — History processors chiains : LastN, tagging, cache control

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : M · **statut** : planned
- **what** : Une chaine de history processors reecrit l'historique avant envoie : omis au dela de N observations, tags d'outils, cache control anthropique, clipping des fenetres.
- **how** : AbstractHistoryProcessor (sweagent/agent/history_processors.py:13) : Protocol __call__(history)->History ; LastNObservations (history_processors.py:85) garde les N derniers avec _get_omit_indices (147) ; TagToolCallObservations (history_processors.py:179) _add_tags (194) ; _set_cache_control/_clear_cache_control (history_processors.py:53-46). Omega : omega-context couvre la compaction LLM ; le pattern de chaine de transformations testables (Protocol) est exactement un port.
- **adaptateur Omega** : omega-context (chaine de history processors)
- **preuves** : `sweagent/agent/history_processors.py:13-46` · `sweagent/agent/history_processors.py:85-157`

### `swea-retry-loop` — Reviewer + retry loop : self-review du submit, rejouer ou accepter

- **domaine** : agents · **capacite** : agent-loop · **port** : — · **cout** : L · **statut** : planned
- **what** : Un reviewer separat critique la soumission finale ; la boucle de retry decide si une nouvelle tentative strategique est jugee.
- **how** : AbstractReviewer (sweagent/agent/reviewer.py:81) review(instance, submission)->ReviewerResult (reviewer.py:87) ; AbstractRetryLoop (reviewer.py:91) retry/on_submit ; reviewers de type chooser/preselector (reviewer.py:65-71). Omega : pas de self-review du noyau ; un plugin review pose au dessus de l'AgentLoop, acceptable par ExecPolicy d'outils limiting.
- **adaptateur Omega** : plugin/self-review (reviewer + retry loop)
- **preuves** : `sweagent/agent/reviewer.py:81-100`

### `swea-action-sampler` — Action sampler : propositions paralleles de plusieurs modeles (ask colleagues)

- **domaine** : routage · **capacite** : router-policy · **port** : Selector · **cout** : L · **statut** : planned
- **what** : Plutôt qu'une seule action, le sampler propose des actions concurrentes par plusieurs fournisseurs et demande la make choice a un chooser/collegue.
- **how** : AbstractActionSampler (sweagent/agent/action_sampler.py:23) ; AskColleagues (action_sampler.py:49) get_colleague_discussion (54) fait un tour d'avis entre outputs ; get_action remet la meilleur (action_sampler.py:76). Omega : omega-router::Selector pare un mode ; le sampler des collegues est differerent : decision a travers l'agent lui-meme (1M+ selectors).
- **adaptateur Omega** : plugin/action-sampler (avis concurrent des collegues)
- **preuves** : `sweagent/agent/action_sampler.py:23-76`

### `swea-tool-parsing` — Parsing des actions : backup function-call ou backticks ouvert

- **domaine** : agents · **capacite** : tool-surface · **port** : ToolSurface · **cout** : S · **statut** : planned
- **what** : Deux modes d'appel d'outils : function-calling natif (tools declares en JSON schema) ou parsing des backticks de bloc bash ; le parsing est un re-entrant evitent de forcer le model.
- **how** : sweagent/tools/parsing.py + ToolConfig.use_function_calling (sweagent/tools/tools.py:155) ; Command.get_function_calling_tool (sweagent/tools/commands.py:133) construit le schema du tool JSON. Omega : le noyau parse un movement ; un mode parsing de output magistral peut ajouter a omega-context (tool-parse des backticks).
- **adaptateur Omega** : omega-context (tool parsing backticks)
- **preuves** : `sweagent/tools/parsing.py` · `sweagent/tools/commands.py:133` · `sweagent/tools/tools.py:155-168`

### `swea-env` — SWEEnv : env bash durable avec reset/clone repo et hooks

- **domaine** : contexte · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : M · **statut** : planned
- **what** : L'environnement est un process bash persistant avec copy/reset/hard_reset du repo, et des hooks (env) sur chaque etape ; l'outillage (tools) est installe des le deploy.
- **how** : class SWEEnv (sweagent/environment/swe_env.py:51) ; _copy_repo (swe_env.py:116), hard_reset/reset (swe_env.py:128-135), reset_repository (swe_env.py:149) ; env hooks en add_hook (swe_env.py:100). Omega : omega-sandbox::LocalSandboxBackend + NomeSandbox (namespace) : le pattern worktree reset budget par task est couvert.
- **adaptateur Omega** : omega-sandbox (repo clone + reset par task)
- **preuves** : `sweagent/environment/swe_env.py:51-149`

### `swea-repo-diff` — Edit fenetre 1000 lignes : diff diff_state sur tools windowed

- **domaine** : execution · **capacite** : event-store · **port** : EventStore · **cout** : S · **statut** : planned
- **what** : L'edition de fichier est diffe : le tools edit_windowed est windowed: seuils limitant le nombre de lignes et linter-only.
- **how** : tools/windowed_edit_linting/ + tools/diff_state/ : l'outil d'edition garde le diff IHM. Omega : EventStore capture le sys ça ; les edits sont des atomicéstics reversibles dans les EventStore...
- **adaptateur Omega** : omega-core (edit diff windowed + reversible)
- **preuves** : `tools/windowed_edit_linting` · `tools/diff_state`

### `swea-forfeit` — Forfeit : capa de rendre la tâch fait à un mode speciale expert

- **domaine** : securite · **capacite** : tool-surface · **port** : ToolSurface · **cout** : S · **statut** : planned
- **what** : Le outil forfeit émet un signal d'abandon de la tâche pendant le loop.
- **how** : tools/forfeit/ + sweagent/agent/agents.py:_ExitForfeit (agents.py:211). Omega : la control de l'agent a un mode expert existe (Execution complète certificate, gold inherent); l'ecart au noyau est le signal 'mockground' agnosticos.
- **adaptateur Omega** : tool forfeit plugin (signal d'abandon coloc)
- **preuves** : `sweagent/agent/agents.py:211` · `tools/forfeit`

### `swea-hooks` — Hooks modélisés partout : agent, env, run, model, tools

- **domaine** : extension · **capacite** : hook · **port** : Hook · **cout** : S · **statut** : adopted
- **what** : Un pattern de hook uniforme (AbstractAgentHook, EnvHook, etc.) avec add_hook sur chaque subsystem, pour des props de réversibilité en betaiment.
- **how** : sweagent/agent/agents.py:233 AbstractAgent.add_hook ; sweagent/environment/swe_env.py:100 SWEEnv.add_hook ; hooks/__init__ par side. Omega : la totalité des hooks Omega couvre déjà le pattern Hook ; a retenir : le batch run hook a aussi des side effects sur les sous taches.
- **adaptateur Omega** : omega-ports::Hook (hooks par sous-systems)
- **preuves** : `sweagent/agent/agents.py:224-242`

### `swea-run-batch` — Run batch reproducible : run/inspector/compare_runs sur les seeds

- **domaine** : observabilite · **capacite** : event-store · **port** : EventStore · **cout** : M · **statut** : planned
- **what** : Une suite de run (run.py, run_batch.py, run_replay...) rejoue une trace, compare les runs, extrait les preducts diff.
- **how** : sweagent/run/run.py + run_batch.py + run_replay.py + compare_runs.py + extract_pred.py. Omega : la reutilisation de trace (Traces panel) est deja EventStore ; l'ecart : comparaison + extraction de diffs entre runs.
- **adaptateur Omega** : omega-app (comparaison/replay de run)
- **preuves** : `sweagent/run/run.py` · `sweagent/run/run_replay.py`

### `swea-cost-tracking` — Traçage de cout et statuts de run : tokens par forme, quick_stats

- **domaine** : observabilite · **capacite** : telemetry-sink · **port** : TelemetrySink · **cout** : S · **statut** : rejected
- **what** : Le cout de chaque tour et le nombre de forms du model sont tracker par run, avec une stats rapide sur les batches.
- **how** : sweagent/run/quick_stats.py + api_cost/_l. Omega : TelemetrySink + EventStore.
- **adaptateur Omega** : omega-otel (quick stats + coût)
- **preuves** : `sweagent/run/quick_stats.py`

