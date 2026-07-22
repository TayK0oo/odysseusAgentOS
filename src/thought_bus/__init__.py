"""
Thought Bus — Event-driven phase orchestration for Agent OS.

The Thought Bus emits events at each phase transition in the agent's
cheminement de pensée (7 canonical phases). Modules subscribe to phases
via the @on_phase decorator and receive enter/exit events.

Architecture:
    thought_bus
    ├── bus.py         — ThoughtBus: phase walker + event emitter
    ├── subscriber.py  — @on_phase decorator, SubscriberRegistry
    └── events.py      — PhaseEvent, BusContext dataclasses
"""

from src.thought_bus.bus import ThoughtBus, Phase, CANONICAL_SEQUENCE
from src.thought_bus.subscriber import on_phase, SubscriberRegistry
from src.thought_bus.events import PhaseEvent, BusContext

__all__ = [
    "ThoughtBus",
    "Phase",
    "CANONICAL_SEQUENCE",
    "on_phase",
    "SubscriberRegistry",
    "PhaseEvent",
    "BusContext",
]
