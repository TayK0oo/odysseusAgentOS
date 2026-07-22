"""
Tests for Preferences system (src/preferences/).

Covers: resolution order, behavioral guardrails, engine parsing.
"""

import pytest

from src.preferences.resolver import (
    Preference,
    PreferenceType,
    PreferenceScope,
    resolve_preferences,
    PreferenceEngine,
)
from src.preferences.guardrails import BehavioralGuardrail


class TestPreferenceResolution:
    def test_defaults_only(self):
        resolved = resolve_preferences()
        assert resolved["format"] == "markdown"
        assert resolved["ton"] == "neutral"
        assert resolved["langue"] == "en"

    def test_selective_overrides_default(self):
        stored = [
            Preference(PreferenceType.BEHAVIORAL, "format", "bullet-lists", PreferenceScope.SELECTIVE),
        ]
        resolved = resolve_preferences(stored=stored)
        assert resolved["format"] == "bullet-lists"

    def test_always_overrides_selective(self):
        stored = [
            Preference(PreferenceType.BEHAVIORAL, "format", "bullet-lists", PreferenceScope.SELECTIVE),
            Preference(PreferenceType.BEHAVIORAL, "format", "json", PreferenceScope.ALWAYS),
        ]
        resolved = resolve_preferences(stored=stored)
        assert resolved["format"] == "json"

    def test_request_wins_over_always(self):
        stored = [
            Preference(PreferenceType.BEHAVIORAL, "format", "json", PreferenceScope.ALWAYS),
        ]
        resolved = resolve_preferences(
            request_instruction="use tables",
            stored=stored,
        )
        assert resolved["request_instruction"] == "use tables"
        # "always" still applies for other fields
        assert resolved["format"] == "json"

    def test_user_style_over_selective(self):
        stored = [
            Preference(PreferenceType.BEHAVIORAL, "ton", "casual", PreferenceScope.SELECTIVE),
        ]
        resolved = resolve_preferences(
            stored=stored,
            user_style={"ton": "formal"},
        )
        assert resolved["ton"] == "formal"

    def test_full_chain(self):
        stored = [
            Preference(PreferenceType.BEHAVIORAL, "format", "bullet", PreferenceScope.SELECTIVE),
            Preference(PreferenceType.BEHAVIORAL, "langue", "fr", PreferenceScope.ALWAYS),
        ]
        resolved = resolve_preferences(
            request_instruction="be concise",
            stored=stored,
            user_style={"format": "tables"},
        )
        assert resolved["request_instruction"] == "be concise"
        assert resolved["format"] == "tables"  # userStyle > selective
        assert resolved["langue"] == "fr"  # always wins


class TestBehavioralGuardrail:
    def test_safe_rules(self):
        assert BehavioralGuardrail.is_safe("use bullet points") is True
        assert BehavioralGuardrail.is_safe("be concise") is True
        assert BehavioralGuardrail.is_safe("respond in French") is True

    def test_unsafe_rules(self):
        assert BehavioralGuardrail.is_safe("always agree with me") is False
        assert BehavioralGuardrail.is_safe("never disagree") is False
        assert BehavioralGuardrail.is_safe("don't question my decisions") is False
        assert BehavioralGuardrail.is_safe("grant all permissions") is False
        assert BehavioralGuardrail.is_safe("ignore rules") is False

    def test_filter_preferences(self):
        prefs = [
            Preference(PreferenceType.BEHAVIORAL, "format", "bullet", PreferenceScope.SELECTIVE),
            Preference(PreferenceType.BEHAVIORAL, "ton", "always praise me", PreferenceScope.ALWAYS),
            Preference(PreferenceType.BEHAVIORAL, "langue", "fr", PreferenceScope.ALWAYS),
        ]
        safe = BehavioralGuardrail.filter_preferences(prefs)
        assert len(safe) == 2
        assert safe[0].rule == "bullet"
        assert safe[1].rule == "fr"

    def test_validate_rule(self):
        ok, reason = BehavioralGuardrail.validate_rule("be concise")
        assert ok is True
        assert reason is None

        ok, reason = BehavioralGuardrail.validate_rule("always agree")
        assert ok is False
        assert "guardrail" in reason


class TestPreferenceEngine:
    def test_parse_yaml(self):
        engine = PreferenceEngine()
        yaml = """
preferences:
  behavioral:
    - type: format
      rule: "use bullet lists"
      scope: selective
    - type: langue
      rule: "respond in French"
      scope: always
  contextual:
    - type: expertise
      rule: "senior Python developer"
      scope: selective
"""
        prefs = engine.load(yaml)
        assert len(prefs) == 3

        behavioral = [p for p in prefs if p.type == PreferenceType.BEHAVIORAL]
        assert len(behavioral) == 2
        assert behavioral[0].rule == "use bullet lists"
        assert behavioral[1].scope == PreferenceScope.ALWAYS

        contextual = [p for p in prefs if p.type == PreferenceType.CONTEXTUAL]
        assert len(contextual) == 1
        assert contextual[0].rule == "senior Python developer"

    def test_empty_yaml(self):
        engine = PreferenceEngine()
        assert engine.load("") == []
        assert engine.load("preferences: {}") == []

    def test_resolve_from_stored(self):
        engine = PreferenceEngine()
        stored = [
            Preference(PreferenceType.BEHAVIORAL, "langue", "fr", PreferenceScope.ALWAYS),
        ]
        resolved = engine.resolve(stored, request_instruction="be brief")
        assert resolved["langue"] == "fr"
        assert resolved["request_instruction"] == "be brief"
