# EXPLICIT FEATURE MAP — OpenCode ↔ Odysseus ↔ Nos Créations

## Architecture: Odysseus = Interface + Couche Métier sur OpenCode

```
┌─────────────────────────────────────────────────────────────┐
│                     ODYSSEUS (Frontend + Backend)            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  │
│  │ Chat UI  │  │ Cockpit  │  │ Settings │  │ Dashboard  │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └─────┬──────┘  │
│       │              │              │              │         │
│  ┌────▼──────────────▼──────────────▼──────────────▼──────┐ │
│  │              COUCHE MÉTIER SFD v3.0 (NOS CRÉATIONS)    │ │
│  │  Pipeline 7 phases, Mémoire+Provenance, Préférences,   │ │
│  │  Exécution durable, Classification, Sécurité, Visuel   │ │
│  └────────────────────────┬───────────────────────────────┘ │
│                           │                                  │
│  ┌────────────────────────▼───────────────────────────────┐ │
│  │         OP ENCODE (via OPENCODE_API_KEY)                │ │
│  │  ZenRouter, Models, Agents, MCP, Tools, Streaming       │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

OUI — Odysseus agit comme une interface (chat UI, cockpit, settings) avec une couche métier SFD v3.0 (nos 11 modules) par-dessus les capacités natives d'OpenCode (LLM, agents, MCP, ZenRouter).

---

## TABLEAU COMPLET

| # | Feature | OpenCode natif | Odysseus natif | Notre création | Adaptation | Testé live |
|---|---------|---------------|----------------|----------------|------------|------------|
| 1 | **Chat UI** | Non | OUI (FastAPI+SSE+JS) | — | — | OUI |
| 2 | **ZenRouter (model routing)** | OUI | — | — | Déjà intégré | OUI |
| 3 | **Provider switching** | OUI | OUI (model-routing.json) | — | Déjà intégré | OUI |
| 4 | **LLM streaming (SSE)** | OUI (via API) | OUI (stream_agent_loop) | — | Déjà intégré | OUI |
| 5 | **MCP Servers** | OUI (protocole) | OUI (mcp_manager.py, 6 built-in) | — | Déjà intégré | OUI |
| 6 | **Tools (BASH, WRITE_FILE...)** | OUI (via LLM) | OUI (tool_implementations.py) | — | Déjà intégré | OUI |
| 7 | **Skills (SKILL.md)** | OUI (Fable 5) | OUI (SkillsManager) | 7 skills SFD | OUI (chargement auto) | OUI |
| 8 | **12 Agents .opencode/** | OUI (architecture) | OUI (registry+dispatcher) | — | OUI (activation) | NON (OFF→ON) |
| 9 | **Kill-switches** | Non | OUI (38 existants) | +5 SFD | OUI (registre) | OUI |
| 10 | **Memory (ChromaDB)** | Non | OUI (memory_vector.py) | — | — | OUI |
| 11 | **Cockpit (phase bar)** | Non | OUI (cockpit.js) | Extension 7 phases | OUI (JS+CSS) | OUI |
| 12 | **Email/Calendar/Tasks** | Non | OUI (routes natives) | — | — | OUI |
| 13 | **Cookbook (model mgmt)** | Non | OUI (cookbook.js) | — | — | OUI |
| 14 | **Gallery/Documents** | Non | OUI (routes natives) | — | — | OUI |
| 15 | **Deep Research** | Non | OUI (research_handler) | — | — | OUI |
| 16 | **Search (Meilisearch)** | Non | OUI (gated) | — | — | OUI |
| 17 | **Notifications (ntfy)** | Non | OUI (apprise) | — | — | OUI |
| 18 | **Thought Bus (7 phases)** | Non | Non | OUI (src/thought_bus/) | — | OUI |
| 19 | **Durable Execution** | Non | Non | OUI (src/durable_execution/) | — | OUI |
| 20 | **Memory Provenance** | Non | Non | OUI (src/memory_provenance/) | OUI (dual-write) | OUI |
| 21 | **Preferences System** | Non | Non | OUI (src/preferences/) | — | OUI |
| 22 | **Visual Output Router** | Non | Non | OUI (src/visual_output/) | OUI (Kroki) | OUI |
| 23 | **Data Classification** | Non | Non | OUI (src/classification/) | — | OUI |
| 24 | **Content Security** | Non | Non | OUI (src/content_security/) | — | OUI |
| 25 | **Tool Discovery** | Non | Non | OUI (src/tool_discovery/) | — | OUI |
| 26 | **Conversation Search** | Non | Non | OUI (src/conversation_search/) | — | OUI |
| 27 | **Multi-Agent Decision** | Non | Non | OUI (src/multi_agent_decision/) | — | OUI |
| 28 | **Context Manager** | Non | Non | OUI (src/context_manager/) | — | OUI |
| 29 | **Mode Detector** | Non | Non | OUI (src/mode_detector.py) | — | OUI |
| 30 | **Agent Pipeline** | Non | Non | OUI (src/agent_pipeline.py) | — | OUI |
| 31 | **Full System Integration** | Non | Non | OUI (src/full_system.py) | — | OUI |
| 32 | **Agent Instructions (BP)** | Non | Non | OUI (src/agent_instructions.py) | — | OUI |
| 33 | **SFDD System Wiring** | Non | Non | OUI (src/sfd_wiring.py) | — | OUI |
| 34 | **Service Connector** | Non | Non | OUI (src/service_connector.py) | — | OUI |
| 35 | **Plugin System** | Non | Non | OUI (src/plugin_system.py) | — | OUI |
| 36 | **Git Worktrees** | Non | Non | OUI (src/worktree_support.py) | — | OUI |
| 37 | **E2E Test Script** | Non | Non | OUI (scripts/test-e2e.sh) | — | OUI |
| 38 | **CI/CD (GitHub Actions)** | Non | Non | OUI (.github/workflows/e2e.yml) | — | OUI |
| 39 | **7 Skills (SKILL.md)** | Non | Non | OUI (skills/*/SKILL.md) | — | OUI |
| 40 | **Phase Bar (UI)** | Non | Non | OUI (cockpit.js modifié) | — | OUI |

---

## RÉPARTITION

| Origine | Nombre | Exemples |
|---------|--------|----------|
| **OpenCode natif** | 4 | ZenRouter, MCP protocol, LLM streaming, Skills concept |
| **Odysseus natif** | 13 | Chat UI, Tools, Cockpit, Email, Cookbook, Memory(ChromaDB), Kill-switches, etc. |
| **Nos créations SFD v3.0** | 20 | ThoughtBus, DurableExec, MemoryProvenance, Preferences, Visual, Classification, Security, Discovery, ConversationSearch, MultiAgentDecision, ContextManager, ModeDetector, Pipeline, FullSystem, Instructions, Wiring, Connector, Plugin, Worktree, E2E |
| **Adaptations (Odysseus←SFD)** | 3 | Memory dual-write, Visual→Kroki, Phase bar→cockpit |

## RÉPONSE DIRECTE

**Odysseus agit-il comme une interface avec une couche métier sur OpenCode ?**

OUI. Exactement :
- **OpenCode** = le moteur (LLM, agents, MCP, ZenRouter, streaming)
- **Odysseus** = l'interface utilisateur (chat, cockpit, settings, tools)
- **Notre couche SFD v3.0** = la couche métier (pipeline 7 phases, mémoire avec provenance, exécution durable, préférences, classification, sécurité, etc.) qui orchestre et enrichit chaque interaction

Les trois couches sont interconnectées : l'utilisateur parle à l'UI Odysseus → notre pipeline SFD analyse/orchestre → OpenCode exécute avec le LLM/agents → le résultat remonte enrichi dans l'UI.
