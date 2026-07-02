import pytest
from src.orchestrator.router_advice import advise, router_enabled


class _FakeRouter:
    def __init__(self, ret="openai/glm-5.2"):
        self.ret = ret
        self.calls = []

    def route(self, intent_category, stage=None):
        self.calls.append((intent_category, stage))
        return self.ret


class _BoomRouter:
    def route(self, intent_category, stage=None):
        raise RuntimeError("provider down")


def test_router_disabled_by_default(monkeypatch):
    monkeypatch.delenv("ODYSSEUS_MODEL_ROUTER", raising=False)
    assert router_enabled() is False


def test_router_enabled_when_on(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_MODEL_ROUTER", "on")
    assert router_enabled() is True


def test_router_enabled_accepts_truthy(monkeypatch):
    for val in ("1", "true", "yes", "ON"):
        monkeypatch.setenv("ODYSSEUS_MODEL_ROUTER", val)
        assert router_enabled() is True


def test_advise_deep_intent_and_router_called():
    r = _FakeRouter("openai/deepseek-v4-pro")
    out = advise("implement a distributed consensus algorithm with strong security", router=r)
    assert out["intent"] == "deep"
    assert out["suggested_model"] == "openai/deepseek-v4-pro"
    assert r.calls == [("deep", None)]


def test_advise_passes_stage_through():
    r = _FakeRouter()
    out = advise("hello", stage="planning", router=r)
    assert out["stage"] == "planning"
    assert r.calls == [(out["intent"], "planning")]


def test_advise_empty_text_is_utility():
    r = _FakeRouter()
    out = advise("", router=r)
    assert out["intent"] == "utility"


def test_advise_survives_router_exception():
    out = advise("anything", router=_BoomRouter())
    assert out["suggested_model"] is None
    assert out["intent"]  # still classified


def test_exported_from_package():
    from src.orchestrator import advise as a, router_enabled as re_
    assert a is advise
    assert re_ is router_enabled
