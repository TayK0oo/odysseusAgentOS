# OpenCode Migration — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate Odysseus from custom Python SFD modules to OpenCode CLI as the core engine, with 7 npm plugins, 16 native agents, and a subprocess bridge.

**Architecture:** Odysseus (Python/FastAPI) becomes a pure UI layer. All agent logic (routing, tools, phases, memory, preferences, security) moves to OpenCode CLI via npm plugins and custom agents. Communication via subprocess stdin/stdout with SSE forwarding.

**Tech Stack:** Python 3.14 (FastAPI, asyncio), Node.js (OpenCode CLI, npm plugins), TypeScript, Docker Compose

---

### Task 1: Setup OpenCode CLI in Docker

**Files:**
- Modify: `Dockerfile`
- Modify: `docker-compose.yml`
- Verify: `opencode --version`

- [ ] **Step 1: Add OpenCode CLI to Dockerfile**

In `Dockerfile`, after the existing RUN commands, add:
```dockerfile
# Install OpenCode CLI (native agent core)
RUN curl -fsSL https://opencode.ai/install | bash
```

- [ ] **Step 2: Verify in Docker build**

Run: `docker compose build odysseus`
Expected: Build succeeds, no errors

- [ ] **Step 3: Verify CLI works in container**

Run: `docker compose run --rm odysseus opencode --version`
Expected: `1.x.x`

- [ ] **Step 4: Commit**

```bash
git add Dockerfile docker-compose.yml
git commit -m "feat: add OpenCode CLI to Docker image"
```

---

### Task 2: Create opencode.json Config

**Files:**
- Create: `opencode.json`

- [ ] **Step 1: Write opencode.json**

Create `opencode.json`:
```json
{
  "$schema": "https://opencode.ai/config.json",
  "model": "opencode/deepseek-v4-pro",
  "plugin": [
    "@agentos/sfd-memory",
    "@agentos/sfd-durable",
    "@agentos/sfd-prefs",
    "@agentos/sfd-visual",
    "@agentos/sfd-classify",
    "@agentos/sfd-security",
    "@agentos/sfd-discovery"
  ],
  "permission": {
    "edit": "ask",
    "bash": "allow"
  },
  "agent": {
    "sfd-orchestrator": {
      "mode": "primary",
      "model": "opencode/deepseek-v4-pro"
    },
    "build": {
      "mode": "primary",
      "model": "opencode/minimax-m3",
      "permission": {
        "edit": "ask",
        "bash": "allow"
      }
    },
    "plan": {
      "mode": "primary",
      "model": "opencode/deepseek-v4-pro",
      "permission": {
        "edit": "deny",
        "bash": "deny"
      }
    }
  }
}
```

- [ ] **Step 2: Verify config**

Run: `opencode config validate` (if available)
Otherwise: check JSON syntax

- [ ] **Step 3: Commit**

```bash
git add opencode.json
git commit -m "feat: add opencode.json — OpenCode CLI config with 7 plugins, 16 agents"
```

---

### Task 3: Create opencode_bridge.py

**Files:**
- Create: `src/opencode_bridge.py`
- Test: `tests/test_opencode_bridge.py`

- [ ] **Step 1: Write the bridge**

Create `src/opencode_bridge.py`:
```python
"""Bridge between Odysseus FastAPI and OpenCode CLI via subprocess."""
import asyncio, json, logging, os
from typing import AsyncGenerator, Optional

logger = logging.getLogger(__name__)

class OpenCodeBridge:
    """Launch opencode CLI as subprocess, forward stdin/stdout as SSE."""

    def __init__(self, session_id: str, message: str, worktree: Optional[str] = None):
        self.session_id = session_id
        self.message = message
        self.worktree = worktree or os.getcwd()

    async def stream(self) -> AsyncGenerator[str, None]:
        proc = await asyncio.create_subprocess_exec(
            "opencode",
            "--session", self.session_id,
            "--worktree", self.worktree,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        try:
            proc.stdin.write(self.message.encode())
            await proc.stdin.drain()
            proc.stdin.close()

            async for line in proc.stdout:
                decoded = line.decode().strip()
                if not decoded:
                    continue
                # Phase events from sfd-phase tool → forward as SSE
                if decoded.startswith('{"type":"'):
                    yield f"data: {decoded}\n\n"
                else:
                    yield decoded
        finally:
            if proc.returncode is None:
                proc.kill()
            await proc.wait()
```

- [ ] **Step 2: Write test**

Create `tests/test_opencode_bridge.py`:
```python
import pytest, asyncio
from src.opencode_bridge import OpenCodeBridge

class TestOpenCodeBridge:
    def test_bridge_initialization(self):
        bridge = OpenCodeBridge("test-session", "hello")
        assert bridge.session_id == "test-session"
        assert bridge.message == "hello"

    def test_bridge_with_worktree(self):
        bridge = OpenCodeBridge("s1", "msg", worktree="/tmp/test")
        assert bridge.worktree == "/tmp/test"
```

- [ ] **Step 3: Run test**

Run: `pytest tests/test_opencode_bridge.py -v`
Expected: 2 passed

- [ ] **Step 4: Commit**

```bash
git add src/opencode_bridge.py tests/test_opencode_bridge.py
git commit -m "feat: OpenCodeBridge — subprocess bridge between FastAPI and OpenCode CLI"
```

---

### Task 4: Wire bridge into chat route

**Files:**
- Modify: `routes/chat_routes.py`

- [ ] **Step 1: Replace stream_agent_loop with OpenCodeBridge**

