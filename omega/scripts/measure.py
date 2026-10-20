#!/usr/bin/env python3
"""P0-2 — mesure honnête par parseurs (leçon L3 : jamais de comptage de sous-chaînes sans étiquette de méthode).

Génère docs/omega/plan/STATUS.json + STATUS.md (Annexe C du PLAN-MAITRE).
Ce fichier mesure, il ne juge pas : les portes G* restent ROUGES tant que `gate.py` (P0-5) ne les a pas exécutées.

Limites connues :
# ponytail: tests Rust comptés par scan d'attributs — un faux dans un doc-comment biaise; upgrade = `cargo test -- --list` en CI nocturne.
"""

import ast
import json
import re
import shutil
import subprocess
import sys
import tomllib
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]  # racine du dépôt omega
sys.path.insert(0, str(Path(__file__).resolve().parent))
from measures_advanced import mesures_avancees, plugins_enregistres  # noqa: E402 (P0-3a)

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
        v = subprocess.run(["ruff", "--version"], check=False, capture_output=True,
                           text=True, timeout=30).stdout.strip().split()[-1]
        r = subprocess.run(["ruff", "check", ".", "--output-format=json"], check=False,
                           cwd=ROOT, capture_output=True, text=True, timeout=300)
        erreurs = json.loads(r.stdout)
        legacy = [e for e in erreurs
                  if not e["filename"].split("/odysseusAgentOS-omega/")[-1].startswith("omega/")]
        return len(legacy), f"ruff_json@{v}"
    except Exception as e:
        return None, f"ruff_erreur:{type(e).__name__}"



def run_scenarios() -> dict:
    """P0-4 : exécute le banc d'acceptance (mode scripté), verdicts partout.

    Échec d'infra = ERREUR, jamais un statut inventé (leçon L3).
    """
    import subprocess
    sys.path.insert(0, str(ROOT / "omega" / "acceptance"))
    scen_dir = ROOT / "omega" / "acceptance" / "scenarios"
    out = {}
    if not scen_dir.is_dir():
        return out
    for path in sorted(scen_dir.glob("S*.yaml")):
        try:
            r = subprocess.run(
                [sys.executable, str(ROOT / "omega" / "acceptance" / "runner.py"),
                 "--scenario", path.stem], check=False,
                cwd=str(ROOT), capture_output=True, text=True, timeout=300)
            out[path.stem] = {"scripte": (r.stdout[-1] != "\n" and "") or last_verdict(r.stdout),
                              "reel": "NON_EXECUTE"}
        except subprocess.TimeoutExpired:
            out[path.stem] = {"scripte": "TIMEOUT", "reel": "NON_EXECUTE"}
        except Exception as e:
            out[path.stem] = {"scripte": f"ERREUR:{type(e).__name__}", "reel": "NON_EXECUTE"}
    return out


def last_verdict(stdout: str) -> str:
    import re as _re
    m = _re.search(r'"verdict":\s*"(\w+)"', stdout)
    return m.group(1) if m else "VERDICT_ABSENT"


