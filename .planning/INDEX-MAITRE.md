# 📚 INDEX-MAITRE — Odysseus AgentOS
## Référence unique et exhaustive du projet

**Date:** 2026-07-21 | **Version:** 1.0 | **Branche:** `feat/inventaire-global-v1`

---

## 🔖 NAVIGATION RAPIDE

| Section | Contenu |
|---------|---------|
| [1. Vision](#1-vision) | SFD v3.0, mission, principes |
| [2. Architecture](#2-architecture) | Couches, flux, entrée/sortie |
| [3. Code — Routes](#3-code--routes) | 55 modules, ~500 endpoints |
| [4. Code — src/](#4-code--src) | 161 modules, statut actif/dormant |
| [5. Code — Orchestrateur](#5-code--orchestrateur) | 18 modules, kill-switches |
| [6. Code — Services](#6-code--services) | 58 fichiers, 18 sous-domaines |
| [7. Infrastructure](#7-infrastructure) | Docker, configs, scripts, agents |
| [8. Frontend](#8-frontend) | 162 fichiers JS, CSS, HTML |
| [9. Tests](#9-tests) | 662 fichiers, 4566 items |
| [10. Documentation](#10-documentation) | 90+ fichiers, liens croisés |
| [11. SFD vs Code](#11-sfd-vs-code) | Écarts, statut par module |
| [12. Ajouts — global-v1](#12-ajouts--global-v1) | 22 agents, 5 vagues |
| [13. Index alphabétique](#13-index-alphabétique) | Chaque fichier → description |

---

## 1. VISION

### Mission
> "Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, apprend de chaque run, et ne dépasse jamais ses limites." — STATE.md

### Document de référence
- **SFD v3.0** (`SFD.md`, 1065 lignes) : 22 principes, 20 modules fonctionnels, 19 exigences NF
- Comparaison SFD vs Code : voir [Section 11](#11-sfd-vs-code)

### Principes invariants (10)
| # | Principe | Implémenté? | Fichier |
|---|----------|------------|---------|
| P1 | Risque modifie la boucle | 🟡 Partiel | `src/orchestrator/gate.py` |
| P2 | Draft ≠ commit | 🔴 Absent | — |
| P3 | Contexte construit (MVI) | 🟡 Partiel | `src/context_budget.py` |
| P4 | Budgets obligatoires | 🟡 Partiel | `src/budget_enforcer.py` |
| P5 | Divulgation progressive | 🔴 Absent | — |
| P6 | Échecs → règles | 🔴 Absent | — |
| P7 | Structure > autonomie | 🟡 Partiel | `config/phase-lock.yaml` |
| P8 | Plan = mêmes portes | 🔴 Absent | — |
| P9 | Évaluer harnais | 🟢 Présent | `src/observer.py` |
| P10 | Humain ON the loop | 🔴 Absent | — |

---

## 2. ARCHITECTURE

### Flux principal
```
Navigateur → FastAPI (app.py) → Auth → Routes → Chat Handler → stream_agent_loop
                                      ↓
                              LLM (llm_core) → Zen/LiteLLM → Modèle
                                      ↓
                              Tools (tool_execution) → bash/files/web/MCP
                                      ↓
                              Observer → trace_writer → JSONL traces
```

### Couches
```
┌─────────────────────────────────────────────┐
│  FRONTEND — 162 fichiers JS, vanilla SPA    │
├─────────────────────────────────────────────┤
│  ROUTES — 55 modules FastAPI, ~500 endpoints│
├─────────────────────────────────────────────┤
│  SRC/ — 161 modules métier                  │
│  ├── agent_loop.py (3637 lignes) — CŒUR    │
│  ├── llm_core.py (2308 lignes)              │
│  ├── orchestrator/ (18 modules)             │
│  ├── agent_tools/ (9 modules)               │
│  └── tools/ (11 modules)                    │
├─────────────────────────────────────────────┤
│  SERVICES/ — 58 fichiers backend            │
├─────────────────────────────────────────────┤
│  CORE/ — 9 modules (auth, DB, middleware)   │
├─────────────────────────────────────────────┤
│  MCP_SERVERS/ — 6 serveurs MCP              │
├─────────────────────────────────────────────┤
│  DOCKER — 21 services, 12 volumes           │
└─────────────────────────────────────────────┘
```

---

## 3. CODE — ROUTES

**55 modules, ~500 endpoints, 36,494 lignes**

### Routes principales

| Module | Lignes | Endpoints | Domaine |
|--------|--------|-----------|---------|
| `email_routes.py` | 3434 | 47 | Email (IMAP/SMTP, folders, compose) |
| `cookbook_routes.py` | 3313 | 16 | Modèles (download/serve/cache) |
| `model_routes.py` | 2217 | 19 | Endpoints modèles (CRUD, probe) |
| `gallery_routes.py` | 1765 | 32 | Images (upload, edit, albums) |
| `document_routes.py` | 1600 | 23 | Documents, PDF, export |
| `shell_routes.py` | 1536 | 6 | Terminal, packages |
| `chat_routes.py` | 1508 | 8 | Chat, streaming, context |
| `skills_routes.py` | 1507 | 21 | Skills (CRUD, audit, import) |
| `calendar_routes.py` | 1400 | 19 | CalDAV, events, sync |
| `session_routes.py` | 1211 | 19 | Sessions (CRUD, export, merge) |
| `task_routes.py` | 1085 | 22 | Tâches planifiées, webhooks |
| `contacts_routes.py` | 806 | 10 | CardDAV contacts |
| `codex_routes.py` | 802 | 29 | Codex + Claude plugins |
| `note_routes.py` | 851 | 10 | Notes, todos |
| `auth_routes.py` | 752 | 29 | Auth, users, 2FA, intégrations |
| `mcp_routes.py` | 619 | 11 | MCP servers |
| `research_routes.py` | 617 | 15 | Deep research |
| `history_routes.py` | 596 | 11 | Historique messages |
| `memory_routes.py` | 518 | 15 | Mémoire (CRUD, search, facts) |
| `search_routes.py` | 128 | 5 | Recherche web + fulltext |

### Routes orchestration (nouvelles)
| Module | Lignes | Endpoints | Domaine |
|--------|--------|-----------|---------|
| `killswitch_routes.py` | 11 | 1 | Dashboard kill-switches |
| `observer_routes.py` | 27 | 1 | Drift visibility |
| `governance_routes.py` | 134 | 6 | Budgets, heartbeat, goals |
| `phase_routes.py` | 45 | 3 | Phase lock |
| `autoeval_routes.py` | 60 | 4 | Auto-évaluation |
| `channel_routes.py` | 46 | 3 | Channel gateway |
| `knowledge_routes.py` | 150 | 9 | CBM/Graphify/Trinity |
| `n8n_routes.py` | 82 | 2 | Workflow automation |
| `mcp_tools_routes.py` | 54 | 2 | Outils MCP externes |

### Routes inline (app.py)
`/`, `/notes`, `/calendar`, `/cookbook`, `/email`, `/memory`, `/gallery`, `/tasks`, `/library`, `/login`, `/api/health`, `/api/version`, `/api/ready`, `/api/runtime`, `/api/agents`, `/api/agents/dispatch`

---

## 4. CODE — src/

**161 modules Python, ~65,000 lignes**

### Top 15 fichiers par taille

| Fichier | Lignes | Rôle | Statut |
|---------|--------|------|--------|
| `agent_loop.py` | 3637 | CŒUR — boucle agent streaming | 🟢 Actif |
| `llm_core.py` | 2308 | Dispatch LLM canonique | 🟢 Actif |
| `task_scheduler.py` | 2285 | Planificateur cron-like | 🟢 Actif |
| `builtin_actions.py` | 2111 | Actions automatisées | 🟢 Actif |
| `visual_report.py` | 1787 | Rapports HTML recherche | 🟢 Actif |
| `tool_schemas.py` | 1430 | Schémas OpenAI 60+ outils | 🟢 Actif |
| `ai_interaction.py` | 974 | Pont session/mémoire/RAG | 🟢 Actif |
| `tool_execution.py` | 978 | Dispatch outils → MCP/natif | 🟢 Actif |
| `deep_research.py` | 824 | Recherche profonde itérative | 🟢 Actif |
| `rag_vector.py` | 802 | RAG vectoriel (ChromaDB/Qdrant) | 🟢 Actif |
| `tool_parsing.py` | 799 | Parsing blocs outils LLM | 🟢 Actif |
| `langgraph_loop.py` | 730 | Graphe LangGraph 7 nœuds | 🔵 Gated |
| `upload_handler.py` | 736 | Uploads fichiers | 🟢 Actif |
| `admin_tools.py` | 705 | Outils admin | 🟢 Actif |
| `mcp_manager.py` | 666 | Gestionnaire MCP | 🟢 Actif |

### Orchestrateur — 18 modules

| Fichier | Lignes | Rôle | Kill-switch |
|---------|--------|------|-------------|
| `phases.py` | 54 | Enum 7 phases canoniques | — (toujours actif) |
| `spec.py` | 66 | AgentSpec dataclass | — |
| `registry.py` | 40 | Catalogue agents | — |
| `dispatcher.py` | 68 | Dispatch → multi/loop | — |
| `agent_dispatcher.py` | 209 | Dispatch par agent nommé | — |
| `multi_agent.py` | 438 | Workflow multi-agent | `ODYSSEUS_LIVE_ORCHESTRATION` |
| `loop.py` | 38 | CanonicalLoop squelette | `ODYSSEUS_LIVE_ORCHESTRATION` |
| `langgraph_loop.py` | 730 | Graphe LangGraph 7 nœuds | `ODYSSEUS_LANGGRAPH` |
| `gate.py` | 40 | Bloque commandes dangereuses | `ODYSSEUS_DESTRUCTIVE_GATE` (ON) |
| `phase_tracker.py` | 53 | Phase par round | `ODYSSEUS_PHASE_TRACKER` |
| `router_advice.py` | 52 | Conseil routage modèle | `ODYSSEUS_MODEL_ROUTER` |
| `autoeval.py` | 115 | Keep/revert post-build | `ODYSSEUS_AUTOEVAL` |
| `autoevolve.py` | 92 | Auto-amélioration agents | `ODYSSEUS_AUTOEVOLVE` |
| `codeburn_runner.py` | 110 | Détection drift code | `ODYSSEUS_CODEBURN` |
| `checkpoint_tracker.py` | 109 | Checkpoints Obsidian | `ODYSSEUS_CHECKPOINT` |
| `ancestry_tracker.py` | 60 | Ancestry mission→goal | `ODYSSEUS_GOVERNANCE_ANCESTRY` |
| `phase_resolver.py` | 32 | Résolution phase | — |

### Agent Tools — 9 modules

| Fichier | Lignes | Outils |
|---------|--------|-------|
| `filesystem_tools.py` | 459 | read/write/edit/ls/glob/grep |
| `subprocess_tools.py` | 143 | bash, python |
| `web_tools.py` | 147 | web_search, web_fetch |
| `document_tools.py` | 538 | create/update/edit/suggest document |
| `session_tools.py` | 396 | create/list/send session |
| `admin_tools.py` | 705 | manage_endpoints/mcp/webhooks/tokens |
| `model_interaction_tools.py` | 169 | chat_with_model, ask_teacher |
| `diagram_tools.py` | 127 | render_diagram (Kroki) |
| `bg_job_tools.py` | 79 | manage_bg_jobs |

### Domain Tools — 11 modules

| Fichier | Lignes | Domaine |
|---------|--------|---------|
| `cookbook.py` | 1420 | Modèles (download/serve/cache) |
| `system.py` | 637 | Skills, tasks, api_call |
| `calendar.py` | 490 | CalDAV events |
| `notes.py` | 238 | Notes/checklists |
| `vault.py` | 160 | Bitwarden vault |
| `contacts.py` | 133 | CardDAV contacts |
| `research.py` | 131 | Deep research |
| `image.py` | 35 | Édition images |
| `search.py` | 42 | Recherche conversations |

---

## 5. CODE — ORCHESTRATEUR

### État réel
```
stream_agent_loop (3637 lignes) ──→ LE SEUL ORCHESTRATEUR ACTIF
     │
     ├── PhaseTracker ──────────── GATED (ODYSSEUS_PHASE_TRACKER=off)
     ├── CanonicalLoop ─────────── GATED (ODYSSEUS_LIVE_ORCHESTRATION=off)
     ├── LangGraph ─────────────── GATED (ODYSSEUS_LANGGRAPH=off)
     ├── MultiAgentWorkflow ────── GATED (ODYSSEUS_LIVE_ORCHESTRATION=off)
     ├── AgentDispatcher ───────── ACTIF (route /api/agents/dispatch)
     ├── DestructiveGate ───────── ACTIF (ON par défaut)
     ├── Autoeval ──────────────── GATED (ODYSSEUS_AUTOEVAL=off)
     ├── Governance ────────────── GATED (ODYSSEUS_GOVERNANCE_ANCESTRY=off)
     └── 12 agents .opencode/ ──── GATED (13 kill-switches off)
```

### Kill-switches (38 au total)

| Catégorie | Switches | Défaut |
|-----------|----------|--------|
| Orchestration | 5 | Tous OFF sauf DESTRUCTIVE_GATE (ON) |
| Gouvernance/Mémoire | 7 | Tous OFF |
| RAG | 1 | OFF (RRF_FUSION) |
| Document Processing | 1 | OFF (DOCLING) |
| Qualité | 1 | OFF (DEEPEVAL) |
| Code Parsing | 1 | OFF (TREESITTER) |
| MCP/Services | 6 | Tous OFF |
| Channels | 3 | Tous OFF |
| Agents | 13 | Tous OFF |

---

## 6. CODE — SERVICES

**58 fichiers dans 18 sous-domaines**

| Service | Fichiers | Lignes | Rôle |
|---------|----------|--------|------|
| `search/` | 10 | ~2800 | Multi-provider web search + Meilisearch |
| `memory/` | 9 | ~2800 | Mémoire, extraction, skills, Mem0, Letta |
| `hwfit/` | 6 | ~2300 | Détection hardware, fit modèles |
| `research/` | 3 | ~570 | Deep research handler |
| `acontext/` | 2 | ~240 | Distillation mémoire → SKILL.md |
| `tts/` | 2 | ~250 | Text-to-speech |
| `stt/` | 2 | ~180 | Speech-to-text |
| `youtube/` | 2 | ~280 | Transcripts YouTube |
| `shell/` | 2 | ~170 | Exécution shell sécurisée |
| `docs/` | 2 | ~90 | Documents personnels RAG |
| `observability/` | 2 | ~290 | LangFuse + OpenTelemetry |
| `pipelines/` | 3 | ~490 | Prefect (RAG, maintenance, modèles) |
| `code/` | 2 | ~340 | Tree-sitter parsing |
| `documents/` | 2 | ~120 | Docling PDF processing |
| `vector/` | 2 | ~280 | Qdrant store |
| `security/` | 2 | ~180 | OPA client |
| `notifications/` | 2 | ~120 | Apprise notifications |

### MCP Servers (6)

| Serveur | Lignes | Protocole | Rôle |
|---------|--------|-----------|------|
| `email_server.py` | 2170 | SSE | Email MCP |
| `graphify_mcp.py` | 242 | SSE | Graphe sémantique |
| `memory_server.py` | 229 | SSE | Mémoire MCP |
| `obsidian_mcp.py` | 200 | stdio | Obsidian vault |
| `image_gen_server.py` | 144 | SSE | Génération images |
| `rag_server.py` | 136 | SSE | RAG MCP |

---

## 7. INFRASTRUCTURE

### Docker Compose — 21 services

| Service | Port | Profil | Rôle |
|---------|------|--------|------|
| **odysseus** | 7000 | default | App principale FastAPI |
| **chromadb** | 8100 | default | Vector DB |
| **searxng** | 8080 | default | Meta-search |
| **ntfy** | 8091 | default | Push notifications |
| **kroki** | 8700 | default | Diagrammes |
| **kroki-mermaid** | 8002 | default | Mermaid renderer |
| **serena-mcp** | 8765 | default | Code intelligence LSP |
| meilisearch | 7700 | default | Full-text search |
| codebase-memory | 9749 | knowledge | CBM code graph |
| graphify | 9750 | knowledge | Graphe sémantique |
| acontext | 8029 | acontext | Distillation mémoire |
| scrapling-mcp | 8800 | scrapling | Web scraping |
| localai | 8081 | localai | LLM+embeddings+TTS+STT |
| qdrant | 6333 | vectordb | Vector DB alternatif |
| langfuse-server | 3000 | observability | LLM tracing |
| n8n | 5678 | automation | Workflow automation |
| opa | 8181 | security | Policy engine |
| postgres | 5432 | production | PostgreSQL+pgvector |
| decision-engine | 8001 | decision-engine | Orchestrateur externe |
| traefik | 80/443 | gateway | Reverse proxy+SSL |

### Configs clés
- `config/phase-lock.yaml` — 10 phases, mapping outils
- `config/policies/*.rego` — OPA policies (tool_access, phase_lock)
- `config/localai/models/*.yaml` — Modèles LocalAI
- `model-routing.json` — Routage Zen (6 providers, 6 modèles, 8 stages)

### Agents .opencode/ (12)
`constitution`, `debate-5-personas`, `design-extract`, `edge-case-gen`, `gsd-debugger`, `gsd-executor`, `gsd-planner`, `gsd-researcher`, `gsd-roadmapper`, `gsd-verifier`, `open-design`, `security-audit`

---

## 8. FRONTEND

**162 fichiers JS, 1 CSS (1.22 MB → 16.3 KB avec Tailwind)**

### Structure
```
static/
├── index.html (2577 lignes) — SPA shell
├── login.html — Page login
├── style.css (37499 lignes, 1220 KB) — CSS legacy
├── tailwind.css — Source Tailwind v4
├── css/style.tailwind.min.css (16.3 KB) — CSS compilé
├── js/ (~60 modules) — Logique applicative
│   ├── chat.js — Streaming, messages, envoi
│   ├── sessions.js — CRUD sessions
│   ├── cockpit.js — Indicateurs phase/drift
│   ├── admin.js — Settings, killswitches
│   ├── models.js — Endpoints modèles
│   ├── cookbook.js — Download/serve modèles
│   ├── emailInbox.js — Client email complet
│   └── ...
├── lib/ (8 vendor) — htmx, alpine, highlight.js, mammoth, xlsx...
└── fonts/ (4 polices) — Inter, FiraCode, OpenDyslexic, GohuFont
```

---

## 9. TESTS

**662 fichiers, 4566 items collectés, 4393 pass (96.7%)**

### Par domaine
| Domaine | Fichiers |
|---------|----------|
| Cookbook | 21 |
| Search | 18 |
| Research | 16 |
| Session | 16 |
| HW Fit | 16 |
| Document | 15 |
| Memory | 14 |
| Calendar | 14 |
| Skills | 13 |
| Orchestrator | 13 |
| LLM Core | 17 |
| Gallery | 11 |
| Tools | 11 |
| CalDAV | 10 |
| Embeddings | 10 |
| MCP | 9 |
| Auth | 7 |
| RAG | 7 |
| Email | ~10 |
| Chat | ~12 |

### Échecs (149)
- 95% environnementaux : `rg`/`npx` absents sur Windows, symlinks, GPU AMD
- ~5% réels : auth_regressions, calendar CSS escape

---

## 10. DOCUMENTATION

**90+ fichiers documentaires, ~30,000 lignes**

### Plans et spécifications
| Répertoire | Fichiers | Contenu |
|-----------|----------|---------|
| `.planning/` | 50 | STATE, ROADMAP, intel, orchestration |
| `docs/` | 15 | Setup, services, sécurité, backup |
| `docs/audit/` | 5 | Baseline, mapping, nettoyage, couverture, master plan |
| `docs/superpowers/specs/` | 7 | Designs UI, orchestration, channel |
| `docs/superpowers/plans/` | 10 | Plans GSD d'implémentation |
| `INVENTAIRE/` | 7 | Inventaire structurel complet |
| `docs/archive/` | 3 | Docs historiques archivées |

### Documents racine
| Fichier | Lignes |
|---------|--------|
| `SFD.md` | 1065 — spécification fonctionnelle v3.0 |
| `README.md` | 93 — page d'accueil |
| `ROADMAP.md` | 80 — roadmap publique |
| `CONTRIBUTING.md` | 133 — guide contribution |
| `SECURITY.md` | 54 — politique sécurité |
| `THREAT_MODEL.md` | 81 — modèle de menace |
| `loop-canonique.md` | 43 — spécification boucle |
| `permission-matrix.md` | 52 — matrice permissions |
| `ACKNOWLEDGMENTS.md` | 173 — crédits OSS |

---

## 11. SFD vs CODE

### Alignement global : 🟡 ~45%

| Module SFD | Code | Statut |
|-----------|------|--------|
| Routage modèles | `zen_router.py`, `llm_core.py` | 🟢 |
| Exécution (agent loop) | `agent_loop.py` (3637 lignes) | 🟢 |
| Skills | `SkillsManager` | 🟢 |
| Mémoire vectorielle | `MemoryVectorStore`, `rag_vector.py` | 🟢 |
| Observabilité | `Observer`, `trace_writer` | 🟢 |
| Phase-lock | `PhaseTracker` + `phase-lock.yaml` | 🔵 Dormant |
| Boucle 7 phases | `CanonicalLoop` | 🔵 Dormant |
| Agents spécialisés | 12 `.opencode/agents/` | 🔵 Dormants |
| Auto-évaluation | `autoeval.py` | 🔵 Dormant |
| Channel Gateway | Discord/Telegram adapters | 🔵 Dormants |
| **Mémoire avec provenance** | — | 🔴 Absent |
| **Préférences structurées** | `prefs_routes.py` (basique) | 🔴 Partiel |
| **Découverte d'outils** | — | 🔴 Absent |
| **Arbre de décision sortie** | — | 🔴 Absent |
| **Exécution durable** | — | 🔴 Absent |
| **Classification/rétention** | — | 🔴 Absent |

### Écarts critiques (top 6)
1. 🔴 **Mémoire avec provenance** — tagging `[stated]`/`[observed]`/`[inferred]` absent
2. 🔴 **Système de préférences** — résolution de conflits absente
3. 🔴 **Découverte d'outils** — `tool_search`, `search_mcp_registry` absents
4. 🔴 **Arbre de décision sortie** — pas de `router_sortie()`
5. 🔴 **Exécution durable** — workflows, retry, compensation absents
6. 🔴 **Boucle canonique active** — 7 phases gated OFF

---

## 12. AJOUTS — global-v1

**Branche:** `feat/inventaire-global-v1` | **12 commits** | **160+ fichiers** | **~12,000 lignes**

### 22 agents déployés en 5 vagues

| Vague | Agents | Outils | Effort |
|-------|--------|--------|--------|
| **V1** Quick Wins | A1-A6 | Meilisearch, Apprise, Docling, Mem0, Tailwind, gVisor | ~13h |
| **V2** Observabilité | A7-A11 | LangFuse, OTel, Promptfoo, DeepEval, Ragas | ~14h |
| **V3** Refactor | A12-A16 | LangGraph, Qdrant, Traefik, OPA, HTMX+Alpine | ~37h |
| **V4** Automatisation | A17-A19 | n8n, Prefect, Letta | ~14h |
| **V5** Production | A20-A22 | PostgreSQL, LocalAI, Tree-sitter | ~11h |

### Fichiers clés créés
- `services/search/meilisearch_client.py` (281 lignes)
- `services/memory/mem0_provider.py` (399 lignes)
- `services/vector/qdrant_store.py` (308 lignes)
- `services/security/opa_client.py` (217 lignes)
- `services/observability/langfuse_tracer.py` (161 lignes)
- `services/pipelines/*.py` (3 flows Prefect)
- `services/code/treesitter_parser.py` (380 lignes)
- `src/orchestrator/langgraph_loop.py` (848 lignes)
- `config/policies/*.rego` (4 politiques OPA)
- `tailwind.config.js` + `static/tailwind.css`
- `docker-compose.yml` (+210 lignes, 9 services)
- `tests/quality/*.py` (DeepEval + Ragas)
- `tests/prompts/promptfooconfig.yaml`

---

## 13. INDEX ALPHABÉTIQUE

> Chaque fichier → description. Format: `chemin/relatif` — description (lignes, statut)

### A
- `app.py` — Point d'entrée FastAPI, monte 55 routeurs (1324 lignes, 🟢)
- `ACKNOWLEDGMENTS.md` — Crédits OSS, licences (173 lignes)

### C
- `config/phase-lock.yaml` — 10 phases, mapping outils (114 lignes, 🔵 dormant)
- `config/policies/tool_access.rego` — OPA: autorisation outils par phase (136 lignes)
- `config/policies/phase_lock.rego` — OPA: transitions de phase (161 lignes)
- `config/localai/models/` — Configs modèles LocalAI (4 fichiers)
- `CONTRIBUTING.md` — Guide contribution, style (133 lignes)
- `core/auth.py` — Auth: bcrypt, sessions, 2FA (687 lignes, 🟢)
- `core/database.py` — SQLAlchemy: 25+ tables (2476 lignes, 🟢)
- `core/middleware.py` — CSP, HSTS, CORS (126 lignes, 🟢)
- `companion/routes.py` — API companion mobile (217 lignes, 🟢)

### D
- `docker-compose.yml` — 21 services Docker (693 lignes)
- `Dockerfile` — Python 3.14-slim (97 lignes)
- `docs/setup.md` — Guide installation complet (544 lignes)
- `docs/services.md` — Référence services Docker (69 lignes)
- `docs/security.md` — 4 couches sandbox (54 lignes)
- `docs/testing.md` — Promptfoo setup (133 lignes)
- `docs/observability.md` — LangFuse setup (146 lignes)
- `docs/n8n-workflows.md` — 3 templates n8n (305 lignes)

### I
- `INVENTAIRE/INVENTAIRE-MAITRE.md` — Synthèse exécutive (110 lignes)
- `INVENTAIRE/A-INVENTAIRE-FONCTIONNEL.md` — 12 couches fonctionnelles (187 lignes)
- `INVENTAIRE/B-ORCHESTRATION.md` — Graphe orchestration (209 lignes)
- `INVENTAIRE/C-COUVERTURE-UI.md` — Matrice UI 45 capacités (152 lignes)

### L
- `loop-canonique.md` — Spécification 7 phases (43 lignes)

### M
- `model-routing.json` — Routage Zen: 6 providers, 6 modèles (143 lignes)
- `mcp_servers/email_server.py` — Email MCP (2170 lignes, 🟢)
- `mcp_servers/obsidian_mcp.py` — Obsidian vault (200 lignes, 🔵 gated)

### P
- `.planning/STATE.md` — État projet, M3 status (180 lignes)
- `.planning/ROADMAP-15-PHASES.md` — 15 phases, 76 exigences (357 lignes)
- `.planning/ROADMAP-M3-ORCHESTRATION.md` — M3 orchestration (206 lignes)
- `.planning/AGENT-OS-ANALYSE-PROFONDE.md` — Analyse 60 outils (722 lignes)
- `.planning/intel/INDEX.md` — Table routage L3 (116 lignes)
- `.planning/intel/domains/*.md` — 7 fichiers domaine (~250 lignes chacun)
- `.planning/orchestration/global-v1/MASTER-PLAN.md` — Plan orchestration v1 (90 lignes)
- `permission-matrix.md` — 5 niveaux risque, 4 gates (52 lignes)
- `PROJECT.yaml.example` — Template manifest projet

### R
- `README.md` — Page d'accueil (93 lignes)
- `ROADMAP.md` — Roadmap publique (80 lignes)
- `routes/email_routes.py` — 47 endpoints email (3434 lignes, 🟢)
- `routes/chat_routes.py` — 8 endpoints chat (1508 lignes, 🟢)
- `routes/cookbook_routes.py` — 16 endpoints modèles (3313 lignes, 🟢)

### S
- `SECURITY.md` — Politique sécurité (54 lignes)
- `SFD.md` — Spécification fonctionnelle v3.0 (1065 lignes)
- `src/agent_loop.py` — CŒUR: boucle agent (3637 lignes, 🟢)
- `src/llm_core.py` — Dispatch LLM (2308 lignes, 🟢)
- `src/task_scheduler.py` — Cron scheduler (2285 lignes, 🟢)
- `src/orchestrator/langgraph_loop.py` — Graphe 7 nœuds (848 lignes, 🔵 gated)
- `src/orchestrator/multi_agent.py` — Workflow multi-agent (438 lignes, 🔵 gated)
- `src/orchestrator/gate.py` — Bloque commandes dangereuses (40 lignes, 🟢 ON)
- `src/observer.py` — Drift monitoring (177 lignes, 🟢)
- `src/trace_writer.py` — Traces JSONL (166 lignes, 🟢)
- `src/memory_provider.py` — Registry providers mémoire (390 lignes, 🟢)
- `src/rag_vector.py` — RAG vectoriel (802 lignes, 🟢)
- `src/tool_execution.py` — Dispatch outils (978 lignes, 🟢)
- `src/killswitch_registry.py` — Registre 38 switches (142 lignes, 🟢)
- `services/search/meilisearch_client.py` — Full-text search (281 lignes, 🟢)
- `services/memory/mem0_provider.py` — Mem0 mémoire (399 lignes, 🟢)
- `services/vector/qdrant_store.py` — Qdrant store (308 lignes, 🔵 gated)
- `services/observability/langfuse_tracer.py` — LLM tracing (161 lignes, 🔵 gated)
- `services/code/treesitter_parser.py` — Parsing syntaxique (380 lignes, 🔵 gated)
- `services/pipelines/rag_pipeline.py` — Prefect RAG (168 lignes, 🔵 gated)
- `services/security/opa_client.py` — Client OPA (217 lignes, 🔵 gated)

### T
- `THREAT_MODEL.md` — Modèle de menace (81 lignes)
- `tests/` — 662 fichiers, 4566 items

---

*FIN DU DOCUMENT — Version 1.0 — 2026-07-21*
