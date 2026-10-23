"""Tests pour detect_orphans.py (P0-6)."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent.parent / "scripts" / "detect_orphans.py"
GATES = Path(__file__).resolve().parent.parent.parent / "gates.toml"


def test_detect_orphans_baseline():
    """Mode baseline : exit 0, JSON écrit."""
    r = subprocess.run(
        [sys.executable, str(SCRIPT)],
        capture_output=True, text=True,
    )
    assert r.returncode == 0, r.stderr
    data = json.loads(r.stdout)
    assert "orphelins" in data
    assert len(data["orphelins"]) >= 8


def test_detect_orphans_strict_names_fake():
    """Contrôle négatif : un plugin factice sans scénario ⇒ --strict échoue en le nommant."""
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        # gates.toml minimal avec un plugin factice
        gates = td / "gates.toml"
        gates.write_text(
            '[phase1]\n'
            '[[phase1.criteres]]\n'
            'id = "G1.d.plugins"\n'
            'plugins = { attendus = [\n'
            '  { id = "fake/test-plugin", capability = "test" },\n'
            '] }\n',
            encoding="utf-8",
        )
        scen = td / "scenarios"
        scen.mkdir()
        r = subprocess.run(
            [sys.executable, str(SCRIPT), "--gates", str(gates),
             "--scenarios", str(scen), "--strict", "--baselines", str(td / "baselines")],
            capture_output=True, text=True,
        )
        assert r.returncode == 1, f"attendu exit 1, obtenu {r.returncode}"
        combined = r.stdout + r.stderr
        assert "fake/test-plugin" in combined, "plugin factice non nommé"
