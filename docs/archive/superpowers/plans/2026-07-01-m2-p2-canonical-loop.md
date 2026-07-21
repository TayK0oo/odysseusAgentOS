# M2-P2 — Canonical 7-Phase Loop + Active Phase-Lock Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the canonical 7-phase loop as data (`phases.py`) and a state machine (`loop.py`) that drives the existing (but currently inert) `ToolRegistry` phase-lock — proving that advancing the loop actually changes which tools are allowed per phase.

**Architecture:** `phases.py` declares the ordered canonical phases (CLASSIFY → KNOW → PLAN → BUILD → QUALITY → AUTOEVAL → MEMORY_OBSERVE), each mapped to a phase-lock enforcement name plus forced tools. `loop.py` is a `CanonicalLoop` that tracks the current phase, advances through the sequence, and calls `ToolRegistry.set_phase(session_id, phase)` so enforcement (already wired in `tool_execution.py:582`) takes effect. `config/phase-lock.yaml` gains the 5 new canonical phase entries (CLASSIFY, KNOW, QUALITY, AUTOEVAL, MEMORY_OBSERVE) reusing the existing restriction shapes; PLAN and BUILD already exist.

**Tech Stack:** Python 3.12 `enum`, existing `src/tool_registry.py::ToolRegistry`, pytest. No network.

**Reference:** Design spec Section 2 (`docs/superpowers/specs/2026-07-01-odysseus-integration-milestone2-design.md`). Tracking Bloc A/B. Enforcement call-site already exists: `src/tool_execution.py:582` calls `ToolRegistry.get_instance().is_tool_allowed(...)`; the gap is that `set_phase` is never called (phase always defaults to BUILD).

---

## File Structure

- Create: `src/orchestrator/phases.py` — `Phase` enum + `CANONICAL_SEQUENCE` + `PHASE_LOCK_NAME` + `FORCED_TOOLS`
- Create: `src/orchestrator/loop.py` — `CanonicalLoop` state machine
- Modify: `config/phase-lock.yaml` — add CLASSIFY, KNOW, QUALITY, AUTOEVAL, MEMORY_OBSERVE entries
- Modify: `src/orchestrator/__init__.py` — export `Phase`, `CanonicalLoop`
- Test: `tests/test_orchestrator_phases.py`
- Test: `tests/test_orchestrator_loop.py`
- Test: `tests/test_orchestrator_phaselock_integration.py`

---

## Task 1: Canonical phase definitions (`phases.py`)

**Files:**
- Create: `src/orchestrator/phases.py`
- Test: `tests/test_orchestrator_phases.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_orchestrator_phases.py
"""Tests for src/orchestrator/phases.py — canonical phase definitions."""
from src.orchestrator.phases import (
    Phase,
    CANONICAL_SEQUENCE,
    phase_lock_name,
    forced_tools,
)


def test_seven_canonical_phases_in_order():
    assert [p.name for p in CANONICAL_SEQUENCE] == [
        "CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE",
    ]


def test_each_phase_maps_to_a_phaselock_name():
    for p in CANONICAL_SEQUENCE:
        name = phase_lock_name(p)
        assert isinstance(name, str) and name
        # The phase-lock name is an uppercase token usable as a YAML key.
        assert name == name.upper()


def test_classify_maps_to_readonly_enforcement():
    # CLASSIFY must not allow writes/exec — it maps to the read-only RESEARCH lock.
    assert phase_lock_name(Phase.CLASSIFY) == "CLASSIFY"


def test_build_maps_to_build():
    assert phase_lock_name(Phase.BUILD) == "BUILD"


def test_forced_tools_are_lists():
    for p in CANONICAL_SEQUENCE:
        assert isinstance(forced_tools(p), list)


def test_classify_forces_risk_classifier():
    assert "risk_classifier" in forced_tools(Phase.CLASSIFY)
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python -m pytest tests/test_orchestrator_phases.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.orchestrator.phases'`

- [ ] **Step 3: Write the implementation**

