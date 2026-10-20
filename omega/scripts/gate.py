#!/usr/bin/env python3
"""gate.py — execute les portes d'une phase (travail P0-5 du PLAN-MAITRE).

    python3 omega/scripts/gate.py --phase 0
    python3 omega/scripts/gate.py --phase all
    python3 omega/scripts/gate.py --list

Definitions : omega/gates.toml (TOML, parse par tomllib — stdlib, 0 dependance).
Resume sur stdout + rapport JSON dans docs/omega/plan/gates/phase-<N>-<ts>.json.

Code de sortie : 0 si tous les criteres bloquants sont au vert, 1 sinon.
Un critere `kind = "ci"` est DELEGUE_CI : ni vert ni rouge, il ne fait pas
echouer la porte (il sera verifie par la CI, pas en local).
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
import tomllib
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GATES = ROOT / "omega" / "gates.toml"
RAPPORTS = ROOT / "docs" / "omega" / "plan" / "gates"

VERT = "VERT"
ECHEC = "ECHEC"
TIMEOUT = "TIMEOUT"
DELEGUE_CI = "DELEGUE_CI"
VIOLATION = "VIOLATION_PORTE_CODE"


def charger() -> dict:
    with GATES.open("rb") as f:
        return tomllib.load(f)


def phases(gates: dict) -> list[str]:
    return sorted(k for k in gates if k.startswith("phase"))


def _exec(cmd: list[str], cwd: Path, timeout: int) -> tuple[int | None, str]:
    try:
        p = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout,
            errors="replace", check=False,
        )
    except subprocess.TimeoutExpired as e:
        return None, f"TIMEOUT apres {timeout}s\n{(e.stdout or '')[:4000]}"
    except FileNotFoundError as e:
        return 127, f"COMMANDE INTROUVABLE: {e}"
    return p.returncode, ((p.stdout or "") + (p.stderr or ""))[:4000]


def _dernier_commit(root: Path, chemins: list[str]) -> str | None:
    if not chemins:
        return None
    try:
        p = subprocess.run(
            ["git", "log", "-1", "--format=%H", "--"] + chemins,
            cwd=root, capture_output=True, text=True, timeout=60, check=False,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return None
    return p.stdout.strip() or None


def _fichiers_juges(root: Path, critere: dict) -> list[str]:
    """Chemins du depot qu'un critere juge (pour la regle porte/code)."""
    if critere.get("kind") == "file":
        motif = critere.get("path", "")
        return [motif] if motif else []
    chemins = []
    for arg in critere.get("argv", []):
        if arg.startswith("-") or "/" not in arg:
            continue
        if (root / arg).exists():
            chemins.append(arg)
    return chemins


def _regle_porte_code(root: Path, critere: dict) -> str | None:
    """Une porte ne doit pas etre modifiee dans le meme commit que le code juge."""
    if critere.get("kind") == "ci":
        return None
    juges = _fichiers_juges(root, critere)
    if not juges:
        return None
    c_porte = _dernier_commit(root, ["omega/gates.toml"])
    c_code = _dernier_commit(root, juges)
    if c_porte and c_code and c_porte == c_code:
        return f"gates.toml et {', '.join(juges)} modifies dans le commit {c_porte[:12]}"
    return None


