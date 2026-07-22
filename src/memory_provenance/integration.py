"""
Integration between MemoryOperations and the Thought Bus.

Subscribes to MEMORY_OBSERVE phase to trigger the
Generation → Reflection → Curation cycle.
"""

import logging
from typing import Optional

from src.memory_provenance.operations import MemoryOperations
from src.thought_bus.subscriber import on_phase
from src.thought_bus.events import BusContext

logger = logging.getLogger(__name__)

_memory_ops: Optional[MemoryOperations] = None


def set_memory_operations(ops: MemoryOperations) -> None:
    """Wire up the memory operations reference (called from app startup)."""
    global _memory_ops
    _memory_ops = ops


def get_memory_operations() -> Optional[MemoryOperations]:
    return _memory_ops


@on_phase("MEMORY_OBSERVE", priority=10)
class MemoryObserverSubscriber:
    """Triggers memory reflection and curation at the end of each run."""

    def __init__(self):
        self._active = False

    async def on_phase_enter(self, phase: str, ctx: BusContext) -> None:
        self._active = True
        logger.info("Memory observer active — extracting lessons from run")

    async def on_phase_exit(self, phase: str, ctx: BusContext) -> None:
        self._active = False
        logger.info("Memory observer released")
