# Acontext Memory Pipeline

Phase 8 of odysseusAgentOS introduces a full distillation pipeline that turns
raw agent traces into durable, queryable knowledge stored both in OpenCode
skills and in an Obsidian vault.

---

## Overview

```
Agent session
    │
    │ (session ends)
    ▼
agent_loop.py hook
    │  POST /api/sessions/end  →  Acontext  (port 8029)
    ▼
┌─────────────────────────────────────────┐
│             Acontext service            │
│                                         │
│  Experience Agent                       │
│    • reads JSONL trace for the session  │
│    • extracts learnings, patterns,      │
│      errors, and breakthroughs          │
│                                         │
│  Skill Agent                            │
│    • distills learnings → SKILL.md fmt  │
│    • writes to .opencode/skills/        │
│    • writes to Obsidian vault           │
└────────────┬────────────────────────────┘
             │
    ┌────────┴────────┐
    ▼                 ▼
.opencode/skills/    Obsidian vault
(native OpenCode)    agentos/ folder
                     (via obsidian_mcp)
```

---

## Components

### 1. `agent_loop.py` — session hook

At the end of every agent session, `agent_loop.py` sends a POST request to
Acontext:

```
POST http://acontext:8029/api/sessions/end
{
  "session_id": "<uuid>",
  "trace_path": "/data/traces/<session_id>.jsonl"
}
```

Controlled by env vars:

| Variable           | Default                    | Description                        |
|--------------------|----------------------------|------------------------------------|
| `ACONTEXT_URL`     | `http://acontext:8029`     | Acontext service URL               |
| `ACONTEXT_ENABLED` | `false`                    | Set `true` to activate the hook    |

---

### 2. Acontext Experience Agent

Reads the JSONL trace file and extracts:

- **Learnings** — things the agent discovered or confirmed
- **Patterns** — recurring strategies that worked well
- **Errors** — mistakes made and their root causes
- **Breakthroughs** — novel solutions worth preserving

Output is stored internally before being handed to the Skill Agent.

---

### 3. Acontext Skill Agent

Distills the extracted knowledge into Markdown using the OpenCode SKILL.md
format:

```markdown
# Skill: <topic>

## Context
When to apply this skill.

## Steps
1. …

## Examples
…

## Anti-patterns
…
```

The Skill Agent writes to two destinations simultaneously:

#### a) `.opencode/skills/` (native OpenCode format)

Skills here are auto-loaded by OpenCode on next startup and are available
as slash commands inside the IDE.

```
.opencode/skills/
    agentos/
        learnings/
            2026-06-27-agentos.md
        patterns/
            routing.md
        ...
```

#### b) Obsidian vault — `agentos/` folder (via `obsidian_mcp`)

`mcp_servers/obsidian_mcp.py` exposes four tools used by the Skill Agent
to write into the vault:

| Tool            | Description                                      |
|-----------------|--------------------------------------------------|
| `list_notes`    | List markdown files in a vault folder            |
| `get_note`      | Read a specific note by relative path            |
| `create_note`   | Create or overwrite a note (creates parent dirs) |
| `search_notes`  | Full-text search across vault notes              |

Controlled by env vars:

| Variable               | Default           | Description                          |
|------------------------|-------------------|--------------------------------------|
| `OBSIDIAN_VAULT_PATH`  | `/obsidian-vault` | Mount point of the Obsidian vault    |
| `OBSIDIAN_MCP_ENABLED` | `false`           | Set `true` to enable vault writes    |

---

### 4. ChromaDB RAG (separate concern)

ChromaDB is **not** part of the agent memory pipeline. It handles
**document indexing** for retrieval-augmented generation:

- PDFs, web pages, email attachments → chunked → embedded → stored in ChromaDB
- Queried at inference time via `/api/rag/*` routes

Agent learnings and skills do **not** go into ChromaDB. They live in
`.opencode/skills/` (structured Markdown) and the Obsidian vault.

---

## Data-flow diagram (detailed)

```
JSONL trace
  └─► Acontext Experience Agent
        ├─► learnings[]
        ├─► patterns[]
        ├─► errors[]
        └─► breakthroughs[]
              └─► Acontext Skill Agent
                    ├─► .opencode/skills/agentos/<date>-<topic>.md
                    └─► obsidian_mcp.create_note("agentos/<date>-<topic>.md", …)
                              └─► OBSIDIAN_VAULT_PATH/agentos/<date>-<topic>.md
```

---

## Docker Compose setup

The `acontext` service is declared in `docker-compose.yml` on port 8029.
To enable the full pipeline:

```yaml
# docker-compose.yml (excerpt)
acontext:
  image: acontext:latest
  ports:
    - "8029:8029"
  volumes:
    - ./data/traces:/data/traces
    - obsidian-vault:/obsidian-vault
  environment:
    OBSIDIAN_VAULT_PATH: /obsidian-vault
```

Set in your `.env`:

```env
ACONTEXT_ENABLED=true
OBSIDIAN_MCP_ENABLED=true
OBSIDIAN_VAULT_PATH=/obsidian-vault
```

---

## Phase roadmap

| Phase | Feature                                        | Status   |
|-------|------------------------------------------------|----------|
| 8     | Acontext hook, obsidian_mcp, memory pipeline   | Done     |
| 9     | Knowledge routes (`/api/knowledge/obsidian/*`) | Done     |
| 10    | Discord / Telegram channel gateway             | Planned  |
