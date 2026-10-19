#!/usr/bin/env python3
"""P0-2 — mesure honnête par parseurs (leçon L3 : jamais de comptage de sous-chaînes sans étiquette de méthode).

Génère docs/omega/plan/STATUS.json + STATUS.md (Annexe C du PLAN-MAITRE).
Ce fichier mesure, il ne juge pas : les portes G* restent ROUGES tant que `gate.py` (P0-5) ne les a pas exécutées.

Limites connues :
# ponytail: tests Rust comptés par scan d'attributs — un faux dans un doc-comment biaise; upgrade = `cargo test -- --list` en CI nocturne.
"""

import ast
import json
import shutil
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]  # racine du dépôt omega
STATUS_DIR = ROOT / "docs" / "omega" / "plan"
CARGO = ROOT / "omega" / "Cargo.toml"
TEST_ATTRS = ("#[test]", "#[tokio::test]")


def git_sha() -> str:
    try:
        return subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT,
                              capture_output=True, text=True, check=True).stdout.strip()
    except Exception as e:
        return f"inconnu({e})"


def count_rust_tests() -> tuple[int, list[str]]:
    """Nombre d'attributs test par crate. Méthode déclarée dans la sortie."""
    detail = []
    total = 0
    for src_dir in sorted((ROOT / "omega" / "crates").glob("*/src")):
        n = 0
        for rs in src_dir.rglob("*.rs"):
            text = rs.read_text(errors="replace")
            n += sum(text.count(m) for m in TEST_ATTRS)
        total += n
        if n:
            detail.append(f"{src_dir.parent.name}:{n}")
    return total, detail


def count_python_tests() -> tuple[int, str]:
    """AST parse de tests/ (arbre Python complet du dépôt)."""
    tests_dir = ROOT / "tests"
    if not tests_dir.is_dir():
        return 0, "absent(tests/)"
    n = 0
    for py in tests_dir.rglob("*.py"):
        try:
            tree = ast.parse(py.read_text(errors="replace"))
        except SyntaxError:
            continue  # fichier cassé : sera signalé par la porte verify (P0-1)
        n += sum(1 for node in ast.walk(tree)
                 if isinstance(node, (ast.FunctionDef | ast.AsyncFunctionDef))
                 and node.name.startswith("test_"))
    return n, "ast"


def run_script(script: str, *args: str) -> dict:
    p = ROOT / "omega" / "scripts" / script
    try:
        r = subprocess.run([sys.executable, str(p), *args], cwd=ROOT,
                           capture_output=True, text=True, timeout=60, check=False)
        extrait = (r.stdout or r.stderr).strip()[-300:]
        return {"verdict": "OK" if r.returncode == 0 else f"ECHEC({r.returncode})",
                "extrait": extrait}
    except Exception as e:
        return {"verdict": f"SKIPPED({e})", "extrait": ""}



def count_ruff_legacy() -> "tuple[int | None, str]":
    """Compte les erreurs ruff hors omega/ (socle legacy, dette P1 — ADR-0005).

    R Opportun : None si ruff absent (jamais une valeur inventee).
    """
    if shutil.which("ruff") is None:
        return None, "ruff_absent"
    try:
        v = subprocess.run(["ruff", "--version"], capture_output=True,
                           text=True, timeout=30).stdout.strip().split()[-1]
        r = subprocess.run(["ruff", "check", ".", "--output-format=json"],
                           cwd=ROOT, capture_output=True, text=True, timeout=300)
        erreurs = json.loads(r.stdout)
        legacy = [e for e in erreurs
                  if not e["filename"].split("/odysseusAgentOS-omega/")[-1].startswith("omega/")]
        return len(legacy), f"ruff_json@{v}"
    except Exception as e:
        return None, f"ruff_erreur:{type(e).__name__}"


def main() -> int:
    crates = []
    if CARGO.exists():
        r = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1",
             "--manifest-path", str(CARGO)],
            capture_output=True, text=True, check=False)
        if r.returncode == 0:
            crates = sorted(p.get("name") for p in json.loads(r.stdout)["packages"])
    rust_tests, detail_rust = count_rust_tests()
    python_tests, methode_python = count_python_tests()

    ruff_legacy, methode_ruff = count_ruff_legacy()

    status = {
        "git_sha": git_sha(),
        "mesure_le": datetime.now(UTC).isoformat(timespec="seconds"),
        "moteur_reference": "INDETERMINE (ADR D0, P0-9)",
        "socles": {
            "rust": {
                "crates": len(crates), "liste": crates,
                "tests": {"valeur": rust_tests, "methode": "scan_attributs",
                          "detail_par_crate": detail_rust},
            },
            "python": {
                "tests": {"valeur": python_tests, "methode": methode_python},
                "ruff_legacy_erreurs": {"valeur": ruff_legacy, "methode": methode_ruff,
                                        "note": "dette P1, non bloquante (ADR-0005)"},
            },
        },
        "portes_federees": {
            "model_policy": run_script("model_policy.py", "--check"),
            "dep_graph": run_script("check_dep_graph.py"),
        },
        "registre": {"plugins_attendus": None, "plugins_enregistres": None,
                     "orphelins": None, "note": "mesuré en P0-6 (détecteurs)"},
        "scenarios": {f"S{i}": {"scripte": "NON_EXECUTE", "reel": "NON_EXECUTE"}
                      for i in range(12)},
        "couverture_ponderee": 0.0,
        "portes": {f"G{i}": "ROUGE" for i in range(8)},
        "prochaine_action": "P0-4 : banc d'acceptation (runner boite noire, ScriptedProvider, S0 vert / S1 xfail nomme)",
    }

    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    (STATUS_DIR / "STATUS.json").write_text(
        json.dumps(status, indent=2, ensure_ascii=False) + "\n")

    s = status["socles"]
    md = "\n".join([
        "# STATUS — AgentOS Omega (généré par measure.py — jamais édité à la main)",
        "",
        f"SHA `{status['git_sha']}` · mesuré le {status['mesure_le']} · moteur {status['moteur_reference']}",
        "",
        "| Mesure | Valeur | Méthode |",
        "|---|---|---|",
        f"| Crates Rust | {s['rust']['crates']} | cargo metadata |",
        f"| Tests Rust (attributs test) | {s['rust']['tests']['valeur']} | scan attributs |",
        f"| Tests Python (test_*) | {s['python']['tests']['valeur']} | {s['python']['tests']['methode']} |",
        f"| model_policy --check | {status['portes_federees']['model_policy']['verdict']} | script |",
        f"| dep_graph | {status['portes_federees']['dep_graph']['verdict']} | script |",
        "",
        "**Scénarios** : S0–S11 `NON_EXECUTE` (banc d'acceptation = P0-4).",
        "**Portes** : G0–G7 `ROUGE` — seule la sortie de gate.py fait foi (leçon L6).",
        f"**Prochaine action** : {status['prochaine_action']}",
    ]) + "\n"
    (STATUS_DIR / "STATUS.md").write_text(md)

    print(json.dumps({"socles": s, "portes_federees": status["portes_federees"],
                      "prochaine_action": status["prochaine_action"]},
                     indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
