# ADR-0024 — Branchement des composants W6 dans le pipeline

## Contexte

Les composants W6 (`ToolSurfaceIndex`, `SkillIndex`, `PinnedFacts`) sont implémentés et testés (v21-v23, v30) mais non branchés dans `state.rs` (v30). L'économie de contexte W6 n'est pas réalisée en production.

## Décision

Brancher les 3 composants dans `omega-server/src/state.rs`, à l'aide des builders existants (`with_tool_index`, `with_skill_index`, `with_pinned_facts`), avec des instances vides par défaut (à peupler par le registre en v31+).

## Justification

- `ToolSurfaceIndex` : réduit les tokens consommés par les schémas d'outils (progressive disclosure).
- `SkillIndex` : réduit les tokens consommés par les skills (progressive disclosure 3 niveaux).
- `PinnedFacts` : préserve les faits durables à travers la compaction.

Sans branchement, W6 (économie de contexte) n'est pas réalisé.

## Alternatives

- **(a)** Brancher en v33 (consolidation) — repousse l'économie de contexte.
- **(b)** Ne pas brancher — W6 non réalisé, écart avec la vision.

## Conséquence

- Modification de production dans `state.rs` (seul fichier autorisé).
- Tests de bout-en-bout requis (chaque composant doit être effectivement consommé par le pipeline).
- Métriques à mesurer (tokens/run, compaction rate, cache hits, latence).

## Portée

Uniquement `omega-server/src/state.rs`. Aucune autre production.
