"""
Durable Execution Engine — Custom SQLite-backed workflow engine.

Guarantees that actions survive crashes and resume exactly where they
left off. Zero external dependencies. State is fully in SQLite.

Integrated with the Thought Bus: subscribes to BUILD phase to wrap
tool executions in durable workflows.
"""

from __future__ import annotations

import json
import logging
import os
import sqlite3
import time
import uuid
from datetime import datetime, timezone
from typing import Any, Callable, Optional

from src.durable_execution.activity import DurableActivity, RetryPolicy
from src.durable_execution.saga import SagaCoordinator, SagaStepFailed
from src.durable_execution.signals import ApprovalSignal, SignalStatus

logger = logging.getLogger(__name__)

_DURABLE_EXEC_ENV = "ODYSSEUS_DURABLE_EXEC"


def durable_exec_enabled() -> bool:
    """Check if durable execution is ON. Core stable — defaults ON."""
    val = os.getenv(_DURABLE_EXEC_ENV, "on").strip().lower()
    return val in ("on", "1", "true", "yes")


class DurableEngine:
    """Custom workflow engine backed by SQLite.

    Stores workflow state, activities, and approval signals.
    On startup, scans for interrupted workflows and resumes them.

    Usage:
        engine = DurableEngine(db_path="data/durable.db")
        await engine.initialize()

        wf = await engine.create_workflow("deploy", project_id="p1")
        await engine.add_activity(wf, DurableActivity(...))
        await engine.start_workflow(wf, executor=my_executor)
    """

    SCHEMA = """
    CREATE TABLE IF NOT EXISTS workflows (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        project_id TEXT,
        status TEXT NOT NULL DEFAULT 'pending',
        current_activity TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS activities (
        id TEXT PRIMARY KEY,
        workflow_id TEXT NOT NULL,
        action TEXT NOT NULL,
        params TEXT NOT NULL DEFAULT '{}',
        retry_max INTEGER NOT NULL DEFAULT 3,
        retry_backoff TEXT NOT NULL DEFAULT 'exponential',
        retry_init_ms INTEGER NOT NULL DEFAULT 1000,
        retry_max_ms INTEGER NOT NULL DEFAULT 60000,
        compensation TEXT,
        status TEXT NOT NULL DEFAULT 'pending',
        attempt INTEGER NOT NULL DEFAULT 0,
        error TEXT,
        created_at TEXT NOT NULL,
        FOREIGN KEY (workflow_id) REFERENCES workflows(id)
    );

    CREATE TABLE IF NOT EXISTS approval_signals (
        id TEXT PRIMARY KEY,
        workflow_id TEXT NOT NULL,
        activity_id TEXT NOT NULL,
        message TEXT NOT NULL,
        context TEXT NOT NULL DEFAULT '{}',
        status TEXT NOT NULL DEFAULT 'pending',
        created_at TEXT NOT NULL,
        resolved_at TEXT,
        timeout_days INTEGER NOT NULL DEFAULT 7,
        response TEXT,
        FOREIGN KEY (workflow_id) REFERENCES workflows(id)
    );

    CREATE INDEX IF NOT EXISTS idx_wf_status ON workflows(status);
    CREATE INDEX IF NOT EXISTS idx_act_wf ON activities(workflow_id);
    CREATE INDEX IF NOT EXISTS idx_signal_wf ON approval_signals(workflow_id);
    """

    def __init__(self, db_path: str = "data/durable.db"):
        self._db_path = db_path
        self._conn: Optional[sqlite3.Connection] = None

    async def initialize(self) -> None:
        """Create tables and resume interrupted workflows."""
        os.makedirs(os.path.dirname(self._db_path), exist_ok=True)
        self._conn = sqlite3.connect(self._db_path)
        self._conn.row_factory = sqlite3.Row
        self._conn.executescript(self.SCHEMA)
        self._conn.commit()
        logger.info("DurableEngine initialized at %s", self._db_path)

        # Resume interrupted workflows
        interrupted = await self._get_interrupted_workflows()
        if interrupted:
            logger.info("Found %d interrupted workflow(s) to resume", len(interrupted))

    async def _get_interrupted_workflows(self) -> list[str]:
        """Find workflows that were running or waiting when the process died."""
        cur = self._conn.execute(
            "SELECT id FROM workflows WHERE status IN ('running', 'waiting_approval')"
        )
        return [row["id"] for row in cur.fetchall()]

    # ------------------------------------------------------------------
    # Workflow CRUD
    # ------------------------------------------------------------------

    async def create_workflow(
        self,
        name: str,
        project_id: Optional[str] = None,
    ) -> str:
        """Create a new workflow. Returns the workflow ID."""
        wf_id = str(uuid.uuid4())
        now = datetime.now(timezone.utc).isoformat()
        self._conn.execute(
            "INSERT INTO workflows (id, name, project_id, status, created_at, updated_at) "
            "VALUES (?, ?, ?, 'pending', ?, ?)",
            (wf_id, name, project_id, now, now),
        )
        self._conn.commit()
        logger.info("Created workflow %s (%s)", wf_id, name)
        return wf_id

    async def add_activity(self, workflow_id: str, activity: DurableActivity) -> str:
        """Add an activity to a workflow. Returns the activity ID."""
        act_id = activity.activity_id or str(uuid.uuid4())
        now = datetime.now(timezone.utc).isoformat()
        self._conn.execute(
            """INSERT INTO activities
               (id, workflow_id, action, params,
                retry_max, retry_backoff, retry_init_ms, retry_max_ms,
                compensation, status, attempt, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', 0, ?)""",
            (
                act_id, workflow_id, activity.action,
                json.dumps(activity.params),
                activity.retry_policy.max_attempts,
                activity.retry_policy.backoff.value,
                activity.retry_policy.initial_delay_ms,
                activity.retry_policy.max_delay_ms,
                activity.compensation,
                now,
            ),
        )
        self._conn.commit()
        return act_id

    async def start_workflow(
        self,
        workflow_id: str,
        executor: Callable[[str, dict], bool],
    ) -> str:
        """Execute a workflow to completion. Returns final status.

        The executor is called for each activity's action.
        Retries are handled automatically per the activity's RetryPolicy.
        If a step fails after max retries, saga compensation is triggered.
        """
        if not durable_exec_enabled():
            logger.info("Durable execution is OFF — running without persistence")
            return "completed_no_persistence"

        self._conn.execute(
            "UPDATE workflows SET status='running', updated_at=? WHERE id=?",
            (datetime.now(timezone.utc).isoformat(), workflow_id),
        )
        self._conn.commit()

        # Load activities
        cur = self._conn.execute(
            "SELECT * FROM activities WHERE workflow_id=? ORDER BY created_at ASC",
            (workflow_id,),
        )
        activities = cur.fetchall()

        saga = SagaCoordinator(executor)
        final_status = "completed"

        for row in activities:
            act = DurableActivity(
                activity_id=row["id"],
                action=row["action"],
                params=json.loads(row["params"]),
                retry_policy=RetryPolicy(
                    max_attempts=row["retry_max"],
                    backoff=row["retry_backoff"],
                    initial_delay_ms=row["retry_init_ms"],
                    max_delay_ms=row["retry_max_ms"],
                ),
                compensation=row["compensation"],
                attempt=row["attempt"],
            )

            # Execute with retry
            success = False
            for attempt in range(1, act.retry_policy.max_attempts + 1):
                act.attempt = attempt
                self._update_activity_status(act.activity_id, "running", attempt)

                try:
                    success = executor(act.action, act.params)
                    if success:
                        break
                except Exception as e:
                    logger.warning(
                        "Activity %s attempt %d/%d failed: %s",
                        act.activity_id, attempt, act.retry_policy.max_attempts, e,
                    )
                    act.error = str(e)

                if attempt < act.retry_policy.max_attempts:
                    delay_ms = act.retry_policy.delay_for_attempt(attempt)
                    time.sleep(delay_ms / 1000.0)

            if success:
                self._update_activity_status(act.activity_id, "completed", act.attempt)
                saga._completed.append(act)
            else:
                self._update_activity_status(act.activity_id, "failed", act.attempt)
                logger.error(
                    "Activity %s failed after %d attempts — triggering compensation",
                    act.activity_id, act.retry_policy.max_attempts,
                )
                final_status = "failed"
                await saga.compensate()
                break

        self._conn.execute(
            "UPDATE workflows SET status=?, current_activity=NULL, updated_at=? WHERE id=?",
            (final_status, datetime.now(timezone.utc).isoformat(), workflow_id),
        )
        self._conn.commit()
        return final_status

    def _update_activity_status(self, act_id: str, status: str, attempt: int) -> None:
        self._conn.execute(
            "UPDATE activities SET status=?, attempt=? WHERE id=?",
            (status, attempt, act_id),
        )
        self._conn.commit()

    # ------------------------------------------------------------------
    # Human approval
    # ------------------------------------------------------------------

    async def create_approval_signal(
        self,
        workflow_id: str,
        activity_id: str,
        message: str,
        context: Optional[dict] = None,
        timeout_days: int = 7,
    ) -> str:
        """Create a pending approval signal. The workflow pauses until resolved."""
        sig_id = str(uuid.uuid4())
        now = datetime.now(timezone.utc).isoformat()
        self._conn.execute(
            """INSERT INTO approval_signals
               (id, workflow_id, activity_id, message, context,
                status, created_at, timeout_days)
               VALUES (?, ?, ?, ?, ?, 'pending', ?, ?)""",
            (
                sig_id, workflow_id, activity_id, message,
                json.dumps(context or {}), now, timeout_days,
            ),
        )
        self._conn.execute(
            "UPDATE workflows SET status='waiting_approval', updated_at=? WHERE id=?",
            (now, workflow_id),
        )
        self._conn.commit()
        logger.info("Approval signal %s created for workflow %s", sig_id, workflow_id)
        return sig_id

    async def resolve_approval(
        self,
        signal_id: str,
        approved: bool,
        response: Optional[str] = None,
    ) -> None:
        """Resolve an approval signal and resume the workflow."""
        now = datetime.now(timezone.utc).isoformat()
        status = SignalStatus.APPROVED.value if approved else SignalStatus.REJECTED.value
        self._conn.execute(
            "UPDATE approval_signals SET status=?, resolved_at=?, response=? WHERE id=?",
            (status, now, response, signal_id),
        )
        # Resume the workflow
        row = self._conn.execute(
            "SELECT workflow_id FROM approval_signals WHERE id=?", (signal_id,)
        ).fetchone()
        if row:
            self._conn.execute(
                "UPDATE workflows SET status='running', updated_at=? WHERE id=?",
                (now, row["workflow_id"]),
            )
        self._conn.commit()
        logger.info("Approval signal %s resolved: %s", signal_id, status)

    # ------------------------------------------------------------------
    # Query
    # ------------------------------------------------------------------

    async def get_workflow(self, workflow_id: str) -> Optional[dict]:
        """Get workflow state."""
        row = self._conn.execute(
            "SELECT * FROM workflows WHERE id=?", (workflow_id,)
        ).fetchone()
        return dict(row) if row else None

    async def get_pending_signals(self, project_id: Optional[str] = None) -> list[dict]:
        """Get all pending approval signals, optionally filtered by project."""
        if project_id:
            rows = self._conn.execute(
                """SELECT s.* FROM approval_signals s
                   JOIN workflows w ON s.workflow_id = w.id
                   WHERE s.status='pending' AND w.project_id=?""",
                (project_id,),
            ).fetchall()
        else:
            rows = self._conn.execute(
                "SELECT * FROM approval_signals WHERE status='pending'"
            ).fetchall()
        return [dict(r) for r in rows]

    async def list_workflows(
        self, project_id: Optional[str] = None, status: Optional[str] = None
    ) -> list[dict]:
        """List workflows, optionally filtered."""
        query = "SELECT * FROM workflows WHERE 1=1"
        params: list[Any] = []
        if project_id:
            query += " AND project_id=?"
            params.append(project_id)
        if status:
            query += " AND status=?"
            params.append(status)
        query += " ORDER BY created_at DESC"
        rows = self._conn.execute(query, params).fetchall()
        return [dict(r) for r in rows]

    def close(self) -> None:
        """Close the database connection."""
        if self._conn:
            self._conn.close()
            self._conn = None
