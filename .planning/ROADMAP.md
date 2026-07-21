# ROADMAP — Odysseus AgentOS

**Version :** 1.0 | **Date :** 2026-07-21 | **Méthodologie :** GSD (Get Shit Done)

---

## VISION

> Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, apprend de chaque run, et ne dépasse jamais ses limites.

## STRUCTURE

```
M1 — Fondations           ✅ COMPLET (code Odysseus de base)
M2 — Inventaire           ✅ COMPLET (INDEX-MAITRE, SFD vs Code)
M3 — Intégration outils   ✅ COMPLET (22 outils global-v1)
M4 — Documentation        ✅ COMPLET (refonte docs/, 41 fichiers)
M5 — Activation           🔨 EN COURS (activer kill-switches)
M6 — SFD manquants        📋 PLANIFIÉ (7 modules SFD)
```

---

## MILESTONE 1 — Fondations ✅

> Base Odysseus opérationnelle

| Phase | Description | Fichiers clés | Statut |
|-------|------------|---------------|--------|
| 1.1 | Chat agentique | `agent_loop.py:3637`, `llm_core.py:2308` | ✅ |
| 1.2 | Outils (bash, files, web) | `agent_tools/` (9), `tools/` (11) | ✅ |
| 1.3 | RAG + Mémoire | `rag_vector.py:802`, `memory_vector.py:329` | ✅ |
| 1.4 | Auth + Security | `auth.py:687`, `middleware.py:126` | ✅ |
| 1.5 | Docker Compose | `docker-compose.yml` (7 services) | ✅ |
| 1.6 | Tests | 662 fichiers, 4393 pass | ✅ |

**Livrables :** Application fonctionnelle, chat, RAG, email, calendrier, galerie, cookbook.

**Références :** `README.md`, `docs/setup.md`, `app.py`

---

## MILESTONE 2 — Inventaire et cartographie ✅

> Cartographie exhaustive du projet

| Phase | Description | Fichiers clés | Statut |
|-------|------------|---------------|--------|
| 2.1 | Exploration CBM | 11,749 nœuds, 38,126 arêtes | ✅ |
| 2.2 | Inventaire fonctionnel | `INVENTAIRE/A-INVENTAIRE-FONCTIONNEL.md` (archivé) | ✅ |
| 2.3 | SFD v3.0 | `SFD.md:1065` | ✅ |
| 2.4 | SFD vs Code | `PLANIFICATION-COMPLETE.md:251` | ✅ |
| 2.5 | INDEX-MAITRE | `.planning/INDEX-MAITRE.md:568` | ✅ |

**Livrables :** SFD v3.0, INDEX-MAITRE, PLANIFICATION-COMPLETE.

**Références :** `.planning/INDEX-MAITRE.md`, `SFD.md`, `.planning/PLANIFICATION-COMPLETE.md`, `docs/archive/inventaire/`

---

## MILESTONE 3 — Intégration outils ✅

> 22 outils intégrés en 5 vagues

| Phase | Vague | Outils | Statut |
|-------|-------|--------|--------|
| 3.1 | Quick Wins | Meilisearch, Apprise, Docling, Mem0, Tailwind, gVisor | ✅ |
| 3.2 | Observabilité | LangFuse, OpenTelemetry, Promptfoo, DeepEval, Ragas | ✅ |
| 3.3 | Refactor | LangGraph, Qdrant, Traefik, OPA, HTMX+Alpine | ✅ |
| 3.4 | Automatisation | n8n, Prefect, Letta | ✅ |
| 3.5 | Production | PostgreSQL, LocalAI, Tree-sitter | ✅ |

**Livrables :** 110 fichiers, ~12,000 lignes de code, 38 kill-switches, `docker-compose.yml` (+9 services).

**Références :** `.planning/orchestration/global-v1/MASTER-PLAN.md`, `services/`, `config/policies/`, `src/orchestrator/langgraph_loop.py`

---

## MILESTONE 4 — Documentation ✅

> Refonte complète de la documentation

