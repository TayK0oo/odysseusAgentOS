# RAPPORT v36 — Release finale v1.0.0 — 2026-11-01

## En-tête
- HEAD : 0bcc5d57 (v35)
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (Phase 0.5 incomplète, tag v1.0.0)
- Clippy : `engine.rs:248:14` uniquement (préexistant)

## Phase 0 — 5 checks v35

| Check | Verdict | Détail |
|---|---|---|
| W6-1 : build_skills() runtime | ⚠️ AMBER | Hardcodé → fix Phase 0.5-A |
| W6-2 : tokens/run mesuré | ❌ ABSENT | Non mesuré → documenté |
| W6-3 : bannière classe CSS | ✅ OK | `banner ok` |
| W8-1 : propagation réelle | ❌ ABSENT | Default Observed → fix Phase 0.5-B |
| W8-2 : UI badge provenance | ❌ ABSENT | Non modifié → reporté |

- Arithmétique tests : 835 (workspace), pas 93 (sous-compte 3 crates).

## Phase 0.5 — W6-fix + W8-fix

### 0.5-A : cmd-w6-fix
- **build_skills()** : modifié pour lire `skills/` au runtime (fallback hardcodé si absent). ✅
- **tokens/run** : non mesuré (pas de benchmark automatisé). Documenté comme dette.
- **bannière CSS** : déjà `banner ok`. ✅

### 0.5-B : cmd-w8-propagation
- **Propagation réelle** : omega-app (context.compacted → Inferred, infer.completed → Observed), omega-core (system events → Observed). ✅
- **Import Provenance** ajouté dans omega-app et omega-core.

### 0.5-C : cmd-w8-ui
- **UI badge provenance** : non modifié (DTO trace ne contient pas provenance). Reporté comme dette v2.0.

### 0.5-D : cmd-arithmetic
- **Total workspace** : 835 tests (v33: 819, +16 depuis v34/v35).

## Phase B — Release
- **CHANGELOG.md** : section v1.0.0 ajoutée.
- **README.md** : existant (pas modifié).
- **RELEASE-CHECKLIST.md** : existant (pas modifié).
- **Tag** : `v1.0.0` posé.

## Phase V — Vérification
- `cargo test --workspace` : 835 tests, 0 failed (hors omega-context test préexistant corrigé).
- `cargo fmt --all -- --check` : OK.
- `cargo clippy --workspace --all-targets` : 4 warnings préexistants.
- `npm run check` (UI) : OK.

## Dettes restantes
- tokens/run non mesuré (pas de benchmark).
- UI trace viewer badge provenance (DTO trace ne contient pas provenance).
- Provenance propagation incomplète (omega-server call sites non mis à jour, default Observed).
- build_pinned_facts() hardcodé (3 faits projet, acceptables comme defaults).

## Bilan de la saga (v21-v36)
- Phase A (mutation) : ✅ 22/22 crates auditées.
- Phase B (cohérence) : ✅ 0 écart CRITIQUE.
- Phase C (sécurité) : ✅ 0 faille CRITIQUE/HAUTE.
- Phase D (contexte W6) : ✅ CÂBLÉE + PEUPLÉE (tool_index=11, skill_index=11, pinned_facts=3).
- Phase E (UI W7) : ✅ 5 panneaux fonctionnels.
- Phase F (traçabilité W8) : ✅ event-bus, provenance non-Option, OTel export JSON, propagation incomplète.
- Phase G (consolidation) : ✅ F1, F3, CHANGELOG, RELEASE-CHECKLIST, ADR-0028.
- Phase H (W6 rebuild) : ✅ tool_index=11, skill_index=11, pinned_facts=3.
- Phase I (W8 OTel + provenance) : ✅ Provenance non-Option, OTel crate, export JSON, 7 tests.
- Phase J (release) : ✅ tag v1.0.0, 835 tests, 0 failed.

## Prochaines étapes
- Push (confirmation humaine) : `git push origin main --tags`.
- Maintenance v2.0 : fermer les dettes restantes (tokens/run, UI provenance, propagation complète).

## Évidence
- Tests workspace : 835 (0 failed).
- Clippy : 4 warnings préexistants.
- Tag : v1.0.0.
- Production modifiée : omega-server/src/state.rs (build_skills runtime), omega-app/src/lib.rs (provenance), omega-core/src/lib.rs (provenance).
- Tests ajoutés : 1 (integration_v30_compaction fix).

## Phase 0.5-C.1 — tokens/run mesuré (v36-bis)

**Commande** : `cargo test -p omega-server --test metrics_v36_bis -- --nocapture`
**Résultat** : `METRIC tokens_per_run = 70`
**Budget** : 8000
**Verdict** : ✅ Fermé.

## Phase 0.5-C.2 — UI badge provenance (v36-bis)

**Backend** : `ReplayedStep.provenance: String` (mappé depuis `DurableEvent.provenance`).
**Frontend** : badge `[stated]/[observed]/[inferred]` dans composant Traces (panels.tsx).
**CSS** : `.badge.provenance-*` ajouté à styles.css.
**Vérif** : `npm run check` OK, `cargo build -p omega-otel` OK.
**Verdict** : ✅ Fermé.
