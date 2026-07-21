# M2-P3 — Orchestrated Dispatch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `dispatch()` — a coordinator that runs an objective for one `AgentSpec` **through** the `CanonicalLoop` (resolving the model via `ModelRouter`, walking the 7 phases, driving the phase-lock), completely isolated from the live chat path.

**Architecture:** `dispatch(spec, objective, ...)` resolves the agent's model (`resolve_model`), creates a `CanonicalLoop` bound to a phase-lock registry + session id, and walks every phase — applying it (so the phase-lock takes effect for anything that runs under that session id) and invoking an **injected `runner` callback** to perform the phase's work. The runner is injected so this is unit-testable with no network and no touching of `agent_loop.py`. A real `stream_agent_loop`-backed runner is a later phase; here we prove the orchestration contract.

**Why isolated:** `stream_agent_loop` (agent_loop.py:1934) serves ALL chat and defaults the phase-lock to BUILD on purpose. Injecting a CLASSIFY-starting loop there would block writes/exec for normal chat. Orchestrated dispatch is opt-in: only structured agent runs walk the phases.

**Tech Stack:** Python 3.12 dataclasses, existing `src/orchestrator/{spec,dispatcher,loop,phases}.py`, pytest. No network.

**Reference:** Design spec Section 1 ("dispatcher spawns sub-agents") + Section 2. Corrects roadmap M2-P3 from "inject into agent_loop" to "isolated orchestrated dispatch" (user-approved 2026-07-01).

---

## File Structure

- Modify: `src/orchestrator/dispatcher.py` — add `DispatchResult` + `dispatch()`
- Modify: `src/orchestrator/__init__.py` — export `DispatchResult`, `dispatch`
- Test: `tests/test_orchestrator_dispatch_run.py`

---

## Task 1: `dispatch()` walks phases with an injected runner

**Files:**
- Modify: `src/orchestrator/dispatcher.py`
- Test: `tests/test_orchestrator_dispatch_run.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_orchestrator_dispatch_run.py
"""Tests for src/orchestrator/dispatcher.py::dispatch (orchestrated run)."""
from src.orchestrator.spec import AgentSpec
from src.orchestrator.phases import Phase
from src.orchestrator.dispatcher import dispatch, DispatchResult


class _FakeRegistry:
    def __init__(self):
        self.phase_calls = []

    def set_phase(self, session_id, phase):
        self.phase_calls.append((session_id, phase))


class _FakeRouter:
    def route(self, intent_category, stage=None):
        return "openai/deepseek-v4-flash"


def test_dispatch_walks_all_seven_phases_in_order():
    reg = _FakeRegistry()
    spec = AgentSpec(name="gsd-planner")
    calls = []

    def runner(phase, spec, objective, model):
        calls.append(phase.name)
        return f"did {phase.name}"

    result = dispatch(
        spec, "build feature X",
        session_id="run1", registry=reg, router=_FakeRouter(), runner=runner,
    )
    assert isinstance(result, DispatchResult)
    assert result.phases_run == [
        "CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE",
    ]
    assert calls == result.phases_run
    assert result.completed is True


def test_dispatch_applies_phase_to_registry_each_step():
    reg = _FakeRegistry()
    spec = AgentSpec(name="a")
    dispatch(spec, "obj", session_id="run2", registry=reg, router=_FakeRouter(),
             runner=lambda **kw: None)
    assert [p for _, p in reg.phase_calls] == [
        "CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE",
    ]
    assert all(sid == "run2" for sid, _ in reg.phase_calls)


def test_dispatch_resolves_model_via_router():
    spec = AgentSpec(name="a", model=None)
    result = dispatch(spec, "obj", session_id="r", registry=_FakeRegistry(),
                      router=_FakeRouter(), runner=lambda **kw: None)
    assert result.model == "openai/deepseek-v4-flash"


def test_dispatch_uses_explicit_spec_model():
    spec = AgentSpec(name="a", model="openai/kimi-k2.7-code")
    result = dispatch(spec, "obj", session_id="r", registry=_FakeRegistry(),
                      router=_FakeRouter(), runner=lambda **kw: None)
    assert result.model == "openai/kimi-k2.7-code"


def test_dispatch_collects_runner_outputs_per_phase():
    spec = AgentSpec(name="a")
    result = dispatch(spec, "obj", session_id="r", registry=_FakeRegistry(),
                      router=_FakeRouter(),
                      runner=lambda phase, **kw: f"out-{phase.name}")
    assert result.outputs["CLASSIFY"] == "out-CLASSIFY"
    assert result.outputs["MEMORY_OBSERVE"] == "out-MEMORY_OBSERVE"


def test_dispatch_without_runner_still_walks_phases():
    reg = _FakeRegistry()
    spec = AgentSpec(name="a")
    result = dispatch(spec, "obj", session_id="r", registry=reg, router=_FakeRouter())
    assert len(result.phases_run) == 7
    assert result.outputs == {}


def test_runner_receives_resolved_model_and_objective():
    spec = AgentSpec(name="a", model="openai/glm-5.2")
    seen = {}

    def runner(phase, spec, objective, model):
        seen["model"] = model
        seen["objective"] = objective
        return None

    dispatch(spec, "ship it", session_id="r", registry=_FakeRegistry(),
             router=_FakeRouter(), runner=runner)
    assert seen["model"] == "openai/glm-5.2"
    assert seen["objective"] == "ship it"
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python -m pytest tests/test_orchestrator_dispatch_run.py -v`
Expected: FAIL with `ImportError: cannot import name 'dispatch'`

