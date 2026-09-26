"""SFD §5.27 — P19: Memory Impact Verification.

Before storing a memory fact, compare the agent's response with and without
the fact. Facts that don't change the response ("earn their place") are
flagged as low-impact and may be discarded to keep the memory store lean.

Gated behind ODYSSEUS_MEMORY_IMPACT kill-switch (default OFF).
"""

from __future__ import annotations

import hashlib
import logging
import os
from dataclasses import dataclass

logger = logging.getLogger(__name__)

# ─── Kill-switch ────────────────────────────────────────────────────────


def memory_impact_enabled() -> bool:
    val = os.getenv("ODYSSEUS_MEMORY_IMPACT", "on").strip().lower()
    return val in {"on", "1", "true", "yes"}


# ─── Constants ──────────────────────────────────────────────────────────

DEFAULT_IMPACT_THRESHOLD = 0.15


@dataclass
class ImpactResult:
    fact: str
    impact_score: float
    threshold: float
    should_store: bool
    reason: str = ""


class MemoryImpactVerifier:
    def __init__(self, threshold: float = DEFAULT_IMPACT_THRESHOLD):
        self.threshold = threshold

    def evaluate_impact(
        self,
        fact: str,
        current_response: str,
        hypothetical_response: str,
    ) -> float:
        if not current_response or not hypothetical_response:
            return 0.0

        current_norm = _normalize(current_response)
        hypo_norm = _normalize(hypothetical_response)

        if current_norm == hypo_norm:
            return 0.0

        current_hash = _token_set_hash(current_norm)
        hypo_hash = _token_set_hash(hypo_norm)

        if not current_hash or not hypo_hash:
            return 0.0

        intersection = current_hash & hypo_hash
        union = current_hash | hypo_hash
        if not union:
            return 0.0

        similarity = len(intersection) / len(union)
        impact = 1.0 - similarity
        return round(impact, 4)

    def should_store(self, fact: str, impact_score: float) -> bool:
        return impact_score > self.threshold

    def verify(
        self,
        fact: str,
        current_response: str,
        hypothetical_response: str,
    ) -> ImpactResult:
        impact = self.evaluate_impact(fact, current_response, hypothetical_response)
        store = self.should_store(fact, impact)

        reason = ""
        if not store:
            if impact == 0.0:
                reason = "no_impact"
            else:
                reason = f"below_threshold ({impact:.4f} <= {self.threshold})"
        else:
            reason = f"above_threshold ({impact:.4f} > {self.threshold})"

        result = ImpactResult(
            fact=fact,
            impact_score=impact,
            threshold=self.threshold,
            should_store=store,
            reason=reason,
        )

        if store:
            logger.debug("[p19] memory impact: storing fact (impact=%.4f)", impact)
        else:
            logger.debug("[p19] memory impact: discarding fact (impact=%.4f, %s)", impact, reason)

        return result


def _normalize(text: str) -> str:
    return " ".join(text.lower().split())


def _token_set_hash(text: str, n: int = 3) -> set[int] | None:
    tokens = text.split()
    if len(tokens) < n:
        return None
    result: set[int] = set()
    for i in range(len(tokens) - n + 1):
        ngram = " ".join(tokens[i : i + n])
        h = int(hashlib.sha256(ngram.encode()).hexdigest()[:16], 16)
        result.add(h)
    return result


_verifier: MemoryImpactVerifier | None = None


def get_memory_impact_verifier(threshold: float = DEFAULT_IMPACT_THRESHOLD) -> MemoryImpactVerifier:
    global _verifier
    if _verifier is None:
        _verifier = MemoryImpactVerifier(threshold=threshold)
    return _verifier
