# Orchestrator Consolidation — Design (Sub-projet D, scope A)

**Date:** 2026-07-08
**Statut:** Validé, prêt pour planification
**Nature:** Refactor runtime — **flaggé risqué**, donc portée volontairement minimale et **sans changement de comportement**.

## Contexte et correction d'une hypothèse

L'audit initial supposait un `src/llm_router.py::ModelRouter` mort à supprimer. **Vérification faite : ce module a déjà été retiré au jalon M3.1** (`src/__init__.py:3`, `src/llm_core.py:3-4`, `tests/test_harness_core.py:156`). De plus `model-routing.json` **n'est pas** un fantôme : il est activement lu par `src/zen_router.py:27`. Le volet « suppression de code mort » de la consolidation est donc **déjà fait**. Il ne reste qu'une seule duplication réelle.

## La duplication restante

Le live loop `stream_agent_loop` (`src/agent_loop.py`) contient **deux systèmes d'inférence de phase**, sélectionnés par un `if/else` sur `ODYSSEUS_LIVE_ORCHESTRATION` :

- **OFF (défaut)** : `PhaseTracker` (`src/orchestrator/phase_tracker.py`) — inférence conservatrice (`BUILD`, ou `PLAN` en plan_mode).
- **ON** : `CanonicalLoop` (`src/orchestrator/loop.py`) — parcourt la séquence 7 phases CLASSIFY→…→MEMORY_OBSERVE.

