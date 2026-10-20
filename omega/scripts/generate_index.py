#!/usr/bin/env python3
"""P0-3d : régénère les sections machine de INDEX.md.

Les sections balisées ``<!-- gen:<id> -->``/``<!-- /gen -->`` sont réécrites
par ce script ; tout le reste de la carte est maintenu à la main, intact.
Chiffre généré, jamais saisi (principe 7) — les données viennent de
``STATUS.json`` (measure.py) ou du système de fichiers.
"""
import ast
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "docs" / "omega" / "INDEX.md"
STATUS_PATH = ROOT / "docs" / "omega" / "plan" / "STATUS.json"

BALISE = re.compile(r"(<!-- gen:(\w+) -->\n)(.*?)(<!-- /gen -->)", re.S)


def git_sha() -> str:
    return subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"],
        cwd=ROOT, capture_output=True, text=True, check=False,
    ).stdout.strip()


def arbre_adr() -> str:
    adr_dir = ROOT / "docs" / "omega" / "adr"
    lignes = []
    for adr in sorted(adr_dir.glob("*.md")):
        tete = adr.read_text(encoding="utf-8").split("\n", 4)
        m = re.match(r"# ADR\s*-?\s*(\d{4}) — (.+)", tete[0] if tete else "")
        if not m:
            continue
        sujet = re.sub(r"\s*\(.+\)$", "", m.group(2))
        statut_ligne = next((s for s in tete if "Statut" in s), "").lower()
        decisions = {"accepté": "✅ Accepté", "proposé": "🟡 Proposé", "déprécié": "❌ Déprécié"}
        statut_court = next((v for k, v in decisions.items() if k in statut_ligne), "") or "—"
        lignes.append(f"| {m.group(1)} | {sujet} | {statut_court} |")
    return "\n".join(lignes)


def arbre_scripts() -> str:
    script_dir = ROOT / "omega" / "scripts"
    lignes = []
    for script in sorted(script_dir.glob("*.py")):
        if script.name.startswith("__"):
            continue
        # Role = 1re ligne de la docstring du module (ast, pas une regex :
        # `"""` court-circuite un lazy match sur `"`).
        try:
            doc = ast.get_docstring(ast.parse(script.read_text(encoding="utf-8")))
        except SyntaxError:
            doc = None
        role = doc.strip().split("\n")[0].strip() if doc else "—"
        lignes.append(f"| `{script.name}` | {role} |")
    lignes.append("| `run-local.sh` | Lance système complet (UI + API) |")
    return "\n".join(lignes)


def arbre_rust(status: dict) -> str:
    rust = status.get("socles", {}).get("rust", {})
    crates = rust.get("liste", [])
    lignes = [f"### Crates ({len(crates)}) — mesurés par `measure.py` [observed]", "```"]
    lignes.append(", ".join(crates))
    lignes.append("```")
    return "\n".join(lignes)


def arbres(status: dict) -> dict:
    return {
        "adr": arbre_adr(),
        "scripts": arbre_scripts(),
        "rust": arbre_rust(status),
    }


def banner() -> str:
    sha = git_sha()
    return (
        f"> **Généré le** par `omega/scripts/generate_index.py` à HEAD `{sha}` [observed].\n"
        f"> Les sections balisées `<!-- gen:... -->` sont régénérées à chaque run ; le reste\n"
        f"> de la carte est maintenu à la main. Chiffres vivants : `docs/omega/plan/STATUS.json`.\n"
    )


def main() -> int:
    status = json.loads(STATUS_PATH.read_text(encoding="utf-8"))
    sections = arbres(status)
    texte = INDEX.read_text(encoding="utf-8")

    # Bannière : remplace tout ce qui suit le H1 jusqu'au premier "---".
    texte = re.sub(r"^#.*?\n---\n", f"# {INDEX.stem} — Espace Omega (carte)\n\n" + banner() + "\n---\n",
                   texte, count=1, flags=re.S)

    def remplace(m: re.Match) -> str:
        _debut, ident, _contenu, fin = m.group(1), m.group(2), m.group(3), m.group(4)
        corps = sections.get(ident)
        if corps is None:
            return m.group(0)  # section inconnue : on n'y touche pas
        return f"{m.group(1)}\n{corps}\n\n{fin}"

    texte, n = BALISE.subn(remplace, texte)
    if n == 0:
        raise SystemExit("Aucune section <!-- gen:... --> balisée dans INDEX.md — refus d'éditer")
    INDEX.write_text(texte, encoding="utf-8")
    print(f"OK : {n} section(s) régénérée(s) dans {INDEX.name} ({len(sections)} disponibles)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
