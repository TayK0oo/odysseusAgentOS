# B — ORCHESTRATION (Axe B)

**Date** : 2026-07-07 | **Commit** : `2821daa`

---

## B.1 — GRAPHE DE DÉPENDANCES (STATIQUE)

### Edge-list : arêtes prouvées avec mécanisme

| Source | → | Cible | Mécanisme | Direction | Statut lien | Preuve |
|---|---|---|---|---|---|---|
| **BOUCLE LIVE** | | | | | | |
| `agent_loop.stream_agent_loop` | → | `PhaseTracker` | import + appel `on_round_start()` | sync | 🟢 (gated) | agent_loop.py:2485-2538 |
| `agent_loop.stream_agent_loop` | → | `CanonicalLoop` | import + appel `advance()` | sync | 🔵 (gated OFF) | agent_loop.py:2488-2497 |
| `agent_loop.stream_agent_loop` | → | `router_advice.advise` | import + appel | sync | 🔵 (gated OFF) | agent_loop.py:2504-2513 |
| `agent_loop.stream_agent_loop` | → | `AgentDispatcher.dispatch_for_phase` | import + appel async | async | 🔵 (gated OFF) | agent_loop.py:2518-2570 |
| `agent_loop.stream_agent_loop` | → | `llm_core.stream_llm_with_fallback` | import + appel | async (stream) | 🟢 | agent_loop.py (CBM trace outbound) |
| `agent_loop.stream_agent_loop` | → | `tool_execution` (format/execute) | import + appel | sync | 🟢 | agent_loop.py (CBM trace) |
| `agent_loop.stream_agent_loop` | → | `skills_manager.get_relevant_skills` | import + appel | sync | 🟢 | agent_loop.py (CBM trace) |
| `agent_loop.stream_agent_loop` | → | `memory_vector.MemoryVectorStore.search` | import + appel | sync | 🟢 | agent_loop.py (CBM trace) |
| **POST-ROUND** | | | | | | |
| `agent_loop` | → | `MemoryProviderRegistry.dispatch_session_end` | import + appel await | async | 🟢 | agent_loop.py:3586-3601 |
| `agent_loop` | → | `record_run_ancestry` | import + appel | sync | 🔵 (gated OFF) | agent_loop.py:3604-3617 |
| `agent_loop` | → | `record_checkpoint` | import + appel | sync | 🔵 (gated OFF) | agent_loop.py:3625-3633 |
| `agent_loop` | → | `run_codeburn` | import + appel await | async | 🔵 (gated OFF) | agent_loop.py:3640-3645 |
| `agent_loop` | → | `Observer.ingest_metrics` | import + appel | sync | 🟢 | agent_loop.py:3649-3653 |
| `agent_loop` | → | `Observer.compute_drift_score` | import + appel | sync | 🟢 | agent_loop.py:3654 |
| `agent_loop` | → | `apply_autoeval` | import + appel | sync | 🔵 (gated OFF) | agent_loop.py:3667-3693 |
| `agent_loop` | → | `maybe_autoevolve` | import + appel await | async | 🔵 (gated OFF) | agent_loop.py:3698-3712 |
| **GATES** | | | | | | |
| `tool_execution` | → | `gate.should_block_destructive` | import + appel | sync | 🟢 | tool_execution.py:563-565 |
| `tool_execution` | → | `gate.gate_enabled` | import + appel | sync | 🟢 | tool_execution.py:564 |
| `gate.should_block_destructive` | → | `risk_classifier.classify_bash` | import + appel | sync | 🟢 | gate.py:18,49 |
| **CHANNELS** | | | | | | |
| `app._startup_channel_adapters` | → | `channel_bootstrap.bootstrap_channels` | import + appel await | async | 🔵 (gated OFF) | app.py:1243-1252 |
| `channel_bootstrap` | → | `ChannelGateway` | import + appel (get_gateway) | sync | 🟢 | channel_bootstrap.py:24-30 |
| `channel_bootstrap` | → | `DiscordAdapter` | import conditionnel | lazy | 🔵 (gated) | channel_bootstrap.py:185 |
| `channel_bootstrap` | → | `TelegramAdapter` | import conditionnel | lazy | 🔵 (gated) | channel_bootstrap.py:192 |
| `channel_bootstrap` | → | `event_bus.fire_event` | import + appel | async | 🟢 (via handler) | channel_bootstrap.py:144-150 |
| `channel_bootstrap.run_agent_reply` | → | `task_endpoint.task_llm_call_async` | lazy import + appel await | async | 🔵 (gated) | channel_bootstrap.py:89-97 |
| **MCP** | | | | | | |
| `app._startup_mcp_connections` | → | `builtin_mcp.register_builtin_servers` | import + appel await | async | 🟢 | app.py:1030-1035 |
| `app._startup_mcp_connections` | → | `mcp_manager.connect_all_enabled` | appel await | async | 🟢 | app.py:1037 |
| `app._startup_mcp_connections` | → | `mcp_manager.connect_external_enabled` | appel await | async | 🟢 | app.py:1046 |
| **AGENTS** | | | | | | |
| `app._startup_agent_catalog` | → | `AgentRegistry.discover` | import + appel | sync | 🔵 (gated OFF) | app.py:1254-1266 |
| `AgentDispatcher` | → | `AgentRegistry` (lazy via app.state) | import conditionnel | lazy | 🔵 (gated) | agent_dispatcher.py:96-112 |
| `AgentDispatcher._run_agent` | → | `task_endpoint.task_llm_call_async` | import + appel await | async | 🔵 (gated) | agent_dispatcher.py:198,217 |
| `AgentRegistry.discover` | → | `parse_agent_spec` (spec.py) | import + appel | sync | 🟢 | registry.py:37 |
| **TRINITÉ** | | | | | | |
| `knowledge_routes` | → | `VectorRAG` (`get_rag_manager()`) | import + appel | sync | 🟢 | knowledge_routes.py (délégation recherche sémantique) |
| `checkpoint_tracker.record_checkpoint` | → | `obsidian_mcp.create_note` | appel fonction | sync | 🔵 (gated OFF) | checkpoint_tracker.py:119 |

