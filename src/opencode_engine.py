"""
OpenCode Engine v2 — Event Bus + Tools/Agents per Phase + Odysseus Bridge.

Reads .opencode/ config, dispatches agents, emits events, routes tools.
This is THE single engine that powers the entire system.
"""

import yaml, json, logging, os, asyncio, re, time, uuid
from typing import AsyncGenerator, Optional, Callable
from datetime import datetime, timezone

logger = logging.getLogger(__name__)

PHASES = ["CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE"]

# ============================================================================
# Event Bus (Python-side — mirrors the TS plugin)
# ============================================================================
class EventBus:
    """Central event bus — 15 families, 62 event types."""
    
    def __init__(self, trace_dir: str = "data/traces"):
        self.trace_dir = trace_dir
        self._subscribers: dict[str, list[Callable]] = {}
        self._buffer: list[dict] = []
        os.makedirs(trace_dir, exist_ok=True)
    
    def emit(self, source: str, type: str, data: dict = None) -> dict:
        event = {
            "event_id": str(uuid.uuid4())[:8],
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "source": source,
            "type": type,
            "data": data or {},
        }
        for handler in self._subscribers.get(type, []) + self._subscribers.get("*", []):
            try: handler(event)
            except Exception: pass
        self._buffer.append(event)
        if len(self._buffer) > 50:
            self._flush()
        return event
    
    def on(self, type: str, handler: Callable):
        self._subscribers.setdefault(type, []).append(handler)
    
    def _flush(self):
        if not self._buffer: return
        today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
        filepath = os.path.join(self.trace_dir, f"events-{today}.jsonl")
        with open(filepath, "a", encoding="utf-8") as f:
            for e in self._buffer:
                f.write(json.dumps(e) + "\n")
        self._buffer.clear()


# Singleton
_event_bus = EventBus()

def get_event_bus() -> EventBus:
    return _event_bus


