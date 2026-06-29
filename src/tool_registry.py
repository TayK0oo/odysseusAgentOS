"""
ToolRegistry — Filtre les tools disponibles selon la phase courante.
Pattern inspiré de Serena ToolMarker / ToolRegistry.
"""
import yaml
import threading
from pathlib import Path
from typing import Optional
import logging

logger = logging.getLogger(__name__)

class PhaseContext:
    """Contexte de phase actuel pour une session."""
    def __init__(self, phase: str = "BUILD", session_id: Optional[str] = None):
        self.phase = phase.upper()
        self.session_id = session_id

class ToolRegistry:
    """
    Registry qui filtre tool_index selon la phase courante.
    Thread-safe : chaque session a son propre PhaseContext.
    """
    _instance = None
    _lock = threading.Lock()

    def __init__(self, config_path: str = "config/phase-lock.yaml"):
        self._config_path = config_path
        self._config = self._load_config()
        self._session_phases: dict[str, PhaseContext] = {}
        self._session_lock = threading.Lock()

    @classmethod
    def get_instance(cls) -> 'ToolRegistry':
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    cls._instance = cls()
        return cls._instance

    def _load_config(self) -> dict:
        """Charge phase-lock.yaml. Fallback sur BUILD si absent."""
        try:
            path = Path(self._config_path)
            if path.exists():
                with open(path) as f:
                    return yaml.safe_load(f)
        except Exception as e:
            logger.warning(f"phase-lock.yaml non chargé: {e}")
        return {"phases": {"BUILD": {"blocked_tools": []}}, "default_phase": "BUILD"}

    def set_phase(self, session_id: str, phase: str) -> None:
        """Définit la phase pour une session."""
        with self._session_lock:
            self._session_phases[session_id] = PhaseContext(phase, session_id)
            logger.info(f"Session {session_id}: phase → {phase.upper()}")

    def get_phase(self, session_id: str) -> str:
        """Retourne la phase actuelle d'une session."""
        with self._session_lock:
            ctx = self._session_phases.get(session_id)
            return ctx.phase if ctx else self._config.get("default_phase", "BUILD")

    def is_tool_allowed(self, tool_name: str, session_id: str, tool_args: dict = None) -> dict:
        """
        Vérifie si un tool est autorisé dans la phase actuelle.
        Retourne {'allowed': bool, 'reason': str, 'phase': str}
        """
        phase = self.get_phase(session_id)
        phase_config = self._config.get("phases", {}).get(phase, {})
        blocked = phase_config.get("blocked_tools", [])

        if tool_name in blocked:
            return {
                "allowed": False,
                "reason": phase_config.get("message", f"Tool {tool_name} bloqué en phase {phase}"),
                "phase": phase
            }

        # Vérification write_restriction (Phase PLAN)
        if "write_restriction" in phase_config and tool_args:
            path = tool_args.get("path", tool_args.get("file_path", ""))
            if path:
                allowed_paths = phase_config["write_restriction"].get("allowed_paths", [])
                if not any(str(path).startswith(p) for p in allowed_paths):
                    return {
                        "allowed": False,
                        "reason": phase_config["write_restriction"].get("message", f"Écriture dans {path} non autorisée en phase {phase}"),
                        "phase": phase
                    }

        # Vérification exec_restriction (Phase VERIFY)
        if "exec_restriction" in phase_config and tool_name in ("bash", "run_command"):
            command = ""
            if tool_args:
                command = tool_args.get("command", tool_args.get("cmd", ""))
            patterns = phase_config["exec_restriction"].get("allowed_commands_patterns", [])
            if command and not any(p in command for p in patterns):
                return {
                    "allowed": False,
                    "reason": phase_config["exec_restriction"].get("message", f"Commande non autorisée en phase {phase}"),
                    "phase": phase
                }

        return {"allowed": True, "reason": "ok", "phase": phase}

    def get_allowed_tools(self, all_tools: list, session_id: str) -> list:
        """Filtre une liste de tools selon la phase. Retourne uniquement les autorisés."""
        return [t for t in all_tools if self.is_tool_allowed(
            t.get("name", t) if isinstance(t, dict) else t, session_id
        )["allowed"]]

    def get_phase_message(self, session_id: str) -> str:
        """Message affiché à l'agent au début de la phase."""
        phase = self.get_phase(session_id)
        phase_config = self._config.get("phases", {}).get(phase, {})
        return phase_config.get("message", f"Phase active : {phase}")
