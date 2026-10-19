"""Moteur Python : la séquence de démarrage du socle pythons n'est pas
documentée de façon fiable (aucun run.sh trouvé à la racine du dépôt). Pas
d'invention (leçon L3) : SKIPPED explicite tant que le chemin d'invocation
est découvert. Découverte = action P0-9 (spike D0), qui tranche le moteur
de référence et la façon de le démarrer ici.
"""
from .base import Engine


class PythonEngine(Engine):
    name = "python"

    def __init__(self, repo):
        self.repo = repo

    def start(self, *, llm_url, model, event_dir, port):
        raise RuntimeError(
            "SKIPPED(python_engine_absent) : séquence de démarrage du socle "
            "Python non documentée — à découvrir en P0-9 (spike D0).")

    def stop(self):
        pass

    def health(self, url):
        return {}
