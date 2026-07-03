# INTEL — INDEX (L3 routing table + redundancy map)

> Généré 2026-07-02 après cartographie native complète d'Odysseus (7 agents parallèles).
> But : router chaque tâche vers le bon `domains/*.md` ET trancher, pour CHAQUE module
> greffé (harness / "intelligence layer"), s'il est **REDONDANT / PARTIEL / UNIQUE / DORMANT**
> face au natif — afin de ne câbler que ce qu'Odysseus n'a PAS déjà.

---

## 1. Routing table — quel domaine charger

| # | Domaine | Fichier | Charger quand la tâche touche… |
|---|---|---|---|
| 01 | Models & Providers | `domains/01-models-providers.md` | routing modèle, endpoints, providers, discovery, fallback, curated lists, zen_router, ModelRouter |
| 02 | Chat & Agent Loop | `domains/02-chat-agent-loop.md` | `stream_agent_loop`, rounds, guardrails (budget/loop-breaker/runaway/force-answer), verifier subagent, PhaseTracker seam, intent classification |
| 03 | Tools System | `domains/03-tools-system.md` | registres d'outils, schémas, `execute_tool_block`, phase-lock, gating, catalogue des ~70 outils natifs |
| 04 | Memory / RAG / Knowledge | `domains/04-memory-rag-knowledge.md` | mémoire, RAG, RRF/hybride, blend 0.7/0.3, embeddings, context budget/compaction, acontext, Trinité |
| 05 | Sessions / Persistence / Settings | `domains/05-sessions-persistence-settings.md` | SQLite/SQLAlchemy, sessions, chat_messages, settings.json, secrets Fernet, governance tables, trace_writer, observer, token accounting |
| 06 | Background / Tasks / Integrations | `domains/06-background-tasks-integrations.md` | TaskScheduler, bg_jobs, event_bus, webhooks, integrations, email/calendar channels, channel_gateway, adapters Discord/Telegram |
| 07 | Security / Sandbox / Validation | `domains/07-security-sandbox-validation.md` | prompt-injection, SSRF, tool_security RBAC/plan-mode, secrets, shell exec paths, docker hardening, destructive gate, command_validator |

---

## 2. Redundancy map — verdict par module greffé

Légende : **REDONDANT** = le natif fait déjà tout → supprimer/ne pas câbler ·
**PARTIEL** = chevauche le natif, garder seulement le delta unique · **UNIQUE** =
aucun équivalent natif → à câbler · **DORMANT** = unique mais jamais branché.

### Models & routing (domaine 01/02)
| Module greffé | Verdict | Équivalent natif | Action |
|---|---|---|---|
| `src/llm_router.py::ModelRouter` (+ `.complete()`) | **REDONDANT** | `resolve_endpoint` + `stream_llm_with_fallback` (rôle→endpoint→modèle+fallback) | Supprimer. Catalogue fantôme, 0 call-site de dispatch, dép. litellm morte. **Retirer d'abord le re-export `src/__init__.py:3`.** |
| `model-routing.json` + `stage-model-assignment.yaml` | **REDONDANT** | `_PROVIDER_CURATED` + settings rôles | Supprimer une fois zen migré. IDs fantômes (deepseek-v4-pro, kimi-k2.6, glm-5.2…). |
| `routes/routing_routes.py` (`/api/route*`) | **REDONDANT** | `/api/model-routes` + resolver | Supprimer (wrapper REST de zen_router). |
| `src/zen_router.py` | **PARTIEL** (1 bit unique) | `_resolve_fallback_candidates` + `stream_llm_with_fallback` | **NE PAS supprimer tel quel** : seul greffon sur un vrai chemin de dispatch (`llm_core.py:2457`, gated `OPENCODE_API_KEY`). Migrer l'endpoint Zen Go en `ModelEndpoint` natif avant retrait ; sinon le routing Go-plan + la blacklist par-modèle sont perdus. **[~] Slice 1 (2026-07-03)** : `_zen_provider_conn`/`_resolve_zen_endpoint_row` — base_url/key résolus depuis un `ModelEndpoint` natif (host `opencode.ai`) si enregistré, sinon fallback JSON+env inchangé. `providers.opencode_zen` n'est plus la seule source de vérité. Reste : gate d'activation (env → row) + retrait `model-routing.json`. |
| `src/intent_gate.py::classify_intent` | **UNIQUE (garder)** | — (natif `_classify_agent_request` produit des *domains*, pas des catégories quick/deep/…) | Garder comme classifieur ; recâbler vers les **rôles natifs**, pas un catalogue parallèle. |
| `src/orchestrator/router_advice.py` | **PARTIEL (bâti sur redondant)** | — | Retarget : garder intent_gate, retirer l'appel `ModelRouter.route`. Advisory only, kill-switch `ODYSSEUS_MODEL_ROUTER` OFF. |

