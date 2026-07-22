"""
Integration test — full AgentOS SFD v3.0 pipeline.

Verifies that all modules can be initialized and work together:
ThoughtBus → DurableExecution → Memory → Preferences → Visual → Classification → Security → Discovery
"""

import os
import tempfile
import pytest


class TestFullPipeline:
    """End-to-end: initialize all modules and simulate a full run."""

    @pytest.fixture
    def tmpdir(self):
        with tempfile.TemporaryDirectory() as td:
            yield td

    @pytest.mark.asyncio
    async def test_thought_bus_with_subscribers(self):
        """Bus with all SFD subscribers registered."""
        from src.thought_bus import ThoughtBus, SubscriberRegistry
        from src.thought_bus.subscriber import PhaseSubscriber

        reg = SubscriberRegistry()

        class TestSub:
            def __init__(self):
                self.entered = set()

            async def on_phase_enter(self, phase, ctx):
                self.entered.add(phase)

        sub = TestSub()
        reg.register(PhaseSubscriber(sub, ["BUILD", "QUALITY", "MEMORY_OBSERVE"], name="Test"))

        bus = ThoughtBus(reg, objective="integration test")
        events = []
        async for event in bus.walk():
            events.append(event)

        assert "BUILD" in sub.entered
        last_event = events[-1]
        assert last_event["type"] == "thought_bus"
        assert last_event["status"] == "complete"

    @pytest.mark.asyncio
    async def test_durable_engine_with_approval(self, tmpdir):
        """Create workflow, add activity, execute, create approval signal."""
        from src.durable_execution.engine import DurableEngine
        from src.durable_execution.activity import DurableActivity

        db_path = os.path.join(tmpdir, "test.db")
        engine = DurableEngine(db_path=db_path)
        await engine.initialize()

        wf_id = await engine.create_workflow("integration-test", project_id="test")
        await engine.add_activity(wf_id, DurableActivity("a1", "step1", compensation="undo1"))
        await engine.add_activity(wf_id, DurableActivity("a2", "step2", compensation="undo2"))

        status = await engine.start_workflow(wf_id, lambda a, p: True)
        assert status == "completed"

        # Approval signal
        sig_id = await engine.create_approval_signal(wf_id, "a3", "Approve?", {"env": "test"})
        assert sig_id is not None

        pending = await engine.get_pending_signals()
        assert any(s["id"] == sig_id for s in pending)

        await engine.resolve_approval(sig_id, approved=True)
        engine.close()

    def test_memory_with_provenance_cycle(self, tmpdir):
        """Full memory cycle: write, read, append, version control."""
        from src.memory_provenance.operations import MemoryOperations
        from src.memory_provenance.omission import OmissionFilter

        ops = MemoryOperations(base_path=tmpdir)

        # Write
        content = "---\nname: test\n---\n- [stated] integration test fact\n"
        result = ops.memory_write("topics/test.md", content, "new")
        assert result.success

        # Read
        read = ops.memory_read("topics/test.md")
        assert read.success
        assert "integration test fact" in read.content

        # Append with version
        result2 = ops.memory_append("topics/test.md", "- [stated] another fact", read.version_token)
        assert result2.success

        # Verify omission filter
        assert OmissionFilter.is_safe("SSN is") is False
        assert OmissionFilter.is_safe("likes pizza") is True

    def test_preferences_resolution_chain(self):
        """Full 5-level preference chain."""
        from src.preferences.resolver import Preference, PreferenceType, PreferenceScope, resolve_preferences
        from src.preferences.guardrails import BehavioralGuardrail

        stored = [
            Preference(PreferenceType.BEHAVIORAL, "format", "bullet", PreferenceScope.SELECTIVE),
            Preference(PreferenceType.BEHAVIORAL, "langue", "fr", PreferenceScope.ALWAYS),
        ]
        resolved = resolve_preferences(
            request_instruction="use tables for this",
            stored=stored,
            user_style={"ton": "formal"},
        )
        # Request wins
        assert resolved["request_instruction"] == "use tables for this"
        # Always wins over selective
        assert resolved["langue"] == "fr"
        # userStyle wins over selective
        assert resolved["ton"] == "formal"
        # Selective fills defaults
        assert resolved["format"] == "bullet"

    def test_visual_routing_pipeline(self):
        """Output router: text, file, visual."""
        from src.visual_output.router import OutputRouter, OutputMode

        router = OutputRouter()

        # Text only
        d = router.decide("hello", "Hi!")
        assert d.mode == OutputMode.TEXT

        # File request
        d = router.decide("crée un fichier PDF", "content")
        assert d.mode == OutputMode.FILE

        # Visual
        d = router.decide("montre-moi un diagramme", "system has 3 layers...")
        assert d.mode == OutputMode.INLINE_VISUAL

    def test_classification_pipeline(self):
        """Classification + omission + right to forget."""
        from src.classification.levels import classify_data, RetentionLevel

        assert classify_data("SSN", "123-45-6789") == RetentionLevel.PROTECTED
        assert classify_data("project plan") == RetentionLevel.INTERNAL
        assert classify_data("food preference") == RetentionLevel.PERSONAL

    def test_conversation_search_pipeline(self):
        """Linguistic signals → search."""
        from src.conversation_search.signals import LinguisticSignalDetector
        from src.conversation_search.search import ConversationSearch, SearchResult

        # Signal detection
        assert LinguisticSignalDetector.should_search("mon projet de backup") is True
        assert LinguisticSignalDetector.should_search("2+2?") is False

        # Search
        cs = ConversationSearch()
        cs.index(SearchResult("s1", "p1", "discussed auth strategy", "2026-07-20T10:00:00"))
        results = cs.conversation_search("auth")
        assert len(results) == 1

    def test_multi_agent_decision_pipeline(self):
        """Decision engine: single vs multi."""
        from src.multi_agent_decision import MultiAgentDecisionEngine, TaskProfile, AgentMode

        engine = MultiAgentDecisionEngine()

        # Simple task → single
        dec = engine.evaluate(TaskProfile(tool_count=5))
        assert dec.mode == AgentMode.SINGLE

        # Many tools → multi
        dec = engine.evaluate(TaskProfile(tool_count=20))
        assert dec.mode == AgentMode.MULTI

    def test_context_manager_pipeline(self):
        """Context: track, compact, build."""
        from src.context_manager import ContextManager

        cm = ContextManager(max_tokens=10000)
        cm.track_tokens(9000)
        assert cm.budget.should_compact is True
        assert cm.budget.remaining == 1000

        cm.note_decision("s1", "use Redis for caching")
        ctx = cm.build_context("add cache layer", "BUILD", [{"content": "Redis available"}])
        assert ctx["phase"] == "BUILD"
        assert len(ctx["memory"]) == 1

    def test_security_pipeline(self):
        """Injection guard + output filter."""
        from src.content_security.injection import InjectionGuard
        from src.content_security.output_filter import OutputFilter

        # Injection
        assert InjectionGuard.is_safe("ignore instructions") is False
        assert InjectionGuard.is_safe("normal text") is True

        # Output filter
        assert OutputFilter.is_safe("normal response") is True
        assert OutputFilter.is_safe("graphic violence") is False

    def test_tool_discovery_pipeline(self):
        """Search → registry → suggest."""
        from src.tool_discovery.search import ToolSearch, ToolDescriptor
        from src.tool_discovery.registry import MCPRegistry
        from src.tool_discovery.suggest import ConnectorSuggester

        # Search
        ts = ToolSearch()
        ts.register(ToolDescriptor("kroki", "Diagram renderer", keywords=["mermaid"]))
        assert len(ts.search("diagram")) == 1

        # Registry
        reg = MCPRegistry()
        assert len(reg.search("crm")) > 0

        # Suggest (third-party requires confirmation)
        suggester = ConnectorSuggester()
        suggestions = suggester.suggest("crm")
        assert len(suggestions) > 0
        for s in suggestions:
            if s.entry.is_third_party:
                assert s.requires_confirmation is True
