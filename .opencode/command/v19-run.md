---
description: Campagne complète omega-tools (vagues 1 à 5)
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Pré-requis : /v19-wave0 sans écart. Exécute :
V1 : 8× reader (un fichier de omega/crates/omega-tools/src chacun) + reader transverse + verifier réconciliateur (9/10 tests io_accessors, 15/16 serde_defaults, missed 15/7 vs 6+3, "7 unviable"/"4 équipotents", unique_tmp_*).
     Porte : chaque mutant de `cargo mutants --list -p omega-tools` rattaché à une fonction inventoriée.
V2 : mutator ×N (shardé selon `--list`, max 6 en parallèle), verifier contrôleur (caught+missed+unviable+timeout = total).
V3 : tester ×N (incr-J1..J4, un fichier de tests chacun, écrit dans l'arbre principal puis `wt.sh sync <id>` et preuves dans le worktree).
V4 : mutator rejoue ; reader prouveur d'équivalence ; verifier adversaire (max 2 itérations) ; dossier unique_tmp_*.
V5 : scope-check, fmt, clippy (seul warning engine.rs:248), tests par crate (attendu 323 + tests ajoutés), scribe, `scripts/v19/lint-report.sh`, `scripts/v19/wt.sh clean`.
Lance les Task d'une même vague dans un seul message. Rapport final au format v18/v19 (§1 synthèse, A, B, D, F, C+E), uniquement depuis index.csv.
