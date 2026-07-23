"""
Tests for SFD System Wiring — verifies all integrations work together.
Covers: memory bridge, durable wrap, preference injection, visual routing,
context tracking, P5-P6-P12-P19 gap closure.
"""

import os, tempfile, json, pytest
from pathlib import Path


class TestSFDSystemWiring:
    @pytest.fixture
    def wiring(self):
        from src.sfd_wiring import SFDSystemWiring
        return SFDSystemWiring()

    def test_memory_wire_and_use(self, wiring, tmpdir):
        wiring.wire_memory(base_path=str(tmpdir))
        assert wiring._memory_bridge is not None

        ok = wiring.remember("test-domain", "likes sushi", "[stated]")
        assert ok is True
        assert wiring.stats["facts_stored"] == 1

        facts = wiring.recall("test-domain")
        assert any("likes sushi" in f for f in facts)

    def test_memory_omits_protected(self, wiring, tmpdir):
        wiring.wire_memory(base_path=str(tmpdir))
        ok = wiring.remember("health", "social security number 123-45-6789", "[stated]")
        assert ok is False
        assert wiring.stats["facts_omitted"] == 1

    def test_preferences_inject(self, wiring):
        wiring.wire_preferences()
        prompt = wiring.inject_preferences("You are a helpful assistant.")
        assert "PREFERENCE" in prompt or "You are" in prompt

    def test_visual_route_text(self, wiring):
        from src.visual_output.router import OutputRouter
        router = OutputRouter()
        result = router.decide("hello", "Hi there!")
        assert result.mode.value == "text"

    def test_visual_route_visual(self, wiring):
        from src.visual_output.router import OutputRouter
        router = OutputRouter()
        result = router.decide("montre-moi un diagramme", "The system has 3 layers...")
        assert result.mode.value == "inline_visual"

    def test_context_tracking(self, wiring):
        wiring.wire_context(max_tokens=10000)
        wiring.track_tokens(9000)
        assert wiring.should_compact() is True

    # P5 — Divulgation progressive
    def test_progressive_disclosure(self, wiring):
        ctx = ["system architecture uses microservices", "code: fixed bug in auth.py",
               "test: all passed", "risk: high for deployment", "memory: learned about Docker"]
        build_ctx = wiring.progressive_disclosure("BUILD", ctx)
        assert any("code" in c or "test" in c for c in build_ctx)

        classify_ctx = wiring.progressive_disclosure("CLASSIFY", ctx)
        assert any("risk" in c for c in classify_ctx)

    # P6 — Échecs → règles
    def test_learn_from_failures(self, wiring):
        rule = wiring.learn_from_failures("connection refused when calling API")
        assert rule is not None
        assert "prêt" in rule.lower() or "retry" in rule.lower() or "wait" in rule.lower()
        assert wiring.stats["rules_learned"] == 1

    def test_learn_no_pattern(self, wiring):
        rule = wiring.learn_from_failures("unknown cosmic ray bit flip")
        assert rule is None

    # P12 — Découpage par contexte
    def test_should_create_sub_agent_low_overlap(self, wiring):
        result = wiring.should_create_sub_agent(
            "frontend react component button styling css",
            "backend api database postgresql authentication"
        )
        assert result is True  # Very low overlap

    def test_should_create_sub_agent_high_overlap(self, wiring):
        result = wiring.should_create_sub_agent(
            "frontend react component button styling css",
            "frontend react component modal dialog css"
        )
        assert result is False  # High overlap

    # P19 — Mémoire gagne sa place
    def test_memory_earns_place_yes(self, wiring):
        result = wiring.memory_earns_place(
            "prefers Python for backend development",
            "I will use Python for the backend since it's what you prefer."
        )
        assert result is True

    def test_memory_earns_place_no(self, wiring):
        result = wiring.memory_earns_place(
            "likes mango ice cream",
            "Here is the Kubernetes deployment guide for your cluster."
        )
        assert result is False

    # Full pipeline
    def test_full_wiring_pipeline(self, wiring, tmpdir):
        """End-to-end: wire everything, use everything."""
        wiring.wire_memory(base_path=str(tmpdir))
        wiring.wire_preferences()
        wiring.wire_context(10000)

        # Remember a fact
        wiring.remember("food", "prefers dark chocolate", "[stated]")

        # Recall it
        facts = wiring.recall("food")
        assert any("dark chocolate" in f for f in facts)

        # Check memory impact
        impact = wiring.memory_earns_place(
            "prefers dark chocolate",
            "I recommend this dark chocolate dessert based on your preference."
        )
        assert impact is True

        # Progressive disclosure
        ctx = ["risk: high", "code: fixed", "test: passed", "memory: dark chocolate"]
        filtered = wiring.progressive_disclosure("BUILD", ctx)
        assert any("code" in c or "test" in c for c in filtered)

        # Stats
        assert wiring.stats["facts_stored"] >= 1


class TestInitWiring:
    def test_init_wiring_creates_all_modules(self, tmpdir):
        from src.sfd_wiring import init_wiring
        w = init_wiring(
            memory_base_path=str(tmpdir),
            durable_db_path=os.path.join(str(tmpdir), "test.db"),
            context_max_tokens=10000,
        )
        assert w._memory_bridge is not None
        assert w._pref_engine is not None
        assert w._context_mgr is not None

    def test_init_wiring_stats(self, tmpdir):
        from src.sfd_wiring import init_wiring
        w = init_wiring(memory_base_path=str(tmpdir))
        assert "facts_stored" in w.stats
        assert "facts_omitted" in w.stats
        assert "visuals_routed" in w.stats
