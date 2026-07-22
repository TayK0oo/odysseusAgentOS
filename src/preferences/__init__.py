"""
User Preferences System.

SFD §5.15: behavioral + contextual preferences with priority resolution.
Resolution order: request > stored "always" > userStyle > stored selective > default.

Guardrails: never persist flattery, disagreement suppression,
emotional dependency, evaluation abandonment.
"""

from src.preferences.engine import PreferenceEngine
from src.preferences.resolver import resolve_preferences, Preference
from src.preferences.guardrails import BehavioralGuardrail

__all__ = [
    "PreferenceEngine",
    "resolve_preferences",
    "Preference",
    "BehavioralGuardrail",
]
