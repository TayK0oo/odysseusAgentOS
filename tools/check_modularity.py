"""Porte de modularité (MOD-11) — le job qui rend MOD-1 à MOD-10 conduisables.

Dix chantiers de modularité étaient listés sans qu'aucun ne soit mesuré. Le
résultat est que `core.database` est passé de 64 à 120 importeurs sans qu'une
seule porte ne le voie. MOD-11 n'est donc pas une routine : c'est la
condition pour que les dix autres deviennent pilotables, et c'est pourquoi il
vient en premier dans le plan.

**Trois seuils, et ce que chacun dit.**

| Seuil | Valeur | Ce qu'il dit |
|---|---|---|
| god node `core.database` | **120** fichiers | MOD-6 part d'ici. Toute hausse est une régression. |
| arêtes `src/* → routes.*` | **30** | MOD-2. C'est la violation de couches, et elle est mesurée *après* retrait des cas légitimes. |
| imports statiques de `route_loader` | **54** | MOD-1. Le passage à `pkgutil` fera baisser ce nombre ; ici on gèle la hausse. |

**Deux totirs de mesure, et ils sont.delta.** Une arête `routes.*` tirée depuis
un **test** est légitime : un test de route doit importer sa route. Une arête
depuis `core/route_loader.py` l'est aussi : c'est le chargeur. Compter les deux
donnerait 421, et surtout un seuil qu'on contourne en supprimant un test — la
porte mesurerait la mauvaise grandeur et serait gamingable. La violation réelle
est de **30**, dans 11 fichiers.

Le seuil est « ne pas augmenter », jamais « être sous un ideal » : l'idéal est
MOD-6 et MOD-2, qui demanderont des dizaines de commits. Une porte qui exige la
fin du chantier dès le premier jour est une porte qu'on contourne.

Usage :
    python tools/check_modularity.py           # exit 1 si un seuil est franchi
    python tools/check_modularity.py --json    # compte pour le rapport de gouvernance
    python tools/check_modularity.py --explain # le détail des trois grandeurs
"""

from __future__ import annotations

import argparse
import ast
import collections
import json
import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent

# ── Les seuils ──────────────────────────────────────────────────────────────
# Chaque ligne dit ce qu'elle protège et de quel chantier elle vient. Un seuil
# sans chantier associé est un nombre que personne ne contestera ni ne fera
# baisser : c'est une constante décorative.

SEUILS = {
    "core_database_importeurs": {
        "valeur": 120,
        "chantier": "MOD-6 — contrat DB (Protocol/repository) sur core.database",
        "raison": "2e nœud le plus importé du dépôt, après la stdlib. C'est le god node de fait.",
        "releve_si": None,
    },
    "aretes_src_vers_routes": {
        "valeur": 30,
        "chantier": "MOD-2 — inverser les arêtes src/* → routes.*",
        "raison": (
            "Violation de couches, hors tests (un test de route importe sa route) et hors "
            "route_loader (c'est le chargeur). 30 arêtes dans 11 fichiers, dont "
            "builtin_actions.py pour 11 à lui seul."
        ),
        "releve_si": None,
    },
    "route_loader_imports_statiques": {
        "valeur": 54,
        "chantier": "MOD-1 — route_loader réellement dynamique (pkgutil)",
        "raison": "La découverte dynamique remplacera ces 54 imports. Ici on gèle la hausse.",
        "releve_si": None,
    },
}

GOD_NODE = "core.database"


def _rel(p: pathlib.Path, root: pathlib.Path = REPO) -> str:
    """Chemin affichable, relatif au dépôt quand il en est un.

    Un outil de mesure ne doit pas planter sur un chemin inattendu : il doit le
    nommer. Sans cela il n'est testable que depuis le dépôt lui-même, donc
    impossible a faire tomber proprement — ce qui est le seul moyen de prouver
    qu'une porte tombe.
    """
    try:
        return str(p.relative_to(root))
    except ValueError:
        return str(p)


def _dans_le_dossier_de_tete(p: pathlib.Path, dossier: str, root: pathlib.Path) -> bool:
    """Le fichier est-il dans le `dossier/` de premier niveau de `root` ?

    « Premier niveau » est nécessaire : `packages/sfd-*/src/` existe, dix fois.
    Un test par « `src` apparait dans le chemin » engloberait ces paquets et
    ferait bouger la metrique a chaque `.py` qu'on y ajoute — c'est-a-dire une
    porte dont le seuil depend d'un depot qu'elle ne mesure pas.

    `root` est injectable, et il n'y a **aucun repli** hors dépôt : la même
    règle s'applique partout. C'est ce qui rend la porte testable depuis un
    temporaire sans relacher la precision. Un repli « le nom suffit » aurait
    exactement le défaut qu'on cherche a eviter, et il serait invisible — un
    `packages/*/src/` temporaire serait alors compté comme une violation.
    """
    try:
        return p.relative_to(root).parts[0] == dossier
    except (ValueError, IndexError):
        return False


def _fichiers_py() -> list[pathlib.Path]:
    return [
        p
        for p in REPO.rglob("*.py")
        if "venv" not in p.parts and "node_modules" not in p.parts
    ]


def _arbre(p: pathlib.Path):
    try:
        return ast.parse(p.read_text(encoding="utf-8", errors="replace"))
    except SyntaxError:
        return None


