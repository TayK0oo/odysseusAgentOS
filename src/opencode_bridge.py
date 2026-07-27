"""Odysseus ↔ OpenCode Bridge via HTTP API (ZenRouter).

Uses the existing OPENCODE_API_KEY + ZenRouter that already works.
Simplified: no subprocess, no CLI dependency. Just HTTP.
"""

import asyncio, json, logging
from typing import AsyncGenerator, Optional

logger = logging.getLogger(__name__)

class OpenCodeBridge:
    """Bridge that delegates to the existing stream_agent_loop.

    Since OpenCode CLI is a TUI (not a server), we use the HTTP API
    via ZenRouter (already proven working). The bridge simplifies
    the chat route by providing a clean interface.
    """

    def __init__(self, session_id: str, message: str, worktree: Optional[str] = None):
        self.session_id = session_id
        self.message = message
        self.worktree = worktree

    async def stream(self, agent_stream) -> AsyncGenerator[str, None]:
        """Forward the existing agent stream (ZenRouter → OpenCode API)."""
        async for chunk in agent_stream:
            yield chunk
