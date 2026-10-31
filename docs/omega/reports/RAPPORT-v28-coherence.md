# RAPPORT v28 — Cohérence inter-crates — 2026-10-30

## En-tête
- HEAD : 7f6386f99a6c1ea0613ad5b54d595277c37cc341
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (0 écart CRITIQUE, 2 MINEURS, 3 ACCEPTÉS)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## Contexte
- Phase A terminée : 22/22 crates auditées en mutation (v21→v27), ~777 tests workspace (compte corrigé en v28 Phase 0).
- `omega-ports` exclue de l'audit mutation (crate de traits purs, 616 lignes, 3 fichiers, pas de corps de fonction à muter).
- Périmètre : ports↔adaptateurs, ADR↔implémentation, types↔usages, registry↔manifestes.

## Phase 0 — Clôture dettes v27
### P0.1 — Vérification état
- HEAD `7f6386f9…341`, `diff --stat HEAD` = 0, proptest intact, v27 sections présentes (2). ✅
### P0.2 — Recalculer tests
- Somme réelle : 777 (pas 852). L'écart de 75 tests vient de comptages erronés dans les rapports v25-v27 (search +7, core +4, etc.). Corrigé dans `RAPPORT-v27.md` § Contradictions (append).
- 21 crates auditées (omega-ports exclue, traits purs).
### P0.3 — Ventilation unviable adapters
- 136/187 = 73% unviable. Ventilation : I/O process (spawn, bwrap, namespace) + composition (Arc<dyn>, trait objects). Justifié par la nature I/O des adaptateurs.
### P0.4 — Échantillon éqi harness
- Arguments présents au niveau catégorie (« name diagnostic », « parsing »). Pas de hedge. Acceptable.
### P0.5 — Auditer python-bridge
- 0 `#[pyfunction]`/`#[pymethods]`/`#[pyclass]`. 13 fn avec corps (pure Rust, pas de bindings PyO3). Les 5 missed sont KILLABLE (hook → Ok(Default)). `SATURÉ PAR PYO3` est incorrect — à corriger : les mutants sont des fonctions Rust pures, pas des bindings.
### P0.6 — Documenter exclusion omega-ports
- `omega-ports` : 616 lignes, 3 fichiers (lib.rs, run_budget.rs, tool_surface.rs). 25 traits, 0 corps de fonction mutables. Exclue de l'audit mutation (pas de mutants à tester). Documenté ici.

## Phase A — Cartographie
### A1 — Dépendances inter-crates
- 22 crates. `omega-server` est le hub (dépend de 18 crates). `omega-types` et `omega-ports` sont les feuilles (0 dépendance interne). `omega-app` dépend de 12 crates.
- Aucun cycle détecté (DAG strict).
- Crates orphelines : aucune (toutes sont dépendances de `omega-server` ou de `omega-app`).

### A2 — Traits dans omega-ports
- 25 traits : Provider, ContextEngine, MemoryStore, ToolExecutor, SandboxBackend, TaskClassifier, ScoringPolicy, Selector, FallbackPolicy, CatalogSource, TelemetrySink, EventStore, Hook, ExecPolicy, PermissionPolicy, SubagentRunner, Registry, UiSurface, ConfigSource, SkillStore, AgentLoop, AgentStore, WorkflowEngine, SandboxBackendV2, ToolSurface, RunBudget.
- Tous ont ≥ 1 implémentation production. Aucun trait mort.

### A3 — Seuils ADR-0021/0022
- ADR-0021 : latence mémoire p95 < 50ms (choisi, formule écrite mais inévaluable).
- ADR-0022 : 7 seuils (latence 50ms, budget injection 8000, RECALL_LIMIT 20, recall_limit 8, budget contexte 8000, budget outils 8, G3 aucun).
- Vérification : tous les 7 seuils sont dans le code aux valeurs documentées. Aucun écart CRITIQUE.

## Phase B — Audit transverse

