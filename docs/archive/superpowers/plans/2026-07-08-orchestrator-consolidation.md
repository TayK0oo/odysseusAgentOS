# Orchestrator Consolidation (Sub-projet D, portée A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pin the live loop's two phase-inference systems with characterization tests, then extract the inline phase-resolution branch into a pure, tested seam — with zero runtime behavior change.

**Architecture:** Two units. Unit 1 adds `tests/test_orchestrator_phases.py` characterizing the *current* behavior of `PhaseTracker` and `CanonicalLoop`. Unit 2 adds a pure module `src/orchestrator/phase_resolver.py::resolve_current_phase(...)` that mirrors exactly the inline block at `src/agent_loop.py:2562-2572`, then replaces that block with a call. No kill-switch defaults change; the resolver reads only, never mutates.

**Tech Stack:** Python 3, pytest (with `monkeypatch`), FastAPI app (only for the boot smoke-check). All shell commands are prefixed with `rtk` per repo convention.

---

## Preamble — conventions for the implementer

- **RTK is mandatory.** Every shell command in this repo is prefixed with `rtk`. Run `rtk python -m pytest ...`, `rtk git add ...`, never the bare command.
- **Run from repo root:** `C:\Users\ttmdu\Documents\GitHub\odysseusAgentOS`.
- **Behavior preservation is the whole point.** The characterization tests pin *what the code does today*, not what it should ideally do. If a characterization test fails, suspect the test, not the code.
- **The seam is an extract-function-strict refactor.** `resolve_current_phase` must return byte-for-byte the same `Phase | None` the inline block returns for the same inputs. Do not "improve" the logic.
- **Kill-switch defaults must not move:** `ODYSSEUS_LIVE_ORCHESTRATION` stays OFF, `ODYSSEUS_PHASE_TRACKER` stays OFF. No task touches those defaults.

### Reference: the exact inline block being characterized and extracted

Currently at `src/agent_loop.py:2562-2572`:

```python
        # M4: fire agents for current phase (best-effort, never blocks)
        _current_phase = None
        if _use_canonical and _canonical_loop is not None:
            _current_phase = _canonical_loop.current
        elif _phase_tracker is not None:
            # Map PhaseTracker phase name to canonical Phase enum
            _pt_phase = _phase_tracker.infer_phase(round_num, intent=_intent, plan_mode=plan_mode)
            try:
                from src.orchestrator.phases import Phase
                _current_phase = Phase(_pt_phase) if _pt_phase in {p.value for p in Phase} else None
            except Exception:
                _current_phase = None
```

### Reference: the modules under test (current source, unchanged by this plan)

`src/orchestrator/phase_tracker.py`:
- `tracker_enabled()` → reads `ODYSSEUS_PHASE_TRACKER`, truthy set `{"on","1","true","yes"}`, default `"off"` → `False`.
- `PhaseTracker(session_id, registry=None, enabled=None)`: if `enabled` is None it calls `tracker_enabled()`; if a `registry` is passed it is stored regardless of enabled; if no registry and enabled it lazy-imports the singleton; else `_registry = None`.
- `PhaseTracker.infer_phase(round_num, intent=None, plan_mode=False)` (staticmethod) → `"PLAN"` if `plan_mode` else `"BUILD"`.
- `on_round_start(round_num, intent=None, plan_mode=False)`: infers phase; **only if** `self._enabled and self._registry is not None` calls `self._registry.set_phase(self.session_id, phase)`; returns the phase name.

`src/orchestrator/loop.py`:
- `CanonicalLoop(session_id, registry=None)`, `_index = 0`.
- `current` property → `CANONICAL_SEQUENCE[self._index]`.
- `advance()` → `False` if `_index >= len-1` (no move), else `_index += 1` and `True`.
- `goto(phase)` → sets `_index` to that phase's position and calls `apply()`.
- `apply()` → no-op if `_registry is None`; else `registry.set_phase(session_id, phase_lock_name(current))`.

