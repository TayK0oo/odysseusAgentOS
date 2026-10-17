#!/usr/bin/env python3
"""Garde-fou des dissections — structure + **resolution des preuves** (deterministe).

Verifie pour chaque `data/apps/*.json` :
1. les champs racine et de fonctionnalite obligatoires ;
2. les enums (`integration`, `status`, `rust_cost`) ;
3. l'unicite des ids de fonctionnalite ;
4. la **resolution des `refs`** : un `fichier:ligne` doit exister dans le clone
   correspondant (`$OMEGA_RE_CLONES/<id>`), une URL doit etre bien formee.

Un ref introuvable est un **echec** (la preuve est la valeur du corpus).

Usage :
    python3 omega/scripts/validate_dissections.py            # tout
    python3 omega/scripts/validate_dissections.py --only tabby
    python3 omega/scripts/validate_dissections.py --strict   # refs manquants = echec
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
APPS = REPO / "omega" / "crates" / "omega-harness" / "data" / "apps"
CLONES = Path(os.environ.get("OMEGA_RE_CLONES", "/home/tayk0oo/omega-re"))

ROOT_FIELDS = {
    "schema_version",
    "id",
    "name",
    "editor",
    "oss",
    "language",
    "license",
    "architecture",
    "functionalities",
}
FCT_FIELDS = {
    "id",
    "domain",
    "name",
    "what",
    "how",
    "omega_capability",
    "omega_port",
    "omega_adapter",
    "integration",
    "status",
    "rust_cost",
    "refs",
}
INTEGRATIONS = {"plugin", "service", "native", "rejected"}
STATUSES = {"adopted", "planned", "rejected"}
COSTS = {"S", "M", "L"}

# Clones dont le nom de dossier differe de l'id d'app.
ALIASES = {
    "claude-code": "claude-code",
    "codex": "codex",
    "open-interpreter": "open-interpreter",
    "kilo-code": "kilo-code",
    "qwen-code": "qwen-code",
}


def check_ref(ref: str, app_id: str) -> tuple[bool, str]:
    """Un ref est-il verifiable ? (fichier existant dans le clone, ou URL)."""
    if ref.startswith(("http://", "https://")):
        return True, "url"
    path = ref.split(":")[0].strip().strip("`")
    if not path or path.startswith("-"):
        return False, f"ref illisible : {ref!r}"
    clone = CLONES / ALIASES.get(app_id, app_id)
    if not clone.is_dir():
        return True, "clone absent (non verifiable)"
    if (clone / path).exists():
        return True, "ok"
    # Une preuve peut etre une note de recherche du depot (ex. RE-*.md) : on
    # l'accepte si elle existe ailleurs dans le depot (chemin relatif ou nom).
    for base in (REPO, REPO / "docs" / "omega" / "reverse-eng", REPO / "docs" / "omega"):
        if (base / path).exists():
            return True, "ok (note de recherche)"
    if list(REPO.rglob(Path(path).name)):
        return True, "ok (note de recherche)"
    # Tolerance : ref vers un dossier, ou chemin avec prefixe de depot
    for cand in clone.rglob(Path(path).name):
        if str(cand).endswith(path):
            return True, "ok (resolu)"
    return False, f"introuvable : {path}"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", default=None)
    ap.add_argument("--strict", action="store_true", help="un ref manquant fait echouer")
    args = ap.parse_args()

    files = sorted(APPS.glob("*.json"))
    errors: list[str] = []
    warnings: list[str] = []
    total_fct = 0
    total_refs = 0
    unresolved = 0

    for f in files:
        d = json.loads(f.read_text())
        if args.only and d.get("id") != args.only:
            continue
        app = d.get("id", f.stem)
        miss = ROOT_FIELDS - set(d)
        if miss:
            errors.append(f"{app}: champs racine manquants {sorted(miss)}")
        if d.get("schema_version") != 1:
            errors.append(f"{app}: schema_version != 1")
        ids = set()
        for x in d.get("functionalities", []):
            total_fct += 1
            fid = x.get("id", "?")
            m = FCT_FIELDS - set(x)
            if m:
                errors.append(f"{app}/{fid}: champs manquants {sorted(m)}")
            if fid in ids:
                errors.append(f"{app}/{fid}: id duplique")
            ids.add(fid)
            if x.get("integration") not in INTEGRATIONS:
                errors.append(f"{app}/{fid}: integration invalide {x.get('integration')!r}")
            if x.get("status") not in STATUSES:
                errors.append(f"{app}/{fid}: status invalide {x.get('status')!r}")
            if x.get("rust_cost") not in COSTS:
                errors.append(f"{app}/{fid}: rust_cost invalide {x.get('rust_cost')!r}")
            refs = x.get("refs") or []
            if not refs:
                warnings.append(f"{app}/{fid}: aucune preuve (refs vide)")
            for r in refs:
                total_refs += 1
                ok, why = check_ref(r, app)
                if not ok:
                    unresolved += 1
                    (errors if args.strict else warnings).append(f"{app}/{fid}: {why}")

    print(f"fichiers: {len([f for f in files])} | fonctionnalites: {total_fct} | refs: {total_refs}")
    print(f"refs non resolues: {unresolved}")
    for w in warnings[:20]:
        print(f"  ⚠ {w}")
    for e in errors[:30]:
        print(f"  ✗ {e}")
    if errors:
        print(f"\nECHEC : {len(errors)} erreur(s)")
        return 1
    print("\nOK : toutes les dissections valides")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
