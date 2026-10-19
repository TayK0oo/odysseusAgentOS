# STATUS — AgentOS Omega (généré par measure.py — jamais édité à la main)

SHA `ba37a51d` · mesuré le 2026-10-19T02:53:18+00:00 · moteur INDETERMINE (ADR D0, P0-9)

| Mesure | Valeur | Méthode |
|---|---|---|
| Crates Rust | 22 | cargo metadata |
| Tests Rust (attributs test) | 370 | scan attributs |
| Tests Python (test_*) | 4415 | ast |
| model_policy --check | OK | script |
| dep_graph | ECHEC(1) | script |

**Scénarios** : S0–S11 `NON_EXECUTE` (banc d'acceptation = P0-4).
**Portes** : G0–G7 `ROUGE` — seule la sortie de gate.py fait foi (leçon L6).
**Prochaine action** : P0-1 : commande verify sur clone vierge (worktree jetable, porte G0.a)
