"""
Human approval signals for durable workflows.

A workflow can pause and wait for a human approval signal
for an arbitrary duration (hours, days) without consuming
resources. The signal is routed through the Thought Bus + Gateway.
"""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Optional


class SignalStatus(str, Enum):
    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    TIMED_OUT = "timed_out"


@dataclass
class ApprovalSignal:
    """A human approval request embedded in a durable workflow.

    The workflow engine stores this and waits for an external signal.
    When received, the workflow resumes automatically.
    """
    signal_id: str
    workflow_id: str
    activity_id: str
    message: str  # what the human needs to decide
    context: dict = field(default_factory=dict)  # decision context
    status: SignalStatus = SignalStatus.PENDING
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    resolved_at: Optional[str] = None
    timeout_days: int = 7  # max wait before auto-timeout
    response: Optional[str] = None


class SignalWaiter:
    """Manages approval signals within the workflow engine.

    Signals are stored in SQLite alongside workflow state.
    The Gateway polls for pending signals and presents them to the user.
    When the user responds, the signal is resolved and the workflow resumes.
    """

    @staticmethod
    def is_pending(signal: ApprovalSignal) -> bool:
        """Check if a signal is still awaiting human response."""
        if signal.status != SignalStatus.PENDING:
            return False
        if signal.timeout_days > 0:
            created = datetime.fromisoformat(signal.created_at)
            elapsed = (datetime.now(timezone.utc) - created).days
            if elapsed >= signal.timeout_days:
                signal.status = SignalStatus.TIMED_OUT
                return False
        return True
