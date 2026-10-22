"""P0-5 — controle du runner de portes (`omega/scripts/gate.py`).

Le controle **negatif** est obligatoire (plan §7, leçon L2) : un critère qui
doit échouer DOIT faire échouer la porte. Sans lui, `gate.py` serait un écho
vert — exactement le défaut que le projet s'est reproché.

Les tests ne dépendent d'aucun état du dépôt hors `omega/gates.toml` : ils
portent sur la logique (verdict, timeout, delegation CI, code de retour).
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "omega" / "scripts" / "gate.py"


def _charger():
    spec = importlib.util.spec_from_file_location("gate", SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


gate = _charger()


def test_gates_toml_couvre_les_phases_0_a_7_et_G0():
    gates = gate.charger()
    assert gate.phases(gates) == [f"phase{i}" for i in range(8)]
    ids = [c["id"] for c in gates["phase0"]["criteres"]]
    # `G0.c.s1` a été retiré : le critère affirmait « S1 rouge sur l'état
    # actuel », devenu faux une fois la boucle d'outils livrée (ADR-0015).
    # S1 est désormais couvert par `G1.a.s1` en phase 1.
    for attendu in ("G0.a", "G0.b", "G0.c.s0", "G0.d", "G0.e"):
        assert attendu in ids
    assert "G0.c.s1" not in ids, "critère « S1 rouge » retiré (ADR-0015)"


def test_commande_qui_sort_du_code_attendu_est_echec():
    critere = {
        "id": "X", "kind": "command", "bloquant": True,
        "argv": ["python3", "-c", "raise SystemExit(3)"], "expect_exit": 0,
    }
    res = gate.executer_critere(critere, ROOT)
    assert res["status"] == gate.ECHEC
    assert "3" in res["sortie"]


def test_commande_qui_sort_du_code_attendu_est_verte():
    critere = {
        "id": "X", "kind": "command", "bloquant": True,
        "argv": ["python3", "-c", "pass"], "expect_exit": 0,
    }
    res = gate.executer_critere(critere, ROOT)
    assert res["status"] == gate.VERT


def test_fichier_absent_est_echec():
    critere = {
        "id": "X", "kind": "file", "bloquant": True,
        "path": "docs/omega/adr/zzz-inexistant-*.md",
    }
    res = gate.executer_critere(critere, ROOT)
    assert res["status"] == gate.ECHEC


def test_critere_ci_est_delegue_et_ne_ferme_pas_la_porte():
    gates = {"phase0": {"nom": "test", "criteres": [
        {"id": "G0.e", "kind": "ci", "bloquant": False, "note": "delegue"},
    ]}}
    rapport = gate.executer_phase(gates, "phase0", ROOT)
    assert rapport["criteres"][0]["status"] == gate.DELEGUE_CI
    assert rapport["verdict"] == gate.VERT


def test_verdict_vert_quand_le_seul_critere_bloquant_est_vert():
    gates = {"phase0": {"nom": "test", "criteres": [
        {"id": "G0.d", "kind": "file", "bloquant": True,
         "path": "omega/gates.toml", "must_contain": "toml"},
    ]}}
    rapport = gate.executer_phase(gates, "phase0", ROOT)
    assert rapport["verdict"] == gate.VERT
