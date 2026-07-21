# Architecture — Vue d'ensemble

## Flux principal

```
Navigateur → FastAPI (app.py) → Auth → Routes → Chat Handler → stream_agent_loop
                                      ↓
                              LLM (llm_core) → Zen/LiteLLM → Modèle
                                      ↓
                              Tools (tool_execution) → bash/files/web/MCP
                                      ↓
                              Observer → trace_writer → JSONL traces
```

## Couches

```mermaid
graph TD
    FRONTEND["FRONTEND — 162 fichiers JS, vanilla SPA"]
    ROUTES["ROUTES — 55 modules FastAPI, ~500 endpoints"]
    SRC["SRC/ — 161 modules métier"]
    AGENT["agent_loop.py (3637 lignes) — CŒUR"]
    LLM["llm_core.py (2308 lignes)"]
    ORCH["orchestrator/ (18 modules)"]
    TOOLS["agent_tools/ (9) + tools/ (11)"]
    SERVICES["SERVICES/ — 58 fichiers backend"]
    CORE["CORE/ — 9 modules (auth, DB, middleware)"]
    MCP["MCP_SERVERS/ — 6 serveurs MCP"]
    DOCKER["DOCKER — 21 services, 12 volumes"]

    FRONTEND --> ROUTES --> SRC
    SRC --> AGENT
    SRC --> LLM
    SRC --> ORCH
    SRC --> TOOLS
    SRC --> SERVICES
    ROUTES --> CORE
    SERVICES --> MCP
    DOCKER --> SERVICES
```

## Stack technique

| Couche | Technologie |
|---------|------------|
| Backend | Python 3.14, FastAPI, Uvicorn |
| LLM Routing | LiteLLM + Zen Router (OpenCode Go) |
| Frontend | Vanilla JS SPA (162 fichiers), Tailwind CSS v4 (16.3 KB) |
| Database | SQLite (SQLAlchemy), ChromaDB (vecteur), Qdrant (optionnel) |
| Auth | bcrypt, pyotp TOTP, session cookies |
| MCP | 6 serveurs built-in + 5 Docker externes |
| Déploiement | Docker Compose (21 services, 12 profils) |
| CI/CD | GitHub Actions (9 workflows) |
| Tests | pytest (662 fichiers, 4393 pass) |

## Chiffres clés

| Métrique | Valeur |
|----------|--------|
| Fichiers Python | 989 (~160K lignes) |
| Fichiers JS | 162 |
| Routes API | 55 modules, ~500 endpoints |
| Kill-switches | 38 (1 actif par défaut) |
| Agents spécialisés | 12 (.opencode/) |
| Tests | 4393 pass / 149 fail (96.7%) |

## État de l'orchestration

```mermaid
graph LR
    LIVE["stream_agent_loop\n(3637 lignes)\n🟢 ACTIF"]
    PHASE["PhaseTracker\n🔵 GATED"]
    CANON["CanonicalLoop\n🔵 GATED"]
    LANG["LangGraph (7 nœuds)\n🔵 GATED"]
    MULTI["MultiAgentWorkflow\n🔵 GATED"]
    DISP["AgentDispatcher\n🟢 ACTIF"]

    LIVE --> PHASE
    LIVE --> CANON
    LIVE --> LANG
    LIVE --> MULTI
    LIVE --> DISP
```

> **Réalité :** `stream_agent_loop` est le seul orchestrateur actif. Les 4 autres sont codés, testés, mais gated OFF derrière des kill-switches. Le design "câblé mais dormant" permet l'activation progressive.

---

→ Voir aussi : [Boucle agent](agent-loop.md) · [Services](services.md) · [Flux de données](data-flow.md) · [INDEX-MAITRE](../../.planning/INDEX-MAITRE.md)
