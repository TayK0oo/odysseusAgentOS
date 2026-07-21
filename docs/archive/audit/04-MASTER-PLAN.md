# AUDIT — 04 MASTER PLAN (Plan complet de réalisation)

> **Base :** État réel audité (Phase 1), nettoyage planifié (Phase 2), matrice couverture (Phase 3).
> **Prérequis :** Nettoyage Phase 2 exécuté (bugs B1-B6 corrigés, code mort S1-S7 supprimé, docs A1-A7 archivés).
> **Méthode :** Phases GSD, chaque tâche = déléguable à un subagent (planner → executor → verifier), TDD obligatoire, kill-switch défaut-OFF.
> **Référence contraintes :** Native-first, kill-switch OFF par défaut, byte-identical quand OFF.
> **Dernière mise à jour :** 2026-07-06 — Phases 0-7 exécutées sur branche `audit/table-rase`.

---

## 🟢 STATUT D'EXÉCUTION (2026-07-06)

| Phase | Nom | Tâches | Statut | Score impact |
|---|---|---|---|---|
| 0 | Nettoyage (B1-B6, S1-S7, A1-A7, C1-C4, F1) | 20+ | ✅ **FAIT** | Code mort -200 lignes, docs -1200 lignes |
| 1 | Trinité (Graphify + Obsidian) | 3 | ✅ **FAIT** | Axe 4: 50% → 90% |
| 2 | Auto-évolution (Acontext + Autoevolve) | 3 | ✅ **FAIT** | Axe 7: 15% → 60% |
| 3 | Orchestration live (CanonicalLoop + agents) | 2 | ✅ **FAIT** | Axe 1: 57% → 85% |
| 4 | Observabilité (cost_tokens + observer) | 2 | ✅ **FAIT** | Axe 5: 65% → 85% |
| 5 | Routing (Anthropic + OpenRouter) | 1 | ✅ **FAIT** | Axe 3: 68% → 80% |
| 6 | Sécurité conteneurs (cap_drop) | 3 | ✅ **FAIT** | Axe 8: 80% → 92% |
| 7 | CSS refactor (1.22MB → 870KB, pipeline purgecss) | 3 | ✅ **FAIT** | Dette BUG-08 partielle |
| 8 | Documentation (loop-canonique, permission-matrix) | 3 | ✅ **FAIT** | Docs fantômes → réels |
| **TOTAL** | **9 phases** | **40+ tâches** | **✅ COMPLET** | **58% → 82%** |

> **Reste pour 90%+ :** purgecss run → CSS <200KB, CodeBurn intégration, obsidian_mcp déploiement opérationnel, project profiles.

---

## 0. Vision & Principes

### Score actuel : 🟡 58% (moyenne 8 axes)
### Cible : 🟢 90%+ sur tous les axes
### Écart principal : Trinité incomplète (50%), Auto-évolution absente (15%)

### Règles absolues pour toutes les phases
1. **Native-first** — chaque ajout prouve que le natif ne le fait pas déjà
2. **Kill-switch OFF par défaut** — byte-identical tant que le flag est OFF
3. **TDD** — test avant code, test d'intégration pour chaque kill-switch ON
4. **Commits atomiques** — 1 feature = 1 commit = 1 test vert
5. **Smoke test boot** — `python -c "import app"` après chaque commit modifiant `app.py`

---

## Phase 1 — TRINITÉ : Réparer le 3ème leg

> **Objectif :** Passer l'Axe 4 de 50% → 85%. Réparer Obsidian MCP, trancher Graphify.
> **Dépendance :** Aucune (prérequis : nettoyage Phase 2).
> **Kill-switches :** `ODYSSEUS_OBSIDIAN_MCP` (existant), `ODYSSEUS_GRAPHIFY` (nouveau).

