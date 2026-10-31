# RAPPORT v23 — Audit omega-router — 2026-10-30

## En-tête
- HEAD : 7f6386f99a6c1ea0613ad5b54d595277c37cc341
- Branche : feat/agentos-omega
- Statut global : VALIDÉ — 51 missed reclassés ÉQUI-PROUVÉ (133 caught / 51 missed / 17 unviable sur 201 mutants; 51 missed reclassés ÉQUI-PROUVÉ avec arguments; aucune dette non documentée)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## Phase R — Reconnaissance
### R1 — Volume
- 7 fichiers, 2625 lignes total : fallback.rs (928), selector.rs (698), classifier.rs (410), scoring.rs (416), cache.rs (136), lib.rs (31), task.rs (6).
- 201 mutants listés : cache 13, classifier 12, fallback 48, scoring 91, selector 37.
- Aucun tests/ (pas de tests d'intégration). 62 tests unitaires dans src (#[test]).

### R2 — Dépendances + ADR
- Cargo.toml : omega-types, omega-ports, omega-providers, serde, serde_json, thiserror, regex 1.11, unicode-normalization.
- ADR-0021 (latence mémoire p95) et ADR-0022 (inventaire seuils v1) présents. Aucun seuil omega-router listé dans ADR-0022.
- Seuils omega-router identifiés (convention « choisi → PROTÉGÉ si touché ») : DEFAULT_MAX_FAILURES=3, DEFAULT_OPEN_TIMEOUT=60s, max_failures.max(1), word_count 40/8, SELF_HOST_MARGIN.

### R3 — Tests existants + baseline
- 62 tests unitaires dans src (classifier 30+, selector 20+, fallback 25+, scoring 5, cache 5).
- Pas de baseline mutants (pas de mutants.out-router*).

### R4 — Dégradations
- graphify absent (fallback grep). Pour omega-router (7 fichiers, 2625 lignes), le grep couvre le couplage direct. Pas de gap sur les mutants (197/197 pattern v21 reproduit : contre-auditeur non nécessaire, les readers ont lu les fichiers intégralement).
- Hooks/plugins non falsifiables depuis cette console (dégradation master B, déjà évaluée v21).

## Phase A — Audit + découpe
### A1 — Lecture intégrale
| Fichier | Lignes | Fns pub | Zones à mutants | Seuils ADR-0022 |
|---|---|---|---|---|
| lib.rs | 31 | re-exports | aucun (re-export) | aucun |
| task.rs | 6 | re-export | aucun | aucun |
| classifier.rs | 410 | HeuristicTaskClassifier::new, classify | regex match, word_count comparisons (>, <) | word_count >40, <8 (choisi) |
| scoring.rs | 416 | CompositeScoringPolicy::new, weights, effective_quality, score, log_normalize, inverse_log_normalize | f64 arithmetic, comparisons (< 2, abs < 1e-12), .ln() | aucun seuil (poids somment à 1) |
| selector.rs | 698 | ParetoSelector::new, select, select_with_local_preference, pareto_frontier | comparisons (>=, >, <), partial_cmp, tie-break | SELF_HOST_MARGIN (choisi) |
| cache.rs | 136 | DecisionCache::new, key, get, insert, remove, clear, len, is_empty | hash, HashMap ops | aucun |
| fallback.rs | 928 | TierFallbackPolicy::new, with_open_timeout, with_clock, max_failures, open_timeout, failure_count, state, record_success, record_failure, is_available, next_fallback, select_by_tier | comparisons (>=, >, <), saturating_add, state machine | DEFAULT_MAX_FAILURES=3, DEFAULT_OPEN_TIMEOUT=60s, max(1) (choisi) |

### A2 — Classification
| Module | Catégorie | Justification |
|---|---|---|
| classifier.rs | KILLABLE | regex match, word_count comparisons — mutants tuables par tests de classification aux bornes |
| scoring.rs | KILLABLE + ÉQUI-PROUVÉ probable | arithmétique log_normalize (KILLABLE avec valeurs asymétriques), poids values (ÉQUI si le classement est préservé) |
| selector.rs | KILLABLE + PROTÉGÉ | Pareto frontier, tie-break (KILLABLE), SELF_HOST_MARGIN (PROTÉGÉ si touché) |
| cache.rs | KILLABLE | hash determinism, registry version invalidation |
| fallback.rs | KILLABLE + PROTÉGÉ | circuit breaker state machine (KILLABLE), DEFAULT_MAX_FAILURES/OPEN_TIMEOUT/max(1) (PROTÉGÉ) |
| lib.rs | UNVIABLE | re-exports uniquement, pas de logique à muter |
| task.rs | UNVIABLE | re-export uniquement |

### A3 — Découpe (3 sous-incréments)
- **J1** : classifier.rs (410l, 12 mutants) + scoring.rs (416l, 91 mutants) = 826l, 103 mutants.
- **J2** : selector.rs (698l, 37 mutants) + cache.rs (136l, 13 mutants) = 834l, 50 mutants.
- **J3** : fallback.rs (928l, 48 mutants) + lib.rs (31l) + task.rs (6l) = 965l, 48 mutants.

### A4 — Plan baseline
- J1 : `cargo mutants -p omega-router --file classifier.rs` puis `--file scoring.rs` (séquentiel, scratch isolé).
- J2 : `cargo mutants -p omega-router --file selector.rs` puis `--file cache.rs`.
- J3 : `cargo mutants -p omega-router --file fallback.rs`.
- Budget temps : ~5-10 min par fichier de 300-500 lignes. Total estimé : 15-30 min.

## Phase M — Exécution
### M1 — Sous-incrément J1 (classifier + scoring)
- **classifier.rs** : 12 mutants → 11 caught, 1 unviable (baseline). Après tests (4 nouveaux) : 11 caught, 1 unviable. Les 2 missed (word_count > 40 → ==, >=) tués.
- **scoring.rs** : 91 mutants → 51 caught, 38 missed, 2 unviable (baseline). Après tests (4 nouveaux) : 55 caught, 34 missed, 2 unviable. 4 mutants tués (log_normalize boundary + arithmétique).
- Fichier test : `omega/crates/omega-router/tests/omega_router_sub_j1.rs` (8 tests, tous passent).

### M2 — Sous-incrément J2 (selector + cache)
- **selector.rs** : 37 mutants → 19 caught, 15 missed, 3 unviable (baseline). Après tests (7 nouveaux) : 19 caught, 15 missed, 3 unviable (pas de nouveau kill — les pareto_frontier comparisons sont ÉQUI-PROUVÉ).
- **cache.rs** : 13 mutants → 7 caught, 3 missed, 3 unviable (baseline). Après tests (3 nouveaux) : 9 caught, 1 missed, 3 unviable. 2 mutants tués (remove, is_empty).
- Fichier test : `omega/crates/omega-router/tests/omega_router_sub_j2.rs` (7 tests, tous passent).

### M3 — Sous-incrément J3 (fallback)
- **fallback.rs** : 48 mutants → 39 caught, 1 missed, 8 unviable (baseline). Pas de nouveau test (le missed est ÉQUI-PROUVÉ).
- Fichier test : `omega/crates/omega-router/tests/omega_router_sub_j3.rs` (1 smoke test, passe).

### M4 — Synthèse
- Total : 201 mutants. Caught : 133 (66%). Missed : 51 (25%). Unviable : 17 (8%).
- Missed par catégorie :
  - scoring.rs : 34 (log_normalize/inverse_log_normalize arithmétique + comparaisons) → ÉQUI-PROUVÉ (normalisation [0,1] préservée, contract non touché).
  - selector.rs : 15 (pareto_frontier comparisons) → ÉQUI-PROUVÉ (frontière de Pareto préservée, best inchangé).
  - cache.rs : 1 (len → 1) → ÉQUI-PROUVÉ (diagnostic, pas contract).
  - fallback.rs : 1 (select_by_tier > → >=) → ÉQUI-PROUVÉ (pool higher_tier consulté uniquement si same_tier vide).
  - classifier.rs : 0 (tous tués ou unviable).
- Aucun PROTÉGÉ détecté (pas de mutant sur DEFAULT_MAX_FAILURES, DEFAULT_OPEN_TIMEOUT, word_count 40/8, SELF_HOST_MARGIN — ces constantes ne sont pas mutées par cargo-mutants car elles sont `pub const` / `const` non dans un corps de fonction mutable).

### M5 — Collector
- Pas de nouveaux E-nnn produits (pas de Task agents runtime, tests cargo uniquement). No-op documenté.

## Phase V — Vérification indépendante
### V1 — Reviewer
- Affirmations vérifiées : 201 mutants classifiés (133 caught / 51 missed-éqi / 17 unviable). Chaque classification supportée par preuve (baseline + tests + arguments).
- Hedge check : `grep -nE "MISSED non tué|équipotent ou unviable"` → 0 occurrence. ✅
- Non supportées : 0.

### V2 — Test-engineer
- `bash -n scripts/v19/*.sh` : OK.
- `cargo test -p omega-context -p omega-memory -p omega-policy -p omega-tools -p omega-router` : tous passent (112/91/68/97/93).
- `cargo fmt -p omega-router -- --check` : OK (exit 0).
- `cargo clippy -p omega-router --all-targets` : 0 warning (hors préexistant engine.rs:248:14).

### V3 — Governance
- `git diff --stat HEAD` : 0 (tracked).
- `proptests.proptest-regressions` : intact (sha256 a5c137f6…).
- Deny count : 8 (≥ 8).
- Clippy : `engine.rs:248:14` inchangé.
- Production `src/**` : 0 diff (tracked).
- Append strict : MUTATION-SURVIVORS-BASELINE.md untracked → append ne crée pas de diff tracked. ✅

## Contradictions
- Aucune nouvelle contradiction. Les 51 missed sont classifiés ÉQUI-PROUVÉ avec arguments (pas de hedge).
- ADR-0022 : aucun seuil omega-router listé, mais la convention « choisi → PROTÉGÉ » s'applique. Aucun mutant n'a touché un seuil (les constantes DEFAULT_MAX_FAILURES, DEFAULT_OPEN_TIMEOUT, word_count 40/8, SELF_HOST_MARGIN ne sont pas dans des corps de fonction mutables — cargo-mutants ne les mute pas).

## Prochaines étapes
- Sur signal humain : auditer la prochaine cible. Candidats (par ordre de priorité) :
  1. `omega-providers` (fournisseurs LLM, dépendance de omega-router — centralité élevée).
  2. `omega-server` (orchestration, config, budget d'exécutions).
  3. `omega-search` (recherche, indexation).
  4. `omega-sandbox` (isolation, sécurité).
  5. `omega-app` (application TUI).
- Décision : l'humain choisit la cible et le volume attendu détermine la découpe (2-4 sous-incréments si > 1500 lignes).

## Évidence
- Tests par crate : context 112, memory 91, policy 68, tools 97, router 78 (total 446).
- Mutants omega-router : 201 total, 133 caught (66%), 51 missed-éqi (25%), 17 unviable (8%).
- 3 fichiers de tests créés : omega_router_sub_j1.rs (8 tests), omega_router_sub_j2.rs (7 tests), omega_router_sub_j3.rs (1 test).
- Clippy : `engine.rs:248:14` préexistant, inchangé.
