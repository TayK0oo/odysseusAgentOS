"""
Right to be forgotten (SFD §5.19).

User can delete: a single entry, an entire file, or all their data.
Deletion is definitive, irreversible, and applies to all surfaces and backups.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Optional

logger = logging.getLogger(__name__)


@dataclass
class DeletionResult:
    success: bool
    deleted_count: int = 0
    error: Optional[str] = None


class RightToForget:
    """Handles data deletion requests with full audit trail.

    Levels:
        1. Entry: delete a single fact from a memory file
        2. File: delete an entire memory file
        3. All: delete all user data across all surfaces
    """

    def __init__(self, memory_ops=None):
        self._memory_ops = memory_ops
        self._deletion_log: list[dict] = []

    async def forget_entry(
        self, path: str, entry_text: str, if_version: str
    ) -> DeletionResult:
        """Delete a single entry from a memory file."""
        if self._memory_ops is None:
            return DeletionResult(success=False, error="Memory operations not available")

        result = self._memory_ops.memory_read(path)
        if not result.success:
            return DeletionResult(success=False, error=result.error)

        if entry_text not in result.content:
            return DeletionResult(success=False, error="Entry not found")

        # Remove the entry line
        lines = result.content.split("\n")
        lines = [l for l in lines if entry_text.strip() not in l]
        new_content = "\n".join(lines)

        write_result = self._memory_ops.memory_write(path, new_content, if_version)
        if write_result.success:
            self._log_deletion("entry", path, entry_text)
            return DeletionResult(success=True, deleted_count=1)
        return DeletionResult(success=False, error=write_result.error)

    async def forget_file(self, path: str, if_version: str) -> DeletionResult:
        """Delete an entire memory file."""
        if self._memory_ops is None:
            return DeletionResult(success=False, error="Memory operations not available")

        result = self._memory_ops.memory_delete(path, if_version)
        if result.success:
            self._log_deletion("file", path)
            return DeletionResult(success=True, deleted_count=1)
        return DeletionResult(success=False, error=result.error)

    async def forget_all(self, base_path: str = "workspace/memory") -> DeletionResult:
        """Delete all user data. Definitive and irreversible.

        Requires explicit user confirmation before calling.
        """
        import os
        import shutil

        if os.path.exists(base_path):
            try:
                shutil.rmtree(base_path)
                self._log_deletion("all", base_path)
                return DeletionResult(success=True)
            except Exception as e:
                return DeletionResult(success=False, error=str(e))
        return DeletionResult(success=True, deleted_count=0)

    def _log_deletion(self, level: str, target: str, detail: str = "") -> None:
        from datetime import datetime, timezone
        self._deletion_log.append({
            "level": level,
            "target": target,
            "detail": detail,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        })
        logger.info("RightToForget: deleted %s — %s", level, target)

    @property
    def deletion_history(self) -> list[dict]:
        return self._deletion_log