`src/orchestrator/phases.py`:
- `Phase` enum: `CLASSIFY, KNOW, PLAN, BUILD, QUALITY, AUTOEVAL, MEMORY_OBSERVE` (value == name).
- `CANONICAL_SEQUENCE`: those 7 in that order.
- `phase_lock_name(phase)` → identity string.

---

## Task 1: Characterization tests for PhaseTracker and CanonicalLoop

Pins the current observable behavior of both phase systems. Purely additive — no source file changes, zero risk.

**Files:**
- Create: `tests/test_orchestrator_phases.py`

- [ ] **Step 1: Write the failing characterization tests**

Create `tests/test_orchestrator_phases.py` with exactly this content:

```python
"""Characterization tests for the live loop's two phase-inference systems.

These pin the CURRENT observable behavior of PhaseTracker and CanonicalLoop
(and, in Task 2, resolve_current_phase). They are behavior snapshots: if one
fails after a change, the change altered runtime behavior — suspect the change,
not the test.
"""
from __future__ import annotations

import pytest

from src.orchestrator.phase_tracker import PhaseTracker, tracker_enabled
from src.orchestrator.loop import CanonicalLoop
from src.orchestrator.phases import Phase


class _FakeRegistry:
    """Records set_phase calls so tests can assert on them without the singleton."""

    def __init__(self):
        self.calls = []

    def set_phase(self, session_id, phase):
        self.calls.append((session_id, phase))


# --- PhaseTracker -----------------------------------------------------------

def test_infer_phase_defaults_to_build():
    assert PhaseTracker.infer_phase(1, plan_mode=False) == "BUILD"
    assert PhaseTracker.infer_phase(99, plan_mode=False) == "BUILD"


def test_infer_phase_plan_mode_returns_plan():
    assert PhaseTracker.infer_phase(1, plan_mode=True) == "PLAN"


def test_tracker_enabled_defaults_off(monkeypatch):
    monkeypatch.delenv("ODYSSEUS_PHASE_TRACKER", raising=False)
    assert tracker_enabled() is False


def test_tracker_enabled_truthy_env(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_PHASE_TRACKER", "on")
    assert tracker_enabled() is True


def test_on_round_start_sets_phase_when_enabled():
    reg = _FakeRegistry()
    tracker = PhaseTracker("sess", registry=reg, enabled=True)
    tracker.on_round_start(1, plan_mode=True)
    assert reg.calls == [("sess", "PLAN")]


def test_on_round_start_noop_when_disabled():
    reg = _FakeRegistry()
    tracker = PhaseTracker("sess", registry=reg, enabled=False)
    tracker.on_round_start(1, plan_mode=True)
    assert reg.calls == []


# --- CanonicalLoop ----------------------------------------------------------

def test_canonical_loop_starts_at_classify():
    loop = CanonicalLoop("sess")
    assert loop.current == Phase.CLASSIFY


def test_canonical_loop_advances_to_memory_observe():
    loop = CanonicalLoop("sess")
    for _ in range(6):
        assert loop.advance() is True
    assert loop.current == Phase.MEMORY_OBSERVE


def test_canonical_loop_advance_past_end_is_bounded():
    loop = CanonicalLoop("sess")
    for _ in range(6):
        loop.advance()
    assert loop.advance() is False
    assert loop.current == Phase.MEMORY_OBSERVE


def test_canonical_loop_goto_applies_to_registry():
    reg = _FakeRegistry()
    loop = CanonicalLoop("sess", registry=reg)
    loop.goto(Phase.QUALITY)
    assert loop.current == Phase.QUALITY
    assert reg.calls == [("sess", "QUALITY")]


def test_canonical_loop_apply_without_registry_is_noop():
    loop = CanonicalLoop("sess")
    loop.apply()  # must not raise
```

- [ ] **Step 2: Run the tests to verify they PASS (characterization pins current behavior)**