### Chat loop guardrails (domaine 02)
| Module greffé | Verdict | Équivalent natif | Action |
|---|---|---|---|
| `PhaseTracker` (phase par round) | **UNIQUE** (inerte tant que `infer_phase` = BUILD/PLAN stub) | aucun (loop n'avait pas de notion de phase) | Garder ; enrichir `infer_phase` pour rendre phase-lock utile. Kill-switch `ODYSSEUS_PHASE_TRACKER` OFF. |
| `BudgetEnforcer` (budgets manifest) | **PARTIEL** | natif `max_rounds` + `max_tool_calls` (déjà des caps) | Garder seulement le budget multi-dim manifest ; ne pas re-compter l'itération cap. Inerte sans PROJECT.yaml. |
| Verifier / autoeval greffé | **REDONDANT** | natif `_run_verifier_subagent` (mécanisme 3a, effectful-only, capé, OFF par défaut) | **Réutiliser/étendre** le verifier natif ; ne PAS ajouter un 2ᵉ verifier parallèle (coût d'un round/tour). |
| Observer / metrics greffé | **PARTIEL** | natif `_compute_final_metrics` + `tool_events`/`round_texts` SSE persistés | Consommer le SSE `metrics`, ne pas recomputer. |

### Tools (domaine 03)
| Module greffé | Verdict | Équivalent natif | Action |
|---|---|---|---|
| `src/tool_registry.py` phase-lock (`is_tool_allowed`) | **PARTIEL** | `disabled_tools` + `plan_mode_disabled_tools` (name-based deny) | Garder l'axe UNIQUE (phase workflow + writes path-scoped + exec test-command allowlists) ; **réutiliser** le deny name-based natif au lieu de dupliquer en YAML. ⚠ fail-OPEN (natif plan-mode fail-closed). |
| `src/orchestrator/gate.should_block_destructive` | **UNIQUE-COMPLÉMENT** | risk_classifier natif ne faisait que *logger* | Garder — 1er vrai *block* du shell catastrophique. |
| Constitution trace (`risk_classifier` + `trace_writer`) | **UNIQUE-COMPLÉMENT** | aucun trace per-tool risk/outcome | Garder (audit). |

### Memory / RAG (domaine 04)
| Module greffé | Verdict | Équivalent natif | Action |
|---|---|---|---|
| RRF hybrid retrieval | **PARTIEL** | blend 0.7/0.3 ACTIF `rag_vector.py:378` ; un `hybrid_search` RRF EXISTE DÉJÀ `:731` mais ORPHELIN + bug reflection (`top_k` vs `k` `:749`) ; `ChatProcessor._hybrid_retrieve` = vrai BM25+vector pour la mémoire | **Ne pas ajouter un 3ᵉ RRF.** Réparer + câbler l'existant à la place du blend naïf (constantes `:37-38`). Faible blast radius. |
| acontext service | **PARTIEL** | `context_compactor` (résumés structurés), `MemoryProviderRegistry` (providers externes), `/extract` `/audit` | Distillation fin-de-session est nouvelle ; mais **alimenter le provider mémoire natif** via `MemoryProviderRegistry.register`, ne pas ré-implémenter recall/store. |
| Knowledge "Trinité" (CBM+Graphify+Obsidian) | **UNIQUE** | aucun (RAG natif = Chroma, pas graph/Obsidian) | Additif. Déléguer la recherche sémantique doc au `VectorRAG` natif plutôt qu'à Graphify. |

### Persistence / governance (domaine 05)
| Module greffé | Verdict | Équivalent natif | Action |
|---|---|---|---|
| `governance` budgets (`agent_budgets`, auto-pause) | **PARTIEL** | `settings.agent_*` (config only), `task_runs.tokens_used` (record, pas enforce) | Enforcement+persistence nouveaux ; dédupliquer la source de vérité tokens (voir risque #1). |
| `governance` heartbeat (`agent_heartbeats`) | **UNIQUE** | aucun (scheduler = temps, pas état agent reprenable) | Garder. |
| `governance` goal ancestry (missions/goals/…) | **UNIQUE** | aucun (natif = flat : folder, crew, tasks) | Garder. |
| `governance` approval gates (log-only) | **PARTIEL** | `agent_email_confirm` + `scheduled_emails` draft-staging (vrai flow d'approbation email) | Réutiliser le pattern email natif pour le générique. |
| `trace_writer` JSONL | **PARTIEL** | `task_runs.steps` + message meta_data | Granularité per-tool + champs risk/permission nouveaux ; cost_tokens/session_id dupliquent meta_data. |
| `observer` drift score | **UNIQUE** (RAM only) | aucun | Garder (éphémère, non persisté). |

### Background / channels (domaine 06)
| Module greffé | Verdict | Équivalent natif | Action |
|---|---|---|---|
| `ChannelGateway` outbound send/broadcast | **PARTIEL** | `TaskScheduler._deliver_task_result` (email+MCP+notif), `WebhookManager`, `execute_api_call` (preset `discord_webhook`) | Abstraction uniforme sans transport nouveau ; ⚠ broadcast `agent_loop.py:3521` spammerait chaque tour. |
| `ChannelType.EMAIL` / `WEBHOOK` enums | **REDONDANT** | email pollers + webhook_manager matures | Supprimer ces arms. |
| Event fan-out (gateway = bus) | **REDONDANT** | `event_bus.py` + `ScheduledTask(trigger_type=event)` | Utiliser le bus natif (étendre `ALLOWED_EVENTS`/publishers). |
| Adapters Discord/Telegram (2-way bots) | **UNIQUE mais DORMANT** | aucun (natif = discord_webhook outbound only) | Seul vrai neuf. `register_adapter`/`set_inbound_handler`/`start_all` = **0 caller hors tests**. Câbler à `event_bus.fire_event` (inbound) + delivery natif (outbound). |

### Security (domaine 07)
| Module greffé | Verdict | Équivalent natif | Action |
|---|---|---|---|
| `gate.should_block_destructive` (bash/python/#!bg) | **UNIQUE-COMPLÉMENT** | tool_security = RBAC/plan-mode orthogonal | Garder. |
| `command_validator` | **PARTIEL** | `risk_classifier.classify_bash` (liste de patterns séparée, ni superset) | Fusionner les 2 listes en UNE source canonique ; router `run_script`/`run_local`/`/api/shell/*` à travers. ⚠ bug SAFE_PREFIXES `startswith` (`ls; rm -rf /` passe). |
| prompt_security / url_safety / tls_overrides | **N/A natif** | complets nativement | Ne PAS dupliquer. |

---

## 3. Pièges de retrait (à respecter avant tout cleanup)

1. **`src/__init__.py:3-4` importe `ModelRouter` + `intent_gate` EAGERLY** → supprimer `llm_router.py` sans corriger `__init__.py` casse tout `import src.*`. **Retirer le re-export d'abord.**
2. **`zen_router` est LIVE** (`llm_core.py:2457`, gated `OPENCODE_API_KEY`) — seul greffon sur un vrai chemin de dispatch. Le supprimer change silencieusement le chat pour qui a cette clé. Migrer l'endpoint Zen Go → `ModelEndpoint` natif d'abord.
3. **Tests couplés** : `tests/test_harness_core.py:156-196` + `tests/test_orchestrator_router_advice.py` importent encore les routeurs greffés → à mettre à jour/retirer.
4. **Verifier duplication** : le verifier natif est OFF par défaut (faux rejets des petits modèles). Tout autoeval greffé doit garder le design effectful-only + capé + opt-in.

---

## 4. Risques transversaux (cross-domaine)

1. **Token accounting éclaté (risque #1)** : tokens dans `chat_messages.meta_data`, `sessions.total_*_tokens`, `task_runs.tokens_used`, `agent_budgets.tokens_used`, trace `cost_tokens`. Choisir UN writer de vérité (metrics message) ; governance/trace *lisent*, ne recomptent pas.
2. **`run_id` fragmenté** : trace_writer = `_RUN_ID` process-level, governance = caller-supplied, agent_runs = par session_id, task_runs.id = par exécution. Aucun id de corrélation partagé → joindre traces↔budgets↔task_runs impossible aujourd'hui.
3. **Deux boucles** : live `stream_agent_loop` vs orchestrée `CanonicalLoop` — la live n'appelait jamais `set_phase` → tout tombait sur `default_phase: BUILD` (racine du "35%"). PhaseTracker (M3.0) est le pont.
4. **Chemin low-signal direct** (`agent_loop.py:2022-2095`) saute tout le harness (pas de tools/phase/budget) — tout observer devant voir chaque tour a besoin d'un hook là.
5. **Sandbox** : `cap_drop:[ALL]` commenté + `docker.sock` monté = compromission shell → root hôte. Faiblesse #1.
6. **Deux "event" concepts** : `event_bus` natif (DB, owner-scopé) vs gateway (global, sans owner) → risque split-brain / fuite multi-user.

---

## 5. Ligne directrice M3 (re-plan)

**Ne câbler que l'UNIQUE / UNIQUE-COMPLÉMENT ; réparer le PARTIEL en place ; supprimer le REDONDANT ; réveiller le DORMANT via les seams natifs.**

- **Supprimer** (redondant) : ModelRouter, model-routing.json, stage-model-assignment.yaml, routing_routes.py, channel enums EMAIL/WEBHOOK, gateway-as-bus.
- **Réparer en place** (partiel) : RRF (câbler l'existant, fix reflection), command_validator (fusion 1 liste + couvrir les bypass shell), budget/trace/observer (lire la source tokens unique).
- **Câbler** (unique) : PhaseTracker+phase-lock (enrichir infer_phase), destructive gate (fait), governance heartbeat/ancestry, Trinité, adapters Discord/Telegram (via event_bus natif).
- **Migration bloquante** avant retrait routing : zen Go endpoint → `ModelEndpoint` natif.
