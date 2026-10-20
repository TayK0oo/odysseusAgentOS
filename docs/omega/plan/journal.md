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

## 2026-10-19bis — ADR-0006 + P0-3a
- Découverte structurante S1 : le wire ne transporte pas de tool_calls (`WireMessage {role, content}`). ADR-0006 : décision A+C (schéma étendu optionnel + repli ReAct, une seule décision) ; P1-1 rescopé L → XL (règle 11 déclenchée).
- `STATUS.json` v2 : 10 chiffres contestés mesurés avec méthodes déclarées — dissections 51, fonctionnalités 910 (parse JSON, pas grep), census 67, panneaux UI 16 (union `type Panel`), agents 20, clones 62, socle Python `present;cmd=app.py`, plugins_enregistres **5** (instance jetable HTTP), plugins_attendus INCONNU(gates_yaml P0-5). Le faux `capacites=0` (champ absent) remplacé par la clé réelle `omega_capability` = 102 uniques.
- Erreur capitalisée : un `git checkout --` a effacé la v1 de mesure avancée (non commitée) — v2 réécrite en module séparé `measures_advanced.py`, testable isolément. `git commit` intermédiaires désormais systématiques.
- Prochaine action : P0-3b `check_doc_truth.py` (tolérant d'abord, contrôle négatif obligatoire), puis P0-3c marquage.

## 2026-10-19ter — P0-3b livre (doc-truth, convention A)
- `check_doc_truth.py` + `tests/test_omega_doc_truth.py` : 4 tests, dont le contrôle négatif (tag faux -> exit 1) et la tolérance de `verify` prouvée dans les deux sens ; `verify` étape 10 `"bloquant": false` jusqu'à P0-3c. ADR-0007 tranché (A, tag auto-suffisant).
- Écart ruff levé : `3720 = 3729 (ruff check .) − 9 (omega/)` avec le ruff **système 0.17.1** [observed] — la valeur publiée se reproduit exactement. Le `1 114` venait du **0.6.9 épinglé**, absent du disque ici : non rejouable [inferred]. Ce n'est donc pas un scope élargi, c'est une version — et ADR-0005 promet « le ruff épinglé » alors que `measure.py` prend le ruff du PATH.
- Ticket `measure.py` v3 : un seul ruff (épinglé) pour `measure` et `verify` ; `registre` canonique peuplé depuis `avancee` en fin de run (Annexe C), `avancee` reste l'atelier — plus deux endroits pour la même information.
- Fait consigné : sous 0.17.1 le scope de `verify` (`omega/scripts` + `omega/acceptance`) a **4** erreurs (C420, PLC0415 ×2, PLC0207) [observed] ; sous 0.6.9 le journal le donne vert [inferred] — le pin décide donc du verdict de G0.a.
- Deux exemples littéraux de tag restent en WARNING (`PLAN-MAITRE.md:140`, `REPRISE.md:27`) : P0-3c les réécrit, aucune exception codée dans l'outil (ADR-0007 §6). Prochaine action : P0-3c (marquage), puis `--strict` pour G0.b.

## 2026-10-19quater — P0-3b-bis + P0-3c : `--strict` VERT(0)
- Fence/span-awareness (ADR-0007 §6 amendé) : les deux exemples littéraux bloquants étaient des **spans inline**, pas des fences — la règle couvre les deux formes de code, par sémantique Markdown et non par exception. 4 tests, dont le contrôle dans les deux sens.
- P0-3c : `PROJET-ET-VISION` marqué (18 tags ; 109→102 capacités, 424→370 tests Rust, 5 068 « tests verts »→4 415 fonctions recensées, « 13 entrées » et « 0,4 % » retirés car non mesurés), bannières sur `VISION-INTEGRATION`/`RECAP-COMPLET`/`INDEX`/`MEMORY`, `02-WORKSTREAMS` daté, `RULES`/`MODEL-POLICY`/`00-VISION` audités sans tag (aucun chiffre mesuré par STATUS).
- `--strict` par défaut, `verify` bloquant, mécanisme de tolérance supprimé. Limite écrite : le validateur prouve les tags, pas l'absence de chiffres non marqués. Reste ouvert : `measure.py` v3 (`*_count` pour les listes, un seul ruff épinglé). Prochaine action : G0.a (verify complet ×2).

## 2026-10-20 — P0-5 livré, G0 verte, D0 tranché
- P0-5 : `omega/gates.toml` (TOML/tomllib, 0 dépendance) + `omega/scripts/gate.py` (command/file/ci, rapport JSON, règle porte/code) + `tests/test_omega_gate.py` (6 tests, contrôle négatif).
- `measure.py` v3 : `prochaine_action` et `portes` ne sont plus écrits en dur — déduits des rapports gate.py ; `registre.plugins_attendus` devient mesuré (parse de `composition.rs`).
- G0.a : première exécution **complète** de `verify` = `ECHEC(pytest)` — le venv hermétique n'installe que ruff+pytest, le socle legacy n'est pas collectable → ADR-0011 (pytest omega bloquant, legacy = dette P1 non bloquante, `SKIPPED(raison)` jamais `PASS`).
- Bug réel trouvé par la porte : `omega-mcp` test instable (course sur la variable globale `OMEGA_REA_ENV` entre tests Rust parallèles) → correctif à la racine (`load_toolchain_env_from` / `rea_with_toolchain_env` purs, plus aucune variable mutée par un test).
- G0 : **VERTE** (G0.a, G0.b, G0.c, G0.d). ADR-0008 (P0 dépasse sa boîte de temps : P0-6 → P1-6, P0-8 → P7), ADR-0009 (**D0 = moteur Rust**, O2, ADR-0001 confirmé). Reste G0.e : la CI doit jouer `gate.py --phase 0` (job `omega-gates` poussé).

## 2026-10-20 — session reprise (contexte recréé depuis journal + STATUS)
- STATUS régénéré au HEAD `8ca3626c` — le regen de 22:53 (G0 VERT) n'avait jamais été commité, et son SHA embarqué précédait les 4 derniers commits (ADR-0012 inclus).
- `measure.py` : `moteur_reference` n'est plus une chaîne figée — dérivé du titre d'ADR-0009 (principe 7, même famille que le bug `prochaine_action`) → `Rust (ADR-0009)`.
- Prochaine action : P0-3d (`generate_index.py`, non bloquant, resté en P0 par ADR-0008), puis fermeture de P0 → G1 (`gate.py --phase 1`).

## 2026-10-20bis — P0-3d livré (INDEX + MEMORY générés, sous-agents en parallèle)
- `generate_index.py` créé : sections machine d'INDEX réécrites via balises `<!-- gen:adr|scripts|rust -->` (12 ADR, 18 scripts, crates mesurés), le reste de la carte reste écrit à la main ; bannière calculée (SHA, [observed]). Idempotence prouvée (2 runs, md5 identique).
- `generate_memory.py` réécrit : chiffres RE/avancée depuis `STATUS.json` (102/67/51/910, plus aucun « 109/66 » en dur), `cargo test` interne supprimé (370/4425 lus du STATUS), bannière auto à HEAD, `shell=True` remplacé par argv.
- Contrôles : doc-truth VERT, 4/4 tests du validateur verts. F541 (f-string sans placeholder) introduit puis corrigé à la racine — pris par ruff 0.6.9 épinglé ET système, contrôle négatif vérifié.
- Reste en P0 : rien de bloquant ; P0 fermable, prochaine porte G1 (`gate.py --phase 1`).
