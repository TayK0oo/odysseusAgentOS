#!/usr/bin/env python3
"""G1.d (ADR-0013) — le registre vif doit porter la liste nominative.

Scrape `GET /api/plugins` d'une instance jetable du moteur (methode creditee:
`measure.py::plugins_enregistres` / `engines.rust`, idee `instance_jetable_http`)
et compare au registre vif attendu : la liste nominative inscrite dans
`omega/gates.toml` (critere `G1.d.plugins`, sous-table `plugins.attendus`).

Exit 0 ssi le registre vif == liste nominative exactement (couples id+capability).
Sinon 1, avec le detail : couples manquants (adaptateurs non branches, A4 / P1-2)
et couples en trop (cable qui n'aurait pas sa place).
"""
from __future__ import annotations

import json
import sys
import tempfile
import tomllib
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "engines"))

from engines.rust import RustEngine, get_json  # noqa: E402

REPO_ROOT = HERE.parent.parent

ID = "G1.d.plugins"


def liste_attendue(gates: dict, id_critere: str = ID) -> set[tuple[str, str]] | None:
    for c in gates.get("phase1", {}).get("criteres", []):
        if c.get("id") == id_critere:
            coupl = []
            for p in c.get("plugins", {}).get("attendus", []):
                if not isinstance(p, dict) or set(p) != {"id", "capability"}:
                    return None
                coupl.append((p["id"], p["capability"]))
            return set(coupl)
    return None


def discrepances(scraped: set[tuple[str, str]], attendue: set[tuple[str, str]]) -> dict:
    return {
        "manquants": sorted(attendue - scraped),
        "en_trop": sorted(scraped - attendue),
    }


def main() -> int:
    with (REPO_ROOT / "omega" / "gates.toml").open("rb") as f:
        attendue = liste_attendue(tomllib.load(f))
    if attendue is None:
        print(json.dumps({"verdict": "ECHEC",
                          "erreur": "liste nominative absente ou invalide dans gates.toml"}))
        return 1
    rapport = {"verdict": "ECHEC", "attendus": sorted(attendue)}
    import socket
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    engine = RustEngine(REPO_ROOT)
    try:
        url = engine.start(llm_url="http://127.0.0.1:9", model="probe",
                           event_dir=Path(tempfile.mkdtemp(prefix="g1d-events-")),
                           port=port)
        scraped = {(p.get("id"), p.get("capability"))
                   for p in get_json(url + "/api/plugins")}
        rapport["registre_vif"] = sorted(scraped)
        rapport["discrepances"] = discrepances(scraped, attendue)
        ok = not rapport["discrepances"]["manquants"] \
            and not rapport["discrepances"]["en_trop"]
        rapport["verdict"] = "VERT" if ok else "ECHEC"
    except Exception as e:
        rapport["erreur"] = f"{type(e).__name__}: {e}"
    finally:
        engine.stop()
    print(json.dumps(rapport, indent=2, ensure_ascii=False))
    return 0 if rapport["verdict"] == "VERT" else 1


if __name__ == "__main__":
    sys.exit(main())