```python
# src/orchestrator/phases.py
"""Canonical 7-phase loop definitions (Milestone 2, design Section 2).

Each canonical phase maps to a phase-lock enforcement name (a key in
config/phase-lock.yaml) and a set of "forced" tools the loop should encourage
in that phase. The enforcement itself lives in src/tool_registry.py; this module
is pure data + helpers.
"""
from __future__ import annotations

from enum import Enum
from typing import Dict, List


class Phase(Enum):
    CLASSIFY = "CLASSIFY"
    KNOW = "KNOW"
    PLAN = "PLAN"
    BUILD = "BUILD"
    QUALITY = "QUALITY"
    AUTOEVAL = "AUTOEVAL"
    MEMORY_OBSERVE = "MEMORY_OBSERVE"


# Ordered sequence the loop walks through.
CANONICAL_SEQUENCE: List[Phase] = [
    Phase.CLASSIFY,
    Phase.KNOW,
    Phase.PLAN,
    Phase.BUILD,
    Phase.QUALITY,
    Phase.AUTOEVAL,
    Phase.MEMORY_OBSERVE,
]

# Map each canonical phase to the phase-lock.yaml enforcement key.
# The canonical names are used directly as YAML keys (added in this phase's
# Task 3), so the mapping is identity — but kept explicit so a future rename of
# an enforcement profile does not silently break the loop.
_PHASE_LOCK_NAME: Dict[Phase, str] = {
    Phase.CLASSIFY: "CLASSIFY",
    Phase.KNOW: "KNOW",
    Phase.PLAN: "PLAN",
    Phase.BUILD: "BUILD",
    Phase.QUALITY: "QUALITY",
    Phase.AUTOEVAL: "AUTOEVAL",
    Phase.MEMORY_OBSERVE: "MEMORY_OBSERVE",
}

# Tools the loop should force/encourage per phase (advisory; enforcement of
# BLOCKED tools is done by the phase-lock config).
_FORCED_TOOLS: Dict[Phase, List[str]] = {
    Phase.CLASSIFY: ["risk_classifier"],
    Phase.KNOW: ["memory_search"],
    Phase.PLAN: [],
    Phase.BUILD: [],
    Phase.QUALITY: [],
    Phase.AUTOEVAL: ["autoeval"],
    Phase.MEMORY_OBSERVE: ["memory_distill", "trace_writer"],
}


def phase_lock_name(phase: Phase) -> str:
    return _PHASE_LOCK_NAME[phase]


def forced_tools(phase: Phase) -> List[str]:
    return list(_FORCED_TOOLS.get(phase, []))
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_orchestrator_phases.py -v`
Expected: PASS (6 passed)

- [ ] **Step 5: Commit**

```bash
git add src/orchestrator/phases.py tests/test_orchestrator_phases.py
git commit -m "feat(orchestrator): canonical 7-phase definitions"
```

---

## Task 2: `CanonicalLoop` state machine (`loop.py`)

