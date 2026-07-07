import pytest
from src import killswitch_registry as ksr


def test_unset_var_is_default_and_uses_code_default(monkeypatch):
    monkeypatch.delenv("ODYSSEUS_AUTOEVAL", raising=False)
    row = _find(ksr.read_states(), "ODYSSEUS_AUTOEVAL")
    assert row["is_default"] is True
    assert row["raw"] is None
    assert row["effective"] is False  # default "off"


def test_set_on_is_effective_true(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_AUTOEVAL", "on")
    row = _find(ksr.read_states(), "ODYSSEUS_AUTOEVAL")
    assert row["is_default"] is False
    assert row["raw"] == "on"
    assert row["effective"] is True


def test_set_off_is_effective_false(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_LIVE_ORCHESTRATION", "off")
    row = _find(ksr.read_states(), "ODYSSEUS_LIVE_ORCHESTRATION")
    assert row["effective"] is False
    assert row["is_default"] is False


def test_destructive_gate_defaults_on(monkeypatch):
    monkeypatch.delenv("ODYSSEUS_DESTRUCTIVE_GATE", raising=False)
    row = _find(ksr.read_states(), "ODYSSEUS_DESTRUCTIVE_GATE")
    assert row["is_default"] is True
    assert row["effective"] is True  # default "on"


def test_every_descriptor_has_required_keys():
    required = {"name", "env_var", "default", "category", "timing", "desc", "source"}
    for row in ksr.read_states():
        assert required.issubset(row.keys()), row
        assert row["source"], f"empty source for {row['env_var']}"
        assert row["timing"] in ("startup", "runtime")


def test_categories_cover_all_switches():
    cats = set(ksr.categories())
    for row in ksr.read_states():
        assert row["category"] in cats


def _find(rows, env_var):
    for r in rows:
        if r["env_var"] == env_var:
            return r
    raise AssertionError(f"{env_var} not in registry")
