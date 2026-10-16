#!/usr/bin/env python3
"""Balayage REA — applique l'outil de reverse-engineering a NOS artefacts.

« Utiliser REA sur tout » : chaque artefact du depot qui a un provider REA
disponible est analyse, et la **preuve** (Evidence MCP) est conservee avec ses
limites. Deterministe dans sa logique ; les analyses elles-memes sont faites par
REA (`npx -y rea-agents@latest`).

Sorties :
- `docs/omega/research/rea/evidence/<cible>.summary.json` (resume versionnable) ;
- `docs/omega/research/rea/SWEEP.md` (rapport lisible, regenere) ;
- preuve complete (volumineuse) hors depot : `$OMEGA_REA_EVIDENCE` ou
  `/home/tayk0oo/omega-re/rea-evidence/`.

Usage :
    python3 omega/scripts/rea-sweep.py            # tout
    python3 omega/scripts/rea-sweep.py --only elf # une cible
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
UI_DIST = REPO / "omega" / "ui" / "dist"
TARGET_DIR = REPO / "omega" / "target" / "debug" / "deps"
OUT_DIR = REPO / "docs" / "omega" / "research" / "rea"
EVIDENCE_DIR = Path(
    os.environ.get("OMEGA_REA_EVIDENCE", "/home/tayk0oo/omega-re/rea-evidence")
)
REA = ["npx", "-y", "rea-agents@latest"]
TIMEOUT = 900


def rea_env() -> dict[str, str]:
    env = dict(os.environ)
    # REA n'installe jamais de paquets : l'appelant fournit pwntools/pyelftools/unicorn.
    if "REA_PWNTOOLS_PYTHON" not in env:
        venv = Path("/home/tayk0oo/omega-re/rea-venv3/bin/python")
        if venv.exists():
            env["REA_PWNTOOLS_PYTHON"] = str(venv)
    return env


def run_rea(args: list[str]) -> tuple[int, dict]:
    """Lance une commande REA en JSON ; renvoie (code, objet).

    REA emet des lignes de progression puis un objet JSON (parfois indente sur
    plusieurs lignes) : on extrait le **premier objet JSON equilibre** de stdout.
    """
    proc = subprocess.run(
        REA + args + ["--format", "json"],
        capture_output=True,
        text=True,
        env=rea_env(),
        timeout=TIMEOUT,
    )
    obj = extract_json(proc.stdout)
    if obj is None:
        obj = {"code": "unparsable", "message": proc.stderr.strip()[-400:] or "sortie non-JSON"}
    return proc.returncode, obj


def extract_json(text: str) -> dict | None:
    """Extrait le premier objet JSON equilibre d'une sortie mêlant logs et JSON."""
    stripped = text.strip()
    if not stripped:
        return None
    try:
        return json.loads(stripped)
    except json.JSONDecodeError:
        pass
    for start in (i for i, ch in enumerate(text) if ch == "{"):
        depth = 0
        in_str = False
        esc = False
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


def smallest_crate_elf() -> Path | None:
    """ELF le plus petit de nos crates (REA limite l'entree a 32 Mio)."""
    if not TARGET_DIR.is_dir():
        return None
    limit = 32 * 1024 * 1024
    best: tuple[int, Path] | None = None
    for p in TARGET_DIR.glob("omega_*"):
        if not p.is_file() or not os.access(p, os.X_OK):
            continue
        size = p.stat().st_size
        if size >= limit:
            continue
        if best is None or size < best[0]:
            best = (size, p)
    return best[1] if best else None


def targets() -> list[dict]:
    elf = smallest_crate_elf()
    out = [
        {
            "name": "ui-dist",
            "what": "Cockpit React builde (JS/Electron-style)",
            "command": ["analyze-javascript-application", str(UI_DIST)],
            "subject": str(UI_DIST),
        },
    ]
    # Capture navigateur : exige un Chrome en debug distant (CDP) fourni par
    # l'appelant (REA n'embarque pas de navigateur). Activable via l'env :
    # REA_CDP_ENDPOINT (ex. http://127.0.0.1:9222) + REA_CDP_TARGET (id de page).
    cdp = os.environ.get("REA_CDP_ENDPOINT")
    target_id = os.environ.get("REA_CDP_TARGET")
    if cdp and target_id:
        out.append(
            {
                "name": "omega-server-web",
                "what": f"Bundle servi par omega-server (via CDP {cdp})",
                "command": ["analyze-web-bundle", cdp, target_id],
                "subject": cdp,
            }
        )
    if elf is not None:
        out.append(
            {
                "name": "omega-crate-elf",
                "what": f"Binaire ELF de nos crates ({elf.name})",
                "command": ["inspect-binary-layout", str(elf)],
                "subject": str(elf),
            }
        )
    out.append(
        {
            "name": "repo-overview",
            "what": "Vue d'ensemble des sources du cockpit (omega/ui/src)",
            "command": ["analyze", str(REPO / "omega" / "ui" / "src")],
            "subject": str(REPO / "omega" / "ui" / "src"),
        }
    )
    return out