Les deux écrivent la phase dans le même registre phase-lock. **Ils sont mutuellement exclusifs** (`agent_loop.py:2554-2557` pour l'avance, `2562-2572` pour la lecture) : aucune course réelle aujourd'hui. Le vrai problème est double :
1. **Zéro test de caractérisation** sur ces deux modules — le comportement n'est pinné nulle part.
2. **La logique de résolution de phase est inline** dans le live loop (`2562-2572`), dupliquée conceptuellement entre les deux branches.

## Objectif (portée A, sûre)

Transformer ce point chaud non testé en un point testé avec une couture unique, **sans changer quel orchestrateur tourne, ni le comportement runtime**. Deux unités.

## Principe directeur : préservation du comportement

- Aucune modification de la valeur de retour observable du live loop.
- Aucune modification des défauts de kill-switch (`ODYSSEUS_LIVE_ORCHESTRATION` reste off, `ODYSSEUS_PHASE_TRACKER` reste off).
- L'extraction de couture est un **extract-function strict** : la fonction renvoie exactement la même `Phase` (ou `None`) que le code inline actuel, pour les mêmes entrées.
- Les tests de caractérisation pinnent le comportement **actuel** (pas un comportement souhaité). S'ils échouent, c'est un bug du test, pas du code.

## Architecture

Aucun nouveau backend UI, aucun endpoint. Deux fichiers touchés + un nouveau module pur + un nouveau test.

```
src/orchestrator/phase_resolver.py   (NOUVEAU — module pur)
        │  resolve_current_phase(...) → Phase | None
        ▼
src/agent_loop.py:2562-2572          (remplacé par un appel à la couture)
        ▲
tests/test_orchestrator_phases.py    (NOUVEAU — caractérisation)
        │  pinne PhaseTracker, CanonicalLoop, resolve_current_phase
```

### Unité 1 — Tests de caractérisation (additif, risque nul)

Nouveau `tests/test_orchestrator_phases.py`. Pinne le comportement observable **actuel** :

**PhaseTracker** (`src/orchestrator/phase_tracker.py`) :
- `PhaseTracker.infer_phase(round_num, plan_mode=False)` → `"BUILD"` (quel que soit `round_num`).
- `PhaseTracker.infer_phase(round_num, plan_mode=True)` → `"PLAN"`.
- `tracker_enabled()` → `False` par défaut ; `True` si `ODYSSEUS_PHASE_TRACKER` truthy (via `monkeypatch.setenv`).
- Avec un registre factice injecté + `enabled=True`, `on_round_start(1, plan_mode=True)` appelle `registry.set_phase(session_id, "PLAN")` ; avec `enabled=False`, `set_phase` **n'est jamais** appelé.

**CanonicalLoop** (`src/orchestrator/loop.py`) :
- `current` initial → `Phase.CLASSIFY`.
- `advance()` fait progresser dans `CANONICAL_SEQUENCE` ; 6 `advance()` mènent à `Phase.MEMORY_OBSERVE`.
- Un 7ᵉ `advance()` renvoie `False` et laisse `current == Phase.MEMORY_OBSERVE` (borne haute).
- `goto(Phase.QUALITY)` avec un registre factice → `current == Phase.QUALITY` et `registry.set_phase(session_id, "QUALITY")` appelé.
- `apply()` sans registre → no-op (aucune exception).

### Unité 2 — Couture de résolution de phase (extract-function strict)

Nouveau module pur `src/orchestrator/phase_resolver.py` :

```python
"""Pure resolution of the live loop's current canonical Phase.

Behavior-preserving extraction of the inline branch previously in
stream_agent_loop (src/agent_loop.py). No side effects: reads only.
"""
from __future__ import annotations
from typing import Optional
from src.orchestrator.phases import Phase


def resolve_current_phase(use_canonical, canonical_loop, phase_tracker,
                          round_num, intent=None, plan_mode=False) -> Optional[Phase]:
    """Return the canonical Phase for this round, or None.

    Mirrors exactly the prior inline logic:
      - canonical active → the loop's current Phase;
      - else PhaseTracker → its inferred phase name coerced to Phase (None if
        the name is not a valid Phase value);
      - else None.
    """
    if use_canonical and canonical_loop is not None:
        return canonical_loop.current
    if phase_tracker is not None:
        pt_phase = phase_tracker.infer_phase(round_num, intent=intent, plan_mode=plan_mode)
        try:
            return Phase(pt_phase) if pt_phase in {p.value for p in Phase} else None
        except Exception:
            return None
    return None
```

Puis dans `src/agent_loop.py`, le bloc inline `2562-2572` est **remplacé** par :
```python
from src.orchestrator.phase_resolver import resolve_current_phase
_current_phase = resolve_current_phase(
    _use_canonical, _canonical_loop, _phase_tracker,
    round_num, intent=_intent, plan_mode=plan_mode,
)
```
La ligne `_current_phase = None` d'initialisation (2562) et les lignes 2563-2572 disparaissent au profit de cet appel. Le reste du live loop (avance à 2554-2557, dispatch agents à 2573+) est **inchangé**.

**Tests de la couture** (dans le même `tests/test_orchestrator_phases.py`) :
- `use_canonical=True` + `CanonicalLoop` positionné sur `Phase.PLAN` → renvoie `Phase.PLAN`.
- `use_canonical=False` + `PhaseTracker`, `plan_mode=True` → renvoie `Phase.PLAN`.
- `use_canonical=False` + `PhaseTracker`, `plan_mode=False` → renvoie `Phase.BUILD`.
- `phase_tracker=None` et `canonical_loop=None` → renvoie `None`.
- Un `PhaseTracker` factice dont `infer_phase` renvoie une valeur hors-enum (ex. `"WAT"`) → renvoie `None` (pas d'exception).

## Flux de données

Aucun changement de flux runtime : la couture renvoie la même valeur que le code qu'elle remplace. Le dispatch d'agents (`agent_loop.py:2573+`) consomme `_current_phase` exactement comme avant.

## Gestion d'erreurs

- `resolve_current_phase` ne lève jamais : la coercition hors-enum est enveloppée en `try/except → None`, identique au comportement inline actuel.
- Le live loop conserve son `try/except` existant autour de l'avance de phase (2558-2559).

## Tests

- `tests/test_orchestrator_phases.py` (nouveau) : caractérisation PhaseTracker + CanonicalLoop + `resolve_current_phase` (cas ci-dessus).
- Non-régression : `import app` boote proprement.
- Garde comportement : `rtk git diff` sur `src/agent_loop.py` doit montrer **uniquement** le remplacement du bloc `2562-2572` par l'appel à la couture — aucune autre ligne du loop modifiée.

## Hors périmètre (YAGNI / trop risqué — reporté en B/C)

- Fusionner `PhaseTracker` et `CanonicalLoop` en une seule implémentation.
- Changer l'orchestrateur par défaut / supprimer `PhaseTracker` / retirer le `if/else`.
- Toucher aux hooks post-loop (autoeval, autoevolve, checkpoint, ancestry, codeburn) ou au `gate`.
- Toute suppression de code (le mort a déjà été retiré en M3.1).
- Toute UI (D est un refactor interne ; la visibilité des switches est déjà couverte par B).

## Critères de succès

1. `tests/test_orchestrator_phases.py` au vert : le comportement actuel des deux systèmes de phase est pinné.
2. `src/orchestrator/phase_resolver.py` fournit `resolve_current_phase`, testé, sans effet de bord.
3. `src/agent_loop.py` utilise la couture ; le diff ne touche que le bloc `2562-2572` (extract-function strict, comportement identique).
4. `import app` boote ; aucun défaut de kill-switch modifié ; aucun changement de comportement runtime observable.
