"""
Thought Bus — Event-driven phase walker for Agent OS.

The ThoughtBus walks the 7 canonical phases (CLASSIFY → KNOW → PLAN →
BUILD → QUALITY → AUTOEVAL → MEMORY_OBSERVE), emitting enter/exit events
at each transition. Modules subscribe via @on_phase and receive context.

Integrated with the existing kill-switch system: gated behind
ODYSSEUS_THOUGHT_BUS (default ON for core stable modules).
"""

from __future__ import annotations
import logging
import os
from enum import Enum
from typing import AsyncGenerator, Optional

from src.thought_bus.events import BusContext, PhaseEvent
from src.thought_bus.subscriber import SubscriberRegistry

logger = logging.getLogger(__name__)

_THOUGHT_BUS_ENV = "ODYSSEUS_THOUGHT_BUS"


# Duplicated from src/orchestrator/phases.py to avoid triggering
# orchestrator/__init__.py which imports broken langgraph_loop.py.
# These 7 phases are the canonical Agent OS specification — stable.
class Phase(str, Enum):
    CLASSIFY = "CLASSIFY"
    KNOW = "KNOW"
    PLAN = "PLAN"
    BUILD = "BUILD"
    QUALITY = "QUALITY"
    AUTOEVAL = "AUTOEVAL"
    MEMORY_OBSERVE = "MEMORY_OBSERVE"


CANONICAL_SEQUENCE: list[Phase] = [
    Phase.CLASSIFY,
    Phase.KNOW,
    Phase.PLAN,
    Phase.BUILD,
    Phase.QUALITY,
    Phase.AUTOEVAL,
    Phase.MEMORY_OBSERVE,
]


def thought_bus_enabled() -> bool:
    """Check if the Thought Bus kill-switch is ON.

    Core stable module — defaults to ON.
    """
    val = os.getenv(_THOUGHT_BUS_ENV, "on").strip().lower()
    return val in ("on", "1", "true", "yes")


class ThoughtBus:
    """Event-driven phase walker.

    Walks the 7 canonical phases, notifies subscribers, and yields
    SSE events for the frontend to display progress.

    Usage:
        bus = ThoughtBus(registry=SubscriberRegistry())
        async for event in bus.walk(session_id="abc", objective="Build X"):
            yield event  # SSE to frontend

        # Or manually:
        await bus.enter_phase("BUILD", ctx)
        # ... work ...
        await bus.exit_phase("BUILD", ctx)
    """

    PHASES = CANONICAL_SEQUENCE  # [CLASSIFY, KNOW, PLAN, BUILD, QUALITY, AUTOEVAL, MEMORY_OBSERVE]

    def __init__(
        self,
        registry: SubscriberRegistry,
        objective: Optional[str] = None,
        session_id: Optional[str] = None,
        project_id: Optional[str] = None,
    ):
        self._registry = registry
        self._index: int = -1
        self._ctx = BusContext(
            objective=objective,
            session_id=session_id,
            project_id=project_id,
        )

    @property
    def current_phase(self) -> Optional[Phase]:
        """The current phase, or None if not started."""
        if 0 <= self._index < len(self.PHASES):
            return self.PHASES[self._index]
        return None

    @property
    def context(self) -> BusContext:
        """The current bus context (mutable — subscribers enrich it)."""
        return self._ctx

    @property
    def is_complete(self) -> bool:
        """True when all phases have been walked."""
        return self._index >= len(self.PHASES)

    async def _notify(self, event: PhaseEvent, phase_name: str) -> None:
        """Notify all subscribers for a phase."""
        subscribers = self._registry.get(phase_name)
        if not subscribers:
            return

        for sub in subscribers:
            try:
                if event == PhaseEvent.ENTER:
                    await sub.on_enter(phase_name, self._ctx)
                elif event == PhaseEvent.EXIT:
                    await sub.on_exit(phase_name, self._ctx)
            except Exception:
                logger.exception(
                    "Subscriber %s failed on %s:%s",
                    sub.name, phase_name, event.value,
                )

    async def enter_phase(self, phase_name: str) -> None:
        """Emit phase_enter event to subscribers."""
        logger.info("Thought Bus (pid=%d) → ENTER %s", os.getpid(), phase_name)
        await self._notify(PhaseEvent.ENTER, phase_name)

    async def exit_phase(self, phase_name: str) -> None:
        """Emit phase_exit event to subscribers."""
        logger.info("Thought Bus (pid=%d) ← EXIT  %s", os.getpid(), phase_name)
        await self._notify(PhaseEvent.EXIT, phase_name)

    def advance(self) -> Optional[Phase]:
        """Move to the next phase. Returns the new phase, or None if done."""
        if self._index >= 0:
            current_name = self.PHASES[self._index].value
            logger.debug("Thought Bus: advancing from %s", current_name)
        self._index += 1
        if self._index >= len(self.PHASES):
            return None
        return self.PHASES[self._index]

    async def walk(
        self,
        stop_after: Optional[str] = None,
    ) -> AsyncGenerator[dict, None]:
        """Walk all phases, yielding SSE events.

        Args:
            stop_after: Optional phase name to stop after (inclusive).
                        Useful for partial walks.

        Yields:
            Dicts with keys: type, phase, index, total, context
        """
        if not thought_bus_enabled():
            logger.info("Thought Bus is OFF (ODYSSEUS_THOUGHT_BUS)")
            yield {
                "type": "thought_bus",
                "status": "disabled",
            }
            return

        total = len(self.PHASES)

        while True:
            phase = self.advance()
            if phase is None:
                break

            phase_name = phase.value
            await self.enter_phase(phase_name)

            yield {
                "type": "phase_enter",
                "phase": phase_name,
                "index": self._index + 1,
                "total": total,
                "context": {
                    "objective": self._ctx.objective,
                    "risk_level": self._ctx.risk_level,
                    "tokens_used": self._ctx.tokens_used,
                },
            }

            # Yield a "phase_active" event — the caller does the actual
            # work between ENTER and EXIT
            yield {
                "type": "phase_active",
                "phase": phase_name,
                "index": self._index + 1,
                "total": total,
            }

            await self.exit_phase(phase_name)

            yield {
                "type": "phase_exit",
                "phase": phase_name,
                "index": self._index + 1,
                "total": total,
            }

            if stop_after and phase_name.upper() == stop_after.upper():
                break

        yield {
            "type": "thought_bus",
            "status": "complete",
            "phases_walked": self._index + 1,
            "total": total,
        }

    def sub_count(self, phase: str) -> int:
        """Number of subscribers for a phase (for debugging)."""
        return len(self._registry.get(phase))

    def summary(self) -> dict:
        """Return a debug summary of the bus state."""
        return {
            "current_phase": self.current_phase.value if self.current_phase else None,
            "index": self._index,
            "is_complete": self.is_complete,
            "subscribers": self._registry.list_all(),
            "context": {
                k: v for k, v in self._ctx.__dict__.items()
                if not k.startswith("_")
            },
        }