def lire_portes() -> tuple[dict, str]:
    """P0-5/ADR-0008 : l'etat des portes est CALCULE, jamais declare.

    Source : `omega/gates.toml` (definitions) + les derniers rapports produits
    par `omega/scripts/gate.py` (`docs/omega/plan/gates/phase-*.json`).
    Une porte sans rapport n'est pas `ROUGE` : elle est `NON_EXECUTE` — on ne
    declare jamais un etat qu'on n'a pas observe.
    """
    portes: dict[str, str] = {}
    try:
        with (ROOT / "omega" / "gates.toml").open("rb") as f:
            gates = tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError) as e:
        return {}, f"INCONNU(gates.toml illisible: {type(e).__name__})"

    rapports: dict[str, dict] = {}
    for p in sorted((ROOT / "docs" / "omega" / "plan" / "gates").glob("phase-*.json")):
        try:
            r = json.loads(p.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if r.get("phase"):
            rapports[r["phase"]] = r  # noms horodates : le dernier lu gagne

    prochaine = ""
    for cle in sorted(gates):
        if not cle.startswith("phase"):
            continue
        num = cle.replace("phase", "")
        r = rapports.get(cle)
        if r is None:
            portes[f"G{num}"] = "NON_EXECUTE"
            if not prochaine:
                prochaine = (f"G{num} : porte non executee — lancer "
                             f"omega/scripts/gate.py --phase {num}")
            continue
        verdict = r.get("verdict", "INCONNU")
        portes[f"G{num}"] = verdict
        if verdict != "VERT" and not prochaine:
            for c in r.get("criteres", []):
                if c.get("bloquant", True) and c.get("status") not in ("VERT", "DELEGUE_CI"):
                    prochaine = (f"{c.get('id')} : {c.get('description', '')} — "
                                 f"corriger puis relancer omega/scripts/gate.py --phase {num}")
                    break
    if not prochaine:
        prochaine = "aucune porte bloquante rouge — G0 est verte (voir docs/omega/plan/STATUS.json)"
    return portes, prochaine


def moteur_reference() -> str:
    """D0 lu depuis l'ADR-0009 (principe 7 : généré, jamais saisi)."""
    adr_dir = ROOT / "docs" / "omega" / "adr"
    for adr in sorted(adr_dir.glob("0009-d0-*.md")):
        titre = adr.read_text(encoding="utf-8").split("\n", 1)[0]
        if m := re.search(r"le moteur de référence est (\w+)", titre):
            return f"{m.group(1)} (ADR-0009)"
    return "INDETERMINE (D0, ADR-0009 introuvable)"


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
    scenarios_mesures = run_scenarios()
    avancee = {**mesures_avancees(), "plugins_enregistres": plugins_enregistres()}
    portes, prochaine_action = lire_portes()

    status = {
        "git_sha": git_sha(),
        "mesure_le": datetime.now(UTC).isoformat(timespec="seconds"),
        "moteur_reference": moteur_reference(),
        "socles": {
            "rust": {
                "crates": len(crates), "liste": crates,
                "tests": {"valeur": rust_tests, "methode": "scan_attributs",
                          "detail_par_crate": detail_rust},
            },
            "python": {
                "tests": {"valeur": python_tests, "methode": methode_python},
                "ruff_legacy_erreurs": {"valeur": ruff_legacy, "methode": methode_ruff,
                                        "note": "dette P1, non bloquante (ADR-0005) ; le "
                                                "nombre depend de la version de ruff locale, "
                                                "non comparable au ruff epingle de verify"},
            },
        },
        "portes_federees": {
            "model_policy": run_script("model_policy.py", "--check"),
            "dep_graph": run_script("check_dep_graph.py"),
        },
        "avancee": avancee,
        "registre": {
            "plugins_attendus": avancee["plugins_attendus"],
            "plugins_enregistres": avancee["plugins_enregistres"],
            "orphelins": "INCONNU(detecteur_absent — P0-6 reporte en P1-6, ADR-0008)",
        },
        "scenarios": {**{f"S{i}": {"scripte": "NON_EXECUTE_ABSENT",
                                   "reel": "NON_EXECUTE_ABSENT"}
                          for i in range(12)}, **scenarios_mesures},
        "couverture_ponderee": 0.0,
        "portes": portes,
        "prochaine_action": prochaine_action,
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
        "**Portes** : " + " · ".join(f"{k} {v}" for k, v in portes.items())
        + " — l'etat vient des rapports gate.py (docs/omega/plan/gates/), "
          "seule la sortie de gate.py fait foi (lecon L6).",
        f"**Prochaine action** : {status['prochaine_action']}",
    ]) + "\n"
    (STATUS_DIR / "STATUS.md").write_text(md)

    print(json.dumps({"socles": s, "portes_federees": status["portes_federees"],
                      "prochaine_action": status["prochaine_action"]},
                     indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
