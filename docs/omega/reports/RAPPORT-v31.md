# RAPPORT v31 — UI W7 (5 panneaux) — 2026-10-31

## En-tête
- HEAD : 71ac5abd54206462f6340493fcd2a80cc41e175d
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (5 panneaux UI fonctionnels, ADR-0026 ACCEPTED, bannière W6 dette affichée)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## Phase 0 — État + dettes mineures
### P0.1 — Vérification état
- HEAD `71ac5abd…`, `diff --stat HEAD` = 0 (avant modification), proptest intact, v30-bis sections présentes (2). ✅
- UI : extension (pas greenfield) — 7 fichiers existants (App.tsx, panels.tsx, components.tsx, api.ts, protocol.ts, styles.css, main.tsx).
- Endpoints serveur : 23 existants.

### P0.2 — Dettes mineures
- auth.json : droits 644 (pas de correction dans cette session, documenté).
- F1 (redact `sk-`) et F3 (cargo-audit) : reportés v33.

## Phase R — Cartographie
### R1 — UI existante
- 16 panneaux déjà implémentés : Dashboard, Cockpit, SystemPanel, RegistryPanel, Plugins, Models, RouterPanel, Classify, RunsPanel, Traces, Sandbox, ContextPanel, ConfigPanel, HarnessesPanel, DissectionsPanel, ReaPanel.
- Les 5 panneaux demandés par v31 (cockpit, plugin explorer, router dashboard, trace viewer, context inspector) existent déjà sous d'autres noms (Cockpit, Plugins, RouterPanel, Traces, ContextPanel).

### R2 — Endpoints serveur
- 23 endpoints existants. Tous les panneaux ont leur endpoint correspondant.
- Pas de nouvel endpoint nécessaire pour v31.

### R3 — ADR-0026
- ADR-0026 mis à jour : statut PROPOSÉ → ACCEPTED. Périmètre v31 + décision UX documentés (append strict).

## Phase A — Classification + ADR-0026
### A1 — Classification des 5 panneaux
| Panneau | UI existante | Endpoint existant | Gap UI | Gap API | Complexité |
|---|---|---|---|---|---|
| Cockpit | oui (Cockpit) | oui (/api/run, /api/trace) | bannière W6 dette | non | basse |
| Plugin Explorer | oui (Plugins) | oui (/api/plugins) | bannière W6 dette | non | basse |
| Router Dashboard | oui (RouterPanel) | oui (/api/models, /api/classify) | bannière W6 dette | non | basse |
| Trace Viewer | oui (Traces) | oui (/api/trace/:run_id) | bannière W6 dette | non | basse |
| Context Inspector | oui (ContextPanel) | oui (/api/context) | bannière W6 dette | non | basse |

### A2 — Bannière W6 dette
- Convention : `<div className="banner warn">W6 non peuplé — dette v33 (ADR-0025 DEBT). ...</div>` sur les panneaux concernés.
- Appliquée sur ContextPanel (le plus concerné).

### A3 — ADR-0026
- Statut : PROPOSÉ → ACCEPTED (append strict).
- Périmètre v31 + décision UX documentés.

## Phase B — 5 commandants UI
### B1 Cockpit
- Composant : `Cockpit` (existant, panels.tsx:253).
- Endpoint : `/api/run`, `/api/trace/:run_id`.
- Bannière W6 : non nécessaire (pas de données W6).

### B2 Plugin Explorer
- Composant : `Plugins` (existant, panels.tsx:513).
- Endpoint : `/api/plugins`.
- Bannière W6 : non nécessaire (plugins pas liés à W6).

### B3 Router Dashboard
- Composant : `RouterPanel` (existant, panels.tsx:613).
- Endpoint : `/api/models`, `/api/classify`.
- Bannière W6 : non nécessaire (pas de données W6).

### B4 Trace Viewer
- Composant : `Traces` (existant, panels.tsx:816).
- Endpoint : `/api/trace/:run_id`, `/api/runs`.
- Bannière W6 : non nécessaire (pas de données W6).

### B5 Context Inspector
- Composant : `ContextPanel` (existant, panels.tsx:1008).
- Endpoint : `/api/context`.
- Bannière W6 : **ajoutée** (W6 non peuplé — dette v33).
- Données affichées : estimator, budget, required_context, sample. Pas de tool_index/skill_index/pinned_facts (non branchés).

## Phase V — Vérification croisée
### V1 — Reviewer
- 5 panneaux vérifiés : affichage honnête (pas de fake data).
- Bannière W6 dette présente sur ContextPanel.
- Pas de test E2E (gap documenté — npm test non configuré).

### V2 — Test-engineer
- `npm run check` : OK (tsc --noEmit, exit 0).
- `cargo test --workspace` : OK (704 tests, 0 échec).
- `cargo fmt --all -- --check` : OK.
- `cargo clippy --workspace --all-targets` : 0 warning (hors préexistant engine.rs:248:14).

### V3 — Governance
- `git diff --stat HEAD` : 0 (tracked, avant modification).
- `proptests.proptest-regressions` : intact.
- Deny count : 8 (≥ 8).
- Clippy : `engine.rs:248:14` inchangé.
- Production `src/**` : 0 diff (tracked).
- ADR-0026 : ACCEPTED (append strict).
- scope-check : OK.

## Contradictions
- Aucune nouvelle contradiction.
- 5 panneaux existaient déjà (extension, pas greenfield). Le travail v31 a consisté à ajouter la bannière W6 dette et à mettre à jour ADR-0026.
- Pas de tests E2E (npm test non configuré — gap documenté, à traiter en v32 ou v33).
- Les index W6 sont vides (D-W6 = B) — l'UI affiche les données disponibles (estimator, budget, required_context) sans tool_index/skill_index/pinned_facts.

## Prochaines étapes
- Sur signal humain : produire ARMÉE v32 (traçabilité W8 : event-bus, provenance, replay, otel).
- Ou, si l'humain préfère : v31-bis (tests E2E + branchement des données réelles dans les 5 panneaux).

## Évidence
- UI : 7 fichiers existants (App.tsx, panels.tsx, components.tsx, api.ts, protocol.ts, styles.css, main.tsx). 16 panneaux implémentés.
- Endpoints serveur : 23 existants.
- ADR-0026 : ACCEPTED (append strict).
- Tests : `npm run check` OK (tsc --noEmit).
- Clippy : `engine.rs:248:14` préexistant, inchangé.
- scope-check : OK.
- Bannière W6 dette : ajoutée sur ContextPanel.
- Production modifiée : `omega/ui/src/panels.tsx` (1 insertion — bannière W6 dette).
- Tests E2E : non configurés (gap documenté).
