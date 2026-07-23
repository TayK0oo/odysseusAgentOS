"""
Memory Provenance → Live Memory Bridge.

Wires our structured memory system (MD + provenance + versioning)
into the existing Odysseus memory pipeline. Replaces ChromaDB-only
with dual-write: ChromaDB for search + MD files for provenance.
"""

from __future__ import annotations

import logging
import os
from typing import Optional

from src.memory_provenance.operations import MemoryOperations
from src.memory_provenance.omission import OmissionFilter

logger = logging.getLogger(__name__)

_memory_bridge: Optional["MemoryBridge"] = None


class MemoryBridge:
    """Bridges SFD v3.0 memory provenance into the live Odysseus system.

    Dual-write approach:
    1. Write to MD files (source of truth, provenance, versioning)
    2. Continue using ChromaDB (search index, backward compat)

    Every fact goes through OmissionFilter before storage.
    """

    def __init__(self, base_path: str = "workspace/memory"):
        self._ops = MemoryOperations(base_path=base_path)
        self._base_path = base_path
        os.makedirs(base_path, exist_ok=True)

    def remember(self, domain: str, fact: str, tag: str = "[stated]") -> bool:
        """Store a fact with provenance. Called during agent runs.

        Returns True if stored, False if omitted.
        """
        if not OmissionFilter.is_safe(fact):
            logger.info("Omitted protected fact: %s...", fact[:50])
            return False

        path = f"topics/{domain}.md"
        entry = f"- {tag} {fact}"

        # Read current state
        current = self._ops.memory_read(path)
        if current.success:
            # Append with version control
            result = self._ops.memory_append(path, entry, current.version_token)
            if result.success:
                logger.debug("Memory appended: %s → %s", entry[:60], path)
                return True
            elif "already exists" in str(result.error):
                return True  # Not an error, just dedup
            elif "Version mismatch" in str(result.error):
                # Retry once with fresh read
                retry = self._ops.memory_read(path)
                if retry.success:
                    result2 = self._ops.memory_append(path, entry, retry.version_token)
                    return result2.success
        else:
            # Create new file
            content = f"---\nname: {domain}\ndescription: Facts about {domain}\nsources: [chat]\n---\n\n{entry}\n"
            result = self._ops.memory_write(path, content, "new")
            return result.success

        return False

    def recall(self, domain: str, query: Optional[str] = None) -> list[str]:
        """Read facts from a memory domain."""
        path = f"topics/{domain}.md"
        result = self._ops.memory_read(path)
        if not result.success:
            return []
        return [line.strip() for line in result.content.split("\n") if line.strip().startswith("- [")]

    def list_domains(self) -> list[str]:
        """List all memory domains."""
        files = self._ops.memory_list("topics")
        return [f["path"].replace("topics/", "").replace(".md", "") for f in files]


def set_memory_bridge(bridge: MemoryBridge) -> None:
    global _memory_bridge
    _memory_bridge = bridge


def get_memory_bridge() -> Optional[MemoryBridge]:
    return _memory_bridge
