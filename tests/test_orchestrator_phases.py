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
