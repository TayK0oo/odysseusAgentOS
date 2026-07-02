"""PhaseTracker — the bridge that gives the LIVE loop a per-round phase (M3.0).

The live loop (stream_agent_loop) historically never called
ToolRegistry.set_phase(), so every session fell back to default_phase=BUILD and
the phase-lock was effectively inert live. This module lets the live round loop
set a phase per round through a single, shared, kill-switched call site.

SAFETY: the tracker is OFF by default (ODYSSEUS_PHASE_TRACKER). When OFF,
set_phase() is never called and live behavior is byte-for-byte what it is today.
When ON, phases are inferred and pushed into the phase-lock registry. The
inference is intentionally conservative for now (BUILD, i.e. permissive, unless
plan_mode); later M3.x waves enrich infer_phase() as each module needs its phase.
"""
from __future__ import annotations

import logging
import os
from typing import Optional

logger = logging.getLogger(__name__)


def tracker_enabled() -> bool:
    """OFF unless ODYSSEUS_PHASE_TRACKER is set to a truthy value."""
    val = os.getenv("ODYSSEUS_PHASE_TRACKER", "off").strip().lower()
    return val in {"on", "1", "true", "yes"}


class PhaseTracker:
    def __init__(self, session_id: str, registry=None, enabled: Optional[bool] = None):
        self.session_id = session_id or "live"
        self._enabled = tracker_enabled() if enabled is None else bool(enabled)
        if registry is not None:
            self._registry = registry
        elif self._enabled:
            # Lazy import so importing this module never forces the singleton.
            from src.tool_registry import ToolRegistry
            self._registry = ToolRegistry.get_instance()
        else:
            self._registry = None

    @staticmethod
    def infer_phase(round_num: int, intent: Optional[dict] = None,
                    plan_mode: bool = False) -> str:
        """Return the phase-lock name for this round.

        Conservative live mapping (M3.0): plan_mode reinforces PLAN (writes are
        already read-only there); everything else stays BUILD (permissive) so
        enabling the tracker does not change normal chat behavior. Enriched by
        later waves (M3.1 CLASSIFY routing, M3.2 KNOW retrieval, ...).
        """
        if plan_mode:
            return "PLAN"
        return "BUILD"

    def on_round_start(self, round_num: int, intent: Optional[dict] = None,
                       plan_mode: bool = False) -> str:
        """Infer the phase for this round and (if enabled) push it to the registry."""
        phase = self.infer_phase(round_num, intent=intent, plan_mode=plan_mode)
        if self._enabled and self._registry is not None:
            self._registry.set_phase(self.session_id, phase)
            logger.info("[PhaseTracker] session=%s round=%s → %s",
                        self.session_id, round_num, phase)
        return phase