**Files:**
- Create: `src/orchestrator/loop.py`
- Modify: `src/orchestrator/__init__.py`
- Test: `tests/test_orchestrator_loop.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_orchestrator_loop.py
"""Tests for src/orchestrator/loop.py — CanonicalLoop state machine."""
import pytest
from src.orchestrator.phases import Phase
from src.orchestrator.loop import CanonicalLoop


class _FakeRegistry:
    def __init__(self):
        self.calls = []

    def set_phase(self, session_id, phase):
        self.calls.append((session_id, phase))


def test_starts_at_classify():
    loop = CanonicalLoop(session_id="s1")
    assert loop.current is Phase.CLASSIFY


def test_advance_walks_the_sequence():
    loop = CanonicalLoop(session_id="s1")
    seen = [loop.current]
    while loop.advance():
        seen.append(loop.current)
    assert [p.name for p in seen] == [
        "CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE",
    ]


def test_advance_returns_false_at_end():
    loop = CanonicalLoop(session_id="s1")
    for _ in range(6):
        assert loop.advance() is True
    assert loop.current is Phase.MEMORY_OBSERVE
    assert loop.advance() is False
    assert loop.current is Phase.MEMORY_OBSERVE  # stays put


def test_apply_calls_registry_set_phase_with_lock_name():
    reg = _FakeRegistry()
    loop = CanonicalLoop(session_id="s1", registry=reg)
    loop.apply()  # applies current (CLASSIFY)
    loop.advance()
    loop.apply()  # applies KNOW
    assert reg.calls == [("s1", "CLASSIFY"), ("s1", "KNOW")]


def test_apply_without_registry_is_noop():
    loop = CanonicalLoop(session_id="s1", registry=None)
    loop.apply()  # must not raise
    assert loop.current is Phase.CLASSIFY


def test_goto_jumps_to_named_phase_and_applies():
    reg = _FakeRegistry()
    loop = CanonicalLoop(session_id="s1", registry=reg)
    loop.goto(Phase.BUILD)
    assert loop.current is Phase.BUILD
    assert reg.calls == [("s1", "BUILD")]
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python -m pytest tests/test_orchestrator_loop.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.orchestrator.loop'`

- [ ] **Step 3: Write the implementation**

```python
# src/orchestrator/loop.py
"""CanonicalLoop — walks the 7 canonical phases and drives the phase-lock.

The loop's only side effect is calling registry.set_phase(session_id, name)
so the already-wired enforcement in tool_execution.py takes effect. If no
registry is injected, apply() is a no-op (useful for pure unit tests).
"""
from __future__ import annotations

import logging
from typing import Optional

from src.orchestrator.phases import (
    Phase,
    CANONICAL_SEQUENCE,
    phase_lock_name,
)

logger = logging.getLogger(__name__)


class CanonicalLoop:
    def __init__(self, session_id: str, registry=None):
        self.session_id = session_id
        self._registry = registry
        self._index = 0

    @property
    def current(self) -> Phase:
        return CANONICAL_SEQUENCE[self._index]

    def advance(self) -> bool:
        """Move to the next phase. Returns False if already at the last phase."""
        if self._index >= len(CANONICAL_SEQUENCE) - 1:
            return False
        self._index += 1
        return True

    def goto(self, phase: Phase) -> None:
        """Jump directly to a named phase and apply it."""
        self._index = CANONICAL_SEQUENCE.index(phase)
        self.apply()

    def apply(self) -> None:
        """Push the current phase into the phase-lock registry."""
        if self._registry is None:
            return
        name = phase_lock_name(self.current)
        self._registry.set_phase(self.session_id, name)
        logger.info("[CanonicalLoop] session=%s → phase %s", self.session_id, name)
```

Update exports:

```python
# src/orchestrator/__init__.py
"""Native orchestration layer for Odysseus (Milestone 2)."""
from src.orchestrator.spec import AgentSpec, parse_agent_spec
from src.orchestrator.registry import AgentRegistry
from src.orchestrator.dispatcher import resolve_model
from src.orchestrator.phases import Phase, CANONICAL_SEQUENCE, phase_lock_name, forced_tools
from src.orchestrator.loop import CanonicalLoop

__all__ = [
    "AgentSpec", "parse_agent_spec", "AgentRegistry", "resolve_model",
    "Phase", "CANONICAL_SEQUENCE", "phase_lock_name", "forced_tools", "CanonicalLoop",
]
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_orchestrator_loop.py -v`
Expected: PASS (6 passed)

- [ ] **Step 5: Commit**

```bash
git add src/orchestrator/loop.py src/orchestrator/__init__.py tests/test_orchestrator_loop.py
git commit -m "feat(orchestrator): CanonicalLoop drives phase-lock set_phase"
```

---

## Task 3: Add canonical phases to `phase-lock.yaml`

**Files:**
- Modify: `config/phase-lock.yaml`

The existing file has RESEARCH, INNOVATE, PLAN, BUILD, VERIFY. Add the 5 missing canonical
names so `set_phase("CLASSIFY"|"KNOW"|"QUALITY"|"AUTOEVAL"|"MEMORY_OBSERVE")` resolves to a real
enforcement profile. PLAN and BUILD already exist and are reused as-is.

