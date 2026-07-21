# A — INVENTAIRE FONCTIONNEL (Axe A)

**Date** : 2026-07-07 | **Commit** : `2821daa`

> Chaque composant = une ligne. Statut = un seul émoji + preuve fichier:ligne.
> Légende statuts : 🟢 CÂBLÉ-ACTIF | 🔵 CÂBLÉ-DORMANT (kill-switch OFF) | 🟡 CONFIGURÉ-NON-CÂBLÉ | 🟠 STUB | 🔴 ORPHELIN | 👻 FANTÔME | ⚪ DOC-ONLY

---

## COUCHE 1 : BOUCLE D'EXÉCUTION (Agent Loop)

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| `stream_agent_loop` | fonction | `src/agent_loop.py:1934-3733` (~1800 lignes) | Boucle live principale : reçoit message utilisateur, itère rounds LLM, exécute tools, émet SSE | messages, session_id, model, options | SSE stream (yield), memory, traces | llm_core, tool_execution, mcp_manager, skills_manager | 🟢 | app.py (mount chat_routes → appelé à chaque message) |
| `PhaseTracker` | classe | `src/orchestrator/phase_tracker.py:29-64` | Pont loop live ↔ phase-lock : infère phase par round et push vers ToolRegistry | round_num, intent, plan_mode | phase (str), side-effect: set_phase() | ToolRegistry | 🔵 | agent_loop.py:2485-2538 ; kill-switch `ODYSSEUS_PHASE_TRACKER`=off |
| `CanonicalLoop` | classe | `src/orchestrator/loop.py:20-48` | Boucle 7 phases (CLASSIFY→KNOW→PLAN→BUILD→QUALITY→AUTOEVAL→MEMORY_OBSERVE) | session_id, registry | phase courante, side-effect: set_phase() | phases.py | 🔵 | agent_loop.py:2488-2497 ; kill-switch `ODYSSEUS_LIVE_ORCHESTRATION`=off |
| `router_advice` | module | `src/orchestrator/router_advice.py` | Advisory routing : classify_intent → rôle natif (log-only, n'écrase pas modèle user) | user_message, stage | {intent, suggested_role} | intent_gate | 🔵 | agent_loop.py:2504-2513 ; kill-switch `ODYSSEUS_MODEL_ROUTER`=off |
| `AgentDispatcher` | classe | `src/orchestrator/agent_dispatcher.py:86-228` | Dispatch auto agents .opencode/ par phase canonique + explicite (/agent, /design) | phase, session_id, context | Résultat LLM (best-effort) | AgentRegistry, task_endpoint | 🔵 | agent_loop.py:2518-2570 ; tous les kill-switches agents=off |
| `_budget_enforcer` | objet | `src/agent_loop.py` (intégré) | Hard-stop après max_rounds : stop la loop quand budget itérations épuisé | round_num, max_rounds | stop ou continue | - | 🟢 | agent_loop.py:2572-2576 (budget itération) |
| `_classify_agent_request` | fonction | `src/agent_loop.py:860-943` | Classification de la requête (admin, guide-only, plan_mode, teacher) | messages, settings | (is_admin, guide_only, plan_mode, teacher, …) | risk_classifier | 🟢 | agent_loop.py:860 |
| `_build_system_prompt` | fonction | `src/agent_loop.py:975-1459` | Construction du prompt système (tools, mémoire, skills, contexte) | session, model, tools, memory, skills | system_prompt (str) | memory, skills_manager, tool_schemas | 🟢 | agent_loop.py:975 |

---

## COUCHE 2 : ORCHESTRATEUR — PHASES & GATES

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| `Phase` (enum) | enum | `src/orchestrator/phases.py:14-21` | 7 phases canoniques : CLASSIFY, KNOW, PLAN, BUILD, QUALITY, AUTOEVAL, MEMORY_OBSERVE | - | - | - | 🟢 | phases.py:14-21 ; utilisé par CanonicalLoop, agent_dispatcher |
| `phase-lock.yaml` | config | `config/phase-lock.yaml:1-114` | 10 phases (dont RESEARCH, INNOVATE, VERIFY en + des 7 canoniques) + restrictions outils | - | - | - | 🟢 | config/phase-lock.yaml ; chargé par ToolRegistry |
| `should_block_destructive` | fonction | `src/orchestrator/gate.py:38-52` | Détecte commandes shell catastrophiques (rm -rf, mkfs, dd, fork bomb…) | tool_name, tool_args | block_reason ou None | risk_classifier | 🟢 | tool_execution.py:563-565 ; appliqué dans le flux d'exécution live |
| `gate_enabled` | fonction | `src/orchestrator/gate.py:24-27` | Vérifie kill-switch `ODYSSEUS_DESTRUCTIVE_GATE` (défaut ON) | - | bool | - | 🟢 | gate.py:24-27 ; défaut ON, appliqué dans tool_execution.py:564 |
| `Permission matrix` | doc+code | `permission-matrix.md` + `config/phase-lock.yaml` | Spécification : quels outils sont autorisés par phase | - | - | - | 🟡 | permission-matrix.md:17-34 ; **les phases ne sont appliquées que si PhaseTracker ON** (off par défaut) |
| `risk_classifier` | module | `src/risk_classifier.py` | Classifie outils/bash en niveaux READ/DRAFT/WRITE/EXEC/DESTRUCTIVE | tool_name, command | RiskLevel | - | 🟢 | gate.py:18 (utilisé par destructive gate) ; agent_loop.py (classification requêtes) |
| `command_validator` | intégré | `src/builtin_actions.py` (helper `_validate_shell_or_block`) | Valide commandes shell agent-autonomes (ssh, run_script, run_local) | command, session_id | block_reason ou None | BLOCKED_PATTERNS, SAFE_PREFIXES | 🟢 | builtin_actions.py ; STATE.md:91 (bypass shell couverts) |

---

## COUCHE 3 : OBSERVABILITÉ & QUALITÉ

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| `Observer` | classe | `src/observer.py:58-181` | Analyse drift : ingestion metrics + tool_events → compute one_shot_rate, waste_patterns, drift_score | metrics, tool_events | DriftLevel (LOW/MED/HIGH) | - | 🟢 | agent_loop.py:3649-3654 ; appelé dans bloc try-guardé (fresh-per-turn) |
| `Observer.ingest_metrics` | méthode | `src/observer.py:107` | Ingère metrics post-round (tool success/failure, touched_files) | metrics, tool_events | side-effect: state interne | - | 🟢 | agent_loop.py:3653 |
| `Observer.compute_drift_score` | méthode | `src/observer.py:146` | Calcule niveau de drift (HIGH si écriture harness, MEDIUM si reads harness, LOW sinon) | - | DriftLevel | - | 🟢 | agent_loop.py:3654 ; pilote décision autoeval (agent_loop.py:3669) |
| `apply_autoeval` | fonction | `src/orchestrator/autoeval.py` | Décision keep/revert post-build : revert si verifier_reasons non-vides OU DriftLevel.HIGH | verifier_reasons, drift_level, git_runner | {enabled, decision, reverted, error} | git_runner (injecté) | 🔵 | agent_loop.py:3667-3693 ; kill-switch `ODYSSEUS_AUTOEVAL`=off |
| `autoeval_enabled` | fonction | `src/orchestrator/autoeval.py` | Vérifie kill-switch `ODYSSEUS_AUTOEVAL` (défaut OFF) | - | bool | - | 🔵 | agent_loop.py:3667 ; défaut OFF |
| `_run_verifier_subagent` | fonction | `src/agent_loop.py:1801-1848` | Sous-agent vérification (effectful-only, capé, OFF par défaut) | messages, model | verifier_reasons | llm_core | 🔵 | agent_loop.py:1801 ; gate `agent_verifier_subagent` |
| `run_codeburn` | fonction | `src/orchestrator/codeburn_runner.py` | Exécute CodeBurn CLI → rapport one-shot rate, waste patterns, coût vs commits | session_id | rapport | - | 🔵 | agent_loop.py:3640-3645 ; kill-switch `ODYSSEUS_CODEBURN`=off |
| `maybe_autoevolve` | fonction | `src/orchestrator/autoevolve.py` | Auto-évolution : déclenche recherche d'amélioration après drift HIGH + revert | session_id, run_id, drift_level | - | - | 🔵 | agent_loop.py:3698-3712 ; kill-switch `ODYSSEUS_AUTOEVOLVE`=off |
| `trace_writer` | module | `src/trace_writer.py` | Écriture traces JSONL (run_id, tool, risk_level, outcome…) + token accounting unifié | tool_call_data | JSONL ligne dans data/traces/ | - | 🟢 | **CÂBLÉ-ACTIF** : call-sites `agent_loop.py:1934,3584`, `tool_execution.py:542,630`, kill-switch `ODYSSEUS_UNIFIED_TOKENS` (`killswitch_registry.py:64`). Le « non trouvé » initial était une lacune d'indexation CBM. |

---

## COUCHE 4 : GOUVERNANCE & MÉMOIRE

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| `record_run_ancestry` | fonction | `src/orchestrator/ancestry_tracker.py` | Écrit GoalTask avec ancestry (mission→goal→project→task) dans la DB | session_id, run_id, metrics, project_id | side-effect: DB write | core.database | 🔵 | agent_loop.py:3604-3617 ; kill-switch `ODYSSEUS_GOVERNANCE_ANCESTRY`=off ; no-op sans project_id |
| `record_checkpoint` | fonction | `src/orchestrator/checkpoint_tracker.py` | Persiste checkpoint post-round dans Obsidian (write leg) | session_id, run_id, metrics, outcome | side-effect: Obsidian create_note | obsidian_mcp | 🔵 | agent_loop.py:3625-3633 ; kill-switch `ODYSSEUS_CHECKPOINT`=off |
| `AcontextMemoryProvider` | classe | `src/memory_provider.py` (via registry) | Provider observe-only : distillation fin-de-session → Acontext | session data | side-effect: HTTP POST → acontext | acontext (service) | 🔵 | agent_loop.py:3586-3601 ; gate `ACONTEXT_ENABLED`=off |
| `MemoryProviderRegistry` | classe | `src/memory_provider.py` | Registre de providers mémoire. dispatch_session_end() → tous les providers actifs | session data | side-effect: chaque provider actif | AcontextMemoryProvider (si activé) | 🟢 | agent_loop.py:3592 ; appelé à chaque fin de session |
| `MemoryVectorStore` | classe | `src/memory_vector.py` | Store vectoriel de mémoire (ajout + recherche sémantique) | embeddings, metadata | search results | chromadb, fastembed | 🟢 | agent_loop.py (appelé dans le flux de mémoire) |
| `SkillsManager` | classe | `services/memory/skills.py` | Gestion des skills extraits des sessions : search, record_use, backfill_owner | query, owner | skills pertinentes | - | 🟢 | app.py:548,610 ; agent_loop.py (appelé pour skills pertinentes) |
| `VectorRAG` (Personal Docs) | module | `src/rag_manager.py` + `src/rag_vector.py` | RAG vectoriel sur documents personnels (ChromaDB + BM25 + embeddings) | query, documents | ranked results | chromadb, fastembed, rank-bm25 | 🟢 | app.py:511-520 ; utilisé par /api/personal/* |
| `RRF hybrid search` | fonction | `src/rag_vector.py` (fonction pure `hybrid_search`) | Fusion vecteur+BM25 par Reciprocal Rank Fusion (alpha·vecteur + (1-alpha)·BM25) | vector_results, bm25_results, alpha, k | re-ranked results | - | 🔵 | rag_vector.py ; kill-switch `ODYSSEUS_RRF_FUSION`=off ; quand OFF → blend naïf 0.7/0.3 |
| `budget_enforcer` (projet) | module | Référencé dans `agent_loop.py:2572` et `PROJECT.yaml.example` | Hard-stop tokens/itérations par projet | max_tokens, max_iterations | stop signal | - | 🟡 | PROJECT.yaml.example existe ; budget itérations 🟢 (agent_loop.py:2572) ; budget tokens 🟡 (déclaré, intégration partielle) |

---

## COUCHE 5 : CONNAISSANCE (Trinité)

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| CBM (codebase-memory) | service MCP | `docker-compose.yml:366-388` | Graphe de connaissance du code : search_graph, trace_path, query_graph… | requêtes Cypher/BM25/sémantique | résultats structurés | ghcr.io/superpowers-sh/codebase-memory-mcp | 🟢 | docker-compose.yml:366 ; profil `knowledge` ; **CBM externe actif** (11749 nœuds) |
| Graphify | service Docker | `docker-compose.yml:390-414` | Graphe sémantique on-demand → communautés, HTML+JSON+audit | code source | graphe sémantique | graphifyy (pip) | 🔵 | docker-compose.yml:390 ; gate `ODYSSEUS_GRAPHIFY`=off ; STATE.md:46 « 0 svc » |
| Obsidian MCP (read) | script Python | `mcp_servers/obsidian_mcp.py` | Expose `list_notes`/`get_note`/`search_notes` (pas create_note — write leg = checkpoint natif) | vault path | notes | obsidian-vault (volume) | 🔵 | mcp_servers/obsidian_mcp.py ; gate `ODYSSEUS_OBSIDIAN_MCP`=off |
| Obsidian MCP (write) | intégré | `src/orchestrator/checkpoint_tracker.py:119` | Write leg : `obsidian_mcp.create_note` pour persister checkpoints | session data | note Obsidian | checkpoint_tracker | 🔵 | STATE.md:135 ; gate `ODYSSEUS_CHECKPOINT`=off |
| Knowledge routes | routes API | `routes/knowledge_routes.py` + `app.py:823-824` | Routes `/api/knowledge/*` (status, search) — orchestre Trinité | - | JSON status, search results | VectorRAG (natif, remplace Graphify pour search) | 🔵 | app.py:823-824 ; routes chargées mais dormantes (0 appelant frontend/loop) |

---

## COUCHE 6 : CHANNEL GATEWAY

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| `ChannelGateway` | classe | `src/channel_gateway.py` | Abstraction inbound/outbound bus : register_adapter, set_inbound_handler, send, start_all | adapters, handlers | delivery | adapters | 🟢 | channel_bootstrap.py:24-30 (importé, utilisé) |
| `bootstrap_channels` | fonction | `src/channel_bootstrap.py:201-247` | Point d'éveil unique : construit adapters, enregistre sur gateway, démarre écoute | gateway, adapter_factory | liste des channels démarrés | ChannelGateway, adapters | 🔵 | app.py:1243-1252 (appelé au startup) ; gate `ODYSSEUS_INPROCESS_*`=off |
| `DiscordAdapter` | classe | `src/adapters/discord_adapter.py` | Adapter Discord (discord.py) : listen, send, reply_fn natif | bot_token, channel_id | messages inbound/outbound | discord.py | 🔵 | channel_bootstrap.py:185 ; gate `ODYSSEUS_INPROCESS_DISCORD`=off |
| `TelegramAdapter` | classe | `src/adapters/telegram_adapter.py` | Adapter Telegram (python-telegram-bot) : listen, send, reply_fn natif | bot_token, chat_id | messages inbound/outbound | python-telegram-bot | 🔵 | channel_bootstrap.py:192 ; gate `ODYSSEUS_INPROCESS_TELEGRAM`=off |
| `run_agent_reply` | fonction | `src/channel_bootstrap.py:68-126` | Round-trip inbound→agent→reply : one-shot LLM sur message entrant | InboundMessage | reply text, delivery | task_llm_call_async | 🔵 | channel_bootstrap.py:68 ; gate `ODYSSEUS_CHANNEL_AGENT_REPLY`=off |
| `make_inbound_handler` | fonction | `src/channel_bootstrap.py:129-158` | Construit handler inbound : message → event_bus.fire_event + agent reply (si activé) | fire_event, agent_call | handler async | event_bus | 🔵 | channel_bootstrap.py:129 ; appelé dans bootstrap_channels |

---

## COUCHE 7 : MCP EXTERNES

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| Scrapling MCP | service Docker | `docker-compose.yml:336-364` | Web scraping via MCP (streamable HTTP /mcp) | URLs | Markdown extrait | scrapling[all] | 🔵 | docker-compose.yml:336 ; profil `scrapling` ; `SCRAPLING_ENABLED`=false |
| Kroki (diagrammes) | service Docker | `docker-compose.yml:253-288` | Rendu diagrammes (Mermaid, BPMN, Excalidraw…) → SVG/PNG | diagramme texte | SVG/PNG | kroki + kroki-mermaid | 🔵 | docker-compose.yml:253 ; `KROKI_ENABLED`=off |
| Serena MCP | service Docker | `docker-compose.yml:290-312` | Édition symbolique de code : find_symbol, replace_symbol_body | projet, symboles | modifications | serena (uvx) | 🔵 | docker-compose.yml:290 ; toujours démarré (pas de profile) ; **intégration agent non vérifiée** |

---

## COUCHE 8 : MODEL ROUTING

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| `model-routing.json` | config | `model-routing.json:1-136` | Politique de routing : 6 providers, 6 modèles, 8 stages, heuristic, fallback chains OmO | intent_category, stage | model_id | - | 🟡 | model-routing.json ; **PARTIEL** : `providers.*` redondant avec ModelEndpoint natif ; `models`/`stages`/`heuristic` = politique UNIQUE de routing par complexité |
| `zen_router` | module | `src/zen_router.py` | Routing Zen (OpenCode Go) : résout endpoint/candidats, fallback chains, blacklist | intent | model_id + endpoint | model-routing.json, OPENCODE_API_KEY | 🟢 | llm_core.py:2457 (dispatch Zen live) ; gated `OPENCODE_API_KEY` |
| `llm_core.stream_llm_with_fallback` | fonction | `src/llm_core.py` | Streaming LLM avec fallback : essaie modèles en cascade (429/5xx → suivant) | messages, model, endpoint | SSE stream | zen_router, endpoint_resolver | 🟢 | agent_loop.py (appelé à chaque tour de chat) |
| `ModelEndpoint` (natif) | DB model | `core/models.py` (via `manage_endpoints`) | Endpoint modèle natif (host, api_key, provider…) | - | - | - | 🟢 | STATE.md:85-86 (migration Zen→ModelEndpoint) ; admin tool `manage_endpoints` natif |
| `resolve_endpoint` | fonction | `src/endpoint_resolver.py` | Résout endpoint modèle depuis la DB native (ModelEndpoint) | host, provider | endpoint config | core.database | 🟢 | agent_loop.py:696-717 (utilisé pour résoudre endpoints) |
| `ModelRouter` (supprimé) | classe | `src/llm_router.py` (SUPPRIMÉ M3.1) | Ancien routeur par stage/model — jugé REDONDANT avec le natif | - | - | - | 👻 | STATE.md:92 ; supprimé commit c4cda4b ; **ne plus référencer** |

---

## COUCHE 9 : UI & FRONTEND

| Composant | Type | Emplacement | Fonction réelle | Entrées | Sorties | Dépend de | Statut | Preuve |
|---|---|---|---|---|---|---|---|---|
| SPA Shell (index.html) | page HTML | `static/index.html` | Application monopage : chat, notes, calendrier, cookbook, email, mémoire, galerie, tâches, library | - | UI interactive | static/js/* (modules ES) | 🟢 | app.py:881-907 (9 routes servent le même SPA) |
| Agent indicators (M4) | composant JS | `static/js/` (intégré dans SSE handler) | Affichage temps réel du dispatch d'agents (SSE events `agent_dispatch`) | SSE events | Badges UI "agent: running/completed" | agent_dispatcher (backend) | 🟢 | agent_loop.py:2558-2560 (yield SSE agent_dispatch) ; commit `2b58fd3` |
| Login page | page HTML | `static/login.html` | Authentification : login, signup, setup | - | Formulaire auth | auth_routes | 🟢 | app.py:934-938 |
| Gallery | composant JS | `static/js/gallery` + `routes/gallery_routes.py` | Galerie d'images : upload, browse, transform | images | galerie UI | upload_handler | 🟢 | app.py:688-689 ; routes/gallery_routes.py |
| Editor (image) | composant JS | `static/js/editor/` (build, filters, fx, tools) | Éditeur d'images avec filtres et outils | images | images modifiées | canvas API | 🟢 | static/js/editor/ ; routes/editor_draft_routes.py |
| Calendar UI | composant JS | `static/js/calendar/` | Calendrier CalDAV | events | UI calendrier | calendar_routes | 🟢 | app.py:897-899 (route /calendar) |
| Email client | composant JS | `static/js/emailLibrary/` | Client email complet (IMAP/SMTP) | emails | UI email | email_routes | 🟢 | app.py:910-911 (route /email) |
| Cookbook UI | composant JS | `static/js/model/` + `routes/cookbook_routes.py` | Gestion modèles : download, serve, probe, hwfit | modèles | UI modèles | cookbook_routes | 🟢 | app.py:905-907 (route /cookbook) |
| Backgrounds sandbox | page HTML | `static/backgrounds.html` | Page sandbox prototypage effets visuels | - | CSS effects | - | 🟢 | app.py:930-932 |

---

## COUCHE 10 : AGENTS .OPENCODE/ (12 agents)

Tous les agents sont en mode `primary`. Aucun n'a de `model` explicite dans son frontmatter (utilisent le défaut système). Tous ont leur kill-switch individuel (défaut OFF).

| Agent | Phase attachée | Kill-switch | Statut | Preuve |
|---|---|---|---|---|
| `constitution` | CLASSIFY | `ODYSSEUS_AGENT_CONSTITUTION` | 🔵 | agent_dispatcher.py:40-42 |
| `gsd-planner` | PLAN | `ODYSSEUS_AGENT_PLANNER` | 🔵 | agent_dispatcher.py:47-49 |
| `gsd-researcher` | PLAN | `ODYSSEUS_AGENT_RESEARCHER` | 🔵 | agent_dispatcher.py:48-49 |
| `gsd-executor` | BUILD | `ODYSSEUS_AGENT_EXECUTOR` | 🔵 | agent_dispatcher.py:51-53 |
| `gsd-debugger` | BUILD (on fail) | `ODYSSEUS_AGENT_DEBUGGER` | 🔵 | agent_dispatcher.py:16 (mentionné, pas dans _PHASE_AGENTS) |
| `debate-5-personas` | QUALITY | `ODYSSEUS_AGENT_DEBATE` | 🔵 | agent_dispatcher.py:54-56 |
| `security-audit` | QUALITY | `ODYSSEUS_AGENT_SECURITY` | 🔵 | agent_dispatcher.py:55-56 |
| `gsd-verifier` | AUTOEVAL | `ODYSSEUS_AGENT_VERIFIER` | 🔵 | agent_dispatcher.py:58-60 |
| `edge-case-gen` | AUTOEVAL | `ODYSSEUS_AGENT_EDGECASE` | 🔵 | agent_dispatcher.py:59-60 |
| `gsd-roadmapper` | MEMORY_OBSERVE | `ODYSSEUS_AGENT_ROADMAPPER` | 🔵 | agent_dispatcher.py:62-64 |
| `design-extract` | Explicite seulement | `ODYSSEUS_AGENT_DESIGN_EXTRACT` | 🔵 | agent_dispatcher.py:68-69 |
| `open-design` | Explicite seulement | `ODYSSEUS_AGENT_OPEN_DESIGN` | 🔵 | agent_dispatcher.py:70 |

---

## COUCLE 11 : INFRASTRUCTURE & SERVICES

| Composant | Type | Emplacement | Fonction réelle | Statut | Preuve |
|---|---|---|---|---|---|
| FastAPI app | serveur | `app.py:120-1302` | Application principale : monte 60+ routeurs, CORS, auth, middlewares, static, lifespan | 🟢 | app.py:120 |
| Auth (session + bearer) | middleware | `app.py:312-427` | AuthMiddleware : cookie session + token bearer (API tokens `ody_*`) + internal tool bypass | 🟢 | app.py:312-427 |
| CORS | middleware | `app.py:127-145` | CORSMiddleware : origins configurables, credentials, custom headers | 🟢 | app.py:128-145 |
| GZip compression | middleware | `app.py:147-155` | GZipMiddleware : compresse assets >1KB (exclut SSE streams) | 🟢 | app.py:155 |
| Security headers | middleware | `core/middleware.py:62-126` | SecurityHeadersMiddleware : CSP, HSTS, X-Frame-Options, etc. | 🟢 | app.py:157-158 |
| Request timeout | middleware | `app.py:171-200` | Timeout 45s sur requêtes (sauf streaming/longs chemins exemptés) | 🟢 | app.py:171-200 |
| SQLite DB | stockage | `core/database.py` + `data/app.db` | Base principale : sessions, messages, tokens, gallery, tasks, calendar… | 🟢 | docker-compose.yml:45 (DATABASE_URL) |
| ChromaDB | service | `docker-compose.yml:160-178` | Vector store pour RAG + embeddings | 🟢 | docker-compose.yml:160 (toujours démarré) |
| SearXNG | service | `docker-compose.yml:180-230` | Meta-search engine (web search) | 🟢 | docker-compose.yml:180 (dépendance odysseus) |
| ntfy | service | `docker-compose.yml:232-251` | Notifications push | 🟡 | docker-compose.yml:232 (toujours démarré) ; **intégration UI non vérifiée** |
| Decision Engine | service | `docker-compose.yml:416-440` (profil `decision-engine`) | Orchestration externe (repo ConfigOpenCodeNew) | 🟡 | docker-compose.yml:416 ; profil, pas démarré par défaut |
| Acontext | service | `docker-compose.yml:314-334` (profil `acontext`) | Distillation mémoire self-hosted | 🔵 | docker-compose.yml:314 ; profil ; gate `ACONTEXT_ENABLED`=false |
| Docker hardening | config | `docker-compose.yml:134-144` | cap_drop: ALL, no-new-privileges: true, tmpfs /tmp | 🟢 | docker-compose.yml:134-144 |
| Email pollers | module | `routes/email_pollers.py` | Polling IMAP périodique | 🟢 | app.py:1181-1183 (gate `ODYSSEUS_INPROCESS_POLLERS`) |
| Task scheduler | module | `src/task_scheduler.py` + `app.py:696-701` | Tâches planifiées (cron, event, webhook) + event bus intégré | 🟢 | app.py:696-701 ; démarrage gate `ODYSSEUS_INPROCESS_TASKS` |

---

## COUCHE 12 : DOUBLONS FONCTIONNELS DÉTECTÉS

| Fonction | Doublon 1 | Doublon 2 | Verdict | Preuve |
|---|---|---|---|---|
| Routing modèle | `zen_router` (src/zen_router.py) — live | `ModelEndpoint` natif (core/models.py) — DB | **PARTIEL** : `zen_router` utilise `model-routing.json providers.*` (redondant avec DB). `models`/`stages`/`heuristic` du JSON = UNIQUE (routing par complexité que le natif n'a pas). | STATE.md:85-86 ; ROADMAP-M3-ORCHESTRATION.md:198 |
| Classification risque shell | `risk_classifier.DESTRUCTIVE_PATTERNS` (classifieur large) | `command_validator.BLOCKED_PATTERNS` (enforceur étroit) | **2 POLITIQUES DISTINCTES, PAS DOUBLON** : classifieur = tout `rm -rf`/`DROP TABLE` (gate + trace) ; enforceur = catastrophique-irréversible seulement (`rm -rf /`/`*`). | STATE.md:118-125 |
| Autoeval | `autoeval.py` (greffé orchestrator) | `_run_verifier_subagent` (natif agent_loop.py) | **REDONDANT résolu** : `apply_autoeval` consomme la sortie du verifier natif. | agent_loop.py:3667 |
| Bus événements | `ChannelGateway` (greffé) | `event_bus.py` (natif) + `ScheduledTask(trigger_type=event)` | **REDONDANT résolu** : inbound → `event_bus.fire_event`, plus de bus parallèle. | channel_bootstrap.py:150 |
| Recherche sémantique | `Graphify` (service externe) | `VectorRAG` (natif ChromaDB) | **REDONDANT résolu** : knowledge_routes utilise VectorRAG natif. Graphify = profil Docker seulement. | STATE.md:88 |
| Tokens accounting | 5 lieux de comptage (agent_loop, llm_core, trace_writer, budget, observer) | `unified_tokens` (M3.X) | **UNIFIÉ** : kill-switch `ODYSSEUS_UNIFIED_TOKENS` avec run_id de corrélation. | STATE.md:202 |
