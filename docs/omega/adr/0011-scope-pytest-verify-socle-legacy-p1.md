# ADR-0011 — Scope `pytest` de `verify.py` : Python nouveau bloquant, socle legacy dette P1

**Date** : 2026-10-20 · **Statut** : Accepté · **Décidé par** : chef de projet
**Prolonge** : ADR-0005 (scope `ruff`) · **Travail** : P0-1 (G0.a)

## Constat (observed)

Première exécution **complète** de `verify.py` (2026-10-20, SHA `e2757ccb`,
worktree jetable). Verdict : `ECHEC(pytest)`. Les neuf autres étapes sont
vertes :

| Étape | Verdict |
|---|---|
| `ruff` (scope ADR-0005) | VERT |
| `cargo fmt --check` | VERT |
| `cargo clippy -D warnings` | VERT |
| `cargo test --workspace` | VERT (les 22 crates) |
| `dep_graph` | VERT (0 cycle, 0 arête interdite) |
| `model_policy` | VERT |
| `doc_truth` | VERT |
| `pytest` | **ECHEC** |

La cause n'est pas un test rouge : c'est un **échec de collecte**. Le venv
hermétique de `verify.py` n'installe que `PINNED_PY_DEPS = ["ruff==0.6.9",
"pytest==8.3.3"]`, alors que la suite legacy importe le socle :

```
tests/test_active_document_clear.py → src.agent_tools → src.ai_interaction
  → core.database → E ModuleNotFoundError: No module named 'sqlalchemy.engine'
```

Le socle Python n'est installable dans aucun environnement de cette machine :
ni `/usr/bin/python3`, ni le venv `omega-re/tools-venv`, ni le venv de `verify`
n'ont `fastapi`/`sqlalchemy`. L'état réel de la suite legacy n'est donc pas
« rouge » — il est **inconnu**.

## Décision

`verify.py` exécute `pytest` en **deux étapes**, symétriques du scope `ruff`
d'ADR-0005 :

1. **`pytest`** — scope `tests/test_omega*.py` (le Python nouveau). **Bloquant.**
2. **`pytest_legacy`** — scope `tests/` moins le scope ci-dessus. **Non
   bloquant.** Si le socle n'est pas installable, l'étape est
   `SKIPPED(prerequis absents …)` avec la raison en clair — **jamais `PASS`**
   (cas limites du plan §6 : « un prérequis manquant donne `SKIPPED(raison),
   jamais `PASS` »).

Le verdict global ne compte que les étapes bloquantes. L'étape legacy reste
**visible** dans le rapport et dans la sortie : un `pytest_legacy` rouge ou
skippé ne ferme pas la porte, mais il se voit.

## Conséquences

- **G0.a peut devenir vert** sur le code réellement nouveau (Rust + Python
  omega), qui est vert sur ses neuf étapes.
- **La suite legacy reste non mesurée.** Le chiffre « 5 068 tests verts » des
  sprints précédents, repris dans les documents, **n'est pas reproduit** sur
  cette machine : `STATUS.json` ne mesure que `4 415` *fonctions* `test_*`
  par AST (une méthode d'inventaire, pas une exécution). Mesurer la suite
  legacy exige un venv dédié avec `requirements.txt` (Chroma, Mem0, Docker,
  clés API) — c'est **P1-0**, hors G0.
- **Limite assumée** : tant que P1-0 n'a pas produit un nombre, la porte G0.a
  ne prouve rien sur le socle legacy. Le jour où le socle est installable, le
  scope legacy redevient **mesuré et bloquant** au même titre que `ruff`
  legacy, qui est aujourd'hui une dette chiffrée (3 720 erreurs).

## Alternatives rejetées

- **Installer les dépendances du socle dans le venv hermétique de `verify`** —
  rejeté pour l'instant : `requirements.txt` tire Chroma, Mem0, Docker et des
  clés API ; le plan qualifie exactement ce cas de « prérequis impossible à
  installer », donc `SKIPPED(raison)`. L'installateur de `verify` deviendrait
  lui-même une porte (installations non reproductibles).
- **Laisser `pytest` legacy bloquant** — rejeté : G0.a resterait rouge en
  permanence, la porte perdrait tout pouvoir de discrimination.
- **Supprimer la suite legacy** — interdit : destructif, et la suite est la
  mémoire du socle qui marche.

## Provenance

- [observed] Verdict `ECHEC(pytest)` et sortie de collecte :
  `/tmp/verify-e2757ccb-184515.json` (2026-10-20, SHA `e2757ccb`).
- [observed] `PINNED_PY_DEPS` : `omega/scripts/verify.py:38`.
- [inferred] Le socle legacy serait vert si ses dépendances étaient
  installées : **non vérifié**, aucune mesure ne le soutient aujourd'hui.
