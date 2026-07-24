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
                if decoded.startswith('{"type":"'):
                    yield f"data: {decoded}\n\n"
                else:
                    yield decoded
        finally:
            if proc.returncode is None:
                proc.kill()
            await proc.wait()
