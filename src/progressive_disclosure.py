"""SFD §5.26 — P5: Progressive Disclosure.

Controls tool visibility based on the current phase and risk level so the
model never operates with more tool surface than its immediate task demands.

Gated behind ODYSSEUS_PROGRESSIVE_DISCLOSURE kill-switch (default OFF).
"""

from __future__ import annotations

import logging
import os
from enum import Enum
from typing import Any

from src.orchestrator.phases import Phase
from src.risk_classifier import RiskLevel

logger = logging.getLogger(__name__)

# ─── Kill-switch ────────────────────────────────────────────────────────


def progressive_disclosure_enabled() -> bool:
    val = os.getenv("ODYSSEUS_PROGRESSIVE_DISCLOSURE", "on").strip().lower()
    return val in {"on", "1", "true", "yes"}


# ─── Disclosure levels ──────────────────────────────────────────────────


class DisclosureLevel(Enum):
    MINIMAL = "minimal"
    STANDARD = "standard"
    FULL = "full"


# ─── Phase → disclosure mapping ─────────────────────────────────────────

_PHASE_DISCLOSURE: dict[Phase, DisclosureLevel] = {
    Phase.CLASSIFY: DisclosureLevel.MINIMAL,
    Phase.KNOW: DisclosureLevel.STANDARD,
    Phase.PLAN: DisclosureLevel.STANDARD,
    Phase.BUILD: DisclosureLevel.FULL,
    Phase.QUALITY: DisclosureLevel.FULL,
    Phase.AUTOEVAL: DisclosureLevel.MINIMAL,
    Phase.MEMORY_OBSERVE: DisclosureLevel.MINIMAL,
}

# Higher risk caps disclosure (riskier → fewer tools visible)
_RISK_CAP: dict[RiskLevel, DisclosureLevel] = {
    RiskLevel.READ: DisclosureLevel.FULL,
    RiskLevel.DRAFT: DisclosureLevel.FULL,
    RiskLevel.WRITE: DisclosureLevel.STANDARD,
    RiskLevel.EXEC: DisclosureLevel.MINIMAL,
    RiskLevel.DESTRUCTIVE: DisclosureLevel.MINIMAL,
}

# Tool categories permitted per disclosure level
_TOOL_CATEGORIES: dict[DisclosureLevel, set[str]] = {
    DisclosureLevel.MINIMAL: {"read", "classify"},
    DisclosureLevel.STANDARD: {"read", "classify", "search", "memory", "planning"},
    DisclosureLevel.FULL: {"read", "classify", "search", "memory", "planning", "write", "exec", "mcp", "network"},
}

_TOOL_CATEGORY_MAP: dict[str, str] = {
    "read_file": "read",
    "grep": "read",
    "glob": "read",
    "ls": "read",
    "search_chats": "search",
    "list_served_models": "read",
    "list_downloads": "read",
    "list_cached_models": "read",
    "list_serve_presets": "read",
    "list_cookbook_servers": "read",
    "list_models": "read",
    "list_sessions": "read",
    "vault_search": "search",
    "vault_get": "read",
    "risk_classifier": "classify",
    "memory_search": "memory",
    "memory_distill": "memory",
    "trace_writer": "memory",
    "manage_memory": "memory",
    "create_plan": "planning",
    "update_plan": "planning",
    "write_file": "write",
    "edit_file": "write",
    "create_document": "write",
    "update_document": "write",
    "edit_document": "write",
    "suggest_document": "write",
    "manage_notes": "write",
    "manage_tasks": "write",
    "manage_calendar": "write",
    "manage_skills": "write",
    "manage_documents": "write",
    "bash": "exec",
    "python": "exec",
    "api_call": "exec",
    "serve_model": "exec",
    "adopt_served_model": "exec",
    "trigger_research": "exec",
    "pipeline": "exec",
    "chat_with_model": "exec",
    "ask_teacher": "exec",
    "edit_image": "exec",
    "remove_dir": "exec",
    "stop_served_model": "exec",
    "cancel_download": "exec",
}


class ProgressiveDisclosure:
    def __init__(self):
        pass

    def resolve_level(self, phase: Phase, risk_level: RiskLevel | None = None) -> DisclosureLevel:
        phase_level = _PHASE_DISCLOSURE.get(phase, DisclosureLevel.FULL)

        if risk_level is not None:
            risk_cap = _RISK_CAP.get(risk_level, DisclosureLevel.FULL)
            disclosure_order = [DisclosureLevel.MINIMAL, DisclosureLevel.STANDARD, DisclosureLevel.FULL]
            phase_idx = disclosure_order.index(phase_level)
            risk_idx = disclosure_order.index(risk_cap)
            return disclosure_order[min(phase_idx, risk_idx)]

        return phase_level

    def get_allowed_tools(self, level: DisclosureLevel) -> set[str]:
        allowed_categories = _TOOL_CATEGORIES.get(level, set())
        allowed_tools: set[str] = set()
        for tool, category in _TOOL_CATEGORY_MAP.items():
            if category in allowed_categories:
                allowed_tools.add(tool)
        return allowed_tools

    def is_tool_allowed(self, tool_name: str, phase: Phase, risk_level: RiskLevel | None = None) -> bool:
        level = self.resolve_level(phase, risk_level)
        allowed = self.get_allowed_tools(level)
        return tool_name in allowed

    def get_disclosure_summary(self, phase: Phase, risk_level: RiskLevel | None = None) -> dict[str, Any]:
        level = self.resolve_level(phase, risk_level)
        allowed = self.get_allowed_tools(level)
        return {
            "phase": phase.value,
            "risk_level": risk_level.value if risk_level else "none",
            "disclosure_level": level.value,
            "allowed_tool_count": len(allowed),
            "allowed_tools": sorted(allowed),
        }


_disclosure: ProgressiveDisclosure | None = None


def get_progressive_disclosure() -> ProgressiveDisclosure:
    global _disclosure
    if _disclosure is None:
        _disclosure = ProgressiveDisclosure()
    return _disclosure
