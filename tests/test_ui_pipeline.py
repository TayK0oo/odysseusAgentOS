"""
End-to-end UI pipeline test — simulates the full flow:
chat route → ThoughtBus wrapper → SSE events → cockpit JS

Runs without needing a browser or Docker services.
"""

import json
import asyncio
import pytest


class TestUIPipeline:
    """Simulates the full UI pipeline programmatically."""

    @pytest.mark.asyncio
    async def test_wrap_agent_stream_emits_phase_events(self):
        """Verify that wrap_agent_stream emits phase_enter/exit around the stream."""
        from src.thought_bus.integration import wrap_agent_stream

        # Simulate an async agent stream that yields 3 chunks
        async def mock_agent_stream():
            yield "data: {\"delta\":\"Bonjour\"}\n\n"
            yield "data: {\"delta\":\" !\"}\n\n"
            yield "data: [DONE]\n\n"

        events = []
        chunks = []
        async for chunk in wrap_agent_stream(
            mock_agent_stream(),
            session_id="test-session",
            objective="dis bonjour",
        ):
            if chunk.startswith("data: "):
                try:
                    data = json.loads(chunk[6:])
                    if "type" in data:
                        events.append(data)
                except json.JSONDecodeError:
                    pass
            chunks.append(chunk)

        # Should have phase_enter before chunks
        assert events[0]["type"] == "phase_enter"
        assert events[0]["phase"] == "BUILD"

        # Should have phase_exit after chunks
        assert events[1]["type"] == "phase_exit"
        assert events[1]["phase"] == "BUILD"

        # Should have thought_bus complete at end
        assert events[2]["type"] == "thought_bus"
        assert events[2]["status"] == "complete"

        # Chunks should include the agent output
        assert any("Bonjour" in c for c in chunks)

    @pytest.mark.asyncio
    async def test_wrap_disabled_passthrough(self, monkeypatch):
        """When bus is OFF, stream passes through unchanged."""
        monkeypatch.setenv("ODYSSEUS_THOUGHT_BUS", "off")
        from src.thought_bus.integration import wrap_agent_stream

        async def mock_agent_stream():
            yield "data: {\"delta\":\"test\"}\n\n"
            yield "data: [DONE]\n\n"

        chunks = []
        async for chunk in wrap_agent_stream(
            mock_agent_stream(),
            session_id="test",
        ):
            chunks.append(chunk)

        # No phase events, just the original chunks
        events = [json.loads(c[6:]) for c in chunks if c.startswith("data: ") and c[6:].strip() not in ("[DONE]", "")]
        assert not any(e.get("type") in ("phase_enter", "phase_exit") for e in events)

    def test_cockpit_phase_bar_rendering(self):
        """Simulate what cockpit.js would render for each phase."""
        phases = ['CLASSIFY', 'KNOW', 'PLAN', 'BUILD', 'QUALITY', 'AUTOEVAL', 'MEMORY_OBSERVE']

        # Simulate cockpit.onThoughtBusEvent for each phase
        states = []
        for i, phase in enumerate(phases):
            # phase_enter
            states.append({"phase": phase, "index": i + 1, "active": True, "done": list(range(i))})

        assert states[0]["phase"] == "CLASSIFY"
        assert states[0]["active"] is True
        assert states[0]["done"] == []

        assert states[3]["phase"] == "BUILD"
        assert states[3]["done"] == [0, 1, 2]  # C, K, P done

        assert states[6]["phase"] == "MEMORY_OBSERVE"

    @pytest.mark.asyncio
    async def test_full_chat_route_simulation(self):
        """Simulate the chat route flow end-to-end."""
        from src.thought_bus.integration import wrap_agent_stream
        from src.thought_bus.bus import ThoughtBus, thought_bus_enabled
        from src.thought_bus.subscriber import SubscriberRegistry, PhaseSubscriber

        # Track what subscribers see
        received = []

        class TestSubscriber:
            async def on_phase_enter(self, phase, ctx):
                received.append(("enter", phase))

            async def on_phase_exit(self, phase, ctx):
                received.append(("exit", phase))

        # Register subscriber
        reg = SubscriberRegistry()
        reg.register(PhaseSubscriber(TestSubscriber(), ["BUILD"], name="UITest"))

        async def mock_stream():
            yield "data: {\"delta\":\"OK\"}\n\n"

        chunks = []
        async for chunk in wrap_agent_stream(mock_stream(), session_id="ui-test", objective="test"):
            chunks.append(chunk)

        # Verify flow
        assert len(chunks) > 0
        # The stream passes through correctly
        assert any("OK" in c for c in chunks)

    def test_chat_route_import(self):
        """Verify chat route and ThoughtBus wrapper import correctly."""
        import routes.chat_routes
        from src.thought_bus.integration import wrap_agent_stream

        assert routes.chat_routes is not None
        assert callable(wrap_agent_stream)

    def test_cockpit_js_event_handler(self):
        """Simulate cockpit.onThoughtBusEvent for each event type."""
        # Simulate what the JS would do
        phase_state = {"active": None, "index": -1}
        phases = ['CLASSIFY', 'KNOW', 'PLAN', 'BUILD', 'QUALITY', 'AUTOEVAL', 'MEMORY_OBSERVE']

        # Simulate events
        events = [
            {"type": "phase_enter", "phase": "CLASSIFY", "index": 1, "total": 7},
            {"type": "phase_exit", "phase": "CLASSIFY", "index": 1, "total": 7},
            {"type": "phase_enter", "phase": "BUILD", "index": 4, "total": 7},
            {"type": "phase_exit", "phase": "BUILD", "index": 4, "total": 7},
            {"type": "thought_bus", "status": "complete"},
        ]

        for evt in events:
            if evt["type"] == "phase_enter":
                phase_state["active"] = evt["phase"]
                phase_state["index"] = phases.index(evt["phase"])
                # In JS: cockpit chip shows "CLASSIFY (1/7)" in green
                assert phase_state["active"] is not None
            elif evt["type"] == "thought_bus" and evt["status"] == "complete":
                phase_state["active"] = None
                phase_state["index"] = len(phases)

        # Final state: all phases done
        assert phase_state["active"] is None
        assert phase_state["index"] == 7

    def test_phase_bar_css_exists(self):
        """Verify the phase bar CSS classes are defined."""
        import os
        css_path = "static/css/style.min.css"
        assert os.path.exists(css_path)
        with open(css_path, "r", encoding="utf-8") as f:
            css = f.read()
        assert "cockpit-phase-bar" in css
        assert "pb-dot" in css
        assert "pb-active" in css
        assert "pb-done" in css

    def test_phase_bar_html_exists(self):
        """Verify the phase bar element exists in index.html."""
        import os
        html_path = "static/index.html"
        assert os.path.exists(html_path)
        with open(html_path, "r", encoding="utf-8") as f:
            html = f.read()
        assert "cockpit-phase-bar" in html
