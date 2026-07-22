"""
Subscriber registry and @on_phase decorator for the Thought Bus.

Modules declare their phase subscriptions via decorators.
The registry maintains priority-ordered subscriber lists per phase.
"""

from __future__ import annotations

import inspect
import logging
from collections import defaultdict
from typing import Any, Callable, Optional

from src.thought_bus.events import BusContext, PhaseEvent

logger = logging.getLogger(__name__)


class PhaseSubscriber:
    """Wraps a subscriber class with its phase bindings and priority."""

    def __init__(
        self,
        instance: Any,
        phases: list[str],
        priority: int = 50,
        name: Optional[str] = None,
    ):
        self.instance = instance
        self.phases = phases
        self.priority = priority
        self.name = name or type(instance).__name__

    async def on_enter(self, phase: str, ctx: BusContext) -> None:
        """Called when the bus enters a phase."""
        handler = getattr(self.instance, "on_phase_enter", None)
        if handler and inspect.iscoroutinefunction(handler):
            await handler(phase, ctx)
        elif handler:
            handler(phase, ctx)

    async def on_exit(self, phase: str, ctx: BusContext) -> None:
        """Called when the bus exits a phase."""
        handler = getattr(self.instance, "on_phase_exit", None)
        if handler and inspect.iscoroutinefunction(handler):
            await handler(phase, ctx)
        elif handler:
            handler(phase, ctx)


def on_phase(
    *phases: str,
    priority: int = 50,
    enabled: bool = True,
):
    """Decorator: register a class as a Thought Bus subscriber.

    Usage:
        @on_phase("BUILD", "QUALITY", priority=30)
        class DurableExecutionSubscriber:
            async def on_phase_enter(self, phase, ctx):
                ...

            async def on_phase_exit(self, phase, ctx):
                ...

    The decorated class is automatically registered in the global
    SubscriberRegistry on import. Set `enabled=False` to skip registration
    (useful for kill-switch gated modules).

    Args:
        *phases: One or more phase names to subscribe to.
        priority: Lower = earlier execution. Default 50.
        enabled: If False, the decorator is a no-op (module gated OFF).
    """
    def decorator(cls):
        if not enabled:
            logger.debug("Subscriber %s skipped (enabled=False)", cls.__name__)
            return cls

        original_init = cls.__init__

        def patched_init(self, *args, **kwargs):
            original_init(self, *args, **kwargs)

            # Catch the case where SubscriberRegistry was set after import
            if SubscriberRegistry._instance is None:
                SubscriberRegistry._pending.append((self, phases, priority))
                return

            subscriber = PhaseSubscriber(
                instance=self,
                phases=list(phases),
                priority=priority,
                name=cls.__name__,
            )
            SubscriberRegistry._instance.register(subscriber)

        cls.__init__ = patched_init
        return cls

    if len(phases) == 1 and callable(phases[0]):
        # Called without parentheses: @on_phase
        cls = phases[0]
        return decorator(cls)

    return decorator


class SubscriberRegistry:
    """Global registry of phase subscribers, ordered by priority."""

    _instance: Optional["SubscriberRegistry"] = None
    _pending: list[tuple[Any, list[str], int]] = []

    def __init__(self):
        self._subscribers: dict[str, list[PhaseSubscriber]] = defaultdict(list)
        SubscriberRegistry._instance = self

        # Register any subscribers that were created before the registry
        for instance, phases, priority in SubscriberRegistry._pending:
            subscriber = PhaseSubscriber(
                instance=instance,
                phases=phases,
                priority=priority,
                name=type(instance).__name__,
            )
            self.register(subscriber)
        SubscriberRegistry._pending.clear()

    def register(self, subscriber: PhaseSubscriber) -> None:
        """Register a subscriber for its declared phases."""
        for phase in subscriber.phases:
            self._subscribers[phase.upper()].append(subscriber)
            self._subscribers[phase.upper()].sort(key=lambda s: s.priority)
        logger.info(
            "Registered subscriber %s for phases %s (priority %d)",
            subscriber.name, subscriber.phases, subscriber.priority,
        )

    def get(self, phase: str) -> list[PhaseSubscriber]:
        """Get subscribers for a phase, ordered by priority."""
        return self._subscribers.get(phase.upper(), [])

    def list_all(self) -> dict[str, list[str]]:
        """Return {phase: [subscriber_names]} for debugging."""
        return {
            phase: [s.name for s in subs]
            for phase, subs in sorted(self._subscribers.items())
        }

    @classmethod
    def instance(cls) -> Optional["SubscriberRegistry"]:
        return cls._instance
