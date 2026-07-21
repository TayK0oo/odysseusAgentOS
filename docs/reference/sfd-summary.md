# Reference — Résumé SFD v3.0

> Document complet : `SFD.md` (1065 lignes)

## Mission

Agent OS est un assistant autonome de conduite de projet capable de comprendre une demande, élaborer un plan, l'exécuter, et apprendre de chaque expérience.

## 22 Principes invariants

| # | Principe | Implémenté? |
|---|----------|------------|
| P1 | Risque modifie la boucle | 🟡 Partiel |
| P2 | Draft ≠ commit | 🔴 |
| P3 | Contexte construit (MVI) | 🟡 |
| P4 | Budgets obligatoires | 🟡 |
| P5 | Divulgation progressive | 🔴 |
| P6 | Échecs → règles | 🔴 |
| P7 | Structure > autonomie | 🟡 |
| P8 | Plan = mêmes portes | 🔴 |
| P9 | Évaluer harnais | 🟢 |
| P10 | Humain ON the loop | 🔴 |
| P11 | Commencer simple | 🟡 |
| P12 | Découpage par contexte | 🔴 |
| P13 | Contexte = budget d'attention | 🟡 |
| P14 | Actions longues durables | 🔴 |
| P15 | Observabilité dès conception | 🟢 |
| P16 | Provenance explicite | 🔴 |
| P17 | Ne pas stocker sensible | 🟡 |
| P18 | Lire avant d'écrire | 🔴 |
| P19 | Mémoire gagne sa place | 🔴 |
| P20 | Préférences par priorité | 🟡 |
| P21 | Bon outil, sans friction | 🔴 |
| P22 | Sortie visuelle premier rang | 🔴 |

## 20 Modules fonctionnels

| Module | Statut |
|--------|--------|
| Décision multi-agent (§5.1) | 🔵 Dormant |
| Ingénierie du contexte (§5.2) | 🟡 Partiel |
| Planification (§5.3) | 🔴 Absent |
| Exécution contrôlée (§5.4) | 🟢 Actif |
| Exécution durable (§5.5) | 🔴 Absent |
| Routage modèles (§5.6) | 🟢 Actif |
| Mémoire (§5.7) | 🟡 Partiel |
| Communication multi-canal (§5.8) | 🔵 Dormant |
| Gouvernance (§5.9) | 🟡 Partiel |
| Auto-évaluation (§5.10) | 🔵 Dormant |
| Observabilité (§5.11) | 🟢 Actif |
| Workspaces (§5.12) | 🟢 Actif |
| Extensibilité MCP (§5.13) | 🟢 Actif |
| Configuration (§5.14) | 🟢 Actif |
| Préférences (§5.15) | 🔴 Absent |
| Recherche conversations (§5.16) | 🟢 Actif |
| Skills (§5.17) | 🟢 Actif |
| Modalités sortie (§5.18) | 🔴 Absent |
| Classification données (§5.19) | 🔴 Absent |
| Sécurité contenus (§5.20) | 🔴 Absent |

## Alignement global : ~45%

6 modules 🟢 Actifs · 5 modules 🟡 Partiels · 4 modules 🔵 Dormants · 7 modules 🔴 Absents

---

→ Voir aussi : [Glossaire](glossary.md) · [Index maître](index-master.md) · `SFD.md`
