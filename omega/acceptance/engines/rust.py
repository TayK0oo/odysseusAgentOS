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


class RustEngine(Engine):
    name = "rust"

    def __init__(self, repo: Path):
        self.repo = repo
        self.proc: subprocess.Popen | None = None

    def start(self, *, llm_url, model, event_dir, port):
        binary = self.repo / "omega" / "target" / "debug" / "omega-server"
        if not binary.exists():
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
