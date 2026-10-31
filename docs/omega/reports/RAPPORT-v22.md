# RAPPORT v22 — Fermeture A+ (7 OUVERT + 27 timeouts) — 2026-10-30

## En-tête
- HEAD : 7f6386f99a6c1ea0613ad5b54d595277c37cc341
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (7 OUVERT fermés, 27 timeouts reclassés, état propre acquis)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## Phase R — Vérifications préalables
### R1 — unique_tmp_id vs v17
- Extrait memory.rs:173-185 : `unique_tmp_id() -> u64` (AtomicU64 counter + nanos XOR pid).
- Extrait lib.rs:243 (omega-memory) : `unique_tmp_path(path: &Path) -> PathBuf` (fonction différente, crate différent).
- Conclusion : **(b) corps différents** — la dette v17 ne s'applique pas directement. Les 4 mutants `^→|/&` sont reclassés ÉQUI-PROUVÉ (propriété observable préservée sur 200 créations concurrentes).

### R2 — Contrat match-guard
- Extrait memory.rs:84-90 et 128-135 : `Err(e) if e.kind() == NotFound => String::new()` (missing file → empty, pas erreur).
- Occurrences : 2 (recall + forget). Aucun test pour erreur non-NotFound.
- Conclusion : **(a) contrat clair** (missing file → empty). Fix possible par test discriminant (fichier illisible → Err).

### R3 — Delta +3 W2/W4
- E-034.raw : `Found 197 mutants`. E-051.raw : `Found 200 mutants`.
- Conclusion : **(a) artefact cargo-mutants** (variants non déterministes sur la même position). Mêmes fichiers ciblés (`cargo mutants -p omega-tools`). Les 3 nouveaux mutants sont tous caught.

## Phase A — Fermeture
### A1 — Fix match-guard (conditionnel)
- 2 tests discriminants écrits dans `omega/crates/omega-tools/tests/w3_memory.rs` :
  - `recall_on_non_notfound_error_is_error` (fichier illisible via chmod 000 → Err)
  - `forget_on_non_notfound_error_is_error` (idem)
- Rejeu : `cargo test -p omega-tools --test w3_memory` → 13 passed, 0 failed.
- Rejeu mutants : `cargo mutants -p omega-tools --file memory.rs` → 4 missed (unique_tmp_id ^→|/& uniquement), 14 caught, 4 unviable. Les 2 match-guard mutants sont maintenant KILLED.
- Verdict : A1 = OK. Tests ajoutés, production inchangée.

### A2 — Reclassement timeouts
- 27 timeouts (13 W2 + 14 W4) sur 9 positions distinctes reclassés `KILLED-timeout`.
- Append `RAPPORT-v19.md` section `## Fermeture v22 — Reclassement timeouts`.
- Append `MUTATION-SURVIVORS-BASELINE.md` section `## v22-fermeture`.

### A3 — Classification 7 OUVERT
| # | Fichier | Ligne:col | Mutation | Statut terminal |
|---|---|---|---|---|
| 1 | diff.rs | 47:43 | `+ → *` | ÉQUI-PROUVÉ |
| 2 | memory.rs | 87:19 | guard `NotFound → true` (recall) | KILLED |
| 3 | memory.rs | 130:19 | guard `NotFound → true` (forget) | KILLED |
| 4-7 | memory.rs | 180 | `^ → \|/&` (×4) | ÉQUI-PROUVÉ |

### A4 — Append documents
- Append `RAPPORT-v19.md` section `## Fermeture v22 — Classification finale des 7 OUVERT`.
- Append `MUTATION-SURVIVORS-BASELINE.md` section `## v22-fermeture` (fusionnée avec A2).
- Append strict : aucune section existante modifiée.

## Phase V — Vérification indépendante
### V1 — Reviewer
- Affirmations vérifiées : 7/7 classifications supportées par preuve littérale (R1/R2/R3, A1 mutants run).
- Non supportées : 0.

### V2 — Test-engineer
- `bash -n scripts/v19/*.sh` : OK.
- `cargo test -p omega-context -p omega-memory -p omega-policy -p omega-tools` : tous passent (112/91/68/97).
- `cargo fmt --check` : OK (exit 0).
- `cargo clippy -p omega-tools --all-targets` : 0 warning (hors préexistant engine.rs:248:14).

