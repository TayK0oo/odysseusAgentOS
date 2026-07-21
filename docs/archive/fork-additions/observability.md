# Observability — LangFuse LLM Tracing

## Overview

Odysseus integrates [LangFuse](https://langfuse.com/) (MIT, self-hosted) for structured LLM tracing, replacing/augmenting the JSONL-based `trace_writer.py` with a full observability dashboard.

**What you get:**
- Structured traces for every LLM call, tool execution, and agent round
- Token usage and cost tracking per session/run
- Prompt versioning and comparison
- Latency and error tracking
- Session replay — "What did the LLM actually say?"

## Quick Start

### 1. Enable the kill-switch

```bash
# In .env or environment
ODYSSEUS_LANGFUSE=on
```

### 2. Start LangFuse (Docker)

```bash
docker compose --profile observability up -d
```

This starts LangFuse on `http://localhost:3000` with a PostgreSQL database.

### 3. First-time setup

1. Open `http://localhost:3000` in your browser
2. Create an admin account
3. Create a new project
4. Go to **Settings → API Keys** and copy the public + secret keys
5. Add them to your `.env`:

```bash
LANGFUSE_PUBLIC_KEY=lf-pub-...
LANGFUSE_SECRET_KEY=lf-sec-...
```

### 4. Restart Odysseus

The tracing will activate automatically on the next chat session.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Odysseus App                                   │
│                                                 │
│  stream_agent_loop()  ←── @observe()            │
│    ├── stream_llm_with_fallback() ← @observe() │
│    │     └── stream_llm()          ← @observe()│
│    └── execute_tool_block()  ← trace_langfuse() │
│                                                 │
│  All traces ──────────────► LangFuse Dashboard  │
│                            (localhost:3000)     │
└─────────────────────────────────────────────────┘
```

### Traced Functions

| Function | File | What it captures |
|---|---|---|
| `stream_agent_loop` | `src/agent_loop.py` | Full agent round (multi-turn) |
| `stream_llm_with_fallback` | `src/llm_core.py` | LLM fallback chain |
| `stream_llm` | `src/llm_core.py` | Individual LLM API call |
| `execute_tool_block` | `src/tool_execution.py` | Tool execution |

### Trace Hierarchy

```
Trace: stream_agent_loop (session_id, user_id, model)
  └── Generation: stream_llm_with_fallback
        └── Generation: stream_llm
              └── Span: tool:web_search
              └── Span: tool:bash
```

## Kill-Switch

| Variable | Default | Description |
|---|---|---|
| `ODYSSEUS_LANGFUSE` | `off` | Master kill-switch |
| `LANGFUSE_HOST` | `http://localhost:3000` | LangFuse server URL |
| `LANGFUSE_PUBLIC_KEY` | (empty) | API public key |
| `LANGFUSE_SECRET_KEY` | (empty) | API secret key |

When `ODYSSEUS_LANGFUSE=off`:
- Zero overhead: no imports, no network calls, no initialization
- Traces continue to go to JSONL files as before
- Existing behavior is byte-identical

When `ODYSSEUS_LANGFUSE=on`:
- Structured traces are sent to LangFuse
- JSONL traces are still written (dual-write)
- Dashboard at `http://localhost:3000` shows all traces

## Docker Compose

```bash
# Start LangFuse with its own profile
docker compose --profile observability up -d

# Check status
docker compose --profile observability ps

# View logs
docker compose --profile observability logs -f langfuse-server
```

### Services

| Service | Port | Description |
|---|---|---|
| `langfuse-server` | `127.0.0.1:3000` | LangFuse web UI + API |
| `langfuse-postgres` | internal | PostgreSQL database |

### Persistence

LangFuse data is stored in the `langfuse-db-data` Docker volume. To back up:

```bash
docker compose --profile observability exec langfuse-postgres pg_dump -U odysseus langfuse > backup.sql
```

## Troubleshooting

### Traces not appearing in dashboard

1. Verify `ODYSSEUS_LANGFUSE=on` in `.env`
2. Check LangFuse is running: `docker compose --profile observability ps`
3. Check API keys are set: `LANGFUSE_PUBLIC_KEY` and `LANGFUSE_SECRET_KEY`
4. Check Odysseus logs for `[langfuse]` messages

### LangFuse server won't start

- Check PostgreSQL is healthy: `docker compose --profile observability logs langfuse-postgres`
- Ensure ports are free: `lsof -i :3000`

### Performance concerns

When enabled, LangFuse adds ~5-15ms overhead per traced function call. The SDK batches writes and flushes asynchronously, so it should not noticeably impact chat latency.
