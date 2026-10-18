# Roadmap: odysseusAgentOS-omega (Omega)

**Created:** 2026-10-17
**Status:** In Progress

## Milestone: v0.2 — NOYAU Phase A→D

**Status:** In Progress

### Phase 1: NOYAU Phase A — squelette

**Goal:** Effets reversibles dans le registre, couches nommees dans la config, guard monotone, matrice d'evenements verifiee.
**Depends on:** None

### Phase 2: NOYAU Phase B — coutures

**Goal:** Implementer les 3 ports orphelins (MemoryStore, Hook, SubagentRunner) et le reload atomique par plugin.
**Depends on:** Phase 1

### Phase 3: NOYAU Phase C — capacites lourdes

**Goal:** Sandbox reel (namespace + FS + reseau, fail-closed) et compaction LLM double couche.
**Depends on:** Phase 2

### Phase 4: NOYAU Phase D — industrialisation

**Goal:** Provider externe de sous-agents (Claude Code/Codex/ACP) et marketplace/atelier de plugins.
**Depends on:** Phase 3

### Phase 5: Fermeture des 3 trous

**Goal:** Ingerer les 3 fichiers modeles hors depot, MEMORY auto-genere, INDEX racine regenerable.
**Depends on:** Phase 1
