#!/usr/bin/env python3
"""Garde-fou graphe de dépendances du workspace Omega (déterministe, zéro LLM).

Parse les `Cargo.toml` du workspace, construit le graphe inter-crates, refuse :
- tout cycle de dépendances ;
- toute arête interdite selon l'architecture du blueprint (§2.2).

Code retour 0 si le graphe est valide, 1 sinon.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path
from typing import Dict, List, Set, Tuple


ARCH_RULES = {
    # Catégorie -> set de catégories cibles autorisées (y compris soi-même si pertinent).
    "types": set(),
    "ports": {"types"},
    "config": {"types"},
    "protocol": {"types"},
    "core": {"types", "ports"},
    "registry": {"types", "ports", "config"},
    "plugin-api": {"types", "registry"},
    "adapters": {"types", "ports", "config"},
    "plugins": {"types", "ports", "config", "registry"},
    "app": {"types", "ports", "config", "protocol", "core", "registry",
            "plugin-api", "adapters", "plugins"},
    "app-edge": {"types", "app", "protocol"},
    "other": set(),  # pas de règle spécifique, seulement l'absence de cycle
}


def categorize(name: str, path: Path) -> str:
    """Classe un crate selon son nom ou son emplacement."""
    if name == "omega-types":
        return "types"
    if name == "omega-ports":
        return "ports"
    if name == "omega-config":
        return "config"
    if name == "omega-protocol":
        return "protocol"
    if name == "omega-core":
        return "core"
    if name == "omega-registry":
        return "registry"
    if name == "omega-plugin-api":
        return "plugin-api"
    if name == "omega-app":
        return "app"
    if name in ("omega-server", "omega-cli", "omega-ui-bindings"):
        return "app-edge"
    if name.startswith(("omega-context", "omega-memory", "omega-tools",
                        "omega-providers", "omega-sandbox", "omega-otel")):
        return "adapters"
    if name.startswith(("harness-", "plugin-")):
        return "plugins"
    # Chemin explicite sous plugins/
    try:
        rel = path.relative_to(path.parents[1])
        if str(rel).startswith("plugins/"):
            return "plugins"
    except ValueError:
        pass
    return "other"


def find_workspace_root(start: Path) -> Path | None:
    """Remonte jusqu'à trouver un Cargo.toml contenant `[workspace]`."""
    current = start.resolve()
    while True:
        candidate = current / "Cargo.toml"
        if candidate.exists():
            text = candidate.read_text(encoding="utf-8")
            if "[workspace]" in text:
                return current
        parent = current.parent
        if parent == current:
            return None
        current = parent


def discover_crates(root: Path) -> Dict[str, Path]:
    """Découvre les crates membres du workspace."""
    workspace_toml = root / "Cargo.toml"
    with workspace_toml.open("rb") as f:
        data = tomllib.load(f)

    crates: Dict[str, Path] = {}
    members = data.get("workspace", {}).get("members", [])
    for pattern in members:
        for cargo in root.glob(pattern + "/Cargo.toml"):
            name = cargo.parent.name
            crates[name] = cargo
    return crates


def parse_internal_deps(cargo_path: Path) -> Set[str]:
    """Extrait les dépendances internes au workspace depuis un Cargo.toml."""
    with cargo_path.open("rb") as f:
        data = tomllib.load(f)

    deps: Set[str] = set()
    for section in ("dependencies", "build-dependencies"):
        for dep_name, spec in data.get(section, {}).items():
            if not dep_name.startswith("omega-"):
                continue
            # Si c'est une table, on vérifie qu'il s'agit bien d'une dépendance de workspace
            if isinstance(spec, dict):
                if "path" in spec or "workspace" in spec:
                    deps.add(dep_name)
            elif spec == "*" or isinstance(spec, str):
                # dépendance simple : on considère qu'elle est interne si omega-*
                deps.add(dep_name)
    return deps


def detect_cycles(graph: Dict[str, Set[str]]) -> List[List[str]]:
    """Détecte les cycles par coloration DFS."""
    WHITE, GRAY, BLACK = 0, 1, 2
    color: Dict[str, int] = {node: WHITE for node in graph}
    cycles: List[List[str]] = []
    stack: List[str] = []

    def dfs(node: str) -> None:
        color[node] = GRAY
        stack.append(node)
        for neighbor in sorted(graph.get(node, set())):
            if color.get(neighbor, WHITE) == WHITE:
                dfs(neighbor)
            elif color.get(neighbor) == GRAY:
                # Cycle trouvé : on enregistre la portion grise.
                idx = stack.index(neighbor)
                cycle = stack[idx:] + [neighbor]
                cycles.append(cycle)
        stack.pop()
        color[node] = BLACK

    for node in sorted(graph):
        if color[node] == WHITE:
            dfs(node)
    return cycles


def main() -> int:
    start = Path.cwd()
    root = find_workspace_root(start)
    if root is None:
        print(f"ERREUR : aucun workspace Cargo trouvé depuis {start}", file=sys.stderr)
        return 1

    crates = discover_crates(root)
    if not crates:
        print("ERREUR : aucun crate découvert", file=sys.stderr)
        return 1

    graph: Dict[str, Set[str]] = {}
    for name, path in crates.items():
        graph[name] = parse_internal_deps(path)

    errors: List[str] = []

    # 1. Cycles
    cycles = detect_cycles(graph)
    for cycle in cycles:
        errors.append(f"cycle de dépendances : {' -> '.join(cycle)}")

    # 2. Arêtes interdites
    for name, deps in sorted(graph.items()):
        src_cat = categorize(name, crates[name])
        allowed = ARCH_RULES.get(src_cat, set())
        for dep in sorted(deps):
            dep_cat = categorize(dep, crates.get(dep, root / "Cargo.toml"))
            if dep_cat not in allowed:
                errors.append(
                    f"arête interdite : {name} ({src_cat}) -> {dep} ({dep_cat})"
                )

    if errors:
        print(f"Graphe de dépendances invalide ({len(errors)} violation(s)) :", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(f"Graphe de dépendances valide ({len(crates)} crates, 0 cycle, 0 arête interdite).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
