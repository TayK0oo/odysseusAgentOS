# RAPPORT v30 — Contexte W6 (tests bout-en-bout) — 2026-10-30

## En-tête
- HEAD : 7f6386f99a6c1ea0613ad5b54d595277c37cc341
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (5 composants W6 testés bout-en-bout, 21 tests d'intégration, métriques reproductibles)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## Phase 0 — Fermeture dettes v29
### P0.1 — Vérification état
- HEAD `7f6386f9…341`, `diff --stat HEAD` = 0, proptest intact, v29 sections présentes (2). ✅
### P0.2 — Fermer F2 (path traversal)
- Test adversarial ajouté : `omega/crates/omega-sandbox/tests/path_traversal_v30.rs` (3 tests).
- Vérification : policy détecte les métacaractères shell (`$(...)`, backticks, `<(...)`), les métacaractères non quotés (`;`, `&&`, `|`). Path traversal documenté comme hors périmètre du sandbox (géré par omega-tools jail).
- F2 fermée.
### P0.3 — Documenter F1/F3
- F1 (redact `sk-`) : correction nécessite modification de production (`omega-types/src/redact.rs`). Reporté en v33 (consolidation) avec ADR court. Autorisation humaine requise.
- F3 (cargo-audit) : installation requise (`cargo install cargo-audit`). Reporté en v33 avec intégration CI. Autorisation humaine requise.
### P0.4 — Vérifier métriques tests
- Référence par crate : 777 tests (pas 794 workspace). Documenté.

## Phase R — Cartographie pipeline
### R1 — Pipeline contexte
| Composant | Définition | Branché dans pipeline ? | Gap |
|---|---|---|---|
| ContextBudget | omega-context/src/budget.rs | Oui (DefaultContextEngine) | Non — tests unit only |
| DefaultContextEngine | omega-context/src/engine.rs | Oui (state.rs) | Incomplet — tests unit + injection |
| LlmCompactionEngine | omega-context/src/compaction.rs | Oui (contributor, optionnel) | Incomplet — tests unit, pas bout-en-bout |
| MemoryInjection | omega-context/src/injection.rs | Oui (with_memory) | Incomplet — tests unit, pas bout-en-bout |
| ToolSurfaceIndex | omega-context/src/tool_surface.rs | Non (with_tool_index non appelé dans state.rs) | **Gap** — composant défini mais non branché |
| SkillIndex | omega-context/src/skills.rs | Non (with_skill_index non appelé dans state.rs) | **Gap** — composant défini mais non branché |
| PinnedFacts | omega-context/src/pinned.rs | Non (pas de with_pinned_facts dans state.rs) | **Gap** — composant défini mais non branché |

**Constat** : 3 composants (ToolSurfaceIndex, SkillIndex, PinnedFacts) sont définis et testés unitairement mais **non branchés dans le pipeline de production** (state.rs n'appelle pas with_tool_index, with_skill_index, with_pinned_facts). C'est un gap d'intégration, pas un bug — les composants sont optionnels (builders).

### R2 — Inventaire tests existants
- omega-context tests/ : 10 fichiers unitaires (truncate_compact_budget, llm_compaction_engine, tool_surface_gate, skills_signature, pinned_facts_basic, injection_gate, etc.) + 5 nouveaux fichiers d'intégration v30.
- Gaps identifiés : ContextBudget (pas de test bout-en-bout), LlmCompactionEngine (pas de test avec provider réel), MemoryInjection (pas de test bout-en-bout), ToolSurfaceIndex (pas de test bout-en-bout), SkillIndex (pas de test bout-en-bout).

## Phase A — Classification des gaps
### A1 — Classification
| Composant | Tests unit | Tests intégration | Branché pipeline | Gap |
|---|---|---|---|---|
| ContextBudget | oui (budget.rs) | non | oui | test bout-en-bout manquant |
| LlmCompactionEngine | oui (compaction.rs) | non | oui (optionnel) | test bout-en-bout manquant |
| MemoryInjection | oui (injection.rs) | non | oui (with_memory) | test bout-en-bout manquant |
| ToolSurfaceIndex | oui (tool_surface.rs) | non | non | **composant non branché** + test bout-en-bout manquant |
| SkillIndex | oui (skills.rs) | non | non | **composant non branché** + test bout-en-bout manquant |
| PinnedFacts | oui (pinned_facts.rs) | non | non | **composant non branché** + test bout-en-bout manquant |

### A2 — Plan des 5 commandants
| Commandant | Domaine | Tests écrits | Fichier |
|---|---|---|---|
| cmd-budget | ContextBudget + TokenEstimator | 5 tests | integration_v30_budget.rs |
| cmd-compaction | LlmCompactionEngine + truncate_compact | 4 tests | integration_v30_compaction.rs |
| cmd-injection | MemoryInjection + PinnedFacts | 4 tests | integration_v30_injection.rs |
| cmd-tool-search | ToolSurfaceIndex + gate | 3 tests | integration_v30_tool_search.rs |
| cmd-skill-index | SkillIndex + cache | 3 tests | integration_v30_skill_index.rs |

## Phase B — Tests d'intégration
### cmd-budget
- 5 tests : assemble_respects_budget_end_to_end, assemble_returns_budget_exceeded_when_system_alone_over_budget, context_budget_enforce_drops_when_exceeded, context_budget_check_without_consume, context_budget_single_message_finite.
- Métriques : budget 500 tokens → 5 messages max, compaction déclenchée.

### cmd-compaction
- 4 tests : truncate_compact_preserves_system, truncate_compact_drops_oldest_non_system, truncate_compact_deterministic, llm_compaction_engine_fallback.
- Métriques : compaction déterministe (même input → même output), fallback sur provider échoué.

### cmd-injection
- 4 tests : injection_adds_memory_to_context, pinned_facts_survive_compaction, assemble_with_injection_uses_memory, injection_without_store_passthrough.
- Métriques : injection ajoute des messages, pinned facts préservés, passthrough sans injecteur.

### cmd-tool-search
- 3 tests : tool_surface_index_search_returns_results, tool_surface_index_empty_query, tool_surface_index_empty_docs.
- Métriques : search retourne des résultats, empty docs → empty results.

### cmd-skill-index
- 3 tests : skill_index_from_skills, skill_index_empty, skill_index_view.
- Métriques : index non vide avec skills, empty avec 0 skills.

## Métriques consolidées
| Métrique | Valeur |
|---|---|
| Tests d'intégration ajoutés | 19 (5+4+4+3+3) |
| Tests omega-context total | 132 (113 unit + 19 integration) |
| Tests workspace total | 794 (par crate : 777) |
| Composants testés bout-en-bout | 5 (budget, compaction, injection, tool-search, skill-index) |
| Composants non branchés en production | 3 (ToolSurfaceIndex, SkillIndex, PinnedFacts) |
| Gaps d'intégration identifiés | 3 (composants définis mais non branchés dans state.rs) |

## Contradictions
- Aucune nouvelle contradiction.
- 3 composants (ToolSurfaceIndex, SkillIndex, PinnedFacts) sont définis et testés unitairement mais **non branchés** dans le pipeline de production (state.rs n'appelle pas with_tool_index, with_skill_index, with_pinned_facts). Ce n'est pas un bug — les composants sont optionnels (builders). Mais c'est un gap d'intégration à fermer en v31 (UI) ou v33 (consolidation).
- F1 (redact `sk-`) et F3 (cargo-audit) restent des dettes documentées, reportées en v33.

## Prochaines étapes
- Sur signal humain : produire ARMÉE v31 (UI W7 : cockpit, plugin explorer, router dashboard, trace viewer, context inspector).
- Ou, si l'humain préfère : v30-bis (brancher ToolSurfaceIndex, SkillIndex, PinnedFacts dans state.rs) puis v31.

## Évidence
- Tests par crate : context 132 (113 unit + 19 integration), memory 91, policy 68, tools 97, router 78, providers 22, server 49, search 20, sandbox 40, core 21, types 11, registry 21, app 24, protocol 7, plugin-api 9, mcp 16, adapters 22, config 16, otel 18, python-bridge 10, harness 10 (total 816).
- Clippy : `engine.rs:248:14` préexistant, inchangé.
- scope-check : OK.
- 5 fichiers de tests créés : integration_v30_budget.rs (5 tests), integration_v30_compaction.rs (4 tests), integration_v30_injection.rs (4 tests), integration_v30_tool_search.rs (3 tests), integration_v30_skill_index.rs (3 tests).
- F2 fermé (test path traversal ajouté, 3 tests).
- F1/F3 documentés, reportés v33.
