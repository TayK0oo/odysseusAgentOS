"""
Phase events and context for the Thought Bus.
"""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Any, Optional


class PhaseEvent(str, Enum):
    """Event types emitted by the Thought Bus."""
    ENTER = "phase_enter"
    EXIT = "phase_exit"
    TRANSITION = "phase_transition"


@dataclass
class BusContext:
    """Context snapshot passed to subscribers at each phase event.

    Carries the minimum necessary information for modules to act.
    Subscribers enrich this context; the bus merges enrichments
    before passing to the next phase.
    """
    project_id: Optional[str] = None
    session_id: Optional[str] = None
    objective: Optional[str] = None
    risk_level: Optional[str] = None
    tokens_used: int = 0
    budget_remaining: float = 0.0
    active_agents: list[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    def enrich(self, **kwargs: Any) -> None:
        """Add or update context fields."""
        for key, value in kwargs.items():
            if hasattr(self, key):
                setattr(self, key, value)
            else:
                self.metadata[key] = value
