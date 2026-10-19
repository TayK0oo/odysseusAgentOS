# ADR-0005 — Scope `ruff` de `verify` : Python nouveau d'abord, legacy dette P1

**Date** : 2026-10-19 · **Statut** : Accepté · **Décidé par** : chef de projet
**Contexte** : `verify` (P0-1) exige `ruff check .` vert pour G0.a. Mesure JSON (ruff 0.6.9 pinned) : **1 114 erreurs**, dont 1 040 dans le socle Python legacy (`src/`, `services/`, `core/`, `routes/`, `tests/`) et 74 dans `omega/`.

## Constat (observed)

Un `ruff check .` sur tout le repo inclut le socle Python pré-existant à la
session Omega, écrit avant les règles ruff en vigueur. Le réparé intégral est
un chantier L qui n'entre pas dans P0. Deux écueils interdits :

- **A** réparer 1 040 lignes legacy dans P0 — hors périmètre, effort L.
- **B** exclure le legacy **sans trace** — gaming de porte (R2), rejeté.

## Décision

1. `verify.py` scope `ruff` au **Python nouveau** : `omega/scripts/` +
   `omega/acceptance/`. Commentaire dans le code citant cet ADR.
2. Le socle legacy est déclaré **dette P1**, **visible et suivie** : `STATUS.json`
   porte `socles.python.ruff_legacy_erreurs` (mesurée par `measure.py` avec le
   ruff épinglé, valeur `null` si ruff absent — jamais inventée).
3. Le rapport `verify` mentionne le scope. Toute passe future de `S0`… `S11`
   peut exiger l'inverse (legacy scruté) — c'est la porte du banc qui déciderait.
4. **Lever de dette écrit** : quand `ruff_legacy_erreurs` = 0, ou un
   futur ADR « P1.5 », le scope s'élargit au repo entier sans retouche de
   `verify.py` (la valeur dans `STATUS.json` redevient bloquante via G1).

## Conséquences

- G0.a redevient atteignable sans gaming : la restriction est **écrite, mesurée et datée**.
- Les 74 erreurs du Python nouveau : traitées en P0 (fait le jour même, `ruff check omega/scripts` vert).
- Le legacy reste sujet à ses ci-origines ; on ne régresse pas dessus en silence (le compteur peut **monter** : c'est une alerte, pas du bruit).
