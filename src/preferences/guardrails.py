"""
Behavioral guardrails — preferences that must NEVER be persisted.

SFD §5.15.4: flattery, disagreement suppression, emotional dependency,
evaluation abandonment, permission escalation, rule ignorance.

These are treated as absent even if they appear in the preferences file.
"""

from typing import Optional

# Phrases that indicate a dangerous preference
_GUARDRAIL_PATTERNS = [
    "always agree",
    "never disagree",
    "don't question",
    "unconditional flattery",
    "suppress disagreement",
    "emotional dependency",
    "treat me as",
    "i am your",
    "you are my",
    "don't evaluate",
    "don't assess",
    "ignore rules",
    "bypass security",
    "grant all permissions",
    "administrator access",
    "never say no",
    "always praise",
    "be my friend",
    "love me",
    "care about me",
]


class BehavioralGuardrail:
    """Filters out dangerous preferences before persistence.

    These are NEVER written to /preferences.md.
    If found in existing preferences, they are treated as absent.
    """

    @staticmethod
    def is_safe(rule: str) -> bool:
        """Check if a preference rule is safe to persist."""
        rule_lower = rule.lower()
        for pattern in _GUARDRAIL_PATTERNS:
            if pattern in rule_lower:
                return False
        return True

    @staticmethod
    def filter_preferences(preferences: list) -> list:
        """Remove unsafe preferences. Returns safe subset."""
        from src.preferences.resolver import Preference
        return [p for p in preferences if BehavioralGuardrail.is_safe(p.rule)]

    @staticmethod
    def validate_rule(rule: str) -> tuple[bool, Optional[str]]:
        """Validate a preference rule. Returns (is_safe, reason)."""
        if not BehavioralGuardrail.is_safe(rule):
            return False, "Rule violates behavioral guardrails — not persisted."
        return True, None