Run: `rtk python -m pytest tests/test_orchestrator_phases.py -v`
Expected: all 11 tests PASS. (Unlike normal TDD, characterization tests describe code that already exists, so they pass immediately. If any fail, the reference behavior above was misread — fix the test to match the actual current behavior, do not change source.)

- [ ] **Step 3: Commit**

```bash
rtk git add tests/test_orchestrator_phases.py
rtk git commit -m "test(orchestrator): characterize PhaseTracker + CanonicalLoop behavior"
```

---

## Task 2: Extract the phase-resolution seam and wire it in

Create the pure resolver mirroring the inline block, test it, then replace the inline block with a single call. Extract-function-strict: same return for same inputs.

**Files:**
- Create: `src/orchestrator/phase_resolver.py`
- Modify: `src/agent_loop.py:2562-2572`
- Test: `tests/test_orchestrator_phases.py` (append resolver tests)

- [ ] **Step 1: Write the failing resolver tests**

Append these tests to the end of `tests/test_orchestrator_phases.py`:

```python
# --- resolve_current_phase (the extracted seam) -----------------------------

from src.orchestrator.phase_resolver import resolve_current_phase


class _FakeTracker:
    """Minimal PhaseTracker stand-in returning a fixed infer_phase result."""

    def __init__(self, phase_name):
        self._phase_name = phase_name

    def infer_phase(self, round_num, intent=None, plan_mode=False):
        return self._phase_name


def test_resolve_prefers_canonical_when_active():
    loop = CanonicalLoop("sess")
    loop.goto(Phase.PLAN)
    result = resolve_current_phase(True, loop, None, round_num=1)
    assert result == Phase.PLAN


def test_resolve_uses_tracker_plan_mode():
    tracker = _FakeTracker("PLAN")
    result = resolve_current_phase(False, None, tracker, round_num=1, plan_mode=True)
    assert result == Phase.PLAN


def test_resolve_uses_tracker_build_default():
    tracker = _FakeTracker("BUILD")
    result = resolve_current_phase(False, None, tracker, round_num=1, plan_mode=False)
    assert result == Phase.BUILD


def test_resolve_returns_none_when_no_system():
    result = resolve_current_phase(False, None, None, round_num=1)
    assert result is None


def test_resolve_coerces_out_of_enum_to_none():
    tracker = _FakeTracker("WAT")  # not a valid Phase value
    result = resolve_current_phase(False, None, tracker, round_num=1)
    assert result is None
```

- [ ] **Step 2: Run the resolver tests to verify they FAIL**

Run: `rtk python -m pytest tests/test_orchestrator_phases.py -k resolve -v`
Expected: collection error / FAIL — `ModuleNotFoundError: No module named 'src.orchestrator.phase_resolver'` (module does not exist yet).

- [ ] **Step 3: Create the pure resolver module**

Create `src/orchestrator/phase_resolver.py` with exactly this content:

```python
"""Pure resolution of the live loop's current canonical Phase.

Behavior-preserving extraction of the inline branch previously in
stream_agent_loop (src/agent_loop.py). No side effects: reads only. Returns
exactly the same Phase | None the inline block returned for the same inputs.
"""
from __future__ import annotations

from typing import Optional

from src.orchestrator.phases import Phase


def resolve_current_phase(use_canonical, canonical_loop, phase_tracker,
                          round_num, intent=None, plan_mode=False) -> Optional[Phase]:
    """Return the canonical Phase for this round, or None.

    Mirrors exactly the prior inline logic:
      - canonical active and present → the loop's current Phase;
      - else PhaseTracker present → its inferred phase name coerced to Phase
        (None if the name is not a valid Phase value);
      - else None.
    """
    if use_canonical and canonical_loop is not None:
        return canonical_loop.current
    if phase_tracker is not None:
        pt_phase = phase_tracker.infer_phase(round_num, intent=intent, plan_mode=plan_mode)
        try:
            return Phase(pt_phase) if pt_phase in {p.value for p in Phase} else None
        except Exception:
            return None
    return None
```

