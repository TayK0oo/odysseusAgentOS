"""Moteur Rust : lance le binaire omega-server configuré par env.

Contrat env réel (omega-server/src/config.rs) : OMEGA_PORT, OLLAMA_URL,
OLLAMA_MODEL, OMEGA_EVENT_DIR, OMEGA_UI_DIR (vide = API seule).
Route santé réelle : GET /api/health (routes.rs:124).
"""
import json
import os
import subprocess
import time
import urllib.request
from pathlib import Path

from .base import Engine


def needs_rebuild(binary: Path, sources_root: Path) -> bool:
    """Le binaire doit-il etre reconstruit ?

    Oui s'il est absent, ou si une source Rust est plus recente que lui.
    Fonction pure (testable sans compiler) : la decision de reconstruire
    est le sujet — un binaire perime fait mesurer un etat faux a la porte.
    """
    if not binary.exists():
        return True
    try:
        built = binary.stat().st_mtime
    except OSError:
        return True
    for src in sources_root.rglob("*.rs"):
        try:
            if src.stat().st_mtime > built:
                return True
        except OSError:
            continue
    return False


class RustEngine(Engine):
    name = "rust"

    def __init__(self, repo: Path):
        self.repo = repo
        self.proc: subprocess.Popen | None = None
    def start(self, *, llm_url, model, event_dir, port, workspace=None,
              tool_surface=True, tool_budget=None):
        binary = self.repo / "omega" / "target" / "debug" / "omega-server"
        # Rebuild si le binaire est ABSENT **ou PERIME**. Sans ce second
        # cas, le moteur lance un binaire anterieur au code : la porte
        # mesure alors un etat faux. Observe deux fois le 2026-10-22
        # (G1.d.plugins puis G1 entier) — le meme piege, deux faux rouges.
        if needs_rebuild(binary, self.repo / "omega" / "crates"):
            r = subprocess.run(["cargo", "build", "-p", "omega-server"],
                               cwd=self.repo / "omega",
                               capture_output=True, text=True, check=False)
            if r.returncode != 0:
                raise RuntimeError(f"build: {r.stderr[-1500:]}")
        env = {**os.environ,
               "OLLAMA_URL": llm_url,
               "OLLAMA_MODEL": model,
               "OMEGA_PORT": str(port),
               "OMEGA_EVENT_DIR": str(event_dir),
               "OMEGA_UI_DIR": ""}  # API seule : la UI absente n'échoue pas S0
        if workspace is not None:
            # Racine de la surface d'outils (P1-1) : c'est le workdir du
            # scenario, la ou les canaries sont ecrits.
            env["OMEGA_WORKSPACE_ROOT"] = str(workspace)
        if tool_budget is not None:
            # Budget d'executions d'outils du moteur (P1-3). Controle
            # negatif : une valeur enorme = aucun plafond effectif.
            env["OMEGA_TOOL_BUDGET"] = str(tool_budget)
        if not tool_surface:
            # Controle negatif : moteur SANS surface d'outils. L'agent ne peut
            # alors pas lister les fichiers — les canaries doivent rester
            # absents de la reponse (controle du controle).
            env["OMEGA_TOOL_SURFACE"] = "off"
        self.proc = subprocess.Popen([str(binary)], env=env,
                                     stdout=subprocess.PIPE,
                                     stderr=subprocess.PIPE, text=True)
        url = f"http://127.0.0.1:{port}"
        for _ in range(100):
            try:
                urllib.request.urlopen(url + "/api/health", timeout=0.3)
                return url
            except Exception:
                if self.proc.poll() is not None:
                    err = (self.proc.stderr.read() if self.proc.stderr else "")[-1000:]
                    raise RuntimeError(f"omega-server mort: {err}")
                time.sleep(0.1)
        raise RuntimeError("omega-server ne répond pas")

    def stop(self):
        if self.proc and self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.proc.kill()

    def health(self, url):
        with urllib.request.urlopen(url + "/api/health", timeout=3) as r:
            return {"status": r.status, "body": r.read().decode()[:200]}


def get_json(url: str, timeout: int = 10) -> dict:
    with urllib.request.urlopen(url, timeout=timeout) as r:
        return json.loads(r.read().decode())


def post_json(url: str, payload: dict, timeout: int = 60):
    data = json.dumps(payload).encode()
    req = urllib.request.Request(url, data=data,
                                 headers={"Content-Type": "application/json"},
                                 method="POST")
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.status, json.loads(r.read().decode())
