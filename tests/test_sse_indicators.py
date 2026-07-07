import json
from src.sse_indicators import (
    normalize_drift,
    run_status_event,
    autoeval_event,
    verifier_event,
)


def test_normalize_drift_from_enum_like():
    class _D:
        value = "HIGH"
    assert normalize_drift(_D()) == "high"


def test_normalize_drift_from_string():
    assert normalize_drift("Med") == "med"


def test_normalize_drift_none_and_unknown():
    assert normalize_drift(None) is None
    assert normalize_drift("garbage") is None


def test_run_status_event_shape():
    evt = run_status_event(phase="build", drift=None, used=3, max_rounds=12,
                           budget_pct=42, budget_tokens=18450)
    assert evt == {
        "type": "run_status",
        "phase": "build",
        "drift": None,
        "iters": {"used": 3, "max": 12},
        "budget": {"pct": 42, "tokens": 18450},
    }
    json.dumps(evt)


def test_run_status_event_nulls_when_unknown():
    evt = run_status_event(phase=None, drift="low", used=1, max_rounds=8,
                           budget_pct=None, budget_tokens=None)
    assert evt["phase"] is None
    assert evt["budget"] is None
    assert evt["drift"] == "low"


def test_autoeval_event_shape():
    evt = autoeval_event(decision="revert", reason="drift HIGH")
    assert evt == {"type": "autoeval_result", "decision": "revert", "reason": "drift HIGH"}
    json.dumps(evt)


def test_verifier_event_pass_and_fail():
    assert verifier_event(reasons=None) == {
        "type": "verifier_result", "status": "pass", "detail": ""}
    evt = verifier_event(reasons=["missing test", "lint fail", "x", "y"])
    assert evt["status"] == "fail"
    assert evt["detail"] == "missing test; lint fail; x"
    json.dumps(evt)