def importeurs_de_god_node(module: str = GOD_NODE, root: pathlib.Path = REPO) -> list[str]:
    rx = re.compile(rf"(?:^|\n)\s*(?:from\s+{re.escape(module)}\b|import\s+{re.escape(module)}\b)")
    return [_rel(p, root) for p in _fichiers_py() if rx.search(p.read_text(encoding="utf-8", errors="replace"))]


def aretes_src_vers_routes(root: pathlib.Path = REPO) -> tuple[int, list[tuple[str, int]]]:
    """Arêtes `src/* → routes.*`, cas légitimes exclus.

    Le chargeur de routes est exclu parce que son métier EST d'importer les
    routes ; sans cette exclusion, MOD-1 et MOD-2 se contrediraient — corriger
    l'un ferait bouger le compte de l'autre.
    """
    par_fichier: collections.Counter = collections.Counter()
    for p in _fichiers_py():
        if not _dans_le_dossier_de_tete(p, "src", root):
            continue
        rel = _rel(p, root)
        arbre = _arbre(p)
        if arbre is None:
            continue
        for n in ast.walk(arbre):
            c = 0
            if isinstance(n, ast.ImportFrom) and n.module and n.module.startswith("routes"):
                c = 1
            elif isinstance(n, ast.Import):
                c = sum(1 for a in n.names if a.name.startswith("routes."))
            par_fichier[rel] += c
    return sum(par_fichier.values()), sorted(par_fichier.items(), key=lambda kv: -kv[1])


def imports_statiques_route_loader() -> int:
    rl = (REPO / "core/route_loader.py")
    if not rl.is_file():
        return 0
    return len(re.findall(r"^\s*from routes\.", rl.read_text(encoding="utf-8", errors="replace"), re.M))


def route_loader_dynamique() -> bool:
    rl = (REPO / "core/route_loader.py")
    if not rl.is_file():
        return False
    t = rl.read_text(encoding="utf-8", errors="replace")
    return "pkgutil" in t or "iter_modules" in t


def mesure(root: pathlib.Path = REPO) -> dict[str, int]:
    return {
        "core_database_importeurs": len(importeurs_de_god_node(root=root)),
        "aretes_src_vers_routes": aretes_src_vers_routes(root=root)[0],
        "route_loader_imports_statiques": imports_statiques_route_loader(),
    }


def main(racine: pathlib.Path | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true", help="compte machine, pour le rapport de gouvernance")
    ap.add_argument("--explain", action="store_true", help="le détail des trois grandeurs")
    ap.add_argument(
        "--update",
        action="store_true",
        help="relever les seuils a l'etat mesure (demande une justification)",
    )
    args = ap.parse_args()

    root = racine or REPO
    m = mesure(root)

    if args.json:
        print(json.dumps(m, ensure_ascii=False, sort_keys=True))
        return 0

    if args.explain:
        print("god node, top des importateurs :")
        importeurs = importeurs_de_god_node(root=root)
        print(f"  {GOD_NODE} : {m['core_database_importeurs']} fichier(s), dont "
              f"{len([f for f in importeurs if not f.startswith('tests/')])} de production")
        total, detail = aretes_src_vers_routes(root)
        reels = [(f, c) for f, c in detail if c]
        print(f"\narêtes src/* → routes.* : {total} dans {len(reels)} fichier(s) "
              f"({len(detail) - len(reels)} fichier(s) src/ n'en ont aucune)")
        for f, c in reels:
            print(f"  {c:4}  {f}")
        print(f"\nroute_loader : {m['route_loader_imports_statiques']} imports statiques, "
              f"decouverte dynamique = {route_loader_dynamique()}")
        return 0

    if args.update:
        # Relever n'est pas gratuit : une hausse doit se justifier. On exige donc
        # une raison non vide, sinon le seuil devient un nombre qu'on inflacje pour
        # faire passer la porte — c'est-a-dire une porte qui se contourne seule.
        for cle, seuil in SEUILS.items():
            if m[cle] > seuil["valeur"]:
                motif = input(f"  {cle} passe de {seuil['valeur']} a {m[cle]} — pourquoi ? ") if sys.stdin.isatty() else ""
                if not motif.strip():
                    print(f"  {cle} : refuse, aucune raison donnee. Le seuil reste {seuil['valeur']}.")
                    return 1
                print(f"  {cle} : {motif.strip()}")
        print("\nMise a jour a faire manuellement dans SEUILS, avec la justification dans la ligne.")
        return 0

    deborde = []
    for cle, seuil in SEUILS.items():
        if m[cle] > seuil["valeur"]:
            deborde.append((cle, m[cle], seuil))

    if deborde:
        print("SEUILS FRANCHIS :")
        for cle, valeur, seuil in deborde:
            print(f"\n  {cle} : {valeur} > {seuil['valeur']}  (+{valeur - seuil['valeur']})")
            print(f"    chantier : {seuil['chantier']}")
        print("\nSoit vous corrigez, soit le chantier a Change et le seuil doit etre releve :")
        print("    python tools/check_modularity.py --update")
        return 1

    for cle, seuil in SEUILS.items():
        marge = seuil["valeur"] - m[cle]
        etat = "au seuil" if marge == 0 else f"marge {marge}"
        print(f"  {cle:34} {m[cle]:5} / {seuil['valeur']:<5} ({etat})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
