"""
Tests for the Durable Execution engine (src/durable_execution/).

Covers: RetryPolicy, DurableActivity, SagaCoordinator,
DurableEngine, SignalWaiter, kill-switch.
"""

import os
import tempfile
import pytest

from src.durable_execution.activity import (
    DurableActivity,
    RetryPolicy,
    BackoffStrategy,
)
from src.durable_execution.saga import SagaCoordinator, SagaStepFailed
from src.durable_execution.signals import ApprovalSignal, SignalWaiter, SignalStatus
from src.durable_execution.engine import DurableEngine, durable_exec_enabled


# ---------------------------------------------------------------------------
# RetryPolicy
# ---------------------------------------------------------------------------

class TestRetryPolicy:
    def test_defaults(self):
        rp = RetryPolicy()
        assert rp.max_attempts == 3
        assert rp.backoff == BackoffStrategy.EXPONENTIAL
        assert rp.initial_delay_ms == 1000
        assert rp.max_delay_ms == 60000

    def test_constant_delay(self):
        rp = RetryPolicy(backoff=BackoffStrategy.CONSTANT, initial_delay_ms=500)
        assert rp.delay_for_attempt(1) == 500
        assert rp.delay_for_attempt(5) == 500

    def test_linear_delay(self):
        rp = RetryPolicy(backoff=BackoffStrategy.LINEAR, initial_delay_ms=1000, max_delay_ms=5000)
        assert rp.delay_for_attempt(1) == 1000
        assert rp.delay_for_attempt(3) == 3000
        assert rp.delay_for_attempt(10) == 5000  # capped

    def test_exponential_delay(self):
        rp = RetryPolicy(backoff=BackoffStrategy.EXPONENTIAL, initial_delay_ms=1000, max_delay_ms=30000)
        assert rp.delay_for_attempt(1) == 1000
        assert rp.delay_for_attempt(2) == 2000
        assert rp.delay_for_attempt(3) == 4000
        assert rp.delay_for_attempt(10) == 30000  # capped


# ---------------------------------------------------------------------------
# DurableActivity
# ---------------------------------------------------------------------------

class TestDurableActivity:
    def test_basic_activity(self):
        act = DurableActivity(
            activity_id="a1",
            action="docker_build",
            params={"image": "myapp"},
        )
        assert act.activity_id == "a1"
        assert act.action == "docker_build"
        assert act.params == {"image": "myapp"}
        assert act.status == "pending"
        assert act.attempt == 0

    def test_with_compensation(self):
        act = DurableActivity(
            activity_id="a1",
            action="docker_build",
            compensation="docker_rmi",
        )
        assert act.compensation == "docker_rmi"


# ---------------------------------------------------------------------------
# SagaCoordinator
# ---------------------------------------------------------------------------

class TestSagaCoordinator:
    def test_successful_execution(self):
        executed = []
        saga = SagaCoordinator(executor=lambda action, params: executed.append(action) or True)

        act1 = DurableActivity("a1", "step1")
        act2 = DurableActivity("a2", "step2")

        saga.execute_step(act1)
        saga.execute_step(act2)

        assert executed == ["step1", "step2"]
        assert saga.completed_count == 2
        assert act1.status == "completed"
        assert act2.status == "completed"

    def test_compensation_on_failure(self):
        executed = []
        compensations = []

        def executor(action, params):
            executed.append(action)
            if action == "step2":
                return False
            return True

        saga = SagaCoordinator(executor=executor)

        act1 = DurableActivity("a1", "step1", compensation="undo_step1")
        act2 = DurableActivity("a2", "step2", compensation="undo_step2")

        saga.execute_step(act1)

        with pytest.raises(SagaStepFailed):
            saga.execute_step(act2)

        import asyncio
        compensated = asyncio.run(saga.compensate())
        assert len(compensated) == 1
        assert compensated[0].activity_id == "a1"
        assert act1.status == "compensated"

    def test_no_compensation_skipped(self):
        def executor(action, params):
            return False

        saga = SagaCoordinator(executor=executor)
        act = DurableActivity("a1", "step1")  # no compensation
        saga._completed.append(act)

        import asyncio
        compensated = asyncio.run(saga.compensate())
        assert len(compensated) == 0  # skipped, no compensation


# ---------------------------------------------------------------------------
# SignalWaiter
# ---------------------------------------------------------------------------

