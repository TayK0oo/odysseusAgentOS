"""Central Tool Registry — every tool is a modular brick.

Loaded by the Decision Engine and the SFD orchestrator.
Combines phase-lock enforcement (config/phase-lock.yaml) with
modular tool brick discovery (config/tool-decision-tree.yaml).
"""

import logging
import socket
import threading
from pathlib import Path

import yaml

logger = logging.getLogger(__name__)


# ============================================================
# PhaseContext — Per-session phase state
# ============================================================


class PhaseContext:
    """Contexte de phase actuel pour une session."""

    def __init__(self, phase: str = "BUILD", session_id: str | None = None):
        self.phase = phase.upper()
        self.session_id = session_id


# ============================================================
# ToolBrick — Modular tool brick for any phase
# ============================================================


class ToolBrick:
    """A modular tool brick that can be plugged into any phase."""

    def __init__(self, name: str, config: dict):
        self.name = name
        self.type = config.get("type", "unknown")
        self.phases = config.get("phase", [])
        if isinstance(self.phases, str):
            self.phases = [self.phases]
        self.provides = config.get("provides", [])
        self.requires = config.get("requires", [])
        self.fallback = config.get("fallback", [])
        self.optional = config.get("optional", False)

    def is_available(self) -> bool:
        for dep in self.requires:
            if isinstance(dep, str) and ":" in dep:
                host, port = dep.split(":", 1)
                if self._check_port(host, int(port)):
                    continue
                if self.fallback:
                    return any(
                        self._check_port(*f.split(":", 1)) for f in self.fallback if isinstance(f, str) and ":" in f
                    )
                return False
        return True

    @staticmethod
    def _check_port(host: str, port: str | int) -> bool:
        try:
            port = int(port)
            s = socket.create_connection((host, port), timeout=2)
            s.close()
            return True
        except (OSError, ValueError):
            return False

    def __repr__(self):
        return (
            f"ToolBrick(name={self.name!r}, type={self.type!r}, "
            f"phases={self.phases!r}, available={self.is_available()})"
        )


# ============================================================
# ToolRegistry — Central registry combining phase-lock + bricks
# ============================================================


