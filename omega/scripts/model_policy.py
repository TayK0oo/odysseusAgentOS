#!/usr/bin/env python3
"""Politique de modeles — verification live + selection par usage (deterministe).

Regle globale (voir `docs/omega/rules/MODEL-POLICY.md`) : sous-agents sur modeles
GRATUITS par defaut ; exception payante uniquement parmi `opencode-go/*`.

Ce script ne fait aucun appel a un LLM. Il interroge les listes **live** des
modeles (Zen + Go), met en cache le resultat, et :

- `--list-free`  : liste les modeles gratuits reellement disponibles ;
- `--check`      : verifie que TOUS les modeles cites par la politique existent ;
- `--for <usage>`: affiche le modele a utiliser pour un usage type ;
- `--apply-agents`: aligne le frontmatter `model:` des agents projet sur la politique ;
- `--json`       : sortie machine.

Usage :
    python3 omega/scripts/model_policy.py --check
    python3 omega/scripts/model_policy.py --for subagent-code
    python3 omega/scripts/model_policy.py --apply-agents
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
POLICY = REPO / "docs" / "omega" / "rules" / "model-policy.json"
CACHE = REPO / "docs" / "omega" / "rules" / "models-live.json"
AGENTS_DIR = REPO / ".opencode" / "agents"

ZEN_URL = "https://opencode.ai/zen/v1/models"
GO_URL = "https://opencode.ai/inference/go/v1/models"


def api_key() -> str | None:
    if os.environ.get("OPENCODE_API_KEY"):
        return os.environ["OPENCODE_API_KEY"]
    env = REPO / ".env"
    if env.is_file():
        m = re.search(r"^OPENCODE_API_KEY=(.+)$", env.read_text(), re.M)
        if m:
            return m.group(1).strip()
    return None


def fetch(url: str) -> list[dict]:
    """Liste des modeles depuis un endpoint OpenCode (liste live)."""
    key = api_key()
    cmd = ["curl", "-sS", "--max-time", "30", url]
    if key:
        cmd += ["-H", f"Authorization: Bearer {key}"]
    r = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if r.returncode != 0:
        return []
    try:
        data = json.loads(r.stdout)
    except json.JSONDecodeError:
        return []
    items = data.get("data", data if isinstance(data, list) else [])
    out = []
    for m in items:
        if not isinstance(m, dict):
            continue
        mid = m.get("id")
        if not mid:
            continue
        out.append({"id": mid, "free": None})
    return out


def live_models(refresh: bool) -> dict[str, bool | None]:
    """{id: est_gratuit} pour tous les modeles disponibles, avec cache."""
    if not refresh and CACHE.is_file():
        cached = json.loads(CACHE.read_text())
        return {m["id"]: m.get("free") for m in cached.get("models", [])}
    policy = json.loads(POLICY.read_text())
    det = policy.get("free_detection", {})
    suffix = det.get("suffix", "-free")
    explicit = set(det.get("explicit", []))
    models: dict[str, bool | None] = {}
    for url, prefix in ((ZEN_URL, "opencode/"), (GO_URL, "opencode-go/")):
        for m in fetch(url):
            mid = m["id"] if "/" in m["id"] else f"{prefix}{m['id']}"
            models[mid] = mid.endswith(suffix) or mid in explicit
    if models:
        CACHE.parent.mkdir(parents=True, exist_ok=True)
        CACHE.write_text(
            json.dumps(
                {
                    "fetched_at": __import__("datetime").datetime.now().isoformat(timespec="seconds"),
                    "models": [{"id": k, "free": v} for k, v in sorted(models.items())],
                },
                ensure_ascii=False,
                indent=2,
            )
            + "\n"
        )
    return models


def policy_models(policy: dict) -> list[str]:
    refs: list[str] = []
    for rule in policy["rules"].values():
        refs += rule.get("free", []) + rule.get("paid", [])
    refs += policy.get("no_retention_free", [])
    refs += policy.get("avoid_for_confidential", [])
    refs += list(policy.get("agent_defaults", {}).values())
    seen, out = set(), []
    for r in refs:
        if r not in seen:
            seen.add(r)
            out.append(r)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--list-free", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--for", dest="usage", default=None)
    ap.add_argument("--apply-agents", action="store_true")
    ap.add_argument("--refresh", action="store_true", help="forcer la mise a jour du cache")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    policy = json.loads(POLICY.read_text())
    live = live_models(args.refresh)
    if not live:
        print("aucune liste live recuperee (reseau/cle ?) — verifier le cache", file=sys.stderr)

    if args.list_free:
        frees = sorted(k for k, v in live.items() if v is True)
        if args.json:
            print(json.dumps(frees, indent=2))
        else:
            print(f"{len(frees)} modeles gratuits disponibles :")
            for f in frees:
                print(f"  {f}")
        return 0

    if args.usage:
        rule = policy["rules"].get(args.usage)
        if rule is None:
            print(f"usage inconnu : {args.usage}", file=sys.stderr)
            print("usages : " + ", ".join(sorted(policy["rules"])), file=sys.stderr)
            return 2
        # On ne renvoie que ce qui est REELLEMENT disponible (live), gratuit d'abord.
        free = [m for m in rule.get("free", []) if m in live and live[m] is True] or rule.get("free", [])
        paid = [m for m in rule.get("paid", []) if m in live] or rule.get("paid", [])
        if args.json:
            print(json.dumps({"usage": args.usage, "free": free, "paid": paid, "note": rule.get("note")}, indent=2))
        else:
            print(f"usage: {args.usage}")
            print(f"  gratuit (defaut) : {', '.join(free) or '—'}")
            print(f"  payant Go (exception) : {', '.join(paid) or '—'}")
            if rule.get("note"):
                print(f"  note : {rule['note']}")
        return 0

    if args.check:
        refs = policy_models(policy)
        missing = [r for r in refs if r not in live]
        forbidden = set(policy.get("forbidden", []))
        problems = []
        # (a) une regle ne doit JAMAIS citer un modele interdit
        for m in refs:
            if m in forbidden:
                problems.append(f"regle citant un modele INTERDIT : {m}")
        print(f"modeles live: {len(live)} ({sum(1 for v in live.values() if v)} gratuits)")
        print(f"references par la politique: {len(refs)}")
        if missing:
            print(f"\nABSENTS de la liste live ({len(missing)}) — gratuit 'limited time' disparu ?")
            for m in missing:
                print(f"  ✗ {m}")
        for p in problems:
            print(f"  ✗ {p}")
        print(f"\nmodeles interdits surveilles (ne doivent jamais apparaitre dans une regle) : {len(forbidden)}")
        ok = not missing and not problems
        print("\nOK : politique coherente avec la liste live" if ok else "\nA CORRIGER")
        return 0 if ok else 1

    if args.apply_agents:
        defaults = policy["agent_defaults"]
        changed = []
        for name, model in defaults.items():
            f = AGENTS_DIR / f"{name}.md"
            if not f.is_file():
                continue
            text = f.read_text()
            new, n = re.subn(r"^model: .*$", f"model: {model}", text, count=1, flags=re.M)
            if n == 0:
                # pas de champ model : on l'insere apres `mode:`
                new, n = re.subn(r"^(mode: .*)$", rf"\1\nmodel: {model}", text, count=1, flags=re.M)
            if n and new != text:
                f.write_text(new)
                changed.append(f"{name} -> {model}")
        print(f"agents alignes : {len(changed)}")
        for c in changed:
            print(f"  {c}")
        return 0

    ap.print_help()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
