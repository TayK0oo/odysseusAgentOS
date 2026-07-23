"""
Tests for Agent/Chat Mode Detector.
"""

import pytest
from src.mode_detector import detect_mode, is_agent_mode, InteractionMode


class TestModeDetector:
    def test_project_triggers_agent(self):
        assert detect_mode("build a backup system") == InteractionMode.AGENT
        assert detect_mode("create a new API endpoint") == InteractionMode.AGENT
        assert detect_mode("implement user authentication") == InteractionMode.AGENT
        assert detect_mode("deploy to production") == InteractionMode.AGENT
        assert detect_mode("refactor the database module") == InteractionMode.AGENT
        assert detect_mode("fix the login bug") == InteractionMode.AGENT
        assert detect_mode("design the system architecture") == InteractionMode.AGENT
        assert detect_mode("test the payment flow") == InteractionMode.AGENT
        assert detect_mode("analyze the performance bottleneck") == InteractionMode.AGENT

    def test_chat_messages_stay_chat(self):
        assert detect_mode("hello") == InteractionMode.CHAT
        assert detect_mode("how are you?") == InteractionMode.CHAT
        assert detect_mode("what is Python?") == InteractionMode.CHAT
        assert detect_mode("thanks!") == InteractionMode.CHAT
        assert detect_mode("explique-moi ce qu'est Docker") == InteractionMode.CHAT

    def test_short_agent_triggers(self):
        assert detect_mode("build it") == InteractionMode.AGENT
        assert detect_mode("create one") == InteractionMode.AGENT
        assert detect_mode("deploy now") == InteractionMode.AGENT

    def test_switch_off_forces_chat(self):
        assert detect_mode("build a backup system", agent_switch_on=False) == InteractionMode.CHAT
        assert detect_mode("deploy to production", agent_switch_on=False) == InteractionMode.CHAT

    def test_french_triggers(self):
        assert detect_mode("crée un système de backup") == InteractionMode.AGENT
        assert detect_mode("déploie l'application") == InteractionMode.AGENT
        assert detect_mode("corrige le bug") == InteractionMode.AGENT

    def test_english_triggers(self):
        assert detect_mode("build me a landing page") == InteractionMode.AGENT
        assert detect_mode("fix the broken pipeline") == InteractionMode.AGENT
        assert detect_mode("optimize database queries") == InteractionMode.AGENT

    def test_is_agent_mode_convenience(self):
        assert is_agent_mode("build a backup system") is True
        assert is_agent_mode("hello") is False
        assert is_agent_mode("build a system", agent_switch_on=False) is False

    def test_edge_cases(self):
        # Empty message
        assert detect_mode("") == InteractionMode.CHAT
        # Very long chat message
        assert detect_mode("can you please tell me about the history of computer science and how it evolved over time") == InteractionMode.CHAT
        # Single word with agent verb
        assert detect_mode("build") == InteractionMode.AGENT
