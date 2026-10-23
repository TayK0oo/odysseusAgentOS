#!/usr/bin/env python3
"""Détecteur d'orphelins — outil de mesure (P0-6).

Lit la liste nominative des plugins (gates.toml G1.d.plugins) et les scénarios
d'acceptation, produit un rapport de différence : quels plugins sont exercés
par au moins un scénario, quels sont orphelins.

Mode par défaut (baseline) : écrit docs/omega/plan/baselines/orphans-<date>.json,
exit 0. Mode --strict : exit 1 si des orphelins sont détectés, avec la liste.

Usage :
  python3 omega/scripts/detect_orphans.py
  python3 omega/scripts/detect_orphans.py --strict
  python3 omega/scripts/detect_orphans.py --gates PATH --scenarios DIR
"""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from datetime import UTC, datetime
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
GATES_DEFAULT = REPO_ROOT / "omega" / "gates.toml"
SCENARIOS_DEFAULT = REPO_ROOT / "omega" / "acceptance" / "scenarios"
BASELINES_DEFAULT = REPO_ROOT / "docs" / "omega" / "plan" / "baselines"

# Plugins « vifs » : exercés par tout scénario qui démarre le moteur Rust.
VIF_PLUGINS = {"ollama", "default", "mcp:rea", "file", "embedded"}


def load_plugins(gates_path: Path) -> list[dict]:
    with gates_path.open("rb") as f:
        data = tomllib.load(f)
    for phase in data.values():
        if not isinstance(phase, dict):
            continue
        for c in phase.get("criteres", []):
            if c.get("id") == "G1.d.plugins":
                return list(c.get("plugins", {}).get("attendus", []))
    return []


def exercised_plugins(scenarios_dir: Path) -> dict[str, list[str]]:
    """Retourne {plugin_id: [scenario_ids]} pour les plugins exercés."""
    result: dict[str, list[str]] = {p: [] for p in VIF_PLUGINS}
    if scenarios_dir.exists():
        for yaml_path in sorted(scenarios_dir.glob("*.yaml")):
            sid = yaml_path.stem
            for p in VIF_PLUGINS:
                result[p].append(sid)
    return result


def build_report(gates_path: Path, scenarios_dir: Path) -> dict:
    plugins = load_plugins(gates_path)
    plugin_ids = [p["id"] for p in plugins]
    exercised = exercised_plugins(scenarios_dir)
    exercised_set = set(exercised.keys())
    orphans = [pid for pid in plugin_ids if pid not in exercised_set]
    return {
        "plugins_attendus": plugin_ids,
        "plugins_avec_scenario": sorted(exercised_set),
        "orphelins": orphans,
        "source_scenario": exercised,
        "mesure_le": datetime.now(UTC).isoformat(timespec="seconds"),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--gates", type=Path, default=GATES_DEFAULT)
    ap.add_argument("--scenarios", type=Path, default=SCENARIOS_DEFAULT)
    ap.add_argument("--strict", action="store_true", help="exit 1 si orphelins")
    ap.add_argument("--baselines", type=Path, default=BASELINES_DEFAULT)
    args = ap.parse_args(argv)

    report = build_report(args.gates, args.scenarios)

    args.baselines.mkdir(parents=True, exist_ok=True)
    date = datetime.now(UTC).strftime("%Y%m%d")
    out = args.baselines / f"orphans-{date}.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")

    print(json.dumps(report, indent=2, ensure_ascii=False))

    if args.strict and report["orphelins"]:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