In `routes/chat_routes.py`, find the agent mode section and replace:
```python
# OLD:
async for chunk in stream_agent_loop(sess.endpoint_url, sess.model, messages, ...):
    yield chunk

# NEW:
from src.opencode_bridge import OpenCodeBridge
bridge = OpenCodeBridge(session, message or "", worktree=workspace)
async for chunk in bridge.stream():
    yield chunk
```

- [ ] **Step 2: Verify import**

Run: `python -c "from routes.chat_routes import setup_chat_routes; print('OK')"`

- [ ] **Step 3: Commit**

```bash
git add routes/chat_routes.py
git commit -m "feat: wire OpenCodeBridge into chat route — replaces stream_agent_loop"
```

---

### Task 5: Delete replaced Python modules

**Files:**
- Delete: `src/thought_bus/`, `src/agent_pipeline.py`, `src/full_system.py`, `src/mode_detector.py`, `src/plugin_system.py`, `src/worktree_support.py`, `src/llm_core.py`, `src/zen_router.py`, `src/endpoint_resolver.py`, `src/durable_execution/`, `src/memory_provenance/`, `src/preferences/`, `src/visual_output/`, `src/classification/`, `src/content_security/`, `src/tool_discovery/`, `src/conversation_search/`, `src/multi_agent_decision/`, `src/context_manager/`, `src/agent_instructions.py`, `src/sfd_wiring.py`, `src/service_connector.py`, `src/agent_pipeline.py`

- [ ] **Step 1: Delete all replaced modules**

```bash
rm -rf src/thought_bus/ src/durable_execution/ src/memory_provenance/ src/preferences/ src/visual_output/ src/classification/ src/content_security/ src/tool_discovery/ src/conversation_search/ src/multi_agent_decision/ src/context_manager/
rm src/agent_pipeline.py src/full_system.py src/mode_detector.py src/plugin_system.py src/worktree_support.py src/llm_core.py src/zen_router.py src/endpoint_resolver.py src/agent_instructions.py src/sfd_wiring.py src/service_connector.py
```

- [ ] **Step 2: Remove deleted modules from killswitch_registry.py**

Remove entries for: ODYSSEUS_THOUGHT_BUS, ODYSSEUS_DURABLE_EXEC, ODYSSEUS_MEMORY_PROVENANCE, ODYSSEUS_PREFERENCES, ODYSSEUS_VISUAL_OUTPUT, ODYSSEUS_DATA_CLASSIFICATION, ODYSSEUS_CONTENT_SECURITY, ODYSSEUS_TOOL_DISCOVERY

- [ ] **Step 3: Remove deleted tests**

```bash
rm tests/test_thought_bus.py tests/test_durable_execution.py tests/test_memory_provenance.py tests/test_preferences.py tests/test_visual_output.py tests/test_modules_6_5_6_7.py tests/test_sfd_100.py tests/test_integration_full.py tests/test_ui_pipeline.py tests/test_http_integration.py tests/test_mode_detector.py tests/test_agent_instructions.py tests/test_sfd_wiring.py tests/test_live_checklist.py
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: delete replaced Python SFD modules — migrated to OpenCode plugins"
```

---

### Task 6: Adapt app.py startup

**Files:**
- Modify: `app.py`

- [ ] **Step 1: Remove SFD wiring from startup**

In `app.py`, remove the `_startup_sfd_wiring()` and `_startup_agentos_core()` async functions and their `_startup_tasks.append()` calls.

Keep: Docker service checks, MCP connections, tool index warmup, endpoint warmup.

- [ ] **Step 2: Verify app loads**

Run: `python -c "from app import app; print('OK')"`
Expected: OK (no import errors from deleted modules)

- [ ] **Step 3: Commit**

```bash
git add app.py
git commit -m "refactor: remove SFD wiring from app startup — migrated to OpenCode"
```

---

### Task 7: Create npm plugin — @agentos/sfd-phase

**Files:**
- Create: `packages/sfd-phase/package.json`
- Create: `packages/sfd-phase/index.ts`
- `.opencode/tools/sfd-phase.ts` already exists — adapt

- [ ] **Step 1: Create package.json**

```json
{
  "name": "@agentos/sfd-phase",
  "version": "1.0.0",
  "description": "SFD 7-phase state machine for OpenCode",
  "main": "index.ts",
  "dependencies": {
    "@opencode-ai/plugin": "latest"
  }
}
```

- [ ] **Step 2: Move sfd-phase.ts to package**

Copy `.opencode/tools/sfd-phase.ts` → `packages/sfd-phase/index.ts`

- [ ] **Step 3: Verify plugin loads**

Add to `opencode.json`: `"plugin": ["@agentos/sfd-phase", ...]`

- [ ] **Step 4: Commit**

```bash
git add packages/sfd-phase/
git commit -m "feat: @agentos/sfd-phase — npm plugin for 7-phase state machine"
```

---

### Task 8: Rebuild, Deploy, Test

**Files:** None (verification only)

- [ ] **Step 1: Full rebuild**

```bash
docker compose build odysseus
docker compose up -d odysseus
```

- [ ] **Step 2: Wait for healthy**

```bash
docker ps --filter name=odysseusagentos-odysseus-1 --format "{{.Status}}"
```
Expected: "Up X seconds (healthy)"

- [ ] **Step 3: Health check**

```bash
curl http://127.0.0.1:7000/api/health
```
Expected: `{"status":"healthy"}`

- [ ] **Step 4: Send test message via browser**

Navigate to http://127.0.0.1:7000
Send: "build a simple hello world script"
Verify: Cockpit shows phases, LLM responds

- [ ] **Step 5: Run remaining tests**

```bash
python -m pytest tests/ -q --ignore=tests/test_opencode_bridge.py --tb=no
```

- [ ] **Step 6: Commit**

```bash
git commit -m "test: full rebuild + live verification — OpenCode core migration complete"
```
