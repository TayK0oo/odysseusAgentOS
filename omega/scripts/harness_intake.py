#!/usr/bin/env python3
"""Intake des harnais recenses — clone + analyse REA, par popularite.

Chaine (deterministe, sans LLM) :
1. lit le recensement (`harnesses_census.json`, popularite mesuree) ;
2. clone en profondeur 1 chaque harnais OSS non encore disseques ;
3. detecte la nature du depot (JS/TS, Rust, Go, Python) ;
4. applique REA : analyse JavaScript profonde pour le JS/TS, vue d'ensemble sinon ;
5. conserve les preuves (Evidence) + un rapport lisible.

Sorties :
- `docs/omega/reverse-eng/HARNESS-INTAKE.md` ;
- `docs/omega/research/rea/evidence/intake-<id>.summary.json` ;
- preuve complete hors depot (`$OMEGA_REA_EVIDENCE`).

Usage :
    python3 omega/scripts/harness_intake.py --top 14
    python3 omega/scripts/harness_intake.py --only gemini-cli
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CENSUS = REPO / "omega" / "crates" / "omega-harness" / "data" / "harnesses_census.json"
OUT_MD = REPO / "docs" / "omega" / "reverse-eng" / "HARNESS-INTAKE.md"
OUT_EVIDENCE = REPO / "docs" / "omega" / "research" / "rea" / "evidence"
EVIDENCE_DIR = Path(os.environ.get("OMEGA_REA_EVIDENCE", "/home/tayk0oo/omega-re/rea-evidence"))
CLONES = Path(os.environ.get("OMEGA_RE_CLONES", "/home/tayk0oo/omega-re"))
REA = ["npx", "-y", "rea-agents@latest"]
TIMEOUT = 900


def rea_env() -> dict[str, str]:
    env = dict(os.environ)
    venv = Path("/home/tayk0oo/omega-re/rea-venv3/bin/python")
    if "REA_PWNTOOLS_PYTHON" not in env and venv.exists():
        env["REA_PWNTOOLS_PYTHON"] = str(venv)
    return env


def extract_json(text: str) -> dict | None:
    text = text.strip()
    if not text:
        return None
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        pass
    for start in (i for i, ch in enumerate(text) if ch == "{"):
        depth, in_str, esc = 0, False, False
        for i in range(start, len(text)):
            ch = text[i]
            if in_str:
                if esc:
                    esc = False
                elif ch == "\\":
                    esc = True
                elif ch == '"':
                    in_str = False
                continue
            if ch == '"':
                in_str = True
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    try:
                        return json.loads(text[start : i + 1])
                    except json.JSONDecodeError:
                        break
    return None


def nature_of(path: Path) -> str:
    if (path / "package.json").is_file():
        return "js"
    if (path / "Cargo.toml").is_file():
        return "rust"
    if (path / "go.mod").is_file():
        return "go"
    if (path / "pyproject.toml").is_file() or (path / "requirements.txt").is_file():
        return "python"
    return "other"


def clone(repo: str, dest: Path) -> tuple[bool, str]:
    if (dest / ".git").is_dir():
        return True, "deja clone"
    dest.parent.mkdir(parents=True, exist_ok=True)
    r = subprocess.run(
        ["git", "clone", "--depth", "1", f"https://github.com/{repo}.git", str(dest)],
        capture_output=True,
        text=True,
        timeout=1800, check=False,
    )
    if r.returncode != 0:
        return False, (r.stderr or r.stdout).strip()[-200:]
    return True, "clone ok"


def rea_command(nature: str, path: Path) -> list[str]:
    if nature == "js":
        return ["analyze-javascript-application", str(path)]
    # Python/Rust/Go : REA n'a pas d'analyseur de source dedie -> vue d'ensemble.
    return ["analyze", str(path)]


def run_rea(args: list[str]) -> tuple[int, dict]:
    proc = subprocess.run(
        REA + args + ["--format", "json"],
        capture_output=True,
        text=True,
        env=rea_env(),
        timeout=TIMEOUT, check=False,
    )
    obj = extract_json(proc.stdout)
    if obj is None:
        obj = {"code": "unparsable", "message": (proc.stderr or proc.stdout).strip()[-300:]}
    return proc.returncode, obj


def summarize(entry: dict, nature: str, cloned: bool, clone_note: str, code: int, obj: dict) -> dict:
    base = {
        "id": entry["id"],
        "name": entry["name"],
        "repo": entry["repo"],
        "stars": entry["stars"],
        "language": entry.get("language"),
        "license": entry.get("license"),
        "nature": nature,
        "cloned": cloned,
        "clone_note": clone_note,
    }
    if not cloned:
        base |= {"status": "clone_failed", "message": clone_note}
        return base
    if "evidence_id" not in obj:
        base |= {
            "status": "unavailable",
            "code": obj.get("code"),
            "message": obj.get("message"),
        }
        return base
    nr = obj.get("normalized_result") or {}
    graph = nr.get("graph") or {}
    counts = {}
    if isinstance(graph.get("nodes"), list):
        counts["graph_nodes"] = len(graph["nodes"])
    if isinstance(graph.get("edges"), list):
        counts["graph_edges"] = len(graph["edges"])
    for key in ("sections", "segments", "symbols"):
        if isinstance(nr.get(key), list):
            counts[key] = len(nr[key])
    base |= {
        "status": "analyzed",
        "evidence_id": obj["evidence_id"],
        "provider": (obj.get("provider") or {}).get("id"),
        "confidence": obj.get("confidence"),
        "operation": obj.get("operation"),
        "digest": (obj.get("subject") or {}).get("digest"),
        "counts": counts,
        "limitations_count": len(obj.get("limitations") or []),
    }
    return base


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--top", type=int, default=14, help="nombre de harnais a traiter")
    ap.add_argument("--only", default=None, help="un seul identifiant")
    ap.add_argument("--force", action="store_true", help="traiter memose si deja dissecute")
    args = ap.parse_args()

    census = json.loads(CENSUS.read_text())
    gaps = [c for c in census["candidates"] if c["kind"] == "oss" and (args.force or not c["dissected"])]
    gaps.sort(key=lambda c: -c["stars"])
    selected = [c for c in gaps if args.only is None or c["id"] == args.only]
    if args.only is None:
        selected = selected[: args.top]
    if not selected:
        print("aucun harnais a traiter", file=sys.stderr)
        return 1

    OUT_EVIDENCE.mkdir(parents=True, exist_ok=True)
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    results = []

    for entry in selected:
        print(f"== {entry['name']} ({entry['stars']:,} etoiles) — {entry['repo']}", flush=True)
        dest = CLONES / entry["id"]
        cloned, note = clone(entry["repo"], dest)
        nature = nature_of(dest) if cloned else "?"
        print(f"   clone: {'ok' if cloned else 'ECHEC'} ({note}) ; nature: {nature}", flush=True)
        code, obj = (1, {}) if not cloned else run_rea(rea_command(nature, dest))
        summary = summarize(entry, nature, cloned, note, code, obj)
        results.append(summary)
        (OUT_EVIDENCE / f"intake-{entry['id']}.summary.json").write_text(
            json.dumps(summary, ensure_ascii=False, indent=2) + "\n"
        )
        if "evidence_id" in obj:
            (EVIDENCE_DIR / f"intake-{entry['id']}.evidence.json").write_text(
                json.dumps(obj, ensure_ascii=False)
            )
        print(
            f"   -> {summary['status']}"
            + (
                f" ({summary.get('provider')}, {summary.get('limitations_count')} limites)"
                if summary["status"] == "analyzed"
                else f" ({summary.get('code') or summary.get('message', '')})"
            ),
            flush=True,
        )

    lines = [
        "# Intake des harnais — clone + REA (popularite mesuree)",
        "",
        "> Genere par `omega/scripts/harness_intake.py`. Analyse profonde REA pour le JS/TS ; "
        "vue d'ensemble sinon (REA n'a pas d'analyseur de source Python/Rust/Go).",
        "",
        "| # | Harnais | Etoiles | Nature | Clone | REA | Provider | Limites |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for i, r in enumerate(results, 1):
        rea = {
            "analyzed": "✅ analysee",
            "unavailable": "⚠️ indisponible",
            "clone_failed": "❌ clone",
        }[r["status"]]
        lines.append(
            f"| {i} | {r['name']} | {r['stars']:,} | {r['nature']} | "
            f"{'ok' if r['cloned'] else 'echec'} | {rea} | {r.get('provider') or '—'} | "
            f"{r.get('limitations_count', '—')} |"
        )
    lines += ["", "## Detail", ""]
    for r in results:
        lines.append(f"### {r['name']} ({r['stars']:,} etoiles) — `{r['repo']}`")
        lines.append("")
        lines.append(f"- nature : {r['nature']} ; licence : {r.get('license') or '—'}")
        if r["status"] == "analyzed":
            lines.append(f"- evidence : `{r['evidence_id']}`")
            lines.append(f"- provider : `{r.get('provider')}` ; confiance : **{r.get('confidence')}**")
            if r.get("counts"):
                lines.append(
                    "- structure : " + ", ".join(f"{k}={v}" for k, v in sorted(r["counts"].items()))
                )
        else:
            lines.append(f"- **{r['status']}** : {r.get('code') or ''} {str(r.get('message'))[:200]}")
        lines.append("")

    OUT_MD.write_text("\n".join(lines) + "\n")
    ok = sum(1 for r in results if r["status"] == "analyzed")
    print(f"\n{ok}/{len(results)} analyses -> {OUT_MD}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
