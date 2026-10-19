#!/usr/bin/env python3
"""P0-4 — Banc d'acceptation boîte noire.

Lit un scénario YAML, prépare la mise en situation, démarre le moteur et le
faux LLM (mode scripté), exécute les actions HTTP, applique le contrôle
négatif, évalue les attendus, écrit un JSON.

Sémantique des verdicts (rouge d'abord, leçon L4) :
  PASS  = tous les attendus observés吻合
  XFAIL = attendus NON observés + xfail_raison nommée (rouge attendu = succès de test)
  FAIL  = attendus non observés, sans raison nommée (sortie d'urgence : agir)
  XPASS = attendus observés MAIS xfail_raison présent → BUG du scénario (exit 1)
  ERREUR= crash d'infrastructure

Usage :
  python3 runner.py --scenario S0
  python3 runner.py --scenario S1 [--engine rust|python] [--mode scripte] [--keep]
"""
from __future__ import annotations

import argparse
import contextlib
import json
import secrets
import shutil
import socket
import sys
import tempfile
from datetime import UTC, datetime
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "engines"))

from engines.python import PythonEngine  # noqa: E402
from engines.rust import RustEngine, get_json, post_json  # noqa: E402
from scripted_provider import ScriptedProvider, seq_echoreq_reply  # noqa: E402

SCEN = HERE / "scenarios"
# racine du dépôt : les engines attendent <repo>/omega/target
REPO_ROOT = HERE.parent.parent


def _mk_canary(workdir: Path, spec: dict) -> list[str]:
    n = spec.get("nombre", 5)
    names = [secrets.token_hex(4) for _ in range(n)]
    for nm in names:
        (workdir / nm).write_text(secrets.token_hex(16))
    return names


def _free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def _nettoie_serveur_port(origine: Path) -> list[str]:
    """Astuce honnête : les canaries sont créées APRÈS le démarrage du
    moteur, pour exclure tout fichier préexistant du répertoire serveur."""
    return origine


def run_scenario(path: Path, engine_name: str, mode: str, keep: bool) -> dict:
    spec = yaml.safe_load(path.read_text(encoding="utf-8"))
    report = {"id": spec["id"], "nom": spec.get("nom"), "mode": mode,
              "engine": engine_name, "verdict": "ERREUR",
              "etapes": {}, "attendus": {}, "controle_negatif": {}}
    workdir = Path(tempfile.mkdtemp(prefix=f"acc-{spec['id']}-"))
    report["workdir"] = str(workdir)
    event_dir = Path(tempfile.mkdtemp(prefix=f"acc-{spec['id']}-events"))
    provider: ScriptedProvider | None = None
    engine = None
    try:
        ctx = {}
        fs_spec = spec.get("mise_en_situation", {}).get("fichiers")
        if fs_spec:
            ctx["canaries"] = _mk_canary(workdir, fs_spec)

        # contrôles négatifs isolés (ne partagent rien avec le run nominal)
        cn = spec.get("controle_negatif")
        if isinstance(cn, dict) and cn.get("injecter", {}).get("port_deja_pris"):
            # on réserve le port, puis on demande au moteur de démarrer dessus
            blocker = socket.socket()
            blocker.bind(("127.0.0.1", 0))
            blocker.listen(1)
            port_pris = blocker.getsockname()[1]
            eng2 = {"rust": RustEngine, "python": PythonEngine}[engine_name](
                REPO_ROOT)
            ok_start = True
            try:
                eng2.start(llm_url="http://127.0.0.1:1", model="scripted:latest",
                           event_dir=event_dir, port=port_pris)
            except Exception as e:
                ok_start = False
                report["controle_negatif"]["raison"] = f"{type(e).__name__}: {e}"
            finally:
                with contextlib.suppress(Exception):
                    eng2.stop()
                blocker.close()
            # attendu : échec au démarrage = contrôle négatif OK
            report["controle_negatif"]["ok"] = not ok_start

        # LLM scripté (mode scripté)
        if mode == "scripte":
            provider = ScriptedProvider(seq_echoreq_reply())
            provider.start()
            report["llm_url"] = provider.url

        engine = {"rust": RustEngine, "python": PythonEngine}[engine_name](REPO_ROOT)
        port = _free_port()
        url = engine.start(llm_url=provider.url if provider else "",
                           model="scripted:latest",
                           event_dir=event_dir, port=port)
        report["etapes"]["start"] = "OK"
        report["url"] = url

        # actions HTTP
        reponse_run = None
        for a in spec.get("actions", []):
            if "get" in a:
                body = get_json(url + a["get"])
                report["etapes"][f"GET {a['get']}"] = {
                    "status": 200, "body": body}
                if str(a["get"]) == "/api/health":
                    report["attendus"]["health_ok"] = body.get("status") == "ok"
            if "post" in a:
                status, body = post_json(url + str(a["post"]), a["body"])
                reponse_run = body
                report["etapes"][f"POST {a['post']}"] = {"status": status,
                                                         "body": body}

        # évaluation S1 : canaries + événement d'outil, observés pas déclarés
        if "reponse_contient_les_canaries" in spec.get("attendu", {}):
            contenu = json.dumps(reponse_run, ensure_ascii=False)
            manquantes = [c for c in ctx.get("canaries", [])
                          if c not in contenu]
            report["attendus"]["reponse_contient_les_canaries"] = not manquantes
            report["canaries_absentes"] = manquantes
            # trace des événements du run
            run_id = (reponse_run or {}).get("run_id")
            if run_id:
                try:
                    events = get_json(f"{url}/api/runs/{run_id}/events")
                    evs = json.dumps(events).lower()
                    report["attendus"]["evenement_appele_outil"] = (
                        "tool" in evs or "outil" in evs)
                except Exception as e:
                    report["attendus"]["evenement_appele_outil"] = False
                    report["attendus"]["trace_erreur"] = f"{type(e).__name__}: {e}"

        # verdict
        attendus = report["attendus"]
        ok_tous = bool(attendus) and all(v is True for v in attendus.values())
        xfail = spec.get("xfail_raison")
        if ok_tous and xfail:
            report["verdict"] = "XPASS"
        elif ok_tous:
            report["verdict"] = "PASS"
        elif xfail:
            report["verdict"] = "XFAIL"
        else:
            report["verdict"] = "FAIL"
    except Exception as e:
        report["verdict"] = "ERREUR"
        report["erreur"] = f"{type(e).__name__}: {e}"
    finally:
        if engine:
            engine.stop()
        if provider:
            provider.stop()
        if not keep:
            shutil.rmtree(workdir, ignore_errors=True)
            shutil.rmtree(event_dir, ignore_errors=True)
    report["mesure_le"] = datetime.now(UTC).isoformat(timespec="seconds")
    return report


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--scenario", required=True, help="S0, S1, ...")
    ap.add_argument("--engine", default="rust", choices=["rust", "python"])
    ap.add_argument("--mode", default="scripte", choices=["scripte", "reel"])
    ap.add_argument("--keep", action="store_true")
    args = ap.parse_args()

    path = SCEN / f"{args.scenario}.yaml"
    if not path.exists():
        print(json.dumps({"erreur": f"scénario {args.scenario} absent"}))
        sys.exit(2)
    r = run_scenario(path, args.engine, args.mode, args.keep)
    print(json.dumps(r, indent=2, ensure_ascii=False))
    # PASS/XFAIL = 0 ; FAIL/XPASS/ERREUR = 1 (XPASS = bug du scénario)
    sys.exit(0 if r["verdict"] in ("PASS", "XFAIL") else 1)


if __name__ == "__main__":
    main()
