# STATE — Omega

## Current Phase

Phase 5 — Fermeture des 3 trous — **Complète** (tous les phases NOYAU A→D + 3 trous fermés).

## Last Completed

Phase 5 — Fermeture des 3 trous livrée et vérifiée.

## Milestone

v0.2 — NOYAU Phase A→D (effets réversibles, coutures, sandbox réel, industrialisation) + fermeture des 3 trous.
**Statut:** ✅ COMPLET

## Test Status

- Workspace Rust : 300+ tests verts.
- Clippy : vert.
- Graphe de dépendances : 0 cycle, 0 arête interdite.
- UI : tsc vert.

## Open Items

- W8/W9 revue indépendante en attente de quota Go (hors scope NOYAU).
- Marketplace plugins (Phase D partiel — provider externe fait, marketplace à affiner).

## Model Policy

Sous-agents sur modèles gratuits par défaut (`opencode/*-free`). Payants `opencode/*` (Zen) interdits.
Voir `docs/omega/rules/MODEL-POLICY.md` + `model_policy.py --check`.

## Résumé des phases complétées

- Phase 1 — NOYAU Phase A ✅ (effets réversibles, couches nommées, guard monotone, matrice événements)
- Phase 2 — NOYAU Phase B ✅ (MemoryStore, Hook, SubagentRunner adaptateurs + reload atomique)
- Phase 3 — NOYAU Phase C ✅ (sandbox namespace Linux, compaction LLM double couche)
- Phase 4 — NOYAU Phase D ✅ (provider externe sous-agents Claude Code/Codex/ACP)
- Phase 5 — Fermeture 3 trous ✅ (modèles ingérés, MEMORY auto-généré, INDEX racine)
