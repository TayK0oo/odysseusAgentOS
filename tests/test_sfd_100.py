"""
Tests for Conversation Search, Multi-Agent Decision, Context Manager.
"""

import pytest

from src.conversation_search.signals import LinguisticSignalDetector, SignalType, DetectedSignal
from src.conversation_search.search import ConversationSearch, SearchResult
from src.multi_agent_decision import MultiAgentDecisionEngine, TaskProfile, AgentMode
from src.context_manager import ContextManager, ContextBudget, ContextFilter


# ============================================================================
# Linguistic Signal Detection
# ============================================================================

class TestLinguisticSignals:
    def test_possessive_french(self):
        signals = LinguisticSignalDetector.detect("continue mon projet de backup")
        assert len(signals) > 0
        assert any(s.type == SignalType.POSSESSIVE for s in signals)

    def test_definite_reference_french(self):
        signals = LinguisticSignalDetector.detect("le script ne marche plus")
        assert any(s.type == SignalType.DEFINITE_REFERENCE for s in signals)

    def test_past_exchange_french(self):
        signals = LinguisticSignalDetector.detect("tu m'avais recommandé d'utiliser Docker")
        assert any(s.type == SignalType.PAST_EXCHANGE for s in signals)

    def test_direct_request_french(self):
        signals = LinguisticSignalDetector.detect("tu te souviens de ce qu'on a fait hier?")
        assert any(s.type == SignalType.DIRECT_REQUEST for s in signals)

    def test_possessive_english(self):
        signals = LinguisticSignalDetector.detect("continue my project setup")
        assert any(s.type == SignalType.POSSESSIVE for s in signals)

    def test_no_signal(self):
        signals = LinguisticSignalDetector.detect("what is 2+2?")
        assert len(signals) == 0

    def test_should_search(self):
        assert LinguisticSignalDetector.should_search("mon projet") is True
        assert LinguisticSignalDetector.should_search("2+2") is False

    def test_extract_query(self):
        q = LinguisticSignalDetector.extract_query(
            "de quoi on avait discuté des robots chinois hier ?"
        )
        assert "robots" in q.lower()
        assert "chinois" in q.lower()
        assert "discuté" not in q.lower()

    def test_has_direct_request(self):
        assert LinguisticSignalDetector.has_direct_request("tu te souviens du projet?") is True
        assert LinguisticSignalDetector.has_direct_request("explique moi Python") is False


# ============================================================================
# Conversation Search
# ============================================================================

class TestConversationSearch:
    def setup_method(self):
        self.cs = ConversationSearch()
        self.cs.index(SearchResult(
            session_id="s1", project_id="p1",
            snippet="discussed deployment strategy for auth service",
            timestamp="2026-07-20T10:00:00",
            score=0.9,
        ))
        self.cs.index(SearchResult(
            session_id="s2", project_id="p1",
            snippet="decided to use PostgreSQL for the backend",
            timestamp="2026-07-21T14:00:00",
            score=0.7,
        ))
        self.cs.index(SearchResult(
            session_id="s3", project_id="p2",
            snippet="frontend should use React with Tailwind",
            timestamp="2026-07-22T09:00:00",
            score=0.5,
        ))

    def test_conversation_search(self):
        results = self.cs.conversation_search("auth")
        assert len(results) == 1
        assert "auth" in results[0].snippet

    def test_project_isolation(self):
        results = self.cs.conversation_search("frontend", project_id="p1")
        assert len(results) == 0  # belongs to p2

    def test_recent_chats(self):
        results = self.cs.recent_chats(window_days=30)
        assert len(results) == 3

    def test_search_by_session(self):
        results = self.cs.search_by_session("s1")
        assert len(results) == 1
        assert results[0].session_id == "s1"

    def test_count(self):
        assert self.cs.count == 3

    def test_clear_by_project(self):
        removed = self.cs.clear(project_id="p2")
        assert removed == 1
        assert self.cs.count == 2


# ============================================================================
# Multi-Agent Decision
# ============================================================================