- [ ] **Step 1: Append the new phase entries**

Insert these blocks under `phases:` in `config/phase-lock.yaml`, immediately before the
`default_phase: BUILD` line (indentation: 2 spaces, matching existing entries):

```yaml
  CLASSIFY:
    description: "Classification du risque — lecture seule, aucune écriture ni exécution"
    allowed_categories: [read, search, knowledge]
    blocked_categories: [write, exec, destructive]
    blocked_tools:
      - write_file
      - edit_file
      - create_file
      - bash
      - run_command
      - delete_file
    message: "Phase CLASSIFY : classification du risque, écriture et exécution désactivées"

  KNOW:
    description: "Récupération mémoire/connaissance — lecture et recherche uniquement"
    allowed_categories: [read, search, knowledge]
    blocked_categories: [write, exec, destructive]
    blocked_tools:
      - write_file
      - edit_file
      - create_file
      - bash
      - run_command
      - delete_file
    message: "Phase KNOW : récupération de connaissance, écriture et exécution désactivées"

  QUALITY:
    description: "Qualité — lecture et commandes de test uniquement"
    allowed_categories: [read, search, knowledge, exec]
    blocked_categories: [write, destructive]
    exec_restriction:
      allowed_commands_patterns:
        - "pytest"
        - "npm test"
        - "cargo test"
        - "go test"
        - "jest"
        - "vitest"
        - "playwright"
        - "lint"
      message: "Phase QUALITY : seules les commandes de test/lint sont autorisées"
    blocked_tools:
      - delete_file
    message: "Phase QUALITY : tests et lint uniquement"

  AUTOEVAL:
    description: "Auto-évaluation — lecture et exécution d'évaluation, pas de destruction"
    allowed_categories: [read, search, knowledge, exec]
    blocked_categories: [destructive]
    blocked_tools:
      - delete_file
    message: "Phase AUTOEVAL : auto-évaluation, destruction désactivée"

  MEMORY_OBSERVE:
    description: "Distillation mémoire et observabilité — écriture mémoire, pas de destruction"
    allowed_categories: [read, search, knowledge, write]
    blocked_categories: [destructive]
    blocked_tools:
      - delete_file
    message: "Phase MEMORY_OBSERVE : distillation et observation, destruction désactivée"
```

- [ ] **Step 2: Validate the YAML parses**

Run: `python -c "import yaml; d=yaml.safe_load(open('config/phase-lock.yaml')); print(sorted(d['phases'].keys()))"`
Expected: a list containing `AUTOEVAL, BUILD, CLASSIFY, INNOVATE, KNOW, MEMORY_OBSERVE, PLAN, QUALITY, RESEARCH, VERIFY`

- [ ] **Step 3: Commit**

```bash
git add config/phase-lock.yaml
git commit -m "feat(phase-lock): add canonical CLASSIFY/KNOW/QUALITY/AUTOEVAL/MEMORY_OBSERVE phases"
```

---

## Task 4: Integration test — driving the loop actually gates tools

**Files:**
- Test: `tests/test_orchestrator_phaselock_integration.py`

This proves the whole point of M2-P2: advancing the `CanonicalLoop` through a **real**
`ToolRegistry` (loaded from the real `config/phase-lock.yaml`) changes what
`is_tool_allowed` returns. This is the integration the audit said was missing.

- [ ] **Step 1: Write the test**

