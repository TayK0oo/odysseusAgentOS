# Journal (append-only, 5 lignes max par session)

## 2026-10-19 — P0 démarre
- PLAN-MAITRE.md v1 écrit à `docs/omega/PLAN-MAITRE.md` (brièvement commité au mauvais dépôt puis déplacé ici — seul dépôt ayant les 2 moteurs).
- `omega/scripts/measure.py` écrit (P0-2) : cargo metadata, scan attributs Rust, AST Python → `STATUS.json` + `STATUS.md`.
- Mesure 1ère : 22 crates ✓ ; 370 attributs test Rust (≠ 424/293 documentés) ; 4 415 fonctions test_ Python (≠ 5 068 tests verts annoncés).
- Porte fédérée `check_dep_graph.py` : ÉCHEC — arêtes interdites réelles (omega-search→omega-ports/omega-types, omega-python-bridge→omega-types, omega-tools→omega-policy). Dette réelle détectée.
- Prochaine action : P0-1 (verify) puis P0-4 (banc d'acceptation S0/S1).

## 2026-10-19 — P0-1 livré, ADR-0004 tranchée
- `omega/scripts/verify.py` (P0-1) : worktree jetable → venv figé → ruff → pytest → fmt/clippy/test Rust → dep_graph → model_policy. Écart honnête : comparaison de version Python en tuple, pas substring.
- Décision D-ADR-0004 : **Policy reste un port** (traits `ExecPolicy`/`PermissionPolicy` déjà dans `omega-ports`, rien à extraire — enquête a évité un refactor superflu). 5 crates recatégorisées en `adapters` ; règle `config: {types, ports}` (omega-config implémente `ConfigSource`) ; **dép. morte `omega-tools → omega-policy` supprimée** (0 référence).
- `dep_graph` : 10 violations → **0**. `verify` 1er run : ROUGE (ruff 3 795 socle Python legacy, fmt 172 diffs, clippy 33) — baseline JSON à `docs/omega/plan/baselines/verify-2026-10-19.json`.
- `cargo fmt --all` appliqué (0 diff restant). Restants : ruff (3 795, dette socle Python P1+), clippy (33), pytest + cargo_test jamais mesurés dans verify complet.
- Prochaine action : P0-4 banc d'acceptation (S0 vert, S1 xfail nommé) — ruff/clippy bascule en dette P1, pas bloquants.
