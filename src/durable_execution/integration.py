"""
Integration between DurableExecution and the Thought Bus.

Subscribes to the BUILD phase and wraps tool executions in
durable workflows. When the Thought Bus is disabled, this
is a no-op.
"""

import logging
from typing import Optional

from src.durable_execution.engine import DurableEngine, durable_exec_enabled
from src.thought_bus.subscriber import on_phase
from src.thought_bus.events import BusContext

logger = logging.getLogger(__name__)

_engine: Optional[DurableEngine] = None


def set_durable_engine(engine: DurableEngine) -> None:
    """Wire up the engine reference (called from app startup)."""
    global _engine
    _engine = engine


def get_durable_engine() -> Optional[DurableEngine]:
    return _engine


@on_phase("BUILD", priority=20)
class DurableExecutionSubscriber:
    """Wraps BUILD-phase tool executions in durable workflows.

    When the Thought Bus walks into BUILD, this subscriber activates.
    Tool calls during BUILD are automatically wrapped in activities
    with retry policies if they meet the durability threshold.
    """

    def __init__(self):
        self._active = False

    async def on_phase_enter(self, phase: str, ctx: BusContext) -> None:
        if not durable_exec_enabled():
            return
        self._active = True
        ctx.enrich(durable_execution=True)
        logger.info("Durable execution active for phase %s", phase)

    async def on_phase_exit(self, phase: str, ctx: BusContext) -> None:
        self._active = False
        logger.info("Durable execution released for phase %s", phase)
