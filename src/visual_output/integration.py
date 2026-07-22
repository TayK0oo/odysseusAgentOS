"""
Visual Output integration with the Thought Bus.

Subscribes to the end of the pipeline to route the final
response through the output decision tree.
"""

import logging
from typing import Optional

from src.visual_output.router import OutputRouter, OutputDecision, OutputMode
from src.thought_bus.subscriber import on_phase
from src.thought_bus.events import BusContext

logger = logging.getLogger(__name__)

_router: Optional[OutputRouter] = None
_last_decision: Optional[OutputDecision] = None


def set_output_router(router: OutputRouter) -> None:
    global _router
    _router = router


def get_last_decision() -> Optional[OutputDecision]:
    return _last_decision


@on_phase("MEMORY_OBSERVE", priority=90)  # Runs LAST — after memory, before response
class VisualOutputSubscriber:
    """Routes the final response through the output decision tree.

    Runs at the end of the pipeline (priority 90) to determine
    whether the response stays as text or gets visual treatment.
    """

    def __init__(self):
        pass

    async def on_phase_enter(self, phase: str, ctx: BusContext) -> None:
        pass

    async def on_phase_exit(self, phase: str, ctx: BusContext) -> None:
        global _last_decision, _router

        if _router is None:
            return

        request = ctx.objective or ""
        response = ctx.metadata.get("response_text", "")

        decision = _router.decide(request, response)
        _last_decision = decision

        ctx.enrich(
            output_mode=decision.mode.value,
            output_module=decision.module,
            output_mcp_tool=decision.mcp_tool,
        )
        logger.info("Output routed: %s → %s", decision.mode.value, decision.reason)
