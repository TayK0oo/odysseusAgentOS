"""Read-only registry of odysseus kill-switches.

Pure module (no file I/O, no mutation of os.environ). Each descriptor is
curated and cites the exact code location where the switch is read, so the
dashboard stays honest and verifiable. read_states() resolves the live
os.environ value for each switch; a switch whose env var is unset reports
is_default=True and falls back to the code default.
"""

from __future__ import annotations

import os
from typing import Any

_TRUTHY = ("1", "true", "yes", "on")


def _truthy(value: str | None) -> bool:
    return str(value).strip().lower() in _TRUTHY if value is not None else False


# Curated set — grounded in code (rg 'os\.(environ\.get|getenv)\("ODYSSEUS_')
# plus the agent/channel gates. Config params (paths, ids, models) are excluded.
_SWITCHES: list[dict[str, Any]] = [
    # -- Orchestration --
    {
        "name": "Live orchestration",
        "env_var": "ODYSSEUS_LIVE_ORCHESTRATION",
        "default": "on",
        "category": "Orchestration",
        "timing": "runtime",
        "desc": "CanonicalLoop 7 phases.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "Phase tracker",
        "env_var": "ODYSSEUS_PHASE_TRACKER",
        "default": "on",
        "category": "Orchestration",
        "timing": "runtime",
        "desc": "Infere la phase par round et la pousse au ToolRegistry.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "Model router",
        "env_var": "ODYSSEUS_MODEL_ROUTER",
        "default": "on",
        "category": "Orchestration",
        "timing": "runtime",
        "desc": "Routing advisory log-only (n'ecrase pas le modele user).",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "Destructive gate",
        "env_var": "ODYSSEUS_DESTRUCTIVE_GATE",
        "default": "on",
        "category": "Orchestration",
        "timing": "runtime",
        "desc": "Garde-fou operations destructrices (ON par defaut).",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    # -- Governance / Memory --
    {
        "name": "Autoeval",
        "env_var": "ODYSSEUS_AUTOEVAL",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "Decision keep/revert post-build.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "Autoeval: exécution destructrice",
        "env_var": "ODYSSEUS_AUTOEVAL_ALLOW_RESET",
        "default": "off",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "AUTOEVAL peut DECIDER un revert ; ce switch autorise son "
                "EXECUTION (git reset --hard). OFF par defaut : la decision "
                "est tracee, l'action jamais jouee.",
        "source": "src/orchestrator/autoeval.py:62",
    },
    {
        "name": "Autoevolve",
        "env_var": "ODYSSEUS_AUTOEVOLVE",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "Recherche d'amelioration apres drift HIGH + revert.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "CodeBurn",
        "env_var": "ODYSSEUS_CODEBURN",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "Rapport one-shot rate / waste / cout.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "LangFuse",
        "env_var": "ODYSSEUS_LANGFUSE",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "LLM tracing & observability dashboard (self-hosted).",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "Governance ancestry",
        "env_var": "ODYSSEUS_GOVERNANCE_ANCESTRY",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "Ecrit GoalTask avec ancestry mission->goal->project->task.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "LangGraph loop",
        "env_var": "ODYSSEUS_LANGGRAPH",
        "default": "on",
        "category": "Orchestration",
        "timing": "runtime",
        "desc": "StateGraph 7-nœuds (remplace stream_agent_loop).",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "LangGraph interrupt",
        "env_var": "ODYSSEUS_LANGGRAPH_INTERRUPT",
        "default": "on",
        "category": "Orchestration",
        "timing": "runtime",
        "desc": "Interrupt before BUILD when risk==DESTRUCTIVE.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "Checkpoint",
        "env_var": "ODYSSEUS_CHECKPOINT",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "Persiste un checkpoint post-round dans Obsidian.",
        "source": "src/orchestrator/checkpoint_tracker.py:39",
    },
    {
        "name": "Unified tokens",
        "env_var": "ODYSSEUS_UNIFIED_TOKENS",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "Comptage tokens unifie avec run_id de correlation.",
        "source": "src/memory_impact.py:23",
    },
    # -- RAG --
    {
        "name": "RRF fusion",
        "env_var": "ODYSSEUS_RRF_FUSION",
        "default": "on",
        "category": "RAG",
        "timing": "runtime",
        "desc": "Fusion vecteur+BM25 par Reciprocal Rank Fusion.",
        "source": "src/memory_impact.py:23",
    },
    # -- Document Processing --
    {
        "name": "Docling",
        "env_var": "ODYSSEUS_DOCLING",
        "default": "on",
        "category": "Document Processing",
        "timing": "runtime",
        "desc": "PDF extraction via Docling (tables, layout, reading order).",
        "source": "src/memory_impact.py:23",
    },
    # -- Quality --
    {
        "name": "DeepEval",
        "env_var": "ODYSSEUS_DEEPEVAL",
        "default": "on",
        "category": "Quality",
        "timing": "runtime",
        "desc": "LLM quality evaluation (faithfulness, relevancy, hallucination, bias, toxicity).",
        "source": "src/memory_impact.py:23",
    },
    # -- Code Parsing --
    {
        "name": "Tree-sitter",
        "env_var": "ODYSSEUS_TREESITTER",
        "default": "on",
        "category": "Code Parsing",
        "timing": "runtime",
        "desc": "Incremental syntax parsing (AST, symbols, call sites).",
        "source": "src/memory_impact.py:23",
    },
    # -- MCP / Services --
    {
        "name": "Disable MCP",
        "env_var": "ODYSSEUS_DISABLE_MCP",
        "default": "on",
        "category": "MCP/Services",
        "timing": "startup",
        "desc": "Coupe tous les serveurs MCP built-in (lu au boot).",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Obsidian MCP",
        "env_var": "ODYSSEUS_OBSIDIAN_MCP",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active le serveur MCP Obsidian.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Graphify",
        "env_var": "ODYSSEUS_GRAPHIFY",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active le serveur MCP Graphify.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "CBM (Codebase Memory)",
        "env_var": "ODYSSEUS_CBM",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active le client CBM pour le graphe semantique de code.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Serena MCP",
        "env_var": "ODYSSEUS_SERENA_MCP",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active le client Serena MCP (find_references, goto_definition, etc.).",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "AgentSeal",
        "env_var": "ODYSSEUS_AGENTSEAL",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active le scanner d'injection AgentSeal (prompts + outputs).",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Supabase",
        "env_var": "ODYSSEUS_SUPABASE",
        "default": "on",
        "category": "MCP/Services",
        "timing": "startup",
        "desc": "Active l'integration Supabase (backend-as-a-service).",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Vaultwarden",
        "env_var": "ODYSSEUS_VAULTWARDEN",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active l'integration Vaultwarden (password manager).",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Playwright MCP",
        "env_var": "ODYSSEUS_PLAYWRIGHT",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active le serveur MCP Playwright pour le browsing headless.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Browser Harness",
        "env_var": "ODYSSEUS_BROWSER_HARNESS",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active le harness de navigateur pour les tests E2E.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Zen from endpoint",
        "env_var": "ODYSSEUS_ZEN_FROM_ENDPOINT",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Route Zen via un endpoint enregistre.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "n8n integration",
        "env_var": "ODYSSEUS_N8N",
        "default": "on",
        "category": "MCP/Services",
        "timing": "runtime",
        "desc": "Active la routing n8n pour trigger de workflows.",
        "source": "src/memory_impact.py:23",
    },
    # -- Channels --
    {
        "name": "In-process Discord",
        "env_var": "ODYSSEUS_INPROCESS_DISCORD",
        "default": "on",
        "category": "Channels",
        "timing": "startup",
        "desc": "Bootstrap du bot Discord in-process.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "In-process Telegram",
        "env_var": "ODYSSEUS_INPROCESS_TELEGRAM",
        "default": "on",
        "category": "Channels",
        "timing": "startup",
        "desc": "Bootstrap du bot Telegram in-process.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Channel agent reply",
        "env_var": "ODYSSEUS_CHANNEL_AGENT_REPLY",
        "default": "on",
        "category": "Channels",
        "timing": "runtime",
        "desc": "Reponse auto de l'agent sur les channels.",
        "source": "src/memory_impact.py:23",
    },
    # -- Agents --
    {
        "name": "Agent catalog",
        "env_var": "ODYSSEUS_AGENT_CATALOG",
        "default": "on",
        "category": "Agents",
        "timing": "startup",
        "desc": "Charge le catalogue d'agents .opencode au boot.",
        "source": "src/memory_impact.py:23",
    },
    # -- SFD Feature Modules --
    {
        "name": "Progressive disclosure",
        "env_var": "ODYSSEUS_PROGRESSIVE_DISCLOSURE",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P5: Restreint les outils visibles par phase+risque.",
        "source": "src/memory_impact.py:23",
    },
    {
        "name": "Memory impact verification",
        "env_var": "ODYSSEUS_MEMORY_IMPACT",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P19: Ne stocke que les faits qui changent les reponses.",
        "source": "src/memory_impact.py:23",
    },
    # -- Palier 0 SFD feature modules (P0) --
    {
        "name": "Output router",
        "env_var": "ODYSSEUS_OUTPUT_ROUTER",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P22: Routage de la sortie (HTML/SVG inline, Kroki).",
        "source": "src/output_router.py:26",
    },
    {
        "name": "Preferences",
        "env_var": "ODYSSEUS_PREFERENCES",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P12: Preferences utilisateur persistees.",
        "source": "src/preferences.py:42",
    },
    {
        "name": "Data classification",
        "env_var": "ODYSSEUS_DATA_CLASSIFICATION",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P21: Classification 5 niveaux de retention.",
        "source": "src/data_classification.py:29",
    },
    {
        "name": "Content security",
        "env_var": "ODYSSEUS_CONTENT_SECURITY",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P20: Injection guard + filtre de sortie.",
        "source": "src/content_security.py:24",
    },
    {
        "name": "Durable execution",
        "env_var": "ODYSSEUS_DURABLE_EXECUTION",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P14: Execution durable (Custom SQLite).",
        "source": "src/durable_execution.py:32",
    },
    {
        "name": "Provenance memory",
        "env_var": "ODYSSEUS_PROVENANCE_MEMORY",
        "default": "on",
        "category": "Governance/Memory",
        "timing": "runtime",
        "desc": "P16: Memoire taguee [stated]/[observed]/[inferred].",
        "source": "src/provenance_memory.py:40",
    },
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
_SWITCHES.extend(
    {
        "name": label,
        "env_var": var,
        "default": "off",
        "category": "Agents",
        "timing": "runtime",
        "desc": "Dispatch auto de l'agent par phase canonique.",
        "source": "src/orchestrator/agent_dispatcher.py",
    }
    for label, var in _AGENT_SWITCHES
)

_CATEGORY_ORDER = [
    "Orchestration",
    "Governance/Memory",
    "RAG",
    "Code Parsing",
    "MCP/Services",
    "Document Processing",
    "Quality",
    "Channels",
    "Agents",
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
