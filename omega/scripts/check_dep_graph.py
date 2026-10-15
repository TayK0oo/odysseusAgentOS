#!/usr/bin/env python3
"""Garde-fou graphe de dependances du workspace Omega (deterministe, zero LLM).

Parse les `Cargo.toml` du workspace, construit le graphe inter-crates, refuse :
- tout cycle de dependances (y compris via dev-dependencies et target.*) ;
- toute arete interdite selon l'architecture du blueprint (§2.2).

Identite reelle d'une dependance :
- cle locale `alias = { package = "vrai-nom", ... }` -> le vrai nom est `package` ;
- `workspace = true` -> la definition est cherchee dans `[workspace.dependencies]` ;
- `path = "..."` -> le nom du crate cible est lu dans son `Cargo.toml`.

Code retour 0 si le graphe est valide, 1 sinon.
Mode `--self-test` : construit des workspaces volontairement invalides et
s'assure que chacun produit le code retour 1.
"""

from __future__ import annotations

import argparse
import shutil
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Dict, List, Set, Tuple


ARCH_RULES = {
    # Categorie -> set de categories cibles autorisees (y compris soi-meme si pertinent).
    "types": set(),
    "ports": {"types"},
    "config": {"types"},
    "protocol": {"types"},
    "core": {"types", "ports"},
    "registry": {"types", "ports", "config"},
    "plugin-api": {"types", "registry"},
    "router": {"types", "ports", "config", "adapters"},
    # Les adaptateurs (context, memory, tools, providers...) peuvent aussi fournir des
    # contributeurs de plugin (omega-plugin-api) et s'enregistrer dans le registre
    # (omega-registry) — cf. OMEGA-BLUEPRINT.md §2.1 (omega-context depend de registry).
    "adapters": {"types", "ports", "config", "registry", "plugin-api"},
    "plugins": {"types", "ports", "config", "registry"},
    "app": {"types", "ports", "config", "protocol", "core", "registry",
            "plugin-api", "adapters", "plugins", "router"},
    # `omega-server` (W7 inc2) est une racine de composition comme `app`, mais
    # au bord (HTTP) : il cable les adaptateurs via le registre (ports,
    # registry, plugin-api), execute via `app`/`core` (Operation, RunStatus),
    # route en deterministe (`router`), rejoue les journaux (`adapters` :
    # otel/context/providers) et expose les wire types (`protocol`).
    # Justification detaillee : `docs/omega/workstreams/W7-cockpit.md` (§5).
    # `config` est reserve au cablage futur de `ServerConfig` sur les couches
    # (profils/port) ; `app` autorise la composition sur l'assemblage existant.
    "app-edge": {"types", "ports", "config", "protocol", "core", "registry",
                 "plugin-api", "adapters", "router", "app"},
    "other": set(),  # pas de regle specifique, seulement l'absence de cycle
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
    if name == "omega-router":
        return "router"
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
    """Remonte jusqu'a trouver un Cargo.toml contenant `[workspace]`."""
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


def read_package_name(cargo_path: Path) -> str:
    """Lit le nom declare dans `[package] name` d'un Cargo.toml."""
    with cargo_path.open("rb") as f:
        data = tomllib.load(f)
    return data["package"]["name"]


def discover_crates(root: Path) -> Dict[str, Path]:
    """Decouvre les crates membres du workspace a partir de `[workspace] members`."""
    workspace_toml = root / "Cargo.toml"
    with workspace_toml.open("rb") as f:
        data = tomllib.load(f)

    crates: Dict[str, Path] = {}
    members = data.get("workspace", {}).get("members", [])
    for pattern in members:
        # Support simple : pattern termine par `/*` (ex. "crates/*").
        if pattern.endswith("/*"):
            base = root / pattern[:-2]
            if base.is_dir():
                for dir_ in sorted(base.iterdir()):
                    cargo = dir_ / "Cargo.toml"
                    if cargo.is_file():
                        name = read_package_name(cargo)
                        crates[name] = cargo
        else:
            cargo = root / pattern / "Cargo.toml"
            if cargo.is_file():
                name = read_package_name(cargo)
                crates[name] = cargo
    return crates


def read_workspace_dependencies(root: Path) -> Dict[str, object]:
    """Retourne la table `[workspace.dependencies]` si elle existe."""
    workspace_toml = root / "Cargo.toml"
    with workspace_toml.open("rb") as f:
        data = tomllib.load(f)
    return data.get("workspace", {}).get("dependencies", {})


def _is_under_root(path: Path, root: Path) -> bool:
    """Vrai si `path` est la racine ou un de ses descendants."""
    path = path.resolve()
    root = root.resolve()
    return path == root or root in path.parents


def _resolve_dep_name(
    key: str,
    spec: object,
    workspace_deps: Dict[str, object],
    member_crates: Dict[str, Path],
    root: Path,
    cargo_dir: Path,
) -> Tuple[str | None, Path | None]:
    """Determine le nom reel d'une dependance et son Cargo.toml si elle est interne.

    Une dependance est interne si :
    - c'est un membre du workspace ; ou
    - c'est une dependance `path` (directe ou via `[workspace.dependencies]`)
      pointant sous la racine du workspace (membre ou non).

    Retourne `(nom_interne, cargo_path)` ou `(None, None)`.
    """
    if isinstance(spec, dict):
        real_name = spec.get("package", key)

        if spec.get("workspace"):
            ws_spec = workspace_deps.get(real_name)
            if isinstance(ws_spec, dict):
                # Les chemins dans [workspace.dependencies] sont relatifs a la racine.
                if "path" in ws_spec:
                    path_dep = (root / ws_spec["path"]).resolve()
                    cargo = path_dep / "Cargo.toml"
                    if cargo.exists() and _is_under_root(path_dep, root):
                        try:
                            name = read_package_name(cargo)
                        except Exception:
                            return None, None
                        return name, cargo
                # Dependance workspace sans path : interne seulement si membre.
                if real_name in member_crates:
                    return real_name, member_crates[real_name]
                return None, None
            # `workspace = true` mais la dependance workspace est externe (version seule).
            if real_name in member_crates:
                return real_name, member_crates[real_name]
            return None, None

        # Dependance directe : path relatif au Cargo.toml du crate consommateur.
        if "path" in spec:
            path_dep = (cargo_dir / spec["path"]).resolve()
            cargo = path_dep / "Cargo.toml"
            if cargo.exists() and _is_under_root(path_dep, root):
                try:
                    name = read_package_name(cargo)
                except Exception:
                    return None, None
                return name, cargo
            return None, None

        # Tableau inline sans path/workspace : interne seulement si membre.
        if real_name in member_crates:
            return real_name, member_crates[real_name]
        return None, None

    # Spec simple : chaine de version ou "*" -> interne seulement si membre.
    if key in member_crates:
        return key, member_crates[key]
    return None, None


def parse_internal_deps(
    cargo_path: Path,
    workspace_deps: Dict[str, object],
    member_crates: Dict[str, Path],
    root: Path,
    crate_paths: Dict[str, Path],
) -> Set[str]:
    """Extrait les dependances internes depuis un Cargo.toml.

    Parcourt `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]` ainsi
    que leurs variantes `[target.'<cfg>'.*]`. Les crates decouverts par `path`
    (membres ou non) sont ajoutes a `crate_paths` pour inspection recursive.
    """
    with cargo_path.open("rb") as f:
        data = tomllib.load(f)

    cargo_dir = cargo_path.parent
    deps: Set[str] = set()

    sections: List[Tuple[str, Dict[str, object]]] = []
    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        table = data.get(section, {})
        if table:
            sections.append((section, table))

    for _cfg, cfg_table in data.get("target", {}).items():
        for section in ("dependencies", "dev-dependencies", "build-dependencies"):
            table = cfg_table.get(section, {})
            if table:
                sections.append((section, table))

    for _section_name, table in sections:
        for dep_name, spec in table.items():
            internal, cargo = _resolve_dep_name(
                dep_name, spec, workspace_deps, member_crates, root, cargo_dir
            )
            if internal is not None:
                deps.add(internal)
                if cargo is not None:
                    crate_paths.setdefault(internal, cargo)

    return deps


def detect_cycles(graph: Dict[str, Set[str]]) -> List[List[str]]:
    """Detecte les cycles par coloration DFS."""
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
                # Cycle trouve : on enregistre la portion grise.
                idx = stack.index(neighbor)
                cycle = stack[idx:] + [neighbor]
                cycles.append(cycle)
        stack.pop()
        color[node] = BLACK

    for node in sorted(graph):
        if color[node] == WHITE:
            dfs(node)
    return cycles


def check_workspace(root: Path) -> Tuple[int, List[str]]:
    """Verifie un workspace. Retourne (nombre de crates connus, liste d'erreurs)."""
    member_crates = discover_crates(root)
    if not member_crates:
        return 0, ["ERREUR : aucun crate decouvert"]

    workspace_deps = read_workspace_dependencies(root)

    # Graphe complet : membres + crates atteints par dependances `path` hors members.
    crate_paths: Dict[str, Path] = dict(member_crates)
    graph: Dict[str, Set[str]] = {}

    to_process = list(crate_paths.items())
    processed: Set[str] = set()

    while to_process:
        name, cargo_path = to_process.pop()
        if name in processed:
            continue
        processed.add(name)

        deps = parse_internal_deps(
            cargo_path, workspace_deps, member_crates, root, crate_paths
        )
        graph[name] = deps

        for dep_name in sorted(deps):
            dep_cargo = crate_paths.get(dep_name)
            if dep_cargo is not None and dep_name not in processed:
                to_process.append((dep_name, dep_cargo))

    errors: List[str] = []

    # 1. Cycles
    cycles = detect_cycles(graph)
    for cycle in cycles:
        errors.append(f"cycle de dependances : {' -> '.join(cycle)}")

    # 2. Aretes interdites
    for name, deps in sorted(graph.items()):
        src_cat = categorize(name, crate_paths[name])
        allowed = ARCH_RULES.get(src_cat, set())
        for dep in sorted(deps):
            dep_cat = categorize(dep, crate_paths[dep])
            if dep_cat not in allowed:
                errors.append(
                    f"arete interdite : {name} ({src_cat}) -> {dep} ({dep_cat})"
                )

    return len(crate_paths), errors


CrateFixture = Tuple[Dict[str, str], Dict[str, str], Dict[str, Dict[str, str]]]


def write_cargo_fixture(
    root: Path,
    crate_name: str,
    deps: Dict[str, str] | None = None,
    dev_deps: Dict[str, str] | None = None,
    target_deps: Dict[str, Dict[str, str]] | None = None,
    prefix: str = "crates",
) -> None:
    """Ecrit un Cargo.toml minimal de crate interne pour les fixtures.

    Les arguments sont des fragments TOML (ex. `{ workspace = true }`).
    Le parametre `prefix` permet de placer le crate hors `members`
    (ex. `prefix="hidden"`).
    """
    cargo_dir = root / prefix / crate_name
    cargo_dir.mkdir(parents=True, exist_ok=True)
    lines = [
        "[package]",
        f'name = "{crate_name}"',
        'version = "0.1.0"',
        "",
    ]

    def add_section(name: str, table: Dict[str, str]) -> None:
        lines.append(f"[{name}]")
        for dep_name, spec in table.items():
            lines.append(f'{dep_name} = {spec}')
        lines.append("")

    if deps:
        add_section("dependencies", deps)
    if dev_deps:
        add_section("dev-dependencies", dev_deps)
    if target_deps:
        for cfg, table in target_deps.items():
            add_section(f"target.'{cfg}'.dependencies", table)

    (cargo_dir / "Cargo.toml").write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def self_test() -> int:
    """Construit des workspaces invalides et verifie qu'ils sont tous detectes.

    Retourne 0 si tous les cas adversariaux produisent exit != 0, 1 sinon.
    """
    if shutil.which("python3") is None:
        print("SELF-TEST ERREUR : python3 non disponible", file=sys.stderr)
        return 1

    base_members: Dict[str, CrateFixture] = {
        "omega-types": ({}, {}, {}),
        "omega-ports": ({"omega-types": '{ workspace = true }'}, {}, {}),
        "omega-core": (
            {"omega-types": '{ workspace = true }', "omega-ports": '{ workspace = true }'},
            {},
            {},
        ),
    }
    # Crates presents dans le depot mais absents de `[workspace] members`.
    base_hidden: Dict[str, CrateFixture] = {
        "omega-registry": ({"omega-types": '{ workspace = true }'}, {}, {}),
    }

    cases: List[Tuple[str, Dict[str, CrateFixture], Dict[str, CrateFixture]]] = [
        (
            "dev-dependency interdite (core -> registry)",
            {"omega-core": ({}, {"omega-registry": '{ workspace = true }'}, {})},
            {},
        ),
        (
            "renommage package= (core -> registry)",
            {
                "omega-core": (
                    {"stealth": '{ package = "omega-registry", workspace = true }'},
                    {},
                    {},
                )
            },
            {},
        ),
        (
            "target.*.dependency interdite (core -> registry)",
            {
                "omega-core": (
                    {},
                    {},
                    {"cfg(unix)": {"omega-registry": '{ workspace = true }'}},
                )
            },
            {},
        ),
        (
            "cycle via dev-dependencies (types -> core)",
            {"omega-types": ({}, {"omega-core": '{ workspace = true }'}, {})},
            {},
        ),
        (
            "dependance path hors members (core -> registry)",
            {
                "omega-core": (
                    {"omega-registry": '{ path = "../../hidden/omega-registry" }'},
                    {},
                    {},
                )
            },
            {},
        ),
    ]

    failures = 0
    for case_name, overrides, hidden_overrides in cases:
        with tempfile.TemporaryDirectory(prefix="omega_dep_guard_") as tmp:
            tmp_root = Path(tmp)
            workspace_deps = {
                "omega-types": '{ path = "crates/omega-types" }',
                "omega-ports": '{ path = "crates/omega-ports" }',
                "omega-core": '{ path = "crates/omega-core" }',
                "omega-registry": '{ path = "hidden/omega-registry" }',
            }
            workspace_toml = [
                "[workspace]",
                'members = ["crates/*"]',
                "",
                "[workspace.dependencies]",
            ]
            for dep_name, spec in workspace_deps.items():
                workspace_toml.append(f'{dep_name} = {spec}')
            (tmp_root / "Cargo.toml").write_text(
                "\n".join(workspace_toml) + "\n", encoding="utf-8"
            )

            members = dict(base_members)
            members.update(overrides)
            for name, (deps, dev_deps, target_deps) in members.items():
                write_cargo_fixture(tmp_root, name, deps, dev_deps, target_deps)

            hidden = dict(base_hidden)
            hidden.update(hidden_overrides)
            for name, (deps, dev_deps, target_deps) in hidden.items():
                write_cargo_fixture(
                    tmp_root, name, deps, dev_deps, target_deps, prefix="hidden"
                )

            _count, errors = check_workspace(tmp_root)
            if not errors:
                print(
                    f"SELF-TEST ECHEC : '{case_name}' n'a detecte aucune violation",
                    file=sys.stderr,
                )
                failures += 1
            else:
                print(f"SELF-TEST OK : '{case_name}' -> {len(errors)} violation(s) detectee(s)")

    if failures:
        print(f"SELF-TEST ECHEC : {failures}/{len(cases)} cas non detectes", file=sys.stderr)
        return 1

    print(f"SELF-TEST OK : {len(cases)}/{len(cases)} cas adversariaux detectes.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Garde-fou graphe de dependances du workspace Omega."
    )
    parser.add_argument(
        "workspace",
        nargs="?",
        help="Chemin vers la racine du workspace Cargo (par defaut : dossier parent du script).",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Construit des fixtures invalides et verifie qu'elles sont detectees.",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if args.workspace:
        start = Path(args.workspace)
    else:
        start = Path(__file__).resolve().parent.parent
    root = find_workspace_root(start)
    if root is None:
        print(f"ERREUR : aucun workspace Cargo trouve depuis {start}", file=sys.stderr)
        return 1

    count, errors = check_workspace(root)
    if errors:
        print(
            f"Graphe de dependances invalide ({count} crates, {len(errors)} violation(s)) :",
            file=sys.stderr,
        )
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(f"Graphe de dependances valide ({count} crates, 0 cycle, 0 arete interdite).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
