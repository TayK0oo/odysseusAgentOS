"""P0-3a — mesures avancées : les chiffres contestés du PLAN-MAITRE §1.2.

Chaque clé porte une méthode déclarée. Un objet non mesurable devient
`INCONNU(raison)` — jamais 0, jamais une valeur inventée (leçon L3).
Importé par measure.py ; testable isolément : python3 measures_advanced.py
"""
from __future__ import annotations

import contextlib
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# union TS documentée dans App.tsx (approximation, pas de dépendance ts-morph)
REGEX_PANEL = r"type Panel =(.*?);"


def m(v, methode: str) -> dict:
    return {"valeur": v, "methode": methode}


def unk(raison: str, methode: str) -> dict:
    return {"valeur": f"INCONNU({raison})", "methode": methode}


def _dissections() -> tuple[list[dict], Path | None]:
    apps_dir = ROOT / "omega" / "crates" / "omega-harness" / "data" / "apps"
    if not apps_dir.is_dir():
        return [], apps_dir
    out = []
    for p in sorted(apps_dir.glob("*.json")):
        with contextlib.suppress(Exception):
            out.append(json.loads(p.read_text()))
    return out, apps_dir


def mesures_avancees() -> dict:
    out: dict = {}

    # 1-2. dissections + fonctionnalités + capacités (tablature JSON réelle)
    apps, apps_dir = _dissections()
    out["dissections"] = (m(len(apps), "parse_json") if apps else
                          unk(f"apps_illisibles_{apps_dir.name}", "parse_json"))
    fct = 0
    om_caps: set[str] = set()
    try:
        for a in apps:
            fct += len(a.get("functionalities") or [])
            for fn in a.get("functionalities") or []:
                with contextlib.suppress(AttributeError):
                    if fn.get("omega_capability"):
                        om_caps.add(str(fn["omega_capability"]))
    except AttributeError as e:
        return {"avancee_erreur": unk(f"{type(e).__name__}", "parse_json")}
    out["fonctionnalites_total"] = m(fct, "parse_json")
    out["capacites_omega"] = (
        m(len(om_caps), "parse_json_omega_capability") if om_caps else
        unk("champ_omega_capability_absent", "parse_json"))

    # 3. census recenses (champ `count` du JSON)
    census_p = ROOT / "omega" / "crates" / "omega-harness" / "data" / "harnesses_census.json"
    if census_p.is_file():
        with contextlib.suppress(Exception):
            c = json.loads(census_p.read_text())
            out["census_recenses"] = m(c.get("count"), "parse_json")
        if "census_recenses" not in out:
            out["census_recenses"] = unk("json_illisible", "parse_json")
    else:
        out["census_recenses"] = unk("census_absent", "parse_json")

    # 4. panneaux UI (union `type Panel` de App.tsx)
    app_tsx = ROOT / "omega" / "ui" / "src" / "App.tsx"
    if app_tsx.is_file():
        mm = re.search(REGEX_PANEL, app_tsx.read_text(), re.S)
        membres = [x.strip().strip("'\"") for x in
                   (mm.group(1).split("|") if mm else [])]
        n = len([x for x in membres if x])
        out["ui_panels"] = m(n, "regex_union_tsx") if n else unk(
            "union_Panel_absente", "regex_union_tsx")
    else:
        out["ui_panels"] = unk("App.tsx_absent", "regex_union_tsx")

    # 5. agents opencode (FS)
    ag_dir = ROOT / ".opencode" / "agents"
    n = len(list(ag_dir.glob("*.md"))) if ag_dir.is_dir() else 0
    out["agents_opencode"] = m(n, "fs_count_md") if n else unk(
        "agents_absents", "fs_count_md")

    # 6. clones harnais (FS, chemin extérieur — méthode déclarée)
    clones_p = Path("/home/tayk0oo/omega-re")
    if clones_p.is_dir():
        out["clones_harnais"] = m(len(list(clones_p.iterdir())), "fs_count_ext")
    else:
        out["clones_harnais"] = unk("chemin_absent", "fs_count_ext")

    # 7. socle python (probe existence ; jamais de commande inventée)
    py = Path("/home/tayk0oo/Projets/odysseusAgentOS")
    if py.is_dir():
        cmd = next((c for c in ("run.sh", "app.py", "manage.py")
                    if (py / c).is_file()), None)
        out["python_socle"] = m(
            f"present;cmd={cmd}" if cmd else "present;sans_commande_documentee",
            "fs_probe")
    else:
        out["python_socle"] = m("absent", "fs_probe")

    # 8. plugins attendus : gates.yaml est P0-5 → INCONNU authentique
    out["plugins_attendus"] = _adaptateurs_attendus()

    return out


def _adaptateurs_attendus() -> dict:
    """Les adaptateurs que le code sait construire (source : composition.rs).

    Parse deterministe des appels `register("<id>", "<capability>"` de
    `omega-adapters` — c'est l'ensemble *attendu* dans le registre. Compare
    ensuite aux plugins reellement enregistres (`plugins_enregistres`, mesure
    HTTP) : l'ecart entre les deux est le creux "adaptateurs non branches".
    """
    src = ROOT / "omega" / "crates" / "omega-adapters" / "src" / "composition.rs"
    if not src.is_file():
        return unk("composition_rs_absent", "parse_ids_composition_rs")
    try:
        txt = src.read_text(encoding="utf-8")
    except OSError as e:
        return unk(f"lecture_impossible({type(e).__name__})", "parse_ids_composition_rs")
    paires = re.findall(r'register\(\s*"([^"]+)"\s*,\s*"([^"]+)"', txt, re.S)
    if not paires:
        return unk("aucun_registre_trouve", "parse_ids_composition_rs")
    return m([{"id": i, "capability": c} for i, c in paires],
             "parse_ids_composition_rs")


def plugins_enregistres():
    """GET /api/plugins sur une instance jetable du binaire (méthode créditée)."""
    bin_ = ROOT / "omega" / "target" / "debug" / "omega-server"
    if not bin_.is_file():
        return unk("binaire_absent(cargo build -p omega-server)",
                   "instance_jetable_http")
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    ev = tempfile.mkdtemp(prefix="probe-events-")
    env = os.environ | {"OLLAMA_URL": "http://127.0.0.1:9",
                        "OLLAMA_MODEL": "probe",
                        "OMEGA_PORT": str(port),
                        "OMEGA_EVENT_DIR": ev,
                        "OMEGA_UI_DIR": ""}
    proc = subprocess.Popen([str(bin_)], env=env, stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL, start_new_session=True)
    try:
        for _ in range(60):
            with contextlib.suppress(Exception):
                urllib.request.urlopen(
                    f"http://127.0.0.1:{port}/api/health", timeout=0.3)
                break
            time.sleep(0.1)
        else:
            return unk("serveur_sans_reponse", "instance_jetable_http")
        with urllib.request.urlopen(
                f"http://127.0.0.1:{port}/api/plugins", timeout=5) as r:
            return m(json.loads(r.read().decode()), "instance_jetable_http")
    except subprocess.SubprocessError as e:
        return unk(f"{type(e).__name__}", "instance_jetable_http")
    except (OSError, json.JSONDecodeError) as e:
        return unk(f"{type(e).__name__}", "instance_jetable_http")
    finally:
        with contextlib.suppress(Exception):
            proc.terminate()
            proc.wait(timeout=5)
        with contextlib.suppress(Exception):
            shutil.rmtree(ev)


if __name__ == "__main__":
    print(json.dumps({**mesures_avancees(),
                      "plugins_enregistres": plugins_enregistres()},
                     indent=2, ensure_ascii=False))
    sys.exit(0)
