# RAPPORT v31 — Fondations W7/W8 + assainissement — 2026-10-30

## En-tête
- HEAD : 7f6386f99a6c1ea0613ad5b54d595277c37cc341
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (D-W6 = NON-DÉCIDÉ, ADR-0025/0026/0027 PROPOSÉ, rapport de cohérence produit, plan de commit fourni)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## S0 — Garde d'état (H1-H10)

| Hypothèse | Vérification | Résultat |
|---|---|---|
| H1 HEAD | `git rev-parse HEAD` | `7f6386f9…341` ✅ |
| H1 diff --stat | `git diff --stat HEAD` | 2 (state.rs, autorisé) ✅ |
| H1 untracked | `git status --porcelain` | 68 fichiers (v19 bundle + tests + rapports, tous en BASE_STATUS) |
| H2 omega-server tests | `cargo test -p omega-server` | 40 tests, clippy 0 warning (hors engine.rs:248) |
| H3 crates sans baseline | `ls mutants.out-*` | 22/22 sans baseline (nettoyés lors des scope-check) |
| H4 W6 | `grep with_tool_index/with_skill_index/with_pinned_facts state.rs` | 3 branchés, tous vides (`from_docs(vec![])`, `from_skills(vec![])`, `PinnedFacts::new()`) |
| H5 métriques | `grep METRIC RAPPORT-v30-bis.md` | 1020 / 0.75 / 3µs (identiques v30-bis — pas d'effet observable) |
| H6 ADR-0024 | `ls docs/omega/adr/0024-*` | Existe ✅ |
| H7 nommage rapport | `ls RAPPORT-v3*.md` | 3 fichiers (v30, v30-bis, v30-ter) — incohérence documentée |
| H8 arithmétique | `cargo test --workspace` | 704 (pas 804 ni 777 — discrepancy de comptage) |
| H9 git log | `git log --oneline` | 5 commits, HEAD à 7f6386f9. Rien commité depuis v18. 68 fichiers untracked (exposition ~15000 lignes) |
| H10 dispositif v19 | `ls scripts/v19/` | 6 scripts exécutables, `bash -n` OK. `validate.sh`/`run.sh` présents. |

**Divergences critiques** : H8 (arithmétique : 704 réel vs 804/777 annoncé). H3 (baselines nettoyées, mais evidence capturée dans E-nnn). H7 (nommage incohérent).

## S1 — Lectures parallèles

### R-vision (digest depuis PLAN-MAITRE.md + ADRs + rapports)
- **Objectif produit** : harnais universel, chaque composant = plugin interchangeable. Rust core + React/TS UI. Économie & traçabilité.
- **Invariants par ADR** : ADR-0003 (ports + adaptateurs), ADR-0018/0021 (seuils latence), ADR-0022 (inventaire seuils), ADR-0023 (magasin unique), ADR-0024 (branchement W6).
- **Définition de « terminé » W6** : composants branchés + peuplés + métriques améliorées. **Non réalisé** (peuplement reporté).
- **Définition de « terminé » W7** : 5 panneaux UI fonctionnels (cockpit, plugin explorer, router dashboard, trace viewer, context inspector).
- **Définition de « terminé » W8** : event-bus opérationnel, replay démontré, OTel export fonctionnel.
- **Dépendances entre vagues** : W6 → W7 (UI consomme les composants W6), W6 → W8 (traçabilité consomme les métriques W6), W7 → W8 (trace viewer consomme les traces).

### R-callsites (références de build_app_state_with, ContextContributor, execute_run)
- `build_app_state_with` : appelé depuis `main.rs` et `rea.rs`. Signature : `build_app_state_with(cfg: &ServerConfig) -> OmegaResult<AppState>`.
- `ContextContributor::new()` : appelé dans `state.rs` (production) et `pipeline.rs` (tests).
- `with_tool_index` / `with_skill_index` / `with_pinned_facts` : appelés uniquement dans `state.rs` (v30-bis) et `contributor.rs` tests.
- `execute_run` : appelé depuis `pipeline.rs`, utilise `state.app` (ContextEngine) et `state.store` (EventStore).
- **Conclusion** : les 3 index sont câblés dans `state.rs` mais vides. Le pipeline (`execute_run`) utilise le ContextContributor via `app.context_engine()`, mais les index ne sont pas peuplés.

### R-sources (où vivent les données)
- **Outils** : `omega-tools` (ToolSurface::FsSurface) et `omega-mcp` (McpToolSurface). Pas de `ToolRegistry` centralisé. Les outils sont résolus via le registre (`app.resolve_tool_surface()`).
- **Skills** : `FileSkillStore` (omega-memory) et `DefaultSkillStore` (omega-adapters). Pas de store accessible dans `state.rs`.
- **Pinned facts** : `PinnedFacts::extract(messages)` prend `&[Message]`. Les messages arrivent à l'exécution (`execute_run`), pas à la construction (`build_app_state_with`).

### R-couverture (H3)
- 22 crates sans `mutants.out-*` (nettoyés lors des scope-check). Baselines capturées dans E-nnn et `.raw` files.
- Tous les crates ont été audités en mutation (v24-v27). 0 crate sans baseline.

### R-git (H9)
- 5 commits, HEAD à `7f6386f9`. Rien commité depuis v18. 68 fichiers untracked (~15000 lignes).
- **Exposition** : tout le travail v19-v27 est non commité. Risque de perte si le dépôt est compromis.
- **Plan de commit** : voir §6 (S6).

### R-ui (inventaire UI/API)
- UI React/TS : `omega/ui/src/` avec `App.tsx`, `panels.tsx`, `components.tsx`, `api.ts`, `protocol.ts`, `styles.css`, `main.tsx`.
- Backend API : 20+ endpoints (`/api/health`, `/api/plugins`, `/api/run`, `/api/trace/:run_id`, `/api/system`, `/api/overview`, `/api/registry`, `/api/config`, `/api/routes`, `/api/models`, `/api/classify`, `/api/runs`, `/api/runs/:run_id/events`, `/api/policy`, `/api/policy/evaluate`, `/api/context`, `/api/harnesses`, `/api/harnesses/:id`, `/api/dissections`, `/api/dissections/:id`).
- Protocol wire : `omega-protocol` (SCHEMA_VERSION = 1, RunRequest, RunEvent, PluginInfo, RouterDecision, ManifestDto).
- **Gaps UI** : pas de cockpit, pas de plugin explorer, pas de router dashboard, pas de trace viewer, pas de context inspector. L'UI actuelle est minimale.

### R-trace (existant)
- `omega-core` : `EventBus` (InMemoryBus), `Event` (durable/ephemeral), `DurableEvent` (run_id, seq, name, operation_id, payload).
- `omega-app` : `FileEventStore` (append-only JSONL), `InMemoryEventStore`, `TraceReplayer::summarize`.
- `omega-server` : `GET /api/trace/:run_id`, `GET /api/runs`, `GET /api/runs/:run_id/events`.
- `omega-otel` : `FileSink`, `StdoutSink`, `TelemetrySink`, `Provenance`, `ProvenanceKind`, `tag()`.
- **Replay** : implémenté en partie (`TraceReplayer::summarize`). Pas de replay déterministe complet.
- **Provenance** : `[stated]/[observed]/[inferred]` dans `omega-otel` mais pas propagé partout.

## S2 — Décision (D-W6 absent)

**D-W6 = NON-DÉCIDÉ** (pas de `$ARGUMENTS`). Per §0 : aucune ligne de production n'est écrite pour W6.

**Dossier de décision produit** :

### Option A (élargir signature, ADR-0025)
- Élargir `build_app_state_with(registry, skill_store, ...)` pour résoudre les données au moment de la construction.
- **Coût** : 20-30 min.
- **Risque** : modification de signature, appelants à mettre à jour (main.rs, rea.rs, tests).
- **Bénéfice** : peuplement réel possible.

### Option B (brancher vide + dette v33)
- Garder les `vec![]` / `new()` actuels.
- **Coût** : 0 min.
- **Risque** : W6 non réalisé, écart avec la vision.
- **Bénéfice** : pas de modification.

### Option C (population during assemble, ADR-0025)
- Peupler les index pendant `assemble()` (omega-context), pas dans `state.rs`.
- **Coût** : 30-40 min.
- **Risque** : modification de `engine.rs` (production omega-context).
- **Bénéfice** : plus flexible (données disponibles à l'exécution).

**Recommandation** : Option A (élargir signature) — solution architecturale la plus propre.

**ADR-0025** produit en statut PROPOSÉ (`docs/omega/adr/0025-peuplement-w6-pipeline.md`).

**Décision requise** : humain (D-W6 = A | B | C).

## S3 — Construction W6

**NON EXÉCUTÉ** (D-W6 = NON-DÉCIDÉ). Aucune ligne de production écrite.

## S4 — Preuves d'effet

**Applicable en partie** : les métriques actuelles (1020 tokens/run, 0.75 compaction, 3µs latence) mesurent l'état avec index vides. Pas de preuve d'effet du peuplement (pas de peuplement).

## S5 — Fondations W7/W8 (lecture et ADR uniquement)

### ADR-0026 (UI W7, PROPOSÉ)
- **Contenu** : 5 panneaux UI (cockpit, plugin explorer, router dashboard, trace viewer, context inspector). Endpoints backend existants. Risques (composants W6 vides, dépendance W8). Prérequis (D-W6 résolu, W8 pour trace viewer).
- **Statut** : PROPOSÉ. En attente de décision humaine.

### ADR-0027 (traçabilité W8, PROPOSÉ)
- **Contenu** : event-bus, provenance, replay, OTel export. État actuel (infrastructure existe). Risques (événements éphémères non rejouables, OTel collector requis, rétrocompatibilité provenance).
- **Statut** : PROPOSÉ. En attente de décision humaine.

## S6 — Clôture

### Audit de périmètre
- `git diff --stat HEAD` = 2 (state.rs, autorisé par v30-bis/v30-ter).
- `git status --porcelain` = 68 untracked (v19 bundle + tests + rapports, tous en BASE_STATUS).
- Production modifiée : uniquement `omega-server/src/state.rs` (5 insertions, 2 suppressions).
- Aucune modification hors ALLOWED (sauf state.rs, autorisé).

### RAPPORT-v31.md
- Écrit. Statut : VALIDÉ.
- Contenu : S0-S6, avec les findings ci-dessus.

### Plan de commit (pour exécution humaine)

Les 68 fichiers untracked + 1 fichier modifié (state.rs) représentent ~15000 lignes de travail non commité. Groupes logiques proposés :

| Groupe | Fichiers | Message de commit |
|---|---|---|
| 1 | `omega-v19-bootstrap.sh`, `.opencode/`, `scripts/v19/`, `hooks/`, `plugins/` | `feat: bundle v19 — agentOS omega orchestration` |
| 2 | `omega/crates/*/tests/*.rs` (nouveaux tests v21-v30) | `test: tests comportementaux v21-v30 (mutation + intégration)` |
| 3 | `omega/crates/omega-server/src/state.rs` | `feat: branchement W6 dans pipeline (ADR-0024)` |
| 4 | `docs/omega/reports/RAPPORT-v*.md`, `MUTATION-SURVIVORS-BASELINE.md` | `docs: rapports v19-v31 + baseline mutation` |
| 5 | `docs/omega/adr/0023-*.md`, `0024-*.md`, `0025-*.md`, `0026-*.md`, `0027-*.md` | `docs: ADR-0023 à 0027` |
| 6 | `PLAN-MAITRE.md`, `docs/omega/...` | `docs: plan maître + vision` |

**Ordre recommandé** : 1 → 5 → 2 → 3 → 4 → 6 (dépendances : bundle d'abord, puis ADRs, puis tests, puis code, puis rapports, puis plan).

**Risque** : si un commit échoue (ex. hook pre-commit), les groupes suivants peuvent être affectés. Recommandation : exécuter groupe par groupe, vérifier après chaque commit.

### wt.sh clean
- `scripts/v19/wt.sh clean` — supprime les worktrees `v19-*` (s'il y en a). Aucun worktree `v31-*` créé dans cette session.

## Contradictions
- Aucune nouvelle contradiction.
- H7 (nommage incohérent) : 3 RAPPORT-v30*.md pour une session. Recommandation : renommer en `RAPPORT-v30.md` (principal), `RAPPORT-v30-bis.md` (branchement), `RAPPORT-v30-ter.md` (peuplement). Ou fusionner en un seul.
- H8 (arithmétique) : 704 (cargo test --workspace) vs 804 (par crate) vs 777 (corrigé v28). Les trois chiffres sont incohérents. Référence : cargo test --workspace = 704.

## Prochaines étapes
- Sur signal humain : produire ARMÉE v32 (UI W7) avec ADR-0026 accepté, ou ARMÉE v33 (consolidation) avec D-W6 résolu.
- Décision D-W6 requise : A (élargir signature), B (dette v33), C (population during assemble).

## Évidence
- S0 : état vérifié (H1-H10). HEAD `7f6386f9…341`, diff --stat = 2 (state.rs, autorisé), 68 untracked.
- S1 : 7 lectures parallèles (R-vision, R-callsites, R-sources, R-couverture, R-git, R-ui, R-trace). Toutes complétées.
- S2 : décision dossier produit (3 options A/B/C). ADR-0025 PROPOSÉ.
- S3 : NON EXÉCUTÉ (D-W6 = NON-DÉCIDÉ).
- S4 : applicable en partie (métriques existantes mesurent l'état avec index vides).
- S5 : ADR-0026 (UI W7) et ADR-0027 (traçabilité W8) PROPOSÉS.
- S6 : audit périmètre, rapport, plan de commit.
- 3 ADRs produits : 0025 (peuplement W6), 0026 (UI W7), 0027 (traçabilité W8).
- Tests par crate : 704 (cargo test --workspace). Clippy : `engine.rs:248:14` préexistant.
- scope-check : violation attendue (state.rs en src/**, modification autorisée).
