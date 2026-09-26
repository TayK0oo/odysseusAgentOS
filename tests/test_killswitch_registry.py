from src import killswitch_registry as ksr


def test_unset_var_is_default_and_uses_code_default(monkeypatch):
    """An unset var falls back to the descriptor default.

    AUTOEVAL's descriptor default flipped to "on" with Palier 0, so an unset
    ODYSSEUS_AUTOEVAL now resolves to True. The OFF polarity is asserted
    through the dedicated ALLOW_RESET descriptor right below, which must stay
    "off" — that separation is the safety property, not a detail.
    """
    monkeypatch.delenv("ODYSSEUS_AUTOEVAL", raising=False)
    row = _find(ksr.read_states(), "ODYSSEUS_AUTOEVAL")
    assert row["is_default"] is True
    assert row["raw"] is None
    assert row["effective"] is True  # Palier 0 default


def test_destructive_revert_gate_stays_off_by_default(monkeypatch):
    """The cockpit must never present destructive auto-revert as available."""
    monkeypatch.delenv("ODYSSEUS_AUTOEVAL_ALLOW_RESET", raising=False)
    row = _find(ksr.read_states(), "ODYSSEUS_AUTOEVAL_ALLOW_RESET")
    assert row["is_default"] is True
    assert row["default"] == "off"
    assert row["effective"] is False


def test_all_p0_switches_are_registered():
    """Every Palier 0 switch must be visible in the registry.

    Six of them (OUTPUT_ROUTER, PREFERENCES, DATA_CLASSIFICATION,
    CONTENT_SECURITY, DURABLE_EXECUTION, PROVENANCE_MEMORY) were ON in
    .env.example but absent from the registry, so the cockpit could not
    reflect a state it did not even expose.
    """
    env_vars = {r["env_var"] for r in ksr.read_states()}
    p0 = [
        "PHASE_TRACKER", "MODEL_ROUTER", "PROGRESSIVE_DISCLOSURE", "MEMORY_IMPACT",
        "UNIFIED_TOKENS", "OUTPUT_ROUTER", "PREFERENCES", "DATA_CLASSIFICATION",
        "CONTENT_SECURITY", "DURABLE_EXECUTION", "PROVENANCE_MEMORY",
        "AUTOEVAL", "AUTOEVOLVE", "CHECKPOINT",
    ]
    missing = [f"ODYSSEUS_{s}" for s in p0 if f"ODYSSEUS_{s}" not in env_vars]
    assert missing == []


def test_p0_switches_default_to_on():
    """Palier 0 switches must be active by default in the shipped descriptors."""
    rows = {r["env_var"]: r for r in ksr.read_states()}
    p0 = [
        "PHASE_TRACKER", "MODEL_ROUTER", "PROGRESSIVE_DISCLOSURE", "MEMORY_IMPACT",
        "UNIFIED_TOKENS", "OUTPUT_ROUTER", "PREFERENCES", "DATA_CLASSIFICATION",
        "CONTENT_SECURITY", "DURABLE_EXECUTION", "PROVENANCE_MEMORY",
        "AUTOEVAL", "AUTOEVOLVE", "CHECKPOINT",
    ]
    wrong = [f"ODYSSEUS_{s}" for s in p0 if rows[f"ODYSSEUS_{s}"]["default"] != "on"]
    assert wrong == []


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
