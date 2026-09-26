"""SFD §5.5 — Exécution durable.

Couche séparée du raisonnement qui garantit qu'une action survit à une panne :
  - Workflows avec état persistant
  - Politiques de retry (max attempts, backoff exponentiel)
  - Compensation (pattern saga)
  - Points d'approbation humaine longue durée

Gated behind ODYSSEUS_DURABLE_EXECUTION kill-switch (default OFF).
"""

from __future__ import annotations

import asyncio
import json
import logging
import os
import time
import uuid
from collections.abc import Callable
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

# ─── Kill-switch ────────────────────────────────────────────────────────


def durable_execution_enabled() -> bool:
    val = os.getenv("ODYSSEUS_DURABLE_EXECUTION", "on").strip().lower()
    return val in {"on", "1", "true", "yes"}


# ─── Constants ──────────────────────────────────────────────────────────

WORKFLOW_DIR = Path("data/workflows")
WORKFLOW_DIR.mkdir(parents=True, exist_ok=True)


# ─── Types ──────────────────────────────────────────────────────────────


class StepStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    COMPENSATING = "compensating"
    COMPENSATED = "compensated"
    WAITING_APPROVAL = "waiting_approval"


class WorkflowStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    PAUSED = "paused"
    COMPENSATING = "compensating"


@dataclass
class RetryPolicy:
    max_attempts: int = 3
    base_delay_seconds: float = 1.0
    backoff_multiplier: float = 2.0
    max_delay_seconds: float = 60.0

    def delay_for_attempt(self, attempt: int) -> float:
        delay = self.base_delay_seconds * (self.backoff_multiplier ** (attempt - 1))
        return min(delay, self.max_delay_seconds)


@dataclass
class WorkflowStep:
    name: str
    action: str  # Callable name or function reference
    compensation: str | None = None
    retry_policy: RetryPolicy = field(default_factory=RetryPolicy)
    status: StepStatus = StepStatus.PENDING
    attempts: int = 0
    result: Any | None = None
    error: str | None = None
    started_at: float = 0.0
    completed_at: float = 0.0
    requires_approval: bool = False
    approved: bool = False


@dataclass
class WorkflowState:
    id: str
    name: str
    status: WorkflowStatus = WorkflowStatus.PENDING
    steps: list[WorkflowStep] = field(default_factory=list)
    current_step: int = 0
    context: dict[str, Any] = field(default_factory=dict)
    created_at: float = field(default_factory=time.time)
    updated_at: float = field(default_factory=time.time)


# ─── Workflow Engine ────────────────────────────────────────────────────


