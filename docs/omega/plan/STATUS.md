# STATUS — AgentOS Omega (généré par measure.py — jamais édité à la main)

SHA `ceab213d` · mesuré le 2026-10-23T06:52:38+00:00 · moteur Rust (ADR-0009)

| Mesure | Valeur | Méthode |
|---|---|---|
| Crates Rust | 22 | cargo metadata |
| Tests Rust (attributs test) | 442 | scan attributs |
| Tests Python (test_*) | 4440 | ast |
| model_policy --check | OK | script |
| dep_graph | OK | script |

**Scénarios** : S0–S11 `NON_EXECUTE` (banc d'acceptation = P0-4).
**Portes** : G0 VERT · G1 VERT · G2 ROUGE · G3 NON_EXECUTE · G4 NON_EXECUTE · G5 NON_EXECUTE · G6 NON_EXECUTE · G7 NON_EXECUTE — l'etat vient des rapports gate.py (docs/omega/plan/gates/), seule la sortie de gate.py fait foi (lecon L6).
**Prochaine action** : G2.b.s4 : S4 memoire persistante : 5 runs tous PASS (mot retrouve apres kill -9, oubli cascade) — corriger puis relancer omega/scripts/gate.py --phase 2
