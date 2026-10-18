#!/usr/bin/env python3
"""Genere les fiches markdown des dissections (deterministe).

Regenere `docs/omega/reverse-eng/dissections/<id>.md` depuis le JSON
`omega/crates/omega-harness/data/apps/<id>.json`. Deterministe : la fiche
peut etre regeneree autant de fois que necessaire.

Usage :
    python3 omega/scripts/gen_dissection_md.py            # tout ce qui manque
    python3 omega/scripts/gen_dissection_md.py onlook     # une fiche (force)
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APPS = ROOT / "omega" / "crates" / "omega-harness" / "data" / "apps"
OUT = ROOT / "docs" / "omega" / "reverse-eng" / "dissections"

README = """# Dissection — {name} ({editor}, {license}, {language})

> Source : clone `--depth 1` de {repo} dans `/home/tayk0oo/omega-re/{id}`.
> Donnees : `omega/crates/omega-harness/data/apps/{id}.json` ({nfct} fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | {editor} |
| Langage | {language} |
| Repo | {repo} |
| Architecture | {architecture} |

## 2. Inventaire fonctionnel

{sections}
"""

SECTION = """### `{fctid}` — {name}

- **domaine** : {domain} · **capacite** : {cap} · **port** : {port} · **cout** : {cost} · **statut** : {status}
- **what** : {what}
- **how** : {how}
- **adaptateur Omega** : {adapter}
- **preuves** : {refs}
"""


def render(app: dict) -> str:
    sections = []
    for f in app["functionalities"]:
        cap = f.get("omega_capability") or "—"
        port = f.get("omega_port") or "—"
        adapter = f.get("omega_adapter") or "—"
        refs = " · ".join("`" + r + "`" for r in f.get("refs", [])) or "—"
        sections.append(SECTION.format(
            fctid=f["id"], name=f["name"], domain=f["domain"], cap=cap, port=port,
            cost=f.get("rust_cost", "—"), status=f["status"], what=f["what"],
            how=f["how"], adapter=adapter, refs=refs))
    return README.format(
        name=app["name"], editor=app["editor"], license=app["license"],
        language=app["language"], repo=app["repo"], id=app["id"],
        nfct=len(app["functionalities"]), architecture=app["architecture"],
        sections="\n".join(sections))


def main() -> int:
    only = set(sys.argv[1:]) or None
    OUT.mkdir(parents=True, exist_ok=True)
    n = 0
    for path in sorted(APPS.glob("*.json")):
        app = json.loads(path.read_text())
        if only and app["id"] not in only:
            continue
        (OUT / f"{app['id']}.md").write_text(render(app))
        n += 1
    print(f"{n} fiches md générées -> {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
