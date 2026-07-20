# INVENTAIRE COMPLET — Odysseus AgentOS

**Date:** 2026-07-21 | **Méthode:** 8 agents parallèles + CBM (11,749 nœuds) + Pytest

---

## 1. CHIFFRES GLOBAUX

| Métrique | Valeur |
|----------|--------|
| Fichiers Python | 989 (7.51 MB, 159,267 lignes) |
| Fichiers JS frontend | 154 (5.69 MB) |
| Fichier CSS | 1 (1.22 MB, 37,499 lignes) |
| Routes API | 62 modules, ~200+ endpoints |
| Modules src/ | 160 fichiers |
| Services Docker | 7 actifs + 4 profils |
| Kill-switches | 35+ (1 seul actif: DESTRUCTIVE_GATE) |
| Agents OpenCode | 12 (tous gated OFF) |
| Tests | 4,393 OK / 149 FAIL (96.7%) |
| CBM Graph | 11,749 nœuds, 38,126 arêtes |

## 2. ARCHITECTURE

```
app.py (1320 lignes) → FastAPI monolithique
├── 62 modules routes/ (36,494 lignes)
├── 160 modules src/ (~65,000 lignes)
│   ├── agent_loop.py (3,568 lignes) ← SPOF #1
│   ├── llm_core.py (2,305 lignes)
│   ├── task_scheduler.py (2,285 lignes)
│   ├── orchestrator/ (16 modules, 95% dormant)
│   ├── agent_tools/ (10 modules)
│   └── tools/ (11 modules)
├── 10 modules core/ (auth, DB, middleware)
├── 39 modules services/ (search, memory, TTS, STT)
├── 7 MCP servers internes
└── 6 MCP servers Docker externes
```

## 3. GAPS CRITIQUES

| # | Gap | Sévérité |
|---|-----|----------|
| G1 | stream_agent_loop = 3,568 lignes, 1 fonction monolithique | CRITIQUE |
| G2 | 3 orchestrateurs gated indépendamment, zéro validation cohérence | CRITIQUE |
| G3 | Phase KNOW manquante (pas de checkpoint pré-génération) | MAJEUR |
| G4 | Phase APPROVE manquante (bloque sans approuver) | MAJEUR |
| G5 | CSS = 1.22 MB (cible <200KB) | MAJEUR |
| G6 | 35+ kill-switches sans validation cohérence globale | MAJEUR |
| G7 | CodeBurn configuré mais non intégré au runtime | MOYEN |
| G8 | Pas de full-text search (SQLite LIKE uniquement) | MOYEN |
| G9 | Prompt/context bloat pour petits modèles (4k/8k/16k) | MOYEN |
| G10 | Pas de LLM tracing structuré (trace_writer JSONL) | MOYEN |

## 4. ÉTAT DE L'ORCHESTRATION

| Composant | Code | Kill-switch | Actif ? |
|-----------|------|-------------|---------|
| PhaseTracker | phase_tracker.py | ODYSSEUS_PHASE_TRACKER | OFF |
| CanonicalLoop | loop.py | ODYSSEUS_LIVE_ORCHESTRATION | OFF |
| AgentDispatcher | agent_dispatcher.py | 10 switches | OFF |
| router_advice | router_advice.py | ODYSSEUS_MODEL_ROUTER | OFF |
| autoeval | autoeval.py | ODYSSEUS_AUTOEVAL | OFF |
| governance | governance.py | ODYSSEUS_GOVERNANCE_ANCESTRY | OFF |
| checkpoint | checkpoint_tracker.py | ODYSSEUS_CHECKPOINT | OFF |
| codeburn | codeburn_runner.py | ODYSSEUS_CODEBURN | OFF |
| autoevolve | autoevolve.py | ODYSSEUS_AUTOEVOLVE | OFF |
| destructive_gate | gate.py | ODYSSEUS_DESTRUCTIVE_GATE | **ON** |
| RRF fusion | rag_vector.py | ODYSSEUS_RRF_FUSION | OFF |
| Observer | observer.py | (toujours actif) | ON |

## 5. STACK TECHNIQUE

| Catégorie | Technologie |
|-----------|-------------|
| Backend | Python 3.14, FastAPI, Uvicorn |
| ORM | SQLAlchemy + SQLite |
| Vector DB | ChromaDB (SPOF) |
| Embeddings | fastembed (ONNX local) |
| Auth | bcrypt + pyotp TOTP + session cookies |
| LLM Routing | LiteLLM + zen_router |
| Search | SearXNG + rank-bm25 |
| Notifications | ntfy + Discord/Telegram adapters |
| Frontend | Vanilla JS SPA (154 fichiers) |
| CSS | Monolithique 37,499 lignes |
| Tests | pytest + pytest-asyncio |
| CI/CD | GitHub Actions (9 workflows) |
| Déploiement | Docker Compose (7+ services) |

## 6. TOP 10 FICHIERS LES PLUS GROS

| Fichier | Lignes |
|---------|--------|
| static/style.css | 37,499 |
| src/agent_loop.py | 3,568 |
| routes/email_routes.py | 3,434 |
| routes/cookbook_routes.py | 3,313 |
| core/database.py | 2,476 |
| src/llm_core.py | 2,305 |
| src/task_scheduler.py | 2,285 |
| routes/model_routes.py | 2,217 |
| src/builtin_actions.py | 2,111 |
| src/visual_report.py | 1,787 |

## 7. DETTE TECHNIQUE

| Item | Status |
|------|--------|
| CSS >1MB (cible <200KB) | Partiel (870KB après purgecss) |
| pas de full-text search | Non résolu |
| pas de LLM tracing | Non résolu |
| pas de prompt testing | Non résolu |
| pas de kernel sandbox | Non résolu |
| tests core/auth 0% coverage | Non résolu |
| tests core/database 0% coverage | Non résolu |
| AgentSeal CI non déployé | Non résolu |
| accessibilité UI (keyboard nav, contrast) | Non résolu |
| dead code pass | Partiel |