class TestMultiAgentDecision:
    def setup_method(self):
        self.engine = MultiAgentDecisionEngine()

    def test_simple_task_stays_single(self):
        task = TaskProfile(
            estimated_output_tokens=200,
            tool_count=5,
        )
        decision = self.engine.evaluate(task)
        assert decision.mode == AgentMode.SINGLE
        assert decision.token_overhead_estimate == 1.0

    def test_context_protection_triggers_multi(self):
        task = TaskProfile(
            estimated_output_tokens=3000,
            irrelevant_token_ratio=0.5,  # 1500 irrelevant tokens
        )
        decision = self.engine.evaluate(task)
        assert decision.mode == AgentMode.MULTI
        assert "context_protection" in decision.criteria_met

    def test_too_many_tools_triggers_multi(self):
        task = TaskProfile(tool_count=20)
        decision = self.engine.evaluate(task)
        assert decision.mode == AgentMode.MULTI
        assert "specialization" in decision.criteria_met

    def test_parallelization_triggers_multi(self):
        task = TaskProfile(
            is_parallelizable=True,
            has_independent_subtasks=True,
        )
        decision = self.engine.evaluate(task)
        assert decision.mode == AgentMode.MULTI
        assert "parallelization" in decision.criteria_met

    def test_conflicting_domains(self):
        task = TaskProfile(
            tool_count=10,
            has_conflicting_instructions=True,
            domain_count=3,
        )
        decision = self.engine.evaluate(task)
        assert decision.mode == AgentMode.MULTI

    def test_overhead_measurement(self):
        multiplier = MultiAgentDecisionEngine.measure_overhead(1000, 8000)
        assert multiplier == 8.0

    def test_recommended_agents(self):
        task = TaskProfile(tool_count=20)
        decision = self.engine.evaluate(task)
        assert "domain-specialist" in decision.recommended_agents


# ============================================================================
# Context Manager
# ============================================================================

class TestContextBudget:
    def test_defaults(self):
        budget = ContextBudget(max_tokens=100000)
        assert budget.max_tokens == 100000
        assert budget.used_tokens == 0
        assert budget.compaction_threshold == 0.80

    def test_should_compact(self):
        budget = ContextBudget(max_tokens=1000, used_tokens=850)
        assert budget.should_compact is True

        budget2 = ContextBudget(max_tokens=1000, used_tokens=500)
        assert budget2.should_compact is False

    def test_remaining(self):
        budget = ContextBudget(max_tokens=1000, used_tokens=400)
        assert budget.remaining == 600


class TestContextFilter:
    def test_phase_filter(self):
        cf = ContextFilter(phase="BUILD")
        entries = [
            {"content": "a", "phase": "BUILD"},
            {"content": "b", "phase": "PLAN"},
        ]
        filtered = cf.filter(entries)
        assert len(filtered) == 1
        assert filtered[0]["content"] == "a"

    def test_quota(self):
        cf = ContextFilter(max_entries=2)
        entries = [{"content": str(i)} for i in range(5)]
        filtered = cf.filter(entries)
        assert len(filtered) == 2


class TestContextManager:
    def test_track_tokens(self):
        cm = ContextManager(max_tokens=10000)
        cm.track_tokens(5000)
        assert cm.budget.used_tokens == 5000

    def test_note_decision(self):
        cm = ContextManager()
        did = cm.note_decision("s1", "use PostgreSQL", {"reason": "ACID"})
        assert did is not None
        decisions = cm.get_decisions("s1")
        assert len(decisions) == 1
        assert "PostgreSQL" in decisions[0]["decision"]

    def test_thread_tracking(self):
        cm = ContextManager()
        cm.note_decision("s1", "decision A")
        cm.note_decision("s1", "decision B")
        cm.note_decision("s2", "decision C")
        assert len(cm.get_thread("s1")) == 2
        assert len(cm.get_thread("s2")) == 1

    def test_build_context(self):
        cm = ContextManager()
        cm.track_tokens(30000)
        ctx = cm.build_context(
            task="build auth",
            phase="BUILD",
            memory_entries=[{"content": "previous auth attempt used JWT"}],
            preferences={"format": "bullet"},
        )
        assert ctx["task"] == "build auth"
        assert ctx["phase"] == "BUILD"
        assert ctx["tokens_used"] == 30000
        assert len(ctx["memory"]) == 1

    def test_compaction_increments_count(self):
        cm = ContextManager()
        import asyncio

        async def run():
            result = await cm.compact([{"role": "user", "content": "old msg"}] * 10)
            return result

        result = asyncio.run(run())
        assert cm.budget.compaction_count == 1
