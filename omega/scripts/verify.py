#!/usr/bin/env python3
"""P0-1 — verify : une commande qui installe et vérifie les deux socles.

Sur un worktree jetable (ou clone vierge), exécute dans l'ordre :
  1. prérequis : python >= 3.12, cargo, rustc, git
  2. venv + dépendances Python figées (ruff, pytest)
  3. ruff check .
  4. pytest -q tests/ (si présent)
  5. cargo fmt --check
  6. cargo clippy --workspace --all-targets -- -D warnings
  7. cargo test --workspace --no-fail-fast
  8. check_dep_graph.py
  9. model_policy.py --check

Exit 0 seulement si toutes les étapes passent. Sortie JSON : rapport saveable.

Note : `cargo test` sur un worktree neuf recompile tout (~5 à 15 min, première
fois). `--skip-tests` existe pour le pré-commit rapide.

écart honnête vs. énoncé P0-1 : le contrôle « '3.12' in version » était faux
(3.14 >= 3.12 aurait échoué). Comparaison tuple major.minor réelle plus bas.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import venv
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PINNED_PY_DEPS = ["ruff==0.6.9", "pytest==8.3.3"]


def run(cmd, cwd=None, timeout=900):
    try:
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                           timeout=timeout)
        return r.returncode == 0, ((r.stdout or "") + (r.stderr or "")).strip()[-4000:]
    except FileNotFoundError as e:
        return False, f"introuvable: {cmd[0]} ({e})"
    except subprocess.TimeoutExpired:
        return False, f"timeout {timeout}s: {' '.join(cmd)}"


def git_sha():
    ok, out = run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT)
    return out if ok else "inconnu"


def python_version_tuple(version_out: str):
    """'Python 3.12.14' -> (3, 12) ; None si illisible."""
    try:
        parts = version_out.split()[1].split(".")[:2]
        return tuple(int(x) for x in parts)
    except (IndexError, ValueError):
        return None


def main():
    ap = argparse.ArgumentParser(
        description="P0-1 : installe et vérifie les deux socles, exit 0 si tout est vert.")
    ap.add_argument("--mode", choices=["worktree", "clone"], default="worktree")
    ap.add_argument("--keep", action="store_true", help="garder le répertoire de travail")
    ap.add_argument("--skip-tests", action="store_true", help="pré-commit rapide")
    args = ap.parse_args()

    base = Path(tempfile.mkdtemp(prefix="verify-"))
    report = {
        "mesure_le": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "sha": git_sha(),
        "mode": args.mode,
        "workdir": str(base),
        "etapes": {},
        "verdict": "ROUGE",
    }

    def finish(code):
        report["verdict"] = code
        if not args.keep:
            shutil.rmtree(base, ignore_errors=True)
        print(json.dumps(report, indent=2, ensure_ascii=False))
        return 0 if code == "VERT" else 1

    # 1. Prérequis
    prereqs = {}
    for name, cmd in [
        ("python", [os.environ.get("VERIFY_PYTHON", "python3"), "--version"]),
        ("cargo", ["cargo", "--version"]),
        ("rustc", ["rustc", "--version"]),
        ("git", ["git", "--version"]),
    ]:
        ok, out = run(cmd)
        prereqs[name] = {"ok": ok, "out": out}
    report["etapes"]["prerequis"] = prereqs
    if not all(v["ok"] for v in prereqs.values()):
        return finish("ECHEC(prerequis)")
    pyver = python_version_tuple(prereqs["python"]["out"])
    if pyver is None or pyver < (3, 12):
        return finish("ECHEC(python<3.12)")

    # 2. Worktree ou clone
    if args.mode == "worktree":
        work = base / "wt"
        ok, out = run(["git", "worktree", "add", "--detach", str(work), "HEAD"],
                      cwd=ROOT)
    else:
        work = base / "clone"
        ok, out = run(["git", "clone", "--quiet", "--no-hardlinks", str(ROOT), str(work)])
    report["etapes"][args.mode] = {"ok": ok, "out": out}
    if not ok:
        return finish(f"ECHEC({args.mode})")

    # 3. Venv (hors worktree pour ne pas polluer ruff/pytest)
    vdir = base / "venv"
    venv.create(vdir, with_pip=True, clear=True)
    pip = str(vdir / "bin" / "pip")
    ok, out = run([pip, "install", "--quiet", "--disable-pip-version-check",
                   *PINNED_PY_DEPS], timeout=600)
    report["etapes"]["venv"] = {"ok": ok, "out": out}
    if not ok:
        return finish("ECHEC(venv)")
    ruff = str(vdir / "bin" / "ruff")
    pytestb = str(vdir / "bin" / "pytest")

    # 4. Ruff
    ok, out = run([ruff, "check", "."], cwd=work)
    report["etapes"]["ruff"] = {"ok": ok, "out": out}

    # 5. Pytest
    if args.skip_tests:
        report["etapes"]["pytest"] = {"ok": None, "out": "skippé (--skip-tests)"}
    elif (work / "tests").is_dir():
        ok, out = run([pytestb, "-q", "tests"], cwd=work, timeout=1800)
        report["etapes"]["pytest"] = {"ok": ok, "out": out}
    else:
        report["etapes"]["pytest"] = {"ok": None, "out": "tests/ absent"}

    # 6 à 8. Rust
    om = work / "omega"
    ok, out = run(["cargo", "fmt", "--check"], cwd=om)
    report["etapes"]["cargo_fmt"] = {"ok": ok, "out": out}

    ok, out = run(["cargo", "clippy", "--workspace", "--all-targets",
                   "--", "-D", "warnings"], cwd=om, timeout=1800)
    report["etapes"]["cargo_clippy"] = {"ok": ok, "out": out}

    if args.skip_tests:
        report["etapes"]["cargo_test"] = {"ok": None, "out": "skippé (--skip-tests)"}
    else:
        ok, out = run(["cargo", "test", "--workspace", "--no-fail-fast"],
                      cwd=om, timeout=2400)
        report["etapes"]["cargo_test"] = {"ok": ok, "out": out}

    # 9. Portes fédérées
    ok, out = run([sys.executable, str(om / "scripts" / "check_dep_graph.py")], cwd=work)
    report["etapes"]["dep_graph"] = {"ok": ok, "out": out}

    ok, out = run([sys.executable, str(om / "scripts" / "model_policy.py"), "--check"],
                  cwd=work)
    report["etapes"]["model_policy"] = {"ok": ok, "out": out}

    failures = [k for k, v in report["etapes"].items() if v.get("ok") is False]
    return finish("VERT" if not failures else f"ECHEC({','.join(failures)})")


if __name__ == "__main__":
    sys.exit(main())
