---
description: J0 — clôture : analyse des branches, réconciliation des tests, vérification d'archive, brouillon d'addendum ADR-0024 (LECTURE SEULE)
agent: orchestrator
---
Lis @.opencode/RULES-v19.md (R3, R4, evidence.sh). Tu es l'orchestrateur. LECTURE SEULE : tu n'écris que dans docs/omega/reports/j0/. Jamais de commit, push, prune, suppression, installation.
Lance les 4 readers dans UN SEUL message ; chacun rend un fichier dans docs/omega/reports/j0/ et passe ses commandes par scripts/v19/evidence.sh.

R-branches : pour chaque branche locale en avance sur HEAD (`git rev-list --count HEAD..<b>` ≠ 0) :
 sujets des commits (`git log --oneline HEAD..<b>`), `git cherry -v HEAD <b>` (« - » = déjà équivalent par patch-id, « + » = absent de HEAD), `git diff --stat HEAD...<b>`,
 recouvrement avec les fichiers des commits C1 à C4 (`git diff --name-only <base-avant-C0>..HEAD`).
 Verdict par branche : DÉJÀ-ÉQUIVALENT · TRAVAIL-UNIQUE · RECOUVREMENT-À-ARBITRER · À-VÉRIFIER (avec cause). Aucune fusion, aucune suppression.
R-tests : `env CARGO_TARGET_DIR=/tmp/omega-clean-target cargo test --workspace -- --list` ; tableau tests par binaire (unitaires, intégration, doc) et par crate ; total ; explique 704 / 777 / 804 / 852 ;
 omega-server : 40 ou 51 ? (nombre avant et après l'ajout de w6_branch_v30_bis.rs) ; #[ignore] ; noms de tests dupliqués.
R-archive : si ../omega-mutants-*.tar.zst existe : `tar --zstd -tf` (nombre d'entrées) contre le nombre de fichiers des mutants.out* sur disque ; somme de contrôle ; sinon « archive absente ».
 Contrôle aussi que docs/omega/reports/, evidence-v19/ et plan/gates/ figurent (ou non) dans l'archive complète ../omega-complet-*.tgz.
R-adr : compare le diff de l'assertion `build_wires_expected_plugins` (omega/crates/omega-server/src/state.rs) à l'ADR-0024 ; rédige un ADDENDUM-ADR-0024.md (≤ 25 lignes : contexte, ce qui a changé, pourquoi bm25 apparaît, conséquence sur les métriques « sans effet ») dans docs/omega/reports/j0/ ; ne modifie PAS l'ADR.
Un verifier relit les 4 sorties, signale toute contradiction avec les chiffres de index.csv. Après : scripts/v19/scope-check.sh (seul docs/omega/reports/j0/ doit avoir bougé).
Rapport : docs/omega/reports/j0/J0-REPORT.md, ≤ 120 lignes, avec un RÉSUMÉ COLLABLE ≤ 60 lignes en tête ; chaque chiffre cite son E-nnn. Statuts de RULES-v19.md uniquement.
Dernière ligne : « J0 terminé — décisions humaines : branches, D-W6, ADR-0026/0027 ».