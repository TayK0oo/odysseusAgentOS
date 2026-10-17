# STATE — Omega

## Current Phase

Phase 10 — NOYAU Phase A (squelette) — **à planifier**.

## Last Completed

Phase 9 — Économie & self-hosting (W10 inc1) livrée et vérifiée.

## Milestone

v0.1 — noyau plugin + reverse-eng + UI + routing + contexte + traçabilité + sandbox.
Prochain milestone : v0.2 — NOYAU Phase A→D (effets réversibles, coutures, sandbox réel).

## Test Status

- Workspace Rust : 180+ tests verts (dernier comptage MEMORY).
- Clippy / fmt / graphe de dépendances : verts.
- UI : tsc vert, vérifié au navigateur.

## Open Items

- W8/W9 revue indépendante en attente de quota Go.
- 3 trous à fermer (fichiers modèles hors dépôt, MEMORY manuel, pas d'index racine).
- Phase 10 à planifier.

## Model Policy

Sous-agents sur modèles gratuits par défaut (`opencode/*-free`). Payants `opencode/*` (Zen) interdits.
Voir `docs/omega/rules/MODEL-POLICY.md` + `model_policy.py --check`.