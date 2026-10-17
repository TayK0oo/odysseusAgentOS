# OMEGA-COMPLET — le combine unifie

> Genere par `omega/scripts/omega_complet.py` (deterministe, regenerable). Fusionne **toutes** les dissections de harnais en un seul plan d'action.

## 1. Synthese

- harnais disseques : **34**
- fonctionnalites inventoriees : **727** (adoptees 158 · planifiees 547 · ecartees 22)
- capacites visees (regroupees) : **106**
- popularite : **mesuree** (API GitHub, somme des etoiles des harnais demandeurs)
- preuves REA (intake machine) disponibles pour : 0 harnais

### Ce que le consensus designe (>= 4 harnais demandeurs)

| Capacite | Harnais | Popularite (etoiles) | Cout min | Famille |
|---|---|---|---|---|
| `context-engine` | 28 | 1,773,172 | S | Contexte & memoire de travail |
| `subagent-runner` | 22 | 1,492,452 | S | Orchestration |
| `tool-surface` | 20 | 1,198,361 | S | Outils & surface d'action |
| `config-source` | 19 | 1,007,547 | S | Configuration & extension |
| `registry` | 16 | 1,379,082 | S | Configuration & extension |
| `event-store` | 14 | 1,085,780 | S | Observabilite & tracabilite |
| `agent-store` | 12 | 1,123,880 | S | Orchestration |
| `exec-policy` | 12 | 814,082 | S | Execution & securite |
| `memory-store` | 12 | 448,944 | S | Contexte & memoire de travail |
| `(execution)` | 11 | 993,736 | S | Execution & securite |
| `sandbox-backend` | 11 | 429,844 | M | Execution & securite |
| `(ui)` | 9 | 770,059 | M | Observabilite & tracabilite |
| `permission-policy` | 9 | 746,393 | S | Execution & securite |
| `scheduler-job` | 9 | 394,449 | M | Orchestration |
| `telemetry-sink` | 9 | 323,451 | S | Observabilite & tracabilite |
| `skill-store` | 9 | 239,899 | S | Contexte & memoire de travail |
| `(automatisation)` | 7 | 636,277 | M | Autres |
| `tool` | 7 | 432,964 | S | Outils & surface d'action |
| `llm-provider` | 6 | 464,162 | S | Routage & modeles |
| `router-policy` | 5 | 271,956 | S | Routage & modeles |
| `hook` | 4 | 429,089 | M | Orchestration |
| `agent-server` | 4 | 281,649 | M | Orchestration |
| `task-classifier` | 4 | 222,581 | S | Routage & modeles |
| `channel` | 4 | 221,222 | M | Orchestration |
| `agent-loop` | 4 | 118,609 | S | Orchestration |
| `hook-bus` | 4 | 4,454 | S | Orchestration |

## 2. Le combine, capacite par capacite

### Autres

#### `(automatisation)` — 7 harnais, cout M

- **harnais demandeurs** : autogpt, continue, deepseek-harness, goose, langgraph, qwen-code, tabby
- **popularite cumulee** : 636,277 etoiles
- **adaptateurs proposes** : `omega-app (planificateur cron + index de jobs)`, `omega-core (scheduler cron + taches persistees)`, `omega-sandbox::LocalSandboxBackend (worktrees) + omega-server (API de revue)`, `omega-server (ingress webhook) + omega-app (mapping payload -> nœud)`, `omega/scripts/run-local.sh + omega-server (build UI + serveur une seule origine)`
- **exemplaire de reference** : `deepseek-harness` — *Jobs d'arriere-plan, planification, webhooks, moteur de workflow* (port `—`, cout M)
  - comment il le fait : ctx.jobs est un registre de jobs (JobRegistry, service abstrait) et les outils job_* les lisent ou les arretent ; le tool bash s'enregistre dans ctx.jobs au demarrage pour run_in_background — sans registre de jobs, ou avec enableRunInBackground false, le tool enregistre un schema foreground-only SANS le parametre, donc le catalogue d'outils reflete la composition ; ctx.schedule gere des messages p
  - preuves : `packages/jobs/jobs/src/index.ts:88`, `packages/schedule/schedule/src/index.ts:99`, `packages/experimental/webhook/src/index.ts:58`, `packages/workflow/workflow/src/index.ts:130`

#### `workflow-engine` — 3 harnais, cout S

- **harnais demandeurs** : crewai, devin, langgraph
- **popularite cumulee** : 103,159 etoiles
- **adaptateurs proposes** : `plugins/flow-runtime (DAG etatise + persistance + human-in-the-loop)`, `plugins/workflow-dag`, `plugins/workflow-graph (graphe d'etats + canaux, au-dessus de omega-app::App)`, `plugins/workflow-graph::errors (route d'erreur vers un noud)`, `plugins/workflow-graph::routing (BranchSpec + Send fan-out)`
- **exemplaire de reference** : `langgraph` — *Graphe d'etats declaratif (StateGraph -> CompiledStateGraph)* (port `WorkflowEngine (build/compile/run)`, adaptateur `plugins/workflow-graph (graphe d'etats + canaux, au-dessus de omega-app::App)`, cout L)
  - comment il le fait : StateGraph(Generic[StateT, ContextT, InputT, OutputT]) expose add_node (5 surcharges, inferance du schema d'entree/retour via get_type_hints, detection de Command Literal pour les destinations, noud error_handler interne `__error_handler__<node>`), add_edge (accepte une liste de sources = barriere named), add_conditional_edges (path callable/Runnable + path_map optionnel), add_sequence ; validate(
  - preuves : `libs/langgraph/langgraph/graph/state.py:131`, `libs/langgraph/langgraph/graph/state.py:667`, `libs/langgraph/langgraph/graph/state.py:928`, `libs/langgraph/langgraph/graph/state.py:982`

#### `credentials` — 1 harnais, cout S

- **harnais demandeurs** : deepseek-harness
- **popularite cumulee** : 251,248 etoiles
- **adaptateurs proposes** : `omega-config (refs de secrets, planned)`
- **exemplaire de reference** : `deepseek-harness` — *Couture credentials : references en config, valeurs chez le provider* (port `CredentialProvider`, adaptateur `omega-config (refs de secrets, planned)`, cout S)
  - comment il le fait : CredentialProvider est un service abstrait ; ctx.credentials resout PAR OPERATION, donc un credential tourne atteint la toute prochaine requete ; le controleur de settings expose des vues sans valeur et un stockage en ecriture seule, et chaque refus est classe a la projection, pas sur la definition de la couture ; ctx.authorization enregistre des flux d'obtention (un flux = le credential qu'il ecr
  - preuves : `packages/credentials/credentials/src/index.ts:118-171`, `docs/capability-seams.md:652-654`, `docs/subsystems/credentials.md`

#### `(memoire)` — 1 harnais, cout L

- **harnais demandeurs** : autogpt
- **popularite cumulee** : 187,617 etoiles
- **adaptateurs proposes** : `omega-app (job de consolidation nocturne + ratification)`
- **exemplaire de reference** : `autogpt` — *Consolidation nocturne (« dream pass ») par lots et ratification* (port `—`, adaptateur `omega-app (job de consolidation nocturne + ratification)`, cout L)
  - comment il le fait : `copilot/dream/nightly_batch.py` (`run_nightly_batch_submit`, `NightlyBatchResult`) construit et soumet un lot, `batch_submit.py`/`batch_callbacks.py` gerent la soumission et la reprise des resultats, `ratification.py`/`ratification_hits.py` decident ce qui est applique et `apply.py` ecrit ; `scheduling.py` enregistre les marqueurs `NIGHTLY_BATCH_REGISTRATION_PREFIX` et `COMMUNITY_REBUILD_REGISTRA
  - preuves : `autogpt_platform/backend/backend/copilot/dream/nightly_batch.py:54-120`, `autogpt_platform/backend/backend/copilot/dream/ratification.py:1-40`, `autogpt_platform/backend/backend/executor/batch_executor.py:1-40`, `autogpt_platform/backend/backend/app.py:46-53`

#### `loop-guard` — 1 harnais, cout M

- **harnais demandeurs** : gemini-cli
- **popularite cumulee** : 107,412 etoiles
- **adaptateurs proposes** : `omega-core::AgentLoop (garde de sortie de boucle a ajouter)`
- **exemplaire de reference** : `gemini-cli` — *Detection de boucles (outils, contenu streame, LLM) + coupure* (port `LoopGuard (detect/break) — nouveau port`, adaptateur `omega-core::AgentLoop (garde de sortie de boucle a ajouter)`, cout M)
  - comment il le fait : LoopDetectionService (services/loopDetectionService.ts:133) maintient toolCallHistory (:139), streamContentHistory + stats (:142-143), compteurs de tours LLM (:152-154) et flag disabledForSession (:157) ; LoopDetected est un code d'arret de tour (core/turn.ts:66). Omega : rien de tel — la boucle ne garde pas de trace d'anomalies de repetition.
  - preuves : `packages/core/src/services/loopDetectionService.ts:123-157`, `packages/core/src/core/turn.ts:66`

#### `orchestrator` — 1 harnais, cout S

- **harnais demandeurs** : crewai
- **popularite cumulee** : 59,778 etoiles
- **adaptateurs proposes** : `plugins/agent-executor (automate etatise + garde max_iter)`, `plugins/conditional-task (predicate sur output precedent)`, `plugins/crew-process (sequence + hierarchy manager)`, `plugins/task-graph (contexte amont + sorties typées)`
- **exemplaire de reference** : `crewai` — *Crew sequentiel vs hierarchique (manager agent/LLM)* (port `Orchestration (sequence/hierarchy)`, adaptateur `plugins/crew-process (sequence + hierarchy manager)`, cout L)
  - comment il le fait : Crew (crew.py:164) dispatche vers _run_sequential_process (l.1522) ou _run_hierarchical_process (l.1526) selon Process (process.py:4-10) ; _create_manager_agent (l.1531) construit le manager a partir de manager_llm (l.278) ou manager_agent (l.283), avec validations check_manager_llm (l.725-741) et interdiction que le manager figure dans agents (l.737-741).
  - preuves : `lib/crewai/src/crewai/crew.py:164-358`, `lib/crewai/src/crewai/crew.py:1056-1058`, `lib/crewai/src/crewai/crew.py:1522-1571`, `lib/crewai/src/crewai/process.py:4-10`

#### `output-guardrail` — 1 harnais, cout M

- **harnais demandeurs** : crewai
- **popularite cumulee** : 59,778 etoiles
- **adaptateurs proposes** : `plugins/guardrails (chaine validate-correction-retry)`
- **exemplaire de reference** : `crewai` — *Guardrails de sortie (fonction / LLM / hallucination) + retries* (port `OutputGuardrail (validate/rewrite)`, adaptateur `plugins/guardrails (chaine validate-correction-retry)`, cout M)
  - comment il le fait : Champs guardrail(s) + guardrail_max_retries=3 (task.py:252-280) avec validateurs de signature (l.315-393) ; process_guardrail (utilities/guardrail.py:123) retourne GuardrailResult (l.60) ; LLMGuardrail (tasks/llm_guardrail.py:49) et HallucinationGuardrail (tasks/hallucination_guardrail.py:20) ; events emis via llm_guardrail_events.
  - preuves : `lib/crewai/src/crewai/task.py:252-393`, `lib/crewai/src/crewai/utilities/guardrail.py:60-123`, `lib/crewai/src/crewai/tasks/llm_guardrail.py:39-49`, `lib/crewai/src/crewai/tasks/hallucination_guardrail.py:20`

#### `planner` — 1 harnais, cout M

- **harnais demandeurs** : crewai
- **popularite cumulee** : 59,778 etoiles
- **adaptateurs proposes** : `plugins/crew-planner (plan LLM injecte en contexte)`
- **exemplaire de reference** : `crewai` — *Planning prealable (CrewPlanner) + planning_config* (port `Planner (plan -> taches)`, adaptateur `plugins/crew-planner (plan LLM injecte en contexte)`, cout M)
  - comment il le fait : Champ planning (crew.py:347-351) + planning_llm (l.351-358) ; CrewPlanner (utilities/planning_handler.py:37) preprompte un agent planner (defaut gpt-5.4-mini, l.55) et retourne PlannerTaskPydanticOutput (_handle_crew_planning, l.57) ; agent/core.py:331-344 deprecie reasoning au profit de planning_config.
  - preuves : `lib/crewai/src/crewai/crew.py:347-358`, `lib/crewai/src/crewai/utilities/planning_handler.py:37-58`, `lib/crewai/src/crewai/agent/core.py:331-446`

#### `clarify-loop` — 1 harnais, cout S

- **harnais demandeurs** : gpt-engineer
- **popularite cumulee** : 55,024 etoiles
- **adaptateurs proposes** : `plugins/clarify-loop (sortie sur marqueur + hypotheses explicitees)`
- **exemplaire de reference** : `gpt-engineer` — *Boucle de clarification avant generation (une question a la fois)* (port `ClarificationLoop (boucle humain <-> modele)`, adaptateur `plugins/clarify-loop (sortie sur marqueur + hypotheses explicitees)`, cout S)
  - comment il le fait : clarified_gen (tools/custom_steps.py:122-195) boucle sur ai.next avec le preprompt 'clarify' ; sortie sur 'nothing to clarify' dans la reponse ou reponse commencant par 'no' ; l'entree 'c' (ou vide) signifie 'fais tes propres hypotheses', auquel cas un tour supplementaire impose 'Make your own assumptions and state them explicitly before starting' ; apres chaque question la consigne 'Is anything e
  - preuves : `gpt_engineer/tools/custom_steps.py:122-195`, `gpt_engineer/preprompts/clarify`, `gpt_engineer/applications/cli/main.py:304-309`, `gpt_engineer/applications/cli/main.py:483-488`

#### `code-emitter` — 1 harnais, cout S

