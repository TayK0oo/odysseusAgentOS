# STATUS — AgentOS Omega (généré par measure.py — jamais édité à la main)

SHA `650a025a` · mesuré le 2026-10-22T04:10:47+00:00 · moteur Rust (ADR-0009)

| Mesure | Valeur | Méthode |
|---|---|---|
| Crates Rust | 22 | cargo metadata |
| Tests Rust (attributs test) | 411 | scan attributs |
| Tests Python (test_*) | 4436 | ast |
| model_policy --check | OK | script |
| dep_graph | OK | script |

**Scénarios** : S0–S11 `NON_EXECUTE` (banc d'acceptation = P0-4).
**Portes** : G0 VERT · G1 ROUGE · G2 NON_EXECUTE · G3 NON_EXECUTE · G4 NON_EXECUTE · G5 NON_EXECUTE · G6 NON_EXECUTE · G7 NON_EXECUTE — l'etat vient des rapports gate.py (docs/omega/plan/gates/), seule la sortie de gate.py fait foi (lecon L6).
**Prochaine action** : G1.a.s1 : S1 outil reel canary : 5 runs tous PASS (canary obtenu par un VRAI appel d'outil + controle negatif sans surface) — corriger puis relancer omega/scripts/gate.py --phase 1
