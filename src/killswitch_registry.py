"""Read-only registry of odysseus kill-switches.

Pure module (no file I/O, no mutation of os.environ). Each descriptor is
curated and cites the exact code location where the switch is read, so the
dashboard stays honest and verifiable. read_states() resolves the live
os.environ value for each switch; a switch whose env var is unset reports
is_default=True and falls back to the code default.
"""
from __future__ import annotations

import os
from typing import Any, Optional

_TRUTHY = ("1", "true", "yes", "on")


def _truthy(value: Optional[str]) -> bool:
    return str(value).strip().lower() in _TRUTHY if value is not None else False


# Curated set — grounded in code (rg 'os\.(environ\.get|getenv)\("ODYSSEUS_')
# plus the agent/channel gates. Config params (paths, ids, models) are excluded.
_SWITCHES: list[dict[str, Any]] = [
    # -- Orchestration --
    {"name": "Live orchestration", "env_var": "ODYSSEUS_LIVE_ORCHESTRATION",
     "default": "off", "category": "Orchestration", "timing": "runtime",
     "desc": "CanonicalLoop 7 phases.", "source": "src/agent_loop.py:2493"},
    {"name": "Phase tracker", "env_var": "ODYSSEUS_PHASE_TRACKER",
     "default": "off", "category": "Orchestration", "timing": "runtime",
     "desc": "Infere la phase par round et la pousse au ToolRegistry.",
     "source": "src/orchestrator/phase_tracker.py:25"},
    {"name": "Model router", "env_var": "ODYSSEUS_MODEL_ROUTER",
     "default": "off", "category": "Orchestration", "timing": "runtime",
     "desc": "Routing advisory log-only (n'ecrase pas le modele user).",
     "source": "src/orchestrator/router_advice.py:42"},
    {"name": "Destructive gate", "env_var": "ODYSSEUS_DESTRUCTIVE_GATE",
     "default": "on", "category": "Orchestration", "timing": "runtime",
     "desc": "Garde-fou operations destructrices (ON par defaut).",
     "source": "src/orchestrator/gate.py:26"},
    # -- Governance / Memory --
    {"name": "Autoeval", "env_var": "ODYSSEUS_AUTOEVAL",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Decision keep/revert post-build.",
     "source": "src/orchestrator/autoeval.py:44"},
    {"name": "Autoevolve", "env_var": "ODYSSEUS_AUTOEVOLVE",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Recherche d'amelioration apres drift HIGH + revert.",
     "source": "src/orchestrator/autoevolve.py:18"},
    {"name": "CodeBurn", "env_var": "ODYSSEUS_CODEBURN",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Rapport one-shot rate / waste / cout.",
     "source": "src/orchestrator/codeburn_runner.py:26"},
    {"name": "Governance ancestry", "env_var": "ODYSSEUS_GOVERNANCE_ANCESTRY",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Ecrit GoalTask avec ancestry mission->goal->project->task.",
     "source": "src/orchestrator/ancestry_tracker.py:26"},
    {"name": "Checkpoint", "env_var": "ODYSSEUS_CHECKPOINT",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Persiste un checkpoint post-round dans Obsidian.",
     "source": "src/orchestrator/checkpoint_tracker.py:39"},
    {"name": "Unified tokens", "env_var": "ODYSSEUS_UNIFIED_TOKENS",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Comptage tokens unifie avec run_id de correlation.",
     "source": "src/trace_writer.py:146"},
    # -- RAG --
    {"name": "RRF fusion", "env_var": "ODYSSEUS_RRF_FUSION",
     "default": "off", "category": "RAG", "timing": "runtime",
     "desc": "Fusion vecteur+BM25 par Reciprocal Rank Fusion.",
     "source": "src/rag_vector.py:50"},
    # -- MCP / Services --
    {"name": "Disable MCP", "env_var": "ODYSSEUS_DISABLE_MCP",
     "default": "off", "category": "MCP/Services", "timing": "startup",
     "desc": "Coupe tous les serveurs MCP built-in (lu au boot).",
     "source": "src/builtin_mcp.py:89"},
    {"name": "Obsidian MCP", "env_var": "ODYSSEUS_OBSIDIAN_MCP",
     "default": "off", "category": "MCP/Services", "timing": "runtime",
     "desc": "Active le serveur MCP Obsidian.",
     "source": "src/builtin_mcp.py:98"},
    {"name": "Graphify", "env_var": "ODYSSEUS_GRAPHIFY",
     "default": "off", "category": "MCP/Services", "timing": "runtime",
     "desc": "Active le serveur MCP Graphify.",
     "source": "src/builtin_mcp.py:107"},
    {"name": "Zen from endpoint", "env_var": "ODYSSEUS_ZEN_FROM_ENDPOINT",
     "default": "off", "category": "MCP/Services", "timing": "runtime",
     "desc": "Route Zen via un endpoint enregistre.",
     "source": "src/zen_router.py:131"},
    # -- Channels --
    {"name": "In-process Discord", "env_var": "ODYSSEUS_INPROCESS_DISCORD",
     "default": "off", "category": "Channels", "timing": "startup",
     "desc": "Bootstrap du bot Discord in-process.",
     "source": "src/channel_bootstrap.py:45"},
    {"name": "In-process Telegram", "env_var": "ODYSSEUS_INPROCESS_TELEGRAM",
     "default": "off", "category": "Channels", "timing": "startup",
     "desc": "Bootstrap du bot Telegram in-process.",
     "source": "src/channel_bootstrap.py:50"},
    {"name": "Channel agent reply", "env_var": "ODYSSEUS_CHANNEL_AGENT_REPLY",
     "default": "off", "category": "Channels", "timing": "runtime",
     "desc": "Reponse auto de l'agent sur les channels.",
     "source": "src/channel_bootstrap.py:59"},
    # -- Agents --
    {"name": "Agent catalog", "env_var": "ODYSSEUS_AGENT_CATALOG",
     "default": "off", "category": "Agents", "timing": "startup",
     "desc": "Charge le catalogue d'agents .opencode au boot.",
     "source": "app.py:1254"},
]

