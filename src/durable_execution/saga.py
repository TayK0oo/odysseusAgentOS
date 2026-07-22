"""
Saga Coordinator — compensation in reverse order.

When a multi-step workflow fails after some steps have completed,
the SagaCoordinator triggers compensation actions in reverse order
to undo the effects of already-completed steps.

Pattern: https://microservices.io/patterns/data/saga.html
"""

import logging
from typing import Callable, Optional

from src.durable_execution.activity import DurableActivity

logger = logging.getLogger(__name__)

# Type alias for the action executor callback
ActionExecutor = Callable[[str, dict], bool]


class SagaCoordinator:
    """Orchestrates compensation for partially-completed sagas.

    Usage:
        coordinator = SagaCoordinator(executor=my_execute_function)
        try:
            for activity in activities:
                coordinator.execute_step(activity)
        except Exception:
            await coordinator.compensate()
            raise
    """

    def __init__(self, executor: ActionExecutor):
        self._executor = executor
        self._completed: list[DurableActivity] = []

    def execute_step(self, activity: DurableActivity) -> None:
        """Execute one activity and track it for potential compensation."""
        success = self._executor(activity.action, activity.params)
        if not success:
            raise SagaStepFailed(
                f"Activity {activity.activity_id} ({activity.action}) failed",
                activity=activity,
            )
        activity.status = "completed"
        self._completed.append(activity)

    async def compensate(self) -> list[DurableActivity]:
        """Compensate all completed steps in reverse order.

        Returns the list of compensated activities.
        Each compensation is best-effort — failures are logged, not raised.
        """
        compensated: list[DurableActivity] = []
        for activity in reversed(self._completed):
            if activity.compensation is None:
                logger.warning(
                    "Activity %s has no compensation defined — skipping",
                    activity.activity_id,
                )
                continue
            try:
                success = self._executor(activity.compensation, activity.params)
                if success:
                    activity.status = "compensated"
                else:
                    logger.error(
                        "Compensation failed for activity %s (%s)",
                        activity.activity_id, activity.compensation,
                    )
            except Exception:
                logger.exception(
                    "Compensation error for activity %s", activity.activity_id
                )
            compensated.append(activity)
        self._completed.clear()
        return compensated

    @property
    def completed_count(self) -> int:
        return len(self._completed)


class SagaStepFailed(Exception):
    """Raised when a saga step fails, triggering compensation."""

    def __init__(self, message: str, activity: DurableActivity):
        super().__init__(message)
        self.activity = activity