class TestSignalWaiter:
    def test_pending_signal(self):
        sig = ApprovalSignal(
            signal_id="s1",
            workflow_id="w1",
            activity_id="a1",
            message="Approve deployment?",
        )
        assert SignalWaiter.is_pending(sig) is True

    def test_resolved_signal_not_pending(self):
        sig = ApprovalSignal(
            signal_id="s1",
            workflow_id="w1",
            activity_id="a1",
            message="Approve?",
            status=SignalStatus.APPROVED,
        )
        assert SignalWaiter.is_pending(sig) is False


# ---------------------------------------------------------------------------
# DurableEngine
# ---------------------------------------------------------------------------

class TestDurableEngine:
    @pytest.fixture
    async def engine(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = os.path.join(tmpdir, "test_durable.db")
            eng = DurableEngine(db_path=db_path)
            await eng.initialize()
            yield eng
            eng.close()

    @pytest.mark.asyncio
    async def test_initialize_creates_tables(self, engine):
        # Tables should exist
        cur = engine._conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        )
        tables = [r["name"] for r in cur.fetchall()]
        assert "workflows" in tables
        assert "activities" in tables
        assert "approval_signals" in tables

    @pytest.mark.asyncio
    async def test_create_workflow(self, engine):
        wf_id = await engine.create_workflow("deploy", project_id="p1")
        assert wf_id is not None

        wf = await engine.get_workflow(wf_id)
        assert wf["name"] == "deploy"
        assert wf["status"] == "pending"
        assert wf["project_id"] == "p1"

    @pytest.mark.asyncio
    async def test_add_activity(self, engine):
        wf_id = await engine.create_workflow("test")
        act = DurableActivity(
            activity_id="a1",
            action="echo",
            params={"msg": "hello"},
        )
        act_id = await engine.add_activity(wf_id, act)
        assert act_id == "a1"

    @pytest.mark.asyncio
    async def test_start_workflow_success(self, engine):
        wf_id = await engine.create_workflow("test")
        act = DurableActivity("a1", "step1")
        await engine.add_activity(wf_id, act)

        executed = []
        status = await engine.start_workflow(wf_id, lambda action, params: executed.append(action) or True)
        assert status == "completed"
        assert executed == ["step1"]

    @pytest.mark.asyncio
    async def test_start_workflow_failure_triggers_compensation(self, engine):
        wf_id = await engine.create_workflow("test")

        act1 = DurableActivity("a1", "step1", compensation="undo1")
        act2 = DurableActivity("a2", "step2", compensation="undo2")

        await engine.add_activity(wf_id, act1)
        await engine.add_activity(wf_id, act2)

        def fail_step2(action, params):
            if action == "step2":
                return False
            return True

        status = await engine.start_workflow(wf_id, fail_step2)
        assert status == "failed"

    @pytest.mark.asyncio
    async def test_approval_signal(self, engine):
        wf_id = await engine.create_workflow("deploy")
        sig_id = await engine.create_approval_signal(
            wf_id, "a1", "Approve deployment?", {"env": "prod"}
        )
        assert sig_id is not None

        signals = await engine.get_pending_signals()
        assert len(signals) >= 1

        await engine.resolve_approval(sig_id, approved=True, response="go ahead")
        signals = await engine.get_pending_signals()
        assert all(s["id"] != sig_id for s in signals)

    @pytest.mark.asyncio
    async def test_list_workflows(self, engine):
        await engine.create_workflow("w1", project_id="p1")
        await engine.create_workflow("w2", project_id="p1")
        await engine.create_workflow("w3", project_id="p2")

        p1 = await engine.list_workflows(project_id="p1")
        assert len(p1) == 2

        p2 = await engine.list_workflows(project_id="p2")
        assert len(p2) == 1

    @pytest.mark.asyncio
    async def test_killswitch_disabled(self, engine, monkeypatch):
        monkeypatch.setenv("ODYSSEUS_DURABLE_EXEC", "off")
        wf_id = await engine.create_workflow("test")
        act = DurableActivity("a1", "step1")
        await engine.add_activity(wf_id, act)

        status = await engine.start_workflow(wf_id, lambda a, p: True)
        # When disabled, returns no_persistence status
        assert "no_persistence" in status


# ---------------------------------------------------------------------------
# Kill-switch
# ---------------------------------------------------------------------------

class TestDurableExecKillSwitch:
    def test_enabled_by_default(self):
        assert durable_exec_enabled() is True

    def test_disabled_via_env(self, monkeypatch):
        monkeypatch.setenv("ODYSSEUS_DURABLE_EXEC", "off")
        assert durable_exec_enabled() is False
