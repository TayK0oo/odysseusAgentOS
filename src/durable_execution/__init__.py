"""
Durable Execution — Workflow engine with SQLite persistence.

Guarantees that long-running actions survive crashes and resume exactly
where they left off. Independent of the AI reasoning layer.

Architecture:
    src/durable_execution/
    ├── engine.py     — DurableEngine: create, execute, resume workflows
    ├── activity.py   — DurableActivity + RetryPolicy
    ├── saga.py       — SagaCoordinator: compensation in reverse order
    ├── signals.py    — SignalWaiter: human approval (via Thought Bus)
    ├── models.py     — SQLAlchemy models for workflow state
    ├── integration.py — @on_phase subscriber for Thought Bus
    └── __init__.py
"""

from src.durable_execution.engine import DurableEngine
from src.durable_execution.activity import DurableActivity, RetryPolicy
from src.durable_execution.saga import SagaCoordinator
from src.durable_execution.signals import SignalWaiter, ApprovalSignal

__all__ = [
    "DurableEngine",
    "DurableActivity",
    "RetryPolicy",
    "SagaCoordinator",
    "SignalWaiter",
    "ApprovalSignal",
]
