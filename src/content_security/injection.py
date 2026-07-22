"""
Prompt injection guard for memory files.

Memory files can contain malicious instructions. The InjectionGuard
detects and neutralizes them before they influence agent behavior.

SFD §5.20.1: ignore suspicious data, refuse instructions contradicting
invariant principles, never let behavior drift.
"""

from __future__ import annotations

import re
from typing import Optional

# Patterns that indicate injected instructions in memory content
_INJECTION_PATTERNS = [
    re.compile(r"(?i)ignore\b.*\b(instructions|rules|constraints|guidelines)", re.IGNORECASE),
    re.compile(r"(?i)you are now", re.IGNORECASE),
    re.compile(r"(?i)forget everything", re.IGNORECASE),
    re.compile(r"(?i)new system prompt", re.IGNORECASE),
    re.compile(r"(?i)bypass (security|permission|gate|approval)", re.IGNORECASE),
    re.compile(r"(?i)grant .* access", re.IGNORECASE),
    re.compile(r"(?i)administrator (privilege|right)", re.IGNORECASE),
    re.compile(r"(?i)do not (evaluate|assess|verify|question)", re.IGNORECASE),
    re.compile(r"(?i)always (agree|approve|accept|allow)", re.IGNORECASE),
    re.compile(r"(?i)never (reject|deny|refuse|disagree)", re.IGNORECASE),
    re.compile(r"(?i)override (security|safety|rules)", re.IGNORECASE),
]

# Phrases that contradict invariant SFD principles
_CONTRADICTIONS = [
    "don't ask for approval",
    "skip validation",
    "no budget limit",
    "unlimited execution",
    "ignore phase lock",
]


class InjectionGuard:
    """Detects and neutralizes prompt injections in memory content.

    When suspicious content is found, it's flagged for omission.
    The system never lets memory-sourced instructions override
    its invariant principles (SFD P1-P22).
    """

    @staticmethod
    def scan(content: str) -> Optional[str]:
        """Scan content for injection attempts.

        Returns the matched pattern if found, None if safe.
        """
        for pattern in _INJECTION_PATTERNS:
            match = pattern.search(content)
            if match:
                return f"Injection detected: {match.group(0)}"
        return None

    @staticmethod
    def has_contradiction(content: str) -> bool:
        """Check if content contradicts invariant principles."""
        content_lower = content.lower()
        return any(c in content_lower for c in _CONTRADICTIONS)

    @staticmethod
    def sanitize(entries: list[str]) -> tuple[list[str], list[str]]:
        """Filter memory entries, returning (safe, flagged).

        Flagged entries are NOT included in the context given to agents.
        """
        safe = []
        flagged = []
        for entry in entries:
            threat = InjectionGuard.scan(entry)
            if threat or InjectionGuard.has_contradiction(entry):
                flagged.append(entry)
            else:
                safe.append(entry)
        return safe, flagged

    @staticmethod
    def is_safe(content: str) -> bool:
        """Convenience: True if content passes all checks."""
        return InjectionGuard.scan(content) is None and not InjectionGuard.has_contradiction(content)