---

## B.2 — ORCHESTRATION DYNAMIQUE : TRACE DE LA LOOP RÉELLE

### Chemin d'une requête chat de bout en bout

```
[1] UI (static/index.html) ──POST /api/chat/stream──▶ app.py ──▶ chat_routes
[2] chat_routes ──▶ stream_agent_loop(session_id, messages, model, ...)
```

#### Étape par étape dans `stream_agent_loop` :

| # | Étape | Brique | Kill-switch | Statut réel | Preuve |
|---|---|---|---|---|---|
| 0 | **ENTRÉE** | client HTTP/UI → `POST /api/chat/stream` | - | 🟢 | routes/chat_routes.py |
| 1 | **CLASSIFY (léger)** | `_classify_agent_request()` : admin, guide_only, plan_mode, teacher | - | 🟢 | agent_loop.py:860-943 |
| 2 | **Phase tracking** | `PhaseTracker.on_round_start()` → `infer_phase()` → BUILD (ou PLAN si plan_mode) | `ODYSSEUS_PHASE_TRACKER`=off | 🔵 ignoré si OFF | agent_loop.py:2485-2538 |
| 3 | **Router advice** | `router_advice.advise()` → classify_intent → rôle natif (log-only) | `ODYSSEUS_MODEL_ROUTER`=off | 🔵 ignoré si OFF | agent_loop.py:2504-2513 |
| 4 | **Agent dispatch** | `AgentDispatcher.dispatch_for_phase()` → agents .opencode/ par phase | Tous kill-switches agents=off | 🔵 ignoré si OFF | agent_loop.py:2518-2570 |
| 5 | **Budget itérations** | `budget_enforcer.consume_iteration()` → hard-stop si max_rounds | - | 🟢 | agent_loop.py:2572-2576 |
| 6 | **BUILD (LLM)** | `stream_llm_with_fallback()` → LiteLLM → modèle → SSE stream | - | 🟢 | llm_core.py |
| 7 | **Tool execution** | `tool_execution` → parse + execute + destructive gate | `ODYSSEUS_DESTRUCTIVE_GATE`=on | 🟢 gate actif | tool_execution.py:563-571 |
| 8 | **Memory/Skills** | `MemoryVectorStore.search()` + `SkillsManager.get_relevant_skills()` | - | 🟢 | agent_loop.py (CBM trace) |
| 9 | **POST-ROUND** | | | | |
| 9a | **Acontext** | `MemoryProviderRegistry.dispatch_session_end()` | `ACONTEXT_ENABLED`=off | 🔵 no-op si OFF | agent_loop.py:3586-3601 |
| 9b | **Ancestry** | `record_run_ancestry()` → GoalTask DB | `ODYSSEUS_GOVERNANCE_ANCESTRY`=off | 🔵 ignoré si OFF | agent_loop.py:3604-3617 |
| 9c | **Checkpoint** | `record_checkpoint()` → Obsidian create_note | `ODYSSEUS_CHECKPOINT`=off | 🔵 ignoré si OFF | agent_loop.py:3625-3633 |
| 9d | **CodeBurn** | `run_codeburn()` → rapport → feed_observer | `ODYSSEUS_CODEBURN`=off | 🔵 ignoré si OFF | agent_loop.py:3640-3645 |
| 9e | **Observer** | `Observer.ingest_metrics()` + `compute_drift_score()` | - | 🟢 toujours actif | agent_loop.py:3649-3658 |
| 9f | **Autoeval** | `apply_autoeval()` → keep/revert (git reset --hard) | `ODYSSEUS_AUTOEVAL`=off | 🔵 ignoré si OFF | agent_loop.py:3667-3693 |
| 9g | **Autoevolve** | `maybe_autoevolve()` → recherche amélioration | `ODYSSEUS_AUTOEVOLVE`=off | 🔵 ignoré si OFF | agent_loop.py:3698-3712 |
| 10 | **Teacher** | Escalade enseignant (si échec) | - | 🟢 | agent_loop.py:3714+ |

