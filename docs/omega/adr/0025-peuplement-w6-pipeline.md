# ADR-0025 — Peuplement des index W6 (PROPOSÉ)

**Date** : 2026-10-30 · **Statut** : ACCEPTED (D-W6 = B, peuplement réalisé v34) · **Décidé par** : N0 (chef projet)

## Contexte

Les 3 composants W6 (`ToolSurfaceIndex`, `SkillIndex`, `PinnedFacts`) sont implémentés, testés (unit + intégration v30), et branchés dans `omega-server/src/state.rs` (v30-bis, v30-ter). Mais ils sont alimentés par des données vides (`from_docs(vec![])`, `from_skills(vec![])`, `PinnedFacts::new()`). L'économie de contexte W6 n'est pas réalisée.

## Constat factuel (preuve littérale)

1. **Outils** : `grep "struct.*ToolRegistry\|struct.*PluginRegistry\|struct.*Catalog" omega/crates/*/src/*.rs` → aucun `ToolRegistry`/`PluginRegistry`. Les outils sont dans `omega-tools` (ToolSurface::FsSurface) et `omega-mcp` (McpToolSurface), mais pas accessibles dans `build_app_state_with` (le registre `app` est construit après le ContextContributor).
2. **Skills** : `grep "struct.*SkillStore\|impl.*SkillStore"` → `FileSkillStore` (omega-memory) et `DefaultSkillStore` (omega-adapters). Mais `state.rs` n'a pas de `skill_store` variable — seul `memory_store` (FileMemoryStore) est disponible.
3. **Messages** : `build_app_state_with(cfg)` ne reçoit pas de messages. Les messages arrivent à l'exécution via `execute_run`, pas à la construction. `PinnedFacts::extract(messages)` n'est pas appelable dans `state.rs`.

## Décision (3 options)

- **Option A** : élargir `build_app_state_with(registry, skill_store, ...)` pour résoudre les données au moment de la construction. **ADR-0025 requis.** Coût : 20-30 min. Permet le peuplement réel.
- **Option B** : brancher vide + dette documentée v33 (déjà fait). Coût : 0 min. W6 non réalisé.
- **Option C** : population during `assemble` (omega-context, ADR-0025 requis). Coût : 30-40 min. Plus complexe mais plus flexible (les données sont disponibles à l'exécution).

## Alternatives

- **(a)** Reporter le peuplement à v33 (consolidation) — repousse l'économie de contexte.
- **(b)** Ne pas peupler — W6 non réalisé, écart avec la vision.

## Conséquence

- Si A ou C : modification de production (`state.rs` ou `engine.rs`), ADR-0025 requis, tests de bout-en-bout, métriques re-mesurées.
- Si B : dette documentée, pas de modification.

## Portée

Uniquement le peuplement des 3 index. Aucune autre production.
