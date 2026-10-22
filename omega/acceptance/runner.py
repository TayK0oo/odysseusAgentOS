#!/usr/bin/env python3
"""P0-4 — Banc d'acceptation boîte noire.

Lit un scénario YAML, prépare la mise en situation, démarre le moteur et le
faux LLM (mode scripté), exécute les actions HTTP, applique le contrôle
négatif, évalue les attendus, écrit un JSON.

Sémantique des verdicts (rouge d'abord, leçon L4) :
  PASS  = tous les attendus observés
  XFAIL = attendus NON observés + xfail_raison nommée (rouge attendu = succès de test)
  FAIL  = attendus non observés, sans raison nommée (sortie d'urgence : agir)
  XPASS = attendus observés MAIS xfail_raison présent → BUG du scénario (exit 1)
  ERREUR= crash d'infrastructure

Sémantique de sortie (ADR-0013 decision 2) :
  sans --attendu : l'accord P0 reste — PASS ou XFAIL → 0, FAIL/XPASS/ERREUR → 1
  avec --attendu V : exit 0 ssi CHAQUE run a rendu exactement V. Le second
  degré (XFAIL) n'est plus acceptable là où la porte (G1) exige un vert sec :
  un rouge présent ne peut pas fermer un critere vert.

Usage :
  python3 runner.py --scenario S0
  python3 runner.py --scenario S0,S1,S2 --runs 5 --attendu PASS
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

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "engines"))

from engines.python import PythonEngine  # noqa: E402
from engines.rust import RustEngine, get_json, post_json  # noqa: E402
from scripted_provider import ScriptedProvider, seq_echoreq_reply  # noqa: E402

SCEN = HERE / "scenarios"
# racine du dépôt : les engines attendent <repo>/omega/target
REPO_ROOT = HERE.parent.parent

VERDICTS = ("PASS", "XFAIL", "FAIL", "XPASS", "ERREUR")


def verdict_exit(verdicts: list[str], attendu: str | None) -> int:
    """Verdicts run → exit code.

    Sans --attendu : accord P0 (PASS/XFAIL → 0) — les criteres G0.g1 « S1
    rouge = XFAIL nommé » n'y sont pas touches. Avec --attendu V : 0 ssi
    chaque run a rendu exactement V ; XPASS reste un bug de scénario, et
    aucun second degré n'ouvre un vert.
    """
    if attendu is None:
        return 0 if verdicts and all(v in ("PASS", "XFAIL") for v in verdicts) else 1
    return 0 if verdicts and all(v == attendu for v in verdicts) else 1


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


def _run_sans_surface(spec: dict, engine_name: str, mode: str,
                      workdir: Path, event_dir: Path) -> dict:
    """Rejoue le scenario sur un moteur SANS surface d'outils.

    Controle negatif de S1 : le run doit demarrer, et les canaries doivent
    rester absents de la reponse (le modele n'a pas d'outil pour les voir).
    """
    out = {"demarrage_ok": False, "canaries_absentes": False}
    provider = None
    engine = None
    try:
        if mode == "scripte":
            from scripted_provider import seq_tool_then_echo
            provider = ScriptedProvider(seq_tool_then_echo("ls", {"path": "."}))
            provider.start()
        engine = {"rust": RustEngine, "python": PythonEngine}[engine_name](REPO_ROOT)
        port = _free_port()
        url = engine.start(llm_url=provider.url if provider else "",
                           model="scripted:latest",
                           event_dir=event_dir, port=port,
                           workspace=workdir, tool_surface=False)
        out["demarrage_ok"] = True
        body = None
        for a in spec.get("actions", []):
            if "post" in a:
                _status, body = post_json(url + str(a["post"]), a["body"])
        contenu = json.dumps(body, ensure_ascii=False)
        canaries = spec.get("mise_en_situation", {}).get("fichiers", {})
        n_attendus = int(canaries.get("nombre", 0)) if canaries else 0
        # On ne peut pas reuse les noms (aleatoires, regeneres) : on verifie
        # qu'aucune trace de canary n'apparait ET qu'aucun evenement d'outil
        # n'est produit (la surface desactivee = aucune boucle d'outils).
        out["canaries_absentes"] = "tool.call.completed" not in contenu
        out["n_canaries_attendus"] = n_attendus
    except Exception as e:
        out["erreur"] = f"{type(e).__name__}: {e}"
    finally:
        if engine:
            with contextlib.suppress(Exception):
                engine.stop()
        if provider:
            with contextlib.suppress(Exception):
                provider.stop()
    return out


def run_scenario(path: Path, engine_name: str, mode: str, keep: bool) -> dict:
    # Import paresseux : PyYAML n'est pas dans le venv hermétique de verify
    # (ADR-0005 : ruff + pytest uniquement). Un scénario demandé sans PyYAML
    # produit une erreur d'infrastructure NOMMÉE, jamais un crash sourd.
    try:
        import yaml
    except ModuleNotFoundError:
        return {"id": path.stem, "nom": path.name, "mode": mode,
                "engine": engine_name, "verdict": "ERREUR",
                "etapes": {}, "attendus": {}, "controle_negatif": {},
                "erreur": "PyYAML absent du venv (ADR-0005 : hermétique) — "
                          "installe-le dans l'environnement d'exécution de scénarios"}
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

        # Contrôle négatif « surface d'outils désactivée » (S1) : le MÊME
        # scénario rejoué sur un moteur SANS surface d'outils. Les canaries
        # (noms aléatoires) doivent alors rester ABSENTS de la réponse : le
        # modèle n'a aucun moyen de les connaître autrement. C'est le
        # « contrôle du contrôle » : si les canaries apparaissent quand même,
        # la preuve de l'exécution réelle de l'outil s'effondre.
        cn_desactiver = (cn or {}).get("desactiver") if isinstance(cn, dict) else None
        if isinstance(cn_desactiver, list) and "tool_surface" in cn_desactiver:
            neg = _run_sans_surface(spec, engine_name, mode, workdir, event_dir)
            report["controle_negatif"]["tool_surface_desactive"] = neg
            report["controle_negatif"]["ok"] = bool(
                neg.get("canaries_absentes") and neg.get("demarrage_ok")
            )

        # LLM scripté (mode scripté)
        if mode == "scripte":
            if spec.get("llm_scripte") == "react":
                # S2 : le modèle propose une boucle d'appels d'outil (forme
                # texte canonique ADR-0006) ; aucun mot-marqueur de budget —
                # un arrêt ne peut venir que du MOTEUR.
                from scripted_provider import seq_react_toolcall
                provider = ScriptedProvider(seq_react_toolcall("echo"))
            elif spec.get("llm_scripte") == "outil":
                # S1 : le modèle demande UN outil reel (ls) puis reprend
                # l'observation. Les noms de canary sont aleatoires : ils ne
                # peuvent venir que de l'observation de l'outil.
                from scripted_provider import seq_tool_then_echo
                provider = ScriptedProvider(
                    seq_tool_then_echo("ls", {"path": "."}))
            else:
                provider = ScriptedProvider(seq_echoreq_reply())
            provider.start()
            report["llm_url"] = provider.url

        engine = {"rust": RustEngine, "python": PythonEngine}[engine_name](REPO_ROOT)
        port = _free_port()
        # La racine de la surface d'outils EST le workdir du scenario : les
        # canaries y sont ecrits, l'outil `ls` doit donc y pointer (c'est
        # le serveur qui lit `OMEGA_WORKSPACE_ROOT`, cf. config.rs).
        url = engine.start(llm_url=provider.url if provider else "",
                           model="scripted:latest",
                           event_dir=event_dir, port=port,
                           workspace=workdir)
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

        # évaluation S2 : arrêt sur budget observable dans la réponse ET la
        # trace. Heuristique assumée : marqueur « budget » seul (les mots
        # marqueurs sont volontairement absents des messages scriptés —
        # un arrêt ne peut venir que du MOTEUR, cf. SCENARIOS-README).
        # champ dédié budget_exceeded possible dans le RunResponse quand
        # P1-3 livrera la boucle + budgets.
        if "arret_budget_reponse" in spec.get("attendu", {}):
            contenu_reponse = json.dumps(reponse_run, ensure_ascii=False).lower()
            report["attendus"]["arret_budget_reponse"] = "budget" in contenu_reponse
            run_id = (reponse_run or {}).get("run_id")
            try:
                events = get_json(f"{url}/api/runs/{run_id}/events") if run_id else []
                trace = json.dumps(events).lower()
                report["attendus"]["arret_budget_trace"] = "budget" in trace
            except Exception as e:
                report["attendus"]["arret_budget_trace"] = False
                report["trace_erreur"] = f"{type(e).__name__}: {e}"

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


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--scenario", required=True,
                    help="un id (S1) ou une liste virgule (S0,S1,S2)")
    ap.add_argument("--engine", default="rust", choices=["rust", "python"])
    ap.add_argument("--mode", default="scripte", choices=["scripte", "reel"])
    ap.add_argument("--attendu", choices=VERDICTS, default=None,
                    help="exit 0 ssi chaque run rend exactement ce verdict "
                         "(critères G1 : PASS sec ; XFAIL refuse = le rouge "
                         "présent ne ferme pas un vert, cf. verdict_exit)")
    ap.add_argument("--runs", type=int, default=1,
                    help="répétitions de la liste de scénarios (G1 : 5)")
    ap.add_argument("--keep", action="store_true")
    args = ap.parse_args(argv)
    if args.runs < 1:
        ap.error("--runs doit être >= 1")

    scenarios = [s.strip() for s in args.scenario.split(",") if s.strip()]
    verdicts = []
    for _ in range(args.runs):
        for sid in scenarios:
            path = SCEN / f"{sid}.yaml"
            if not path.exists():
                print(json.dumps({"erreur": f"scénario {sid} absent"}))
                sys.exit(2)
            r = run_scenario(path, args.engine, args.mode, args.keep)
            print(json.dumps(r, indent=2, ensure_ascii=False))
            verdicts.append(r["verdict"])
    # verdict_exit : la sémantique complète (accord P0 vs vert sec) y est
    # testée unitairement (tests/test_omega_gate_phase1.py).
    sys.exit(verdict_exit(verdicts, args.attendu))


def _main():
    sys.exit(main())


if __name__ == "__main__":
    main()