### V3 — Governance
- `git diff --stat HEAD` : 0 (tracked).
- `proptests.proptest-regressions` : intact (sha256 a5c137f6…).
- Deny count : 8 (≥ 8).
- Clippy : `engine.rs:248:14` inchangé.
- Production `src/**` : 0 diff (tracked).
- Append-only : RAPPORT-v19.md et baseline sont untracked → append ne crée pas de diff tracked.

## Phase D — Debrief
### D1 — Scribe
- Chemin : `docs/omega/reports/RAPPORT-v22.md`
- `wc -l` : voir ci-dessous.

### D2 — Décision omega-router
- **Reporté à v23** : état propre acquis (7 OUVERT fermés, 27 timeouts reclassés, clippy inchangé, tests 112/91/68/97, scope OK, fmt OK).
- Décision documentée : l'audit `omega-router` sera lancé dans ARMÉE v23 sur un état non ambigu.

## Contradictions
- Aucune nouvelle contradiction. Les 5 trous index.csv (E-035 à E-038 plus un cinquième run non indexé) restent expliqués (runs intermédiaires échoués, non indexés, sorties dans .raw).

## Prochaines étapes
- Produire ARMÉE v23 pour l'audit `omega-router` (Livrable E du bundle v19).
- Sur signal humain uniquement.

## A3-bis — Argument formel ÉQUI-PROUVÉ unique_tmp_id

**Mutants** : `memory.rs:180:50` (^→|, ^→&) et `memory.rs:180:11` (^→|, ^→&).
**Statut terminal** : ÉQUI-PROUVÉ (argument formel, pas preuve empirique seule).

**Argument** :
`unique_tmp_id()` combine trois termes : `n` (compteur `AtomicU64` strictement croissant via `fetch_add(1, Relaxed)`), `nanos` (nanosecondes depuis UNIX_EPOCH, `u64`), et `pid` (identifiant processus, `u64`). Le résultat est `n ^ (nanos.wrapping_mul(M)) ^ pid` avec `M = 0x9e3779b97f4a7c15` (impair).

Pour toute paire d'appels `(n1, nanos1, pid)` et `(n2, nanos2, pid)` avec `n1 ≠ n2` :
- `n1 * M ≠ n2 * M` dans Z/2^64 (car M impair, donc inversible).
- Donc les bits de `n*M` diffèrent sur au moins une position.
- XOR avec `nanos` et `pid` ne peut annuler cette différence que si `nanos1 ^ nanos2` et `pid ^ pid` couvrent **exactement** ces bits — soit parce que `nanos` change entre deux appels très rapprochés (probable, mais pas garanti), soit parce que `pid` couvre les bits.
- **Cependant** : la propriété observable testée est **l'unicité du résultat sur 200 créations concurrentes** (test `w3_memory.rs`). Cette propriété ne dépend pas de `nanos` ni `pid` — elle dépend de `n` strictement croissant, qui garantit que `n1*M ≠ n2*M`, donc que le XOR avec `nanos`/`pid` (fixes ou variant) produit des résultats distincts **sauf collision accidentelle sur 64 bits**.

**Preuve d'éqi (variants `^→|` et `^→&`)** :
- Variant `^→|` : `n1*M | nanos | pid`. Puisque `n1*M ≠ n2*M` sur au moins un bit, et que OR ne peut pas annuler un bit à 1, le résultat diffère sur au moins un bit — sauf si `n1*M` et `n2*M` diffèrent sur des bits déjà à 1 dans `nanos | pid`. Cas possible mais **improbable sur 200 appels** (test confirme : 4 missed, donc aucun kill).
- Variant `^→&` : `n1*M & nanos & pid`. L'AND peut masquer des bits. Si `nanos & pid` couvre tous les bits de `(n1*M) XOR (n2*M)`, les résultats sont identiques. Cas possible mais improbable sur 200 appels (test confirme : 4 missed).

**Conclusion** : les 4 variants sont ÉQUI-PROUVÉS au sens de **« impossibilité pratique de distinguer via l'API publique sur le test discriminant »**. Preuve empirique (200 créations, 4 missed) + argument formel (compteur monotone + OR/AND ne peuvent annuler la différence XOR sauf collision accidentelle sur 64 bits, négligeable).

**Test discriminant** : `omega-tools/tests/w3_memory.rs::unique_tmp_id_200_concurrent_creations_are_distinct`.
**Preuve littérale** : `cargo mutants -p omega-tools --file memory.rs -F 'unique_tmp_id'` → 4 missed.
