# CODEBASE-MAP — Agent OS / Odysseus V1

> Régénéré le 2026-07-30 après migration V1 (cleanup packages vides,
> mode agent réparé, engine Docker réel). Remplace l'ancienne map
> obsolète qui annonçait 94 fichiers / 223 tests.

## Chiffres réels

| Métrique | Valeur | Source |
|---|---|---|
| Fichiers `.py` (src + routes + core) | **247** | `rg --files src/ routes/ core/` |
| Lignes Python (src + routes + core) | **101 093** | computed 2026-07-30 |
| Tests (`tests/test_*.py`) | **694** | `rg --files 'tests/test_*.py'` |
| Agents `.opencode/agents/` | **17** | markdown prompts, dispatchés par phase SFD |
| Skills `.opencode/skills/` | **3** | sfd-pipeline, sfd-decision-tree, auto-evolve |
| Custom tools `.opencode/tools/` | **1** | `sfd-phase.ts` (pipeline phase manager) |
| Packages npm `@agentos/sfd-*` | **1** | `sfd-eventbus` (seul avec vraie impl TS) |
| Plugins déclarés dans `opencode.json` | **1** | `@agentos/sfd-eventbus` |
| Dockerfiles | **7** | agentos-engine, agentos-sandbox, principal, gpu, build scripts |

## Structure racine

```
odysseusAgentOS/
├── app.py                    # FastAPI slim orchestrator (1324 lignes)
├── opencode.json             # config système: 5 agents, 1 plugin, permissions
├── docker-compose.yml        # 11 services (chromadb, kroki, meilisearch, ...)
├── Dockerfile                # image principale Odysseus (python:3.14 + opencode)
├── src/                      # 173 .py — c})ur agent (agent_loop, llm_core,
│   │                           tool_*, orchestrator/, memory, etc.)
│   ├── opencode_engine.py    # ★ LE pipeline 7 phases (Walk SFD)
│   ├── event_bus.py          # bus lourd scheduler ScheduledTask
│   ├── opencode_bridge.py    # routing engine HTTP / legacy ZenRouter
│   ├── provenance_memory.py  # tags [stated]/[observed]/[inferred]
│   ├── memory_writer.py      # hooks MEMORY_OBSERVE -> vault
│   └── orchestrator/         # 18 .py (langgraph, dispatcher, phase tracker...)
├── routes/                   # 63 .py — endpoints FastAPI
├── core/                     # 11 .py — database, auth, models, middleware
├── services/                 # 109 fichiers — search, memory, hwfit...
├── tests/                    # 694 tests pytest avec taxonomy
├── packages/sfd-eventbus/    # SEUL package npm avec impl TS réelle
├── docker/agentos-engine/    # V1 isolation engine (Python 3.14 + Opencode CLI)
│   ├── Dockerfile            # image agentos-engine:v1
│   └── engine_server.py      # ★ FastAPI app: /health, /run (SSE), /status
├── .opencode/
│   ├── agents/               # 17 agents markdown (constitution, explore, ...)
│   ├── skills/               # 3 skills (sfd-pipeline, sfd-decision-tree, ...)
│   └── tools/sfd-phase.ts    # tool custom (sfd-phase) chargé par opencode
├── docs/master-ref/         # 13 .md (01-SFD → 12-CERTIFICATION) + Guide dev 14 chapitres
└── docker-compose.yml       # 11 services (chromadb, kroki, meilisearch, ...)
```

## Pipeline SFD — Load path

```
utilisateur -> app.py -> routes/chat_routes.py
                                  │
                                  ├─ mode CHAT -> stream_agent_loop (ZenRouter)
                                  │
                                  └─ mode AGENT -> src/opencode_engine.py
                                                     │
                                                     └─ OpenCodeEngine.walk()
                                                         ├─ EventBus (15 familles, 62 types, trace_id)
                                                         ├─ PHASE_AGENTS / PHASE_TOOLS / PHASE_MODELS
                                                         ├─ 7 phases: CLASSIFY→KNOW→PLAN→BUILD→...
                                                         ├─ emit SSE events (phase_enter, agent_dispatch, ...)
                                                         └─ goal-ancestry DB + memory_writer [observed]
```

## Isolation V1 (Docker)

```
HOST (app.py :7000)
   │ POST /api/chat_stream (mode agent)
   ▼
src/opencode_bridge.py — routing conditionnel:
   - if AGENTOS_ENGINE_HOST set -> HTTP POST :7001/run (SSE forward)
   - else -> internal agent_stream fallback
   ▼
docker/agentos-engine (python:3.14-slim) :
   ├─ engine_server.py (FastAPI :7001)
   │    ├─ OpenCodeEngine.walk() (vrai pipeline, identique à l'host)
   │    ├─ _write_trace() -> /home/agentos/data/traces/events-YYYY-MM-DD.jsonl
   │    └─ _write_observed_fact() -> /home/agentos/obsidian-vault/topics/*.md
   ├─ volumes: data, workspace, vault (persistants)
   ├─ security: no-new-privileges, cap_drop ALL
   └─ Docker healthcheck sur /health
```

## V1 — Ce qui marche (prouvé par tests)

- ✅ Pipeline 7 phases en ordre CLASSIFY→KNOW→PLAN→BUILD→QUALITY→AUTOEVAL→MEMORY_OBSERVE
- ✅ Mode détection (CHAT vs AGENT) — "build a flask app" → agent, "bonjour" → chat
- ✅ EventBus avec `trace_id` (corrélation cross-familles, audit UC-12)
- ✅ Traces JSONL persistées
- ✅ Memory `[observed]` avec frontmatter Obsidian (provenance P16)
- ✅ 5 tests E2E permanents `tests/test_engine_bridge_e2e.py`
- ✅ Build Docker `agentos-engine:v1` (277MB, healthcheck OK)
- ✅ Endpoints `/health`, `/run` (SSE), `/status` — `engine: real`
- ✅ 17 agents dispatchés par phase, 7 skills chargés

## V1 — Gaps connus (pour V2)

| Gap | SFD ref | Effort estimé |
|---|---|---|
| 9 packages SFD vides (memory, durable, prefs...) retirés de V1 | §5.7, §5.5 | High — à réimplémenter en V2 si distrib npm requis |
| `engine_server.py` ne dispatche pas de vrais agents OpenCode dans Docker | §5.1 | Medium — brancher OpenCode CLI subprocess dans le conteneur |
| Drift indicator cockpit partiel | Axe 1 | Medium |
| Phase-lock strict par round | §5.4.2 | Medium |
| 10 use cases code-only (UC-02,04,05,06,07,09,11,12,13,15) | §4 | High — tests E2E à écrire |
| gVisor `runsc` non installé hôte | Axe 5 | Low |
| VPS deployment jamais testé | §5.5 | Medium |
| Acontext OFF (auto-skills) | §5.7.7 | Medium |
| Dashboard Trinité unifié | Axe 4 | Medium |

## Voir aussi

- `docs/master-ref/01-SFD-v3.0.md` — 22 principes, 20 modules, 19 UC
- `docs/master-ref/03-OBJECTIFS-SYSTEME.md` — architecture cible
- `docs/master-ref/05-EVENT-BUS.md` — contratt event bus
- `docs/master-ref/11-VERIFICATION-COMPLETE.md` — audit 90% + tests E2E historiques
- `docs/architecture/ODYSSEUS-ARCHITECTURE.md` — vision frontend/backend
- `tests/test_engine_bridge_e2e.py` — tests V1 régulation