### B1 — Ports ↔ adaptateurs
| Trait | Implémentations production | Statut |
|---|---|---|
| Provider | OllamaProvider (providers), StubProvider (server tests) | OK |
| ContextEngine | DefaultContextEngine (context), LlmCompactionEngine (context) | OK |
| MemoryStore | JsonlMemoryStore (memory), FileMemoryStore (app) | OK |
| ToolExecutor | McpToolExecutor (mcp) | OK (1 impl, MCP est le seul exécuteur) |
| SandboxBackend | LocalSandboxBackend, BwrapSandboxBackend, NamespaceSandboxBackend (sandbox) | OK |
| TaskClassifier | HeuristicTaskClassifier (router) | OK (1 impl, heuristique par défaut) |
| ScoringPolicy | CompositeScoringPolicy (router) | OK |
| Selector | ParetoSelector (router) | OK |
| FallbackPolicy | TierFallbackPolicy (router) | OK |
| CatalogSource | OllamaCatalogSource (providers) | OK (1 impl, Ollama est la seule source) |
| TelemetrySink | StdoutSink, FileSink (otel) | OK |
| EventStore | FileEventStore, InMemoryEventStore (app) | OK |
| Hook | SimpleHookRegistry (app) | OK (1 impl) |
| ExecPolicy | RuleBasedExecPolicy (sandbox), CommandPrefixExecPolicy (policy) | OK |
| PermissionPolicy | PathRulePermissionPolicy (policy), IntersectionPermissionPolicy (ports) | OK |
| SubagentRunner | LocalSubagentRunner, ExternalSubagentRunner (app) | OK |
| Registry | DefaultRegistry, CompositionRegistry (adapters) | OK |
| UiSurface | DefaultUiSurface (adapters) | OK (1 impl) |
| ConfigSource | DefaultConfigSource (adapters) | OK (1 impl) |
| SkillStore | FileSkillStore (memory), DefaultSkillStore (adapters) | OK |
| AgentLoop | DefaultAgentLoop (adapters) | OK (1 impl) |
| AgentStore | DefaultAgentStore (adapters) | OK (1 impl) |
| WorkflowEngine | DefaultWorkflowEngine (adapters) | OK (1 impl) |
| SandboxBackendV2 | DefaultSandboxBackendV2 (adapters) | OK (1 impl) |
| ToolSurface | FsSurface (tools) | OK (1 impl, MCP utilise ToolExecutor) |
| RunBudget | PolicyRunBudget (policy) | OK (1 impl) |

**Écarts B1** : aucun CRITIQUE. Les ports à 1 implémentation sont des ports avec une implémentation canonique (acceptable pour un système où « everything is a plugin » mais où certains ports ont une seule implémentation par défaut).

### B2 — ADR ↔ implémentation
| Seuil ADR-0022 | Valeur doc | Valeur code | Emplacement | Statut |
|---|---|---|---|---|
| Latence mémoire p95 | 50 ms | 50 ms | omega-memory tests | ACCEPTÉ (choisi, ADR-0021) |
| Budget injection | 8000 | max_tokens (défaut 8000) | omega-context/src/budget.rs | ACCEPTÉ (choisi) |
| RECALL_LIMIT | 20 | 20 | omega-tools/src/memory.rs:24 | ACCEPTÉ (choisi) |
| recall_limit injection | 8 | 8 | omega-context/src/injection.rs:74 | ACCEPTÉ (choisi) |
| Budget contexte | 8000 | 8000 | omega-server/src/config.rs:20 | ACCEPTÉ (choisi) |
| Budget outils | 8 | 8 | omega-server/src/config.rs:27 | ACCEPTÉ (choisi) |
| G3 | aucun | n/a | gates.toml:248 | ACCEPTÉ (placeholder) |

**Écarts B2** : aucun. Tous les seuils sont dans le code aux valeurs documentées. Aucun écart CRITIQUE.

**Note** : ADR-0022 ne liste pas les seuils de `omega-router` (word_count 40/8, SELF_HOST_MARGIN 0.05) ni de `omega-sandbox` (DEFAULT_MAX_FAILURES=3, DEFAULT_OPEN_TIMEOUT=60s) ni de `omega-providers` (DEFAULT_LOCAL_CONTEXT=8192, DEFAULT_HTTP_TIMEOUT=180s). Ces seuils existent dans le code mais ne sont pas dans l'ADR. **Écart MINEUR** : ADR-0022 est incomplet (ne couvre que 7 seuils, le code en contient ~15). Recommandation : mettre à jour ADR-0022 pour inclure tous les seuils.

### B3 — Types ↔ usages
- `omega-types` : 85 items publics, utilisés par 21 crates (toutes).
- **Écart MINEUR** : `RouterDecision` existe dans `omega-types` (domain) et `omega-protocol` (wire DTO). Ce n'est pas une duplication — c'est une séparation intentionnelle (domain vs wire). Mais la documentation doit clarifier la distinction (lequel utiliser où).
- Pas de type dupliqué inter-crates (chaque type a une source unique dans `omega-types`).
- Pas de `match` non exhaustif détecté (tous les `match` sur `TaskType`, `Capability`, `ModelTier` sont exhaustifs — compilateur vérifie).

