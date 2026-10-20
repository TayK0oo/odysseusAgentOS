# ADR-0013 — La porte G1 devient exécutable : mort du faux vert

**Date** : 2026-10-21 · **Statut** : Accepté · **Décidé par** : agent P1-A1 (délégation, L5 worktree `/tmp/opencode/wt-A1`)
**Déclencheur** : mission « outiller la porte G1 » (P1-2) — `gate.py --phase 1` était un faux vert : un seul critère `kind="ci"` placeholder, jamais exécuté.

## Contexte

`omega/gates.toml` définissait G1 par un critère `id="G1"`, `kind="ci"`, note
« porte non outillée ». `gate.py` rend `kind="ci"` = `DELEGUE_CI`, qui **ne fait pas
échouer la porte** : `--phase 1` renvoyait VERT alors que rien n'était mesuré.
C'est exactement le défaut reproché au plan (leçon L4 : rouge d'abord ; « une porte
que la CI n'exécute pas est décorative », P0-7).

En parallèle, les sources de vérité existent mais ne sont pas branchées : le banc
d'acceptation (P0-4) et son runner boîte noire, la liste nominative vive de
`composition.rs` (parseée dans `STATUS.json` → `registre.plugins_attendus`),
l'endpoint `GET /api/plugins` du `omega-server`.

## Décision

1. **Phase1 de `gates.toml` est réécrite en critères exécutables et mesurables**, chacun
   avec sa citation du plan (`source = "…"`), tous bloquants :
   - `G1.a.s1` — S1 vert scripté, 5/5 runs `PASS` (`runner.py --runs 5 --attendu PASS`),
     plan §6 P1 : « S1 et S2 verts (scripté 5/5) » ;
   - `G1.b.s2` — S2 vert scripté, 5/5 runs `PASS` (limites/budgets, P1-3) ;
   - `G1.c.sequence` — séquence bornée S0→S1→S2, 5 runs, tous `PASS`
     (timeout = borne explicite du critère) ;
   - `G1.d.plugins` — registre vif `==` liste **nominative** (13 couples id+capability
     inscrits dans le critère, sous-tables `plugins` ; contrôle par
     `omega/acceptance/verifier_plugins.py` sur une instance jetable, idée
     `instance_jetable_http` de `measure.py` réutilisée via `engines.rust`, pas copiée) —
     plan §6 P1-2 : « la liste nominative … est dans gates.yaml » ;
   - `G1.e.orphelins` — détecteur d'orphelins : **outil non livré (arrive en P1-6)**.
     Le critère pointe la commande future avec `expect_exit = 0` et déclare
     `verdict_attendu = "ECHEC"` aujourd'hui (commande absente → 127 → `ECHEC`).
     Ce `ECHEC` est assumé, nommé, jamais remplaçable par un faux vert.
2. **Sémantique de verdict du runner étendue, pas remplacée** : sans `--attendu`,
   l'ancien accord PASS/XFAIL → exit 0 reste (les critères G0 de la phase 0 ne
   changent pas). Avec `--attendu V`, exit 0 implique que *chaque* run a rendu
   exactement `V` : le second degré XFAIL n'est acceptable que là où la porte le
   nomme.
3. **S2 = nouveau scénario** (limites/budgets, scripting ReAct texte canonique
   ADR-0006, `llm_scripte: react`) ; **S1 n'est pas renommé** : il est déjà le
   scénario outil réel + canaries (`outil_reel_canary` depuis 24d86a7c/abe4d890).
   Cf. `omega/acceptance/SCENARIOS-README.md`.

## Conséquences

- `gate.py --phase 1` devient **ROUGE sincère** dès aujourd'hui : S1/S2 sont XFAIL
  (boucle d'outils absente, P1-1 / ADR-0006 ; budgets absents, P1-3), le registre
  vif ne contient pas la liste attendue, le détecteur d'orphelins n'existe pas.
- Le faux vert est mort : tous les critères sont bloquants, plus aucun `DELEGUE_CI`
  ne couvre la porte tant que l'outillage n'existe pas.
- A2 (boucle), P1-3 (budgets) et P1-6 (orphelins) feront passer individuellement
  leurs critères — G1 ne devient verte que quand tous sont verts.
- La règle porte/code s'applique à `gates.toml` : sa modification reste hors du
  commit qui code ce qu'elle juge (commits séparés, vérifiés par
  `gate.py --regle-porte-code` via le test dédié).
- Noté pour le tournant A2 : quand la boucle existera, le `xfail_raison` de S1 devra
  être retiré (sinon `XPASS` = exit 1), et le critère G0.c.s1 de phase 0 (« S1 rouge »)
  devra être retiré avec lui — changement de phase G0 → G1 assumé.
