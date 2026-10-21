"""P1 (fin A1) — controles de la porte G1 outillee (ADR-0013).

Deux familles :
  1. structure de [phase1] dans gates.toml : plus aucun placeholder `ci`
     (faux vert mort), criteres executables `command`, expect_exit 0 (jamais
     de vert par inversion — sinon un rouge present deviendrait vert), liste
     nominative des plugins coherente avec composition.rs (production).
  2. semantique de verdict du runner (`verdict_exit`, ADR-0013 decision 2) :
     un rouge present (XFAIL) ne peut pas faire VERT un critere qui attend
     un PASS sec — controle negatif du controle (plan §7, lecon L4).
"""

from __future__ import annotations

import importlib.util
import re
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
GATE = ROOT / "omega" / "scripts" / "gate.py"
RUNNER = ROOT / "omega" / "acceptance" / "runner.py"
COMPOSITION = ROOT / "omega" / "crates" / "omega-adapters" / "src" / "composition.rs"


def _module(nom: str, chemin: Path):
    spec = importlib.util.spec_from_file_location(nom, chemin)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# --- 1. structure [phase1] ---------------------------------------------------


def _phase1():
    gate = _module("gate_phase1", GATE)
    return gate.charger()["phase1"]


def test_phase1_plus_aucun_critere_placeholder_ci():
    criteres = _phase1()["criteres"]
    ids = {c["id"] for c in criteres}
    assert ids == {"G1.a.s1", "G1.b.s2", "G1.c.sequence", "G1.d.plugins"}
    for c in criteres:
        assert c["kind"] == "command", "le faux vert DELEGUE_CI est mort (ADR-0013)"
        assert c.get("bloquant", True) is True


def test_phase1_pas_de_vert_par_inversion():
    # un critere qui attend un echec (expect_exit != 0) transforme un rouge
    # present en vert : interdit ici. Le rougement de G1 se verra dans la
    # porte (ECHEC), jamais numerologiquement inverti.
    for c in _phase1()["criteres"]:
        assert c["expect_exit"] == 0


def test_phase1_criteres_scriptes_bornes():
    specs = {
        "G1.a.s1": ["--scenario", "S1"],
        "G1.b.s2": ["--scenario", "S2"],
        "G1.c.sequence": ["--scenario", "S0,S1,S2"],
    }
    for c in _phase1()["criteres"]:
        attendu = specs.get(c["id"])
        if attendu is None:
            continue
        for motif in (*attendu, "--mode", "scripte", "--runs", "5",
                      "--attendu", "PASS"):
            assert motif in c["argv"], (c["id"], motif)
        assert c["timeout"] > 0, "borne explicite dans la porte (ADR-0013)"


def _registre_production_de_composition_rs() -> set[tuple[str, str]]:
    """Couples (id, capability) de production, meme methode que measures_advanced
    (parse de `register("id", "capability")`), mais SANS le bruit des modules
    de test (#[cfg(test)]) qui gonflait STATUS.json a un faux 13."""
    code = COMPOSITION.read_text(encoding="utf-8").split("#[cfg(test)]")[0]
    return set(re.findall(r'register\(\s*"([^"]+)"\s*,\s*"([^"]+)"', code, re.S))


def test_phase1_liste_nominative_plugins():
    critere = next(c for c in _phase1()["criteres"] if c["id"] == "G1.d.plugins")
    attendus = [(p["id"], p["capability"]) for p in critere["plugins"]["attendus"]]
    assert len(attendus) == len(set(attendus)), "couples uniques"
    production = _registre_production_de_composition_rs()
    assert production, "composition.rs doit contenir des adaptateurs de production"
    for couple in production:
        assert couple in attendus, f"adaptateur {couple} absent de la porte"
    verif = ROOT / "omega" / "acceptance" / "verifier_plugins.py"
    assert verif.is_file(), "le critere pointe le controle reel"
    assert critere["argv"][1].endswith("omega/acceptance/verifier_plugins.py")


# --- 2. semantique de verdict du runner (verdict_exit) -----------------------


@pytest.fixture(scope="module")
def runner():
    return _module("acc_runner", RUNNER)


def test_sans_attendu_l_accord_p0_est_inchange(runner):
    assert runner.verdict_exit(["PASS", "XFAIL"], None) == 0
    assert runner.verdict_exit(["PASS"], None) == 0
    assert runner.verdict_exit(["FAIL"], None) == 1
    assert runner.verdict_exit(["XPASS"], None) == 1
    assert runner.verdict_exit(["ERREUR"], None) == 1
    assert runner.verdict_exit([], None) == 1


def test_attendu_refuse_le_second_degre_xfail(runner):
    # le rouge present (XFAIL) ne fait pas VERT un critere PASS sec
    assert runner.verdict_exit(["XFAIL"] * 5, "PASS") == 1


def test_attendu_exige_tous_les_runs_au_verdict_attendu(runner):
    assert runner.verdict_exit(["PASS"] * 5, "PASS") == 0
    assert runner.verdict_exit(["PASS"] * 4 + ["XFAIL"], "PASS") == 1
    assert runner.verdict_exit(["FAIL"] * 5, "PASS") == 1


def test_xpass_reste_un_bug_de_scenario_attendu_rouge(runner):
    # les attendus observes alors que xfail_raison est encore la : le
    # scenario ment, doit rester rouge, meme sous --attendu PASS.
    assert runner.verdict_exit(["XPASS"] * 2, "PASS") == 1


def test_main_sans_attendu_garde_l_accord_p0(runner, monkeypatch):
    monkeypatch.setattr(runner, "run_scenario",
                        lambda *a, **k: {"id": "S1", "verdict": "XFAIL"})
    with pytest.raises(SystemExit) as e:
        runner.main(["--scenario", "S1"])
    assert e.value.code == 0  # le critere G0.c.s1 de phase 0 ne change pas de sens


def test_main_attendu_refuse_un_verdict_verte_bugge(runner, monkeypatch):
    # controle negatif : un critere --attendu PASS sur un scenario encore
    # rouge (XFAIL) doit cesser l'exit 1 — jamais l'inverse.
    monkeypatch.setattr(runner, "run_scenario",
                        lambda *a, **k: {"id": "S1", "verdict": "XFAIL"})
    with pytest.raises(SystemExit) as e:
        runner.main(["--scenario", "S1", "--runs", "5", "--attendu", "PASS"])
    assert e.value.code == 1


def test_main_attendu_xpass_est_refuse(runner, monkeypatch):
    monkeypatch.setattr(runner, "run_scenario",
                        lambda *a, **k: {"id": "S2", "verdict": "XPASS"})
    with pytest.raises(SystemExit) as e:
        runner.main(["--scenario", "S2", "--attendu", "PASS"])
    assert e.value.code == 1
