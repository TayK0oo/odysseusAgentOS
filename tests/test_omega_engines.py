"""B6 — le moteur Rust reconstruit un binaire PERIME (pas seulement absent).

Le piege : `if not binary.exists()` ne reconstruit que si le binaire est
ABSENT. Un binaire perime (source plus recente) est alors lance tel quel —
et la porte mesure un etat faux. Observe deux fois le 2026-10-22
(`G1.d.plugins` puis `G1` entier) : deux faux rouges pour la meme cause.

Ces tests sont PURS : ils ne compilent rien, ils decident de la
reconstruction.
"""
from __future__ import annotations

import os
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "omega" / "acceptance"))

from engines.rust import needs_rebuild  # noqa: E402


def _ecrire(path: Path, contenu: str = "x") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contenu, encoding="utf-8")


def test_binaire_absent_demande_un_rebuild(tmp_path):
    binary = tmp_path / "omega-server"
    sources = tmp_path / "crates"
    _ecrire(sources / "omega-core" / "src" / "lib.rs")
    assert needs_rebuild(binary, sources) is True


def test_source_plus_recente_demande_un_rebuild(tmp_path):
    """Le cas piege : binaire present mais perime."""
    binary = tmp_path / "omega-server"
    binary.write_text("binaire", encoding="utf-8")
    source = tmp_path / "crates" / "omega-core" / "src" / "lib.rs"
    _ecrire(source)
    # Source posterieure au binaire (fenetre de 2s : immune au grain de
    # resolution des timestamps).
    vieux = time.time() - 10
    os.utime(binary, (vieux, vieux))
    assert source.stat().st_mtime > binary.stat().st_mtime
    assert needs_rebuild(binary, tmp_path / "crates") is True


def test_source_anterieure_ne_demande_pas_de_rebuild(tmp_path):
    binary = tmp_path / "omega-server"
    binary.write_text("binaire", encoding="utf-8")
    source = tmp_path / "crates" / "omega-core" / "src" / "lib.rs"
    _ecrire(source)
    now = time.time()
    os.utime(binary, (now, now))
    os.utime(source, (now - 10, now - 10))
    assert needs_rebuild(binary, tmp_path / "crates") is False


def test_aucune_source_ne_demande_pas_de_rebuild(tmp_path):
    binary = tmp_path / "omega-server"
    binary.write_text("binaire", encoding="utf-8")
    assert needs_rebuild(binary, tmp_path / "crates") is False
