# ODYSSEUS — Architecture Réelle & Optimisations

## Flux natif Odysseus (sans notre couche SFD)

```
Navigateur (SSE)
  │ POST /api/chat_stream
  ▼
routes/chat_routes.py:chat_stream()
  │
  ├── 1. Vérifie auth, session, modèle, budget
  ├── 2. stream_agent_loop(endpoint_url, model, messages, tools, ...)
  │
  ▼
src/agent_loop.py:stream_agent_loop()  ← 3992 lignes, le CŒUR
  │
  ├── 3. Construit le prompt (system + memory + skills + context)
  ├── 4. Boucle max_rounds (défaut 10):
  │     ├── stream_llm_with_fallback(candidates, messages, tools)
  │     │     │
  │     │     ▼
  │     │   src/llm_core.py:stream_llm_with_fallback()  ← 2520 lignes
  │     │     │
  │     │     ├── src/zen_router.py:route_and_call()  ← ZenRouter
  │     │     │     └── POST https://api.opencode.ai/zen/...  ← OpenCode
  │     │     │
  │     │     └── OU httpx → provider direct (Ollama, Anthropic, OpenAI)
  │     │
  │     ├── Parse la réponse SSE → delta + tool_calls
  │     ├── Si tool_call → execute_tool_block()
  │     │     ├── outils internes (BASH, WRITE_FILE, WEB_SEARCH...)
  │     │     └── MCP tools (mcp__scrapling__fetch, mcp__email__send...)
  │     └── Si pas de tool_call → réponse finale
  │
  └── 5. Sauvegarde dans l'historique
```

## Flux avec notre couche SFD v3.0

```
Navigateur (SSE)
  │ POST /api/chat_stream
  ▼
routes/chat_routes.py  ← MODIFIÉ
  │
  ├── mode_detector.detect(message) → CHAT ou AGENT
  ├── Si AGENT:
  │     full_system_pipeline(message, session, agent_stream_fn)
  │       │
  │       ├── CLASSIFY → risk + multi-agent décision
  │       ├── KNOW → memory recall + conversation search
  │       ├── PLAN → decompose objectives + assign agents
  │       ├── BUILD → stream_agent_loop()  ← natif
  │       ├── QUALITY → security + lint
  │       ├── AUTOEVAL → score
  │       └── MEMORY → [stated] write
  │
  └── Si CHAT:
        stream_agent_loop()  ← natif direct
```

## Problèmes identifiés

| Problème | Impact | Solution |
|----------|--------|----------|
| **Double prompt** | Notre pipeline envoie phase_enter/exit SSE AVANT le stream_agent_loop, mais le stream_agent_loop construit déjà son propre prompt avec mémoire/skills | Unifier : injecter nos données (mémoire SFD, préférences, plan) DANS le prompt natif, pas à côté |
| **Phases non bloquantes** | CLASSIFY, KNOW, PLAN sont juste des messages SSE, ils n'affectent pas le comportement du LLM dans BUILD | Faire que chaque phase enrichisse le `messages` array passé à stream_agent_loop |
| **Agents non dispatchés par phase** | Les 12 agents sont dans .opencode/ mais notre pipeline ne les appelle pas via AgentDispatcher | Appeler AgentDispatcher.dispatch_for_phase() à chaque phase |
| **Cockpit ne reçoit que BUILD** | Le LLM ne voit que la phase BUILD parce que stream_agent_loop ne reçoit que le prompt original | Injecter le contexte de phase dans le prompt |
| **ZenRouter pas informé de la phase** | Le routeur ne sait pas si on est en PLAN ou BUILD, il route juste sur le contenu du message | Passer la phase au ZenRouter pour un routage plus fin |

## Plan d'optimisation

1. **Injecter les données de phase dans le prompt natif**
   → CLASSIFY: ajouter "Risk: {level}" au system prompt
   → KNOW: ajouter les mémoires rappelées au context
   → PLAN: ajouter le plan au system prompt

2. **Activer AgentDispatcher par phase**
   → appeler dispatch_for_phase("PLAN") → gsd-planner + gsd-researcher
   → leurs résultats enrichissent le contexte pour BUILD

3. **Utiliser le vrai multi_agent.py pour les projets complexes**
   → quand is_multi=True, utiliser MultiAgentWorkflow au lieu de stream_agent_loop

4. **Passer la phase au ZenRouter**
   → phase="PLAN" → modèle strong (deepseek-v4-pro)
   → phase="BUILD" → modèle code (minimax-m3)
   → phase="QUALITY" → modèle fast (deepseek-v4-flash)
