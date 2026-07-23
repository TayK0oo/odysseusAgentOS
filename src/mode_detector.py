"""
Agent/Chat Mode Detector.

Detects whether a user message is a quick chat or requires
the full agent pipeline (7 phases, ThoughtBus, planning, etc.).

Agent triggers: project verbs (build, create, deploy, implement...),
technical requests (refactor, debug, optimize...), explicit agent mode.

The detector is used by the chat route to decide whether to activate
the ThoughtBus (agent mode) or respond directly (chat mode).
"""

from __future__ import annotations

from enum import Enum


class InteractionMode(str, Enum):
    CHAT = "chat"      # Quick response, no phases, no planning
    AGENT = "agent"    # Full pipeline: 7 phases, ThoughtBus, tools


# Keywords that trigger agent mode
_AGENT_TRIGGERS = [
    # Project initiation (English + French)
    "build", "cré", "create", "develop", "développ", "implement", "implément",
    "generate", "génér", "scaffold", "initialize", "set up", "setup",
    # Project work
    "refactor", "debug", "fix", "corrig", "optimize", "optimis", "improve", "amélior",
    "migrate", "upgrade", "deploy", "déploi", "release", "publish", "publier",
    # Architecture/design
    "design", "architect", "plan", "structure", "organize", "organis",
    # Testing
    "test", "valid", "verify", "vérif", "audit", "review",
    # Documentation
    "document", "write docs", "generate documentation", "documenter", "rédig",
    # Analysis
    "analy", "investig", "research", "explore", "explor",
    # Multi-step
    "workflow", "pipeline", "automate", "automatis", "orchestrate", "orchestr",
]


def detect_mode(message: str, agent_switch_on: bool = True) -> InteractionMode:
    """Detect whether the message should use agent or chat mode.

    Args:
        message: The user's message text.
        agent_switch_on: Whether the Agent toggle is ON in the UI.

    Returns:
        InteractionMode.AGENT if the message triggers the full pipeline,
        InteractionMode.CHAT otherwise.
    """
    if not agent_switch_on:
        return InteractionMode.CHAT

    message_lower = message.lower().strip()

    # Very short messages are usually chat
    if len(message_lower.split()) <= 2 and not any(t in message_lower for t in ["do", "make", "run"]):
        # But "build X", "create Y", "deploy Z" are agent mode even if short
        for trigger in _AGENT_TRIGGERS:
            if message_lower.startswith(trigger):
                return InteractionMode.AGENT
        return InteractionMode.CHAT

    # Check for agent triggers
    for trigger in _AGENT_TRIGGERS:
        if trigger in message_lower:
            return InteractionMode.AGENT

    return InteractionMode.CHAT


def is_agent_mode(message: str, agent_switch_on: bool = True) -> bool:
    """Convenience: True if agent mode should be used."""
    return detect_mode(message, agent_switch_on) == InteractionMode.AGENT
