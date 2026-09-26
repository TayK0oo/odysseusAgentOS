import pathlib
import re

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


# ── the registry must not be able to lie ─────────────────────────────────
#
# Palier 0 exposed a real problem: 16 descriptors declared `default: "on"`
# while their reader fell back to `off`, and 9 more declared `on` with no
# reader anywhere. The dashboard therefore showed switches as ACTIVE that
# were, respectively, OFF or entirely unimplemented. "The cockpit reflects
# the state" is a Palier 0 precondition, so the registry is now policed by
# this test instead of by good intentions.

_REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
_SOURCE_DIRS = ("src", "core", "routes", "archive", "services", "mcp_servers")
_READER_RE = re.compile(
    r'(?:os\.environ\.get|os\.getenv)\(\s*"(ODYSSEUS_[A-Z0-9_]+)"\s*,\s*"([^"]*)"'
)

# Descriptors whose reader resolves the env var name at call time, so no
# literal `os.getenv("ODYSSEUS_...")` exists to match. Each one is bound to
# the module that reads it, and to the default that module applies.
_DYNAMIC_READERS = {
    "src/orchestrator/agent_dispatcher.py": "off",
}


def _real_readers() -> dict[str, set[str]]:
    """Map every env var name to the set of defaults its real readers use."""
    found: dict[str, set[str]] = {}
    for d in _SOURCE_DIRS:
        root = _REPO_ROOT / d
        if not root.is_dir():
            continue
        for f in root.rglob("*.py"):
            try:
                txt = f.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            for m in _READER_RE.finditer(txt):
                found.setdefault(m.group(1), set()).add(m.group(2))
    return found


def _normalise(default: str) -> str:
    """A reader falling back to "" is an OFF switch; say so plainly."""
    return "off" if default == "" else default


def test_every_wired_descriptor_has_a_real_reader():
    """`wired=True` must mean some code really reads the variable."""
    readers = _real_readers()
    for row in ksr.read_states():
        if not row["wired"]:
            continue
        if row["env_var"] in readers:
            continue
        dynamic = {src: d for src, d in _DYNAMIC_READERS.items() if src == row["source"]}
        assert dynamic, (
            f"{row['env_var']} is declared wired but nothing in the codebase reads it "
            f"(source: {row['source']}) — either wire it or flag wired=False"
        )


def test_unwired_descriptors_really_have_no_reader():
    """Conversely: `wired=False` must stay true, so the flag can't rot either."""
    readers = _real_readers()
    for row in ksr.read_states():
        if row["wired"]:
            continue
        assert row["env_var"] not in readers, (
            f"{row['env_var']} now has a real reader ({sorted(readers[row['env_var']])}) "
            f"— drop wired=False and set its default from the code"
        )


def test_descriptor_default_matches_the_real_reader():
    """The dashboard's headline number must be the one the code will use."""
    readers = _real_readers()
    lying = []
    for row in ksr.read_states():
        if not row["wired"] or row["env_var"] not in readers:
            continue
        declared = _normalise(row["default"])
        actual = {_normalise(d) for d in readers[row["env_var"]]}
        if declared not in actual:
            lying.append(f"{row['env_var']}: registre={declared} code={sorted(actual)}")
    assert lying == []


def test_no_switch_presents_itself_active_while_unwired():
    """The cockpit must not offer a switch that provably cannot do anything."""
    for row in ksr.read_states():
        if row["wired"]:
            continue
        assert row["effective"] is False, (
            f"{row['env_var']} is unwired yet reports effective=True — "
            f"the dashboard would be advertising an unreachable switch"
        )