```python
# tests/test_orchestrator_phaselock_integration.py
"""Integration: CanonicalLoop + real ToolRegistry gates tools per phase."""
from src.tool_registry import ToolRegistry
from src.orchestrator.phases import Phase
from src.orchestrator.loop import CanonicalLoop


def _registry():
    # Fresh registry bound to the real project config (not the singleton, to
    # avoid cross-test contamination of session phases).
    return ToolRegistry(config_path="config/phase-lock.yaml")


def test_classify_blocks_writes_build_allows_them():
    reg = _registry()
    loop = CanonicalLoop(session_id="itest", registry=reg)

    # CLASSIFY: write_file must be blocked.
    loop.goto(Phase.CLASSIFY)
    res = reg.is_tool_allowed("write_file", "itest")
    assert res["allowed"] is False
    assert res["phase"] == "CLASSIFY"

    # BUILD: write_file must be allowed.
    loop.goto(Phase.BUILD)
    res = reg.is_tool_allowed("write_file", "itest")
    assert res["allowed"] is True
    assert res["phase"] == "BUILD"


def test_quality_allows_pytest_blocks_arbitrary_bash():
    reg = _registry()
    loop = CanonicalLoop(session_id="itest2", registry=reg)
    loop.goto(Phase.QUALITY)

    ok = reg.is_tool_allowed("bash", "itest2", {"command": "pytest tests/"})
    assert ok["allowed"] is True

    blocked = reg.is_tool_allowed("bash", "itest2", {"command": "rm -rf build"})
    assert blocked["allowed"] is False


def test_full_walk_sets_phase_each_step():
    reg = _registry()
    loop = CanonicalLoop(session_id="itest3", registry=reg)
    loop.apply()
    phases_seen = [reg.get_phase("itest3")]
    while loop.advance():
        loop.apply()
        phases_seen.append(reg.get_phase("itest3"))
    assert phases_seen == [
        "CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE",
    ]
```

- [ ] **Step 2: Run the integration test**

Run: `python -m pytest tests/test_orchestrator_phaselock_integration.py -v`
Expected: PASS (3 passed)

Note: `is_tool_allowed` uses `tool_args.get("command")` for bash exec_restriction (see
`src/tool_registry.py:93-103`), which is why the QUALITY test passes `{"command": ...}`.

- [ ] **Step 3: Run the whole orchestrator suite**

Run: `python -m pytest tests/test_orchestrator_*.py -v`
Expected: PASS (all orchestrator tests green — 16 from P1 + phases 6 + loop 6 + integration 3 = 31)

- [ ] **Step 4: Commit**

```bash
git add tests/test_orchestrator_phaselock_integration.py
git commit -m "test(orchestrator): loop drives real phase-lock gating (integration)"
```

---

## Task 5: Update tracking doc

**Files:**
- Modify: `.planning/INTEGRATION-TRACKING.md`

- [ ] **Step 1:** Add a "Bloc 0b — Loop canonique + phase-lock actif (M2-P2)" note under the tracker
recording: 7-phase sequence defined, `CanonicalLoop` drives `ToolRegistry.set_phase`, canonical
phases added to `phase-lock.yaml`, integration test proves per-phase gating. Note the phase-lock
can now move to 🟢 **when the live agent_loop calls `CanonicalLoop.apply()`** (wiring lands in M2-P3).

- [ ] **Step 2: Commit**

```bash
git add .planning/INTEGRATION-TRACKING.md
git commit -m "docs(tracking): M2-P2 canonical loop + active phase-lock delivered"
```

---

## Self-Review

**Spec coverage (Section 2):** 7-phase loop with per-phase tool gating → Tasks 1–4 ✅. The design's
"forced/blocked tools per phase" is realised as `forced_tools()` (advisory) + phase-lock `blocked_tools`
(enforced). Live wiring into `agent_loop.py` is explicitly deferred to M2-P3 (matches roadmap).

**Placeholder scan:** No TBD/TODO; all code complete. ✅

**Type consistency:** `Phase` enum members (CLASSIFY…MEMORY_OBSERVE) identical across `phases.py`,
`loop.py`, and all tests. `CanonicalLoop(session_id, registry)` + methods `current`/`advance`/`goto`/
`apply` match their tests. `phase_lock_name()` returns the exact YAML keys added in Task 3. ✅

**Scope:** Pure + one real-config integration test; no edits to the 216KB `agent_loop.py`. Produces
working, verifiable software (the loop provably gates tools). ✅