| Phase | Description | Fichiers clés | Statut |
|-------|------------|---------------|--------|
| 4.1 | Hub central | `docs/README.md` | ✅ |
| 4.2 | Architecture | `docs/architecture/` (4 fichiers) | ✅ |
| 4.3 | Setup | `docs/setup/` (3 fichiers) | ✅ |
| 4.4 | Development | `docs/development/` (3 fichiers) | ✅ |
| 4.5 | Operations | `docs/operations/` (1 nouveau + 2 conservés) | ✅ |
| 4.6 | Security | `docs/security/` (3 fichiers) | ✅ |
| 4.7 | Reference | `docs/reference/` (2 fichiers) | ✅ |
| 4.8 | Intégrations | `docs/integrations/` (1 nouveau + 3 conservés) | ✅ |
| 4.9 | Nettoyage | Racine = doc officielle uniquement | ✅ |

**Livrables :** 41 fichiers docs actifs, 8 diagrammes Mermaid, archive 40+ fichiers.

**Références :** `docs/README.md`, `docs/architecture/overview.md`, `docs/archive/`

---

## MILESTONE 5 — Activation 🔨

> Activer les kill-switches et valider

| Phase | Description | Kill-switches | Effort |
|-------|------------|---------------|--------|
| 5.1 | Installer dépendances | `pip install -r requirements.txt` | 30min |
| 5.2 | Phase-lock + RRF | `PHASE_TRACKER`, `RRF_FUSION` | 2h |
| 5.3 | Mémoire intelligente | `MEM0` (déjà actif) | 1h |
| 5.4 | Full-text search | `MEILISEARCH` + Docker | 1h |
| 5.5 | LLM tracing | `LANGFUSE` + Docker | 2h |
| 5.6 | Tailwind CSS | `npm run css:build` | 2h |
| 5.7 | Gouvernance | `GOVERNANCE_ANCESTRY` | 1h |
| 5.8 | Auto-évaluation | `AUTOEVAL` | 2h |
| 5.9 | Multi-agent | 3 agents (planner, executor, verifier) | 4h |
| 5.10 | Tests + Validation | E2E, charge, régression | 8h |

**Livrables :** 10+ kill-switches activés et validés, 0 régression.

**Références :** `.planning/orchestration/global-v1/waves/`, `docker-compose.yml`

---

## MILESTONE 6 — Modules SFD manquants 📋

> Implémenter les 7 modules SFD absents

| Phase | Module SFD | Description | Effort |
|-------|-----------|------------|--------|
| 6.1 | §5.15 Préférences | Matrice always/selective/never, résolution conflits | 8h |
| 6.2 | §5.7 Mémoire + provenance | `/profile.md`, `/topics/`, `[stated]`/`[observed]`, versionnage | 16h |
| 6.3 | §5.5 Exécution durable | Workflows, retry, compensation, approbation humaine | 24h |
| 6.4 | §5.18 Sortie visuelle | Arbre décision, visualiseur inline, modules design | 16h |
| 6.5 | §5.19 Classification données | 5 niveaux rétention, droit à l'oubli | 8h |
| 6.6 | §5.20 Sécurité contenus | Protection injections, rappels système | 8h |
| 6.7 | §5.13 Découverte outils | tool_search, search_mcp_registry, suggest_connectors | 16h |

**Livrables :** 7 modules SFD implémentés, alignement 45% → 85%.

**Références :** `SFD.md` §5.5, §5.7, §5.13, §5.15, §5.18, §5.19, §5.20

---

## RÉFÉRENCES CROISÉES

| Document | Chemin | Contenu |
|----------|--------|---------|
| SFD v3.0 | `SFD.md` | 22 principes, 20 modules, 19 NF |
| INDEX-MAITRE | `.planning/INDEX-MAITRE.md` | Cartographie exhaustive du projet |
| PLANIFICATION | `.planning/PLANIFICATION-COMPLETE.md` | SFD vs Code, outils, tests, plan |
| Master Plan v1 | `.planning/orchestration/global-v1/MASTER-PLAN.md` | 22 agents, 5 vagues |
| WAVE-1 à 5 | `.planning/orchestration/global-v1/waves/` | Plans détaillés par vague |
| Prompts agents | `.planning/orchestration/global-v1/prompts/` | 22 prompts d'agents |
| Doc hub | `docs/README.md` | Hub central documentation |
| Architecture | `docs/architecture/` | Overview, agent-loop, services, data-flow |
| Setup | `docs/setup/` | Prerequisites, installation, configuration |
| Development | `docs/development/` | Guidelines, testing, debugging |
| Security | `docs/security/` | Threat-model, auth, sandboxing |
| STATE | `.planning/STATE.md` | État projet (historique) |

---

*GSD ROADMAP — Odysseus AgentOS v1.0*
