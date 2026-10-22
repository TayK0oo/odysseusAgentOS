#!/usr/bin/env python3
"""A3 (réduit) — reprise après crash pour k événements journalisés.

Pour chaque k dans 1..max_k : lance un run scripté (append_file), tue le
serveur (SIGKILL) après le k-ième événement durable, relance sur le même
event_dir, reprend ; vérifie l'état final (run complété) et l'effet exactement
une fois (fichier append = 1 ligne). Exit 0 ssi toutes les (k, run) vertes.
"""
from __future__ import annotations

import argparse
import contextlib
import json
import shutil
import socket
import sys
import tempfile
import threading
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "engines"))

from engines.rust import RustEngine, get_json, post_json  # noqa: E402
from scripted_provider import ScriptedProvider  # noqa: E402

REPO_ROOT = HERE.parent.parent


def _free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def one(k: int, run_idx: int) -> bool:
    workdir = Path(tempfile.mkdtemp(prefix=f"a3-k{k}-r{run_idx}-"))
    event_dir = Path(tempfile.mkdtemp(prefix=f"a3-k{k}-r{run_idx}-ev-"))
    provider = None
    engine = None
    engine2 = None
    ok = True
    run_id = "a3-run"
    try:
        import json as _json

        def _reply(obj):
            return {"choices": [{"message": {"role": "assistant", "content": obj}}],
                    "usage": {"prompt_tokens": 5, "completion_tokens": 5}}

        def _decider(req):
            msgs = req.get("messages", [])
            last = msgs[-1] if msgs else {}
            content = last.get("content", "") if isinstance(last, dict) else ""
            if isinstance(content, str) and ("appended" in content or "already_present" in content):
                return _reply(content)
            if isinstance(last, dict) and last.get("role") == "tool":
                return _reply(last.get("content", ""))
            return _reply(_json.dumps({"tool": "append_file",
                                       "arguments": {"path": "append.txt",
                                                     "content": "canary-line"}},
                                      sort_keys=True))

        # Sequence assez longue pour absorber les re-runs de reprise sans
        # epuisement (le decider est stateless : il ne depend pas du compteur).
        base_seq = [_decider for _ in range(200)]
        wrapped = []
        for _f in base_seq:
            def _w(req, f=_f):
                time.sleep(0.05)
                return f(req)
            wrapped.append(_w)
        provider = ScriptedProvider(wrapped)
        provider.start()

        engine = RustEngine(REPO_ROOT)
        port = _free_port()
        url = engine.start(llm_url=provider.url, model="scripted:latest",
                           event_dir=event_dir, port=port, workspace=workdir)

        body1 = {"messages": [{"role": "user",
                               "content": "Ajoute une ligne au fichier"}],
                 "run_id": run_id, "resume": False}

        result: dict = {}

        def worker():
            try:
                st, bd = post_json(url + "/api/run", body1, timeout=60)
                result["status"] = st
                result["body"] = bd
            except Exception as e:
                result["erreur"] = f"{type(e).__name__}: {e}"

        t = threading.Thread(target=worker, daemon=True)
        t.start()

        killed = False
        for _ in range(4000):
            if not t.is_alive():
                break
            try:
                evresp = get_json(f"{url}/api/runs/{run_id}/events")
                evs = evresp.get("events", []) if isinstance(evresp, dict) else []
                if len(evs) >= k:
                    engine.kill()
                    killed = True
                    break
            except Exception:
                pass
            time.sleep(0.005)
        t.join(timeout=10)

        if not killed:
            # Le run a fini avant le kill : on considere quand meme, mais
            # l'objectif (crash) n'est pas atteint pour ce k.
            ok = False

        # Relance sur le meme event_dir (nouveau PID).
        engine2 = RustEngine(REPO_ROOT)
        port2 = _free_port()
        url2 = engine2.start(llm_url=provider.url, model="scripted:latest",
                             event_dir=event_dir, port=port2, workspace=workdir)

        body2 = {"messages": [{"role": "user",
                               "content": "Ajoute une ligne au fichier"}],
                 "run_id": run_id, "resume": True}
        status, resp = post_json(url2 + "/api/run", body2, timeout=60)

        # Verifications.
        if not (isinstance(resp, dict) and resp.get("status") == "ok"):
            ok = False
        append_path = workdir / "append.txt"
        if not append_path.exists():
            ok = False
        else:
            lines = [ln for ln in append_path.read_text(encoding="utf-8").splitlines()]
            if len(lines) != 1:
                ok = False
        # Journal coherent : au moins un evenement.
        try:
            evresp = get_json(f"{url2}/api/runs/{run_id}/events")
            if not (isinstance(evresp, dict) and len(evresp.get("events", [])) > 0):
                ok = False
        except Exception:
            ok = False

        print(f"  k={k} run={run_idx}: {'OK' if ok else 'FAIL'} "
              f"(killed={killed}, status={resp.get('status') if isinstance(resp, dict) else resp})")
    except Exception as e:
        ok = False
        print(f"  k={k} run={run_idx}: FAIL ({type(e).__name__}: {e})")
    finally:
        for e in (engine2, engine):
            if e:
                with contextlib.suppress(Exception):
                    e.kill() if hasattr(e, "kill") else e.stop()
        if provider:
            with contextlib.suppress(Exception):
                provider.stop()
        shutil.rmtree(workdir, ignore_errors=True)
        shutil.rmtree(event_dir, ignore_errors=True)
    return ok


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=1)
    ap.add_argument("--max-k", type=int, default=6)
    args = ap.parse_args()
    if args.runs < 1 or args.max_k < 1:
        ap.error("args invalides")

    all_ok = True
    for run_idx in range(1, args.runs + 1):
        print(f"run {run_idx}/{args.runs}")
        for k in range(1, args.max_k + 1):
            if not one(k, run_idx):
                all_ok = False
    print("A3:", "OK" if all_ok else "FAIL")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
