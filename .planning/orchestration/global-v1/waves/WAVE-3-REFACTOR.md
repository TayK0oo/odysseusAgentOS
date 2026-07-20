# VAGUE 3 — Refactor Structurel (5 agents, ~37h effort)

**Objectif:** Restructurer les composants monolithiques, migrer vector DB, sécuriser.
**Dépendance:** Vague 2 terminée.

---

## A12: LangGraph — Décomposition stream_agent_loop

**Outil:** LangGraph (MIT, graphe cyclique, checkpointing natif)
**Prompt:** `prompts/agent-12-langgraph.md`
**Durée estimée:** 16h

### Tâches:
1. Ajouter `langgraph` à `requirements.txt`
2. Modéliser les 7 phases comme `StateGraph`:
   - classify → know → plan → build → quality → autoeval → memory_observe
3. Checkpointing automatique entre chaque nœud
4. Human-in-the-loop: interrupt avant BUILD si DESTRUCTIVE
5. Tests: chaque nœud indépendant, testable isolément
6. Kill-switch: `ODYSSEUS_LANGGRAPH=off`
7. Coexistence: `stream_agent_loop` legacy ET `LangGraph` en parallèle (A/B test)

### Architecture cible:
```python
graph = StateGraph(AgentState)
graph.add_node("classify", classify_node)
graph.add_node("know", know_node)
graph.add_node("plan", plan_node)
graph.add_node("build", build_node)
graph.add_node("quality", quality_node)
graph.add_node("autoeval", autoeval_node)
graph.add_node("memory_observe", memory_observe_node)
graph.add_conditional_edges("classify", route_by_risk, {...})
graph.add_edge("know", "plan")
graph.add_edge("plan", "build")
graph.add_edge("build", "quality")
graph.add_edge("quality", "autoeval")
graph.add_edge("autoeval", "memory_observe")
graph.set_entry_point("classify")
```

---

## A13: Qdrant — Vector DB Migration

**Outil:** Qdrant (Apache 2.0, Rust, payload filtering, quantification)
**Prompt:** `prompts/agent-13-qdrant.md`
**Durée estimée:** 4h

### Tâches:
1. Ajouter `qdrant` au `docker-compose.yml` (profile `vectordb`)
2. Ajouter `qdrant-client` à `requirements.txt`
3. Créer `services/vector/qdrant_store.py` — implémentation parallèle à ChromaDB
4. Script migration: `scripts/migrate_chromadb_to_qdrant.py`
5. Config: `VECTOR_BACKEND=chromadb|qdrant`
6. Kill-switch: `ODYSSEUS_QDRANT=off`

---

## A14: Traefik — API Gateway

**Outil:** Traefik (MIT, reverse proxy + auto-SSL + rate limiting)
**Prompt:** `prompts/agent-14-traefik.md`
**Durée estimée:** 3h

### Tâches:
1. Ajouter `traefik` au `docker-compose.yml`
2. Config labels Docker pour auto-discovery des services
3. Let's Encrypt auto-SSL
4. Rate limiting middleware
5. Dashboard sur port 8080 (interne uniquement)
6. Documenter dans `docs/setup.md`

---

## A15: OPA — Policy-as-Code

**Outil:** OpenPolicyAgent (CNCF, Apache 2.0, Rego policies)
**Prompt:** `prompts/agent-15-opa.md`
**Durée estimée:** 6h

### Tâches:
1. Ajouter `opa` au `docker-compose.yml` (profile `security`)
2. Créer `config/policies/tool_access.rego`
3. Créer `config/policies/phase_lock.rego`
4. Remplacer `phase-lock.yaml` + `destructive_gate` par appels OPA
5. Tests Rego: `opa test config/policies/`
6. Hot-reload policies sans redémarrage

---

## A16: HTMX + Alpine.js — Frontend Réactif

**Outils:** HTMX (BSD) + Alpine.js (MIT) — 14KB + 15KB
**Prompt:** `prompts/agent-16-htmx-alpine.md`
**Durée estimée:** 8h

### Tâches:
1. Ajouter `htmx.min.js` et `alpine.min.js` dans `static/lib/`
2. Remplacer SSE EventSource → `hx-sse` sur chat stream
3. Remplacer fetch+DOM → `hx-post`/`hx-get` sur formulaires
4. Alpine.js pour dropdowns, modals, tabs (x-show, x-data)
5. Supprimer JS redondant (cible: -40% de JS)
6. Garder vanilla JS pour logique complexe (coexistence)

---

## CHECKLIST VAGUE 3

- [ ] LangGraph: 7 nœuds testés indépendamment
- [ ] stream_agent_loop legacy vs LangGraph: byte-identical output (kill-switch OFF)
- [ ] Qdrant: migration script fonctionnel, fallback ChromaDB intact
- [ ] Traefik: reverse proxy + SSL fonctionnels
- [ ] OPA: policies testées, décisions loggées
- [ ] HTMX: chat stream fonctionne via hx-sse