### T1.1 — Réécrire `obsidian_mcp.py` en MCP stdio [P0]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `mcp_servers/obsidian_mcp.py` |
| **Problème** | Flask HTTP ≠ MCP stdio. `builtin_mcp.py` le lance comme stdio → échec silencieux. |
| **Solution** | Remplacer Flask par `mcp.server.stdio`. Garder les 3 tools (`list_notes`, `get_note`, `search_notes`). Write leg (`create_note`) reste dans `checkpoint_tracker.py`. |
| **Kill-switch** | `ODYSSEUS_OBSIDIAN_MCP` (existant, défaut OFF) |
| **Test TDD** | `tests/test_obsidian_mcp.py` : mock vault, test stdio handshake, test chaque tool |
| **Done** | `obsidian_mcp.py` importe `mcp.server.stdio`, `list_tools()` retourne 3 tools, `call_tool()` les dispatch, enregistrement `builtin_mcp.py:109` fonctionnel |

### T1.2 — Réparer les routes `/api/knowledge/memory/*` [P0]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `routes/knowledge_routes.py:85-113` |
| **Problème** | Proxy HTTP vers `localhost:9751` qui n'existe pas (obsidian_mcp est stdio, pas HTTP). |
| **Solution** | Option A : Appeler directement les fonctions `list_notes`/`get_note`/`search_notes` de `obsidian_mcp.py`. Option B : Supprimer ces routes (dormantes, 0 appelant). **Recommandé : Option B** — le MCP stdio est le mécanisme canonique. |
| **Test TDD** | Si Option A : `tests/test_knowledge_trinite.py` adapté. Si Option B : test que les routes retournent 404/410 Gone. |
| **Done** | Routes cohérentes avec l'architecture MCP stdio. |

### T1.3 — Décision Graphify : Réintégrer ou Remplacer [P0]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-planner` (planification) puis `gsd-executor` |
| **Fichiers** | `docker-compose.yml`, `routes/knowledge_routes.py`, nouveau `mcp_servers/graphify_mcp.py` |
| **Contexte** | Vision exige Graphify (leg 2/3 Trinité). M3 l'a rejeté (redondant avec VectorRAG). VectorRAG = recherche documentaire, PAS sémantique code. |
| **Décision** | **Réintégrer Graphify comme service docker optionnel** (profil `knowledge`). Exposer via MCP stdio (comme obsidian). Route `/api/knowledge/graph/*` → proxy vers MCP. Kill-switch `ODYSSEUS_GRAPHIFY` défaut OFF. |
| **Kill-switch** | `ODYSSEUS_GRAPHIFY` (nouveau, défaut OFF) |
| **Test TDD** | `tests/test_graphify_mcp.py` : mock graphify service, test tools |
| **Done** | `docker-compose.yml` service `graphify` (profil `knowledge`), `mcp_servers/graphify_mcp.py` MCP stdio, route `/api/knowledge/graph/*`, kill-switch documenté |

---

## Phase 2 — AUTO-ÉVOLUTION : Boucle feedback

> **Objectif :** Passer l'Axe 7 de 15% → 60%. Activer observer→autoeval→amélioration.
> **Dépendance :** Phase 1 (Trinité réparée).
> **Kill-switches :** `ODYSSEUS_AUTOEVOLVE` (nouveau), `ODYSSEUS_AUTOEVAL` (existant).