class ToolRegistry:
    """Central registry of all modular tool bricks with phase-lock enforcement.

    - Reads config/phase-lock.yaml for phase-based tool restrictions
    - Reads config/tool-decision-tree.yaml for tool capabilities and fallbacks
    - Thread-safe singleton
    """

    _instance = None
    _lock = threading.Lock()

    def __init__(self, config_path: str | None = None, brick_path: str | None = None):
        self._config_path = config_path or "config/phase-lock.yaml"
        self._brick_path = brick_path or "config/tool-decision-tree.yaml"
        self._config = self._load_config()
        self._session_phases: dict[str, PhaseContext] = {}
        self._session_lock = threading.Lock()

        # Tool brick index from decision tree
        self.tools: dict[str, ToolBrick] = {}
        self._brick_config: dict = {}
        self._build_brick_index()

    @classmethod
    def get_instance(cls) -> "ToolRegistry":
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    cls._instance = cls()
        return cls._instance

    # ── Config loading ────────────────────────────────────────

    def _load_config(self) -> dict:
        try:
            path = Path(self._config_path)
            if path.exists():
                with open(path) as f:
                    return yaml.safe_load(f)
        except Exception as e:
            logger.warning(f"phase-lock.yaml non chargé: {e}")
        return {"phases": {"BUILD": {"blocked_tools": []}}, "default_phase": "BUILD"}

    def _build_brick_index(self):
        try:
            path = Path(self._brick_path)
            if path.exists():
                with open(path) as f:
                    self._brick_config = yaml.safe_load(f)
        except Exception as e:
            logger.warning(f"tool-decision-tree.yaml not loaded: {e}")
            self._brick_config = {"decision_tree": {"phases": {}, "tool_capabilities": {}, "fallback_chains": {}}}

        caps = self._brick_config.get("decision_tree", {}).get("tool_capabilities", {})
        for name, cfg in caps.items():
            self.tools[name] = ToolBrick(name, cfg)

    # ── Phase-lock API (existing callers) ─────────────────────

    def set_phase(self, session_id: str, phase: str) -> None:
        with self._session_lock:
            self._session_phases[session_id] = PhaseContext(phase, session_id)
            logger.info(f"Session {session_id}: phase → {phase.upper()}")

    def get_phase(self, session_id: str) -> str:
        with self._session_lock:
            ctx = self._session_phases.get(session_id)
            return ctx.phase if ctx else self._config.get("default_phase", "BUILD")

    def is_tool_allowed(self, tool_name: str, session_id: str, tool_args: dict = None) -> dict:
        phase = self.get_phase(session_id)
        phase_config = self._config.get("phases", {}).get(phase, {})
        blocked = phase_config.get("blocked_tools", [])

        if tool_name in blocked:
            return {
                "allowed": False,
                "reason": phase_config.get("message", f"Tool {tool_name} bloqué en phase {phase}"),
                "phase": phase,
            }

        if "write_restriction" in phase_config and tool_args:
            path = tool_args.get("path", tool_args.get("file_path", ""))
            if path:
                allowed_paths = phase_config["write_restriction"].get("allowed_paths", [])
                if not any(str(path).startswith(p) for p in allowed_paths):
                    return {
                        "allowed": False,
                        "reason": phase_config["write_restriction"].get(
                            "message", f"Écriture dans {path} non autorisée en phase {phase}"
                        ),
                        "phase": phase,
                    }

        if "exec_restriction" in phase_config and tool_name in ("bash", "python"):
            command = ""
            if tool_args:
                command = tool_args.get("command", tool_args.get("cmd", ""))
            patterns = phase_config["exec_restriction"].get("allowed_commands_patterns", [])
            if command and not any(p in command for p in patterns):
                return {
                    "allowed": False,
                    "reason": phase_config["exec_restriction"].get(
                        "message", f"Commande non autorisée en phase {phase}"
                    ),
                    "phase": phase,
                }

        return {"allowed": True, "reason": "ok", "phase": phase}

    def get_allowed_tools(self, all_tools: list, session_id: str) -> list:
        return [
            t
            for t in all_tools
            if self.is_tool_allowed(t.get("name", t) if isinstance(t, dict) else t, session_id)["allowed"]
        ]

    def get_phase_message(self, session_id: str = None) -> str:
        if session_id:
            phase = self.get_phase(session_id)
        else:
            phase = self._config.get("default_phase", "BUILD")
        phase_config = self._config.get("phases", {}).get(phase, {})
        return phase_config.get("message", f"Phase active : {phase}")

    # ── Tool brick API (decision tree) ────────────────────────

    def get_tools_for_phase(self, phase: str) -> list[ToolBrick]:
        dt = self._brick_config.get("decision_tree", {})
        phase_config = dt.get("phases", {}).get(phase, {})
        priority_order = phase_config.get("priority", [])
        optional_tools = set(phase_config.get("optional", []))
        tool_names = phase_config.get("tools", [])

        available = []
        for tool_name in tool_names:
            if tool_name in self.tools:
                brick = self.tools[tool_name]
                if brick.is_available() or tool_name in optional_tools:
                    available.append(brick)

        if priority_order:
            priority_set = {name: i for i, name in enumerate(priority_order)}
            available.sort(key=lambda b: priority_set.get(b.name, 999))
        return available

    def get_tool(self, name: str) -> ToolBrick | None:
        return self.tools.get(name)

    def get_all_available(self) -> dict[str, ToolBrick]:
        return {name: brick for name, brick in self.tools.items() if brick.is_available()}

    def get_fallback_chain(self, tool_name: str) -> list[str]:
        dt = self._brick_config.get("decision_tree", {})
        fallback_chains = dt.get("fallback_chains", {})
        return list(fallback_chains.get(tool_name, []))

    def get_gates_for_phase(self, phase: str) -> list[str]:
        dt = self._brick_config.get("decision_tree", {})
        phase_config = dt.get("phases", {}).get(phase, {})
        return list(phase_config.get("gates", []))

    def resolve_with_fallback(self, tool_name: str) -> ToolBrick | None:
        brick = self.tools.get(tool_name)
        if brick is None:
            return None
        if brick.is_available():
            return brick
        for fallback_name in self.get_fallback_chain(tool_name):
            fb = self.tools.get(fallback_name)
            if fb and fb.is_available():
                logger.info(f"Fallback: {tool_name} → {fallback_name}")
                return fb
        return None

    def __repr__(self):
        return f"ToolRegistry(tools={len(self.tools)}, available={len(self.get_all_available())})"
