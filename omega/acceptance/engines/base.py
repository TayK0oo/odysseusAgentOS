"""Contrat moteur : start() → url ; stop() ; health()."""
from abc import ABC, abstractmethod
from pathlib import Path


class Engine(ABC):
    name: str

    @abstractmethod
    def start(self, *, llm_url: str, model: str, event_dir: Path, port: int,
              workspace: Path | None = None, tool_surface: bool = True,
              tool_budget: int | None = None) -> str:
        """Démarre ; lève si échec ; retourne l'URL HTTP de base.

        `workspace` : racine du workspace de l'agent (surface d'outils P1-1).
        `None` = racine par defaut du moteur.
        """

    @abstractmethod
    def stop(self) -> None: ...

    @abstractmethod
    def health(self, url: str) -> dict:
        """GET /api/health → dict observé. Lève si non-2xx."""