---

## B.3 — LOOP RÉELLE vs LOOP CANONIQUE VISÉE

### Tableau de confrontation

| Phase canonique visée | Équivalent réel | Statut | Notes |
|---|---|---|---|
| **CLASSIFY** | `_classify_agent_request` (admin, guide_only, plan_mode) | 🟡 PARTIEL | Classification admin/plan_mode existe, pas la phase formelle avec forced_tools risk_classifier |
| **KNOW** (checkpoint) | AUCUN checkpoint pré-génération | 🔴 MANQUANT | La loop ne consulte ni CBM, ni Obsidian, ni mémoire avant de générer |
| **PLAN** (MVI) | `plan_mode` + `PhaseTracker.infer_phase` → PLAN | 🟡 PARTIEL | plan_mode existe mais pas de MVI formel (Minimum Viable Input) |
| **DEBATE** (si complexe) | `AgentDispatcher` → agent `debate-5-personas` (gated OFF) | 🔵 DORMANT | Seulement si `ODYSSEUS_AGENT_DEBATE=on` |
| **APPROVE** | AUCUN mécanisme d'approbation humain formel | 🔴 MANQUANT | Seul le destructive gate BLOQUE (ne fait pas approuver). Pas de workflow approbation. |
| **BUILD** | `stream_llm_with_fallback` + tool execution | 🟢 | Pleinement fonctionnel |
| **QUALITY** | `_run_verifier_subagent` (capé, OFF par défaut) | 🔵 DORMANT | Existe mais `agent_verifier_subagent`=off par défaut |
| **AUTOEVAL** | `apply_autoeval` (gated OFF) | 🔵 DORMANT | `ODYSSEUS_AUTOEVAL`=off → forces "keep" |
| **MEMORY_OBSERVE** | ancestry + checkpoint + codeburn + observer + autoevolve | 🟡 PARTIEL | Observer toujours actif ; le reste = gated OFF |

### Diagramme des orchestrateurs

```
                ┌──────────────────────────────┐
                │      stream_agent_loop        │
                │   (orchestrateur de fait)     │
                └──────────┬───────────────────┘
                           │
          ┌────────────────┼────────────────────┐
          ▼                ▼                     ▼
   ┌─────────────┐  ┌─────────────┐     ┌──────────────┐
   │PhaseTracker │  │CanonicalLoop│     │AgentDispatcher│
   │ (pont live) │  │  (7 phases) │     │(agents .opencode)│
   │ gated OFF   │  │ gated OFF   │     │  gated OFF     │
   └──────┬──────┘  └──────┬──────┘     └──────┬───────┘
          │                │                    │
          ▼                ▼                    ▼
   ┌──────────────────────────────────────────────────┐
   │              ToolRegistry.set_phase()             │
   │         → phase-lock.yaml → bloque/allow tools    │
   └──────────────────────────────────────────────────┘
```

**Conclusion** : Il y a **TROIS orchestrateurs** potentiels (PhaseTracker, CanonicalLoop, AgentDispatcher), tous gated OFF par défaut. L'orchestrateur de fait est `stream_agent_loop` lui-même, qui est une boucle linéaire avec des hooks post-round. Aucun de ces trois ne prend le contrôle de la séquence d'exécution dans la configuration par défaut.

---

## B.4 — POINTS DE COUPLAGE FORTS & SPOF

