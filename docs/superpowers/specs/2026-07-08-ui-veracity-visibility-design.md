# UI Veracity & Visibility Batch — Design

**Date:** 2026-07-08
**Statut:** Validé (design approuvé par l'utilisateur), prêt pour planification
**Nature:** Fixes de véracité UI + panneaux read-only. Basé sur reconnaissance parallèle qui a corrigé deux prémisses de la cartographie.

## Contexte et corrections de prémisses

La cartographie signalait des « angles morts UI » et un « fantôme trace_writer ». La reconnaissance a établi :

1. **`trace_writer` n'est PAS un fantôme.** `src/trace_writer.py` existe et est **actif** à chaque tour (token accounting + traces JSONL) : `src/agent_loop.py:1934,3584`, `src/tool_execution.py:542,630`, kill-switch `ODYSSEUS_UNIFIED_TOKENS` référencé dans `src/killswitch_registry.py:64`. La CBM ne l'avait pas indexé → faux fantôme. **Rien à supprimer.** Le seul vrai stub aspirationnel : la chaîne `"memory_distill"` dans `src/orchestrator/phases.py:57` (`_FORCED_TOOLS[MEMORY_OBSERVE]`), sans implémentation ; et `forced_tools(MEMORY_OBSERVE)` n'est jamais appelé au runtime.

2. **Le chip `phase` du cockpit est malhonnête (déjà en prod).** `static/js/cockpit.js:29-30` rend `#cockpit-phase` en `chip-ok` (vert) dès que `s.phase` est non-null. Or `PhaseTracker.infer_phase()` renvoie toujours `"BUILD"`/`"PLAN"` **indépendamment** des kill-switches (les deux orchestrateurs sont OFF par défaut ; le flag `enabled` ne bloque que l'écriture registre, pas `infer_phase`). Donc le chip paraît « actif/sain » alors qu'il n'affiche que le défaut codé en dur. Ce travers viole la règle d'honnêteté maison (cf. jambe Obsidian de la Trinité rendue neutre).

## Principe directeur : honnêteté

Un indicateur ne passe au vert (`chip-ok`) que s'il reflète un **signal réel et actif**. Sinon il reste neutre (`chip-muted`) tout en pouvant afficher sa valeur littérale. Jamais de valeur fabriquée.

## Unités

### Unité 1 — Fix honnêteté du chip phase (le plus prioritaire)

Le backend doit dire au frontend si la phase est **réellement orchestrée** ou juste le défaut.

**`src/sse_indicators.py`** — `run_status_event(...)` gagne un paramètre `phase_active: bool = False` ; le dict retourné inclut `"phase_active": bool(phase_active)`. Fonction pure, testable.

**`src/agent_loop.py:2591-2597`** — au point d'émission `run_status_event`, passer
`phase_active=bool(_use_canonical or getattr(_phase_tracker, "_enabled", False))`.
`_use_canonical` est défini à 2493 ; `_phase_tracker._enabled` reflète la décision constructeur (`tracker_enabled()`). Aucune autre ligne du loop touchée.

**`static/js/cockpit.js:29-30`** — le chip phase utilise la classe `s.phase_active ? 'chip-ok' : 'chip-muted'` (au lieu de `s.phase ? 'chip-ok' : 'chip-muted'`). La **valeur** reste affichée (ex. « BUILD ») mais neutre tant que l'orchestration n'est pas active.

**Tests** : `tests/test_sse_indicators.py` (ajout ou création) — `run_status_event(..., phase_active=False)` → dict contient `"phase_active": False` ; `phase_active=True` → `True` ; défaut omis → `False`. Le reste du dict inchangé (non-régression des champs existants).

### Unité 2 — Panneau Drift (read-only)

Pas d'endpoint REST aujourd'hui ; `src/observer.py::Observer.get_summary()` existe (calcul toujours actif, non gated). Deux sous-parties.

**Nouvelle route** `routes/observer_routes.py` : `router = APIRouter(prefix="/api/observer", tags=["observer"])`, handler async `GET /drift` → `{"ok": True, "drift_level": "low|medium|high", "harness_touched": bool, "runs_tracked": int, "latest_codeburn_one_shot_rate": float|None, "recommendation": str}` en lisant `Observer.get_summary()` du singleton observer utilisé par le loop. try/except → `{"ok": False, "error": str}`. Enregistrée dans le bloc M3 de `app.py`.

**Panneau** : `admin-card#settings-drift-card` dans le panneau System de `static/index.html` (après la card Trinité) ; `initDriftView()`/`loadDrift()`/`renderDrift(data)` dans `static/js/admin.js`, ajouté à `inits`. Affiche : chip niveau (`chip-ok` low / `chip-warn` medium / `chip-bad` high), badge `harness_touched`, `latest_codeburn_one_shot_rate` en %, et la `recommendation`. Réutilise `ksClean()` pour l'échappement. Pas d'historique (l'état vit en mémoire, pas de persistance).

**Tests** : `tests/test_observer_routes.py` — contract test : `GET /api/observer/drift` → 200, `body["ok"] is True`, `body["drift_level"] in {"low","medium","high"}`, présence des clés.

### Unité 3 — Panneau Budgets (read-only)

Endpoint partiel existant : `GET /api/governance/budgets/{agent_id}?run_id=` mais **pas de list-all**.

**Nouvelle route** dans `routes/governance_routes.py` (fichier existant) : `GET /api/governance/budgets` (sans agent_id) → `{"ok": True, "budgets": [ {agent_id, run_id, status, tokens_used, max_tokens, cost_usd, max_cost_usd, iterations, max_iterations, paused_reason}, ... ]}` en énumérant les lignes `AgentBudget` (`core/database.py:2422`). Suivre le style du fichier (attention : il lève `HTTPException` ; le nouveau handler renverra `{"ok": bool}` comme les routes M3, ou suivra le style local — choisir la cohérence locale du fichier et rester lisible). Read-only, aucune mutation.

**Panneau** : `admin-card#settings-budgets-card` dans System de `static/index.html` (après drift) ; `initBudgetsView()`/`loadBudgets()`/`renderBudgets(data)` dans `admin.js`, ajouté à `inits`. Affiche par ligne : nom agent, 3 barres/valeurs usage/limite (tokens, coût $, itérations), chip statut (`chip-ok` active / `chip-warn` seuil d'alerte atteint / `chip-bad` exhausted|paused). État vide honnête : « aucun budget actif » en neutre. `ksClean()` pour l'échappement.

**Tests** : `tests/test_governance_budgets_list.py` — contract : `GET /api/governance/budgets` → 200, `"budgets"` est une liste (éventuellement vide), chaque élément a les clés attendues si non vide.

### Unité 4 — Correction doc INVENTAIRE (clôt « Sub-projet E »)

Corriger la misclassification dans les markdown (aucun code touché) :
- `INVENTAIRE/INVENTAIRE-MAITRE.md:37,49,69` et `INVENTAIRE/A-INVENTAIRE-FONCTIONNEL.md:51` : `trace_writer` passe de `👻 FANTÔME`/`⚪ DOC-ONLY` à `🟢 CÂBLÉ-ACTIF` avec preuve (`src/trace_writer.py`, call-sites `agent_loop.py:1934,3584`, `tool_execution.py:542,630`).
- Noter que `memory_distill` (`phases.py:57`) est un **stub aspirationnel** non implémenté, inerte (forced_tools(MEMORY_OBSERVE) jamais appelé au runtime).

## Hors périmètre (YAGNI / honnêteté)

- **Autoeval** : ne rien ajouter. Le badge transitoire in-chat (`chat.js:2375`, event `autoeval_result`) est déjà la bonne UX — il n'apparaît que quand `ODYSSEUS_AUTOEVAL=on` a réellement quelque chose à dire. Un chip permanent serait muet/trompeur par défaut. `GET /api/autoeval/summary` existe mais un panneau serait low-signal ; reporté.
- Aucune activation de kill-switch. Aucun changement de défaut runtime.
- Aucune persistance nouvelle (drift/budgets lus à la volée).
- Pas de fusion des orchestrateurs (déjà hors périmètre en D).

## Flux de données

- Unité 1 : `agent_loop` calcule `phase_active` (lecture de deux flags déjà présents) → `run_status_event` l'inclut → SSE → `cockpit.update()` colore le chip. Aucun nouveau flux, un champ de plus.
- Unités 2-3 : nouveau `GET` read-only → `admin.js` fetch au `init` → rendu chips/barres. Même pattern que kill-switches (B) et Trinité (C).

## Gestion d'erreurs

- `run_status_event` reste enveloppé dans le `try/except` existant (2592-2597).
- Les nouvelles routes : async try/except → `{"ok": False, "error": ...}` (ou style local du fichier pour governance), jamais de 500 non géré.
- Le frontend : fetch en `.catch` → chip/état neutre « indisponible », jamais de valeur inventée.

## Tests

- `tests/test_sse_indicators.py` : `phase_active` présent et correct ; champs existants inchangés.
- `tests/test_observer_routes.py` : contract `/api/observer/drift`.
- `tests/test_governance_budgets_list.py` : contract `/api/governance/budgets`.
- Non-régression : `import app` boote ; `tests/test_orchestrator_phases.py` reste vert.

## Ordre d'exécution (séquentiel — fichiers partagés)

1. Unité 1 (fix honnêteté phase) — corrige un défaut déjà visible.
2. Unité 2 (drift).
3. Unité 3 (budgets).
4. Unité 4 (doc).

Les unités 2 et 3 touchent toutes deux `static/index.html`, `static/js/admin.js`, `app.py`/routes → **exécution séquentielle obligatoire** (pas d'agents parallèles en écriture).

## Critères de succès

1. Chip phase : neutre par défaut (orchestration OFF), vert seulement si `LIVE_ORCHESTRATION` ou `PHASE_TRACKER` ON. Valeur toujours affichée honnêtement.
2. Panneau Drift : lit le vrai `Observer.get_summary()`, niveau coloré fidèlement, pas de valeur fabriquée.
3. Panneau Budgets : liste les budgets réels, barres usage/limite, chip statut fidèle, état vide honnête.
4. Doc INVENTAIRE corrigée : `trace_writer` = actif, `memory_distill` = stub.
5. `import app` boote ; aucun kill-switch activé ; aucun changement de comportement runtime hormis l'ajout du champ `phase_active` (additif) et des routes read-only.
