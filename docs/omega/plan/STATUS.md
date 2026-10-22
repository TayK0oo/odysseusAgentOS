# STATUS — AgentOS Omega (généré par measure.py — jamais édité à la main)

SHA `2d3bd240` · mesuré le 2026-10-22T22:22:52+00:00 · moteur Rust (ADR-0009)

| Mesure | Valeur | Méthode |
|---|---|---|
| Crates Rust | 22 | cargo metadata |
| Tests Rust (attributs test) | 408 | scan attributs |
| Tests Python (test_*) | 4440 | ast |
| model_policy --check | OK | script |
| dep_graph | ECHEC(1) | script |

**Scénarios** : S0–S11 `NON_EXECUTE` (banc d'acceptation = P0-4).
**Portes** : G0 ROUGE · G1 ROUGE · G2 NON_EXECUTE · G3 NON_EXECUTE · G4 NON_EXECUTE · G5 NON_EXECUTE · G6 NON_EXECUTE · G7 NON_EXECUTE — l'etat vient des rapports gate.py (docs/omega/plan/gates/), seule la sortie de gate.py fait foi (lecon L6).
**Prochaine action** : G0.a : verify exit 0 sur clone vierge (execution complete, tests inclus) — corriger puis relancer omega/scripts/gate.py --phase 0
