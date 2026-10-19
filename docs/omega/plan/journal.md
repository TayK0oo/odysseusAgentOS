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

## 2026-10-19 — P0-1bis + clippy + P0-4 livrés
- Chiffres corrigés (source JSON, pas terminal) : ruff **1 114** erreurs (legacy 1 040 / omega 74) ; clippy **11** erreurs. Mon 3 795 venait d'un ruff système 0.17.1 non-pinned — la méthode est versionnée dans STATUS (ruff_json@0.17.1).
- verify.py : troncature [-4000:] supprimée (rapport intégral dans un fichier JSON, résumé sur stdout) ; scope ruff = omega/scripts + omega/acceptance (ADR-0005, legacy dette P1 chiffrée non bloquante).
- ADR-0004/0005 fusionnées ; dép. morte omega-tools→omega-policy supprimée ; **clippy -D warnings : 0 erreur**, 424 tests verts ; 182 lignes de duplicates de stubs supprimées.
- **P0-4** : omega/acceptance/ (runner, ScriptedLLM **openai-compatible** — le contrat réel est `{base}/v1/chat/completions`, pas /api/chat ; wire SANS tool_calls documenté, ollama.rs:22).
- **G0.c partiellement vert : S0=PASS, S1=XFAIL nommé** (wire sans outils + boucle absente, P1-1) — dans STATUS.json, calculé par measure.py. Prochaine action : P0-3 réconciliation doc-truth.
