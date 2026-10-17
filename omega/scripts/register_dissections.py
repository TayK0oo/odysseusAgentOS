#!/usr/bin/env python3
"""Enregistre les dissections dans le corpus embarque (deterministe, idempotent).

Un JSON present sur le disque mais absent de `APP_DISSECTIONS` est **invisible**
du catalogue : ce script garantit la coherence en regenerant la constante
`APP_DISSECTIONS` de `omega-harness` et en alignant le seuil du test de
completude sur le nombre reel de fichiers.

Usage :
    python3 omega/scripts/register_dissections.py            # applique
    python3 omega/scripts/register_dissections.py --check    # verifie sans ecrire
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
APPS = ROOT / "crates" / "omega-harness" / "data" / "apps"
LIB = ROOT / "crates" / "omega-harness" / "src" / "lib.rs"

BEGIN = "pub const APP_DISSECTIONS: &[&str] = &["
END = "];"


def wanted_block(ids: list[str]) -> str:
    lines = [BEGIN]
    for i in ids:
        lines.append(f'    include_str!("../data/apps/{i}.json"),')
    lines.append(END)
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="ne rien ecrire, juste verifier")
    args = ap.parse_args()

    ids = sorted(p.stem for p in APPS.glob("*.json"))
    src = LIB.read_text()

    start = src.index(BEGIN)
    end = src.index(END, start) + len(END)
    current = src[start:end]
    wanted = wanted_block(ids)

    threshold = len(ids)
    m = re.search(r"d\.apps\.len\(\) >= (\d+)", src)
    if m is None:
        print("seuil du test de completude introuvable dans lib.rs", file=sys.stderr)
        return 1
    old_threshold = int(m.group(1))

    ok_block = current == wanted
    ok_threshold = old_threshold == threshold

    print(f"apps sur disque : {len(ids)}")
    print(f"APP_DISSECTIONS : {'a jour' if ok_block else 'A REGENERER'}")
    print(f"seuil du test   : {old_threshold} -> attendu {threshold}"
          f" {'(a jour)' if ok_threshold else '(A ALIGNER)'}")

    if args.check:
        return 0 if (ok_block and ok_threshold) else 1
    if ok_block and ok_threshold:
        print("rien a faire")
        return 0

    new_src = src[:start] + wanted + src[end:]
    new_src = re.sub(r"d\.apps\.len\(\) >= \d+", f"d.apps.len() >= {threshold}", new_src)
    LIB.write_text(new_src)
    print(f"lib.rs mis a jour ({len(ids)} apps, seuil {threshold})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