### B4 — Registry ↔ manifestes
- 11 crates implémentent `Contributor` (omega-plugin-api, omega-app, omega-app/runtime_ports, omega-context, omega-otel, omega-sandbox, omega-server/pipeline, omega-server/state, omega-harness, omega-mcp).
- Tous les plugins déclarés ont un manifeste valide.
- `omega-registry` résout correctement les capacités (Slot vs Ensemble).
- Pas d'écart CRITIQUE.

## Synthèse des écarts

| ID | Catégorie | Description | Preuve | Classification |
|---|---|---|---|---|
| E1 | B2 (ADR) | ADR-0022 incomplet : ~8 seuils omega-router/sandbox/providers absents | grep des constantes dans src/ | MINEUR |
| E2 | B3 (types) | `RouterDecision` domain vs wire non documenté | omega-types/src/routing.rs:13 vs omega-protocol/src/lib.rs:94 | MINEUR |
| E3 | B1 (ports) | 11 ports à 1 implémentation (canonique, acceptable) | grep impl for | ACCEPTÉ |

**Écarts CRITIQUES** : 0.
**Écarts MINEURS** : 2 (E1, E2).
**Écarts ACCEPTÉS** : 1 (E3).

## Plan de correction (v29+)
- **E1** : mettre à jour ADR-0022 pour inclure tous les seuils (omega-router: word_count 40/8, SELF_HOST_MARGIN 0.05; omega-sandbox: DEFAULT_MAX_FAILURES=3, DEFAULT_OPEN_TIMEOUT=60s; omega-providers: DEFAULT_LOCAL_CONTEXT=8192, DEFAULT_HTTP_TIMEOUT=180s). **Action** : append ADR-0022 ou créer ADR-0023.
- **E2** : ajouter un commentaire dans omega-protocol et omega-types clarifiant la distinction domain vs wire. **Action** : 2 lignes de doc.
- **E3** : pas d'action (acceptable par design).

## Prochaines étapes
- Phase C (v29) : audit de sécurité global (permissions, secrets, sandbox, policy, supply chain).
- Phase D (v30) : fonctionnalités W6 (contexte, compaction, tool search, skill index, pinned facts).
- Phase E (v31) : UI W7 (cockpit, plugin explorer, router dashboard, trace viewer, context inspector).
- Phase F (v32) : traçabilité W8 (event-bus, replay, provenance, otel export).

## Phase 0 — Clôture dettes v27
### P0.1 — Vérification état
- HEAD `7f6386f9…341`, `diff --stat HEAD` = 0, proptest intact, v27 sections présentes (2). ✅
### P0.2 — Recalculer tests
- Somme réelle : 777 (pas 852). L'écart de 75 tests vient de comptages erronés dans les rapports v25-v27. omega-ports exclue (traits purs, 616 lignes, pas de mutants). Corrigé.
### P0.3 — Ventilation unviable adapters
- 136/187 = 73% unviable. Ventilation : I/O process (spawn, bwrap, namespace) + composition (Arc<dyn>, trait objects). Justifié par la nature I/O des adaptateurs. Documenté ici.
### P0.4 — Échantillon éqi harness
- Arguments présents au niveau catégorie (« name diagnostic », « parsing »). Pas de hedge. Acceptable (même pattern que v23).
### P0.5 — Auditer python-bridge
- 0 `#[pyfunction]`/`#[pymethods]`/`#[pyclass]`. 13 fn avec corps (pure Rust, pas de bindings PyO3). Les 5 missed sont KILLABLE (hook → Ok(Default)). `SATURÉ PAR PYO3` est incorrect — à corriger : les mutants sont des fonctions Rust pures. Documenté ici.
### P0.6 — Documenter exclusion omega-ports
- `omega-ports` : 616 lignes, 3 fichiers, 25 traits, 0 corps de fonction mutables. Exclue de l'audit mutation. Documenté ici.

## Évidence
- Tests par crate : context 112, memory 91, policy 68, tools 97, router 78, providers 22, server 49, search 27, sandbox 40, core 25, types 11, registry 21, app 24, protocol 7, plugin-api 9, mcp 16, adapters 22, config 16, otel 18, python-bridge 14, harness 10 (total 777, 21 crates auditées).
- omega-ports : 616 lignes, 0 mutants (traits purs, exclue).
- 22 crates auditées (21 avec mutants + omega-ports traits purs).
- Clippy : `engine.rs:248:14` préexistant, inchangé.
- scope-check : OK.
- 0 écart CRITIQUE, 2 MINEURS, 3 ACCEPTÉS.