def executer_critere(critere: dict, root: Path) -> dict:
    crit = dict(critere)
    crit["duree_s"] = 0.0
    crit["sortie"] = ""
    crit["note"] = critere.get("note", "")
    kind = critere.get("kind")

    if kind == "ci":
        crit["status"] = DELEGUE_CI
        return crit

    if kind == "file":
        motif = critere.get("path", "")
        matchs = sorted(root.glob(motif)) if motif else []
        if not matchs:
            crit["status"] = ECHEC
            crit["sortie"] = f"aucun fichier pour le motif {motif!r}"
            return crit
        attendu = critere.get("must_contain")
        if attendu:
            rx = re.compile(attendu)
            if not any(rx.search(f.read_text(encoding="utf-8", errors="replace")) for f in matchs):
                crit["status"] = ECHEC
                crit["sortie"] = f"regex {attendu!r} absente de {', '.join(f.name for f in matchs)}"
                return crit
        crit["status"] = VERT
        crit["sortie"] = ", ".join(f.name for f in matchs)
        return crit

    if kind == "command":
        argv = critere.get("argv") or []
        if not argv:
            crit["status"] = ECHEC
            crit["sortie"] = "argv vide"
            return crit
        debut = time.monotonic()
        code, sortie = _exec(argv, root, int(critere.get("timeout", 900)))
        crit["duree_s"] = round(time.monotonic() - debut, 1)
        crit["sortie"] = sortie
        crit["code"] = code
        if code is None:
            crit["status"] = TIMEOUT
        elif code != int(critere.get("expect_exit", 0)):
            crit["status"] = ECHEC
            crit["sortie"] = f"code {code}, attendu {critere.get('expect_exit', 0)}\n{sortie}"
        else:
            crit["status"] = VERT
        violation = _regle_porte_code(root, critere)
        if violation:
            crit["status"] = VIOLATION
            crit["sortie"] = violation + "\n" + sortie
        return crit

    crit["status"] = ECHEC
    crit["sortie"] = f"kind inconnu : {kind!r}"
    return crit


def executer_phase(gates: dict, cle: str, root: Path) -> dict:
    phase = gates[cle]
    criteres = [executer_critere(c, root) for c in phase.get("criteres", [])]
    bloquants = [c for c in criteres if c.get("bloquant", True)]
    ratables = [c for c in bloquants if c["status"] not in (VERT, DELEGUE_CI)]
    rapport = {
        "phase": cle,
        "nom": phase.get("nom", ""),
        "mesure_le": datetime.now(UTC).isoformat(timespec="seconds"),
        "verdict": VERT if not ratables else "ROUGE",
        "criteres": criteres,
    }
    return rapport


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="gate.py — portes executables (P0-5).")
    ap.add_argument("--phase", help="0..7, ou 'all'")
    ap.add_argument("--list", action="store_true", help="liste les phases et criteres")
    ap.add_argument("--json", action="store_true", help="n'affiche que le JSON")
    args = ap.parse_args(argv)

    gates = charger()
    cles = phases(gates)

    if args.list:
        for cle in cles:
            print(f"{cle} — {gates[cle].get('nom', '')}")
            for c in gates[cle].get("criteres", []):
                print(f"  {c['id']:14} [{c.get('kind')}] {c.get('description', '')}")
        return 0

    if not args.phase:
        ap.error("--phase est requis (ou --list)")

    if args.phase == "all":
        selection = cles
    else:
        num = args.phase.replace("phase", "")
        selection = [f"phase{num}"] if f"phase{num}" in gates else []
        if not selection:
            print(f"phase inconnue : {args.phase}", file=sys.stderr)
            return 2

    rapports = [executer_phase(gates, cle, ROOT) for cle in selection]
    verdict = VERT if all(r["verdict"] == VERT for r in rapports) else "ROUGE"

    if not args.json:
        for r in rapports:
            print(f"{r['phase']} — {r['nom']}")
            for c in r["criteres"]:
                print(f"  {c['id']:16} {c['status']:18} {c.get('description', '')}")
        print(f"verdict: {verdict}")

    RAPPORTS.mkdir(parents=True, exist_ok=True)
    ts = datetime.now().strftime("%Y%m%d-%H%M%S")
    chemins = []
    for r in rapports:
        p = RAPPORTS / f"phase-{r['phase'].replace('phase', '')}-{ts}.json"
        p.write_text(json.dumps(r, indent=2, ensure_ascii=False), encoding="utf-8")
        chemins.append(str(p.relative_to(ROOT)))
    if not args.json:
        print("rapport: " + ", ".join(chemins))

    if args.json:
        print(json.dumps(rapports if len(rapports) > 1 else rapports[0],
                         indent=2, ensure_ascii=False))
    return 0 if verdict == VERT else 1


if __name__ == "__main__":
    sys.exit(main())
