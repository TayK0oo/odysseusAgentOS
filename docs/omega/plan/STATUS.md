# STATUS — AgentOS Omega (généré par measure.py — jamais édité à la main)

SHA `983a4379` · mesuré le 2026-10-20T20:27:07+00:00 · moteur INDETERMINE (ADR D0, P0-9)

| Mesure | Valeur | Méthode |
|---|---|---|
| Crates Rust | 22 | cargo metadata |
| Tests Rust (attributs test) | 370 | scan attributs |
| Tests Python (test_*) | 4425 | ast |
| model_policy --check | OK | script |
| dep_graph | OK | script |

**Scénarios** : S0–S11 `NON_EXECUTE` (banc d'acceptation = P0-4).
**Portes** : G0 ROUGE · G1 NON_EXECUTE · G2 NON_EXECUTE · G3 NON_EXECUTE · G4 NON_EXECUTE · G5 NON_EXECUTE · G6 NON_EXECUTE · G7 NON_EXECUTE — l'etat vient des rapports gate.py (docs/omega/plan/gates/), seule la sortie de gate.py fait foi (lecon L6).
**Prochaine action** : G0.d : ADR D0 fusionne (moteur de reference : Rust ou Python) — corriger puis relancer omega/scripts/gate.py --phase 0
