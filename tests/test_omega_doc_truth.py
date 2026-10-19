"""P0-3b — controle du validateur doc-truth (ADR-0007).

Le premier test est le **controle negatif** exige par le plan (§7, lecon L2) :
un chiffre faux dans un document DOIT faire echouer l'outil. Sans lui,
`check_doc_truth.py` serait un echo vert.

Les valeurs attendues sont lues dans `STATUS.json`, jamais recopiees en dur :
un chiffre recopie ici ferait echouer ce fichier le jour ou la mesure change,
pour la mauvaise raison.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "omega" / "scripts" / "check_doc_truth.py"
STATUS = ROOT / "docs" / "omega" / "plan" / "STATUS.json"

CLE = "socles.rust.crates"


def _valeur_reelle() -> int:
    """La valeur que `check_doc_truth` doit considerer comme vraie."""
    status = json.loads(STATUS.read_text(encoding="utf-8"))
    return int(status["socles"]["rust"]["crates"])


def _run(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True, text=True, cwd=ROOT, check=False,
    )


def test_doc_truth_detecte_un_chiffre_faux(tmp_path):
    """Controle negatif : un tag faux -> exit 1 et la cle nommee."""
    doc = tmp_path / "faux.md"
    doc.write_text(
        f"Crates : {_valeur_reelle() + 1}. <!--m:{CLE}={_valeur_reelle() + 1}-->\n",
        encoding="utf-8",
    )
    r = _run("--paths", str(doc))
    assert r.returncode == 1, f"attendu 1, obtenu {r.returncode}\n{r.stdout}{r.stderr}"
    assert CLE in r.stdout, f"la cle doit etre nommee dans le rapport:\n{r.stdout}"


def test_doc_truth_accepte_la_valeur_mesuree(tmp_path):
    """Un tag egal a STATUS.json -> exit 0 (contre-epreuve du precedent)."""
    doc = tmp_path / "ok.md"
    doc.write_text(f"Crates : {_valeur_reelle()}. <!--m:{CLE}={_valeur_reelle()}-->\n",
                   encoding="utf-8")
    r = _run("--paths", str(doc))
    assert r.returncode == 0, f"attendu 0, obtenu {r.returncode}\n{r.stdout}{r.stderr}"


def test_doc_truth_cle_absente_tolere_puis_stricte(tmp_path):
    """P0-3b est tolerant (warning) ; P0-3c finira en --strict (erreur)."""
    doc = tmp_path / "inconnu.md"
    doc.write_text("Bidon : 5. <!--m:inexistant.cle=5-->\n", encoding="utf-8")

    tolere = _run("--paths", str(doc))
    assert tolere.returncode == 0, f"tolerant: attendu 0\n{tolere.stdout}{tolere.stderr}"
    assert "inexistant.cle" in tolere.stdout, "la cle doit figurer au rapport"

    strict = _run("--strict", "--paths", str(doc))
    assert strict.returncode == 1, f"strict: attendu 1\n{strict.stdout}{strict.stderr}"
