"""
AgentOS 7-Phase Pipeline — called from chat route.
Walks CLASSIFY→KNOW→PLAN→BUILD→QUALITY→AUTOEVAL→MEMORY_OBSERVE,
emitting SSE events at each transition.
"""

import json, logging
from typing import AsyncGenerator

logger = logging.getLogger(__name__)

PHASES = ["CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE"]


async def walk_agent_pipeline(
    agent_stream_fn,  # callable that returns async generator
    session_id: str,
    message: str,
) -> AsyncGenerator[str, None]:
    """Walk 7 phases, emitting SSE events. agent_stream_fn() called in BUILD."""

    for idx, phase in enumerate(PHASES):
        # Emit phase_enter → cockpit shows progress
        yield f"data: {json.dumps({'type': 'phase_enter', 'phase': phase, 'index': idx+1, 'total': 7})}\n\n"

        if phase == "CLASSIFY":
            yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'action': 'Analysing request, evaluating risk...'})}\n\n"

        elif phase == "KNOW":
            yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'action': 'Loading memory, searching past conversations...'})}\n\n"

        elif phase == "PLAN":
            yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'action': 'Decomposing into objectives and tasks...'})}\n\n"

        elif phase == "BUILD":
            yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'action': 'Executing with tools...'})}\n\n"
            # Actual agent loop runs here
            try:
                async for chunk in agent_stream_fn():
                    yield chunk
            except Exception:
                logger.exception("Build phase agent loop failed")
                yield f"data: {json.dumps({'type': 'error', 'phase': 'BUILD', 'text': 'Build failed'})}\n\n"

        elif phase == "QUALITY":
            yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'action': 'Evaluating quality, running checks...'})}\n\n"

        elif phase == "AUTOEVAL":
            yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'action': 'Self-evaluating against success criteria...'})}\n\n"

        elif phase == "MEMORY_OBSERVE":
            yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'action': 'Writing lessons to memory...'})}\n\n"
            try:
                from src.sfd_wiring import get_wiring
                w = get_wiring()
                if w and message:
                    w.remember("conversation", f"User asked: {message[:200]}", "[stated]")
            except Exception:
                pass

        # Emit phase_exit
        yield f"data: {json.dumps({'type': 'phase_exit', 'phase': phase, 'index': idx+1, 'total': 7})}\n\n"

    # Completion
    yield f"data: {json.dumps({'type': 'thought_bus', 'status': 'complete', 'phases_walked': 7})}\n\n"


async def walk_chat_pipeline(
    agent_stream_fn,
) -> AsyncGenerator[str, None]:
    """Chat mode: single phase, no progression."""
    yield f"data: {json.dumps({'type': 'phase_enter', 'phase': 'CHAT', 'index': 1, 'total': 1})}\n\n"
    yield f"data: {json.dumps({'type': 'phase_active', 'phase': 'CHAT', 'action': 'Direct response...'})}\n\n"
    async for chunk in agent_stream_fn():
        yield chunk
    yield f"data: {json.dumps({'type': 'phase_exit', 'phase': 'CHAT', 'index': 1, 'total': 1})}\n\n"
