# REQUIREMENTS — Omega

Bootstrappé depuis `docs/omega/` (VISION, WORKSTREAMS, MEMORY, ADR, NOYAU-ULTIME,
OMEGA-COMPLET) via `/gsd-ingest-docs` (2026-10-17).

## Fonctionnelles

### R1 — Socle plugin (W2)
- R1.1 Ports typés : `LlmProvider`, `ContextEngine`, `MemoryStore`, `ToolExecutor`,
  `SandboxBackend`, `RouterPolicy`, `CatalogSource`, `TelemetrySink`, `UiPanel`, `Hook`,
  `SubagentRunner`.
- R1.2 Deux régimes de capacité : slot exclusif et ensemble concurrent.
- R1.3 Manifeste schéma-validé (id, capacité, version, config schema, permissions).
- R1.4 Découverte dynamique (scan répertoires + registre), pas d'import en dur.
- R1.5 Effets réversibles : toute inscription retourne un disposer.
- R1.6 Couches nommées de config (provenance de valeur).
- R1.7 Reload atomique par plugin.
- R1.8 Guard monotone : les politiques ne font qu'interdire, jamais élargir.
- R1.9 Matrice d'événements vérifiée (producteurs/consommateurs).

### R2 — Moteur Rust (W3)
- R2.1 Boucle d'agent `Operation/Inference/Effect` + `EventBus` durable/éphémère.
- R2.2 Annulation coopérative.
- R2.3 Exécution durable : reprise après crash sans perte.
- R2.4 Event store append-only (mem + fichier JSONL), replay de session.

### R3 — Routing intelligent (W4)
- R3.1 Registre multi-fournisseurs avec capacités, coûts, vitesses, axes de qualité.
- R3.2 Classifieur de tâche déterministe (zéro LLM).
- R3.3 Scoring composite depuis benchmarks communautaires (AA, Epoch, SWE-bench, …).
- R3.4 Sélecteur Pareto + fallback (même tier → supérieur → self-hosted → backoff) + circuit breaker.
- R3.5 Feedback loop (bandit UCB1/Thompson) ajustant les poids sur succès réels.
- R3.6 Cache de décision versionné.
- R3.7 Souveraineté : blocage fournisseur → bascule self-hosted.

### R4 — Reverse-engineering (W5)
- R4.1 Programme reproductible en 4 étapes (recensement → intake+REA → dissection → combiné).
- R4.2 Fiches par harnais (35), `FEATURE-MATRIX` (96+ features), `OMEGA-BLUEPRINT` (18 crates).
- R4.3 `omega-harness` crate + endpoints `/api/harnesses`, `/api/dissections`.
- R4.4 Validation structure + résolution des preuves (`validate_dissections.py`).

### R5 — Économie de contexte (W6)
- R5.1 Port `ContextEngine` (ingest → assemble → compact → afterTurn, `ownsCompaction`).
- R5.2 Tool search BM25 + rarest-token gate.
- R5.3 Progressive disclosure des skills (3 niveaux).
- R5.4 Prompt caching (breakpoints + affinité).
- R5.5 Budget de contexte (estimate → plafond → arrêt).
- R5.6 Mémoire bornée + FTS.

### R6 — Cockpit UI React/TS (W7)
- R6.1 App `omega/ui/` (Vite+TS) avec Cockpit, Plugin Explorer, Router Dashboard,
  Trace Viewer/Replay, Context Inspector.
- R6.2 Contrats générés depuis `omega-protocol`.
- R6.3 `omega-server` (axum) exposant l'API typée, SPA fallback.
- R6.4 Remplacement d'un plugin depuis l'UI + visualisation décision/coût.

### R7 — Traçabilité (W8)
- R7.1 `TelemetrySink` (OTLP/stdout/fichier), event store append-only.
- R7.2 Provenance `[stated]/[observed]/[inferred]`.
- R7.3 Replay de session depuis les traces.

### R8 — Sécurité & sandbox (W9)
- R8.1 `SandboxBackend` multi-backend (local/docker/ssh/serverless).
- R8.2 `ExecPolicy` DSL, `PermissionPolicy` (allow/ask/deny).
- R8.3 Approbations granulaires.
- R8.4 Fail-closed : absence de canal/provider/décision = refus.

### R9 — Économie & self-hosting (W10)
- R9.1 Providers locaux (Ollama/llama.cpp), gateway, cache.
- R9.2 Budget par run.
- R9.3 Flux complet à coût nul via modèle local sans toucher au noyau.

### R10 — Adaptateurs des ports orphelins (NOYAU-ULTIME Phase B)
- R10.1 `MemoryStore`, `Hook`, `SubagentRunner` implémentés (actuellement orphelins).
- R10.2 Provider externe de sous-agents (Claude Code/Codex/ACP comme providers).

## Non-fonctionnelles

- NF1 Interchangeabilité : remplacer une brique sans toucher au noyau.
- NF2 Économie : part des opérations résolues sans appel LLM ; tokens/run ; coût/run.
- NF3 Qualité : taux de tâches terminées sans intervention ; taux de fallback.
- NF4 Traçabilité : % d'étapes tracées et rejouables.
- NF5 Vitesse : TTFT p50/p95 ; latence de routing p95.
- NF6 Robustesse : taux de crash par brique ; récupération après erreur.
- NF7 Dépendances : `cli/server → runtime → ports → core` ; jamais l'inverse.
- NF8 Zéro-cycle dans le graphe de dépendances (`check_dep_graph.py`).
- NF9 Modèles : sous-agents gratuits par défaut (MODEL-POLICY).

## Backlog (issues ouvertes)

- W8/W9 revue indépendante en attente de quota Go.
- Phase A du noyau (4 mécanismes coût S) : prochain pas logique.
- Fermer les 3 trous : ingérer les 3 fichiers modèles hors dépôt, MEMORY auto-généré, INDEX racine.