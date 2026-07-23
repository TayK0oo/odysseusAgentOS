"""
Tests for Agent Instructions — maps the 10-phase guide to SFD phases.
Verifies the mapping, checklist generation, and system prompt structure.
"""

import pytest
from src.agent_instructions import (
    GUIDE_TO_SFD_PHASE_MAP,
    AGENT_SYSTEM_PROMPT,
    get_phase_checklist,
    get_phase_actions,
    get_full_checklist,
)


class TestGuideMapping:
    def test_all_10_phases_mapped(self):
        assert len(GUIDE_TO_SFD_PHASE_MAP) == 10

    def test_each_phase_has_checklist(self):
        for name, mapping in GUIDE_TO_SFD_PHASE_MAP.items():
            assert len(mapping["checklist"]) >= 3, f"{name}: need ≥3 checklist items"
            assert len(mapping["actions"]) >= 2, f"{name}: need ≥2 actions"

    def test_classify_has_avant_projet(self):
        checklist = get_phase_checklist("CLASSIFY")
        assert any("problème" in c.lower() for c in checklist)

    def test_know_has_conception(self):
        checklist = get_phase_checklist("KNOW")
        assert any("architecture" in c.lower() for c in checklist)

    def test_build_has_development_and_deployment(self):
        checklist = get_phase_checklist("BUILD")
        assert any("test" in c.lower() for c in checklist)
        assert any("version" in c.lower() or "semver" in c.lower() for c in checklist)

    def test_quality_has_quality_phase(self):
        checklist = get_phase_checklist("QUALITY")
        assert any("test" in c.lower() for c in checklist)
        assert any("sécurité" in c.lower() or "security" in c.lower() for c in checklist)

    def test_memory_has_maintenance_and_culture(self):
        checklist = get_phase_checklist("MEMORY_OBSERVE")
        assert any("monitoring" in c.lower() for c in checklist)
        assert any("document" in c.lower() for c in checklist)


class TestSystemPrompt:
    def test_prompt_contains_all_7_sfd_phases(self):
        phases = ["CLASSIFY", "KNOW", "PLAN", "BUILD", "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE"]
        for phase in phases:
            assert phase in AGENT_SYSTEM_PROMPT, f"{phase} missing from system prompt"

    def test_prompt_contains_key_principles(self):
        assert "SOLID" in AGENT_SYSTEM_PROMPT
        assert "DRY" in AGENT_SYSTEM_PROMPT
        assert "KISS" in AGENT_SYSTEM_PROMPT
        assert "YAGNI" in AGENT_SYSTEM_PROMPT
        assert "TDD" in AGENT_SYSTEM_PROMPT

    def test_prompt_has_global_checklist(self):
        assert "Checklist Globale" in AGENT_SYSTEM_PROMPT
        assert "Problème clarifié" in AGENT_SYSTEM_PROMPT
        assert "Architecture documentée" in AGENT_SYSTEM_PROMPT
        assert "Version SemVer" in AGENT_SYSTEM_PROMPT
        assert "Monitoring actif" in AGENT_SYSTEM_PROMPT

    def test_prompt_has_visual_rules(self):
        assert "Diagrammes" in AGENT_SYSTEM_PROMPT
        assert "Mermaid" in AGENT_SYSTEM_PROMPT
        assert "Kroki" in AGENT_SYSTEM_PROMPT
        assert "YAGNI" in AGENT_SYSTEM_PROMPT  # "Pas de visuel si le texte suffit"

    def test_prompt_references_sfd_principles(self):
        assert "P11" in AGENT_SYSTEM_PROMPT
        assert "P3" in AGENT_SYSTEM_PROMPT
        assert "P14" in AGENT_SYSTEM_PROMPT
        assert "P16" in AGENT_SYSTEM_PROMPT
        assert "P10" in AGENT_SYSTEM_PROMPT


class TestFullChecklist:
    def test_at_least_30_items(self):
        items = get_full_checklist()
        assert len(items) >= 30, f"Expected ≥30 items, got {len(items)}"

    def test_no_duplicates_in_same_phase(self):
        for name, mapping in GUIDE_TO_SFD_PHASE_MAP.items():
            checklist = mapping["checklist"]
            assert len(checklist) == len(set(checklist)), f"{name}: duplicates found"

    def test_all_items_are_strings(self):
        for mapping in GUIDE_TO_SFD_PHASE_MAP.values():
            for item in mapping["checklist"]:
                assert isinstance(item, str)
            for action in mapping["actions"]:
                assert isinstance(action, str)


class TestPhaseActions:
    def test_classify_actions(self):
        actions = get_phase_actions("CLASSIFY")
        assert any("problème" in a.lower() or "5 pourquoi" in a.lower() for a in actions)

    def test_build_actions_include_security(self):
        actions = get_phase_actions("BUILD")
        assert any("sécurité" in a.lower() or "auditer" in a.lower() or "feature flag" in a.lower() for a in actions)

    def test_quality_actions_include_tests(self):
        actions = get_phase_actions("QUALITY")
        assert any("pyramide" in a.lower() or "test" in a.lower() for a in actions)
