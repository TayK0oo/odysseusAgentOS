"""
Preference data model and resolution engine.

Priority order (SFD §5.15.3):
    1. Request instruction (highest)
    2. Stored "always" preference
    3. Configured userStyle
    4. Stored selective preference
    5. System default (lowest)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class PreferenceType(str, Enum):
    BEHAVIORAL = "behavioral"
    CONTEXTUAL = "contextual"


class PreferenceScope(str, Enum):
    ALWAYS = "always"
    SELECTIVE = "selective"


@dataclass
class Preference:
    """A single preference rule."""
    type: PreferenceType
    subtype: str  # format, ton, langue, expertise, background, interets, role
    rule: str
    scope: PreferenceScope = PreferenceScope.SELECTIVE
    created_at: str = ""

    def to_dict(self) -> dict:
        return {
            "type": self.type.value,
            "subtype": self.subtype,
            "rule": self.rule,
            "scope": self.scope.value,
        }


def resolve_preferences(
    request_instruction: Optional[str] = None,
    stored: Optional[list[Preference]] = None,
    user_style: Optional[dict[str, str]] = None,
) -> dict[str, str]:
    """Resolve preferences by descending priority.

    Returns a dict of {subtype: rule} with the winning rule.
    """
    resolved: dict[str, str] = {}

    # Level 5: System defaults
    defaults = {
        "format": "markdown",
        "ton": "neutral",
        "langue": "en",
    }
    for subtype, rule in defaults.items():
        resolved[subtype] = rule

    # Level 4: Stored selective preferences
    if stored:
        for pref in stored:
            if pref.scope == PreferenceScope.SELECTIVE:
                resolved[pref.subtype] = pref.rule

    # Level 3: Configured userStyle
    if user_style:
        for subtype, rule in user_style.items():
            resolved[subtype] = rule

    # Level 2: Stored "always" preferences
    if stored:
        for pref in stored:
            if pref.scope == PreferenceScope.ALWAYS:
                resolved[pref.subtype] = pref.rule

    # Level 1: Request instruction (always wins)
    if request_instruction:
        resolved["request_instruction"] = request_instruction

    return resolved


class PreferenceEngine:
    """Manages preference storage, resolution, and guardrail filtering.

    Reads from /preferences.md in the memory system.
    Writes are filtered through BehavioralGuardrail.
    """

    def __init__(self, memory_ops=None):
        self._memory_ops = memory_ops
        self._cache: Optional[list[Preference]] = None

    def load(self, raw_yaml: str) -> list[Preference]:
        """Parse preferences from raw YAML content."""
        import yaml
        preferences = []
        try:
            data = yaml.safe_load(raw_yaml)
            if not data or "preferences" not in data:
                return preferences
            for cat in data.get("preferences", {}).get("behavioral", []):
                preferences.append(Preference(
                    type=PreferenceType.BEHAVIORAL,
                    subtype=cat.get("type", ""),
                    rule=cat.get("rule", ""),
                    scope=PreferenceScope(cat.get("scope", "selective")),
                ))
            for cat in data.get("preferences", {}).get("contextual", []):
                preferences.append(Preference(
                    type=PreferenceType.CONTEXTUAL,
                    subtype=cat.get("type", ""),
                    rule=cat.get("rule", ""),
                    scope=PreferenceScope(cat.get("scope", "selective")),
                ))
        except Exception:
            pass
        return preferences

    def resolve(
        self,
        stored: list[Preference],
        request_instruction: Optional[str] = None,
        user_style: Optional[dict[str, str]] = None,
    ) -> dict[str, str]:
        """Resolve active preferences for the current request."""
        return resolve_preferences(request_instruction, stored, user_style)
