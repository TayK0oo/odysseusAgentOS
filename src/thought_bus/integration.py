"""
Bridge between the Thought Bus and the existing agent loop.

This adapter enables the Thought Bus to run alongside or replace
the current stream_agent_loop, emitting phase events that modules
can subscribe to via @on_phase.

Usage in agent_loop.py:
    from src.thought_bus.integration import ThoughtBusBridge

    bridge = ThoughtBusBridge(session_id, objective)
    async for event in bridge.walk():
        if event["type"] == "phase_active":
            # Do the actual agent work for this phase
            ...
        yield event  # Forward SSE to frontend
"""

from __future__ import annotations

import json
import logging
from typing import AsyncGenerator, Optional

from src.thought_bus.bus import ThoughtBus, thought_bus_enabled, Phase
from src.thought_bus.subscriber import SubscriberRegistry

logger = logging.getLogger(__name__)


class ThoughtBusBridge:
    """Convenience wrapper that combines ThoughtBus + SubscriberRegistry.

    Handles initialization, the kill-switch check, and provides
    a simple async generator for integration into existing routes.
    """

    def __init__(
        self,
        session_id: str,
        objective: Optional[str] = None,
        project_id: Optional[str] = None,
    ):
        self._registry = SubscriberRegistry()
        self._bus = ThoughtBus(
            registry=self._registry,
            objective=objective,
            session_id=session_id,
            project_id=project_id,
        )

    @property
    def enabled(self) -> bool:
        return thought_bus_enabled()

    @property
    def bus(self) -> ThoughtBus:
        return self._bus

    @property
    def registry(self) -> SubscriberRegistry:
        return self._registry

    async def walk(
        self,
        stop_after: Optional[str] = None,
    ) -> AsyncGenerator[dict, None]:
        """Walk all phases, yielding SSE events.

        Integration point: call this from your SSE streaming route.
        When you receive a 'phase_active' event, do the actual agent
        work (LLM call, tool execution) for that phase.
        """
        async for event in self._bus.walk(stop_after=stop_after):
            yield event

    def summary(self) -> dict:
        return self._bus.summary()


async def wrap_agent_stream(
    agent_stream,
    session_id: str,
    objective: Optional[str] = None,
    project_id: Optional[str] = None,
) -> AsyncGenerator[str, None]:
    """Wrap an existing agent SSE stream with ThoughtBus phase events.

    Usage in chat routes:
        async for chunk in wrap_agent_stream(
            stream_agent_loop(...),
            session_id=session_id,
            objective=message,
        ):
            yield chunk
            if data := _extract_json(chunk):
                ...
    """
    if not thought_bus_enabled():
        # Pass-through: no bus events, just forward the stream
        async for chunk in agent_stream:
            yield chunk
        return

    bridge = ThoughtBusBridge(
        session_id=session_id,
        objective=objective,
        project_id=project_id,
    )

    # Walk phases — at each 'phase_active', forward the agent stream
    agent_chunks: list = []
    started = False

    async for event in bridge.bus.walk():
        if event["type"] in ("phase_enter", "phase_exit"):
            # Emit phase event as SSE metadata
            yield f"data: {json.dumps({'type': event['type'], 'phase': event.get('phase', ''), 'index': event.get('index', 0), 'total': event.get('total', 7)})}\n\n"

        elif event["type"] == "phase_active":
            if not started:
                started = True
                # Forward the actual agent stream
                async for chunk in agent_stream:
                    yield chunk

        elif event["type"] == "thought_bus" and event.get("status") == "disabled":
            # Bus disabled — just forward
            async for chunk in agent_stream:
                yield chunk

    # Emit completion
    yield f"data: {json.dumps({'type': 'thought_bus', 'status': 'complete', 'phases': 7})}\n\n"
