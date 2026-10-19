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
 10. check_doc_truth.py — **toléré** jusqu'à la fin de P0-3c (voir plus bas)

Exit 0 seulement si toutes les étapes bloquantes passent. Sortie JSON : rapport
saveable. Une étape `"bloquant": false` est affichée mais ne fait pas échouer le
run : l'étape 10 est déclarée telle, le temps que P0-3c marque les documents —
une porte bloquante avant le marquage rendrait le marquage impossible.

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
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PINNED_PY_DEPS = ["ruff==0.6.9", "pytest==8.3.3"]


def run(cmd, cwd=None, timeout=900):
    try:
        r = subprocess.run(cmd, check=False, cwd=cwd, capture_output=True, text=True,
                           timeout=timeout)
        return r.returncode == 0, ((r.stdout or "") + (r.stderr or "")).strip()
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


def blocking_failures(report: dict) -> list[str]:
    """Etapes en echec qui font echouer le run.

    Une etape `"bloquant": false` est affichee dans le resume mais ne bloque
    pas : c'est le cas de `doc_truth` tant que P0-3c n'a pas marque les
    documents. Le defaut est bloquant — une etape ne devient tolerante que si
    elle le declare, jamais par oubli.
    """
    return [k for k, v in report["etapes"].items()
            if v.get("ok") is False and v.get("bloquant", True)]


def main():
    ap = argparse.ArgumentParser(
        description="P0-1 : installe et vérifie les deux socles, exit 0 si tout est vert.")
    ap.add_argument("--mode", choices=["worktree", "clone"], default="worktree")
    ap.add_argument("--keep", action="store_true", help="garder le répertoire de travail")
    ap.add_argument("--skip-tests", action="store_true", help="pré-commit rapide")
    args = ap.parse_args()

    base = Path(tempfile.mkdtemp(prefix="verify-"))
    report = {
        "mesure_le": datetime.now(UTC).isoformat(timespec="seconds"),
        "sha": git_sha(),
        "mode": args.mode,
        "workdir": str(base),
        "etapes": {},
        "verdict": "ROUGE",
    }

    def finish(code):
        report["verdict"] = code
        report["skipped_par_odus"] = 0
        report.pop("skipped_par_odus")
        rep_path = Path(tempfile.gettempdir()) / f"verify-{report['sha']}-{datetime.now().strftime('%H%M%S')}.json"
        rep_path.write_text(json.dumps(report, indent=2, ensure_ascii=False),
                            encoding="utf-8")
        if not args.keep:
            shutil.rmtree(base, ignore_errors=True)
        # resume stdout : un etat par etape, sortie integrale dans rep_path
        print(f"verdict: {code}")
        for k, v in report["etapes"].items():
            print(f"  {k:12} ok={v.get('ok')}")
        print(f"rapport integral: {rep_path}")
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

    om = work / "omega"

    # 4. Ruff — scope P0 (ADR-0005) : Python nouveau uniquement. Le socle
    #    legacy (src/, services/, tests/, core/, routes/) est une dette P1
    #    chiffree par measure.py, non bloquante pour G0.a.
    ok, out = run([ruff, "check", str(om / "scripts"), str(om / "acceptance")],
                  cwd=work)
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

    # 10. Doc-truth (P0-3b, ADR-0007). Tolere jusqu'a la fin de P0-3c : les six
    #     commits de marquage vivraient sinon dans un arbre rouge pour ce seul
    #     motif. Le verdict brut reste dans le rapport ET dans le resume —
    #     un garde-fou que personne ne voit redevient decoratif (L2).
    ok, out = run([sys.executable, str(om / "scripts" / "check_doc_truth.py")], cwd=work)
    report["etapes"]["doc_truth"] = {"ok": ok, "bloquant": False, "out": out}

    failures = blocking_failures(report)
    return finish("VERT" if not failures else f"ECHEC({','.join(failures)})")


if __name__ == "__main__":
    sys.exit(main())
