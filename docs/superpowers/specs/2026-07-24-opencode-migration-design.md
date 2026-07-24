# Migration Design — Odysseus → OpenCode Core

**Date :** 2026-07-24 | **Status :** Design Approved
**Source :** Brainstorming session — 4 key decisions

---

## Decisions

| # | Question | Decision |
|---|----------|----------|
| 1 | Migration strategy | **Big Bang** — complete switch, one commit |
| 2 | Plugin distribution | **npm packages** — @agentos/sfd-{memory,durable,prefs,visual,classify,security,discovery} |
| 3 | Bridge technology | **Subprocess stdin/stdout** — opencode CLI via asyncio.create_subprocess_exec |
| 4 | Cockpit events | **Plugin → stdout → bridge → SSE** — sfd-phase plugin emits JSON on stdout |

---

## Architecture

```
Odysseus (Python/FastAPI)
  opencode_bridge.py → subprocess → OpenCode CLI (Node.js)
    stdin: message
    stdout: response + phase events → SSE → cockpit + chat UI
    stderr: logs

OpenCode CLI uses:
  - ZenRouter (model selection)
  - 16 agents (.opencode/agents/)
  - 8 MCP servers
  - 7 npm plugins (@agentos/sfd-*)
  - 2 skills (.opencode/skills/)
  - 1 custom tool (.opencode/tools/sfd-phase.ts)
  - Native worktrees, compaction, permissions

Services Docker (unchanged):
  ChromaDB, Kroki, Meilisearch, ntfy, SearXNG, Scrapling, Serena, LangFuse
```

## 7 npm Plugins

| Package | SFD Module | Hooks | Custom Tools |
|---------|-----------|-------|-------------|
| @agentos/sfd-memory | Memory Provenance | session.created, tool.execute.after(write) | sfd-memory |
| @agentos/sfd-durable | Durable Execution | tool.execute.before/after(bash,write) | — |
| @agentos/sfd-prefs | Preferences | session.created | — |
| @agentos/sfd-visual | Visual Output | — | sfd-render |
| @agentos/sfd-classify | Classification | tool.execute.before(write) | sfd-forget |
| @agentos/sfd-security | Content Security | tool.execute.before(read), session.updated | — |
| @agentos/sfd-discovery | Tool Discovery | — | sfd-discover |

## Files to DELETE (~3000 lines Python)

```
src/thought_bus/          src/agent_pipeline.py     src/full_system.py
src/mode_detector.py      src/plugin_system.py      src/worktree_support.py
src/llm_core.py           src/zen_router.py         src/endpoint_resolver.py
src/durable_execution/    src/memory_provenance/    src/preferences/
src/visual_output/        src/classification/       src/content_security/
src/tool_discovery/       src/conversation_search/  src/multi_agent_decision/
src/context_manager/      src/agent_instructions.py src/sfd_wiring.py
src/service_connector.py
```

## Files to ADAPT

```
routes/chat_routes.py → OpenCodeBridge instead of stream_agent_loop
app.py → remove SFD init, keep Docker init
Dockerfile → +RUN curl -fsSL https://opencode.ai/install | bash
```

## Files to KEEP

```
All static/       All Docker services    All non-chat routes
.opencode/        skills/                tests/
scripts/          .github/workflows/     docker-compose.yml
```
