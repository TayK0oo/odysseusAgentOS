"""Native orchestration layer for Odysseus (Milestone 2)."""
from src.orchestrator.spec import AgentSpec, parse_agent_spec
from src.orchestrator.registry import AgentRegistry
from src.orchestrator.dispatcher import resolve_model, dispatch, DispatchResult
from src.orchestrator.phases import Phase, CANONICAL_SEQUENCE, phase_lock_name, forced_tools
from src.orchestrator.loop import CanonicalLoop
from src.orchestrator.gate import should_block_destructive, gate_enabled
from src.orchestrator.phase_tracker import PhaseTracker, tracker_enabled
from src.orchestrator.router_advice import advise, router_enabled
from src.orchestrator.autoeval import (
    autoeval_enabled,
    decide_keep_or_revert,
    apply_autoeval,
    AutoevalDecision,
)
from src.orchestrator.autoevolve import (
    autoevolve_enabled,
    maybe_autoevolve,
    trigger_improvement_research,
)
# LangGraph imports are lazy — only available when langgraph is installed.
try:
    from src.orchestrator.langgraph_loop import (
        langgraph_enabled,
        build_langgraph,
        langgraph_stream,
        build_input_state,
        AgentState,
    )
except ImportError:
    langgraph_enabled = None  # type: ignore
    build_langgraph = None  # type: ignore
    langgraph_stream = None  # type: ignore
    build_input_state = None  # type: ignore
    AgentState = None  # type: ignore

__all__ = [
    "AgentSpec", "parse_agent_spec", "AgentRegistry", "resolve_model",
    "dispatch", "DispatchResult",
    "Phase", "CANONICAL_SEQUENCE", "phase_lock_name", "forced_tools", "CanonicalLoop",
    "should_block_destructive", "gate_enabled",
    "PhaseTracker", "tracker_enabled",
    "advise", "router_enabled",
    "autoeval_enabled", "decide_keep_or_revert", "apply_autoeval", "AutoevalDecision",
    "autoevolve_enabled", "maybe_autoevolve", "trigger_improvement_research",
    "langgraph_enabled", "build_langgraph", "langgraph_stream",
    "build_input_state", "AgentState",
]
