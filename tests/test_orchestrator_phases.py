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
        assert name == name.upper()


def test_classify_maps_to_readonly_enforcement():
    assert phase_lock_name(Phase.CLASSIFY) == "CLASSIFY"


def test_build_maps_to_build():
    assert phase_lock_name(Phase.BUILD) == "BUILD"


def test_forced_tools_are_lists():
    for p in CANONICAL_SEQUENCE:
        assert isinstance(forced_tools(p), list)


def test_classify_forces_risk_classifier():
    assert "risk_classifier" in forced_tools(Phase.CLASSIFY)