| Point | Type | Risque | Preuve |
|---|---|---|---|
| `stream_agent_loop` | SPOF central | Si cette fonction crash, tout le chat s'arrête | agent_loop.py:1934 (3500 lignes, une seule fonction) |
| `llm_core.stream_llm_with_fallback` | SPOF | Si aucun LLM dispo, plus de réponse | llm_core.py (une seule chaîne de fallback) |
| `core.database` (SQLite) | SPOF | Toute l'app dépend d'un seul fichier `data/app.db` | docker-compose.yml:45 |
| `chromadb` | SPOF (RAG) | Si down, RAG + personal docs = 503 | docker-compose.yml:160-178 |
| `SearXNG` | SPOF (recherche) | Si down, web search = 502 | docker-compose.yml:180-230 |
| `app.state` (singleton global) | Couplage fort | auth_manager, session_manager, upload_handler, mcp_manager… tous attachés à `app.state` | app.py:206,536-548,747 |
| Fichiers partagés | Couplage caché | `data/settings.json`, `data/auth.json`, `data/presets.json` — lus/écrits par multiples routes sans locking distribué | app.py, settings.py, auth.py |
| Kill-switches ×30+ | Complexité de configuration | Chaque kill-switch est une variable d'env indépendante ; pas de validation de cohérence globale | docker-compose.yml:86-133 |

---

## B.5 — CYCLES & CHEMINS MORTS

| Pattern | Détail | Preuve |
|---|---|---|
| **Cycle SSE** | `stream_agent_loop` yield vers SSE → frontend → POST suivant → `stream_agent_loop` | Boucle de chat normale (chaque tour = nouvel appel) |
| **Cycle mémoire** | agent_loop → MemoryProviderRegistry → Acontext (si ON) → distillation → SKILL.md → skills_manager → agent_loop (tour suivant) | agent_loop.py:3586,610 |
| **Chemin mort : knowledge_routes** | `/api/knowledge/*` enregistré dans app.py:823 mais 0 appelant | STATE.md:88 |
| **Chemin mort : /api/agents (sans catalogue)** | Retourne `{agents: [], hint: "Set ODYSSEUS_AGENT_CATALOG=on"}` si catalogue non chargé | app.py:833-835 |
| **~~Orphelin potentiel~~ : trace_writer (CÂBLÉ-ACTIF)** | ✅ Corrigé : `src/trace_writer.py` existe et est appelé à chaque tour (`agent_loop.py:1934,3584`, `tool_execution.py:542,630`). Pas un orphelin. | Le faux négatif venait d'une lacune d'indexation CBM |

---

## B.6 — DIAGRAMME D'ORCHESTRATION (texte)

```
                   ┌──────────┐
                   │   UI     │ (static/index.html)
                   │ (SPA)    │
                   └────┬─────┘
                        │ POST /api/chat/stream (SSE)
                        ▼
┌──────────────────────────────────────────────────────┐
│                    app.py (FastAPI)                   │
│  AuthMiddleware → routes/chat_routes → agent_loop    │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│              stream_agent_loop (~1800 lignes)         │
│                                                      │
│  ┌─ ENTRÉE ──────────────────────────────────────┐  │
│  │ _classify_agent_request (admin, plan_mode...)   │  │
│  │ PhaseTracker.on_round_start [gated OFF]        │  │
│  │ router_advice [gated OFF]                      │  │
│  │ AgentDispatcher [gated OFF]                    │  │
│  └────────────────────────────────────────────────┘  │
│  ┌─ BUILD ───────────────────────────────────────┐  │
│  │ stream_llm_with_fallback → LiteLLM → modèle    │  │
│  │ tool_execution → parse + execute + gate        │  │
│  │   └─ destructive gate [ON] → bloque rm -rf...  │  │
│  └────────────────────────────────────────────────┘  │
│  ┌─ POST-ROUND ──────────────────────────────────┐  │
│  │ MemoryProviderRegistry.dispatch [ACONTEXT=off] │  │
│  │ record_run_ancestry [GOV_ANCESTRY=off]         │  │
│  │ record_checkpoint [CHECKPOINT=off]             │  │
│  │ run_codeburn [CODEBURN=off]                    │  │
│  │ Observer.ingest + drift_score [TOUJOURS ON]    │  │
│  │ apply_autoeval [AUTOEVAL=off]                  │  │
│  │ maybe_autoevolve [AUTOEVOLVE=off]              │  │
│  └────────────────────────────────────────────────┘  │
└──────────────────────┬───────────────────────────────┘
                       │
          ┌────────────┼──────────────┐
          ▼            ▼               ▼
   ┌──────────┐ ┌──────────┐  ┌──────────────┐
   │ llm_core │ │  tools   │  │   memory     │
   │ (LiteLLM)│ │(bash,fs, │  │ (vector,     │
   │          │ │  web...) │  │  skills,     │
   └──────────┘ └──────────┘  │  RAG)        │
                              └──────────────┘
```
