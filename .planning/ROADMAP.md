# ROADMAP — Omega

Bootstrappé depuis `docs/omega/` via `/gsd-ingest-docs` (2026-10-17). Phases dérivées
des workstreams W0–W10 (`docs/omega/02-WORKSTREAMS.md`) et du plan NOYAU-ULTIME.

## Phases

- [x] **Phase 0 — Fondation (W0)** : branche, règles, mémoire, ADR 0001-0003, CI modularité. *(fait)*
- [x] **Phase 1 — Recherche & veille (W1)** : corpus TikTok 78 vidéos, 25 harnais, benchmarks routing. *(fait)*
- [x] **Phase 2 — Socle plugin (W2)** : crates types/ports/config/registry/plugin-api/core/protocol, garde-fou zéro-cycle. *(fait — 33 tests)*
- [x] **Phase 3 — Routing intelligent (W4 inc1-3)** : classifieur déterministe, scoring, Pareto, fallback, circuit breaker, cache. *(fait — 86+ tests)*
- [x] **Phase 4 — Moteur Rust (W3)** : boucle d'agent, EventBus, exécution durable, reprise. *(fait — 109 tests)*
- [x] **Phase 5 — Économie de contexte (W6 inc1-2)** : omega-context, BM25 tool search, compaction, budget. *(fait — 40 tests)*
- [x] **Phase 6 — Traçabilité + Sandbox (W8/W9 inc1-2)** : omega-otel, ExecPolicy, sandbox, approbations. *(fait — 180 tests)*
- [x] **Phase 7 — Reverse-engineering (W5)** : 3 fiches, FEATURE-MATRIX, OMEGA-BLUEPRINT, omega-harness (15 harnais/48 principes), dissections. *(fait — 275 tests)*
- [x] **Phase 8 — Cockpit UI (W7 inc1-5)** : omega/ui (Vite+TS, 13 vues), omega-server (axum), API typée. *(fait — 265 tests)*
- [x] **Phase 9 — Économie & self-hosting (W10 inc1)** : OllamaCatalogSource, local_record, self-host-first. *(fait)*
- [ ] **Phase 10 — NOYAU Phase A (squelette)** : effets réversibles, couches nommées, guard monotone, matrice d'événements. *(prochain — coût S)*
- [ ] **Phase 11 — NOYAU Phase B (coutures)** : adaptateurs MemoryStore/Hook/SubagentRunner, reload atomique. *(coût M)*
- [ ] **Phase 12 — NOYAU Phase C (capacités lourdes)** : sandbox réel, compaction LLM double couche. *(coût M/L)*
- [ ] **Phase 13 — NOYAU Phase D (industrialisation)** : provider externe de sous-agents, marketplace plugins. *(coût L)*
- [ ] **Phase 14 — Fermeture des 3 trous** : ingérer les 3 fichiers modèles, MEMORY auto-généré, INDEX racine. *(coût S)*

## Prochaine phase à planifier

**Phase 10 — NOYAU Phase A** (4 mécanismes coût S). Commande : `/gsd-plan-phase 10`.

## Notes

- W8/W9 revue indépendante encore en attente de quota Go (voir MEMORY backlog).
- Les phases 0–9 sont déjà livrées et vérifiées (tests/workspace verts, clippy/fmt/graphe).