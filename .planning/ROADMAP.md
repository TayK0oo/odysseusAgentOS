# Roadmap: odysseusAgentOS-omega (Omega)

**Milestone:** v0.2 — NOYAU Phase A→D
**Created:** 2026-10-17
**Status:** In Progress

## Phase 1: NOYAU Phase A — squelette ✅

**Goal:** Effets réversibles dans le registre, couches nommées dans la config, guard monotone, matrice d'événements vérifiée.
**Depends on:** None

## Phase 2: NOYAU Phase B — coutures ✅

**Goal:** Implémenter les 3 ports orphelins (MemoryStore, Hook, SubagentRunner) et le reload atomique par plugin.
**Depends on:** Phase 1

## Phase 3: NOYAU Phase C — capacités lourdes ✅

**Goal:** Sandbox réel (namespace + FS + réseau, fail-closed) et compaction LLM double couche.
**Depends on:** Phase 2

## Phase 4: NOYAU Phase D — industrialisation

**Goal:** Provider externe de sous-agents (Claude Code/Codex/ACP) + marketplace/atelier de plugins.
**Depends on:** Phase 3

## Phase 5: Fermeture des 3 trous

**Goal:** Ingérer les 3 fichiers modèles hors dépôt, MEMORY auto-généré, INDEX racine régénérable.
**Depends on:** Phase 1
