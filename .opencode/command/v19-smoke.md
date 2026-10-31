---
description: Test de fumée du dispositif (à lancer AVANT /v19-wave0 et /v19-run)
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Vérifie, avec une preuve par commande ou par résultat de Task, chaque point (OK/KO + cause) :
1. Les 5 subagents (reader, mutator, tester, verifier, scribe) sont reconnus et invocables via Task.
2. 3 Task `reader` lancées dans UN SEUL message s'exécutent bien en parallèle (relever les horodatages).
3. Les hooks gsd-write-guard / gsd-secret-read-guard laissent écrire un fichier de test jetable dans omega/crates/omega-tools/tests/ et dans docs/omega/reports/evidence-v19/ (puis le supprimer).
4. Les permissions effectives (inclure un test de composition : `scripts/v19/scope-check.sh ; git commit --help` ne doit PAS passer sans confirmation ni refus — sinon R12 seule ne protège pas, le signaler en KO) : `scripts/v19/evidence.sh <args> -- cargo --version` (avec arguments) passe sans confirmation ; le `tester` écrit sans confirmation, l'`orchestrator` ne peut pas écrire, `git commit` et `git push` sont refusés.
5. Les plugins (rtk.ts, oh-my-openagent) : (a) se chargent sans erreur (distinguer « plugin non chargé » de « sortie altérée ») ; (b) ne modifient pas la sortie de `scripts/v19/evidence.sh` : comparer le .raw avec la sortie réelle d'une commande connue (`cargo --version`). Ne renomme ni ne modifie aucun plugin global.
6. Aucun agent de oh-my-openagent n'entre en conflit de nom ou de rôle avec orchestrator/reader/mutator/tester/verifier/scribe.
7. `scripts/v19/wt.sh add smoke` crée un worktree inscriptible (sinon : le signaler en KO et demander à l'HUMAIN d'écrire le chemin d'un dossier inscriptible dans `.opencode/v19-wt-root` (une ligne) ; l'agent ne l'écrit pas lui-même, l'écriture déclencherait une confirmation. Pas de variable d'environnement. Puis relancer ce point), `evidence.sh` y tourne avec CARGO_TARGET_DIR propre, puis `wt.sh clean` le supprime.
8. Un subagent renvoie bien le JSON du format de RULES-v19.md avec le modèle effectif.
9. Limite d'un appel bash : `scripts/v19/evidence.sh orch limite-bash . 140 -- sleep 130` ; si l'appel est coupé avant la fin, noter la limite L mesurée (< 130 s). Puis `scripts/v19/bg.sh start smoke orch bg-test . 10 -- bash -c 'sleep 3; echo ok'`, `scripts/v19/bg.sh wait smoke 30`, puis `scripts/v19/bg.sh status smoke` DOIT afficher « done — E-nnn exit=... » non vide : un status « done » sans identifiant d'évidence ni exit-code est KO (tighten rev8 — un faux done est plus grave qu'un KO documenté).
Conclus : "Dispositif prêt : OUI/NON" avec la liste des KO, et enregistre le verdict : `scripts/v19/evidence.sh orch SMOKE-OK . 5 -- echo ok` si tout est OK, sinon `scripts/v19/evidence.sh orch SMOKE-KO . 5 -- echo ko`. Ne lance aucune vague.
