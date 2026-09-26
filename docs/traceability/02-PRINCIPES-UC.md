# Traçabilité PRINCIPES & USE CASES → Code réel

*Mesuré le 2026-09-26 (après activation du Palier 0) · branche `feat/inventaire-global-v1`*

Document **vérifiable** reliant les **22 principes SFD v3.1** (P1→P22, `docs/master-ref/01-SFD-v3.1.md` §3) et les **19 cas d'usage** (UC-01→UC-19, §4) au code réellement présent.

> **Méthode.** Analyse statique en lecture seule : `grep` ciblé dans `src/`, `routes/`, `core/`, `archive/`, `services/`, `mcp_servers/`, `.opencode/`, `packages/` ; vérification d'existence des fichiers ; lecture du kill-switch dans `src/killswitch_registry.py` et de sa valeur effective dans `.env` ; comptage des `tests/test_*.py` dont le **nom** matche un mot-clé. Contrôle des routes via `app.openapi()["paths"]` — les routes sont enregistrées par des wrappers paresseux invisibles dans `app.routes`.
>
> **Statuts.**
> - `ACTIF` = présent, sur le chemin d'exécution **par défaut**, sans kill-switch OFF sur ce chemin, et le résultat agit réellement.
> - `PARTIEL` = présent mais non branché au flux live, **ou branché mais inopérant** (résultat seulement loggé, ignoré, ou calcul constant), ou activé mais dont la dépendance externe manque.
> - `DORMANT` = présent mais fermé par défaut.
> - `ABSENT` = non trouvé.
>
> Le point de vérité est le code. (L'ancienne checklist `08-VERIFICATION-ETAT-ACTUEL.md` a été **retirée le 2026-09-25** — périmée ; ce document la remplace.)

> ### ⚠️ Ce que cette révision a changé, et pourquoi c'est une mauvaise et une bonne nouvelle
>
> Ces statuts ont été **re-vérifiés intégralement** après l'activation du Palier 0 (14 kill-switchs basculés de `off` à `on` par défaut dans le code) et après la remise au vert de la suite de tests (4 792 PASS). Les statuts précédents dataient d'une époque où la suite comptait **109 faux échecs** : ils mesuraient un code que rien ne vérifiait.
>
> **Le Palier 0 a fait son travail : plus aucun principe DORMANT (4 → 0).** Les 14 modules câblés le sont pour de vrai, et le cockpit expose enfin leur état.
>
> **Mais l'activation a aussi révélé que « codé » ne veut pas dire « actif »**, et le score a **baissé** là où on l'attendait pas :
> - **ACTIF 12 → 10.** UC-01 perd son statut : sans `OPENCODE_API_KEY` (le projet utilise OpenCode Zen et la clé n'est pas dans `.env`), `model_endpoints` est vide et l'agent n'a aucun endpoint. UC-05 le perd aussi : `on_round_start` **écrase en `BUILD`** la phase posée par l'API avant qu'elle ne soit appliquée. P7 et P11 sont déclassés (structure cosmétique, « complexifier sur preuve » sans mécanisme).
> - **DORMANT 5 → 1.** Les 4 principes DORMANT sont devenus PARTIEL, pas ACTIF : leur switch est `on`, mais le module est appelé avec une entrée vide ou son résultat est seulement loggé. C'est la distinction que l'ancien document ne faisait pas.
>
> Autrement dit : le Palier 0 a démasqué du travail réellement inachevé. C'est le but.

---

## 1. PRINCIPES P1 → P22

| # | Principe | Fichiers (vérifiés) | Kill-switch (défaut) | Tests | Statut | Preuve |
|---|---|---|---|---|---|---|
| **P1** | Le risque modifie la boucle | `src/risk_classifier.py`, `src/orchestrator/gate.py`, `src/tool_execution.py`, `config/phase-lock.yaml` | `DESTRUCTIVE_GATE=on` ; `PHASE_TRACKER=on` (Palier 0) | 7 | **PARTIEL** | `classify_tool` exécuté par outil (`tool_execution.py:573`) ; gate ON (`gate.py:26`) et appelé (`:588-596`) ; phase-lock désormais **vivante** (`phase_tracker.py:25` → poussée `agent_loop.py:2934` → appliquée `tool_execution.py:625-630`). **Mais** l'inférence ne produit que `PLAN`/`BUILD` (`phase_tracker.py:52-54`) et `BUILD` ne bloque rien (`phase-lock.yaml:39`) : le risque ne change ni la phase, ni le modèle, ni l'approbation. Le gate risque→humain de LangGraph (`:802-833`) reste **inatteignable** (`ODYSSEUS_LANGGRAPH` lu `""` puis `off`, absent de `.env`). |
| **P2** | Un brouillon n'est pas un commit | `routes/editor_draft_routes.py`, `archive/legacy/agent_loop.py`, `opencode.json` | — | 2 | **PARTIEL** | Dénylist du mode plan (`agent_loop.py:2241` → `tool_security.py:156-178`) ; plan approuvé épinglé au prompt (`build_active_plan_note`, `:2650-2655`) ; 5 endpoints brouillon **sans** commit/apply (`editor_draft_routes.py:81-165+`) ; écriture mémoire directe (`memory_routes.py:85-118`). |
| **P3** | Contexte construit, pas déversé | `src/context_budget.py`, `src/context_compactor.py`, `src/model_context.py` | — | 8 | **ACTIF** | Budget et trim calculés (`agent_loop.py:2700-2702`), journalisés (`:2704-2710`), **réassignation effective** `messages = trimmed_messages` (`:2711`). *La version précédente citait `:2664-2666` — un `try:` d'import de `context_budget`.* |
| **P4** | Budgets obligatoires par projet | `src/budget_enforcer.py`, `src/project_manifest.py`, `src/governance.py` | — | 5 | **PARTIEL** | `BudgetEnforcer` conditionné au manifeste (`load_manifest(".")` puis `agent_loop.py:2762-2765`, court-circuit `:2873`) et `PROJECT.yaml` **absent** (seul `PROJECT.yaml.example`). Un budget de contexte par défaut s'applique, mais pas de budget *projet* obligatoire. |
| **P5** | Divulgation progressive | `src/progressive_disclosure.py`, `archive/legacy/agent_loop.py` | `PROGRESSIVE_DISCLOSURE=on` (Palier 0) | **28** (`test_progressive_disclosure.py`) | **PARTIEL** | Switch `on` des deux côtés ; bloc **exécuté** (la phase résolue est enfin lue, `agent_loop.py:4404-4419`). **Mais** le bloc ne fait que **logger** : `allowed_tools` / `allowed_tool_count` ne sont jamais injectés dans `disabled_tools` ni dans les schémas envoyés au modèle. Le module n'avait **aucun test** — c'est pourquoi un `NameError` l'a laissé mort sans que rien ne le remarque. |
| **P6** | Échecs répétés → fonctionnalités du harnais | `src/orchestrator/autoeval.py`, `autoevolve.py`, `codeburn_runner.py`, `src/observer.py` | `AUTOEVAL=on`, `AUTOEVOLVE=on`, `CHECKPOINT=on` (Palier 0) ; `CODEBURN=off` ; `AUTOEVAL_ALLOW_RESET=off` | 5 | **PARTIEL** | Observer/dérive réel par run (`agent_loop.py:4154-4164`) ; décision AUTOEVAL tracée en live (`apply_autoeval`, `:4201-4204`) ; CHECKPOINT appelé (`record_checkpoint`, `:4121-4126`). **Mais** la recherche autoevolve n'est **jamais réinjectée** dans la boucle (donc aucune règle auto-apprise) ; `CODEBURN=off` ; et surtout l'**action** reste dormante par défaut — la dérive est détectée et la décision tracée, jamais exécutée. |
| **P7** | Structure > autonomie | `src/opencode_engine.py`, `src/orchestrator/phases.py` | `LIVE_ORCHESTRATION=off` (CanonicalLoop Python) | 3 | **PARTIEL** ⬇ | `walk()` est bien appelé par défaut (`chat_routes.py:1413` → `opencode_engine.py:251-325`). **Mais seule `BUILD` streame l'agent** (`:291-318`) ; les 6 autres n'émettent qu'une chaîne SSE `phase_active` (`:319-322`). `PHASE_TOOLS`/`PHASE_AGENTS`/`PHASE_MODELS` (`:104-240`) ne contraignent **jamais** `agent_stream_fn()`. La structure est cosmétique. |
| **P8** | Le plan passe les mêmes portes | `src/planning_engine.py`, `archive/legacy/agent_loop.py`, `routes/phase_routes.py` | `PLANNING_ENGINE=off` (code) | 2 | **PARTIEL** | Mode plan réel (`agent_loop.py:2252`), `PLAN` bloque `bash` (`phase-lock.yaml:32-33`). **Mais** le moteur de planification structuré est `off` (`planning_engine.py:27`), absent du registre et du `.env` ; `approved_plan` n'est qu'une note de prompt injectée en message system (`:2650-2655`). |
| **P9** | Évaluer le harnais, pas le modèle | `src/observer.py`, `src/trace_writer.py`, `src/sse_indicators.py` | `LANGFUSE=off` (sous-partie) | 3 | **ACTIF** | Drift par run (`agent_loop.py:4151-4154`) ; `run_status` émis à chaque tour (`:2982`) et en fin de run (`:4161`) ; `/api/observer/drift` (`observer_routes.py:17-18`) ; trace écrite pour **chaque** appel outil (`tool_execution.py:665-683`). `perf_profiler.py` n'a toujours aucun appelant ; `LANGFUSE` reste `off`. |
| **P10** | Humain ON the loop | `src/tool_index.py`, `archive/legacy/agent_loop.py`, `routes/chat_routes.py` | — | 2 | **ACTIF** | `ask_user` toujours disponible (`tool_index.py:42`, implémentation `agent_loop.py:3778-3980`) ; Stop/Resume du run (`chat_routes.py:1466`/`:1455`). **Réserve** : aucun gate humain n'est lié au risque — `ask_user` part à l'initiative du modèle. |
| **P11** | Commencer simple, complexifier sur preuve | `archive/legacy/agent_loop.py`, `src/orchestrator/agent_dispatcher.py` | 12 `AGENT_*=off`, `LIVE_ORCHESTRATION=off` | 4 | **PARTIEL** ⬇ | La moitié « simple » est réelle : agent unique par défaut, `agents_for_phase` → `[]` (`agent_dispatcher.py:118-121`). La moitié « sur preuve » est **absente du chemin d'exécution** : `get_decision_engine()` (`src/decision_engine.py:749`) a 0 appelant, et `MultiAgentWorkflow` exige `LIVE_ORCHESTRATION=on` (OFF). |
| **P12** | Découpage par contexte, pas par métier | `src/orchestrator/agent_dispatcher.py`, `multi_agent.py`, `phases.py` | 13 switches agents `off` | 9 | **PARTIEL** | `dispatch_for_phase` (`agent_dispatcher.py:123`) a **0 appelant en production** ; seul `dispatch_explicit` est utilisé (`core/route_loader.py:394-398`). |
| **P13** | Contexte = budget d'attention | `src/context_compactor.py`, `src/provenance_memory.py` | `PROVENANCE_MEMORY=on` (Palier 0) | 8 | **ACTIF** ⬆ | Compaction **appliquée** (`messages = trimmed_messages`, `agent_loop.py:2711`). Mémoire incrémentale vivante (`agent_loop.py:4294` → `provenance_memory.py:366-381`) ; preuve empirique : `data/memory-fs/profile.md`. **Réserve** : seule la branche `[stated]` est vive (cf. P16). |
| **P14** | Survivre à une panne | `src/durable_execution.py`, `src/orchestrator/checkpoint_tracker.py` | `DURABLE_EXECUTION=on`, `CHECKPOINT=on` (Palier 0) | 0 | **PARTIEL** ⬆ | Switches `on` et `wired`. **Mais** `DurableExecutor.create_workflow` (`durable_execution.py:113`) n'a **aucun appelant** et `data/workflows/` est **vide** ⇒ `resume()` (`agent_loop.py:4315`) renvoie toujours `None` — le garde `if _wf and ...` (`:4316`) n'est donc jamais franchi. Le checkpoint échoue en plus car `_ensure_vault()` ne trouve pas `/obsidian-vault` (`obsidian_mcp.py:12-18`) et `checkpoint_tracker.py:120` saute l'écriture. La reprise après crash reste **inexistante**. |
| **P15** | On n'observe pas ce qu'on ne trace pas | `src/trace_writer.py`, `src/event_bus.py`, `src/opencode_engine.py` | `UNIFIED_TOKENS=on` (Palier 0) | 8 | **ACTIF** | `UNIFIED_TOKENS` défaut `"on"` et `wired=True` ; publication effective (`agent_loop.py:4050-4057`, `run_id` posé à `:4045`) ; **réconciliation** réelle (`routes/chat_helpers.py:783-785`). |
| **P16** | Provenance explicite | `src/provenance_memory.py`, `src/memory_writer.py` | `PROVENANCE_MEMORY=on` (Palier 0) | 1 | **PARTIEL** ⬆ | `update_profile` (`agent_loop.py:4300` → `provenance_memory.py:366-381`) écrit réellement des entrées `[stated]`. **Mais** `add_observed` (`:4302`) renvoie `(False, "Domain file does not exist")` (`provenance_memory.py:344-346`) car `topics/agent-output.md` n'est **jamais créé** ⇒ **no-op garanti**. `memory_writer.py` : toujours 0 appelant. 1 tag sur 3 a un producteur vivant. |
| **P17** | Ne jamais stocker le sensible | `src/data_classification.py`, `src/content_security.py`, `routes/memory_routes.py` | `DATA_CLASSIFICATION=on`, `CONTENT_SECURITY=on` (Palier 0) | 14 | **PARTIEL** ⬆ | **Sprint 3 item 1 : la porte PROTECTED bloque réellement.** La classification est **remontée avant l'écriture** (`agent_loop.py:4277-4299`, verdict `:4299`) et **gate l'écriture durable** de M6.2 (`:4327-4336`) ; un tour PROTECTED n'émet plus aucun `update_profile` et le blocage est **visible** dans le flux (`memory_blocked`, `:4329`). Preuve : `tests/test_sprint3_p17_protected_persistence.py` (14 tests), dont la contre-épreuve « un tour PUBLIC est toujours mémorisé ». Constat initial mesuré : le log affirmait « not persisted » **pendant que** `data/memory-fs/profile.md` contenait `- [stated] User said: Mon diagnostic médical…`. **Corrigé aussi** : `_save_index` créait `data/` en dur et levait `FileNotFoundError` hors répertoire projet (`data_classification.py:226`) ⇒ la porte était **absente en silence** (règle 2) ; le log `:123` ne promettait plus que ce qu'il tient. **Réserves** : M6.6 loggue « output blocked » (`:4392`) **sans bloquer l'émission** ; `validate_memory_entry` / `sanitize_memory` : toujours 0 appelant ⇒ `POST /api/memory/add` ne filtre rien. La preuve du plan (`data/classification_index.json`) a été **re-dérivée** : l'index ne stocke que `key`/`level`/horodatages (`data_classification.py:44-50`), jamais la charge utile — y être absent y est donc vrai par construction. |
| **P18** | Lire avant d'écrire | `src/provenance_memory.py`, `src/hash_edit_validator.py` | `PROVENANCE_MEMORY=on` (Palier 0) | 1 | **PARTIEL** | `if_version` réel avec rejet (`provenance_memory.py:213-233`). **Mais** le jeton est lu et réémis dans le **même appel** (`:237` → `:253`) : la branche de rejet ne peut pas se déclencher en production, le contrôle est **tautologique**. `hash_edit_validator.py` n'a **toujours aucun appelant fonctionnel** — la version précédente de ce document affirmait le contraire. |
| **P19** | La mémoire s'applique seulement si elle change la réponse | `src/memory_impact.py` | `MEMORY_IMPACT=on` (Palier 0) | 0 | **PARTIEL** ⬆ | Switch `on` et `wired`, site live (`agent_loop.py:4424-4433`). **Mais** `hypothetical_response=""` (`:4431`) fait court-circuiter `evaluate_impact` à `0.0` (`memory_impact.py:51-52`) ⇒ `should_store` est **toujours** `False` ; et le résultat n'est **que loggé** (`:4433-4438`), jamais relié à un store. |
| **P20** | Les préférences se résolvent par priorité décroissante | `src/preferences.py`, `routes/prefs_routes.py` | `PREFERENCES=on` (Palier 0) | 11 | **PARTIEL** ⬆ | **Sprint 3 item 2 : la résolution atteint enfin le modèle.** `build_prompt_directive()` (`preferences.py:231-256`) traduit la résolution 5-niveaux en directive, injectée **en tête du system prompt** (`agent_loop.py:2645-2652`, à côté de `PLAN_MODE_DIRECTIVE`) ; le bloc M6.1 de fin de boucle ne fait plus que ** rapporter** ce qui a été injecté (`:4323-4341`). La preuve est comportementale, pas un log : le **stub LLM répond en français** quand la préférence est `fr`, et en anglais quand elle est `english` (`tests/test_sprint3_p20_preferences_prompt.py`, 11 tests, vérifiés par mutation). **Trou de sécurité corrigé au passage** : le guardrail `ignore (all \|your )?rules` ne filtrait pas « ignore **all your** rules » — et ces préférences étant désormais injectées au prompt, un guardrail troué est une **prompt injection stockée**. Motif durci (`preferences.py:63-67`) et implémentation unique partagée à l'entrée et à la sortie (`passes_behavioral_guardrails`, `:75-86`). **Réserves** : la directive ne couvre que `language`/`tone`/`format`/`length` — les autres préférences ne sont pas traduisibles en prompt ; `data/preferences.json` n'existe toujours pas par défaut ; l'API live reste un **autre** store, sans priorité contextuelle. |
| **P21** | Le bon outil au bon moment, sans friction | `src/tool_index.py`, `src/content_security.py`, `routes/mcp_routes.py` | `TOOL_DISCOVERY=off` (code), absent du registre | 19 | **PARTIEL** | Index d'outils statique vivant (`agent_loop.py:1646`) ; gestion MCP manuelle (`mcp_routes.py`, 11 endpoints). **Mort** : `ToolDiscovery` / `suggest_connectors` ont **0 appelant dans tout le dépôt** ; `@agentos/sfd-discovery` absent de `opencode.json`. Et même après réparation, `get_allowed_tools` (`progressive_disclosure.py:131`) n'est appelé que **depuis le module lui-même** (`:141`, `:146`) : **aucun appelant depuis la boucle**, donc la divulgation progressive ne restreint aucun outil. |
| **P22** | La sortie visuelle est une modalité de premier rang | `src/output_router.py`, `routes/mcp_tools_routes.py`, `src/visual_report.py` | `OUTPUT_ROUTER=on` (Palier 0) | 6 | **PARTIEL** | `route()` est **pur** — aucun effet de bord (`output_router.py:164-203`) — mais la décision n'est **que loggée** (`agent_loop.py:4335-4339`) : aucun fichier écrit, aucun outil invoqué. `connected_mcp_tools` n'est **jamais peuplé** ⇒ la branche `MCP_TOOL` (`:177-181`) est du code mort. Kroki reste le seul chemin visuel réel. |

> ⬆ = promu · ⬇ = rétrogradé par rapport à la mesure du 2026-09-25.

---

## 2. USE CASES UC-01 → UC-19

| ID | Cas d'usage | Statut | Preuve |
|---|---|---|---|
| **UC-01** | Lancer un nouveau projet | **PARTIEL** ⬇ | Le mode est détecté par mots-clés, pas par défaut agent (`opencode_engine.py:259`) ; `model_endpoints` est **vide** et `OPENCODE_API_KEY` est absent de `.env` ⇒ `agent_loop.py:2118` n'a aucun endpoint. Le flux nominal est donc inatteignable en l'état. |
| **UC-02** | Interrompre / modifier un projet | **PARTIEL** | Stop/Resume d'un run détaché opérationnels (`chat_routes.py:1455-1470`). `resume()` est appelé puis **seulement loggé** (`agent_loop.py:4314-4316`) ; `create_workflow` a 0 appelant. |
| **UC-03** | Consulter l'état d'avancement | **ACTIF** | `/api/observer/drift` (`observer_routes.py:17`) et `/current/{session_id}` (`phase_routes.py:38`) ; vérifié en exécution : HTTP 200 `{"ok":true,"drift_level":"low"}`. |
| **UC-04** | Ajouter un nouvel outil (MCP) | **ACTIF** | `POST /api/mcp/servers` persiste en base (`mcp_routes.py:163`) ; route enregistrée au boot (`core/route_loader.py:266`). La *découverte dynamique* reste partielle (cf. UC-15). |
| **UC-05** | Modifier le workflow d'un projet | **PARTIEL** ⬇ | `POST /api/phase/set` + `config/phase-lock.yaml` existent, **mais** `on_round_start` **écrase en `BUILD`** la phase posée par l'API (`agent_loop.py:2934` → `phase_tracker.py:60`) **avant** que `tool_execution.py:625` ne l'applique. L'API ne peut donc pas tenir. |
| **UC-06** | Forker un projet (worktree) | **ABSENT** | 0 occurrence de `git worktree`. `opencode_engine.py:188` : `worktree=` n'est qu'un `cwd` de travail. Confirmé par l'audit interne `tools/sfd_audit.py:286`. |
| **UC-07** | Fusionner un worktree | **ABSENT** | 0 appel git `merge`/`worktree` dans `src/`, `routes/`, `core/`, `services/`, `archive/`. `tools/sfd_audit.py:287` : « Non implémenté ». |
| **UC-08** | Consulter la mémoire transversale | **ACTIF** | 14 endpoints sur SQLite (`memory_routes.py:132/138/156/531`) ; ChromaDB dégradé **sans impact** (vérifié en exécution : `GET /api/memory` → 200). |
| **UC-09** | Recevoir une alerte | **PARTIEL** | Webhooks et notifications in-app actifs, **mais** une dérive `HIGH` n'est qu'un `logger.warning` (`agent_loop.py:4152`) — **aucune alerte** ; push externe (Apprise) `off` (`apprise_service.py:37`). |
| **UC-10** | Escalader vers une architecture multi-agent | **DORMANT** ⬇ | L'endpoint lève `ValueError` car les 12 switches agents sont `off` (`agent_dispatcher.py:201-204`), puis `RuntimeError` car `AGENT_CATALOG` est `off` (`:206-208`). Le dispatch explicite existe mais **fermé par défaut**. |
| **UC-11** | Reprendre après panne | **PARTIEL** ⬆ | Le switch est désormais `on`, mais `data/workflows/` est vide ⇒ `resume()` (`durable_execution.py:222`) rend toujours `None`. Seule la persistance de session survit à une déconnexion, pas à un crash. |
| **UC-12** | Auditer une décision passée | **PARTIEL** | Les traces **sont** écrites (`tool_execution.py:672` → `data/traces/*.jsonl`, 145 Ko produits), **mais** `trace_writer.py` n'expose **aucune fonction de lecture** et **aucune route** ne lit ce fichier. Les deux routes qui ressemblent à un audit sont des faux amis : `knowledge_routes.py:49` est un proxy CBM (service éteint) et `memory_routes.py:274` dépend d'un LLM. |
| **UC-13** | Retrouver une conversation passée | **ACTIF** | `GET /api/search` → `search_session_messages` (`chat_routes.py:1508-1520`, `session_search.py:301`, FTS5 avec repli `LIKE`) + outil agent. |
| **UC-14** | Définir une préférence persistante | **ACTIF** ⬆ | `GET /api/prefs` / `PUT /api/prefs/{key}` (`prefs_routes.py:79`/`:85`) avec écriture atomique (`os.replace`, `:30`) et scoping utilisateur (`:33-43`). |
| **UC-15** | Découvrir et connecter un nouveau service | **PARTIEL** | Découverte locale de modèles active (`model_routes.py:1743`), **mais** `suggest_connectors` (`content_security.py:245`) a 0 appelant. |
| **UC-16** | Obtenir une visualisation d'un concept | **PARTIEL** | La route existe (`mcp_tools_routes.py:24`) **mais Kroki est injoignable** (`curl` → code 000) et `KROKI_ENABLED=off` (`.env:371`). Le rapport visuel de recherche reste le seul chemin vivant. |
| **UC-17** | Exporter un artefact visuel | **PARTIEL** | `OUTPUT_ROUTER=on` mais la boucle ne logge que `mode`/`reason` (`agent_loop.py:4330-4339`) : `FILE_DOWNLOAD` n'est **jamais appliqué**. |
| **UC-18** | Gérer ses préférences | **PARTIEL** | `GET /api/prefs`, `GET /api/prefs/{key}`, `PUT /api/prefs/{key}` existent (`prefs_routes.py:74/79/85`) ; **aucun** `@router.delete` dans le fichier (confirmé via `openapi()["paths"]`). |
| **UC-19** | Consulter et gérer ses données (droit à l'oubli) | **PARTIEL** | La vraie route d'export est `GET /api/export` (`backup_routes.py:19-62`, `require_admin`) — vérifiée en exécution, 200, ~4,9 Ko. `admin_wipe_routes.py:76` : wipe global **admin-only**, pas d'auto-purge utilisateur. ⚠️ Les routes `POST /api/backup/export` (`:206`) et `/import` (`:211`) sont des **stubs** (`{"ok":true,"export":null}`) et ne doivent pas servir de preuve. |

> ⬆ = promu · ⬇ = rétrogradé par rapport à la mesure du 2026-09-25.

---

## 3. Résumé chiffré

### Principes P1 → P22

| Statut | Nombre | IDs |
|---|---|---|
| **ACTIF** | **5** | P3, P9, P10, P13, P15 |
| **PARTIEL** | **17** | P1, P2, P4, P5, P6, P7, P8, P11, P12, P14, P16, P17, P18, P19, P20, P21, P22 |
| **DORMANT** | **0** | — |
| **ABSENT** | **0** | — |
| **Total** | **22** | |

### Use cases UC-01 → UC-19

| Statut | Nombre | IDs |
|---|---|---|
| **ACTIF** | **5** | UC-03, UC-04, UC-08, UC-13, UC-14 |
| **PARTIEL** | **11** | UC-01, UC-02, UC-05, UC-09, UC-11, UC-12, UC-15, UC-16, UC-17, UC-18, UC-19 |
| **DORMANT** | **1** | UC-10 |
| **ABSENT** | **2** | UC-06, UC-07 |
| **Total** | **19** | |

### Bilan

| | Avant (2026-09-25) | Après (Palier 0) | Δ |
|---|---:|---:|---:|
| ACTIF | 12 | **10** | −2 |
| PARTIEL | 22 | **28** | +6 |
| DORMANT | 5 | **1** | −4 |
| ABSENT | 2 | **2** | = |
| **Total** | 41 | **41** | |

Les 4 principes DORMANT ont migré vers PARTIEL : leur switch est `on`, mais le module est soit appelé avec une entrée vide, soit branché sur un `logger.info`. **C'est la seule interprétation honnête** : « le switch est activé » n'a jamais voulu dire « le module agit ».

Les 2 ACTIF perdus ne sont pas des régressions du code : ce sont des dépendances et des câblages que la mesure précédente ne pouvait pas voir. **UC-01** attend une clé d'API absente. **UC-05** est écrasé par la phase-lock. Les corriger est du travail, pas de la documentation.

---

## 4. Écarts de configuration (section réécue le 2026-09-26)

### 4.1 Kill-switchs enregistrés **sans aucun lecteur dans le code**

Le cockpit (registre) en présentait 9 comme `on`. Aucun code ne les lit : ils annonçaient de l'activation. Marqués `wired=False` et remis à `off` — le registre ne peut plus afficher un switch inatteignable comme actif.

`ODYSSEUS_DEEPEVAL`, `ODYSSEUS_SUPABASE`, `ODYSSEUS_VAULTWARDEN`, `ODYSSEUS_PLAYWRIGHT`, `ODYSSEUS_BROWSER_HARNESS`, `ODYSSEUS_ZEN_FROM_ENDPOINT`, `ODYSSEUS_INPROCESS_DISCORD`, `ODYSSEUS_INPROCESS_TELEGRAM`, `ODYSSEUS_CHANNEL_AGENT_REPLY`

`tests/test_killswitch_registry.py` interdit désormais le retour de cette dérive : un descripteur `wired=True` doit avoir un lecteur réel, un `wired=False` ne doit pas en avoir, et le `default` affiché doit être celui que le code appliquera.

### 4.2 Descripteurs du registre contredisant leur propre lecteur

Seize descripteurs déclaraient `default: "on"` alors que leur lecteur retombe sur `off` ou sur chaîne vide — le cockpit affichait donc `effective=true` sur du OFF.

`LIVE_ORCHESTRATION`, `CODEBURN`, `LANGFUSE`, `GOVERNANCE_ANCESTRY`, `LANGGRAPH`, `RRF_FUSION`, `DOCLING`, `TREESITTER`, `DISABLE_MCP`, `OBSIDIAN_MCP`, `GRAPHIFY`, `CBM`, `SERENA_MCP`, `AGENTSEAL`, `N8N`, `AGENT_CATALOG`

Tous réalignés sur le code. **Corollaire notable** : `AGENT_CATALOG` est `off`, donc `/api/agents` renvoie `[]` — ce qui confirme UC-10 en DORMANT.

### 4.3 Variables d'environnement (la liste précédente était périmée)

Les divergences de nom signalées le 2026-09-25 ont été **nettoyées** : `ODYSSEUS_DURABLE_EXEC`, `ODYSSEUS_MEMORY_PROVENANCE`, `ODYSSEUS_VISUAL_OUTPUT` et `ODYSSEUS_PROJECT_MANIFEST` ont **0 occurrence** dans `.env`. `ODYSSEUS_THOUGHT_BUS` n'y est plus non plus et n'est posée que par `launch_fast.py:8`, sans lecteur — l'événement SSE `thought_bus` (`opencode_engine.py:349`) est inconditionnel.

| Constat | État réel |
|---|---|
| `ODYSSEUS_TRAEFIK=on` | Seule variable `ODYSSEUS_*` de `.env` (**49**) qu'aucun `.py` du dépôt ne lit. Relève du Palier P2 (reverse proxy). |
| `ODYSSEUS_TOOL_DISCOVERY` | Lue (`content_security.py:101`) mais `ToolDiscovery` a 0 appelant ⇒ sans effet. Défaut `off`, et absent du registre. |
| 5 variables P2 lues hors Python | `ODYSSEUS_OPA`, `ODYSSEUS_PREFECT`, `ODYSSEUS_OPA_URL`, `ODYSSEUS_API_TOKEN` : lues par les scripts de lancement, pas par le cœur applicatif. |

### 4.4 Divergences de décompte

- « 35 descriptors » puis « 47 étendus aux switches agents » (mesure du 2026-09-25) → le registre en compte **54**.
- Le bloc « Sprint 1a » de `04-OBJECTIFS-COUVERTURE.md` affirme que « les statuts ACTIF/PARTIEL/DORMANT n'ont pas bougé » et que l'Activation montera « plus tard ». C'est faux : le Palier 0 a **rétrogradé** UC-05 et fait passer UC-11 de DORMANT à PARTIEL.
- La liste « Actives (28) » de ce même document inclut « multi-agent live » et « goal-ancestry live », ce que les switches agents tous `off` contredisent.

### 4.5 Concordance avec l'audit interne existant

`tools/sfd_audit.py` (audit préexistant, non daté) corrobore cette vérification. **Réserve forte** : il référence une arborescence disparue (`src/durable_execution/saga.py`, `memory_provenance/`, `multi_agent_decision/`, `context_manager/`) qui n'existe plus dans le dépôt. Il ne peut donc pas servir de preuve, seulement de corroboration.

---

## 5. Ce qu'il reste à faire pour que ces statuts bougent

Rien de ce qui suit n'est un défaut : ce sont des câblages absents, tous réduits au même geste — **consommer le résultat au lieu de le loguer**.

| Principe / UC | Geste manquant |
|---|---|
| P5, P21 | Injecter `allowed_tools` dans les schémas d'outils envoyés au modèle (la divulgation ne restreint rien aujourd'hui). |
| P19 | Fournir une vraie `hypothetical_response` au vérificateur d'impact, et relier `should_store` au store. |
| P14, UC-02, UC-11 | Appeler `create_workflow` pour persister un workflow, et résoudre le chemin du vault pour le checkpoint. |
| P16 | Créer `topics/agent-output.md` (ou gérer son absence) pour que `[observed]`/`[inferred]` aient un producteur. |
| P17 | ~~Bloquer l'écriture durable sur une branche `PROTECTED`~~ **FAIT (Sprint 3 item 1)** — reste : faire **bloquer l'émission** par M6.6 (`:4386-4388` loggue « output blocked » sans bloquer) et appliquer `validate_memory_entry` sur `POST /api/memory/add` (0 appelant). |
| P20 | ~~Injecter `language`/`tone`/`format` dans le prompt~~ **FAIT (Sprint 3 item 2)** — reste : élargir la directive aux autres préférences, et brancher l'API live sur le même moteur de priorité. |
| P22, UC-17 | Donner un effet de bord à `OutputDecision` (écrire le fichier, invoquer l'outil). |
| UC-05 | Empêcher `on_round_start` d'écraser une phase posée explicitement par l'API. |
| UC-12 | Exposer une lecture des traces (route d'audit), ou assumer que les traces sont un artefact de debogage. |
| P18 | Faire circuler le jeton de version entre deux appels distincts, sinon le contrôle ne peut pas s'exercer. |

---

## 6. Sources

- `docs/master-ref/01-SFD-v3.1.md` §3 (P1-P22) et §4 (UC-01-19).
- Ancienne checklist `08-VERIFICATION-ETAT-ACTUEL.md` (retirée le 2026-09-25) — conservée dans l'historique Git.
- `src/killswitch_registry.py` — **54** descripteurs, `read_states()` (expose `default`, `raw`, `is_default`, `effective`, `wired`).
- `.env` — **49** variables `ODYSSEUS_*` (gitignoré) ; `.env.example` — template committé.
- `tests/test_killswitch_registry.py` — 4 tests d'audit qui interdisent au registre de mentir.
- `tests/test_progressive_disclosure.py` — 28 tests, dont la garde statique sur le site d'appel mort.
- Mesure de tests : `docs/traceability/04-TESTS-REALITE.md` (4 792 PASS · 2 skipped · 0 FAILED).
- `tools/sfd_audit.py` — corroboration seulement (arborescence périmée).
