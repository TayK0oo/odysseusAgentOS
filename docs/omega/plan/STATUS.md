# STATUS — AgentOS Omega (généré par measure.py — jamais édité à la main)

SHA `405f44d9` · mesuré le 2026-10-19T03:42:44+00:00 · moteur INDETERMINE (ADR D0, P0-9)

| Mesure | Valeur | Méthode |
|---|---|---|
| Crates Rust | 22 | cargo metadata |
| Tests Rust (attributs test) | 370 | scan attributs |
| Tests Python (test_*) | 4415 | ast |
| model_policy --check | OK | script |
| dep_graph | OK | script |

**Scénarios** : S0–S11 `NON_EXECUTE` (banc d'acceptation = P0-4).
**Portes** : G0–G7 `ROUGE` — seule la sortie de gate.py fait foi (leçon L6).
**Prochaine action** : P0-4 : banc d'acceptation (runner boite noire, ScriptedProvider, S0 vert / S1 xfail nomme)