- **harnais demandeurs** : gpt-engineer
- **popularite cumulee** : 55,024 etoiles
- **adaptateurs proposes** : `plugins/code-emitter (parseur blocs + assainissement de chemins)`
- **exemplaire de reference** : `gpt-engineer` — *Generation de code : du chat aux fichiers (parsing + nettoyage de chemins)* (port `CodeEmitter (artefacts texte -> fichiers)`, adaptateur `plugins/code-emitter (parseur blocs + assainissement de chemins)`, cout S)
  - comment il le fait : chat_to_files_dict (core/chat_to_files.py:38-66) applique la regex `(\S+)\n\s*```[^\n]*\n(.+?)```` en DOTALL puis, pour chaque chemin : supprime [:<>"\|?*], decoche les crochets [ ] et les backticks, retire le `]`/`:` final, et fait strip() ; le resultat est un FilesDict dont __setitem__ refuse cle non str/Path et valeur non str (core/files_dict.py:29-53). Le prompt impose le format (preprompts/fi
  - preuves : `gpt_engineer/core/chat_to_files.py:38-66`, `gpt_engineer/core/files_dict.py:19-53`, `gpt_engineer/core/default/steps.py:75-94`, `gpt_engineer/core/default/steps.py:121-150`

#### `eval-harness` — 1 harnais, cout L

- **harnais demandeurs** : gpt-engineer
- **popularite cumulee** : 55,024 etoiles
- **adaptateurs proposes** : `omega-eval-harness (runner de taches, assertions et taux de reussite)`
- **exemplaire de reference** : `gpt-engineer` — *Banc d'essai : Tache/Assertion/Assertable + agent charge par importlib* (port `BenchmarkRunner (taches + assertions + agregats)`, adaptateur `omega-eval-harness (runner de taches, assertions et taux de reussite)`, cout L)
  - comment il le fait : benchmark/types.py:32-94 : Assertable{files, env, process, stdout, stderr}, Assertion = Callable[[Assertable], bool], Task{name, initial_code, command, prompt, assertions}, Benchmark{name, tasks, timeout}, TaskResult.success_rate = fraction d'assertions vraies + to_dict qui ajoute 'solved' ; benchmark/run.py:26-87 appelle agent.improve(task.initial_code, task.prompt), chronometre, upload dans un D
  - preuves : `gpt_engineer/benchmark/types.py:32-94`, `gpt_engineer/benchmark/run.py:26-150`, `gpt_engineer/benchmark/__main__.py:43-151`, `gpt_engineer/benchmark/benchmarks/mbpp/load.py:29-49`

#### `patch-apply` — 1 harnais, cout M

- **harnais demandeurs** : gpt-engineer
- **popularite cumulee** : 55,024 etoiles
- **adaptateurs proposes** : `plugins/patch-apply (diff unifie + timeout de parsing)`
- **exemplaire de reference** : `gpt-engineer` — *Mode improve : edition par diff unifie git + application* (port `PatchApplier (diff unifie -> arbre de travail)`, adaptateur `plugins/patch-apply (diff unifie + timeout de parsing)`, cout M)
  - comment il le fait : improve_fn (core/default/steps.py:271-312) construit [SystemMessage(roadmap+improve+philosophy), HumanMessage(files_dict.to_chat() numerote), HumanMessage(prompt)] puis appelle _improve_loop (:315-338) ; en cas d'erreurs, jusqu'a MAX_EDIT_REFINEMENT_STEPS=2 tours (core/default/constants.py:10) ou l'on renvoie au modele 'Some previously produced diffs were not on the requested format, or the code p
  - preuves : `gpt_engineer/core/default/steps.py:271-360`, `gpt_engineer/core/default/constants.py:1-10`, `gpt_engineer/core/chat_to_files.py:69-120`, `gpt_engineer/core/chat_to_files.py:123-161`

#### `patch-validate` — 1 harnais, cout L

- **harnais demandeurs** : gpt-engineer
- **popularite cumulee** : 55,024 etoiles
- **adaptateurs proposes** : `plugins/patch-validate (reparage des hunks par similarite)`
- **exemplaire de reference** : `gpt-engineer` — *Validation + AUTO-correction des hunks (le coeur technique)* (port `PatchValidator (validation + reparage deterministe)`, adaptateur `plugins/patch-validate (reparage des hunks par similarite)`, cout L)
  - comment il le fait : core/diff.py : is_similar/count_ratio (:381-419) comparent deux lignes par RAPPORT DE MULTIPLICITE DES CARACTERES (espaces et casse ignores, seuil 0.9) -- pas de diff structurel. find_start_line (:133-198) : si la premiere ligne du hunk est un ADD, on cherche la premiere ligne non-ADD dans le fichier et on ancre le hunk une ligne avant ; sinon on cartographie tous les numeros de ligne candidats, e
  - preuves : `gpt_engineer/core/diff.py:44-78`, `gpt_engineer/core/diff.py:133-198`, `gpt_engineer/core/diff.py:200-286`, `gpt_engineer/core/diff.py:288-309`

#### `self-healing` — 1 harnais, cout M

- **harnais demandeurs** : gpt-engineer
- **popularite cumulee** : 55,024 etoiles
- **adaptateurs proposes** : `plugins/repair-loop (boucle autoréparatrice bornée + budget)`
- **exemplaire de reference** : `gpt-engineer` — *Self-healing : executer, capturer l'erreur, reparer, re-executer* (port `RepairLoop (execute -> erreur -> patch -> re-execute)`, adaptateur `plugins/repair-loop (boucle autoréparatrice bornée + budget)`, cout M)
  - comment il le fait : tools/custom_steps.py:40-119 : la boucle `while attempts < MAX_SELF_HEAL_ATTEMPTS` (=10, :19) execute entrypoint via execution_env.popen (DiskExecutionEnv, non bloquant) puis p.communicate() pour recuperer stdout/stderr ; l'echec est declenche par returncode != 0 ET != 2 (le code 2 est traite comme un arret volontaire) ; le prompt de reparation est reconstruit a la volee : 'A program with this spe
  - preuves : `gpt_engineer/tools/custom_steps.py:19-119`, `gpt_engineer/applications/cli/main.py:310-315`, `gpt_engineer/applications/cli/main.py:490-494`, `gpt_engineer/core/default/disk_execution_env.py:62-70`

#### `vcs-snapshot` — 1 harnais, cout S

- **harnais demandeurs** : gpt-engineer
- **popularite cumulee** : 55,024 etoiles
- **adaptateurs proposes** : `plugins/vcs-stage (points de retour par fichier avant ecriture)`
- **exemplaire de reference** : `gpt-engineer` — *Garde git : stager les modifications non commitees avant d'ecraser* (port `SnapshotStore (points de retour)`, adaptateur `plugins/vcs-stage (points de retour par fichier avant ecriture)`, cout S)
  - comment il le fait : core/git.py:10-85 : is_git_installed (shutil.which), is_git_repo (git rev-parse --is-inside-work-tree), has_uncommitted_changes (git diff --exit-code), filter_files_with_uncommitted_changes (git diff --name-only croise avec le FilesDict), stage_files (git add) ; stage_uncommitted_to_git (:71-85) n'initialise un depot QUE si on n'est pas en mode improve (en improve le depot doit deja exister et on 
  - preuves : `gpt_engineer/core/git.py:10-85`, `gpt_engineer/applications/cli/main.py:548-550`, `gpt_engineer/applications/cli/file_selector.py:413-414`, `tests/core/test_git.py:16-52`

#### `edit-apply` — 1 harnais, cout S

- **harnais demandeurs** : aider
- **popularite cumulee** : 49,588 etoiles
- **adaptateurs proposes** : `plugins/edit-search-replace`, `plugins/edit-udiff-patch`, `plugins/edit-wholefile`
- **exemplaire de reference** : `aider` — *Formats diff unifie et patch V4A* (port `EditApplier (apply_udiff/apply_patch)`, adaptateur `plugins/edit-udiff-patch`, cout M)
  - comment il le fait : UnifiedDiffCoder parse des diffs unifies standards ; PatchCoder implemente un mini-langage de patch V4A (ActionType/Chunk/PatchAction) avec parseur derive de apply_patch ; application atomique par fichier.
  - preuves : `aider/coders/udiff_coder.py:46-49`, `aider/coders/patch_coder.py:13-46`, `aider/coders/patch_coder.py:210-217`, `aider/coders/patch_coder.py:295-320`

#### `lint-check` — 1 harnais, cout S

- **harnais demandeurs** : aider
- **popularite cumulee** : 49,588 etoiles
- **adaptateurs proposes** : `plugins/lint-after-edit`
- **exemplaire de reference** : `aider` — *Linter multi-langage apres edition* (port `LintCheck (lint_file)`, adaptateur `plugins/lint-after-edit`, cout S)
  - comment il le fait : Classe Linter avec registre par langage (set_linter), lint() dispatchant vers py_compile/flake8 ou commandes externes ; lint_python_compile et basic_lint en repli sans dependance ; appels automatiques apres chaque edition.
  - preuves : `aider/linter.py:21-82`, `aider/linter.py:118-172`, `aider/linter.py:177-234`, `aider/commands.py:356-410`

#### `vcs-commit` — 1 harnais, cout S

- **harnais demandeurs** : aider
- **popularite cumulee** : 49,588 etoiles
- **adaptateurs proposes** : `plugins/git-auto-commit`
- **exemplaire de reference** : `aider` — *Git auto-commit + message genere* (port `VcsCommit (commit/message)`, adaptateur `plugins/git-auto-commit`, cout S)
  - comment il le fait : auto_commit appelle GitRepo.commit avec les fichiers edites ; le message est genere par le weak_model via get_commit_message ; attribution configurable (author vs committer, Co-authored-by, prefixe aider:) ; dirty_commit sauvegarde l'etat pre-edit.
  - preuves : `aider/coders/base_coder.py:2375-2400`, `aider/repo.py:131-270`, `aider/repo.py:326-375`, `aider/models.py:622-624`

#### `hitl-gate` — 1 harnais, cout L

- **harnais demandeurs** : langgraph
- **popularite cumulee** : 43,381 etoiles
- **adaptateurs proposes** : `plugins/hitl-gate (pause repriseable par thread, au-dessus de omega-app::EventStore)`
- **exemplaire de reference** : `langgraph` — *Human-in-the-loop : interrupt() reprendable + Command(resume/goto)* (port `HumanGate (interrupt/resume)`, adaptateur `plugins/hitl-gate (pause repriseable par thread, au-dessus de omega-app::EventStore)`, cout L)
  - comment il le fait : interrupt(value, response_schema=...) leve GraphInterrupt (exception GraphBubbleUp donc non capturee par le retry) la premiere fois ; au rejeu du noud la valeur est lue dans le scratchpad/resume-map, avec appariement par ordre pour plusieurs interrupt dans un meme noud. Le client reprend via Command(resume=<valeur>|{interrupt_id: valeur}, goto=..., update=..., graph=Command.PARENT) ; le checkpoint
  - preuves : `libs/langgraph/langgraph/types.py:833`, `libs/langgraph/langgraph/types.py:903`, `libs/langgraph/langgraph/errors.py:103`, `libs/langgraph/langgraph/errors.py:51`

#### `retry-policy` — 1 harnais, cout M

- **harnais demandeurs** : langgraph
- **popularite cumulee** : 43,381 etoiles
- **adaptateurs proposes** : `plugins/retry-policy (policies par tache, hors du routeur modele)`
- **exemplaire de reference** : `langgraph` — *Politiques de retry et de timeout par noud (backoff, jitter, idle)* (port `RetryPolicy (attempt/backoff)`, adaptateur `plugins/retry-policy (policies par tache, hors du routeur modele)`, cout M)
  - comment il le fait : RetryPolicy(NamedTuple) {initial_interval=0.5, backoff_factor=2, max_interval=128, max_attempts=3, jitter, retry_on} et TimeoutPolicy (frozen) {run_timeout, idle_timeout, refresh_on: auto|heartbeat} portes par tache (PregelExecutableTask.retry_policy/timeout) ; run_with_retry/arun_with_retry (_retry.py) resolvent la premiere politique correspondante, attendent interval*backoff^(n-1)+jitter et lace
  - preuves : `libs/langgraph/langgraph/types.py:427`, `libs/langgraph/langgraph/types.py:461`, `libs/langgraph/langgraph/pregel/_retry.py:573`, `libs/langgraph/langgraph/pregel/_retry.py:660`

#### `stream-surface` — 1 harnais, cout M

- **harnais demandeurs** : langgraph
- **popularite cumulee** : 43,381 etoiles
- **adaptateurs proposes** : `omega-server (flux SSE du cockpit, a ajouter au POST /api/run)`
- **exemplaire de reference** : `langgraph` — *Streaming multi-modes (values/updates/messages/custom/checkpoints/tasks/debug)* (port `StreamSink (values/updates/messages/custom)`, adaptateur `omega-server (flux SSE du cockpit, a ajouter au POST /api/run)`, cout M)
  - comment il le fait : StreamMode = Literal["values","updates","checkpoints","tasks","debug","messages","custom"] ; stream()/astream() acceptent une ou plusieurs modes et un print_mode de debug ; une couche v2/v3 ajoute astream_events(version v1|v2|v3) et un bus StreamMux qui dispatche les ProtocolEvent (seq monotone, namespace) a des StreamTransformer extensibles (required_stream_modes, projections exposees sur le run 
  - preuves : `libs/langgraph/langgraph/types.py:131`, `libs/langgraph/langgraph/pregel/main.py:2709`, `libs/langgraph/langgraph/pregel/main.py:3704`, `libs/langgraph/langgraph/stream/_types.py:44`

#### `debug-session` — 1 harnais, cout L

- **harnais demandeurs** : gpt-pilot
- **popularite cumulee** : 33,654 etoiles
- **exemplaire de reference** : `gpt-pilot` — *Chasseur de bogues et pair programming* (port `Debugger`, cout L)
  - comment il le fait : core/agents/bug_hunter.py:57 BugHunter, check_logs (103), ask_user_to_test (144), start_pair_programming (241), get_bug_reproduction_instructions (82) ; schemas HuntConclusionOptions et ImportantLogsForDebugging ; IterationStatus AWAITING_USER_TEST, BUG_REPRODUCTION, HUNTING_FOR_BUG.
  - preuves : `core/agents/bug_hunter.py:57`, `core/agents/bug_hunter.py:103`, `core/agents/bug_hunter.py:144`, `core/agents/bug_hunter.py:241`

#### `file-system` — 1 harnais, cout S

- **harnais demandeurs** : gpt-pilot
- **popularite cumulee** : 33,654 etoiles
- **exemplaire de reference** : `gpt-pilot` — *Systeme de fichiers virtuel et regles d'ignorance* (port `FileSystem`, cout S)
  - comment il le fait : core/disk/vfs.py:12 VirtualFileSystem, LocalDiskVFS (115), MemoryVFS (88) ; core/disk/ignore.py IgnoreMatcher avec ignore_size_threshold 50 Ko et ignore_paths (example-config.json:64) ; core/state/state_manager.py:628 init_file_system.
  - preuves : `core/disk/vfs.py:12`, `core/disk/vfs.py:115`, `core/disk/ignore.py`, `core/state/state_manager.py:628`

#### `persistence` — 1 harnais, cout L

- **harnais demandeurs** : gpt-pilot
- **popularite cumulee** : 33,654 etoiles
- **exemplaire de reference** : `gpt-pilot` — *Modeles de base de donnees du projet* (port `Store`, cout L)
  - comment il le fait : core/db/models/ : project.py, branch.py, project_state.py:59, specification.py:22, file.py:15, file_content.py, exec_log.py:16, llm_request.py, chat_convo.py, chat_message.py, user_input.py, knowledge_base.py, base.py ; migrations Alembic core/db/migrations/versions/* ; base par defaut sqlite+aiosqlite (example-config.json:52).
  - preuves : `core/db/models/project_state.py:59`, `core/db/models/specification.py:22`, `core/db/models/file.py:15`, `core/db/models/exec_log.py:16`

#### `review-loop` — 1 harnais, cout M

- **harnais demandeurs** : gpt-pilot
- **popularite cumulee** : 33,654 etoiles
- **exemplaire de reference** : `gpt-pilot` — *Depanneur : boucle de revue, detection de boucle, solution alternative* (port `Reviewer`, cout M)
  - comment il le fait : core/agents/troubleshooter.py:35 Troubleshooter, create_iteration (63), propose_solution (48), complete_task (148), get_user_feedback (244), trace_loop (426) ; LOOP_THRESHOLD=3 (22) ; IterationStatus HUNTING_FOR_BUG, NEW_FEATURE_REQUESTED, PROBLEM_SOLVER ; generate_bug_report (368).
  - preuves : `core/agents/troubleshooter.py:35`, `core/agents/troubleshooter.py:63`, `core/agents/troubleshooter.py:244`, `core/agents/troubleshooter.py:426`

#### `task-execution` — 1 harnais, cout L

- **harnais demandeurs** : gpt-pilot
- **popularite cumulee** : 33,654 etoiles
- **exemplaire de reference** : `gpt-pilot` — *Developpeur : decomposition de tache en etapes* (port `TaskRunner`, cout L)
  - comment il le fait : core/agents/developer.py:91 Developer, breakdown_current_task (204), breakdown_current_iteration (121), ask_to_execute_task (349), set_next_steps (313) ; StepType (32) et TaskSteps (80) ; remove_duplicate_steps (330) ; trace_code_event task-start/task-end (303/34).
  - preuves : `core/agents/developer.py:91`, `core/agents/developer.py:204`, `core/agents/developer.py:349`, `core/agents/developer.py:313`

#### `vcs` — 1 harnais, cout S

- **harnais demandeurs** : gpt-pilot
- **popularite cumulee** : 33,654 etoiles
- **exemplaire de reference** : `gpt-pilot` — *Finalisation de tache et commit Git* (port `VCS`, cout S)
  - comment il le fait : core/agents/task_completer.py:11 TaskCompleter, git_commit via GitMixin, log_task_completed (569) avec end_result telemetry ; core/agents/git.py:92 git_commit, 35 init_git_if_needed (git init, .gitignore, premier commit).
  - preuves : `core/agents/task_completer.py:11`, `core/agents/git.py:92`, `core/agents/git.py:35`

#### `apply-engine` — 1 harnais, cout L

- **harnais demandeurs** : void
- **popularite cumulee** : 28,752 etoiles
- **adaptateurs proposes** : `plugins/diff-zone (zones + realign + FindDiffs)`, `plugins/search-replace-apply (parseur streaming + FindDiffs)`
- **exemplaire de reference** : `void` — *Moteur de zones de diff (DiffZone / CtrlKZone / TrackingZone)* (port `ApplyEngine (apply/accept/reject)`, adaptateur `plugins/diff-zone (zones + realign + FindDiffs)`, cout L)
  - comment il le fait : Trois types de DiffArea (editCodeServiceTypes.ts:46-90) ; seul DiffZone peut streamer (_streamState.isStreaming, 71-80). findDiffs (helpers/findDiffs.ts, 251 l.) produit un ComputedDiff de type edit | insertion | deletion (editCodeServiceTypes.ts:6-31). A chaque evenement de texte, _realignAllDiffAreasLines (editCodeService.ts:876-945) decale les bornes de toutes les zones du fichier puis relance 
  - preuves : `src/vs/workbench/contrib/void/common/editCodeServiceTypes.ts:6-90`, `src/vs/workbench/contrib/void/browser/helpers/findDiffs.ts:1-251`, `src/vs/workbench/contrib/void/browser/editCodeService.ts:157-176`, `src/vs/workbench/contrib/void/browser/editCodeService.ts:255-265`

#### `completion-provider` — 1 harnais, cout M

- **harnais demandeurs** : kilo-code
- **popularite cumulee** : 27,626 etoiles
- **adaptateurs proposes** : `plugins/autocomplete-fim`
- **exemplaire de reference** : `kilo-code` — *Autocompletion inline (FIM) et chat-autocomplete* (port `CompletionProvider (FIM)`, adaptateur `plugins/autocomplete-fim`, cout M)
  - comment il le fait : packages/kilo-gateway/src/fim.ts:1-40 resout la cible FIM : resolveFimTarget(provider, model) retourne {provider:"kilo"} (endpoint gateway fim/completions, modele par defaut mistralai/codestral-2501), "mistral" (via MISTRAL_API_KEY) ou "inception" (https://api.inceptionlabs.ai/v1/fim/completions, INCEPTION_API_KEY) ; autocomplete.ts resout le modele et mistral-fim-endpoint.ts parle le protocole Mi
  - preuves : `packages/kilo-gateway/src/fim.ts:1-40`, `packages/kilo-gateway/src/autocomplete.ts`, `packages/kilo-gateway/src/mistral-fim-endpoint.ts`, `packages/kilo-vscode/src/services/autocomplete/AutocompleteServiceManager.ts`

#### `cost-accounting` — 1 harnais, cout S

- **harnais demandeurs** : kilo-code
- **popularite cumulee** : 27,626 etoiles
- **adaptateurs proposes** : `omega-otel (sinks) + omega-router (scoring cout)`
- **exemplaire de reference** : `kilo-code` — *Comptabilite par run d'agent (tokens, cout, issue)* (port `CostLedger (record/aggregate)`, adaptateur `omega-otel (sinks) + omega-router (scoring cout)`, cout S)
  - comment il le fait : opencode/src/kilocode/agent-usage/index.ts:1-45 definit Run (id = premier message assistant, stable et unique, avec parentID + agent), Tokens{input, output, reasoning, cache{read, write}}, Outcome=[completed, cancelled, errored, running] et interroge la base via drizzle ; provider-usage.ts:1-45 agrege des UsageSnapshot par fournisseur via des adaptateurs (cloud, codex, minimax) avec cache TTL (suc
  - preuves : `packages/opencode/src/kilocode/agent-usage/index.ts:1-45`, `packages/core/src/kilocode/provider-usage.ts:1-45`, `packages/core/src/kilocode/provider-usage/cloud.ts`, `packages/core/src/kilocode/cost/max-cost-nudge.ts:1-45`

#### `session-board` — 1 harnais, cout M

- **harnais demandeurs** : kilo-code
- **popularite cumulee** : 27,626 etoiles
- **adaptateurs proposes** : `plugins/session-board`
- **exemplaire de reference** : `kilo-code` — *Tableau de sessions (messagerie inter-sessions)* (port `BoardStore (append/read)`, adaptateur `plugins/session-board`, cout M)
  - comment il le fait : kilocode/board/store.ts:1-30 cree un schema SQL (Row{id, project_id, parent_id, directory, agent, title, time_created} et MessageRow{board_root_session_id, seq, sender_session_id, recipient, type, body, reply_to, source_message_id}) : le board est ancre sur une session racine et se comporte comme un journal ordonne ; board/{context.ts,enabled.ts,notice.ts} activent/gate la fonction ; cote extensio
  - preuves : `packages/opencode/src/kilocode/board/store.ts:1-30`, `packages/opencode/src/kilocode/board/enabled.ts`, `packages/kilo-vscode/src/kilo-provider/session-board.ts:1-40`, `packages/kilo-vscode/src/webview-ui/src/types/messages/board.ts`

#### `snapshot-store` — 1 harnais, cout M

- **harnais demandeurs** : kilo-code
- **popularite cumulee** : 27,626 etoiles
- **adaptateurs proposes** : `omega-app::FileEventStore (journal) + restauration fichiers (a etendre)`
- **exemplaire de reference** : `kilo-code` — *Retour arriere de session par snapshots git + diff/restauration ciblee* (port `SnapshotStore (capture/restore/diff)`, adaptateur `omega-app::FileEventStore (journal) + restauration fichiers (a etendre)`, cout M)
  - comment il le fait : packages/core/src/snapshot.ts:35-90 expose l'interface Snapshot (capture -> ID content-addresse, files/diff/preview/restore, best-effort avec undefined si desactive ou non supporte) ; packages/core/src/session/revert.ts:31-100 implemente plan() (parcourt les messages assistant posterieurs au message frontiere et collecte, par fichier, le premier snapshot 'start'), stage() (capture l'original si ab
  - preuves : `packages/core/src/snapshot.ts:35-90`, `packages/core/src/session/revert.ts:31-100`, `packages/schema/src/revert.ts:19-27`, `packages/kilo-vscode/src/KiloProvider.ts:495-685`

#### `workspace-manager` — 1 harnais, cout L

- **harnais demandeurs** : kilo-code
- **popularite cumulee** : 27,626 etoiles
- **adaptateurs proposes** : `plugins/agent-manager-worktree`
- **exemplaire de reference** : `kilo-code` — *Agent manager : un git worktree par session/groupe* (port `WorkspaceManager (worktrees)`, adaptateur `plugins/agent-manager-worktree`, cout L)
  - comment il le fait : agent-manager/WorktreeStateManager.ts:1-70 persiste dans .kilo/agent-manager.json un modele decouple Worktree{branch, path, parentBranch, remote, groupId (multi-version), label, prNumber/prUrl/prState, originalBranch, branchOwned, sectionId, pinned} et ManagedSession (plusieurs sessions par worktree) ; SessionTerminalManager/terminal-manager.ts donnent un terminal par worktree, SetupScriptRunner.t
  - preuves : `packages/kilo-vscode/src/agent-manager/WorktreeStateManager.ts:1-70`, `packages/kilo-vscode/src/agent-manager/SetupScriptRunner.ts:1-50`, `packages/kilo-vscode/src/agent-manager/sandbox-bootstrap.ts:1-40`, `packages/kilo-vscode/src/agent-manager/orchestration-domain.ts:1-60`

#### `plan-store` — 1 harnais, cout M

- **harnais demandeurs** : plandex
- **popularite cumulee** : 15,750 etoiles
- **adaptateurs proposes** : `plugins/plan-branches`
- **exemplaire de reference** : `plandex` — *Arbre de versions du plan (branches par modèle)* (port `PlanStore (branch/fork/rewind)`, adaptateur `plugins/plan-branches`, cout M)
  - comment il le fait : Chaque plan possède des branches listables (api.Client.ListBranches, cmd/branches.go) ; l'état courant (CurrentPlanState : PlanResult, PlanApplies, ConvoMessageDescriptions) est versionné par apply et réversible via rewind (AnalyzeRewind, ApplyRewindChanges).
  - preuves : `app/cli/cmd/branches.go:20-40`, `app/shared/data_models.go:295-303`, `app/cli/lib/rewind.go:107-258`

#### `acp-agent` — 1 harnais, cout L

- **harnais demandeurs** : gptme
- **popularite cumulee** : 4,454 etoiles
- **adaptateurs proposes** : `plugins/acp-agent`
- **exemplaire de reference** : `gptme` — *Agent ACP (client + serveur, permissions mappées)* (port `AcpAgent (session/prompt/cancel)`, adaptateur `plugins/acp-agent`, cout L)
  - comment il le fait : GptmeAgent implemente le protocole ACP : initialize/new_session/prompt/cancel/load_session/authenticate (gptme/acp/agent.py:588-1732) ; run_chat_turn() pont entre le tour ACP et le chat gptme avec streaming de tokens (gptme/acp/agent.py:1305-1492) ; _request_tool_permission mappe les permissions d'outils ACP vers la confirmation gptme (gptme/acp/agent.py:376-563) ; client ACP pour piloter des agen
  - preuves : `gptme/acp/agent.py:254-300`, `gptme/acp/agent.py:588-869`, `gptme/acp/agent.py:1128-1492`, `gptme/acp/client.py:1-100`

#### `cost-guard` — 1 harnais, cout S

- **harnais demandeurs** : gptme
- **popularite cumulee** : 4,454 etoiles
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

#### `workflow-runner` — 1 harnais, cout S

- **harnais demandeurs** : windsurf
- **adaptateurs proposes** : `plugins/workflow-runner`
- **exemplaire de reference** : `windsurf` — *Workflows markdown invocables en /slash (manuels uniquement)* (port `WorkflowRunner (discover/invoke)`, adaptateur `plugins/workflow-runner`, cout S)
  - comment il le fait : [documenté] Markdown dans .devin/workflows/ (préféré, héritage .windsurf/) + globaux ~/.codeium/windsurf/global_workflows/ + système Entreprise + built-in (/plan) ; invocation manuelle exclusive via /nom (jamais auto-invoqué, à l'inverse des Skills) ; composables (un workflow en appelle d'autres) ; précédence système > workspace > global > built-in ; 12 000 car. par fichier.
  - preuves : `https://docs.devin.ai/desktop/cascade/workflows.md`

### Configuration & extension

#### `config-source` — 19 harnais, cout S

- **harnais demandeurs** : agentgpt, aider, autogpt, claude-code, codex, continue, crewai, cursor, dsh-desktop, hermes, kilo-code, langgraph, opencode, plandex, rea, tabby, void, windsurf, zed
- **popularite cumulee** : 1,007,547 etoiles
- **adaptateurs proposes** : `omega-config`, `omega-config (ConfigLayerSource + precedence)`, `omega-config (LayeredConfig + ConfigLayerSource)`, `omega-config (LayeredConfig + precedence)`, `omega-config (agents-md discovery)`, `omega-config (auth providers)`, `omega-config (couches + precedence + policy managée)`, `omega-config (couches + precedence)`, `omega-config (couches + precedence) + omega-providers::Capabilities (patch)`, `omega-config (feature flags)`, `omega-config (init wizard)`, `omega-config (oauth credentials)`, `omega-config (profils)`, `omega-config (resolution de blocs a ajouter)`, `omega-config (rules layers)`, `omega-types (allow-list de types serialisables du journal, a porter sur FileEventStore)`, `plugins/agent-profiles`, `plugins/catalog-poll + omega-providers::CatalogSource`, `plugins/rules-inject`, `plugins/rules-scoped`
- **exemplaire de reference** : `dsh-desktop` — *Canaux de release isoles (stable/beta) verifies par en-tetes* (port `ConfigSource (canal de release comme couche de configuration isolee)`, adaptateur `omega-config (ConfigLayerSource + precedence)`, cout M)
  - comment il le fait : `upstream.json` enregistre pour chaque canal le commit amont, la version source, la version du runtime empaquete et le manifeste vendorise ; `activeChannel` est `beta`. Les verifications et telechargements envoient `X-DSH-Desktop-Channel: stable|beta` ; une verification envoie aussi la version courante et un UUID d'installation genere localement et persiste (jamais derive du materiel), un telechar
  - preuves : `upstream.json:1-14`, `dsh-plugin-desktop/src/update-checker.ts:11-30`, `dsh-plugin-desktop/src/update-checker.ts:56-77`, `dsh-plugin-desktop/src/update-download.ts`

#### `registry` — 16 harnais, cout S

- **harnais demandeurs** : aider, autogpt, claude-code, cline, continue, crewai, deepseek-harness, dsh-desktop, dsh-web, gemini-cli, goose, kilo-code, metagpt, opencode, qwen-code, void
- **popularite cumulee** : 1,379,082 etoiles
- **adaptateurs proposes** : `omega-config (couches + precedence) + omega-registry (planned)`, `omega-plugin-api (Hook live + waterfall, planned)`, `omega-plugin-api (Hook live par point d'interception)`, `omega-plugin-api (HookBus — cible)`, `omega-plugin-api (Transform rejouable + Hook live + reload par domaine)`, `omega-plugin-api (dispatch de hooks a ecrire sur le port Hook)`, `omega-plugin-api (effets reversibles, planned)`, `omega-registry`, `omega-registry (EntryInfo id opaque + quarantine)`, `omega-registry (EntryInfo par capacite ; la porte reste un script CI)`, `omega-registry (EntryInfo porte le contrat public ; le reste reste hors registre)`, `omega-registry (Manifest versionne par capacite)`, `omega-registry (Scope : fermeture = revelation de la couche precedente)`, `omega-registry (Scope : fermeture = revelation de la couche precedente, planned)`, `omega-registry (Slot/Ensemble + quarantaine)`, `omega-registry (empreinte + reload, planned)`, `omega-registry (empreinte source + reload)`, `omega-registry (entrees declenchees utilisateur) + plugins/command-toml`, `omega-registry (manifeste + marketplace resolver + scopes + namespacing)`, `omega-registry (port EnvBackend nouveau, un plugin par environnement)`, `omega-registry (quarantaine) + omega-plugin-manager (planned)`, `omega-registry (replace/unregister)`, `omega-registry (resolve optionnel + isolation de fibre)`, `omega-registry (slots modeles)`, `omega-registry + omega-plugin-api (planned)`, `omega-registry::Manifest + omega-config (planned)`, `omega-registry::Registry (Slot/Ensemble + quarantaine)`, `omega-registry::Registry (Slot/Ensemble + quarantaine) + plugins/extension-installer`, `plugins/marketplace`
- **exemplaire de reference** : `dsh-desktop` — *Marketplace : contrats JSON Schema publics + types generes* (port `Registre (manifeste de source declare, validation a l'entree, catalogue normalise)`, adaptateur `omega-registry (Manifest versionne par capacite)`, cout M)
  - comment il le fait : `contracts/schemas.ts:21-38` compile `catalog-source`, `catalog-query`, `catalog-provider-page` et `catalog-snapshot` avec Ajv 2020 (`strict:true`, `allErrors:true`, `validateFormats:true`, `ajv-formats`), lus depuis `docs/schemas/*.schema.json` au demarrage ; les types TS correspondants sont generes dans `contracts/generated/`. Toute donnee distante est normalisee avant d'atteindre le client, et 
  - preuves : `dsh-community-market/src/contracts/schemas.ts:21-38`, `dsh-community-market/src/contracts/validate.ts`, `dsh-community-market/src/contracts/generated/catalog-source.ts`, `dsh-community-market/src/adapters/standard-http.ts`

#### `(extension)` — 3 harnais, cout M

- **harnais demandeurs** : continue, kilo-code, tabby
- **popularite cumulee** : 97,776 etoiles
- **adaptateurs proposes** : `omega-server (surface IDE externe, meme origine que le cockpit)`, `omega-server (surface IDE externe, protocole unique au lieu de messages dupliques)`, `omega-server (surface editeur, a evaluer)`
- **exemplaire de reference** : `kilo-code` — *Agent Client Protocol (editeurs tiers : Zed, JetBrains)* (port `—`, adaptateur `omega-server (surface editeur, a evaluer)`, cout M)
  - comment il le fait : packages/opencode/src/acp/service.ts:1-40 importe les types ACP (InitializeRequest/Response, NewSession*, LoadSession*, ResumeSession*, ForkSession*, Prompt*, CancelNotification, SetSessionModel*, SetSessionMode*, SetSessionConfigOption*, ListSessions*, CloseSession*) et module le comportement par acp/{agent,config-option,content,directory,error,event,permission,profile,session,tool,usage}.ts ; le
  - preuves : `packages/opencode/src/acp/service.ts:1-60`, `packages/opencode/src/acp/permission.ts`, `packages/opencode/src/acp/usage.ts`, `packages/opencode/src/cli/cmd/acp.ts`

#### `extension-host` — 2 harnais, cout M

- **harnais demandeurs** : goose, openhands
- **popularite cumulee** : 146,434 etoiles
- **adaptateurs proposes** : `plugins/extension-host`
- **exemplaire de reference** : `openhands` — *Plugins + hooks + extensions Canvas* (port `ExtensionHost (install/enable/hook)`, adaptateur `plugins/extension-host`, cout L)
  - comment il le fait : SDK : Plugin (manifeste, sources, fetch strict, installation) + HookManager/HookExecutor (evenements, decisions) ; serveur : routers plugins (fetch/install/enable/marketplace) et canvas_extensions (manifeste, backend manage, bridge, bundle) ; UI : chargement de modules d'extension.
  - preuves : `sdk/openhands-sdk/openhands/sdk/plugin/plugin.py:36`, `sdk/openhands-sdk/openhands/sdk/plugin/types.py:242`, `sdk/openhands-sdk/openhands/sdk/hooks/manager.py:22`, `sdk/openhands-sdk/openhands/sdk/hooks/types.py:9-35`

#### `(configuration)` — 1 harnais, cout S

- **harnais demandeurs** : crush
- **popularite cumulee** : 28,675 etoiles
- **exemplaire de reference** : `crush` — *Auto-vérification de version (GitHub Releases)* (port `—`, cout S)
  - comment il le fait : update/update.go:15 githubApiUrl, :20 Default client GitHub, :58 Check (comparatif Current/Latest) ; info IsDevelopment:33, Available:43 ; distribution via goreleaser (.goreleaser.yml).
  - preuves : `internal/update/update.go:15`, `internal/update/update.go:58`, `.goreleaser.yml`

#### `plugin-manifest` — 1 harnais, cout M

- **harnais demandeurs** : openclaw
- **adaptateurs proposes** : `plugins/plugin-manifest (serde + JSON Schema)`
- **exemplaire de reference** : `openclaw` — *Manifeste declaratif + policy-par-code* (port `PluginManifest (validate + inspect sans executer)`, adaptateur `plugins/plugin-manifest (serde + JSON Schema)`, cout M)
  - comment il le fait : Chaque plugin embarque openclaw.plugin.json : configSchema (JSON Schema), activation/setup/doctorContract lisibles sans executer le code du plugin, champ contracts (snapshot statique des capacites revendiquees), entree scellee par definePluginEntry ; l'activation-planner decide du chargement sur les seules metadonnees.
  - preuves : `RE-OPENCLAW-HERMES.md`, `src/plugins/manifest-types.ts:429,503-535`, `src/plugin-sdk/plugin-entry.ts:244-260`, `docs/plugins/manifest.md:48-60`

### Contexte & memoire de travail

#### `context-engine` — 28 harnais, cout S

- **harnais demandeurs** : agentgpt, aider, autogpt, claude-code, cline, codex, continue, crewai, crush, deepseek-harness, gemini-cli, goose, gpt-engineer, gpt-pilot, gptme, hermes, kilo-code, langgraph, metagpt, openclaw, opencode, openhands, plandex, qwen-code, tabby, void, windsurf, zed
- **popularite cumulee** : 1,773,172 etoiles
- **adaptateurs proposes** : `omega-context (CompactionModel derriere ContextBudget) + plugins/tool-output-masking`, `omega-context (ContextBudget estimate/compact + resume structure)`, `omega-context (ContextBudget estimate/compact + résumé structuré + réhydratation ciblée)`, `omega-context (a etendre : index vectoriel + recherche semantique)`, `omega-context::ContextBudget`, `omega-context::ContextBudget (a etendre : epoch + promotion d'input)`, `omega-context::ContextBudget + plugins/paginated-tools (Page{index,total,has_next})`, `omega-context::ContextContributor`, `omega-context::ContextContributor (DirectoryStr, budget partagé) + DefaultContextEngine`, `omega-context::ContextContributor (SystemInfo + OpenFiles + SelectionStaging)`, `omega-context::DefaultContextEngine`, `omega-context::DefaultContextEngine (fichiers hierarchiques) + nouveau MemoryStore (souvenirs)`, `omega-context::DefaultContextEngine (index + retrieval a ajouter)`, `omega-context::DefaultContextEngine (index de docs a ajouter)`, `omega-context::DefaultContextEngine (injection de metacompteurs de run)`, `omega-context::DefaultContextEngine (partiel)`, `omega-context::DefaultContextEngine (pipeline retrieval + rerank)`, `omega-context::DefaultContextEngine (planned)`, `omega-context::DefaultContextEngine (regles injectees au prompt)`, `omega-context::FastRetriever`, `omega-context::PromptCacheTelemetry`, `omega-context::TokenEstimator + ContextBudget`, `omega-context::compaction (deterministe existant, resume LLM a ajouter)`, `omega-context::contributor (registre de contributors de contexte)`, `omega-context::truncate_compact`, `omega-context::truncate_compact (partiel)`, `omega-context::{ContextBudget,truncate_compact}`, `plugins/agents-md-merge`, `plugins/context-autoload`, `plugins/context-compress`, `plugins/context-condenser`, `plugins/context-file-picker (selection de fichiers versionnable + .gitignore)`, `plugins/context-normalize (black + rustfmt, registre par extension)`, `plugins/context-per-step`, `plugins/convo-summarize`, `plugins/fim-autocomplete (PrefixSuffix + PredictionStrategy + PostProcess)`, `plugins/knowledge-sources (sources typees + index vectoriel)`, `plugins/lm-vector (port LongTermMemory nouveau, backend vectoriel local)`, `plugins/loop-guard`, `plugins/project-map`, `plugins/prompt-cache`, `plugins/prompt-cache-policy (breakpoints ≤ 4, ordre tools→system→messages, TTL 5m/1h, affinité parent/fork/self)`, `plugins/prompt-cache-policy (breakpoints<=4, TTL 5m/1h)`, `plugins/quick-edit (FimTags + ContextBudget::enforce)`, `plugins/rag-backend (port RetrievalBackend nouveau, un plugin par index)`, `plugins/repomap-pagerank`, `plugins/structured-output (schema derive + validation + revision partielle)`, `plugins/thread-from-summary`, `plugins/todos`
- **exemplaire de reference** : `gpt-engineer` — *Selection de fichiers par TOML commente, ouverte dans l'editeur* (port `ContextBudget (selection + budget de contexte)`, adaptateur `plugins/context-file-picker (selection de fichiers versionnable + .gitignore)`, cout M)
  - comment il le fait : applications/cli/file_selector.py : le fichier .gpteng/file_selection.toml est genere avec l'arbre complet ou chaque entree est commentee (COMMENT : 'Remove # to select a file or turn off linting'), puis ouvert dans l'editeur ($EDITOR ou gedit/notepad/nvim/write/nano/vim/emacs, :212-247) ; les lignes non commentees = fichiers selectionnes (:271-342). Filtrage : get_current_files (:379-416) saute l
  - preuves : `gpt_engineer/applications/cli/file_selector.py:53-64`, `gpt_engineer/applications/cli/file_selector.py:79-121`, `gpt_engineer/applications/cli/file_selector.py:123-210`, `gpt_engineer/applications/cli/file_selector.py:212-247`

#### `memory-store` — 12 harnais, cout S

- **harnais demandeurs** : claude-code, codex, crewai, devin, gemini-cli, gpt-engineer, gptme, hermes, kilo-code, langgraph, openclaw, windsurf
- **popularite cumulee** : 448,944 etoiles
- **adaptateurs proposes** : `omega-context::NamespaceStore (a creer : le port MemoryStore existe mais sans adaptateur namespace/index)`, `plugins/auto-memory (extraction batch + inbox de validation)`, `plugins/memory-bounded`, `plugins/memory-bounded (CLAUDE.md layers + MEMORY.md 25KB + audit)`, `plugins/memory-fs (workspace K/V sur disque ; port MemoryStore existe mais AUCUN adaptateur dans le dépôt)`, `plugins/memory-fts`, `plugins/memory-hierarchical (derriere MemoryStore)`, `plugins/memory-imports (resolution de references)`, `plugins/memory-layered`, `plugins/memory-local`, `plugins/memory-recall (profondeur adaptive)`, `plugins/memory-unified (analyse LLM + score composite + backend vectoriel)`
- **exemplaire de reference** : `langgraph` — *Store memoire hierarchique : namespaces, recherche, TTL, index vectoriel* (port `MemoryStore (recall/remember + namespaces)`, adaptateur `omega-context::NamespaceStore (a creer : le port MemoryStore existe mais sans adaptateur namespace/index)`, cout L)
  - comment il le fait : BaseStore (ABC) : get/put/delete/search/list_namespaces avec ops batchees (GetOp/SearchOp/PutOp/ListNamespacesOp), MatchCondition (prefix/suffix), InvalidNamespaceError, TTLConfig et IndexConfig {dims, embed, fields (JSON path)} ; InMemoryStore (dictionnaire + vector search optionnel) et un backend Postgres (langgraph.store.postgres) partagent le meme contrat ; injecte dans les nouds via le parame
  - preuves : `libs/checkpoint/langgraph/store/base/__init__.py:708`, `libs/checkpoint/langgraph/store/base/__init__.py:784`, `libs/checkpoint/langgraph/store/base/__init__.py:866`, `libs/checkpoint/langgraph/store/base/__init__.py:966`

#### `skill-store` — 9 harnais, cout S

- **harnais demandeurs** : continue, devin, dsh-web, goose, hermes, metagpt, open-interpreter, rea, windsurf
- **popularite cumulee** : 239,899 etoiles
- **adaptateurs proposes** : `omega-context::SkillIndex`, `omega-context::SkillIndex (etendu aux blocs de prompt reutilisables)`, `plugins/exp-pool (port ExperienceStore nouveau ; score + provenance via omega-otel::Provenance)`, `plugins/harness-skill-bridge`, `plugins/skill-evolve`
- **exemplaire de reference** : `devin` — *Skills SKILL.md + Knowledge a triggers (+ migration)* (port `SkillStore (discover/load)`, adaptateur `omega-context::SkillIndex`, cout S)
  - comment il le fait : Documente : SKILL.md au standard Agent Skills decouverts dans 6 chemins (.agents/.devin/.github/.claude/.cognition/.windsurf), sources indexees + disque (re-scan au clone), frontmatter (name, description, allowed-tools, argument-hint, triggers), invocation auto ou @skills:nom + substitution $ARGUMENTS/$1..$9, une seule skill active a la fois, suggestion auto de skills par Devin avec bouton Create 
  - preuves : `https://docs.devin.ai/product-guides/skills.md`, `https://docs.devin.ai/onboard-devin/knowledge-onboarding.md`, `https://docs.devin.ai/product-guides/plugins.md`, `https://agentskills.io/specification`

#### `(contexte)` — 2 harnais, cout M

- **harnais demandeurs** : deepseek-harness, tabby
- **popularite cumulee** : 285,164 etoiles
- **adaptateurs proposes** : `omega-context (planned)`
- **exemplaire de reference** : `deepseek-harness` — *Spill store + stockage d'attachements durables* (port `—`, adaptateur `omega-context (planned)`, cout M)
  - comment il le fait : ctx.spillStore est une couture de stockage de spill (SpillStore abstrait) avec spill-local comme backend et spill-policy comme politique de declenchement ; ctx.attachments est la couture de stockage binaire durable (AttachmentStore abstrait, attachment-local) : l'hote commit les images acceptees AVANT les evenements de session, et les adaptateurs de provider resolvent des references durables AUTOR
  - preuves : `packages/spill/spill/src/index.ts:45`, `packages/attachment/attachment/src/index.ts:53`, `docs/subsystems/spill.md`, `docs/subsystems/attachment.md`

#### `conversation-store` — 1 harnais, cout L

- **harnais demandeurs** : openhands
- **popularite cumulee** : 91,083 etoiles
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

#### `exec-policy` — 12 harnais, cout S

- **harnais demandeurs** : aider, autogpt, continue, crewai, crush, deepseek-harness, dsh-desktop, gemini-cli, gpt-pilot, openclaw, rea, void
- **popularite cumulee** : 814,082 etoiles
- **adaptateurs proposes** : `omega-sandbox::RuleBasedExecPolicy`, `omega-sandbox::RuleBasedExecPolicy (extension au niveau outil)`, `omega-sandbox::RuleBasedExecPolicy (mode plan = deny-by-default hors lecture)`, `omega-sandbox::RuleBasedExecPolicy (profil lecture seule)`, `omega-sandbox::RuleBasedExecPolicy + LocalSandboxBackend`, `omega-sandbox::RuleBasedExecPolicy + LocalSandboxBackend (regles deny sur chemins de secrets)`, `omega-sandbox::{RuleBasedExecPolicy,LocalSandboxBackend}`, `plugins/hooks-runner`, `plugins/lint-feedback (diagnostics -> tool_result)`, `plugins/reflect-lint-test`
- **exemplaire de reference** : `void` — *Boucle de retour lint apres ecriture* (port `ExecPolicy`, adaptateur `plugins/lint-feedback (diagnostics -> tool_result)`, cout M)
  - comment il le fait : Dans callTool.rewrite_file et callTool.edit_file, le retour n'est pas resolu immediatement : result: lintErrorsPromise avec un timeout(2000) puis _getLintErrors(uri) (toolsService.ts:413-445). _getLintErrors (574-588) lit IMarkerService, filtre Error | Warning, plafonne a 100 marqueurs et projette en { code, message, startLineNumber, endLineNumber }. Le texte remis au modele est conditionne par in
  - preuves : `src/vs/workbench/contrib/void/browser/toolsService.ts:391-395`, `src/vs/workbench/contrib/void/browser/toolsService.ts:413-445`, `src/vs/workbench/contrib/void/browser/toolsService.ts:469-474`, `src/vs/workbench/contrib/void/browser/toolsService.ts:515-532`

#### `(execution)` — 11 harnais, cout S

- **harnais demandeurs** : autogpt, continue, crewai, crush, deepseek-harness, dsh-web, gemini-cli, kilo-code, langgraph, opencode, qwen-code
- **popularite cumulee** : 993,736 etoiles
- **adaptateurs proposes** : `omega-app (CLI omega) + omega-server`, `omega-app (file de travail + verrou distribue)`, `omega-app (orchestrateur de graphe, a extraire du noyau)`, `omega-core (boucle de run + event store)`, `omega-core::AgentLoop + omega-app::App::run (a etendre en boucle outilouverte)`, `omega-protocol (contrats types du serveur, desja exposes en /api/*)`, `omega-sandbox::RuleBasedExecPolicy (garde deterministe de boucle)`, `omega-server (cockpit + /api/health|plugins|run|trace)`, `omega-server (endpoint agent + agent card)`, `omega-server (processus unique : API + cockpit)`, `omega-server (routes /api/run + /api/trace existantes)`, `omega-server + plugins/acp (JSON-RPC stdio sur les memes routes)`
- **exemplaire de reference** : `kilo-code` — *CLI unique : run headless, serve, attach, remote, pr, github, import, plug* (port `—`, adaptateur `omega-app (CLI omega) + omega-server`, cout M)
  - comment il le fait : packages/opencode/src/cli/cmd contient run.ts (+ run-stdin, run/stream.ts pour le transport), serve.ts, attach.ts, remote.ts, acp.ts, pr.ts (checkout/link/unlink/status sur un lien PR<->session), github.ts (install/run avec GitHub mock event + token), mcp.ts, plug.ts:1-40 (installPlugin/patchPluginConfig/readPluginManifest + resolvePluginTarget), agent.ts, session.ts, import.ts, export.ts, stats.t
  - preuves : `packages/opencode/src/cli/cmd/plug.ts:1-40`, `packages/opencode/src/cli/cmd/pr.ts:1-50`, `packages/opencode/src/cli/cmd/github.ts:1-40`, `packages/opencode/src/cli/cmd/serve.ts`

#### `sandbox-backend` — 11 harnais, cout M

- **harnais demandeurs** : claude-code, devin, dsh-desktop, gemini-cli, gptme, hermes, openhands, plandex, rea, void, windsurf
- **popularite cumulee** : 429,844 etoiles
- **adaptateurs proposes** : `omega-sandbox::LocalSandboxBackend`, `omega-sandbox::LocalSandboxBackend (a etendre : isolation namespace + FS)`, `omega-sandbox::LocalSandboxBackend + RuleBasedExecPolicy + ApprovalQueue (persistant a ajouter)`, `plugins/diff-sandbox`, `plugins/sandbox-docker`, `plugins/sandbox-docker-wasmtime`, `plugins/sandbox-rea-capture`, `plugins/sandbox-worktree (git worktree + vérification cwd/commande + cleanup)`, `plugins/sandbox-{docker,ssh,modal,…}`, `plugins/worktree-isolated`, `plugins/worktree-isolation (derriere SandboxBackend)`
- **exemplaire de reference** : `dsh-desktop` — *Fenetres locales durcies (sandbox, partition dediee, pas de popup ni de webview)* (port `SandboxBackend (fenetres et vues locales en sandbox, origine unique)`, adaptateur `omega-sandbox::LocalSandboxBackend`, cout M)
  - comment il le fait : `createDesktopLocalWindow()` impose `contextIsolation: true`, `nodeIntegration: false`, `nodeIntegrationInSubFrames: false`, `sandbox: true`, `webSecurity: true`, `webviewTag: false`, `spellcheck: false`, et un `partition` qui doit matcher `/^dsh-[a-z0-9-]+$/` (partition memoire dediee, sinon TypeError) ; il pose `setWindowOpenHandler(() => ({action:'deny'}))` et `will-attach-webview` → `preventDe
  - preuves : `dsh-plugin-desktop/src/local-window-policy.ts:19-46`, `dsh-plugin-desktop/src/desktop-network.ts:58-70`, `dsh-plugin-desktop/src/index.ts:430-445`, `dsh-plugin-desktop/src/electron-platform.ts`

#### `permission-policy` — 9 harnais, cout S

- **harnais demandeurs** : autogpt, deepseek-harness, devin, dsh-desktop, dsh-web, gemini-cli, goose, metagpt, tabby
- **popularite cumulee** : 746,393 etoiles
- **adaptateurs proposes** : `omega-sandbox::ApprovalQueue`, `omega-sandbox::ApprovalQueue (demandes d'humain) + omega-server cockpit`, `omega-sandbox::ApprovalQueue + ruleset de mode persiste`, `plugins/folder-trust + plugins/keychain (config-source/permission-policy)`
- **exemplaire de reference** : `dsh-desktop` — *Aucun pont IPC generique : canaux nommes ou routes loopback filtrees* (port `PermissionPolicy (controle d'origine sur chaque route privee)`, adaptateur `omega-sandbox::ApprovalQueue`, cout S)
  - comment il le fait : Chaque besoin a son module de preload : `preload-locale.ts`, `preload-material.ts`, `preload-menu.ts`, `preload-keyboard.ts`, `preload-browser.ts`, `preload-permissions.ts`, `preload-settings.ts`, `preload-theme.ts` — tous commentes « no generic IPC bridge » et n'exposent que des fonctions nommees via `contextBridge.exposeInMainWorld` (dsh-desktop-next/src/preload-app.ts:1-2). Cote Host, chaque ro
  - preuves : `dsh-plugin-desktop/src/index.ts:88-96`, `dsh-plugin-desktop/src/index.ts:238-262`, `dsh-plugin-desktop/src/desktop-settings-contract.ts:5-43`, `dsh-desktop-next/src/preload-app.ts:1-47`

#### `checkpoint-store` — 3 harnais, cout M

- **harnais demandeurs** : cursor, langgraph, windsurf
- **popularite cumulee** : 43,381 etoiles
- **adaptateurs proposes** : `omega-app::FileEventStore (lecture du journal) — ecriture `as_node` a ajouter`, `plugins/checkpoint-local`, `plugins/checkpoint-store`, `plugins/checkpoint-store (JSONL + index thread, sur le contrat EventStore existant)`
- **exemplaire de reference** : `langgraph` — *Checkpointing : BaseCheckpointSaver + backends memory/sqlite/postgres* (port `CheckpointStore (put/get_tuple/list)`, adaptateur `plugins/checkpoint-store (JSONL + index thread, sur le contrat EventStore existant)`, cout L)
  - comment il le fait : Contrat BaseCheckpointSaver : get_tuple/list/put/put_writes/delete_thread/delete_for_runs + serde interchangeable (JsonPlusSerializer par defaut) ; les ecritures pendantes sont stockees separement (put_writes) pour permettre la reprise apres crash au milieu d'un pas. Implementations : InMemorySaver (dictionnaire + PersistentDict optionnel), SqliteSaver (leger, synchrone), BasePostgresSaver (UPSERT
  - preuves : `libs/checkpoint/langgraph/checkpoint/base/__init__.py:199`, `libs/checkpoint/langgraph/checkpoint/base/__init__.py:262`, `libs/checkpoint/langgraph/checkpoint/base/__init__.py:277`, `libs/checkpoint/langgraph/checkpoint/base/__init__.py:301`

#### `worktree-isolate` — 1 harnais, cout L

- **harnais demandeurs** : zed
- **popularite cumulee** : 91,834 etoiles
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

#### `event-store` — 14 harnais, cout S

- **harnais demandeurs** : agentgpt, claude-code, crewai, crush, deepseek-harness, devin, dsh-desktop, dsh-web, gemini-cli, metagpt, open-interpreter, opencode, qwen-code, void
- **popularite cumulee** : 1,085,780 etoiles
- **adaptateurs proposes** : `omega-app::FileEventStore`, `omega-app::FileEventStore (rejeu conversation) + plugins/file-rewind (git ombre)`, `omega-app::FileEventStore + omega-otel::TraceReplayer`, `omega-app::FileEventStore + omega-otel::TraceReplayer (replay d'un saut)`, `omega-app::{File,InMemory}EventStore + omega-context`, `omega-otel::Provenance + omega-app::FileEventStore`, `omega-otel::TraceReplayer`, `omega-otel::TraceReplayer (rejeu depuis journal)`, `omega-registry + omega-app::EventStore (routage cause_by/source/destination)`, `plugins/artifacts-git (port ArtifactPublisher nouveau ; provenance omega-otel::Provenance)`, `plugins/channel-router`, `plugins/hook-bus (matcher + if + décision exit-code/JSON + async)`, `plugins/session-persistent`, `plugins/session-rollout`, `plugins/workspace-router`
- **exemplaire de reference** : `dsh-desktop` — *Trois points de controle rotatifs de configuration declarative* (port `EventStore (etat declaratif versionne et restaurable, hors journal de run)`, adaptateur `omega-app::FileEventStore`, cout M)
  - comment il le fait : « Exactly three rotating slots are retained. Restoring a slot never launches pnpm and never copies node_modules » ; apres restauration, le premier demarrage sain consomme un « skip marker » au lieu d'ecraser un point de controle (profile-checkpoint.ts:1-10). La liste des fichiers couverts est explicite et versionnee (`MANIFEST_VERSION = 4`, retrocompatible v3 et v2) : `package.json`, `pnpm-lock.ya
  - preuves : `dsh-plugin-desktop/src/profile-checkpoint.ts:1-10`, `dsh-plugin-desktop/src/profile-checkpoint.ts:41-70`, `dsh-plugin-desktop/src/profile-checkpoint.ts:28-40`, `dsh-plugin-desktop/src/recovery.ts`

#### `(ui)` — 9 harnais, cout M

- **harnais demandeurs** : agentgpt, autogpt, continue, crush, deepseek-harness, gemini-cli, gpt-engineer, gpt-pilot, tabby
- **popularite cumulee** : 770,059 etoiles
- **adaptateurs proposes** : `omega-cli (binaire TUI sur omega-app + omega-server)`, `omega-diff (script deterministe de diff/stream, jamais une tache d'agent)`, `omega-server (cockpit + /api/health|plugins|run|trace)`, `omega-server (cockpit React/TS servi sur la meme origine)`, `omega-server (surface web existante) + surface TUI terminal dediee`, `omega-server::main (cockpit React + /api/run sur 127.0.0.1:8787 -- meme role, pas les memes options)`, `service autocomplete (contexte fourni par omega-context, completions par omega-providers)`, `service next-edit (memes ports contexte/completion que l'autocompletion)`
- **exemplaire de reference** : `deepseek-harness` — *CLI dsh : profils nommes, dump de config/schema, audit de demarrage* (port `—`, cout M)
  - comment il le fait : Profils livres : web, headless, sdk, sdk-minimal, acp (plus un profil desktop reserve a Electron) ; dsh --profile <name> --dump-config imprime la liste d'entrees composee avec les expressions !!js verbatim, groupee par fichier source et par couche de patch ayant modifie chaque ligne, en UN document YAML chargeable ; --dump-config-schema importe chaque module compose pour lire son schema (donc a n'
  - preuves : `docs/cli-help.md:21-166`, `packages/boot/app-boot/README.md:36-43`, `packages/boot/app-boot/README.md:71-81`, `packages/boot/app-boot/README.md:98-111`

#### `telemetry-sink` — 9 harnais, cout S

- **harnais demandeurs** : aider, crewai, devin, dsh-desktop, gpt-pilot, metagpt, plandex, tabby, void
- **popularite cumulee** : 323,451 etoiles
- **adaptateurs proposes** : `omega-otel::StdoutSink`, `omega-otel::{Stdout,File}Sink`, `omega-otel::{Stdout,File}Sink + omega-otel::Provenance`, `omega-otel::{Stdout,File}Sink + omega-otel::Provenance (opt-out a ajouter)`, `omega-otel::{Stdout,File}Sink + plugins/budget-guard`, `plugins/eval-harness`
- **exemplaire de reference** : `void` — *Telemetrie PostHog avec opt-out et evenements de boucle* (port `TelemetrySink`, adaptateur `omega-otel::{Stdout,File}Sink + omega-otel::Provenance (opt-out a ajouter)`, cout S)
  - comment il le fait : MetricsMainService (electron-main/metricsMainService.ts, 162 l.) instancie PostHog('phc_UanIdujHiLp55BkUTjB1AuBXcasVkdqRwgnwRlWESH2', { host: 'https://us.i.posthog.com' }) — la cle de projet est codee en dur dans le binaire. L'opt-out est lu au demarrage : si OPT_OUT_KEY ('void.app.optOutAll', storageKeys.ts:22) est vrai alors client.optOut() sinon optIn() + identify() (127-140). Les identifiants 
  - preuves : `src/vs/workbench/contrib/void/electron-main/metricsMainService.ts:54-72`, `src/vs/workbench/contrib/void/electron-main/metricsMainService.ts:86-100`, `src/vs/workbench/contrib/void/electron-main/metricsMainService.ts:100-160`, `src/vs/workbench/contrib/void/common/storageKeys.ts:22`

#### `provenance` — 3 harnais, cout S

- **harnais demandeurs** : crewai, dsh-web, rea
- **popularite cumulee** : 68,614 etoiles
- **adaptateurs proposes** : `omega-otel::Provenance`, `plugins/tool-rea-evidence`
- **exemplaire de reference** : `rea` — *Contrat d'évidence canonique (preuves + limites)* (port `Provenance`, adaptateur `plugins/tool-rea-evidence`, cout M)
  - comment il le fait : Schéma zod evidenceSchema (superRefine) + parseEvidence/immutableEvidence ; text+structuredContent ; raw_result vs normalized_result ; budgets transport 10 MiB, export streaming, vues paginées inspect_analysis_view ; erreurs canoniques isError + error-contract.schema.json.
  - preuves : `src/domain/evidence.ts:37-291`, `docs/mcp-contracts.md:9-150`, `docs/error-contract.schema.json:1-60`

#### `(observabilite)` — 2 harnais, cout S

- **harnais demandeurs** : autogpt, goose
- **popularite cumulee** : 242,968 etoiles
- **adaptateurs proposes** : `omega-otel::Provenance + ledger de credits omega-app`, `omega-server (flux SSE/WS du cockpit + rejeu de trace)`
- **exemplaire de reference** : `autogpt` — *Cout declare par bloc, metrage post-vol et facturation en credits* (port `—`, adaptateur `omega-otel::Provenance + ledger de credits omega-app`, cout L)
  - comment il le fait : `BlockCost` porte `cost_amount`, `cost_filter`, `cost_type` (`BlockCostType.RUN|BYTE|SECOND|ITEMS|COST_USD|TOKENS`, les quatre derniers etant « dynamiques » : 0 avant vol, regles sur les stats apres), `cost_divisor` et une carte de taux `TokenRateDisplay` (USD/1M entree, sortie, cache read/write) ; l'executor appelle `charge_node_usage()` et `_record_execution_failure` distingue erreur courante et
  - preuves : `autogpt_platform/backend/backend/blocks/_base.py:122-205`, `autogpt_platform/backend/backend/executor/manager.py:1151-1200`, `autogpt_platform/backend/backend/executor/cost_tracking.py:1-80`, `autogpt_platform/backend/backend/executor/billing.py:1-60`

#### `ui-panel` — 2 harnais, cout S

- **harnais demandeurs** : langgraph, zed
- **popularite cumulee** : 135,215 etoiles
- **adaptateurs proposes** : `omega-ui (panneaux du cockpit, canal de messages a ajouter)`, `plugins/agent-panel`
- **exemplaire de reference** : `langgraph` — *Messages d'interface pousses depuis un noud (push_ui_message)* (port `UiPanel`, adaptateur `omega-ui (panneaux du cockpit, canal de messages a ajouter)`, cout S)
  - comment il le fait : push_ui_message(value, *, state_key="ui", id) retourne un dict-clair d'ecriture canal, delete_ui_message(id) rend un RemoveUIMessage (cle `__delete__` portant l'id) et ui_message_reducer fusionne les messages par id en appliquant les suppressions — le tout expose comme canal d'etat `ui` standard.
  - preuves : `libs/langgraph/langgraph/graph/ui.py:22`, `libs/langgraph/langgraph/graph/ui.py:43`, `libs/langgraph/langgraph/graph/ui.py:61`, `libs/langgraph/langgraph/graph/ui.py:133`

#### `ui-surface` — 2 harnais, cout S

- **harnais demandeurs** : dsh-desktop, open-interpreter
- **popularite cumulee** : 99,713 etoiles
- **adaptateurs proposes** : `plugins/cli-tui-surface`
- **exemplaire de reference** : `dsh-desktop` — *L'UI est un plugin de slots : racine proprietaire + occupants amont* (port `UiSurface (slots root/sidebar/main/rightbar/overlay + ancres DOM stables)`, cout M)
  - comment il le fait : `applyExtendedShell()` appelle `applyExtendedOwnedShell()` qui fait `ctx.slots.register({ name:'root', children:{ sidebar:{kind:'single'}, main:{kind:'keyed'}, rightbar:{kind:'single'}, 'shell.overlay':{kind:'list'} }, inject: () => ({ layout, platform }) }, ExtendedFrame)` (client/extended-shell.ts:19-49) ; le mode avance fait le meme enregistrement racine (client/advanced-shell.ts:43-44). `insta
  - preuves : `dsh-plugin-desktop/src/client/extended-shell.ts:19-49`, `dsh-plugin-desktop/src/client/extended-shell.ts:52-87`, `dsh-plugin-desktop/src/client/advanced-shell.ts:43-44`, `dsh-plugin-desktop/src/client/layout-service.ts:7-20`

#### `ui-shell` — 1 harnais, cout L

- **harnais demandeurs** : openhands
- **popularite cumulee** : 91,083 etoiles
- **adaptateurs proposes** : `plugins/canvas-ui`
- **exemplaire de reference** : `openhands` — *Canvas UI (chat, terminal, navigateur, fichiers)* (port `UiShell (views/actions)`, adaptateur `plugins/canvas-ui`, cout L)
  - comment il le fait : React 19 + Router 7 + Zustand + React Query : vues conversation/chat/terminal/navigateur/fichiers/reglages/automations ; adaptateur traduisant les actions UI en REST (toAppConversation, suffixes systeme runtime_services) ; entrypoints librairie (build:lib) ; mocks MSW ; client-tools agent->UI (canvas-ui, request-secret, automation-form).
  - preuves : `canvas/docs/architecture.md`, `canvas/src/api/agent-server-adapter.ts:202-414`, `canvas/src/api/conversation-service/agent-server-conversation-service.api.ts:431-1258`, `canvas/src/api/event-service/event-service.api.ts:157`

#### `ui-rendering` — 1 harnais, cout L

- **harnais demandeurs** : gpt-pilot
- **popularite cumulee** : 33,654 etoiles
- **exemplaire de reference** : `gpt-pilot` — *Agent Frontend : construction et iteration de l'interface* (port `UIRunner`, cout L)
  - comment il le fait : core/agents/frontend.py:40 Frontend, start_frontend (61), iterate_frontend (151), remove_mock (451) avec templates create_rag_query et remove_mock ; core/ui/console.py PlainConsoleUI ; actions FE_* dans core/config/actions.py.
  - preuves : `core/agents/frontend.py:40`, `core/agents/frontend.py:61`, `core/agents/frontend.py:151`, `core/agents/frontend.py:451`

### Orchestration

#### `subagent-runner` — 22 harnais, cout S

- **harnais demandeurs** : aider, autogpt, claude-code, cline, codex, continue, crewai, crush, cursor, deepseek-harness, devin, dsh-web, gemini-cli, gptme, hermes, kilo-code, langgraph, open-interpreter, openclaw, opencode, openhands, zed
- **popularite cumulee** : 1,492,452 etoiles
- **adaptateurs proposes** : `omega-app::ExpertRoster (a creer : roster + routines + credentials)`, `omega-app::SubagentRunner (a creer : spawn/resume + budget)`, `omega-ports::SubagentRunner (adaptateur a ecrire)`, `omega-ports::SubagentRunner (adaptateurs CLI externes, planned)`, `omega-ports::SubagentRunner + omega-core (planned)`, `omega-server (run non-interactif + SDK surface)`, `plugins/architect-loop`, `plugins/delegation-tools (outil pair vers SubagentRunner)`, `plugins/subagent-delegation (spawn/resume derriere SubagentRunner)`, `plugins/subagent-explore`, `plugins/subagent-graph (sous-graphe isole + namespace)`, `plugins/subagent-isolated`, `plugins/subagent-isolated (background + notify) + plugins/agent-team`, `plugins/subagent-isolated (spawn/resume, isolation worktree)`, `plugins/subagent-isolated (spawn/resume, isolation worktree, modèle/effort/TTL par invocation)`
- **exemplaire de reference** : `deepseek-harness` — *Couture subagent : un service, N providers nommes* (port `SubagentRunner (spawn/resume)`, adaptateur `omega-ports::SubagentRunner + omega-core (planned)`, cout L)
  - comment il le fait : SubagentRuntime est un registre de PROVIDERS NOMMES : chaque backend s'enregistre sous un nom unique et un tool de delegation choisit un provider ; SubagentManager possede demarrage, admission des messages, capacite, relations de parente et destruction ; startActivation ne rend la main qu'apres acceptation de l'enfant (la publication EST l'aboutissement, le handle peut annuler cette execution exac
  - preuves : `packages/subagent/subagent/src/index.ts:142-269`, `packages/subagent/subagent/src/depth.ts:1-23`, `packages/subagent/subagent/src/manager.ts`, `packages/subagent/subagent/README.md:51-73`

#### `agent-store` — 12 harnais, cout S

- **harnais demandeurs** : agentgpt, autogpt, claude-code, crewai, crush, deepseek-harness, gpt-pilot, kilo-code, metagpt, opencode, qwen-code, tabby
- **popularite cumulee** : 1,123,880 etoiles
- **adaptateurs proposes** : `omega-core (planned)`, `omega-core::agent-loop (planned)`, `omega-registry (slots_role : profil/but/contraintes/prefixe par plugin)`, `plugins/agent-autonome (function calling)`, `plugins/agent-autonome (objectif → tâches → exécution → synthèse)`, `plugins/agent-declarative`, `plugins/agent-declarative (frontmatter YAML + Ruleset)`, `plugins/agent-identity (schema Agent + system prompt compose)`, `plugins/agent-preset (planned)`, `plugins/command-md`, `plugins/command-md (template + $args + argument-hint + overrides)`, `plugins/lite-agent (run simple + garde contexte)`, `plugins/plan-mode`, `plugins/plan-mode (deny Write/Edit + délégation Plan + rappel)`, `plugins/plan-mode (planned)`, `plugins/reasoning-strategies (a creer)`, `plugins/run-modes (mode de run declarae par plugin)`, `plugins/sop-pipeline (chaine de plugins par etape + Ruleset par role)`, `plugins/subagent-isolated (frontmatter + spawn/reprise)`
- **exemplaire de reference** : `kilo-code` — *Agents declaratifs (markdown) et modes d'organisation* (port `AgentStore (discover/load/save)`, adaptateur `plugins/agent-declarative`, cout S)
  - comment il le fait : AgentBuilder (opencode/src/kilocode/agent/builder.ts:1-60) definit Scope=global|project, Mode=primary|subagent|all, ID regex, Body{scope, description, mode, model, color, steps, tools[], permission, prompt} + Input/SaveInput avec preview/save en markdown ; les fichiers vivent dans .kilo/agent/*.md (system-prompt.ts:29 recommande explicitement .kilo/ et interdit .kilocode//.opencode/ pour les nouve
  - preuves : `packages/opencode/src/kilocode/agent/builder.ts:1-60`, `packages/kilo-gateway/src/api/modes.ts:1-80`, `packages/opencode/src/kilocode/modes-migrator.ts:9-130`, `packages/opencode/src/kilocode/config/config.ts:435-450`

#### `scheduler-job` — 9 harnais, cout M

- **harnais demandeurs** : aider, claude-code, cline, devin, gptme, hermes, kilo-code, openclaw, openhands
- **popularite cumulee** : 394,449 etoiles
- **adaptateurs proposes** : `plugins/file-watcher`, `plugins/scheduler-cron`, `plugins/scheduler-cron (local + cloud-runner + skill verify/simplify pre-commit)`, `plugins/scheduler-cron + plugins/channel-*`, `plugins/scheduler-cron + plugins/channel-dispatch`, `plugins/scheduler-service`, `plugins/scheduler-wakeup`
- **exemplaire de reference** : `openhands` — *Automations planifiees + webhooks (backend dedie)* (port `Scheduler (schedule/trigger)`, adaptateur `plugins/scheduler-cron`, cout M)
  - comment il le fait : Backend dedie openhands-automation (paquet PyPI versionne en sync avec le front) servant /api/automation (dont webhooks) ; le front tague les conversations (automationtrigger/id/name/run_id) et propage le contexte d'observabilite ; stack locale via uvx + ingress.
  - preuves : `canvas/src/api/automation-service/automation-service.api.ts:60-737`, `canvas/src/api/agent-server-adapter.ts:558-572`, `sdk/openhands-sdk/openhands/sdk/automation/__init__.py`, `canvas/scripts/check-sdk-version-sync.mjs:63-82`

#### `hook` — 4 harnais, cout M

- **harnais demandeurs** : cline, codex, deepseek-harness, gemini-cli
- **popularite cumulee** : 429,089 etoiles
- **adaptateurs proposes** : `omega-plugin-api (Hook) + omega-sandbox (planned)`, `omega-plugin-api (adaptateur Hook : declencheur + commande)`, `plugins/hook-command`, `plugins/hook-runner`
- **exemplaire de reference** : `codex` — *Hooks declaratifs types et abortables* (port `Hook`, adaptateur `plugins/hook-command`, cout M)
  - comment il le fait : HookResult { Success, FailedContinue(err), FailedAbort(err) } avec should_abort_operation() ; Hook = { name, HookFn } ; HookPayload { session_id, cwd, triggered_at, hook_event } ; 12 HookEventName (PreToolUse, PermissionRequest, PostToolUse, Pre/PostCompact, SessionStart/End, UserPromptSubmit, SubagentStart/Stop, Stop, Interrupt), 4 HookHandlerType (Command, McpTool, Prompt, Agent), modes Sync/Asy
  - preuves : `codex-rs/hooks/src/types.rs:15`, `codex-rs/hooks/src/types.rs:27`, `codex-rs/hooks/src/types.rs:39`, `codex-rs/hooks/src/types.rs:64`

#### `agent-server` — 4 harnais, cout M

- **harnais demandeurs** : goose, langgraph, openhands, zed
- **popularite cumulee** : 281,649 etoiles
- **adaptateurs proposes** : `omega-server (client HTTP interne du cockpit)`, `plugins/acp-bridge`, `plugins/agent-server-rest`
- **exemplaire de reference** : `langgraph` — *RemoteGraph : client d'un deploiement LangGraph distant* (port `RemoteGraph (stream/invoke)`, adaptateur `omega-server (client HTTP interne du cockpit)`, cout M)
  - comment il le fait : RemoteGraph(PregelProtocol) wrappe un LangGraphClient (SDK) : stream/astream (nettoyage config, merge des en-tetes de tracing via _merge_tracing_headers), invoke/ainvoke, get_state/aget_state (conversion ThreadState -> StateSnapshot via _create_state_snapshot), get_state_history, bulk_update_state ; les appels distants rejettent les versions de stream non supportees (_reject_v3_unsupported).
  - preuves : `libs/langgraph/langgraph/pregel/remote.py:118`, `libs/langgraph/langgraph/pregel/remote.py:322`, `libs/langgraph/langgraph/pregel/remote.py:724`, `libs/sdk-py/langgraph_sdk/schema.py:335`

#### `channel` — 4 harnais, cout M

- **harnais demandeurs** : cline, gemini-cli, langgraph, openclaw
- **popularite cumulee** : 221,222 etoiles
- **adaptateurs proposes** : `plugins/channel-*`, `plugins/voice-capture (provider de transcription derriere un port)`, `plugins/workflow-graph::channels (BinaryOperatorAggregate + Topic)`
- **exemplaire de reference** : `langgraph` — *Canaux d'etat versionnes + reducers (merge deterministe)* (port `GraphChannel (get/update/checkpoint)`, adaptateur `plugins/workflow-graph::channels (BinaryOperatorAggregate + Topic)`, cout M)
  - comment il le fait : BaseChannel (get/update/consume/checkpoint/from_checkpoint) avec implémentations LastValue, EphemeralValue, BinaryOperatorAggregate (reducer applicatif, detecte l'overwrite via le type Overwrite, egalite des fonctions par identite), Topic (aplatissement + accumulate optionnel), NamedBarrierValue (attente de toutes les sources) et AnyValue ; add_messages (graph/message.py) est le reducer canonique 
  - preuves : `libs/langgraph/langgraph/channels/base.py:70`, `libs/langgraph/langgraph/channels/base.py:90`, `libs/langgraph/langgraph/channels/binop.py:66`, `libs/langgraph/langgraph/channels/topic.py:23`

#### `agent-loop` — 4 harnais, cout S

- **harnais demandeurs** : gpt-engineer, gptme, langgraph, plandex
- **popularite cumulee** : 118,609 etoiles
- **adaptateurs proposes** : `plugins/agent-loop`, `plugins/agent-loop (boucle plan/act sur le routage Omega)`, `plugins/agent-loop (pipeline d'etapes explicite, sans boucle cachee)`, `plugins/agent-tell-build`, `plugins/workflow-fn (entrypoint/task durables)`
- **exemplaire de reference** : `gpt-engineer` — *Pipeline d'etapes LLM injectables (pas de boucle ReAct)* (port `AgentLoop (pipeline d'etapes)`, adaptateur `plugins/agent-loop (pipeline d'etapes explicite, sans boucle cachee)`, cout M)
  - comment il le fait : CliAgent prend code_gen_fn (gen_code), improve_fn et process_code_fn (execute_entrypoint) en parametres, typees par TypeVar (CodeGenType/CodeProcessor/ImproveType) ; init() enchaine code_gen_fn -> gen_entrypoint -> process_code_fn, improve() n'appelle que improve_fn (entreepoint/execution volontairement commentes) ; chaque etape separee par un prompt systeme distinct (roadmap+generate+philosophie 
  - preuves : `gpt_engineer/applications/cli/cli_agent.py:27-33`, `gpt_engineer/applications/cli/cli_agent.py:84-100`, `gpt_engineer/applications/cli/cli_agent.py:152-183`, `gpt_engineer/applications/cli/cli_agent.py:185-232`

#### `hook-bus` — 4 harnais, cout S

- **harnais demandeurs** : cursor, gptme, openclaw, windsurf
- **popularite cumulee** : 4,454 etoiles
- **adaptateurs proposes** : `plugins/hook-bus`, `plugins/hook-bus (bus async + budgets + ordre deterministe)`, `plugins/hooks`
- **exemplaire de reference** : `gptme` — *Bus de hooks typés (~20 types, sync+async)* (port `HookBus (register/trigger)`, adaptateur `plugins/hook-bus`, cout M)
  - comment il le fait : HookType enumere ~20 types en notation pointee (step.pre/post, tool.execute.pre/post, session.start/end, generation.*, loop.continue, tool.confirm, elicit...) (gptme/hooks/types.py:66-140) ; HookRegistry.register/trigger avec filtrage de kwargs, StopPropagation et hooks async (gptme/hooks/registry.py:109-329) ; ~25 modules de hooks concrets (confirm, guardrails, token/cost/cache awareness...) ; le
  - preuves : `gptme/hooks/types.py:66-140`, `gptme/hooks/registry.py:109-329`, `gptme/hooks/registry.py:434-607`, `gptme/plugins/registry.py:144-165`

#### `agent-runtime` — 3 harnais, cout S

- **harnais demandeurs** : cline, goose, void
- **popularite cumulee** : 154,532 etoiles
- **adaptateurs proposes** : `omega-app::App::run (pipeline Operation) + plugins/agent-loop (StepBudget)`, `plugins/agent-loop`
- **exemplaire de reference** : `void` — *Boucle agent outillee (sendLLM -> validate -> exec -> feedback)* (port `AgentRuntime (step/finish/abort)`, adaptateur `omega-app::App::run (pipeline Operation) + plugins/agent-loop (StepBudget)`, cout L)
  - comment il le fait : _runChatAgent (chatThreadService.ts:732-911). Details observes : retry d'erreur LLM borne a CHAT_RETRIES=3 avec RETRY_DELAY=2500ms (chatThreadService.ts:45-46, 791-875) ; reentree dans la boucle seulement si le toolCall n'a pas demande d'approbation (894-895) ; PAS de plafond d'iterations (seule garde : l'approbation humaine ou l'interruption) ; l'historique est reconstruit a chaque tour via prepa
  - preuves : `src/vs/workbench/contrib/void/browser/chatThreadService.ts:45-46`, `src/vs/workbench/contrib/void/browser/chatThreadService.ts:557-590`, `src/vs/workbench/contrib/void/browser/chatThreadService.ts:732-911`, `src/vs/workbench/contrib/void/browser/chatThreadService.ts:879-898`

#### `(agents)` — 2 harnais, cout M

- **harnais demandeurs** : gemini-cli, qwen-code
- **popularite cumulee** : 135,942 etoiles
- **adaptateurs proposes** : `omega-core (goal reducer + goal store)`, `omega-server + plugins/a2a (client + serveur)`
- **exemplaire de reference** : `gemini-cli` — *Agents distants A2A + serveur A2A autonome* (port `—`, adaptateur `omega-server + plugins/a2a (client + serveur)`, cout L)
  - comment il le fait : A2AClientManager (agents/a2a-client-manager.ts:52) charge les agents distants, cache les clients et stream les messages (sendMessageStream :224) avec proxy HTTP(S) gere ; le paquet packages/a2a-server expose un binaire separe (bin gemini-cli-a2a-server) avec http/app.ts + http/server.ts, agent/executor.ts et persistence/ (taches durables).
  - preuves : `packages/core/src/agents/a2a-client-manager.ts:52`, `packages/a2a-server/src/http/server.ts`, `packages/a2a-server/src/agent/executor.ts`, `packages/a2a-server/package.json (bin gemini-cli-a2a-server)`

#### `agent-runner` — 2 harnais, cout M

- **harnais demandeurs** : cursor, zed
- **popularite cumulee** : 91,834 etoiles
- **adaptateurs proposes** : `plugins/agent-loop`
- **exemplaire de reference** : `zed` — *Boucle agent run_turn streaming annulable* (port `AgentRunner (run_turn/cancel)`, adaptateur `plugins/agent-loop`, cout M)
  - comment il le fait : Thread::run_turn (thread.rs:2843) vide le message partiel, annule le tour en vol, fige le resume, capture les outils actives puis delegue a run_turn_internal (thread.rs:2921) qui streame la completion ; RunningTurn (thread.rs:5090) porte prompt_id, event_stream, outils et canal d'annulation watch. Les evenements (arret/refus/max-tokens/erreur) sont emis via ThreadEventStream.
  - preuves : `crates/agent/src/thread.rs:2843-2920`, `crates/agent/src/thread.rs:5090-5130`

#### `agent-host` — 1 harnais, cout M

- **harnais demandeurs** : openhands
- **popularite cumulee** : 91,083 etoiles
- **adaptateurs proposes** : `plugins/agent-acp`
- **exemplaire de reference** : `openhands` — *Agents tiers via ACP (Claude Code, Codex, Gemini, Pi, OpenCode)* (port `AgentHost (spawn/relay)`, adaptateur `plugins/agent-acp`, cout M)
  - comment il le fait : L'Agent Server engendre la CLI de l'agent tiers en sous-processus et relaie chaque tour en JSON-RPC sur stdio ; registre ACP_PROVIDERS (commandes npx, cles, logins OAuth detectes) ; decouverte de modeles via acp_router ; choix stocke par backend (agent_kind, acp_*).
  - preuves : `sdk/openhands-sdk/openhands/sdk/agent/acp_agent.py:504-637`, `sdk/openhands-sdk/openhands/sdk/settings/acp_providers.py:176-552`, `sdk/openhands-agent-server/openhands/agent_server/acp_router.py:150-175`, `canvas/docs/ACP_AGENTS.md`

#### `headless-run` — 1 harnais, cout M

- **harnais demandeurs** : cline
- **popularite cumulee** : 70,429 etoiles
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
- **popularite cumulee** : 49,588 etoiles
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

#### `tool-surface` — 20 harnais, cout S

- **harnais demandeurs** : agentgpt, aider, autogpt, claude-code, continue, crewai, crush, cursor, deepseek-harness, devin, gemini-cli, kilo-code, langgraph, metagpt, openclaw, openhands, qwen-code, rea, void, windsurf
- **popularite cumulee** : 1,198,361 etoiles
- **adaptateurs proposes** : `nouveau runtime omega-sandbox::QuickJsBackend (rquickjs)`, `omega-context::ToolSurfaceIndex`, `omega-context::ToolSurfaceIndex (enregistrement de plugins de recherche)`, `omega-context::ToolSurfaceIndex (hydrate un sous-ensemble par turn) + omega-router (parsing)`, `omega-context::ToolSurfaceIndex (index existant) + execution a ajouter via ToolExecutor`, `omega-context::ToolSurfaceIndex + ContextContributor (filtre par effet de bord)`, `omega-context::ToolSurfaceIndex + boucle d'outils du noyau`, `omega-mcp (a etendre : oauth/resources/prompts + McpServerSpec -> serveur)`, `omega-mcp (client MCP)`, `omega-mcp (client MCP, crate existant a etendre)`, `omega-mcp (client, cible)`, `omega-mcp (extension : transports multiples + OAuth)`, `omega-sandbox::LocalSandboxBackend + plugins/shell-pty`, `plugins/agent-cmdloop (parsing de commandes + carte de tool_execution + post-check)`, `plugins/agent-tools`, `plugins/browser-cdp (service Playwright hors noyau)`, `plugins/lsp-bridge`, `plugins/mcp-bridge`, `plugins/mcp-client (stdio/http/sse/ws + trust + elicitation)`, `plugins/mcp-rea`, `plugins/question`, `plugins/skill-activation (outil derriere ToolSurface)`, `plugins/skill-rea`, `plugins/slash-commands`, `plugins/tool-contract (validate/call/stringify par outil)`, `plugins/tool-devin-workspace`, `plugins/tool-registry`, `plugins/web-grounding (derriere ToolSurface)`, `plugins/xml-tool-grammar (parseur streaming tolerant + prompt)`
- **exemplaire de reference** : `void` — *Grammaire XML de tool calling pour modeles sans tools natifs* (port `ToolSurface (index/hydrate)`, adaptateur `plugins/xml-tool-grammar (parseur streaming tolerant + prompt)`, cout M)
  - comment il le fait : extractGrammar.ts : findPartiallyWrittenToolTagAtEnd (146) detecte une balise ouvertesur en cours (indispensable en streaming), findIndexOfAny (156), parseXMLPrefixToToolCall (168-262) valide que seuls les parametres declares sont retenus (206 allowedParams), extractXMLToolsWrapper (263-364) branche onText/onFinalMessage et reconnait l'identifiant d'ouverture pour attribuer le bon nom d'outil. Le 
  - preuves : `src/vs/workbench/contrib/void/electron-main/llmMessage/extractGrammar.ts:146-364`, `src/vs/workbench/contrib/void/common/prompt/prompts.ts:380-423`, `src/vs/workbench/contrib/void/browser/convertToLLMMessageService.ts:199-240`, `src/vs/workbench/contrib/void/common/modelCapabilities.ts:170`

#### `tool` — 7 harnais, cout S

- **harnais demandeurs** : cline, deepseek-harness, gptme, hermes, openhands, plandex, rea
- **popularite cumulee** : 432,964 etoiles
- **adaptateurs proposes** : `omega-sandbox + omega-plugin-api (planned)`, `plugins/structured-edits`, `plugins/tool-rea-android`, `plugins/tool-rea-captures`, `plugins/tool-rea-crash`, `plugins/tool-rea-dotnet`, `plugins/tool-rea-evm`, `plugins/tool-rea-firmware`, `plugins/tool-rea-functions`, `plugins/tool-rea-js`, `plugins/tool-rea-native`, `plugins/tool-rea-sqlite`, `plugins/tool-rea-wasm`, `plugins/tool-rpc`, `plugins/tool-sdk`, `plugins/tool-shell`, `plugins/tools-exec`
- **exemplaire de reference** : `rea` — *Routeur multi-providers natifs (Hopper/Ghidra/IDA/Rizin/objdump/Cutter/GDB)* (port `ToolExecutor`, adaptateur `plugins/tool-rea-native`, cout L)
  - comment il le fait : AnalysisProviderRegistry + SessionProviderRouter : une liaison immuable par cible, sélection déterministe sans fallback ; bridge Python Hopper (bridge/hopper_bridge.py), bundle Java Ghidra (bridge/ghidra/), adaptation du MCP upstream ida-pro-mcp, profils Rizin/objdump/GDB, Cutter. Sessions persistantes limitées à la vie du process MCP.
  - preuves : `src/ghidra/GhidraProvider.ts:1-40`, `src/hopper/HopperProvider.ts:1-40`, `src/ida/IdaProvider.ts:39-60`, `src/objdump/ObjdumpCommand.ts:1-40`

#### `mcp-bridge` — 3 harnais, cout M

- **harnais demandeurs** : cline, crewai, zed
- **popularite cumulee** : 222,041 etoiles
- **adaptateurs proposes** : `plugins/mcp-bridge`, `plugins/mcp-client (transports stdio/http/sse + filtres)`
- **exemplaire de reference** : `crewai` — *Integrations MCP (stdio/HTTP/SSE) + filtres d'outils* (port `McpBridge (connect/list/call)`, adaptateur `plugins/mcp-client (transports stdio/http/sse + filtres)`, cout M)
  - comment il le fait : MCPServerStdio (mcp/config.py:12), MCPServerHTTP (l.53), MCPServerSSE (l.90), chacun avec cache_tools_list (l.47,84,117) ; transports stdio/sse/http (mcp/client.py:39-41) ; ToolFilterContext + filtres (mcp/filters.py:17) ; outil natif MCPTool (tools/mcp_native_tool.py:13) ; tool_resolver.py resout les outils en BaseTool.
  - preuves : `lib/crewai/src/crewai/mcp/config.py:12-117`, `lib/crewai/src/crewai/mcp/client.py:39-41`, `lib/crewai/src/crewai/mcp/filters.py:17`, `lib/crewai/src/crewai/tools/mcp_native_tool.py:13`

#### `mcp-client` — 2 harnais, cout M

- **harnais demandeurs** : cursor, gptme
- **popularite cumulee** : 4,454 etoiles
- **adaptateurs proposes** : `plugins/mcp-client`
- **exemplaire de reference** : `gptme` — *Client MCP (stdio+HTTP, tools/resources/prompts)* (port `McpClient (connect/call/list)`, adaptateur `plugins/mcp-client`, cout L)
  - comment il le fait : MCPClient gere les connexions stdio + HTTP avec callbacks roots et elicitation (gptme/mcp/client.py:60-310) ; expose call_tool/list_resources/read_resource/list_prompts/get_prompt et gestion dynamique des roots (gptme/mcp/client.py:351-593) ; create_mcp_tools() convertit les outils MCP distants en ToolSpec locaux avec retry (gptme/tools/mcp_adapter.py:290-523) ; MCPRegistry cherche dans les regist
  - preuves : `gptme/mcp/client.py:60-310`, `gptme/mcp/client.py:351-593`, `gptme/tools/mcp_adapter.py:290-523`, `gptme/mcp/registry.py:50-214`

#### `mention-resolver` — 1 harnais, cout M

- **harnais demandeurs** : zed
- **popularite cumulee** : 91,834 etoiles
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
- **popularite cumulee** : 4,454 etoiles
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

#### `llm-provider` — 6 harnais, cout S

- **harnais demandeurs** : continue, crewai, opencode, openhands, tabby, void
- **popularite cumulee** : 464,162 etoiles
- **adaptateurs proposes** : `omega-providers (catalogue par transforms + scoring communautaire)`, `omega-providers::OllamaProvider`, `omega-providers::OllamaProvider (catalogue multi-fournisseurs a etendre)`, `plugins/llm-registry`
- **exemplaire de reference** : `openhands` — *Registre LLM multi-providers + fallback + device-login* (port `Provider (complete/stream)`, adaptateur `plugins/llm-registry`, cout M)
  - comment il le fait : LLMRegistry (evenements de registre) + modele LLM pydantic avec mixins retry/outils non-natifs + FallbackStrategy ; router list_models/list_verified/device-login OAuth + passerelle OpenAI-compatible (chat/completions, responses) ; suivi de solde cote UI.
  - preuves : `sdk/openhands-sdk/openhands/sdk/llm/llm_registry.py:23`, `sdk/openhands-sdk/openhands/sdk/llm/llm.py:252`, `sdk/openhands-sdk/openhands/sdk/llm/fallback_strategy.py:45`, `sdk/openhands-agent-server/openhands/agent_server/llm_router.py:116-266`

#### `router-policy` — 5 harnais, cout S

- **harnais demandeurs** : autogpt, cursor, open-interpreter, plandex, windsurf
- **popularite cumulee** : 271,956 etoiles
- **adaptateurs proposes** : `omega-router::ArenaSelector`, `omega-router::CompositeScoringPolicy + TierFallbackPolicy`, `plugins/harness-autoselect`, `plugins/harness-router`, `plugins/model-router`, `plugins/router-large-ctx`
- **exemplaire de reference** : `autogpt` — *Routage mode x tier avec chaine de repli et kill-switch registre* (port `ScoringPolicy/Selector/FallbackPolicy`, adaptateur `omega-router::CompositeScoringPolicy + TierFallbackPolicy`, cout M)
  - comment il le fait : `model_router.resolve_model_route()` resout dans l'ordre : flag LaunchDarkly JSON `copilot-model-routing` (par utilisateur, retournant des slugs), cellule du catalogue (`catalog.py` section `routing`, surface `"copilot"`), puis defaut statique `ChatConfig` (env) ; sur le cloud, le registre sert de porte : un slug inconnu ou `is_enabled=False` est refuse bruyamment (log + Sentry) et la resolution r
  - preuves : `autogpt_platform/backend/backend/copilot/model_router.py:1-60`, `autogpt_platform/backend/backend/copilot/provider_tiers.py:1-40`, `autogpt_platform/backend/backend/copilot/engine.py:1-40`, `autogpt_platform/backend/backend/data/llm_registry`

#### `task-classifier` — 4 harnais, cout S

- **harnais demandeurs** : gemini-cli, metagpt, plandex, qwen-code
- **popularite cumulee** : 222,581 etoiles
- **adaptateurs proposes** : `nouveau adapter LLM-classifier au port TaskClassifier`, `omega-router::HeuristicTaskClassifier`, `omega-router::HeuristicTaskClassifier (variante heuristique existante)`, `omega-router::HeuristicTaskClassifier + plugins/quick-path`, `plugins/planner-review + omega-context (contexte structure compact)`
- **exemplaire de reference** : `gemini-cli` — *Routage par classificateur local (Gemma) + seuil de confiance* (port `TaskClassifier`, adaptateur `omega-router::HeuristicTaskClassifier (variante heuristique existante)`, cout M)
  - comment il le fait : gemmaClassifierStrategy.ts (routing/strategies/) appelle le classificateur local (core/localLiteRtLmClient.ts) avec le seuil lu depuis la config (modelRouterService.ts:83) ; une strategie numerique (numericalClassifierStrategy.ts) sert de repli sans modele ; docs/core/local-model-routing.md. Omega : HeuristicTaskClassifier est deterministe (regles), pas entraine.
  - preuves : `packages/core/src/routing/strategies/gemmaClassifierStrategy.ts`, `packages/core/src/routing/strategies/numericalClassifierStrategy.ts`, `packages/core/src/core/localLiteRtLmClient.ts`, `omega/crates/omega-router/src/classifier.rs:106`

## 2bis. Le noyau — ce que le corpus impose a l'architecture de plugins

Capacites qui touchent au **squelette** (registre, coutures, couches, effets, politiques, sous-agents). C'est la partie ou une erreur de conception se paie dans tout le systeme : a traiter **en premier**, avant les fonctionnalites.

| Capacite | Harnais | Cout | Exemplaire de reference | Detail a copier |
|---|---|---|---|---|
| `subagent-runner` | 22 | S | `deepseek-harness` — *Couture subagent : un service, N providers nommes` | SubagentRuntime est un registre de PROVIDERS NOMMES : chaque backend s'enregistre sous un nom unique et un tool de delegation choisit un provider ; Su |
| `tool-surface` | 20 | S | `void` — *Grammaire XML de tool calling pour modeles sans tools natifs` | extractGrammar.ts : findPartiallyWrittenToolTagAtEnd (146) detecte une balise ouvertesur en cours (indispensable en streaming), findIndexOfAny (156),  |
| `config-source` | 19 | S | `dsh-desktop` — *Canaux de release isoles (stable/beta) verifies par en-tetes` | `upstream.json` enregistre pour chaque canal le commit amont, la version source, la version du runtime empaquete et le manifeste vendorise ; `activeCh |
| `registry` | 16 | S | `dsh-desktop` — *Marketplace : contrats JSON Schema publics + types generes` | `contracts/schemas.ts:21-38` compile `catalog-source`, `catalog-query`, `catalog-provider-page` et `catalog-snapshot` avec Ajv 2020 (`strict:true`, `a |
| `event-store` | 14 | S | `dsh-desktop` — *Trois points de controle rotatifs de configuration declarative` | « Exactly three rotating slots are retained. Restoring a slot never launches pnpm and never copies node_modules » ; apres restauration, le premier dem |
| `agent-store` | 12 | S | `kilo-code` — *Agents declaratifs (markdown) et modes d'organisation` | AgentBuilder (opencode/src/kilocode/agent/builder.ts:1-60) definit Scope=global/project, Mode=primary/subagent/all, ID regex, Body{scope, description, |
| `exec-policy` | 12 | S | `void` — *Boucle de retour lint apres ecriture` | Dans callTool.rewrite_file et callTool.edit_file, le retour n'est pas resolu immediatement : result: lintErrorsPromise avec un timeout(2000) puis _get |
| `sandbox-backend` | 11 | M | `dsh-desktop` — *Fenetres locales durcies (sandbox, partition dediee, pas de popup ni de webview)` | `createDesktopLocalWindow()` impose `contextIsolation: true`, `nodeIntegration: false`, `nodeIntegrationInSubFrames: false`, `sandbox: true`, `webSecu |
| `permission-policy` | 9 | S | `dsh-desktop` — *Aucun pont IPC generique : canaux nommes ou routes loopback filtrees` | Chaque besoin a son module de preload : `preload-locale.ts`, `preload-material.ts`, `preload-menu.ts`, `preload-keyboard.ts`, `preload-browser.ts`, `p |
| `tool` | 7 | S | `rea` — *Routeur multi-providers natifs (Hopper/Ghidra/IDA/Rizin/objdump/Cutter/GDB)` | AnalysisProviderRegistry + SessionProviderRouter : une liaison immuable par cible, sélection déterministe sans fallback ; bridge Python Hopper (bridge |
| `hook` | 4 | M | `codex` — *Hooks declaratifs types et abortables` | HookResult { Success, FailedContinue(err), FailedAbort(err) } avec should_abort_operation() ; Hook = { name, HookFn } ; HookPayload { session_id, cwd, |
| `hook-bus` | 4 | S | `gptme` — *Bus de hooks typés (~20 types, sync+async)` | HookType enumere ~20 types en notation pointee (step.pre/post, tool.execute.pre/post, session.start/end, generation.*, loop.continue, tool.confirm, el |
| `agent-runtime` | 3 | S | `void` — *Boucle agent outillee (sendLLM -> validate -> exec -> feedback)` | _runChatAgent (chatThreadService.ts:732-911). Details observes : retry d'erreur LLM borne a CHAT_RETRIES=3 avec RETRY_DELAY=2500ms (chatThreadService. |
| `(extension)` | 3 | M | `kilo-code` — *Agent Client Protocol (editeurs tiers : Zed, JetBrains)` | packages/opencode/src/acp/service.ts:1-40 importe les types ACP (InitializeRequest/Response, NewSession*, LoadSession*, ResumeSession*, ForkSession*,  |
| `provenance` | 3 | S | `rea` — *Contrat d'évidence canonique (preuves + limites)` | Schéma zod evidenceSchema (superRefine) + parseEvidence/immutableEvidence ; text+structuredContent ; raw_result vs normalized_result ; budgets transpo |
| `extension-host` | 2 | M | `openhands` — *Plugins + hooks + extensions Canvas` | SDK : Plugin (manifeste, sources, fetch strict, installation) + HookManager/HookExecutor (evenements, decisions) ; serveur : routers plugins (fetch/in |
| `lifecycle-hook` | 1 | M | `open-interpreter` — *Hooks deterministes du cycle de vie agent` | 10 phases : session_start, user_prompt_submit, pre_tool_use, post_tool_use, permission_request, stop, session_end, compact, interrupt (+ AfterAgent ty |
| `tool-source` | 1 | M | `open-interpreter` — *Serveurs MCP stdio/HTTP geres en CLI` | Serveurs MCP stdio (command/args/env) ou HTTP (url + bearer_token_env_var), geres par interpreter mcp {add,list,get,remove,login,logout} ; connection_ |
| `plugin-manifest` | 1 | M | `openclaw` — *Manifeste declaratif + policy-par-code` | Chaque plugin embarque openclaw.plugin.json : configSchema (JSON Schema), activation/setup/doctorContract lisibles sans executer le code du plugin, ch |

### Regles de conception deduites (appliquees a toutes les capacites ci-dessus)

- **Effets reversibles** : toute inscription (registre, hook, timer, watcher) retourne un disposer ; rien ne fuit au unload.
- **Couches nommees** : toute valeur de configuration vient d'une couche identifiable et est tracable jusqu'a sa source.
- **Reload atomique** : un plugin se recharge entier ; les ecritures de config sont serialisees avec les reloads.
- **Guard monotone** : les politiques ne peuvent que restreindre, jamais elargir (pas de variante permissive).
- **Fail-closed** : a chaque couture, l'absence de canal, de provider ou de decision = refus, jamais execution.
- **Couture explicite** : chaque sous-systeme interchangeable est relie a une cle stable ; echanger le provider deplace tout le sous-systeme d'un coup.
- **Matrice d'evenements verifiee** : producteurs et consommateurs declares et controles automatiquement (pas d'evenement orphelin).

## 3. Feuille de route par phase

### Phase 1 — socle (45 capacites)

- `context-engine` — 28 harnais · 1,773,172 etoiles
- `subagent-runner` — 22 harnais · 1,492,452 etoiles
- `tool-surface` — 20 harnais · 1,198,361 etoiles
- `config-source` — 19 harnais · 1,007,547 etoiles
- `registry` — 16 harnais · 1,379,082 etoiles
- `event-store` — 14 harnais · 1,085,780 etoiles
- `agent-store` — 12 harnais · 1,123,880 etoiles
- `exec-policy` — 12 harnais · 814,082 etoiles
- `memory-store` — 12 harnais · 448,944 etoiles
- `(execution)` — 11 harnais · 993,736 etoiles
- `permission-policy` — 9 harnais · 746,393 etoiles
- `telemetry-sink` — 9 harnais · 323,451 etoiles
- `skill-store` — 9 harnais · 239,899 etoiles
- `tool` — 7 harnais · 432,964 etoiles
- `llm-provider` — 6 harnais · 464,162 etoiles
- `router-policy` — 5 harnais · 271,956 etoiles
- `task-classifier` — 4 harnais · 222,581 etoiles
- `agent-loop` — 4 harnais · 118,609 etoiles
- `hook-bus` — 4 harnais · 4,454 etoiles
- `agent-runtime` — 3 harnais · 154,532 etoiles
- `workflow-engine` — 3 harnais · 103,159 etoiles
- `provenance` — 3 harnais · 68,614 etoiles
- `(observabilite)` — 2 harnais · 242,968 etoiles
- `ui-panel` — 2 harnais · 135,215 etoiles
- `ui-surface` — 2 harnais · 99,713 etoiles
- `credentials` — 1 harnais · 251,248 etoiles
- `harness-emulation` — 1 harnais · 68,589 etoiles
- `orchestrator` — 1 harnais · 59,778 etoiles
- `clarify-loop` — 1 harnais · 55,024 etoiles
- `code-emitter` — 1 harnais · 55,024 etoiles
- `vcs-snapshot` — 1 harnais · 55,024 etoiles
- `agent-mode` — 1 harnais · 49,588 etoiles
- `edit-apply` — 1 harnais · 49,588 etoiles
- `lint-check` — 1 harnais · 49,588 etoiles
- `vcs-commit` — 1 harnais · 49,588 etoiles
- `file-system` — 1 harnais · 33,654 etoiles
- `vcs` — 1 harnais · 33,654 etoiles
- `(configuration)` — 1 harnais · 28,675 etoiles
- `cost-accounting` — 1 harnais · 27,626 etoiles
- `cost-guard` — 1 harnais · 4,454 etoiles
- `agent-control` — 1 harnais
- `agent-modes` — 1 harnais
- `harness-catalog` — 1 harnais
- `playbook-store` — 1 harnais
- `workflow-runner` — 1 harnais

### Phase 2 — structure (45 capacites)

- `sandbox-backend` — 11 harnais · 429,844 etoiles
- `(ui)` — 9 harnais · 770,059 etoiles
- `scheduler-job` — 9 harnais · 394,449 etoiles
- `(automatisation)` — 7 harnais · 636,277 etoiles
- `hook` — 4 harnais · 429,089 etoiles
- `agent-server` — 4 harnais · 281,649 etoiles
- `channel` — 4 harnais · 221,222 etoiles
- `mcp-bridge` — 3 harnais · 222,041 etoiles
- `(extension)` — 3 harnais · 97,776 etoiles
- `checkpoint-store` — 3 harnais · 43,381 etoiles
- `(contexte)` — 2 harnais · 285,164 etoiles
- `extension-host` — 2 harnais · 146,434 etoiles
- `(agents)` — 2 harnais · 135,942 etoiles
- `agent-runner` — 2 harnais · 91,834 etoiles
- `mcp-client` — 2 harnais · 4,454 etoiles
- `loop-guard` — 1 harnais · 107,412 etoiles
- `mention-resolver` — 1 harnais · 91,834 etoiles
- `agent-host` — 1 harnais · 91,083 etoiles
- `headless-run` — 1 harnais · 70,429 etoiles
- `agent-protocol` — 1 harnais · 68,589 etoiles
- `lifecycle-hook` — 1 harnais · 68,589 etoiles
- `tool-source` — 1 harnais · 68,589 etoiles
- `output-guardrail` — 1 harnais · 59,778 etoiles
- `planner` — 1 harnais · 59,778 etoiles
- `patch-apply` — 1 harnais · 55,024 etoiles
- `self-healing` — 1 harnais · 55,024 etoiles
- `retry-policy` — 1 harnais · 43,381 etoiles
- `stream-surface` — 1 harnais · 43,381 etoiles
- `review-loop` — 1 harnais · 33,654 etoiles
- `completion-provider` — 1 harnais · 27,626 etoiles
- `session-board` — 1 harnais · 27,626 etoiles
- `snapshot-store` — 1 harnais · 27,626 etoiles
- `plan-store` — 1 harnais · 15,750 etoiles
- `mcp-server` — 1 harnais · 4,454 etoiles
- `app-server` — 1 harnais
- `channel-connector` — 1 harnais
- `code-index` — 1 harnais
- `code-search` — 1 harnais
- `control-ui` — 1 harnais
- `inline-complete` — 1 harnais
- `memory-search` — 1 harnais
- `plugin-manifest` — 1 harnais
- `task-handoff` — 1 harnais
- `task-planner` — 1 harnais
- `worktree-manager` — 1 harnais

### Phase 3 — lourd (16 capacites)

- `(memoire)` — 1 harnais · 187,617 etoiles
- `worktree-isolate` — 1 harnais · 91,834 etoiles
- `conversation-store` — 1 harnais · 91,083 etoiles
- `ui-shell` — 1 harnais · 91,083 etoiles
- `eval-harness` — 1 harnais · 55,024 etoiles
- `patch-validate` — 1 harnais · 55,024 etoiles
- `hitl-gate` — 1 harnais · 43,381 etoiles
- `debug-session` — 1 harnais · 33,654 etoiles
- `persistence` — 1 harnais · 33,654 etoiles
- `task-execution` — 1 harnais · 33,654 etoiles
- `ui-rendering` — 1 harnais · 33,654 etoiles
- `apply-engine` — 1 harnais · 28,752 etoiles
- `workspace-manager` — 1 harnais · 27,626 etoiles
- `acp-agent` — 1 harnais · 4,454 etoiles
- `cloud-runner` — 1 harnais
- `code-review` — 1 harnais

## 4. Non-objectifs (ecartes, avec raison)

| Harnais | Fonctionnalite | Pourquoi ecartee |
|---|---|---|
| `claude-code` | Mods JS in-process (écarté : runtime JS) | ECARTE : documenté (https://code.claude.com/docs/en/plugins/mods/overview, liens depuis https://code.claude.com/docs/en/hooks observés par webfetch) : les mods sont des hooks JS in-process pouvant des |
| `cline` | Cline Hub cloud (SaaS operateur) | Hub SaaS proprietaire (sdk/packages/core/src/hub, cloud/, remote/) : authentification WorkOS device-auth, sessions cloud distantes (CloudSessionController), facturation/quotas, connecteurs geres par d |
| `continue` | Continue Hub distant (registre owner/package cloud) — ECARTE | Le schema de blocs subsiste mais la resolution distante a ete retiree du depot OSS : RegistryClient.getContent jette "Slug-based package resolution is not supported" (packages/config-yaml/src/registry |
| `crewai` | Adaptateurs LangGraph / OpenAI Agents (ecarte : dependances Python externes) | BaseAgentAdapter (agents/agent_adapters/base_agent_adapter.py:15), LangGraphAgentAdapter (langgraph/langgraph_adapter.py:40) et OpenAIAgentAdapter (openai_agents/openai_adapter.py:51) plus convertisse |
| `crush` | Exécution du crushrc comme Bash (REFUSÉ) | load.go:981 loadFromConfigPaths exécute le shell config via le moteur shellconfig ; README:256-259 et :322-327 déclarent crushrc et crush.json « trusted code » (tout $(...) de crush.json s'exécute au  |
| `cursor` | Telemetrie cloud proprietaire | [documente] Télémétrie d'usage vers les serveurs Anysphere avec Privacy Mode / retention configurable (entreprise) ; [infere] pipeline closed-source, facturation a l'usage indexee dessus. Non repris : |
| `deepseek-harness` | PTC (programmatic tool calling) via run_code — ECARTE (runtime JS/Python) | ECARTE pour le noyau Rust : en mode ptc le registre n'emet plus qu'un transport reserve (run_code) et les autres capacites visibles sont declarees dans une section SDK generee dans le langage du runti |
| `dsh-desktop` | ECARTE : patcher les versions amont pour etendre le noyau | `patches/` contient `dsh@0.2.1-alpha.5.patch`, `dsh-app-boot@…`, `dsh-client-modules@…`, `dsh-client-ui-conversation@…`, `dsh-client-ui-directory-picker-browse@…` etc., un par version pinnee — le depo |
| `dsh-web` | Dsh Market Client Categories | categories.ts : liste des categories avec id, label, icon. filter.ts : filter entries par category, search, installed status. MarketCard.tsx : card avec nom, description, version, boutons. install-sou |
| `dsh-web` | Dsh Plugin Manager Console Output | createOutputCapture(maxBytes) : accumulateur byte, overflow drop front (tail wins), read() decode une fois. decodeConsoleBytes() : TextDecoder utf-8 fatal, si echec essaie gbk puis utf-8 lossy, choisi |
| `dsh-web` | Dsh Plugin Manager Detached Work | AsyncResource au module scope (triggerAsyncId: 0) = pas de store herite. runDetached(work) = DETACHED.runInAsyncScope(work). setImmediate herite du store vide (pas du caller). Utilise par setEnabledHa |
| `goose` | App desktop Electron | Ecarte : ui/desktop est une app Electron (forge.config.ts, electron-builder) separee du workspace Rust, avec son cycle de release propre. Omega vise cockpit React servi par omega-server sur une seule  |
| `gpt-engineer` | Telemetrie RudderStack avec consentement et revue humaine (ECARTE) | applications/cli/learning.py : le consentement est demande une fois et persiste dans le fichier .gpte_consent ('true' = opto, suppression du fichier = retrait) (:183-234) ; human_review_input (:122-17 |
| `kilo-code` | Passerelle Kilo (compte, solde, modes d'organisation, sessions cloud) | ECARTE parce que c'est un backend SaaS operateur : auth/device-auth.ts:30-98 (device code + polling + etats approved/denied/expired), provider.ts:1-78 (createKilo enveloppe createOpenRouter avec base  |
| `metagpt` | Tree-of-Thought (BFS/DFS, refinement, evaluation par valeurs) | ThoughtSolverBase : generate_thoughts puis evaluate_node (valeur numerique par LLM) puis select_nodes selon Strategy (BFS/DFS/sample) ; BFSSolver construit l'arbre en largeur avec ThoughtSolverConfig  |
| `open-interpreter` | Computer-use GUI : absent (constate) | Inventaire exhaustif de codex-rs/ (~127 crates) : aucun crate d'automatisation GUI (pas de controle souris/clavier, capture assiste, ni protocole d'accessibilite) et aucune section computer-use dans d |
| `opencode` | Code Mode + LSP/formatter (ecarte : runtime JS) | LSP/formatter declaratifs en config (command + extensions) : adaptateurs lancant des serveurs externes ; Code Mode : le modele ecrit du code appelant les outils, schemas JSON rendus en types TypeScrip |
| `plandex` | Facturation cloud et RBAC SaaS | Facturation cloud (cmd/billing.go, crédits, rachats auto), invitations et RBAC org (rbac.go, invite_helpers.go) n'ont de sens que dans l'offre SaaS Plandex ; non transférables à un harnais self-hosted |
| `qwen-code` | Stack live multimodale temps reel + versant desktop/agents physiques | packages/live-host/src/preload/audio-engine.ts, packages/browser-use, packages/cua-driver ; preuve REA : docs/omega/research/rea/evidence/intake-qwen-code.summary.json status 'code: resource_constrain |
| `void` | Blacklist d'extensions IA concurrentes + transfert depuis VS Code | extensionBlacklist (extensionTransferService.ts:30-42) liste ms-vscode-remote.remote, ms-vscode.remote et les copilotes sourcegraph.cody-ai, continue.continue, codeium.codeium, saoudrizwan.claude-dev  |
| `void` | Fork amont de VS Code (Void n'est pas une extension) | Le README assume le fork et documente les deux adaptations de build (compilation React, scope Tailwind) ; VOID_CODEBASE_GUIDE.md explique que 'Void is no longer an extension' ; void.contribution.ts im |
| `windsurf` | Catalogue de modèles à crédits propriétaires | [documenté] Catalogue de dizaines de modèles (SWE, Claude, GPT, Gemini, Grok, ...) chacun avec coûts entrée/sortie/cache et multiplicateur de crédits (0.25x à 320x) ; routage Adaptive (sélection auto  |

## 5. Provenance

- dissections : `omega/crates/omega-harness/data/apps/*.json` (schema `DISSECTION-SCHEMA.md`)
- popularite : `omega/crates/omega-harness/data/harnesses_census.json` (API GitHub)
- preuves machine : `docs/omega/research/rea/evidence/` (REA, avec limites declarees)
- ce document ne declare « adopte » que ce qui existe reellement dans le code