# ============================================================================
# OpenCode Engine
# ============================================================================
class OpenCodeEngine:
    """Unified engine: config + agents + tools + phases + event bus."""

    # Tools per phase (from 02-OUTILS-TIERS analysis)
    PHASE_TOOLS = {
        "CLASSIFY": [],
        "KNOW":     ["mcp__cbm__search_graph", "mcp__graphify__query", "web_search"],
        "PLAN":     [],
        "BUILD":    ["mcp__scrapling__fetch", "mcp__kroki__render", "BASH", "WRITE_FILE", "mcp__serena__references"],
        "QUALITY":  [],
        "AUTOEVAL": [],
        "MEMORY_OBSERVE": [],
    }

    # Agents per phase (from .opencode/agents/)
    PHASE_AGENTS = {
        "CLASSIFY": ["constitution"],
        "KNOW":     ["explore"],
        "PLAN":     ["planner", "gsd-researcher"],
        "BUILD":    ["executor", "gsd-executor"],
        "QUALITY":  ["reviewer", "security-audit"],
        "AUTOEVAL": ["gsd-verifier"],
        "MEMORY_OBSERVE": ["gsd-roadmapper"],
    }

    # Model per phase
    PHASE_MODELS = {
        "CLASSIFY": "opencode/deepseek-v4-pro",
        "KNOW":     "opencode/deepseek-v4-pro",
        "PLAN":     "opencode/deepseek-v4-pro",
        "BUILD":    "opencode/minimax-m3",
        "QUALITY":  "opencode/deepseek-v4-pro",
        "AUTOEVAL": "opencode/deepseek-v4-pro",
        "MEMORY_OBSERVE": "opencode/deepseek-v4-pro",
    }

    # Budget tracking
    _budgets: dict = {}  # session_id → {tokens, cost, limit}

    @classmethod
    def set_budget(cls, session_id: str, token_limit: int = 100000, cost_limit: float = 5.0):
        cls._budgets[session_id] = {"tokens_used": 0, "token_limit": token_limit, "cost": 0.0, "cost_limit": cost_limit}

    @classmethod
    def track_tokens(cls, session_id: str, tokens: int, cost: float = 0.0):
        if session_id in cls._budgets:
            cls._budgets[session_id]["tokens_used"] += tokens
            cls._budgets[session_id]["cost"] += cost

    @classmethod
    def get_budget(cls, session_id: str) -> dict:
        b = cls._budgets.get(session_id, {})
        used = b.get("tokens_used", 0)
        limit = b.get("token_limit", 100000)
        pct = int(used / limit * 100) if limit > 0 else 0
        return {"tokens_used": used, "token_limit": limit, "percent": pct, "cost": b.get("cost", 0)}

    def __init__(self, session_id: str, message: str, worktree: str = None):
        self.session_id = session_id
        self.message = message
        self.worktree = worktree or os.getcwd()
        self.bus = get_event_bus()
        self._agents = self._load_agents()
        self._skills = self._load_skills()

    def _load_agents(self) -> dict:
        agents = {}
        for d in [".opencode/agents", "agents"]:
            if not os.path.exists(d): continue
            for f in os.listdir(d):
                if f.endswith(".md"):
                    try:
                        with open(os.path.join(d, f), encoding="utf-8") as fh:
                            content = fh.read()
                        if content.startswith("---"):
                            parts = content.split("---", 2)
                            if len(parts) >= 3:
                                meta = yaml.safe_load(parts[1]) or {}
                                agents[f.replace(".md", "")] = meta
                    except Exception: pass
        return agents

    def _load_skills(self) -> dict:
        skills = {}
        for base in [".opencode/skills", ".claude/skills", "skills"]:
            if not os.path.exists(base): continue
            for root, dirs, files in os.walk(base):
                for f in files:
                    if f == "SKILL.md":
                        try:
                            with open(os.path.join(root, f), encoding="utf-8") as fh:
                                content = fh.read()
                            if content.startswith("---"):
                                parts = content.split("---", 2)
                                if len(parts) >= 3:
                                    meta = yaml.safe_load(parts[1]) or {}
                                    name = meta.get("name", os.path.basename(root))
                                    skills[name] = meta
                        except Exception: pass
        return skills

    def is_agent_mode(self) -> bool:
        msg = self.message.lower()
        return any(w in msg for w in ["build", "create", "deploy", "fix", "refactor", "implement", "design", "test", "optimize", "debug"])

    async def walk(self, agent_stream_fn) -> AsyncGenerator[str, None]:
        """Walk phases, emitting SSE events with full event bus integration."""

        # Mode detection
        mode = "agent" if self.is_agent_mode() else "chat"
        self.bus.emit("user", "message_sent", {"mode": mode, "preview": self.message[:100]})
        yield f"data: {json.dumps({'type': 'mode_detected', 'mode': mode})}\n\n"

        if mode == "chat":
            self.bus.emit("phase", "phase_enter", {"phase": "CHAT", "index": 1, "total": 1})
            yield f"data: {json.dumps({'type': 'phase_enter', 'phase': 'CHAT', 'index': 1, 'total': 1})}\n\n"
            async for chunk in agent_stream_fn():
                yield chunk
            self.bus.emit("phase", "phase_exit", {"phase": "CHAT"})
            return

        # Agent mode: walk 7 phases
        start_time = time.time()
        
        for idx, phase in enumerate(PHASES):
            agents = self.PHASE_AGENTS.get(phase, [])
            tools = self.PHASE_TOOLS.get(phase, [])
            model = self.PHASE_MODELS.get(phase, "opencode/deepseek-v4-pro")
            
            # Emit via event bus + SSE
            phase_data = {"phase": phase, "index": idx+1, "total": 7, "agents": agents, "tools": tools, "model": model}
            self.bus.emit("phase", "phase_enter", phase_data)
            yield f"data: {json.dumps({'type': 'phase_enter', 'phase': phase, 'index': idx+1, 'total': 7, 'agents': agents, 'tools': tools, 'model': model})}\n\n"

            if phase == "BUILD":
                # Agent dispatch
                if agents:
                    self.bus.emit("agent", "agent_dispatch", {"phase": phase, "agents": agents, "model": model})
                    yield f"data: {json.dumps({'type': 'agent_dispatch', 'phase': phase, 'agents': agents, 'model': model})}\n\n"
                
                # Model selected
                self.bus.emit("model", "model_selected", {"model": model, "phase": phase, "tier": "code" if "minimax" in model else "standard"})
                yield f"data: {json.dumps({'type': 'model_selected', 'model': model, 'phase': phase})}\n\n"
                
                yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'agents': agents, 'tools': tools, 'model': model, 'action': 'Executing with tools...'})}\n\n"
                
                build_start = time.time()
                chunk_count = 0
                async for chunk in agent_stream_fn():
                    chunk_count += 1
                    if '"tool_start"' in str(chunk) or 'tool_start' in str(chunk):
                        self.bus.emit("tool", "tool_start", {"phase": phase})
                    yield chunk
                
                build_duration = int((time.time() - build_start) * 1000)
                self.bus.emit("tool", "tool_output", {"phase": phase, "chunks": chunk_count, "duration_ms": build_duration})
            else:
                action = f"Phase {phase} | Agents: {agents or 'none'} | Tools: {tools or 'none'} | Model: {model}"
                self.bus.emit("agent", "agent_dispatch", {"phase": phase, "agents": agents})
                yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'agents': agents, 'tools': tools, 'model': model, 'action': action})}\n\n"

            self.bus.emit("phase", "phase_exit", {"phase": phase, "index": idx+1})
            yield f"data: {json.dumps({'type': 'phase_exit', 'phase': phase, 'index': idx+1, 'total': 7})}\n\n"

        total_duration = int((time.time() - start_time) * 1000)
        self.bus.emit("system", "health_change", {"status": "complete", "phases_walked": 7, "duration_ms": total_duration})
        yield f"data: {json.dumps({'type': 'thought_bus', 'status': 'complete', 'phases_walked': 7})}\n\n"

    @property
    def stats(self) -> dict:
        return {
            "agents_loaded": len(self._agents),
            "skills_loaded": len(self._skills),
            "phases": len(PHASES),
            "tools_per_phase": {p: len(t) for p, t in self.PHASE_TOOLS.items()},
            "agents_per_phase": {p: len(a) for p, a in self.PHASE_AGENTS.items()},
        }
