#!/usr/bin/env python3
"""P0-3b — doc-truth : tout chiffre marque dans un .md doit egaler STATUS.json.

Convention A (ADR-0007) : le tag **porte** la valeur, il est auto-suffisant.

    Le nombre de crates est **22**. <!--m:socles.rust.crates=22-->

Un validateur qui ne lit que des tags n'a pas a parser la prose, et son
controle negatif est trivial : un tag faux doit faire echouer l'outil (lecon L2,
un garde-fou que rien n'observe redevient decoratif).

Cle = chemin pointe dans `STATUS.json` (`socles.rust.crates`). Si la cible est
un objet `{"valeur": ..., "methode": ...}` — la forme des mesures de
`measure.py` — c'est `valeur` qui est comparee : `methode` decrit, elle ne
chiffre pas.

Modes :
  (defaut)   tolerant : une cle absente de `STATUS.json` est un WARNING.
             P0-3c n'a pas encore marque les documents ; un validateur bloquant
             avant le marquage rendrait le marquage impossible.
  --strict   une cle absente est une ERREUR (etat final de P0-3c).
  --json     rapport machine.
  --paths    fichiers ou motifs a scanner (defaut : `docs/**/*.md`, `*.md`).

Exit 0 si aucune divergence, 1 sinon (y compris si `STATUS.json` est illisible :
un validateur qui ne peut pas lire sa reference ne peut pas declarer « vert »).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS_PATH = ROOT / "docs" / "omega" / "plan" / "STATUS.json"
DEFAULT_PATHS = ("docs/**/*.md", "*.md")

TAG = re.compile(r"<!--\s*m:([\w.]+)(?:=([^>]*?))?\s*-->")


def _expand(motif: str) -> list[Path]:
    """Motif relatif (au depot) ou absolu -> fichiers existants.

    `Path.glob` refuse les motifs absolus : un tag dans un fichier hors depot
    (controle negatif sur `tmp_path`) doit rester scannable, sinon le
    validateur n'est testable que depuis le depot lui-meme.
    """
    p = Path(motif)
    if p.is_absolute():
        base, relatif = Path(p.anchor), str(p.relative_to(p.anchor))
    else:
        base, relatif = ROOT, motif
    return [f for f in base.glob(relatif) if f.is_file()]


def _rel(p: Path) -> str:
    try:
        return str(p.relative_to(ROOT))
    except ValueError:
        return str(p)


def _resolve(status: Any, cle: str) -> tuple[bool, Any]:
    """Valeur pointee par `cle`, en devoilant l'enveloppe `{"valeur": ...}`."""
    courant = status
    for part in cle.split("."):
        if not isinstance(courant, dict) or part not in courant:
            return False, None
        courant = courant[part]
    if isinstance(courant, dict) and "valeur" in courant:
        courant = courant["valeur"]
    return True, courant


def _normalize(v: Any) -> str:
    """Forme comparable : `1 114`, `1,114` et `1114` sont le meme chiffre."""
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int | float):
        return str(int(v)) if float(v).is_integer() else str(v)
    if isinstance(v, list | dict):
        return json.dumps(v, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
    return str(v).strip().replace("\u202f", "").replace("\u00a0", "").replace(" ", "").replace(",", "")


def _scanne(status: Any, fichiers: list[Path], strict: bool) -> tuple[list[str], list[str]]:
    erreurs: list[str] = []
    avertissements: list[str] = []
    for f in fichiers:
        text = f.read_text(encoding="utf-8", errors="replace")
        for m in TAG.finditer(text):
            cle, attendu = m.group(1), m.group(2)
            ligne = text.count("\n", 0, m.start()) + 1
            present, reel = _resolve(status, cle)
            if not present:
                msg = f"{_rel(f)}:{ligne} cle absente de STATUS.json: {cle}"
                (erreurs if strict else avertissements).append(msg)
                continue
            if attendu is None:
                avertissements.append(
                    f"{_rel(f)}:{ligne} tag sans valeur ({cle}) — "
                    f"forme attendue : <!--m:{cle}=VALEUR-->"
                )
                continue
            if _normalize(attendu) != _normalize(reel):
                erreurs.append(
                    f"{_rel(f)}:{ligne} {cle}: doc={attendu!r} status={reel!r}"
                )
    return erreurs, avertissements


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="P0-3b — doc-truth (ADR-0007).")
    ap.add_argument("--strict", action="store_true",
                    help="une cle absente de STATUS.json est une erreur (fin de P0-3c)")
    ap.add_argument("--json", action="store_true", help="rapport machine")
    ap.add_argument("--paths", nargs="+", default=list(DEFAULT_PATHS),
                    help="fichiers ou motifs a scanner")
    args = ap.parse_args(argv)

    try:
        status = json.loads(STATUS_PATH.read_text(encoding="utf-8"))
    except OSError as e:
        print(f"ERROR STATUS.json illisible: {e}")
        return 1
    except json.JSONDecodeError as e:
        print(f"ERROR STATUS.json invalide: {e}")
        return 1

    fichiers = sorted({f for motif in args.paths for f in _expand(motif)})
    erreurs, avertissements = _scanne(status, fichiers, args.strict)
    rapport = {
        "files_scanned": len(fichiers),
        "strict": args.strict,
        "errors": erreurs,
        "warnings": avertissements,
        "verdict": "VERT" if not erreurs else f"ROUGE({len(erreurs)})",
    }
    if args.json:
        print(json.dumps(rapport, indent=2, ensure_ascii=False))
    else:
        for a in avertissements:
            print(f"WARN  {a}")
        for e in erreurs:
            print(f"ERROR {e}")
        print(rapport["verdict"])
    return 1 if erreurs else 0


if __name__ == "__main__":
    sys.exit(main())
