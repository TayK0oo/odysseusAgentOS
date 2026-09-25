# Traçabilité PRINCIPES & USE CASES → Code réel

*Généré le 2026-09-25 · branche `feat/inventaire-global-v1` · dernière modification du dépôt : 2026-09-22*

Document **vérifiable** reliant les **22 principes SFD v3.1** (P1→P22, `docs/master-ref/01-SFD-v3.1.md` §3) et les **19 cas d'usage** (UC-01→UC-19, §4) au code réellement présent.

> **Méthode.** Analyse statique en lecture seule : `grep` ciblé dans `src/`, `routes/`, `core/`, `archive/`, `services/`, `.opencode/`, `packages/` ; vérification d'existence des fichiers ; lecture du kill-switch dans `src/killswitch_registry.py` et de sa valeur effective dans `.env` ; comptage des `tests/test_*.py` dont le **nom** matche un mot-clé.
>
> **Statuts.** `ACTIF` = code présent, chemin d'exécution par défaut, pas de kill-switch OFF. `DORMANT` = code présent mais kill-switch OFF par défaut (ou variable d'env. non lue). `PARTIEL` = implémentation partielle ou non branchée sur le flux live. `ABSENT` = non trouvé.
>
> **Avertissement.** Le point de vérité est le code. (L'ancienne checklist `08-VERIFICATION-ETAT-ACTUEL.md` a été **retirée le 2026-09-25** — périmée ; ce document la remplace.) Les écarts constatés sont listés en fin de document (§4).

---

## 1. PRINCIPES P1 → P22

| # | Principe | Fichiers (vérifiés) | Kill-switch (défaut) | Tests (nom) | Statut | Preuve |
|---|---|---|---|---|---|---|
| **P1** | Le risque modifie la boucle | `src/risk_classifier.py`, `src/orchestrator/gate.py`, `src/tool_execution.py`, `config/phase-lock.yaml` | `ODYSSEUS_DESTRUCTIVE_GATE=on` ; phase-lock sans switch | 7 (`test_orchestrator_gate.py`, `test_orchestrator_phaselock_integration.py`, `test_resend_message_nondestructive.py`, `test_e2e_complete.py`) | **PARTIEL** | `gate_enabled()` défaut `on` (`gate.py:24-27`) et réellement appelé (`tool_execution.py:588-594`) ; `classify_tool` est exécuté pour chaque outil (`tool_execution.py:565-580`). Mais la phase-lock retombe sur `default_phase=BUILD` non bloquante (`tool_registry.py:125-126`, `PHASE_TRACKER=off`) et les outils destructifs explicites ne sont ni bloqués ni soumis à approbation. |
| **P2** | Un brouillon n'est pas un commit | `routes/editor_draft_routes.py`, `archive/legacy/agent_loop.py`, `opencode.json` | — | 2 (`test_editor_draft_payload.py`, `test_plan_mode.py`) | **PARTIEL** | Brouillons d'éditeur persistés (`editor_draft_routes.py`, 5 endpoints) ; mode plan + plan approuvé (`agent_loop.py:2079`, `:2240`, `:2644`) ; `opencode.json` → `permission.edit="ask"`. Mais aucune séparation draft/commit généralisée : `routes/memory_routes.py:84` écrit directement en base. |
| **P3** | Contexte construit, pas déversé | `src/context_budget.py`, `src/context_compactor.py`, `src/model_context.py` | — | 8 (`test_context_budget.py`, `test_context_compactor.py`, `test_context_compactor_nonstring.py`, `test_model_context.py`, …) | **ACTIF** | Budget d'entrée calculé et appliqué (`agent_loop.py:2664-2665`) puis compaction `trim_for_context` (`agent_loop.py:2666`). |
| **P4** | Budgets obligatoires par projet | `src/budget_enforcer.py`, `src/project_manifest.py`, `src/governance.py`, `src/context_budget.py` | — (pas de switch) | 5 (`test_budget_auto_sentinel.py`, `test_governance_budgets_list.py`, `test_manage_settings_token_budget.py`, …) | **PARTIEL** | `BudgetEnforcer` n'est instancié que si `PROJECT.yaml` existe (`agent_loop.py:2761-2764`) — **absent** à la racine (seul `PROJECT.yaml.example`). Un budget de contexte par défaut s'applique (`context_budget.py`), mais pas de budget *projet* obligatoire. `ODYSSEUS_PROJECT_MANIFEST` (`.env`) n'est lu nulle part. |
| **P5** | Divulgation progressive | `src/progressive_disclosure.py` | `ODYSSEUS_PROGRESSIVE_DISCLOSURE=off` (registre `killswitch_registry.py:332-340`) | 1 (`test_orchestrator_phaselock_integration.py`) | **DORMANT** | Module complet (`progressive_disclosure.py:24`) mais jamais exécuté : seul appel dans `agent_loop.py:4382-4393`, sous `if env == on`, et il ne fait que **logger** un résumé. `.env` ne définit pas la variable → OFF. |
| **P6** | Échecs répétés → fonctionnalités du harnais | `src/orchestrator/autoeval.py`, `src/orchestrator/autoevolve.py`, `src/orchestrator/codeburn_runner.py`, `src/observer.py` | `ODYSSEUS_AUTOEVAL=off`, `AUTOEVOLVE=off`, `CODEBURN=off`, `CHECKPOINT=off` | 5 (`test_autoeval.py`, `test_orchestrator_autoeval.py`, `test_orchestrator_codeburn_runner.py`, `test_observer_metrics.py`) | **PARTIEL** | Observer/dérive actif par défaut (`observer.py:146`, `agent_loop.py:4142-4153`). En revanche les boucles d'apprentissage (autoeval/autoevolve/codeburn) sont toutes OFF ; aucune règle de superviseur auto-apprise n'est branchée. |
| **P7** | Structure > autonomie | `src/opencode_engine.py`, `src/orchestrator/loop.py`, `src/orchestrator/phases.py` | `ODYSSEUS_LIVE_ORCHESTRATION=off` (CanonicalLoop Python) — le walk 7 phases OpenCode n'a pas de switch | 3 (`test_orchestrator_loop.py`, `test_orchestrator_phases.py`, `test_orchestrator_phase_tracker.py`) | **ACTIF** | `OpenCodeEngine.walk()` impose les 7 phases par défaut (`opencode_engine.py:251-349`) et est appelé en mode agent (`routes/chat_routes.py:1413`). Variante Python `CanonicalLoop` dormante. |
| **P8** | Le plan passe les mêmes portes | `src/planning_engine.py`, `archive/legacy/agent_loop.py`, `routes/phase_routes.py` | `ODYSSEUS_PLANNING_ENGINE=off` (code) | 2 (`test_plan_mode.py`, `test_update_plan_tool.py`) | **PARTIEL** | Validation du plan par mode plan/approuvé (`agent_loop.py:2240`, `:2644`) et API de phase (`phase_routes.py:20`). Le moteur de planification structuré est OFF et n'est appelé que sous switch (`agent_loop.py:4366-4378`). |
| **P9** | Évaluer le harnais, pas le modèle | `src/observer.py`, `src/trace_writer.py`, `src/sse_indicators.py`, `src/perf_profiler.py` | `ODYSSEUS_LANGFUSE=off` (sous-partie) | 3 (`test_observer_metrics.py`, `test_observer_routes.py`, `test_trace_writer_run_id.py`) | **ACTIF** | Drift/observabilité calculés à chaque run (`agent_loop.py:4142-4160`), métriques SSE. `perf_profiler.py` n'a pas d'appelant (sous-partie non branchée). |
| **P10** | Humain ON the loop | `src/tool_index.py`, `archive/legacy/agent_loop.py`, `routes/chat_routes.py` | — | 2 (`test_ask_user_tool.py`, `test_ask_user_persistence.py`) | **ACTIF** | Outil `ask_user` déclaré et implémenté (`tool_index.py:482`, `agent_loop.py:3767-3969`) ; Stop/Resume du run (`chat_routes.py:1455`, `:1466`). |
| **P11** | Commencer simple, complexifier sur preuve | `archive/legacy/agent_loop.py`, `src/orchestrator/agent_dispatcher.py`, `.opencode/skills/sfd-decision-tree/` | Tous les switches `ODYSSEUS_AGENT_*=off` | 4 (`test_orchestrator_agent_dispatcher.py`, `test_orchestrator_dispatcher.py`, `test_orchestrator_dispatch_run.py`, `test_odysseus_dispatcher.py`) | **ACTIF** | Agent unique par défaut ; le multi-agent est opt-in via switches et la skill de décision `sfd-decision-tree`. Pas de décomposition automatique. |
| **P12** | Découpage par contexte, pas par métier | `src/orchestrator/agent_dispatcher.py`, `src/orchestrator/multi_agent.py`, `src/orchestrator/phases.py` | Tous les `ODYSSEUS_AGENT_*=off` ; `LIVE_ORCHESTRATION=off` | 9 (nom `agent`) | **PARTIEL** | Mapping phase→agent par contexte (`agent_dispatcher.py:38-66`), mais **tous désactivés par défaut** : aucun agent ne se déclenche automatiquement. |
| **P13** | Contexte = budget d'attention | `src/context_compactor.py`, `src/context_budget.py`, `src/provenance_memory.py` | Compaction : — ; mémoire incrémentale : `ODYSSEUS_PROVENANCE_MEMORY=off` | 8 | **PARTIEL** | Compaction active (P3). La mise à jour incrémentale de la mémoire par petites touches (provenance) est dormante. |
| **P14** | Survivre à une panne | `src/durable_execution.py`, `src/orchestrator/checkpoint_tracker.py`, `src/agent_runs.py` | `ODYSSEUS_DURABLE_EXECUTION` non défini (off) ; `ODYSSEUS_CHECKPOINT=off` | 0 | **DORMANT** | Module durable complet (`durable_execution.py:31`) mais lu sous switch dans `agent_loop.py:4297`. **Divergence de nom** : `.env` définit `ODYSSEUS_DURABLE_EXEC=on`, variable jamais lue → OFF effectif. |
| **P15** | On n'observe pas ce qu'on ne trace pas | `src/trace_writer.py`, `src/event_bus.py`, `src/opencode_engine.py` | `ODYSSEUS_UNIFIED_TOKENS=off` (sous-partie) | 8 (`test_trace_writer_run_id.py`, `test_classify_events_memory_text.py`, …) | **ACTIF** | Trace écrite pour **chaque** appel outil (`tool_execution.py:665-683`) ; bus d'événements avec `trace_id` corrélé par run (`opencode_engine.py:27-140`). |
| **P16** | Provenance explicite | `src/provenance_memory.py`, `src/memory_writer.py` | `ODYSSEUS_PROVENANCE_MEMORY=off` (non présent dans `killswitch_registry.py`) | 1 (`test_provenance_memory.py`) | **DORMANT** | Le module gère `[stated]/[observed]/[inferred]` (`provenance_memory.py:47`), mais il est lu sous switch (`agent_loop.py:4283`). **Divergence de nom** : `.env` définit `ODYSSEUS_MEMORY_PROVENANCE=on`, jamais lue. `memory_writer.py` (tags) n'a **aucun appelant**. |
| **P17** | Ne jamais stocker le sensible | `src/provenance_memory.py`, `src/data_classification.py`, `src/content_security.py`, `routes/memory_routes.py` | `PROVENANCE_MEMORY=off` ; `ODYSSEUS_DATA_CLASSIFICATION=on` (.env) ; `ODYSSEUS_CONTENT_SECURITY=on` (.env) | 1 (filtre d'omission) | **PARTIEL** | Listes protégées/sensibles/identifiables (`provenance_memory.py:50-86`) et classification (`data_classification.py:28`), mais : la classification ne fait que **logger** (`agent_loop.py:4332-4347`) et le flux mémoire live `POST /api/memory/add` n'applique **aucun** filtre sensible (`memory_routes.py:84-130`). |
| **P18** | Lire avant d'écrire | `src/provenance_memory.py`, `src/hash_edit_validator.py` | `ODYSSEUS_PROVENANCE_MEMORY=off` | 1 (`test_provenance_memory.py`) | **PARTIEL** | Contrôle de version `if_version` réel (`provenance_memory.py:213-281`) mais dormant. Pour les fichiers, `hash_edit_validator.py` est exporté (`src/__init__.py:5`) et actif. `memory_writer.py` calcule un hash **sans jamais le vérifier** — contrairement à ce qu'affirme la checklist 08. |
| **P19** | La mémoire s'applique seulement si elle change la réponse | `src/memory_impact.py` | `ODYSSEUS_MEMORY_IMPACT=off` (registre `killswitch_registry.py:341-349`) | 0 | **DORMANT** | Module `MemoryImpactVerifier` présent (`memory_impact.py:22`) mais appelé seulement sous switch (`agent_loop.py:4397-4416`) et avec `hypothetical_response=""` → score toujours `0.0` donc `should_store=False`. Jamais branché sur le store. |
| **P20** | Les préférences se résolvent par priorité décroissante | `src/preferences.py`, `routes/prefs_routes.py`, `archive/legacy/agent_loop.py` | `ODYSSEUS_PREFERENCES` = `on` dans `.env` (défaut code `off`, absent du registre) | 4 (`test_prefs_routes.py`, `test_prefs_atomic_write.py`, `test_prefs_single_user_no_clobber.py`, `test_new_chat_model_preference.py`) | **PARTIEL** | Résolution à 5 niveaux + guardrails (`preferences.py:12-17`, `:55-68`) mais seulement **loggée** (`agent_loop.py:4265-4277`). L'API live (`prefs_routes.py`) est un **autre** store clé/valeur sans priorité contextuelle. |
| **P21** | Le bon outil au bon moment, sans friction | `src/tool_index.py`, `src/tool_registry.py`, `routes/mcp_routes.py`, `src/content_security.py` | `ODYSSEUS_TOOL_DISCOVERY=on` (.env) mais **sans appelant** ; switches MCP `off` | 19 (13 MCP, 3 `tool_index`, 3 `discovery`) | **PARTIEL** | Index d'outils statique actif (sélection d'outils dans le prompt) et gestion MCP manuelle (`mcp_routes.py`, 11 endpoints). `ToolDiscovery`/`suggest_connectors` (`content_security.py:100-290`) n'est **appelé nulle part** ; `sfd-discovery` n'est pas chargé (`opencode.json` ne référence que `@agentos/sfd-eventbus`). |
| **P22** | La sortie visuelle est une modalité de premier rang | `src/output_router.py`, `routes/mcp_tools_routes.py`, `src/visual_report.py`, `packages/sfd-visual` | `ODYSSEUS_OUTPUT_ROUTER` non défini (off) ; `ODYSSEUS_VISUAL_OUTPUT=on` (.env) **jamais lue** ; `sfd-visual` non chargé | 6 (`test_render_diagram_tool.py`, `test_visual_report.py`, …) | **PARTIEL** | Rendu Kroki réellement exposé (`mcp_tools_routes.py:22-38`) et rapport visuel actif pour la recherche (`visual_report.py:1724`, appelé par `research_handler.py:712`). En revanche l'arbre de décision de modalité (`output_router.py:25`) est OFF et n'est invoqué que pour log (`agent_loop.py:4313-4330`). |

---

## 2. USE CASES UC-01 → UC-19

| ID | Cas d'usage | Fichiers (vérifiés) | Kill-switch (défaut) | Tests (nom) | Statut | Preuve |
|---|---|---|---|---|---|---|
| **UC-01** | Lancer un nouveau projet | `routes/chat_routes.py`, `src/opencode_engine.py`, `archive/legacy/agent_loop.py` | — (pas de switch) | e2e / `test_orchestrator_loop.py` | **ACTIF** | Mode agent → `OpenCodeEngine.walk()` sur 7 phases (`chat_routes.py:1372-1414`). |
| **UC-02** | Interrompre / modifier un projet | `routes/chat_routes.py`, `src/agent_runs.py`, `src/durable_execution.py` | `ODYSSEUS_DURABLE_EXECUTION=off` | `test_agent_rounds_exhausted.py` | **PARTIEL** | Stop/Resume d'un run détaché opérationnels (`chat_routes.py:1455-1470`, `agent_runs.py`). La reprise « exactement où elle s'est arrêtée » après crash dépend de l'exécution durable, dormante. |
| **UC-03** | Consulter l'état d'avancement | `routes/observer_routes.py`, `routes/phase_routes.py`, `archive/legacy/agent_loop.py`, `static/` | — | `test_observer_routes.py` | **ACTIF** | `/drift` (`observer_routes.py:17`), `/current/{session_id}` (`phase_routes.py:38`), `run_status` SSE (drift + budget) (`agent_loop.py:4160`). |
| **UC-04** | Ajouter un nouvel outil (MCP) | `routes/mcp_routes.py`, `src/mcp_manager.py`, `src/tool_registry.py` | `ODYSSEUS_DISABLE_MCP=off` ; connexions externes OFF | 13 (`test_mcp_manager.py`, …) | **ACTIF** | API CRUD/administration MCP active (11 endpoints, `mcp_routes.py`), enregistrée au boot (`route_loader.py:264`). La *découverte dynamique* reste partielle (cf. UC-15). |
| **UC-05** | Modifier le workflow d'un projet | `config/phase-lock.yaml`, `config/tool-decision-tree.yaml`, `routes/phase_routes.py` | — | `test_orchestrator_phaselock_integration.py` | **ACTIF** | Configuration déclarative des phases (`config/phase-lock.yaml`) et changement de phase (`phase_routes.py:20`, `route_loader.py:317-320`). |
| **UC-06** | Forker un projet (worktree) | `src/opencode_engine.py`, `src/opencode_bridge.py`, `src/decision_engine.py` | — | 0 | **ABSENT** | Aucune occurrence de `git worktree` dans le code. `worktree=` n'est qu'un chemin de travail passé à moteur/bridge (`opencode_engine.py:185-188`). Confirmé par l'audit interne `tools/sfd_audit.py:286` : `"UC-06 Forker worktree": {"live": False, "note": "Non implémenté"}`. |
| **UC-07** | Fusionner un worktree | — | — | 0 | **ABSENT** | Même constat : aucun merge de branche/worktree. `tools/sfd_audit.py:287` : `"Non implémenté"`. |
| **UC-08** | Consulter la mémoire transversale | `routes/memory_routes.py`, `src/memory.py`, `services/memory/`, `archive/legacy/agent_loop.py` | Mémoire : préférence utilisateur ; `ODYSSEUS_OBSIDIAN_MCP=off` | 21 (nom `memory`) | **ACTIF** | API riche : liste, recherche, timeline, profil, rappel (`memory_routes.py`, 14 endpoints) et rappel mémoire dans la boucle (tests `test_memory_recall_nondict_rows.py`). |
| **UC-09** | Recevoir une alerte | `src/task_scheduler.py`, `services/notifications/apprise_service.py`, `src/observer.py`, `src/webhook_manager.py` | `ODYSSEUS_APPRISE=off` (défaut), canaux in-process OFF | `test_budget_auto_sentinel.py` | **PARTIEL** | Notifications in-app actives (`task_scheduler.py:499`), observateur de dérive actif. Push externe (Apprise/ntfy) gated OFF (`apprise_service.py:32-37`). |
| **UC-10** | Escalader vers une architecture multi-agent | `src/orchestrator/agent_dispatcher.py`, `src/orchestrator/multi_agent.py`, `src/decision_engine.py`, `.opencode/skills/sfd-decision-tree/` | Tous `ODYSSEUS_AGENT_*=off`, `LIVE_ORCHESTRATION=off` | `test_orchestrator_agent_dispatcher.py` | **PARTIEL** | Dispatch explicite via `POST /api/agents/dispatch` (`route_loader.py:379-407`) et scorer de décision. Aucune escalade automatique chiffrée ; `/api/agents` retourne `[]` sans `ODYSSEUS_AGENT_CATALOG=on` (`route_loader.py:367-371`). |
| **UC-11** | Reprendre après panne | `src/durable_execution.py`, `src/orchestrator/checkpoint_tracker.py` | `ODYSSEUS_DURABLE_EXECUTION=off` ; `CHECKPOINT=off` | 0 | **DORMANT** | Toute la couche de reprise durable est sous switch OFF ; seule la persistance de session/détachement (`agent_runs`) survit à une déconnexion client, pas à un crash process. |
| **UC-12** | Auditer une décision passée | `src/trace_writer.py`, `src/opencode_engine.py`, `routes/history_routes.py` | Base : — ; `UNIFIED_TOKENS=off` | `test_trace_writer_run_id.py` | **PARTIEL** | Traces JSONL par appel outil et `trace_id` corrélé (`opencode_engine.py:254-255`). Aucune route/UI d'audit dédiée (les traces sont sur disque). |
| **UC-13** | Retrouver une conversation passée | `src/session_search.py`, `routes/chat_routes.py`, `src/tools/search.py` | — (`ODYSSEUS_MEILISEARCH=off` pour l'index externe) | 2 (`test_session_search.py`, `test_session_search_batch_fetch.py`) | **ACTIF** | `GET /api/search` → `search_session_messages` (`chat_routes.py:1508-1527`) et outil agent (`tools/search.py:23`). |
| **UC-14** | Définir une préférence persistante | `routes/prefs_routes.py`, `src/preferences.py` | `ODYSSEUS_PREFERENCES=on` (.env) ; résolveur structuré défaut `off` | `test_prefs_routes.py` | **PARTIEL** | Persistance clé/valeur active (`prefs_routes.py:82-92`). Application contextuelle par priorité (module `preferences.py`) seulement loggée (cf. P20). |
| **UC-15** | Découvrir et connecter un nouveau service | `routes/mcp_routes.py`, `src/content_security.py`, `src/model_discovery.py` | `ODYSSEUS_TOOL_DISCOVERY=on` mais sans appelant ; MCP externes OFF | 13 MCP + 3 discovery | **PARTIEL** | Connexion MCP manuelle active ; découverte de modèles locale active (`model_discovery.py`). La recherche de registre/suggestion de connecteurs n'est pas branchée. |
| **UC-16** | Obtenir une visualisation d'un concept | `routes/mcp_tools_routes.py`, `src/visual_report.py`, `src/output_router.py` | `ODYSSEUS_OUTPUT_ROUTER=off` ; `ODYSSEUS_VISUAL_OUTPUT` (morte) | `test_render_diagram_tool.py`, `test_visual_report.py` | **PARTIEL** | Rendu Kroki SVG/PNG via `POST /api/tools/diagram` (`mcp_tools_routes.py:22-38`) et rapport visuel de recherche (`visual_report.py:1724`). Le routage automatique par arbre de décision est dormant. |
| **UC-17** | Exporter un artefact visuel | `src/output_router.py`, `src/visual_report.py` | `ODYSSEUS_OUTPUT_ROUTER=off` | `test_visual_report.py` | **PARTIEL** | Le mode `FILE_DOWNLOAD` de `output_router.py` est dormant. L'export existe indirectement via le HTML téléchargeable généré par `visual_report.py` (ligne ~908 du JS embarqué). |
| **UC-18** | Gérer ses préférences | `routes/prefs_routes.py`, `src/preferences.py` | — (API simple) | 4 | **PARTIEL** | Lecture et modification exposées (`GET /api/prefs`, `PUT /api/prefs/{key}`) mais **aucun** endpoint `DELETE` (`prefs_routes.py` : 0 occurrence de `delete`). Pas d'UI pour le module structuré. |
| **UC-19** | Consulter et gérer ses données (droit à l'oubli) | `routes/admin_wipe_routes.py`, `routes/memory_routes.py` | — | 1 (`test_admin_wipe_gallery.py`) | **PARTIEL** | Wipe par catégorie (`DELETE /api/admin/wipe/{kind}` : chats, memory, skills, notes, tasks, documents, gallery, calendar) et suppression unitaire de mémoire (`memory_routes.py:587`). Le wipe est **admin-only** (`require_admin`) ; pas de flux d'auto-suppression utilisateur global. |

---

## 3. Résumé chiffré

### Principes P1 → P22

| Statut | Nombre | IDs |
|---|---|---|
| **ACTIF** | **6** | P3, P7, P9, P10, P11, P15 |
| **PARTIEL** | **12** | P1, P2, P4, P6, P8, P12, P13, P17, P18, P20, P21, P22 |
| **DORMANT** | **4** | P5, P14, P16, P19 |
| **ABSENT** | **0** | — |
| **Total** | **22** | |

### Use cases UC-01 → UC-19

| Statut | Nombre | IDs |
|---|---|---|
| **ACTIF** | **6** | UC-01, UC-03, UC-04, UC-05, UC-08, UC-13 |
| **PARTIEL** | **10** | UC-02, UC-09, UC-10, UC-12, UC-14, UC-15, UC-16, UC-17, UC-18, UC-19 |
| **DORMANT** | **1** | UC-11 |
| **ABSENT** | **2** | UC-06, UC-07 |
| **Total** | **19** | |

> Constat d'ensemble : sur 41 exigences, **12 sont pleinement actives**, **22 partielles**, **5 dormantes** et **2 absentes**. Aucun principe n'est totalement absent, mais 4 des 5 modules de la vague « mémoire/interaction » (P16, P17 partiel, P18 partiel, P19) sont neutralisés par des kill-switches OFF ou par des variables d'environnement mal nommées.

---

## 4. Divergences avec l'ancienne checklist 08 (retirée 2026-09-25)

### 4.1 Statuts divergents (principes)

| # | Checklist 08 | Réalité vérifiée | Cause de l'écart |
|---|---|---|---|
| P1 | 🟢 SATISFAIT | PARTIEL | Le gate bloque les commandes shell catastrophiques mais n'impose pas d'approbation humaine sur les outils destructifs explicites ; phase-lock neutralisée par `PHASE_TRACKER=off` (défaut BUILD). |
| P2 | 🟢 SATISFAIT | PARTIEL | « Draft/commit séparés dans agents » : seuls les brouillons d'éditeur et le mode plan existent ; la mémoire/écriture directe n'a pas de staging. |
| P4 | 🟢 SATISFAIT | PARTIEL | `budget_enforcer.py` n'agit que si `PROJECT.yaml` existe — absent (seul `.example`). |
| P5 | 🟢 SATISFAIT « Phase-lock : permissions par phase » | **DORMANT** | `progressive_disclosure.py` OFF par défaut ; phase-lock non alimentée (phase par défaut BUILD). |
| P6 | 🟡 PARTIEL | PARTIEL | Cohérent. |
| P12 | 🟡 PARTIEL | PARTIEL | Cohérent, mais plus sévère : 100 % des agents auto sont OFF. |
| P14 | 🟢 SATISFAIT « Durable execution codé » | **DORMANT** | Module OFF et **variable `.env` erronée** (voir §4.3). |
| P16 | 🟢 SATISFAIT « tags actifs » | **DORMANT** | `.env` écrit `ODYSSEUS_MEMORY_PROVENANCE`, le code lit `ODYSSEUS_PROVENANCE_MEMORY` ; `memory_writer.py` sans appelant. |
| P17 | 🟢 SATISFAIT « omission filter » | PARTIEL | Filtre présent uniquement dans des modules dormants/non branchés ; le flux live `POST /api/memory/add` ne filtre pas. |
| P18 | 🟢 SATISFAIT « if_version hash check dans memory_writer » | PARTIEL | Faux : `memory_writer.py` calcule un hash mais ne le compare à rien. Le `if_version` réel est dans `provenance_memory.py` (dormant). |
| P19 | 🟢 SATISFAIT « Earn its place codé » | **DORMANT** | OFF + logique inopérante (`hypothetical_response=""` → impact nul). |
| P20 | 🟢 SATISFAIT « 5 niveaux » | PARTIEL | Résolveur seulement loggé ; l'API live est un store distinct sans priorité. |
| P21 | 🟢 SATISFAIT « Tool discovery + registry MCP » | PARTIEL | `ToolDiscovery` sans appelant ; registre MCP en gestion manuelle. |
| P22 | 🟢 SATISFAIT « Kroki SVG inline, arbre décision » | PARTIEL | Kroki actif, mais `output_router.py` OFF ; `ODYSSEUS_VISUAL_OUTPUT` n'est lue par personne. |

### 4.2 Statuts divergents (use cases)

| ID | Checklist 08 | Réalité vérifiée | Cause de l'écart |
|---|---|---|---|
| UC-04 | 🟡 CODE-ONLY | ACTIF | L'API MCP est enregistrée et opérationnelle (11 endpoints). |
| UC-05 | 🟡 CODE-ONLY | ACTIF | `POST /api/phase/set` + `config/phase-lock.yaml` actifs. |
| UC-06 | 🟡 « Git worktree support codé » | **ABSENT** | Aucune commande `git worktree` dans le dépôt ; confirmé par `tools/sfd_audit.py:286`. |
| UC-07 | 🟡 « Codé » | **ABSENT** | Idem ; `tools/sfd_audit.py:287`. |
| UC-10 | 🟢 SATISFAIT « 16 agents, routing par phase » | PARTIEL | Tous les switches agents sont OFF ; `/api/agents` vide sans `ODYSSEUS_AGENT_CATALOG=on`. |
| UC-11 | 🟡 CODE-ONLY | **DORMANT** | Couche durable OFF. |
| UC-13 | 🟡 CODE-ONLY | ACTIF | `session_search` branché dans `chat_routes.py` et l'outil agent. |
| UC-14 | 🟢 SATISFAIT | PARTIEL | Application contextuelle non branchée. |
| UC-18 | 🟢 « API /api/preferences » | PARTIEL | La route réelle est `/api/prefs` et n'offre **pas** de suppression. |
| UC-19 | 🟡 CODE-ONLY | PARTIEL | Wipe admin-only + suppression unitaire ; pas de flux utilisateur global. |

### 4.3 Variables d'environnement divergentes (nom ou lecteur)

| Variable dans `.env` | Valeur | Réalité |
|---|---|---|
| `ODYSSEUS_DURABLE_EXEC` | `on` | **Jamais lue.** Le code lit `ODYSSEUS_DURABLE_EXECUTION` (non défini) → P14/UC-11 OFF. |
| `ODYSSEUS_MEMORY_PROVENANCE` | `on` | **Jamais lue.** Le code lit `ODYSSEUS_PROVENANCE_MEMORY` (non défini) → P16 OFF. |
| `ODYSSEUS_VISUAL_OUTPUT` | `on` | **Jamais lue.** Le code lit `ODYSSEUS_OUTPUT_ROUTER` (non défini) → P22/UC-16/17 OFF. |
| `ODYSSEUS_THOUGHT_BUS` | `on` | **Jamais lue** dans le code Python. |
| `ODYSSEUS_PROJECT_MANIFEST` | `off` | **Jamais lue** ; `load_manifest()` est appelé inconditionnellement. |
| `ODYSSEUS_TOOL_DISCOVERY` | `on` | Lue (`content_security.py:100`) mais `ToolDiscovery` n'a aucun appelant → sans effet. |

### 4.4 Divergences de décompte

- Checklist 08 : « **45+ switches dans .env** » → `src/killswitch_registry.py` ne curate que **35** descriptors, étendus à **47** avec les 12 switches agents. `.env` contient **48** variables `ODYSSEUS_*`, dont plusieurs ne sont pas des kill-switches (tokens, URLs).
- Checklist 08 : « P1-P22 : 🟢 20 · 🟡 2 · 🔴 0 » → vérification code : **🟢 6 · 🟡 12 · ⚪ 4 · 🔴 0**.
- Checklist 08 : « UC : 🟢 9 · 🟡 10 · 🔴 0 » → vérification code : **🟢 6 · 🟡 10 · ⚪ 1 · 🔴 2**.

### 4.5 Concordance avec l'audit interne existant

`tools/sfd_audit.py` (audit préexistant, non daté) converge davantage avec cette vérification qu'avec `08` : il marque notamment non implémentés P5, P6, P12, P19, « live: False » pour P14/P16/P17/P18/P20/P21/P22, et « Non implémenté » pour UC-06/UC-07. **Réserve** : cet audit référence une arborescence obsolète (`src/durable_execution/saga.py`, `memory_provenance/`, `multi_agent_decision/`, `context_manager/`) qui n'existe plus dans le dépôt — il ne peut donc pas servir de preuve directe, seulement de corroboration.

---

## 5. Sources

- `docs/master-ref/01-SFD-v3.1.md` §3 (P1-P22, lignes ~85-125) et §4 (UC, lignes ~128-152).
- Ancienne checklist `08-VERIFICATION-ETAT-ACTUEL.md` (retirée) — conservée dans l'historique Git.
- `src/killswitch_registry.py` (47 descriptors, `read_states()`).
- `.env` / `.env.example` (48 variables `ODYSSEUS_*`).
- `tools/sfd_audit.py` (audit interne de corroboration).
- Code : `src/orchestrator/*`, `src/opencode_engine.py`, `archive/legacy/agent_loop.py`, `src/tool_execution.py`, `routes/*`, `config/*.yaml`, `packages/*`, `.opencode/*`.
