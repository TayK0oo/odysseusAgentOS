"""
OpenCode Engine Bridge — Uses OpenCode configuration + ZenRouter API.

Reads agent definitions from .opencode/agents/*.md
Uses ZenRouter for LLM calls (already proven working)
Dispatches tools/agents based on OpenCode config
Emits cockpit phase events
"""

import yaml, json, logging, os, asyncio, re
from typing import AsyncGenerator, Optional

logger = logging.getLogger(__name__)

PHASES = ["CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE"]

class OpenCodeEngine:
    """Bridges Odysseus UI with OpenCode configuration and ZenRouter API."""

    def __init__(self, session_id: str, message: str, worktree: Optional[str] = None):
        self.session_id = session_id
        self.message = message
        self.worktree = worktree or os.getcwd()
        self._agents = self._load_agents()
        self._skills = self._load_skills()

    def _load_agents(self) -> dict:
        """Load agent definitions from .opencode/agents/*.md"""
        agents = {}
        agents_dir = ".opencode/agents"
        if not os.path.exists(agents_dir):
            return agents
        for f in os.listdir(agents_dir):
            if f.endswith(".md"):
                try:
                    with open(os.path.join(agents_dir, f), "r", encoding="utf-8") as fh:
                        content = fh.read()
                    if content.startswith("---"):
                        parts = content.split("---", 2)
                        if len(parts) >= 3:
                            meta = yaml.safe_load(parts[1]) or {}
                            meta["body"] = parts[2].strip()
                            agents[f.replace(".md", "")] = meta
                except Exception:
                    pass
        return agents

    def _load_skills(self) -> dict:
        """Load skill definitions from .opencode/skills/*/SKILL.md"""
        skills = {}
        skills_base = ".opencode/skills"
        if not os.path.exists(skills_base):
            # Also check Claude-compatible path
            skills_base = ".claude/skills"
        if not os.path.exists(skills_base):
            return skills
        for root, dirs, files in os.walk(skills_base):
            for f in files:
                if f == "SKILL.md":
                    try:
                        with open(os.path.join(root, f), "r", encoding="utf-8") as fh:
                            content = fh.read()
                        if content.startswith("---"):
                            parts = content.split("---", 2)
                            if len(parts) >= 3:
                                meta = yaml.safe_load(parts[1]) or {}
                                name = meta.get("name", os.path.basename(root))
                                meta["body"] = parts[2].strip()
                                skills[name] = meta
                    except Exception:
                        pass
        return skills

    def get_agents_for_phase(self, phase: str) -> list[str]:
        """Get agent names recommended for a phase."""
        # From our SFD decision tree / orchestrator agent
        phase_agents = {
            "CLASSIFY": [],
            "KNOW": ["explore"],
            "PLAN": ["gsd-planner", "gsd-researcher", "planner"],
            "BUILD": ["gsd-executor", "executor", "open-coder"],
            "QUALITY": ["debate-5-personas", "security-audit", "reviewer"],
            "AUTOEVAL": ["gsd-verifier", "edge-case-gen"],
            "MEMORY_OBSERVE": ["gsd-roadmapper"],
        }
        names = phase_agents.get(phase, [])
        return [n for n in names if n in self._agents]

    def get_model_for_phase(self, phase: str) -> str:
        """Recommend model per phase based on agent configs."""
        defaults = {
            "CLASSIFY": "opencode/deepseek-v4-pro",
            "KNOW": "opencode/deepseek-v4-pro",
            "PLAN": "opencode/deepseek-v4-pro",
            "BUILD": "opencode/minimax-m3",
            "QUALITY": "opencode/deepseek-v4-pro",
            "AUTOEVAL": "opencode/deepseek-v4-pro",
            "MEMORY_OBSERVE": "opencode/deepseek-v4-pro",
        }
        agents = self.get_agents_for_phase(phase)
        for name in agents:
            agent = self._agents.get(name, {})
            if agent.get("model"):
                return agent["model"]
        return defaults.get(phase, "opencode/deepseek-v4-pro")

    async def walk_phases(self, agent_stream_fn) -> AsyncGenerator[str, None]:
        """Walk 7 phases, emitting cockpit events and agent dispatch info."""
        msg = self.message.lower()
        is_agent = any(w in msg for w in ["build", "create", "deploy", "fix", "refactor", "implement", "design"])

        if not is_agent:
            # Chat mode: single phase, no agents
            yield f"data: {json.dumps({'type': 'mode_detected', 'mode': 'chat'})}\n\n"
            async for chunk in agent_stream_fn():
                yield chunk
            return

        # Agent mode: walk all 7 phases
        yield f"data: {json.dumps({'type': 'mode_detected', 'mode': 'agent'})}\n\n"
        
        for idx, phase in enumerate(PHASES):
            agents = self.get_agents_for_phase(phase)
            model = self.get_model_for_phase(phase)
            
            yield f"data: {json.dumps({'type': 'phase_enter', 'phase': phase, 'index': idx+1, 'total': 7})}\n\n"

            if phase == "BUILD":
                # Actual LLM execution
                yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'agents': agents, 'model': model, 'action': 'Executing with tools...'})}\n\n"
                async for chunk in agent_stream_fn():
                    yield chunk
            else:
                action = f"Phase: {phase} | Agents: {agents or 'none'} | Model: {model}"
                yield f"data: {json.dumps({'type': 'phase_active', 'phase': phase, 'agents': agents, 'model': model, 'action': action})}\n\n"

            yield f"data: {json.dumps({'type': 'phase_exit', 'phase': phase, 'index': idx+1, 'total': 7})}\n\n"

        yield f"data: {json.dumps({'type': 'thought_bus', 'status': 'complete', 'phases_walked': 7})}\n\n"

    @property
    def stats(self) -> dict:
        return {
            "agents_loaded": len(self._agents),
            "skills_loaded": len(self._skills),
            "phases": len(PHASES),
        }
