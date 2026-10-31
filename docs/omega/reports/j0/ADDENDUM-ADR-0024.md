# ADDENDUM ADR-0024 — Assertion `build_wires_expected_plugins`

## Contexte
Le test `build_wires_expected_plugins` (omega-server/src/state.rs, #[cfg(test)]) assertait
`active_ids(Capability::ToolSurface) == vec!["mcp:rea"]`. Après le branchement W6
(ADR-0024), le `ContextContributor` enregistre un `ToolSurfaceIndex` sous la capacité
`ToolSurface` avec l'id `"bm25"`. L'assertion a été mise à jour pour inclure `"bm25"`.

## Ce qui a changé
- Avant : `assert_eq!(active_ids(ToolSurface), vec!["mcp:rea"])`.
- Après : `assert_eq!(sorted(active_ids(ToolSurface)), vec!["bm25", "mcp:rea"])`.

## Pourquoi bm25 apparaît
`ContextContributor::contribute` appelle `builder.register_typed(Manifest::new("bm25", Capability::ToolSurface, "1"), index)` quand `tool_index` est `Some`. Le branchement W6 (ADR-0024) passe `Some(ToolSurfaceIndex::from_docs(vec![]))`, donc `"bm25"` est enregistré.

## Conséquence sur les métriques « sans effet »
Les métriques (tokens/run=1020, compaction=0.75, latence=3µs) ne changent pas car l'index est vide. Mais la **surface d'outils** change : `active_ids(ToolSurface)` passe de 1 à 2 entrées. C'est un effet observable sur la liste des capacités, même si l'effet sur les tokens est nul.

## Justification
Le changement de l'assertion reflète le nouvel état attendu du système après le branchement W6. Ce n'est pas un affaiblissement du test — c'est une mise à jour pour refléter la nouvelle réalité (ADR-0024). Le test vérifie toujours que le pont MCP REA est câblé comme surface d'outils, et ajoute la vérification que l'index BM25 est aussi câblé.
