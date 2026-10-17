#!/usr/bin/env python3
"""Recensement des harnais d'agents — popularite = preuve GitHub (deterministe).

« Les meilleurs harnais utilises par le plus grand nombre » : on mesure, on ne
suppose pas. Chaque candidat est interroge via l'API GitHub (etoiles, forks,
derniere activite, licence, langage) ; le classement est mecanique.

Sorties :
- `docs/omega/reverse-eng/HARNESS-CENSUS.md` (classement + ecarts a combler) ;
- `omega/crates/omega-harness/data/harnesses_census.json` (donnees, versionnable).

Usage :
    python3 omega/scripts/harness_census.py            # OSS via API GitHub
    python3 omega/scripts/harness_census.py --no-api   # classement hors ligne
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT_MD = REPO / "docs" / "omega" / "reverse-eng" / "HARNESS-CENSUS.md"
OUT_JSON = REPO / "omega" / "crates" / "omega-harness" / "data" / "harnesses_census.json"
APPS_DIR = REPO / "omega" / "crates" / "omega-harness" / "data" / "apps"

# Candidats : (repo GitHub, nom, nature). `closed` = produit non ouvert (docs only).
CANDIDATES: list[tuple[str, str, str]] = [
    # --- deja disseques (rappel, pour le classement) ---
    ("anthropics/claude-code", "Claude Code", "oss"),
    ("openai/codex", "Codex CLI", "oss"),
    ("microsoft/vscode-copilot-chat", "GitHub Copilot Chat", "oss"),
    ("cline/cline", "Cline", "oss"),
    ("RooCodeInc/Roo-Code", "Roo Code", "oss"),
    ("Kilo-Org/kilocode", "Kilo Code", "oss"),
    ("continuedev/continue", "Continue", "oss"),
    ("Aider-AI/aider", "Aider", "oss"),
    ("block/goose", "Goose", "oss"),
    ("All-Hands-AI/OpenHands", "OpenHands", "oss"),
    ("sst/opencode", "OpenCode", "oss"),
    ("zed-industries/zed", "Zed", "oss"),
    ("plandex-ai/plandex", "Plandex", "oss"),
    ("gptme/gptme", "gptme", "oss"),
    ("OpenInterpreter/open-interpreter", "Open Interpreter", "oss"),
    ("princeton-nlp/SWE-agent", "SWE-agent", "oss"),
    ("stitionai/devika", "Devika", "oss"),
    ("Significant-Gravitas/AutoGPT", "AutoGPT", "oss"),
    ("reworkd/AgentGPT", "AgentGPT", "oss"),
    ("TransformerOptimus/SuperAGI", "SuperAGI", "oss"),
    ("geekan/MetaGPT", "MetaGPT", "oss"),
    ("crewAIInc/crewAI", "CrewAI", "oss"),
    ("langchain-ai/langgraph", "LangGraph", "oss"),
    ("frdel/agent-zero", "Agent Zero", "oss"),
    ("AntonOsika/gpt-engineer", "GPT Engineer", "oss"),
    ("Pythagora-io/gpt-pilot", "GPT Pilot", "oss"),
    ("sweepai/sweep", "Sweep", "oss"),
    ("AbanteAI/mentat", "Mentat", "oss"),
    ("BloopAI/bloop", "Bloop", "oss"),
    ("smallcloudai/refact", "Refact.ai", "oss"),
    ("TabbyML/tabby", "Tabby", "oss"),
    ("sourcegraph/cody", "Sourcegraph Cody", "oss"),
    ("voideditor/void", "Void", "oss"),
    ("trypear/pearai-app", "PearAI", "oss"),
    ("charmbracelet/crush", "Crush", "oss"),
    ("google-gemini/gemini-cli", "Gemini CLI", "oss"),
    ("QwenLM/qwen-code", "Qwen Code", "oss"),
    ("CodebuffAI/codebuff", "Codebuff", "oss"),
    ("Nano-Collective/nanocoder", "Nanocoder", "oss"),
    ("onlook-dev/onlook", "Onlook", "oss"),
    ("dyad-sh/dyad", "Dyad", "oss"),
    ("stackblitz/bolt.new", "bolt.new", "oss"),
    ("nicepkg/aide", "Aide", "oss"),
    ("Doriandarko/claude-engineer", "Claude Engineer", "oss"),
    ("sst/opencode", "OpenCode (doublon garde pour la mesure)", "oss"),
    # --- produits fermes (documentation uniquement) ---
    ("", "Cursor", "closed"),
    ("", "Windsurf / Cascade", "closed"),
    ("", "Devin", "closed"),
    ("", "Warp", "closed"),
    ("", "Augment Code", "closed"),
    ("", "Sourcegraph Amp", "closed"),
    ("", "Factory Droid", "closed"),
    ("", "Trae", "closed"),
    ("", "Kiro", "closed"),
    ("", "Amazon Q Developer CLI", "closed"),
    ("", "Replit Agent", "closed"),
    ("", "Lovable", "closed"),
    ("", "v0", "closed"),
]


def token() -> str | None:
    if os.environ.get("GITHUB_TOKEN"):
        return os.environ["GITHUB_TOKEN"]
    env = REPO / ".env"
    if env.is_file():
        m = re.search(r"^GITHUB_TOKEN=(.+)$", env.read_text(), re.M)
        if m:
            return m.group(1).strip()
    return None


def fetch_repo(repo: str, tok: str | None) -> dict | None:
    req = urllib.request.Request(
        f"https://api.github.com/repos/{repo}",
        headers={
            "Accept": "application/vnd.github+json",
            "User-Agent": "omega-census",
            **({"Authorization": f"Bearer {tok}"} if tok else {}),
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            d = json.load(r)
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError) as e:
        print(f"  ! {repo} : {e}", file=sys.stderr)
        return None
    return {
        "repo": repo,
        "stars": d.get("stargazers_count", 0),
        "forks": d.get("forks_count", 0),
        "open_issues": d.get("open_issues_count", 0),
        "language": d.get("language"),
        "license": (d.get("license") or {}).get("spdx_id"),
        "pushed_at": d.get("pushed_at"),
        "archived": d.get("archived", False),
        "description": (d.get("description") or "")[:160],
    }


def dissected_ids() -> set[str]:
    ids = set()
    if APPS_DIR.is_dir():
        for f in APPS_DIR.glob("*.json"):
            try:
                ids.add(json.loads(f.read_text())["id"])
            except (json.JSONDecodeError, KeyError):
                continue
    return ids


def slug(name: str) -> str:
    s = name.lower()
    s = re.sub(r"[^a-z0-9]+", "-", s).strip("-")
    return s


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-api", action="store_true", help="classement sans appeler GitHub")
    args = ap.parse_args()

    tok = None if args.no_api else token()
    if not args.no_api and tok is None:
        print("GITHUB_TOKEN absent : classement sans etoiles (--no-api implicite).", file=sys.stderr)

    seen: dict[str, dict] = {}
    rows: list[dict] = []
    for repo, name, kind in CANDIDATES:
        if repo in seen:
            continue
        data = None
        if repo and tok:
            data = fetch_repo(repo, tok)
            if data:
                seen[repo] = data
        rows.append(
            {
                "id": slug(name),
                "name": name,
                "repo": repo or None,
                "kind": kind,
                "stars": (data or {}).get("stars", 0),
                "forks": (data or {}).get("forks", 0),
                "language": (data or {}).get("language"),
                "license": (data or {}).get("license"),
                "pushed_at": (data or {}).get("pushed_at"),
                "archived": (data or {}).get("archived", False),
                "description": (data or {}).get("description", ""),
            }
        )

    done = dissected_ids()
    for r in rows:
        r["dissected"] = r["id"] in done or any(
            r["id"].startswith(d) or d.startswith(r["id"]) for d in done
        )

    rows.sort(key=lambda r: (-r["stars"], r["name"]))
    gaps = [r for r in rows if not r["dissected"] and r["kind"] == "oss"]

    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    OUT_JSON.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "note": "Popularite mesuree via l'API GitHub (etoiles/forks) — aucun jugement manuel.",
                "measured": not args.no_api and tok is not None,
                "count": len(rows),
                "candidates": rows,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n"
    )

    lines = [
        "# Recensement des harnais — popularite mesuree (GitHub)",
        "",
        "> Genere par `omega/scripts/harness_census.py`. "
        + ("Mesure API reelle." if (tok and not args.no_api) else "Classement hors ligne."),
        "",
        "## Classement",
        "",
        "| # | Harnais | Etoiles | Forks | Langage | Licence | Derniere activite | Disseque |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for i, r in enumerate(rows, 1):
        lines.append(
            f"| {i} | {r['name']} | {r['stars']:,} | {r['forks']:,} | {r['language'] or '—'} | "
            f"{r['license'] or '—'} | {(r['pushed_at'] or '—')[:10]} | "
            f"{'oui' if r['dissected'] else '**non**'} |"
        )
    lines += [
        "",
        f"## Ecarts a combler ({len(gaps)} harnais OSS non disseques, par popularite)",
        "",
        "| Harnais | Etoiles | Depot |",
        "|---|---|---|",
    ]
    for r in gaps:
        lines.append(f"| {r['name']} | {r['stars']:,} | `{r['repo']}` |")
    lines += [
        "",
        "## Produits fermes (documentation uniquement)",
        "",
        ", ".join(r["name"] for r in rows if r["kind"] == "closed"),
        "",
    ]
    OUT_MD.write_text("\n".join(lines) + "\n")

    print(f"{len(rows)} harnais recenses ; {len(gaps)} OSS non disseques")
    print(f"-> {OUT_MD}")
    print(f"-> {OUT_JSON}")
    for r in gaps[:15]:
        print(f"   {r['stars']:>7,}  {r['name']:<24} {r['repo']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
