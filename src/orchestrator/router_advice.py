"""Advisory model routing for the LIVE loop (M3.1).

`intent_gate.classify_intent` and `llm_router.ModelRouter` were fully coded but
had ZERO live call-sites — the chat never invoked them. This module provides the
first live call-site: at the head of a turn it classifies the user's intent and
asks the ModelRouter which model it *would* pick.

SAFETY / SCOPE: advisory only. It NEVER overrides the user's chosen model
(sess.model). It is OFF by default (ODYSSEUS_MODEL_ROUTER); when ON it only logs
the suggestion. Real override (auto-routing) is deliberately deferred until two
decisions are made: (1) the real model catalog for the enabled provider — the
current model-routing.json IDs are placeholders; (2) whether auto-routing may
override an explicit user model choice.
"""
from __future__ import annotations

import logging
import os
from typing import Optional

from src.intent_gate import classify_intent

logger = logging.getLogger(__name__)


def router_enabled() -> bool:
    """OFF unless ODYSSEUS_MODEL_ROUTER is set to a truthy value."""
    val = os.getenv("ODYSSEUS_MODEL_ROUTER", "off").strip().lower()
    return val in {"on", "1", "true", "yes"}


def advise(user_text: str, stage: Optional[str] = None, router=None) -> dict:
    """Classify intent and get the router's suggested model (advisory).

    Returns {"intent": str, "stage": str|None, "suggested_model": str|None}.
    Never raises: a router failure yields suggested_model=None.
    """
    intent = classify_intent(user_text or "")

    _router = router
    if _router is None:
        try:
            from src.llm_router import ModelRouter
            _router = ModelRouter()
        except Exception as exc:  # litellm/config import issues must not break chat
            logger.debug("[router-advice] ModelRouter unavailable: %s", exc)
            _router = None

    suggested = None
    if _router is not None:
        try:
            suggested = _router.route(intent, stage)
        except Exception as exc:
            logger.debug("[router-advice] route() failed: %s", exc)
            suggested = None

    return {"intent": intent, "stage": stage, "suggested_model": suggested}
