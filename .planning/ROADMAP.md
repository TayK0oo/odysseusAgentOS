# ROADMAP — Omega

Bootstrappé depuis `docs/omega/` via `/gsd-ingest-docs` (2026-10-17).

## Completed (already shipped — not tracked by GSD)

Phases 0–9 du plan Omega sont livrées et vérifiées (tests workspace verts, clippy/fmt/graphe).
Détail : `docs/omega/MEMORY.md` §8 (backlog) et `docs/omega/02-WORKSTREAMS.md`.

- Phase 0 — Fondation (W0)
- Phase 1 — Recherche & veille (W1)
- Phase 2 — Socle plugin (W2) — 33 tests
- Phase 3 — Routing intelligent (W4) — 86+ tests
- Phase 4 — Moteur Rust (W3) — 109 tests
- Phase 5 — Économie de contexte (W6) — 40 tests
- Phase 6 — Traçabilité + Sandbox (W8/W9) — 180 tests
- Phase 7 — Reverse-engineering (W5) — 275 tests
- Phase 8 — Cockpit UI (W7) — 265 tests
- Phase 9 — Économie & self-hosting (W10 inc1)

## Remaining (tracked by GSD)

- [ ] **Phase 1 — NOYAU Phase A (squelette)** : effets réversibles, couches nommées, guard monotone, matrice d'événements. *(coût S — prochain)*
- [ ] **Phase 2 — NOYAU Phase B (coutures)** : adaptateurs MemoryStore/Hook/SubagentRunner, reload atomique. *(coût M)*
- [ ] **Phase 3 — NOYAU Phase C (capacités lourdes)** : sandbox réel, compaction LLM double couche. *(coût M/L)*
- [ ] **Phase 4 — NOYAU Phase D (industrialisation)** : provider externe de sous-agents, marketplace plugins. *(coût L)*
- [ ] **Phase 5 — Fermeture des 3 trous** : ingérer les 3 fichiers modèles, MEMORY auto-généré, INDEX racine. *(coût S)*

## Prochaine phase à planifier

**Phase 1 — NOYAU Phase A.** Commande : `/gsd-plan-phase 1`.

## Notes

- W8/W9 revue indépendante encore en attente de quota Go (voir MEMORY backlog).
- Les phases completed sont déjà livrées ; ne pas les re-exécuter.