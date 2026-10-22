"""Moteur Python : séquence de démarrage du socle.

Découverte P0-9 (spike D0, 2026-10-20) : le socle démarre par
`python3 app.py` — FastAPI/uvicorn, `APP_BIND` (127.0.0.1) et `APP_PORT`
(7000) par défaut (`app.py:370-378`).

On ne l'invoque pas ici : les dépendances du socle ne sont installées dans
aucun environnement de cette machine (`fastapi`, `sqlalchemy` absents — cf.
ADR-0011), donc le chemin de succès n'est pas vérifiable. Plutôt que
d'écrire un démarrage jamais exercé (leçon L3 : pas d'invention), l'étape
reste SKIPPED avec sa raison. Levée : P1-0 (venv dédié avec
`requirements.txt`) rend le chemin testable.
"""
from .base import Engine


class PythonEngine(Engine):
    name = "python"

    def __init__(self, repo):
        self.repo = repo

    def start(self, *, llm_url, model, event_dir, port, workspace=None):
        raise RuntimeError(
            "SKIPPED(python_engine_absent) : le socle Python se lance par "
            "`python3 app.py` (uvicorn, APP_BIND=127.0.0.1, APP_PORT=7000), "
            "mais ses dependances ne sont installees dans aucun environnement "
            "de cette machine (fastapi, sqlalchemy absents — ADR-0011). "
            "Levere : P1-0.")

    def stop(self):
        pass

    def health(self, url):
        return {}
