"""
Retry policies for durable activities.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class BackoffStrategy(str, Enum):
    CONSTANT = "constant"
    LINEAR = "linear"
    EXPONENTIAL = "exponential"


@dataclass
class RetryPolicy:
    """Retry configuration for a durable activity.

    Defaults match SFD recommendations: max 3 attempts, exponential backoff,
    1-second initial delay, 60-second cap.
    """
    max_attempts: int = 3
    backoff: BackoffStrategy = BackoffStrategy.EXPONENTIAL
    initial_delay_ms: int = 1000
    max_delay_ms: int = 60000
    retry_on: tuple[str, ...] = ("timeout", "5xx", "connection_refused", "temporary")

    def delay_for_attempt(self, attempt: int) -> int:
        """Calculate delay in milliseconds for the given attempt (1-indexed)."""
        if self.backoff == BackoffStrategy.CONSTANT:
            return self.initial_delay_ms
        elif self.backoff == BackoffStrategy.LINEAR:
            return min(self.initial_delay_ms * attempt, self.max_delay_ms)
        else:  # EXPONENTIAL
            delay = self.initial_delay_ms * (2 ** (attempt - 1))
            return min(delay, self.max_delay_ms)


@dataclass
class DurableActivity:
    """A unit of work with retry policy and optional compensation.

    Each activity has a unique ID for idempotency checking and a
    compensation action that reverses its effects if the saga fails.
    """
    activity_id: str
    action: str  # qualified action name
    params: dict = field(default_factory=dict)
    retry_policy: RetryPolicy = field(default_factory=RetryPolicy)
    compensation: Optional[str] = None  # reverse action name
    status: str = "pending"  # pending | running | completed | failed | compensated
    attempt: int = 0
    error: Optional[str] = None
