#!/usr/bin/env python3
"""Plan d'intégration consolidé — déterministe (zéro LLM).

Agrège les dissections (`omega/crates/omega-harness/data/apps/*.json`) et classe
les fonctionnalités **planifiées** par consensus : une capacité visée par PLUSIEURS
harnais est un signal fort qu'elle est structurelle (et non un caprice d'éditeur).

Sortie : docs/omega/reverse-eng/PLAN-INTEGRATION.md

Usage : python3 omega/scripts/integration_plan.py
"""
from __future__ import annotations

import glob
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
APPS = sorted(glob.glob(str(ROOT / "crates/omega-harness/data/apps/*.json")))
OUT = ROOT.parent / "docs/omega/reverse-eng/PLAN-INTEGRATION.md"

COST_ORDER = {"S": 0, "M": 1, "L": 2}


def main() -> int:
    apps: list[dict] = [json.loads(Path(p).read_text()) for p in APPS]

    # capacite -> agregat
    groups: dict[str, dict] = defaultdict(
        lambda: {"apps": set(), "examples": [], "costs": [], "domains": set()}
    )
    failed: dict[str, int] = defaultdict(int)

    total = adopted = planned = rejected = 0
    for app in apps:
        for f in app["functionalities"]:
            total += 1
            status = f["status"]
            if status == "adopted":
                adopted += 1
                continue
            if status == "rejected":
                rejected += 1
                failed[f.get("omega_capability") or f["domain"]] += 1
                continue
            planned += 1
            key = f.get("omega_capability") or f"({f['domain']})"
            g = groups[key]
            g["apps"].add(app["id"])
            g["costs"].append(f["rust_cost"])
            g["domains"].add(f["domain"])
            g["examples"].append(
                (app["name"], f["name"], f.get("omega_adapter") or "—", f["rust_cost"])
            )

    rows = []
    for cap, g in groups.items():
        n = len(g["apps"])
        min_cost = min(g["costs"], key=lambda c: COST_ORDER.get(c, 9))
        # score = consensus (apps) x3 + bonus faible coût
        score = n * 3 + (2 if min_cost == "S" else 1 if min_cost == "M" else 0)
        rows.append((score, n, min_cost, cap, sorted(g["apps"]), g["examples"]))
    rows.sort(key=lambda r: (-r[0], r[1] * -1, r[2]))

    lines: list[str] = []
    lines.append("# PLAN-INTEGRATION — ce que les harnais convergent à demander")
    lines.append("")
    lines.append("> **Généré** par `omega/scripts/integration_plan.py` (déterministe, zéro LLM) à partir")
    lines.append(f"> des dissections embarquées : **{len(apps)} applications**, **{total} fonctionnalités**.")
    lines.append("> **Lecture :** une *capacité* réclamée par plusieurs harnais est un signal")
    lines.append("> structurel — c'est elle qu'il faut implémenter en premier (port + plugins).")
    lines.append("")
    lines.append("| Chiffre | Valeur |")
    lines.append("|---|---|")
    lines.append(f"| Applications disséquées | {len(apps)} |")
    lines.append(f"| Fonctionnalités | {total} |")
    lines.append(f"| Adoptées (déjà dans Omega) | {adopted} |")
    lines.append(f"| Planifiées (à faire) | {planned} |")
    lines.append(f"| Écartées (assumées) | {rejected} |")
    lines.append("")

    lines.append("## Priorités par consensus")
    lines.append("")
    lines.append("| # | Capacité visée | Harnais | Coût min | Exemples (harnais → adaptateur) |")
    lines.append("|---|---|---|---|---|")
    for i, (_score, n, min_cost, cap, aps, examples) in enumerate(rows[:30], 1):
        ex = " ; ".join(f"{a} → `{ad}`" for (a, _nm, ad, _c) in examples[:3])
        lines.append(f"| {i} | `{cap}` | {n} ({', '.join(aps[:4])}{'…' if len(aps) > 4 else ''}) | {min_cost} | {ex} |")
    lines.append("")

    lines.append("## Consensus fort (≥ 4 harnais réclament la même capacité)")
    lines.append("")
    strong = [r for r in rows if r[1] >= 4]
    if not strong:
        lines.append("_aucun pour l'instant_")
    for (_score, n, min_cost, cap, aps, _ex) in strong:
        lines.append(f"- **`{cap}`** — {n} harnais ({', '.join(aps)}) — coût min **{min_cost}**")
    lines.append("")

    lines.append("## Quick wins (coût S, le plus réclamé)")
    lines.append("")
    quick = sorted([r for r in rows if r[2] == "S"], key=lambda r: -r[1])[:15]
    if not quick:
        lines.append("_aucun_")
    for (_score, n, _c, cap, _aps, examples) in quick:
        ex = ", ".join(f"{a}" for (a, _nm, _ad, _c2) in examples[:4])
        lines.append(f"- `{cap}` — {n} harnais ({ex})")
    lines.append("")

    lines.append("## Écarté (et pourquoi c'est assumé)")
    lines.append("")
    for cap, n in sorted(failed.items(), key=lambda kv: -kv[1]):
        lines.append(f"- `{cap}` — {n} fonctionnalité(s) écartée(s)")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("_Régénérer : `python3 omega/scripts/integration_plan.py`_")

    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"{OUT} : {len(rows)} capacités agrégées, {planned} planifiées, {adopted} adoptées, {rejected} écartées")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
