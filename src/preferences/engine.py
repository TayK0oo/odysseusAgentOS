"""
Preferences Engine + Thought Bus integration.

The PreferenceEngine loads, caches, and resolves preferences.
It subscribes to the beginning of the Thought Bus pipeline
to inject preferences before CLASSIFY.
"""

import logging
from typing import Optional

from src.preferences.resolver import Preference, PreferenceEngine
from src.preferences.guardrails import BehavioralGuardrail
from src.thought_bus.subscriber import on_phase
from src.thought_bus.events import BusContext

logger = logging.getLogger(__name__)

_engine: Optional[PreferenceEngine] = None


def set_preference_engine(engine: PreferenceEngine) -> None:
    global _engine
    _engine = engine


def get_preference_engine() -> Optional[PreferenceEngine]:
    return _engine


@on_phase("CLASSIFY", priority=5)  # Runs BEFORE classification
class PreferencesSubscriber:
    """Injects resolved preferences into the bus context before CLASSIFY.

    Priority 5 ensures this runs before any other subscriber,
    so preferences are available for the entire pipeline.
    """

    def __init__(self):
        self._resolved: dict[str, str] = {}

    async def on_phase_enter(self, phase: str, ctx: BusContext) -> None:
        if _engine is None:
            return

        # Load stored preferences from memory if available
        stored: list[Preference] = []
        if _engine._memory_ops:
            result = _engine._memory_ops.memory_read("preferences.md")
            if result.success:
                stored = _engine.load(result.content)

        # Filter guardrail violations (treat as absent)
        stored = BehavioralGuardrail.filter_preferences(stored)

        # Resolve with the bus context objective as the request instruction
        self._resolved = _engine.resolve(
            stored=stored,
            request_instruction=ctx.objective,
        )

        # Inject into bus context
        ctx.enrich(preferences=self._resolved)
        logger.debug("Preferences resolved: %s", self._resolved)

    async def on_phase_exit(self, phase: str, ctx: BusContext) -> None:
        pass
