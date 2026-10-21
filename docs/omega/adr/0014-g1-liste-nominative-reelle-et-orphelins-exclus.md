# ADR-0014 — Ajustements d'implémentation pour la porte G1 (suite d'ADR-0013)

**Date** : 2026-10-21 · **Statut** : Accepté · **Décidé par** : agent P1-fin-A1
**Peaufine** : [ADR-0013](0013-porte-g1-outillage-reel.md) (décision d'implémentation jamais exécutée avant cette mission — deux écarts constatés au sol, justifiés ici).

## Contexte

L'implémentation d'ADR-0013 (critère `G1.d.plugins`, exclusion demandée des
orphelins) a montré deux faits observés : (1) le parse brut de `composition.rs`
utilisé par `measures_advanced.py::_adaptateurs_attendus` attrape aussi les
appels `register(...)` des modules `#[cfg(test)]` — les couples `cfg`/`a`/`b`/
`x`/`y` de tests unitaires Rust sont ainsi comptés « adaptateurs attendus » dans
`STATUS.json` ([observed] : `registre.plugins_attendus.valeur` = 13 couples dont
5 de test) ; (2) les orphelins ne sont pas outillés (P1-6 reporté), et
`gate.py` ne connaît ni `verdict_attendu` ni aucun mécanisme de « succès par
échec déclaré » — reprendre un champ en dehors du juge existant serait inventer
l'interface (leçon L2).

## Décision

1. **Liste nominative = 13 couples = {5 câblés vifs au HEAD} ∪ {8 adaptateurs de
   production de `compose_ported_crates`}.** Câblés observés sur une instance
   jetable au HEAD (`GET /api/plugins`) : `ollama/llm-provider`,
   `default/context-engine`, `mcp:rea/tool-surface`, `file/event-store`,
   `embedded/harness-catalog`. Adaptateurs de production extraits sans le bruit
   `#[cfg(test)]` : `config/default`, `loop/default`, `workflow/default`,
   `agentstore/default`, `ui/main`, `ui/sidebar`, `sandbox/default`,
   `skills/default`. Le critère VERT ssi égalité exacte du registre vif avec
   cette liste. Justification : la liste nominative doit être « ce que le code
   de production sait construire, plus ce qui est câblé » — pas un artefact de
   tests unitaires Rust. Le nombre 13 d'ADR-0013 est conservé par coïncidence
   (5 câblés + 8 adaptateurs), pas par fidélité au false positif de
   `STATUS.json`.
2. **Les orphelins sont EXCLUS de [phase1]**, avec commentaire explicite
   dans `omega/gates.toml` : aucun critère ne les juge tant que le détecteur
   (P1-6) n'est pas livré, car un critère pointant une commande inexistante
   serait un ECHEC permanent à la raison sémantiquement fausse (« outil
   introuvable » portant la porte au lieu du vrai constat orphelin). À la
   livraison du détecteur : un nouveau critère `kind="command"` exécutant le
   vrai outil avec `expect_exit = 0` sera ajouté, comme les autres. L'item 5
   d'ADR-0013 (critère `G1.e.orphelins` avec `verdict_attendu = "ECHEC"`) est
   le seul point amendé ; le mécanisme `verdict_attendu` n'existe pas dans
   `gate.py` et ne sera pas inventé pour lui.

## Conséquences

- `gate.py --phase 1` est rouge aujourd'hui sur 4 critères bloquants nommés
  (S1, S2, séquence, registre) — jamais sur un critère d'instrumentation
  manqué.
- Le critère `G1.d.plugins` fait porter la liste nominative dans `gates.toml`
  (plan §6 P1-2 : la liste est dans la porte, pas dans le script) ; le script
  de contrôle n'est qu'un executor de cette autorité.
- Test de structure ajouté (`tests/test_omega_gate_phase1.py`) qui parse
  `composition.rs` sans `#[cfg(test)]` : toute insertion d'adaptateurs de
  production devra la porter dans la liste nominative, sinon la porte rouge.