def summarize(target: dict, code: int, obj: dict) -> dict:
    """Resume honnete : succes (Evidence) ou indisponibilite (diagnostic)."""
    if "evidence_id" not in obj:
        return {
            "target": target["name"],
            "what": target["what"],
            "subject": target["subject"],
            "status": "unavailable",
            "exit_code": code,
            "code": obj.get("code"),
            "message": obj.get("message"),
            "remediation": (obj.get("remediation") or {}).get("action")
            if isinstance(obj.get("remediation"), dict)
            else obj.get("remediation"),
        }
    nr = obj.get("normalized_result") or {}
    graph = nr.get("graph") or {}
    counts: dict[str, int] = {}
    for key in ("sections", "segments", "symbols", "relocations"):
        if isinstance(nr.get(key), list):
            counts[key] = len(nr[key])
    if isinstance(graph.get("nodes"), list):
        counts["graph_nodes"] = len(graph["nodes"])
    if isinstance(graph.get("edges"), list):
        counts["graph_edges"] = len(graph["edges"])
    return {
        "target": target["name"],
        "what": target["what"],
        "subject": target["subject"],
        "status": "analyzed",
        "exit_code": code,
        "evidence_id": obj["evidence_id"],
        "provider": (obj.get("provider") or {}).get("id"),
        "provider_version": (obj.get("provider") or {}).get("version"),
        "confidence": obj.get("confidence"),
        "operation": obj.get("operation"),
        "predicate_type": obj.get("predicate_type"),
        "authority": obj.get("authority"),
        "digest": (obj.get("subject") or {}).get("digest"),
        "format": nr.get("format"),
        "architecture": nr.get("architecture"),
        "counts": counts,
        "limitations": obj.get("limitations") or [],
    }


def write_report(results: list[dict]) -> None:
    lines = [
        "# Balayage REA — REA applique a nos artefacts",
        "",
        "> Genere par `omega/scripts/rea-sweep.py` (regenerable). "
        "Preuve complete hors depot ; resumes ci-dessous.",
        "",
        "| Cible | Sujet | Statut | Provider | Confiance | Evidence | Limites |",
        "|---|---|---|---|---|---|---|",
    ]
    for r in results:
        if r["status"] == "analyzed":
            lines.append(
                f"| `{r['target']}` | {r['what']} | ✅ analysee | `{r['provider']}` | "
                f"{r['confidence']} | `{r['evidence_id'][:24]}…` | {len(r['limitations'])} |"
            )
        else:
            lines.append(
                f"| `{r['target']}` | {r['what']} | ⚠️ indisponible | — | — | "
                f"{r.get('code') or '—'} | — |"
            )
    lines += [
        "",
        "## Detail",
        "",
    ]
    for r in results:
        lines.append(f"### `{r['target']}` — {r['what']}")
        lines.append("")
        lines.append(f"- sujet : `{r['subject']}`")
        if r["status"] == "analyzed":
            lines.append(f"- provider : `{r['provider']}` ({r.get('provider_version')})")
            lines.append(
                f"- confiance : **{r['confidence']}** ; operation : `{r.get('operation')}` ; "
                f"autorite : `{r.get('authority')}`"
            )
            lines.append(f"- evidence : `{r['evidence_id']}`")
            if r.get("digest"):
                lines.append(f"- digest : `{r['digest']}`")
            if r.get("counts"):
                detail = ", ".join(f"{k}={v}" for k, v in sorted(r["counts"].items()))
                lines.append(f"- structure : {detail}")
            if r["limitations"]:
                lines.append("- limites declarees par REA :")
                for lim in r["limitations"][:8]:
                    lines.append(f"  - {str(lim)[:160]}")
        else:
            lines.append(f"- **indisponible** : `{r.get('code')}` — {str(r.get('message'))[:200]}")
            if r.get("remediation"):
                lines.append(f"- remediation : {str(r['remediation'])[:200]}")
        lines.append("")
    (OUT_DIR / "SWEEP.md").write_text("\n".join(lines) + "\n")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", default=None, help="ne traiter qu'une cible")
    args = ap.parse_args()

    if shutil.which("npx") is None:
        print("npx introuvable : installez Node.js (>= 22.19).", file=sys.stderr)
        return 1

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "evidence").mkdir(parents=True, exist_ok=True)
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

    results = []
    for target in targets():
        if args.only and target["name"] != args.only:
            continue
        print(f"== {target['name']} : {target['what']}", flush=True)
        try:
            code, obj = run_rea(target["command"])
        except subprocess.TimeoutExpired:
            code, obj = 1, {"code": "timeout", "message": f"delai {TIMEOUT}s depasse"}
        summary = summarize(target, code, obj)
        results.append(summary)
        (OUT_DIR / "evidence" / f"{target['name']}.summary.json").write_text(
            json.dumps(summary, ensure_ascii=False, indent=2) + "\n"
        )
        # Preuve complete hors depot (peut etre volumineuse).
        if "evidence_id" in obj:
            (EVIDENCE_DIR / f"{target['name']}.evidence.json").write_text(
                json.dumps(obj, ensure_ascii=False)
            )
        state = summary["status"]
        extra = (
            f"({summary.get('provider')}, {len(summary.get('limitations', []))} limites)"
            if state == "analyzed"
            else f"({summary.get('code')})"
        )
        print(f"   -> {state} {extra}", flush=True)

    if results:
        write_report(results)
        ok = sum(1 for r in results if r["status"] == "analyzed")
        print(f"\n{ok}/{len(results)} cibles analysees -> {OUT_DIR / 'SWEEP.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
