# Phase 6 — Intake vague 1 — SUMMARY

**Statut : COMPLETE** (exécution inline, budget délégation épuisé ; validation par scripts déterministes).

## Livre

| Harnais | REA machine | Dissection | Fiches fonctionnalités | Refs résolues |
|---|---|---|---|---|
| onlook (26,946★, TS) | ✅ deep (1941 limites) | ✅ | 14 | 24/24 |
| roo-code (24,272★, TS) | ✅ deep | ✅ | 14 | 20/20 |
| dyad (21,972★, TS) | ✅ deep (4379 limites) | ✅ | 14 | 23/23 |
| swe-agent (20,594★, Python) | ✅ vue d'ensemble (8 limites) | ✅ | 13 | 21/21 |
| devika (19,551★, Python) | ✅ vue d'ensemble (5 limites) | ✅ | 9 | —/0 |
| agent-zero (19,506★, Python) | ✅ vue d'ensemble (141 limites) | ✅ | 13 | 18/18 |

## Faits par phase

- Corpus passe de **35 -> 41 harnais**, 822 fonctionnalités (174 adoptées · 614 planifiées · 34 écartées), **109 capacités cibles**.
- Écarts REA-02 documentés : Python/Rust/Go hors surface REA raison datée en genre (vue d'ensemble, limites chiffreés).
- Script **gen_dissection_md.py** ajoute : la fiche markdown de chaque dissection est une projection déterministe du JSON (source unique).
- register_dissections (APP_DISSECTIONS 41, seuil 41) + census refresh courant.

## Verification croisée
- validate_dissections.py : 0 ref non résolue (41/41 fichiers OK).
- cargo test -p omega-harness : 8/8 verts.
