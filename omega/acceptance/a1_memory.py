#!/usr/bin/env python3
"""A1 (conformité par port) — MemoryStore.

Lance la même suite de conformité sur deux adaptateurs du port `MemoryStore` :
  - JsonlMemoryStore (omega-memory)  — adaptateur durable JSONL
  - InMemoryMemoryStore / FakeMemoryStore (omega-adapters) — adaptateur faux

Sortie : JSON avec résultats par adaptateur. Exit 0 ssi les deux passent.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent
OMEGA = REPO_ROOT / "omega"

def run_cargo(args: list[str]) -> tuple[int, str]:
    r = subprocess.run(["cargo"] + args, cwd=OMEGA, capture_output=True, text=True, check=False)
    return r.returncode, (r.stdout + r.stderr)[-4000:]

def main() -> int:
    results = {}
    # JsonlMemoryStore
    rc, out = run_cargo(["test", "-p", "omega-memory"])
    results["JsonlMemoryStore"] = {"exit": rc, "tail": out[-500:]}
    # FakeMemoryStore (omega-adapters)
    rc2, out2 = run_cargo(["test", "-p", "omega-adapters"])
    results["FakeMemoryStore"] = {"exit": rc2, "tail": out2[-500:]}
    print(json.dumps(results, indent=2, ensure_ascii=False))
    return 0 if (rc == 0 and rc2 == 0) else 1

if __name__ == "__main__":
    sys.exit(main())
