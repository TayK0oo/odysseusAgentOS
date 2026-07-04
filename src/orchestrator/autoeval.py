"""AUTOEVAL keep/revert verifier (M3.3).

The AUTOEVAL canonical phase (see src/orchestrator/phases.py) decides whether to
KEEP or REVERT the changes a run made. REVERT means ``git reset --hard`` — a
DESTRUCTIVE operation. This module keeps that decision small, pure and testable.

SAFETY — three guarantees, mirroring src/orchestrator/phase_tracker.py:

1. Kill-switch default-OFF. ``ODYSSEUS_AUTOEVAL`` gates the destructive path.
   When OFF (the default), ``apply_autoeval`` forces the decision to "keep",
   NEVER touches git, and behaviour is byte-identical to today.
2. The git runner is INJECTED. ``apply_autoeval`` only ever reverts by calling
   the ``git_runner`` callable passed in by the caller. Tests pass a fake, so a
   real ``git reset --hard`` is never executed against the working tree.
3. Best-effort, never raises. Any error from the injected runner is swallowed
   and reported on the returned ``AutoevalDecision`` — AUTOEVAL must never crash
   the agent loop.

The policy is intentionally conservative: we only REVERT on a *clear* failure
signal — a non-empty verifier verdict (from ``_run_verifier_subagent``, which
returns a list of failure reasons; empty == SUCCESS) or ``DriftLevel.HIGH``
(harness/protocol file touched). MEDIUM/LOW drift KEEP.
"""
from __future__ import annotations

import logging
import os
from dataclasses import dataclass
from typing import Callable, Optional, Sequence

logger = logging.getLogger(__name__)

# Literal decision values kept as plain strings to match the task contract.
KEEP = "keep"
REVERT = "revert"


def autoeval_enabled() -> bool:
    """OFF unless ODYSSEUS_AUTOEVAL is set to a truthy value.

    Mirrors src/orchestrator/phase_tracker.py::tracker_enabled so the whole
    destructive-revert path stays behind one default-OFF kill-switch.
    """
    val = os.getenv("ODYSSEUS_AUTOEVAL", "off").strip().lower()
    return val in {"on", "1", "true", "yes"}


def decide_keep_or_revert(
    verifier_reasons: Optional[Sequence[str]],
    drift_level=None,
) -> str:
    """Pure decision: return "keep" or "revert".

    Conservative policy — REVERT only on a clear failure signal:
      * ``verifier_reasons`` is non-empty (verifier returned a FAIL verdict), or
      * ``drift_level`` is ``DriftLevel.HIGH`` (harness/protocol touched).
    Everything else (empty/None reasons, MEDIUM/LOW/None drift) KEEPS.

    This function has NO side effects and never inspects the environment — the
    kill-switch is enforced in ``apply_autoeval``, not here.
    """
    if verifier_reasons:
        return REVERT

    # Compare by value so we don't hard-depend on the Observer import here and
    # so a plain "high" string works too.
    level_val = getattr(drift_level, "value", drift_level)
    if level_val == "high":
        return REVERT

    return KEEP


@dataclass
class AutoevalDecision:
    """Outcome of an AUTOEVAL step.

    enabled  — was the kill-switch ON for this call
    decision — "keep" or "revert" (forced to "keep" when disabled)
    reverted — did the injected git_runner actually perform the revert
    error    — string describing why a revert failed (or None)
    """
    enabled: bool
    decision: str
    reverted: bool
    error: Optional[str] = None


def apply_autoeval(
    verifier_reasons: Optional[Sequence[str]],
    *,
    drift_level=None,
    git_runner: Optional[Callable[[], object]] = None,
) -> AutoevalDecision:
    """Decide keep/revert and, only when enabled AND decision == revert, invoke
    the INJECTED ``git_runner`` to perform the destructive revert.

    Guarantees:
      * When the kill-switch is OFF, returns decision="keep", never calls
        ``git_runner`` — byte-identical to today.
      * The destructive ``git reset --hard`` is never done here directly; it is
        delegated to ``git_runner`` (tests inject a fake).
      * Best-effort: any exception from ``git_runner`` is caught; the function
        never raises.
    """
    enabled = autoeval_enabled()
    if not enabled:
        # Default path: do nothing destructive, behaviour unchanged.
        return AutoevalDecision(enabled=False, decision=KEEP, reverted=False)

    decision = decide_keep_or_revert(verifier_reasons, drift_level=drift_level)
    if decision != REVERT:
        return AutoevalDecision(enabled=True, decision=KEEP, reverted=False)

    # decision == REVERT and enabled: run the injected destructive path.
    if git_runner is None:
        logger.warning("[autoeval] revert decided but no git_runner injected — skipping")
        return AutoevalDecision(
            enabled=True, decision=REVERT, reverted=False,
            error="no git_runner injected",
        )

    try:
        result = git_runner()
        reverted = result is not False  # a runner may return True/None on success
        if reverted:
            logger.warning(
                "[autoeval] REVERT — verifier/drift flagged failure; git reset --hard performed"
            )
        else:
            logger.error("[autoeval] revert requested but git_runner reported failure")
        return AutoevalDecision(
            enabled=True, decision=REVERT, reverted=bool(reverted),
            error=None if reverted else "git_runner reported failure",
        )
    except Exception as e:  # best-effort: never let AUTOEVAL crash the loop
        logger.error("[autoeval] revert git_runner raised — swallowed: %s", e)
        return AutoevalDecision(
            enabled=True, decision=REVERT, reverted=False, error=str(e),
        )