- [ ] **Step 3: Add the implementation to `src/orchestrator/dispatcher.py`**

Append below the existing `resolve_model`:

```python
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional

from src.orchestrator.loop import CanonicalLoop


@dataclass
class DispatchResult:
    agent: str
    model: Optional[str]
    phases_run: List[str] = field(default_factory=list)
    outputs: Dict[str, Any] = field(default_factory=dict)
    completed: bool = False


def dispatch(
    spec: AgentSpec,
    objective: str,
    *,
    session_id: str,
    registry=None,
    router=None,
    stage: Optional[str] = None,
    runner: Optional[Callable] = None,
) -> DispatchResult:
    """Run `objective` for `spec` through the canonical 7-phase loop.

    This is an ORCHESTRATED run — isolated from the live chat path. For each
    phase it (1) applies the phase to the phase-lock registry for `session_id`,
    then (2) invokes the injected `runner(phase, spec, objective, model)` to do
    the work. The runner is injected so this coordinator is fully unit-testable;
    a real stream_agent_loop-backed runner lands in a later phase.
    """
    model = resolve_model(spec, router=router, stage=stage)
    loop = CanonicalLoop(session_id, registry=registry)

    result = DispatchResult(agent=spec.name, model=model)
    while True:
        loop.apply()
        phase = loop.current
        result.phases_run.append(phase.name)
        if runner is not None:
            result.outputs[phase.name] = runner(
                phase=phase, spec=spec, objective=objective, model=model
            )
        if not loop.advance():
            break

    result.completed = True
    return result
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_orchestrator_dispatch_run.py -v`
Expected: PASS (7 passed)

- [ ] **Step 5: Update exports**

```python
# src/orchestrator/__init__.py — add to imports and __all__
from src.orchestrator.dispatcher import resolve_model, dispatch, DispatchResult
```
(Add `"dispatch"`, `"DispatchResult"` to `__all__`.)

- [ ] **Step 6: Run the whole orchestrator suite**

Run: `python -m pytest tests/test_orchestrator_*.py -v`
Expected: PASS (31 prior + 7 = 38 passed)

- [ ] **Step 7: Commit**

```bash
git add src/orchestrator/dispatcher.py src/orchestrator/__init__.py tests/test_orchestrator_dispatch_run.py
git commit -m "feat(orchestrator): dispatch runs objective through canonical loop"
```

---

## Task 2: Update tracking doc

**Files:**
- Modify: `.planning/INTEGRATION-TRACKING.md`

- [ ] **Step 1:** Add "Bloc 0c — Dispatch orchestré (M2-P3)" recording: `dispatch()` walks the 7 phases,
resolves model via router, drives phase-lock per session, runner injected (real runner = later phase).
Note the safety correction: chat path untouched; phase discipline applies only to orchestrated runs.

- [ ] **Step 2: Commit**

```bash
git add .planning/INTEGRATION-TRACKING.md
git commit -m "docs(tracking): M2-P3 orchestrated dispatch delivered"
```

---

## Self-Review

**Spec coverage (Section 1 dispatcher):** `dispatch()` ties spec + model resolution + loop + phase-lock
together → Task 1 ✅. Real session spawning is deferred (runner injection point defined).

**Placeholder scan:** No TBD/TODO; all code complete. ✅

**Type consistency:** `DispatchResult(agent, model, phases_run, outputs, completed)` used identically in
tests. `dispatch(spec, objective, *, session_id, registry, router, stage, runner)` signature matches all
call sites. `runner(phase, spec, objective, model)` keyword contract matches the tests. Reuses
`resolve_model`, `CanonicalLoop`, `Phase` unchanged from P1/P2. ✅

**Scope:** Pure, injected runner, no chat-path edits — safe. Produces a working, tested orchestration
coordinator. ✅
