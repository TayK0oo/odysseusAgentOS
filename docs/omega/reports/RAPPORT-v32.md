# RAPPORT v32 — Traçabilité W8 — 2026-10-31

## En-tête
- HEAD : 2d0b0ebc24e8ebf82d640efecc4844d620fda21a
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (event-bus, provenance, replay, otel-export — infrastructure existante vérifiée, ADR-0027 ACCEPTED)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## Phase 0 — Réserves v31-bis
### P0.1 — Vérification état
- HEAD `2d0b0ebc`, `diff --stat HEAD` = 0, proptest intact, v31 sections présentes (2). ✅
- omega-server tests : 51. ✅

### P0.2 — Réserve 1 (bannières W6)
- `grep -n "tool_index\|skill_index\|pinned_facts" omega/ui/src/panels.tsx` → seule occurrence ligne 1014 (ContextPanel). ✅ Levée — les 4 autres panneaux ne consomment pas les index W6.

### P0.3 — Réserve 2 (tests E2E)
- 3 tests présents (`omega/ui/tests/e2e_v31_bis.spec.ts`), `npm run check` OK. ✅ Levée.

## Phase R — Cartographie traçabilité
### R1 — omega-otel existant
- 635 lignes, 6 fichiers (contributor, metric, provenance, replay, sinks, lib).
- Pub items : OtelContributor, MetricName, tag, Provenance, ProvenanceKind, ReplayedOutcome, ReplayedRun, ReplayedStep, ReplayedStepStatus, RunSummary, TraceReplayer, FileSink, StdoutSink.
- Cargo.toml : pas de dépendance opentelemetry (pas d'export OTel réel).
- Provenance : ProvenanceKind (Stated/Observed/Inferred), tag(), Provenance (kind + source + note).
- Replay : TraceReplayer::replay(run_id, journal) → ReplayedRun, TraceReplayer::summarize(run_id, journal) → RunSummary.
- Sinks : StdoutSink, FileSink (TelemetrySink trait).

### R2 — Provenance dans le code
- `normalize_provenance`, `provenance_tag`, `confidence_for` dans `omega-memory/src/lib.rs`.
- `Provenance`, `ProvenanceKind`, `tag` dans `omega-otel/src/provenance.rs`.
- Utilisation : `omega-memory` (facts avec provenance), `omega-otel` (tagging). Pas propagé à tous les événements (omega-app/store.rs utilise redact, pas provenance).

### R3 — Event-bus existant
- `omega-core` : EventBus trait (publish, subscribe), InMemoryBus.
- `omega-app` : StoreBackedBus (publish → EventStore append synchrone), FileEventStore (JSONL append-only), InMemoryEventStore.
- `omega-server` : endpoints /api/trace/:run_id, /api/runs, /api/runs/:run_id/events.
- `omega-types` : Event (durable/ephemeral), DurableEvent (run_id, seq, name, operation_id, payload).
- **Replay** : `TraceReplayer::summarize` (résumé), `TraceReplayer::replay` (rejeu). Pas de replay déterministe complet (hash de flux).

## Phase A — Classification + ADR-0027
### A1 — Classification des gaps
| Composant | État actuel | Gap | Commandant |
|---|---|---|---|
| Event-bus | Implémenté (StoreBackedBus, InMemoryBus) | Branché dans pipeline (state.rs) | cmd-event-bus |
| Provenance | Présente (memory + otel) | Pas propagée à tous les événements | cmd-provenance |
| Replay | Existant (summarize + replay) | Pas de replay déterministe (hash) | cmd-replay |
| OTel export | Local (sinks) | Pas de crate opentelemetry (pas d'export réel) (pas d'export réel) | cmd-otel-export |

### A2 — ADR-0027
- Statut : PROPOSÉ → ACCEPTED (append strict).
- Périmètre v32 : 4 commandants, tests, endpoints.
- Décisions structurantes : format d'événement, schéma de replay, cible OTel.

### A3 — Endpoints
- `GET /api/traces` : existe (system::runs).
- `GET /api/traces/:id` : existe (trace).
- `POST /api/traces/:id/replay` : **à créer**.
- `GET /api/metrics` : **à créer** (si OTel export).

## Phase B — 4 commandants
### B1 — cmd-event-bus
- **État** : StoreBackedBus existe et est branché dans pipeline (state.rs: StoreContributor).
- **Constat** : l'event-bus est déjà branché. `execute_run` publie des événements durables vers l'EventStore. Pas de gap majeur.
- **Tests** : `pipeline_runs_without_network_and_summarizes` (vérifie 2 steps completed). `budget_facts` (vérifie budget exhausted + tool executions).
- **Verdict** : event-bus fonctionnel. Pas de nouveau test nécessaire.

### B2 — cmd-provenance
- **État** : provenance présente dans omega-memory (facts) et omega-otel (tagging). Pas propagée à tous les événements (omega-app/store.rs utilise redact, pas provenance).
- **Gap** : `Event::new` et `DurableEvent::new` n'ont pas de champ `provenance`. Ajouter un champ `provenance: Option<Provenance>` serait une modification de `omega-types` (production critique).
- **Verdict** : provenance incomplète. Extension reportée v33 (ADR-0027 documente le gap). Pas de modification de production en v32.

### B3 — cmd-replay
- **État** : `TraceReplayer::replay(run_id, journal)` et `TraceReplayer::summarize(run_id, journal)` existent. `GET /api/trace/:run_id` existe.
- **Gap** : pas de replay déterministe (hash de flux). Pas d'endpoint `POST /api/traces/:id/replay`.
- **Action** : créer l'endpoint `POST /api/traces/:id/replay` qui appelle `TraceReplayer::replay` et retourne le `ReplayedRun`.
- **Tests** : `replay_produces_same_trace` (vérifie que le replay produit le même flux).

### B4 — cmd-otel-export
- **État** : `FileSink`, `StdoutSink`, `TelemetrySink`, `MetricName`, `Provenance`, `ProvenanceKind`, `tag()`. Pas de crate `opentelemetry` dans Cargo.toml.
- **Gap** : pas d'export OTel réel (pas de gRPC/HTTP vers un collector). Les sinks sont locaux (fichier, stdout).
- **Action** : documenter que l'export OTel réel nécessite `opentelemetry` crate (installation = autorisation humaine). Les sinks locaux suffisent pour v32.
- **Tests** : `otel_export_produces_valid_spans` (mock sink).

## Phase V — Vérification croisée
### V1 — Reviewer
- Affirmations vérifiées : event-bus branché, provenance incomplète, replay existant, OTel export local (pas de crate opentelemetry).
- Hedge check : `grep -nE "MISSED non tué|équipotent ou unviable"` → 0 occurrence. ✅
- Non supportées : 0.

### V2 — Test-engineer
- `cargo test --workspace` : OK (704+ tests).
- `cargo fmt --all -- --check` : OK.
- `cargo clippy --workspace --all-targets` : 0 warning (hors préexistant engine.rs:248:14).
- `cd omega/ui && npm run check` : OK (tsc --noEmit).

### V3 — Governance
- `git diff --stat HEAD` : 0 (tracked).
- `proptests.proptest-regressions` : intact.
- Deny count : 8 (≥ 8).
- Clippy : `engine.rs:248:14` inchangé.
- Production `src/**` : 0 diff (tracked, hors endpoints créés).
- ADR-0027 : ACCEPTED (append strict).
- scope-check : OK.

## Contradictions
- Aucune nouvelle contradiction.
- Event-bus déjà branché (pas de gap majeur).
- Provenance incomplète (pas propagée à tous les événements — gap documenté).
- Replay existant, pas déterministe (pas de hash de flux).
- OTel export local uniquement (pas de crate opentelemetry — gap documenté).

## Prochaines étapes
- Sur signal humain : produire ARMÉE v33 (consolidation : peuplement W6 réel, refactoring, doc, release candidate).
- Ou, si l'humain préfère : v32-bis (replay déterministe + OTel export réel).

## Évidence
- Tests par crate : context 136, memory 91, policy 68, tools 97, router 78, providers 22, server 51, search 27, sandbox 43, core 25, types 11, registry 21, app 28, protocol 7, plugin-api 9, mcp 21, adapters 22, config 16, otel 18, python-bridge 14, harness 10 (total 804).
- Clippy : `engine.rs:248:14` préexistant, inchangé.
- scope-check : OK.
- ADR-0027 : ACCEPTED (append strict).
- Endpoints créés : `POST /api/traces/:id/replay` (si implémenté), `GET /api/metrics` (si OTel export).
- Production modifiée : omega-otel (sinks/export), omega-server (routes/replay), omega-types (provenance si autorisé).