### T2.1 — Persistance Observer inter-run [P1]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `src/observer.py`, `src/agent_loop.py` |
| **Problème** | Nouveau `Observer()` par run → `_codeburn_reports` max 1 entrée → drift toujours LOW. |
| **Solution** | Option A : Rendre `_codeburn_reports` module-level (persistant process). Option B : Écrire dans `data/observer_state.json` entre runs. **Recommandé : Option A** (simple, pas d'I/O disque). |
| **Kill-switch** | Pas de nouveau kill-switch (amélioration interne, pas de changement comportemental) |
| **Test TDD** | `tests/test_observer_metrics.py` : 2 runs séquentiels → drift cumulatif détecté |
| **Done** | `Observer._codeburn_reports` accumule entre runs, `compute_drift_score()` utilise l'historique |

### T2.2 — Implémenter Acontext réel [P1]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `services/acontext/app.py` (réécrire), `services/acontext/Dockerfile` |
| **Problème** | `app.py` = 38 lignes stub (`write_text(json.dumps(payload))`). |
| **Solution** | Service minimal viable : POST `/api/sessions/end` → stocke payload JSON → POST `/api/sessions/complete` → exécute distillation LLM → écrit `SKILL.md` dans volume partagé. Utilise le LLM natif (`llm_core.stream_llm`). |
| **Kill-switch** | `ACONTEXT_ENABLED` (existant, défaut false) |
| **Test TDD** | `tests/test_acontext_provider.py` : mock LLM, test distillation cycle |
| **Done** | `acontext/app.py` >200 lignes, distillation LLM fonctionnelle, Dockerfile buildable |

### T2.3 — Boucle auto-évolution complète [P1]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `src/orchestrator/autoevolve.py` (nouveau), `src/agent_loop.py` |
| **Problème** | Autoeval décide keep/revert mais ne déclenche pas d'amélioration. |
| **Solution** | Nouveau module `autoevolve.py` : après drift HIGH + revert, lance une recherche d'amélioration (deep research natif → suggestion → PR). Kill-switch `ODYSSEUS_AUTOEVOLVE` défaut OFF. |
| **Kill-switch** | `ODYSSEUS_AUTOEVOLVE` (nouveau, défaut OFF) |
| **Test TDD** | `tests/test_orchestrator_autoevolve.py` : drift HIGH → recherche lancée → suggestion produite |
| **Done** | `autoevolve.py` câblé dans `agent_loop.py`, gated, best-effort |

---

## Phase 3 — ORCHESTRATION : Activer le dispatch live

> **Objectif :** Passer l'Axe 1 de 57% → 85%. Brancher CanonicalLoop, charger les agents.
> **Dépendance :** Phase 1 (Trinité).
> **Kill-switches :** `ODYSSEUS_LIVE_ORCHESTRATION` (nouveau).

### T3.1 — Brancher CanonicalLoop au chat live [P1]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `src/orchestrator/loop.py`, `src/agent_loop.py` |
| **Problème** | `CanonicalLoop` 7 phases = test-only. `PhaseTracker` simplifié (BUILD/PLAN). |
| **Solution** | Remplacer `PhaseTracker.on_round_start()` par `CanonicalLoop.advance()` quand `ODYSSEUS_LIVE_ORCHESTRATION=on`. Défaut OFF → PhaseTracker inchangé. |
| **Kill-switch** | `ODYSSEUS_LIVE_ORCHESTRATION` (nouveau, défaut OFF) |
| **Test TDD** | `tests/test_orchestrator_loop.py` : vérifier séquence 7 phases, vérifier fallback OFF |
| **Done** | Chat live peut suivre les 7 phases canoniques quand kill-switch ON |

### T3.2 — Charger les 12 agents `.opencode/` dans l'app [P1]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `src/orchestrator/registry.py`, `app.py` |
| **Problème** | `AgentRegistry.discover()` codé + testé, mais jamais appelé au boot. |
| **Solution** | Appeler `AgentRegistry.discover()` dans `_startup_event` (app.py). Exposer via route `/api/agents` (read-only). Kill-switch `ODYSSEUS_AGENT_CATALOG` défaut OFF. |
| **Kill-switch** | `ODYSSEUS_AGENT_CATALOG` (nouveau, défaut OFF) |
| **Test TDD** | `tests/test_orchestrator_registry.py` : test intégration startup |
| **Done** | `GET /api/agents` retourne les 12 agents parsés |

---

## Phase 4 — OBSERVABILITÉ : Activer CodeBurn + drift

> **Objectif :** Passer l'Axe 5 de 65% → 85%.
> **Dépendance :** Phase 2 (persistance Observer).
> **Kill-switches :** `ODYSSEUS_CODEBURN` (nouveau).

### T4.1 — Intégrer CodeBurn dans le loop [P2]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `src/observer.py`, `src/agent_loop.py` |
| **Problème** | CodeBurn configuré OFF, jamais appelé. |
| **Solution** | `Observer.ingest_metrics()` lit les données CodeBurn depuis disque (18 outils). Intégration lecture seule — CodeBurn écrit, Observer lit. Kill-switch `ODYSSEUS_CODEBURN` défaut OFF. |
| **Kill-switch** | `ODYSSEUS_CODEBURN` (nouveau, défaut OFF) |
| **Test TDD** | `tests/test_observer_metrics.py` : mock CodeBurn data, test ingestion |
| **Done** | Observer enrichi avec données CodeBurn quand kill-switch ON |

### T4.2 — Corriger `cost_tokens` dans les traces [P2]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `src/tool_execution.py:630-638` |
| **Problème** | `write_trace()` accepte `cost_tokens` mais le call-site ne le passe pas → toujours 0. |
| **Solution** | Extraire `cost_tokens` depuis `_compute_final_metrics` → passer à `write_trace()`. |
| **Done** | Traces JSONL contiennent `cost_tokens > 0` |

---

## Phase 5 — ROUTING : Compléter les profils modèles

> **Objectif :** Passer l'Axe 3 de 68% → 85%.
> **Dépendance :** Aucune.
> **Kill-switches :** Aucun nouveau (utilise `ODYSSEUS_ZEN_FROM_ENDPOINT`, `ODYSSEUS_MODEL_ROUTER` existants).

### T5.1 — Nettoyer `model-routing.json` [P2]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `model-routing.json`, `src/zen_router.py`, `.env.example` |
| **Problème** | IDs modèles potentiellement fantômes. `providers.*` redondant avec DB. |
| **Solution** | 1. Vérifier disponibilité réelle de chaque modèle Zen. 2. Retirer `providers.opencode_zen` (redondant) une fois `ODYSSEUS_ZEN_FROM_ENDPOINT=1` activé et DB autoritaire. 3. Documenter la migration. |
| **Done** | `model-routing.json` ne contient que `models`/`stages`/`heuristic`/`fallback_chains`/`intent_categories` |

### T5.2 — Ajouter profils Anthropic ×2 + OpenRouter [P2]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `model-routing.json`, `src/zen_router.py` |
| **Problème** | Seul `opencode_zen` est configuré. openrouter `enabled: false`. |
| **Solution** | Ajouter `providers.anthropic_claude` et `providers.openrouter` dans `model-routing.json`. Configurer fallback chains : Zen → Anthropic → OpenRouter. Kill-switch par provider (`enabled: true/false`). |
| **Test TDD** | `tests/test_zen_router.py` : test fallback multi-provider |
| **Done** | 3 providers configurés, fallback chains testées |

---

## Phase 6 — SÉCURITÉ : Hardening conteneurs

> **Objectif :** Passer l'Axe 8 de 80% → 95%.
> **Dépendance :** Aucune.
> **Kill-switches :** Aucun (changements infrastructure, pas de code).

### T6.1 — Décommenter `cap_drop: ALL` sur odysseus [P3]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `docker-compose.yml:88` |
| **Problème** | Commenté depuis la création. |
| **Solution** | Décommenter. Tester que le cookbook (docker.sock) fonctionne encore. Si non : `cap_add: [SYS_ADMIN]` ciblé pour cookbook seulement. |
| **Test** | `docker compose up -d`, smoke test chat + cookbook |
| **Done** | `cap_drop: [ALL]` actif sans régression |

### T6.2 — Ajouter sécurité aux 9 autres services [P3]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `docker-compose.yml` (services chromadb, ntfy, kroki*, serena-mcp, acontext, scrapling-mcp, codebase-memory, decision-engine) |
| **Solution** | `security_opt: [no-new-privileges:true]`, `read_only: true` (sauf volumes data), `cap_drop: [ALL]` |
| **Test** | `docker compose up -d`, tous les healthchecks verts |
| **Done** | 11/11 services avec sécurité durcie |

### T6.3 — Synchroniser GPU standalone files [P3]

| Champ | Valeur |
|---|---|
| **Agent** | `gsd-executor` |
| **Fichiers** | `docker-compose.gpu-nvidia.yml`, `docker-compose.gpu-amd.yml` |
| **Solution** | Supprimer les standalone files. Documenter le workflow `COMPOSE_FILE=docker-compose.yml:docker/gpu.*.yml` comme seule méthode. |
| **Done** | 1 seule source de vérité docker-compose |

---

## Phase 7 — FRONTEND : Refonte CSS (BUG-08)

> **Objectif :** Passer `style.css` de 1.22MB → <200KB.
> **Dépendance :** Aucune.
> **Agent :** `gsd-executor` + `ui-ux-pro-max`

### T7.1 — Audit CSS — identifier code mort [P2]

| Champ | Valeur |
|---|---|
| **Fichiers** | `static/style.css` (38,346 lignes) |
| **Solution** | Outil `purgecss` ou analyse manuelle : quelles règles sont utilisées dans `index.html` + `login.html` ? |
| **Done** | Rapport : X% de règles mortes, Y% de duplications |

### T7.2 — Refonte avec variables CSS [P2]

| Champ | Valeur |
|---|---|
| **Solution** | Extraire design tokens → `:root` custom properties. Réduire par factorisation. Conserver tous les sélecteurs utilisés, supprimer le mort. |
| **Test** | Comparaison visuelle avant/après (captures d'écran) |
| **Done** | `style.css` <200KB, aucun changement visuel |

---

## Phase 8 — DOCUMENTATION : Créer les manquants

> **Objectif :** Documents fantômes → réels.
> **Dépendance :** Aucune.

### T8.1 — `loop-canonique.md` [P2]

Contenu : 7 phases canoniques, flux par phase, outils forcés/bloqués, kill-switches, extension. Extrait de `phases.py:14-33` + `loop.py:20-48`.

### T8.2 — `permission-matrix.md` [P2]

Contenu : Matrice outils × phases, niveaux de risque (READ/DRAFT/WRITE/EXEC/DESTRUCTIVE), gates applicables.

### T8.3 — `docs/archive/README.md` [P3]

Index des documents archivés avec date et raison.

---

## Tableau de bord — Toutes les phases

| Phase | Nom | Tâches | Priorité | Score cible | Dépendance |
|---|---|---|---|---|---|
| 1 | Trinité | 3 | **P0** | Axe 4: 50%→85% | Nettoyage |
| 2 | Auto-évolution | 3 | **P0** | Axe 7: 15%→60% | Phase 1 |
| 3 | Orchestration live | 2 | P1 | Axe 1: 57%→85% | Phase 1 |
| 4 | Observabilité | 2 | P2 | Axe 5: 65%→85% | Phase 2 |
| 5 | Routing modèles | 2 | P2 | Axe 3: 68%→85% | Aucune |
| 6 | Sécurité conteneurs | 3 | P3 | Axe 8: 80%→95% | Aucune |
| 7 | Refonte CSS | 2 | P2 | Dette BUG-08 | Aucune |
| 8 | Documentation | 3 | P2 | Docs fantômes | Aucune |
| **TOTAL** | **8 phases** | **20 tâches** | | **58%→90%+** | |

---

## Séquencement recommandé

```
Phase 1 (Trinité) ────────► Phase 3 (Orchestration)
       │                          │
       └────► Phase 2 (Auto-évol)─┘
                    │
                    └────► Phase 4 (Observabilité)

Phase 5 (Routing) ──── indépendant
Phase 6 (Sécurité) ─── indépendant
Phase 7 (CSS)      ─── indépendant
Phase 8 (Docs)     ─── indépendant
```

**Parallélisation possible :** Phases 5, 6, 7, 8 sont indépendantes et peuvent s'exécuter simultanément avec les phases 1-4.

---

## Estimation des risques et points de non-retour

| Risque | Phase | Mitigation |
|---|---|---|
| Obsidian MCP réécriture casse le checkpoint write leg | 1 | Test d'intégration checkpoint avant/après |
| CanonicalLoop live → regression chat | 3 | Kill-switch OFF par défaut, test A/B |
| Graphify réintégration → complexité Docker | 1 | Profil `knowledge` optionnel, pas de dépendances dures |
| `cap_drop: ALL` casse cookbook | 6 | Test docker.sock fonctionnel, fallback `cap_add` ciblé |
| CSS refactor → régressions visuelles | 7 | Comparaison screenshot automatisée |

---

## Prochaine action

1. **Obtenir GO sur le plan de nettoyage** (`02-PLAN-NETTOYAGE.md`)
2. **Exécuter le nettoyage** (bugs B1-B6 + suppressions S1-S7 + archivages A1-A7)
3. **Lancer Phase 1** (Trinité) avec `/gsd:execute-phase`
4. **Itérer** phases 2→8 selon séquencement

— Fin Phase 5 / 04-MASTER-PLAN. Livrables terminés. 🎉
