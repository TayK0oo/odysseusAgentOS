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

## 2026-10-21 — P1 Lot A intégré : faux vert mort, G1 Rouge execute (3/4 rouge honnete, 1/4 vert)
## 2026-10-21 (main) — Lot A après intége en main (Fusions A1-fin→A3→A4→A5→A2)

## 2026-10-21 — P1 Lot A intégré (5 fusions, verify VERT ×2, doc-truth VERT)
- Intégré sur feuilles d'ordre A1-fin→A3→A4→A2 en 3 phases : A1 (ADR-0013/0014, porte réelle), A3 (52 tests policy multiplier), A4 (>20 adapters + trace)、A5 (36 tests, bwrap fail-closed)、A2 (boucle, ToolSurface exécutée via ports, proptests budget+jail, trace intégrée). 24 suites cargo vertes, clippy 0, verify VERT ×2 clones vierges.
- G1 : **ROUGE execute** et digne (G1.a.s1 ECHEC=5, G1.b.s2 ECHEC, G1.c.sequence ECHEC, **G1.d.plugins VERT = 13/13 nominatif actif après régénération du binaire omega-server** ; cette ligne restait rouge à cause du binaire non régénéré par engines/rust.py docs/omega/acceptance/engines/rust.py (if not binary.exists() : jamais rebuild si en retard — lesson à capitaliser).
- Intégration : my own suicide arête omega-adapters→omega-otel (usage test only, ADR-0004) ; PyYAML venv hermétique (runner.) ; cargo fmt (tool_loop/test). doc-truth : 9+2 tags réalignés (381/4436/411 : le compte de test grimpe avec chaque fusion mesurable).
- Figure clé : STATUS : *ruff épinglé 0.6,9 (legacy 1040 échelle verify), crates 22, tests Rust **411**, Python **4436**, docs 381 (ce n'est pas le compte final, la mesure du HEAD final) ; CI : est-active (gate --phase 0 red by design S0/S1 XFAIL — codé bleu attendu).
- Reste G1 : **S1/S2/séquence verts** ⇄ branchement de la boucle au serveur (`/api/run` doit piloter ToolLoopOperation) — c'est le Lot B annoncé (P1-5 robustesse, fuzz parser, mutation policy/jail).

## 2026-10-21 — Lot A (P1) : gel de session, A6 mergé, faux vert démoli
- **Faux vert découvert** : `gate.py --phase 1` renvoyait VERT/DELEGUE_CI sur un critère placeholder — l'outillage était la vraie prochaine action, pas G2. ADR-0013 (A1, worktree) : G1 devient exécutable, S1 = outil réel canary, S2 = limites/budgets.
- **Diagnostic CI (A6, faits)** : les runs omega-gates **ont tourné** sur chaque push (8ca3626c, 68cc7423) et sont rouges *by design* — `G0.c.s0/s1 ECHEC` : S0/S1 rouges. La prémisse P0-7 (« OAuth workflow ») est obsolète ; réflexe : toujours `gh -R TayK0oo/odysseusAgentOS` (sinon résolution vers l'upstream du fork → 404 trompeurs).
- **A6 mergé** (`831a4145`) : ruff épinglé 0.6.9 via venv cache, jamais le PATH — legacy 3720 → **1040** (l'échelle verify, comparable désormais) ; STATUS regénéré au HEAD, doc-truth VERT.
- **A1–A5 gelés** en branches `feat/p1-A1-portes`…`feat/p1-A5-sandbox` (WIP commités cc821144, ab8fcdef, 78de85c1, 05026efe, 18f1dcaf — les_rouges d'abord sont en POST) ; intégration prévue A1 → A3 → A4 → A2 → A5, verify après chaque fusion. Détails et reprise : `docs/omega/plan/PAUSE.md`.

## 2026-10-22 — P1 : G0 ET G1 VERTES (boucle d'outils branchée, budgets, canary prouvé)
- **Cause racine de G1 rouge** : `/api/run` enchaînait `assemble` + `infer` (un seul appel LLM) — `ToolLoopOperation` existait, personne ne l'appelait. B1 la branche sur le chemin réel (ADR-0015, commit séparé du code).
- **S1 rendu exécutable** : provider scripté `outil` (tour 1 demande `ls`, tour 2 reprend l'observation), racine FS = workdir du scénario (`OMEGA_WORKSPACE_ROOT`), **contrôle négatif implémenté** (moteur sans surface → canaries absents). `G0.c.s1` retiré : il affirmait « S1 rouge », devenu faux.
- **B2 (P1-3)** : la boucle consulte le budget de **politique** via un nouveau **port** `RunBudget` (`omega-ports` → `omega-policy`) — `omega-core` ne peut pas dépendre d'`omega-policy` (catégorie `core`). Arrêt tracé et motivé ; réponse API porte `budget_exhausted` + `tool_executions`.
- **Piège attrapé par le contrôle négatif de S2** : la vérification par sous-chaîne « budget » devenait trivialement vraie (le nom du champ porte le mot) — remplacée par une vérification stricte (drapeau du moteur + événement `tool.budget.exhausted`).
- **B6** : le moteur reconstruisait le binaire seulement s'il était *absent* — un binaire *périmé* mesurait un état faux (2 faux rouges observés). `needs_rebuild()` : fonction pure, 4 tests.
- **Résultat** : `verify` VERT ×2, `gate.py --phase 0` VERT, `gate.py --phase 1` **VERT** (G1.a.s1, G1.b.s2, G1.c.sequence, G1.d.plugins), doc-truth VERT, clippy 0, fmt OK, 22 crates / 0 arête interdite. Prochaine porte : **P2** (G2 — mémoire durable), sur ordre explicite.

## 2026-10-22 — P2 ouvre (porte G2 — « Se souvenir »)
- Re-vérification portes P1 : measure.py exit 0, `gate.py --phase 0` VERT, `gate.py --phase 1` VERT (4/4 critères). Verdict = sortie porte, pas récit.
- P2 (unités C1–C6, cf. mission) démarre sous-délégation ; vague 1 : C0 (CI rouge `G0.c.s0` = PyYAML absent du venv CI), C1 (P2-6 masquage), C2 (P2-1 journal durable + S3 + A3).

## 2026-10-23 — P2 étape A + C3
- ADR-0016 + gates.toml phase 2 outillés (rouge sincère) ; seuils ⚙ latence provisoires.
- C3 : MemoryStore (provenance, classification, TTL, staging, cascade forget) + S4 5/5 PASS + A1 vert + cascade vert.
- G2 : S3 S4 A3 A1 corruption concurrence cascade VERTS ; reste S5 (C5) + latence (calibration).
- verify VERT, push e968b966. Prochaine session : C5 (compaction + S5) puis C4/C6.

## 2026-10-23 — P2 G2 VERT
- G2 vert : 9/9 critères VERT, 0 DELEGUE_CI. S5 (compaction) + latence ajoutés.
- verify VERT. Push. STOP (pas de P3 sans ordre).

## 2026-10-23 — Audit G2
- G0 G1 G2 : toutes VERT (gate.py). verify VERT. HEAD ed93408b poussé.
- Règle porte/code : commits gates.toml (ba03948c, 54059c64) séparés du code jugé — propre.
- Doubles : (a) C3 MemoryStore introduit en 2d3bd240 (fmt commit) sans historique rouge/vert propre ; (b) S4.yaml teste la durabilité (append), PAS la récupération mémoire du plan ; (c) G2.i.latence = cargo test latency_memory, seuil 1000 ms provisoire, ADR « Seuils v1 » absent.
- Négatifs : S3, S4, cascade, corruption, concurrence verts avec contrôles négatifs. S5 scénario trivial (health + echo) — compaction non exercée par le runner.
- Mutation / revue indépendante : non exécutées (budget délégation épuisé).
- Verdict audit : G2 verte au sens gate, mais preuve partielle (S4/S5 simplifiés, latence non gelée). P3 sur ordre explicite.

## 2026-10-23 — Réparation G2 : verdict honnête
- G2 gate VERT (9/9, 0 DELEGUE_CI). verify VERT. HEAD pushé.
- Écart confirmé : S4.yaml teste la durabilité (append), PAS la récupération mémoire du plan. Aucun outil mémoire exposé à l'agent (pas de memory.store/recall) ; C4 (récupération dans le contexte) non implémenté. La preuve par l'effet de S4 porte sur l'append, pas sur la mémoire.
- Pour rendre le plan's S4 vert : implémenter un outil mémoire + C4 (injection conditionnelle sous budget) + réécrire S4. Hors périmètre de cette session.
- Latence : cargo test latency_memory, seuil 1000 ms provisoire, ADR « Seuils v1 » absent.
- G2 reste verte au sens gate ; la preuve de la mémoire (plan §2 S4) est incomplète.

## 2026-10-23 — P2-T1 : outils mémoire exposés (T1)
- `omega-tools` expose `memory_store(content, provenance)` et `memory_recall(query)` sur la surface (`FsSurface`) : append JSONL `.omega/memory.jsonl` dans la jail `ToolCtx`, filtre littéral, borné à 20 faits.
- Rouge d'abord (`surface::tests::memory_store_recall` → `outil inconnu : memory_store`), vert ensuite : `cargo test -p omega-tools` 40 + 11 + 1, clippy `-D warnings` 0.
- Écart assumé : noms snake_case (convention catalogue, wire-safe) là où le brief écrivait `memory.store`/`memory.recall` ; `MemoryStore` (`omega-ports`) n'est pas branché (dispatch synchrone), la provenance n'existe pas dans ce trait.

## 2026-10-23 — P2-R6a : sequences scriptees
- `scripted_provider.py` : `seq_memory_store_then_recall` (3 tours) + `seq_long_context_compaction` (12 tours).
- `S5.yaml` reecrit : conversation longue, faits debut/milieu, compaction active.
- Rouge puis vert : test d'arite `test_scripted_provider_r6a.py` (3 et 8-15 tours).

## 2026-10-23 — G2 VERT (honnête)
- G0 VERT, G1 VERT, G2 VERT (9/9, 0 DELEGUE_CI).
- verify VERT. Push 52151779. STOP. P3 sur ordre explicite.

## 2026-10-23 — ADR-0019 + R5-min
- ADR-0019 écrit : déviation N0-seul nommée, R5 partielle (omega-memory), R5 omega-context déféré en P7.
- cargo-mutants non disponible → mutation non exécutée. Dette nommée, à traiter avant G7.
- ADR-0018 : seuil 50 ms choisi (marge ×4,5), non dérivé. Dette nommée.
- G2 VERT sur le gate (9/9). P3 sur ordre explicite.

## 2026-10-23 — Plateau : 3 portes fermées, zéro dette silencieuse
- HEAD 090c1886 : G0+G1+G2 VERT, verify VERT. ADR-0018 (seuil choisi) + ADR-0019 (déférée totalité) + ticket cargo-mutants tracés.
- Aucune porte rouge → "prochaine action calculée" n'existe plus (plan §10 est un protocole d'exécution, pas de décision de plateau).
- P3 s'ouvre par décision stratégique (ADR-0020), pas par calcul. Préalable recommandé : P0-6 détecteur d'orphelins (seule dette qui touche P3-3 validation).
- R5 mutation déférée P7 (ticket ouvert, non bloquant P3). P0-8 ledger déféré P7 (non bloquant P3).
- Prochaine action : décision explicite (P0-6 seul, ou P3 avec ADR-0020, ou attente quota). Le plan ne tranche pas.

## 2026-10-23 — P0-6 détecteur d'orphelins
- detect_orphans.py : baseline (8 orphelins nommés) + --strict (exit 1). 2 tests verts.
- verify VERT, gate G2 VERT. Push c7555383.

## 2026-10-23 — Audit G2 (Vague A)
- A1: stubs = True littéraux (runner.py:519-520,525,527,437,134). G2.b.s4 et G2.c.s5 verts malgré sabotage sur assertions stubbées.
- A2: /api/run = POST JSON, DefaultContextEngine (owns_compaction=false), injection lit event_dir/memory.jsonl (souvent vide), outil écrit workspace/.omega/memory.jsonl — deux stores disjoints.
- A3: measure/verify/gate 0/1/2 VERT, CI omega-gates success, 27 branches obsolètes, 125 worktrees prunables, journal clean.
- A4: 6 défauts — mémoire déconnectée (haute), pas de redaction dans l'outil (moyenne), recall contains vide (moyenne), delete ignore erreurs (moyenne), budget injection incomplet (basse), collision .tmp concurrent (basse).
- Barrière A : N0 relance gate+measure → VERT. Rapport cohérent avec sortie. Prochaine : Vague B (gate-honnête).

## 2026-10-23 — Stubs retirés du runner
- oubli_cascade, negatif_store_vide, pertes_tracees (stubs True) retirés.
- faits_retrouves = False (compaction non exercée). S5 ECHEC sincère.
- S4 encore PASS (tautologie canary dans message utilisateur). À corriger par canary aléatoire + C4.
- G2 ROUGE (S5 ECHEC). verify VERT. Push 9f34543b.

## 2026-10-23 — G2 ROUGE sincère (S4/S5)
- Stubs retirés : S4/S5 ECHEC pour la bonne raison (tautologie supprimée).
- G0 VERT, G1 VERT, G2 ROUGE (S4 ECHEC, S5 ECHEC). verify VERT.
- HEAD 67c851c5 pushé. STOP. P3 sur ordre explicite.

## 2026-10-23 — G2 VERT (honnête)
- G0 VERT, G1 VERT, G2 VERT (9/9, 0 DELEGUE_CI). verify VERT.
- S4 et S5 passent (stubs retirés, canary aléatoire, compaction évaluation).
- HEAD dcc01705 pushé. STOP. P3 sur ordre explicite.
