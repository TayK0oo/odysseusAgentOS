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


def test_doc_truth_cle_absente_erreur_par_defaut_option_tolerante(tmp_path):
    """Strict par defaut depuis P0-3c ; `--tolerant` reste l'option de secours."""
    doc = tmp_path / "inconnu.md"
    doc.write_text("Bidon : 5. <!--m:inexistant.cle=5-->\n", encoding="utf-8")

    tolere = _run("--tolerant", "--paths", str(doc))
    assert tolere.returncode == 0, f"tolerant: attendu 0\n{tolere.stdout}{tolere.stderr}"
    assert "inexistant.cle" in tolere.stdout, "la cle doit figurer au rapport"

    strict = _run("--paths", str(doc))
    assert strict.returncode == 1, f"strict (defaut): attendu 1\n{strict.stdout}{strict.stderr}"


def test_un_tag_dans_du_code_est_de_la_documentation(tmp_path):
    """Bloc delimite et span inline : documentation, pas marqueur (ADR-0007 §6).

    Controle dans les deux sens : le meme tag hors code doit echouer, sinon un
    masquage trop large rendrait le test complaisant.
    """
    cle = "inexistant.cle"

    dans_code = tmp_path / "exemple.md"
    dans_code.write_text(
        f"Exemple inline : `<!--m:{cle}=5-->`\n"
        f"\n```md\n<!--m:{cle}=5-->\n```\n"
        f"\n~~~\n<!--m:{cle}=5-->\n~~~\n",
        encoding="utf-8",
    )
    r = _run("--paths", str(dans_code))
    assert r.returncode == 0, f"code = documentation : attendu 0\n{r.stdout}{r.stderr}"

    hors_code = tmp_path / "actif.md"
    hors_code.write_text(f"Bidon : <!--m:{cle}=5-->\n", encoding="utf-8")
    r = _run("--paths", str(hors_code))
    assert r.returncode == 1, f"hors code = divergence : attendu 1\n{r.stdout}{r.stderr}"