# 12 phase/explicit agent switches (agent_dispatcher.py). All default off,
# runtime-read via AgentDispatcher.dispatch_for_phase.
_AGENT_SWITCHES = [
    ("Agent: constitution", "ODYSSEUS_AGENT_CONSTITUTION"),
    ("Agent: gsd-planner", "ODYSSEUS_AGENT_PLANNER"),
    ("Agent: gsd-researcher", "ODYSSEUS_AGENT_RESEARCHER"),
    ("Agent: gsd-executor", "ODYSSEUS_AGENT_EXECUTOR"),
    ("Agent: gsd-debugger", "ODYSSEUS_AGENT_DEBUGGER"),
    ("Agent: debate-5-personas", "ODYSSEUS_AGENT_DEBATE"),
    ("Agent: security-audit", "ODYSSEUS_AGENT_SECURITY"),
    ("Agent: gsd-verifier", "ODYSSEUS_AGENT_VERIFIER"),
    ("Agent: edge-case-gen", "ODYSSEUS_AGENT_EDGECASE"),
    ("Agent: gsd-roadmapper", "ODYSSEUS_AGENT_ROADMAPPER"),
    ("Agent: design-extract", "ODYSSEUS_AGENT_DESIGN_EXTRACT"),
    ("Agent: open-design", "ODYSSEUS_AGENT_OPEN_DESIGN"),
]
for _label, _var in _AGENT_SWITCHES:
    _SWITCHES.append({
        "name": _label, "env_var": _var, "default": "off",
        "category": "Agents", "timing": "runtime",
        "desc": "Dispatch auto de l'agent par phase canonique.",
        "source": "src/orchestrator/agent_dispatcher.py",
    })

_CATEGORY_ORDER = [
    "Orchestration", "Governance/Memory", "RAG",
    "MCP/Services", "Channels", "Agents",
]


def categories() -> list[str]:
    """Stable display order of switch categories."""
    return list(_CATEGORY_ORDER)


def read_states() -> list[dict[str, Any]]:
    """Return each descriptor enriched with live env state.

    Adds raw (env value or None), is_default (True if unset), and effective
    (resolved bool). Best-effort per descriptor: never raises.
    """
    rows: list[dict[str, Any]] = []
    for d in _SWITCHES:
        raw = os.environ.get(d["env_var"])
        is_default = raw is None
        effective = _truthy(d["default"]) if is_default else _truthy(raw)
        rows.append({**d, "raw": raw, "is_default": is_default, "effective": effective})
    return rows
