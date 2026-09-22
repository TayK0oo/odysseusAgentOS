"""Fast launcher — skips slow MCP connections for quick UI testing."""

import asyncio
import os

# Skip slow services
os.environ["CHROMADB_HOST"] = ""
os.environ["ODYSSEUS_THOUGHT_BUS"] = "on"
os.environ["ODYSSEUS_DURABLE_EXEC"] = "on"
os.environ["AUTH_ENABLED"] = "false"
os.environ["LOCALHOST_BYPASS"] = "true"
os.environ["MCP_CONNECT_TIMEOUT"] = "2"

# Patch MCP manager to skip slow connections
import src.mcp_manager as mcp_mgr

_orig_connect = mcp_mgr.MCPManager.connect_all_enabled


async def _fast_connect(self):
    try:
        await asyncio.wait_for(_orig_connect(self), timeout=3)
    except:
        pass


mcp_mgr.MCPManager.connect_all_enabled = _fast_connect

_orig_external = mcp_mgr.MCPManager.connect_external_enabled


async def _skip_external(self):
    pass


mcp_mgr.MCPManager.connect_external_enabled = _skip_external

# Now start
if __name__ == "__main__":
    import uvicorn

    print("Fast launcher — starting on http://127.0.0.1:7000")
    uvicorn.run("app:app", host="127.0.0.1", port=7000, log_level="warning")