- [ ] **Step 4: Run the resolver tests to verify they PASS**

Run: `rtk python -m pytest tests/test_orchestrator_phases.py -k resolve -v`
Expected: all 5 `resolve_*` tests PASS.

- [ ] **Step 5: Replace the inline block in agent_loop.py with the seam call**

In `src/agent_loop.py`, replace the inline block (lines 2562-2572) — from `_current_phase = None` through the `except Exception:` / `_current_phase = None` — with this call. Keep the leading comment line and the surrounding lines (2561 comment above, the `if _agent_dispatcher ...` at 2573) unchanged.

Old (2562-2572):

```python
        _current_phase = None
        if _use_canonical and _canonical_loop is not None:
            _current_phase = _canonical_loop.current
        elif _phase_tracker is not None:
            # Map PhaseTracker phase name to canonical Phase enum
            _pt_phase = _phase_tracker.infer_phase(round_num, intent=_intent, plan_mode=plan_mode)
            try:
                from src.orchestrator.phases import Phase
                _current_phase = Phase(_pt_phase) if _pt_phase in {p.value for p in Phase} else None
            except Exception:
                _current_phase = None
```

New:

```python
        from src.orchestrator.phase_resolver import resolve_current_phase
        _current_phase = resolve_current_phase(
            _use_canonical, _canonical_loop, _phase_tracker,
            round_num, intent=_intent, plan_mode=plan_mode,
        )
```

- [ ] **Step 6: Verify the diff touches ONLY the extracted block**

Run: `rtk git diff src/agent_loop.py`
Expected: the ONLY change is the 11-line inline block replaced by the 5-line import+call above. No other line of the live loop is modified (the phase advance at 2554-2557 and the dispatch at 2573+ are untouched). If the diff shows anything else, revert and redo the replacement.

- [ ] **Step 7: Run the full orchestrator test file**

Run: `rtk python -m pytest tests/test_orchestrator_phases.py -v`
Expected: all 16 tests PASS (11 characterization + 5 resolver).

- [ ] **Step 8: Boot smoke-check — the app still imports cleanly**

Run: `rtk python -c "import app; print('boot ok')"`
Expected: prints `boot ok` with no traceback. (Confirms the agent_loop edit did not break import of the FastAPI app.)

- [ ] **Step 9: Commit**

```bash
rtk git add src/orchestrator/phase_resolver.py src/agent_loop.py tests/test_orchestrator_phases.py
rtk git commit -m "refactor(orchestrator): extract phase-resolution seam (behavior-preserving)"
```

---

## Self-Review (completed by plan author)

**1. Spec coverage:**
- Spec Unité 1 (characterization tests: PhaseTracker infer_phase/tracker_enabled/on_round_start, CanonicalLoop current/advance/goto/apply) → Task 1, all cases present. ✓
- Spec Unité 2 (pure `phase_resolver.py` + replace inline block + resolver tests incl. out-of-enum→None) → Task 2, all 5 resolver cases present (canonical-active, tracker plan/build, both-None, out-of-enum). ✓
- Spec "diff only touches 2562-2572" guard → Task 2 Step 6. ✓
- Spec "import app boots" non-regression → Task 2 Step 8. ✓
- Spec "no kill-switch default changed / no behavior change" → enforced by extract-function-strict replacement + diff guard; no task edits env defaults. ✓

**2. Placeholder scan:** No TBD/TODO/"handle edge cases"/vague steps. Every code step shows full code. ✓

**3. Type consistency:** `resolve_current_phase(use_canonical, canonical_loop, phase_tracker, round_num, intent=None, plan_mode=False)` signature identical in the resolver module (Task 2 Step 3), its tests (Step 1), and the call site (Step 5). `Phase`, `CanonicalLoop`, `_FakeRegistry`, `_FakeTracker` names consistent across both tasks. ✓