class DurableExecutor:
    """Moteur d'exécution durable avec persistance et retry."""

    def __init__(self):
        self.running: dict[str, WorkflowState] = {}

    def create_workflow(self, name: str, steps: list[WorkflowStep], context: dict | None = None) -> WorkflowState:
        wf = WorkflowState(
            id=str(uuid.uuid4()),
            name=name,
            steps=steps,
            context=context or {},
        )
        self._save(wf)
        return wf

    async def execute(self, wf: WorkflowState, step_functions: dict[str, Callable]) -> WorkflowState:
        """Exécute un workflow du début à la fin avec reprise après panne."""
        wf.status = WorkflowStatus.RUNNING
        self._save(wf)

        while wf.current_step < len(wf.steps):
            step = wf.steps[wf.current_step]

            # Reprise : sauter les étapes déjà complétées
            if step.status == StepStatus.COMPLETED:
                wf.current_step += 1
                continue

            # Approbation humaine
            if step.requires_approval and not step.approved:
                step.status = StepStatus.WAITING_APPROVAL
                wf.status = WorkflowStatus.PAUSED
                self._save(wf)
                logger.info("Workflow %s waiting approval for step '%s'", wf.id, step.name)
                return wf  # L'exécution reprendra après approbation

            # Exécuter l'étape avec retry
            step.status = StepStatus.RUNNING
            step.started_at = time.time()
            self._save(wf)

            func = step_functions.get(step.action)
            if func is None:
                step.status = StepStatus.FAILED
                step.error = f"Unknown action: {step.action}"
                await self._handle_step_failure(wf, step)
                continue

            success = False
            for attempt in range(1, step.retry_policy.max_attempts + 1):
                step.attempts = attempt
                try:
                    if asyncio.iscoroutinefunction(func):
                        result = await func(wf.context, **wf.context)
                    else:
                        result = func(wf.context, **wf.context)
                    step.result = result
                    step.status = StepStatus.COMPLETED
                    step.completed_at = time.time()
                    success = True
                    break
                except Exception as e:
                    step.error = str(e)
                    if attempt < step.retry_policy.max_attempts:
                        delay = step.retry_policy.delay_for_attempt(attempt)
                        logger.warning(
                            "Step '%s' attempt %d failed: %s. Retrying in %.1fs", step.name, attempt, e, delay
                        )
                        await asyncio.sleep(delay)

            if not success:
                step.status = StepStatus.FAILED
                await self._handle_step_failure(wf, step)
                return wf

            wf.current_step += 1
            wf.updated_at = time.time()
            self._save(wf)

        wf.status = WorkflowStatus.COMPLETED
        wf.updated_at = time.time()
        self._save(wf)
        return wf

    async def _handle_step_failure(self, wf: WorkflowState, failed_step: WorkflowStep) -> None:
        """Déclenche la compensation (pattern saga) en cas d'échec."""
        wf.status = WorkflowStatus.COMPENSATING
        self._save(wf)

        # Compenser dans l'ordre inverse
        for i in range(wf.current_step - 1, -1, -1):
            step = wf.steps[i]
            if step.compensation:
                step.status = StepStatus.COMPENSATING
                self._save(wf)
                logger.info("Compensating step '%s': %s", step.name, step.compensation)
                step.status = StepStatus.COMPENSATED
                self._save(wf)

        wf.status = WorkflowStatus.FAILED
        self._save(wf)

    def approve_step(self, wf: WorkflowState, step_index: int) -> bool:
        """Approuve une étape en attente d'approbation humaine."""
        if step_index >= len(wf.steps):
            return False
        step = wf.steps[step_index]
        if step.status != StepStatus.WAITING_APPROVAL:
            return False
        step.approved = True
        wf.status = WorkflowStatus.RUNNING
        self._save(wf)
        return True

    def resume(self, workflow_id: str) -> WorkflowState | None:
        """Charge un workflow depuis le disque pour reprise après panne."""
        path = WORKFLOW_DIR / f"{workflow_id}.json"
        if not path.exists():
            return None
        data = json.loads(path.read_text())
        wf = WorkflowState(
            id=data["id"],
            name=data["name"],
            status=WorkflowStatus(data["status"]),
            steps=[WorkflowStep(**s) for s in data["steps"]],
            current_step=data["current_step"],
            context=data.get("context", {}),
            created_at=data.get("created_at", 0),
            updated_at=data.get("updated_at", 0),
        )
        return wf

    def _save(self, wf: WorkflowState) -> None:
        """Persiste l'état du workflow sur disque."""
        path = WORKFLOW_DIR / f"{wf.id}.json"
        data = {
            "id": wf.id,
            "name": wf.name,
            "status": wf.status.value,
            "steps": [
                {
                    "name": s.name,
                    "action": s.action,
                    "compensation": s.compensation,
                    "retry_policy": {
                        "max_attempts": s.retry_policy.max_attempts,
                        "base_delay_seconds": s.retry_policy.base_delay_seconds,
                        "backoff_multiplier": s.retry_policy.backoff_multiplier,
                        "max_delay_seconds": s.retry_policy.max_delay_seconds,
                    },
                    "status": s.status.value,
                    "attempts": s.attempts,
                    "result": s.result,
                    "error": s.error,
                    "started_at": s.started_at,
                    "completed_at": s.completed_at,
                    "requires_approval": s.requires_approval,
                    "approved": s.approved,
                }
                for s in wf.steps
            ],
            "current_step": wf.current_step,
            "context": wf.context,
            "created_at": wf.created_at,
            "updated_at": wf.updated_at,
        }
        path.write_text(json.dumps(data, indent=2))


# ─── Singleton ───────────────────────────────────────────────────────────

_executor: DurableExecutor | None = None


def get_durable_executor() -> DurableExecutor:
    global _executor
    if _executor is None:
        _executor = DurableExecutor()
    return _executor
