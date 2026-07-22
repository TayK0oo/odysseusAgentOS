"""
Tests for the Thought Bus (src/thought_bus/).

Covers: event types, subscriber registration, phase walking,
kill-switch gating, priority ordering, context enrichment.
"""

import os
import pytest

from src.thought_bus.events import PhaseEvent, BusContext
from src.thought_bus.subscriber import on_phase, SubscriberRegistry, PhaseSubscriber
from src.thought_bus.bus import ThoughtBus, thought_bus_enabled


# ---------------------------------------------------------------------------
# Events
# ---------------------------------------------------------------------------

class TestPhaseEvent:
    def test_enum_values(self):
        assert PhaseEvent.ENTER == "phase_enter"
        assert PhaseEvent.EXIT == "phase_exit"
        assert PhaseEvent.TRANSITION == "phase_transition"

    def test_str_equality(self):
        assert PhaseEvent.ENTER.value == "phase_enter"
        assert PhaseEvent.EXIT.value == "phase_exit"


class TestBusContext:
    def test_defaults(self):
        ctx = BusContext()
        assert ctx.project_id is None
        assert ctx.session_id is None
        assert ctx.tokens_used == 0
        assert ctx.active_agents == []
        assert ctx.metadata == {}

    def test_fields(self):
        ctx = BusContext(
            project_id="p1",
            session_id="s1",
            objective="Build X",
            risk_level="moderate",
        )
        assert ctx.project_id == "p1"
        assert ctx.objective == "Build X"
        assert ctx.risk_level == "moderate"

    def test_enrich_existing_field(self):
        ctx = BusContext(tokens_used=100)
        ctx.enrich(tokens_used=200, objective="test")
        assert ctx.tokens_used == 200
        assert ctx.objective == "test"

    def test_enrich_new_field_goes_to_metadata(self):
        ctx = BusContext()
        ctx.enrich(custom_key="custom_value")
        assert ctx.metadata["custom_key"] == "custom_value"

    def test_timestamp_auto_set(self):
        ctx = BusContext()
        assert ctx.timestamp is not None
        assert "T" in ctx.timestamp


# ---------------------------------------------------------------------------
# Subscriber
# ---------------------------------------------------------------------------

class TestPhaseSubscriber:
    def test_basic(self):
        class MySub:
            def __init__(self):
                self.events = []

            async def on_phase_enter(self, phase, ctx):
                self.events.append(("enter", phase))

            async def on_phase_exit(self, phase, ctx):
                self.events.append(("exit", phase))

        instance = MySub()
        sub = PhaseSubscriber(instance, ["BUILD"], priority=30)

        import asyncio

        async def run():
            ctx = BusContext()
            await sub.on_enter("BUILD", ctx)
            await sub.on_exit("BUILD", ctx)

        asyncio.run(run())
        assert instance.events == [("enter", "BUILD"), ("exit", "BUILD")]


class TestSubscriberRegistry:
    def test_register_and_get(self):
        reg = SubscriberRegistry()

        class SubA:
            pass

        class SubB:
            pass

        reg.register(PhaseSubscriber(SubA(), ["BUILD"], priority=10, name="A"))
        reg.register(PhaseSubscriber(SubB(), ["BUILD"], priority=20, name="B"))

        subs = reg.get("BUILD")
        assert len(subs) == 2
        assert subs[0].name == "A"  # lower priority = first
        assert subs[1].name == "B"

    def test_get_unknown_phase(self):
        reg = SubscriberRegistry()
        assert reg.get("NONEXISTENT") == []

    def test_list_all(self):
        reg = SubscriberRegistry()

        class SubA:
            pass

        class SubB:
            pass

        reg.register(PhaseSubscriber(SubA(), ["BUILD"], name="A"))
        reg.register(PhaseSubscriber(SubB(), ["QUALITY"], name="B"))

        all_subs = reg.list_all()
        assert "BUILD" in all_subs
        assert "QUALITY" in all_subs
        assert "A" in all_subs["BUILD"]
        assert "B" in all_subs["QUALITY"]

    def test_phase_case_insensitive(self):
        reg = SubscriberRegistry()

        class Sub:
            pass

        reg.register(PhaseSubscriber(Sub(), ["build"], name="X"))
        assert len(reg.get("BUILD")) == 1
        assert len(reg.get("build")) == 1


# ---------------------------------------------------------------------------
# ThoughtBus
# ---------------------------------------------------------------------------

class TestThoughtBusBasics:
    def test_initial_state(self):
        reg = SubscriberRegistry()
        bus = ThoughtBus(reg, objective="Test")
        assert bus.current_phase is None
        assert bus.is_complete is False
        assert bus.context.objective == "Test"

    def test_advance_through_phases(self):
        reg = SubscriberRegistry()
        bus = ThoughtBus(reg)
        phases_seen = []
        while True:
            phase = bus.advance()
            if phase is None:
                break
            phases_seen.append(phase.value)

        assert len(phases_seen) == 7
        assert phases_seen == [
            "CLASSIFY", "KNOW", "PLAN", "BUILD",
            "QUALITY", "AUTOEVAL", "MEMORY_OBSERVE",
        ]
        assert bus.is_complete is True

    def test_sub_count(self):
        reg = SubscriberRegistry()

        class Sub:
            pass

        reg.register(PhaseSubscriber(Sub(), ["BUILD", "QUALITY"], name="X"))
        bus = ThoughtBus(reg)
        assert bus.sub_count("BUILD") == 1
        assert bus.sub_count("QUALITY") == 1
        assert bus.sub_count("PLAN") == 0

    def test_summary(self):
        reg = SubscriberRegistry()
        bus = ThoughtBus(reg, objective="Test", session_id="abc")
        summary = bus.summary()
        assert summary["current_phase"] is None
        assert summary["is_complete"] is False
        assert summary["context"]["objective"] == "Test"
        assert summary["context"]["session_id"] == "abc"


class TestThoughtBusWalk:
    @pytest.mark.asyncio
    async def test_walk_yields_all_events(self):
        reg = SubscriberRegistry()
        bus = ThoughtBus(reg)
        events = []
        async for event in bus.walk():
            events.append(event)

        # Check structure: enter+active+exit per phase, plus complete
        types = [e["type"] for e in events]
        assert types[0] == "phase_enter"
        assert types[-1] == "thought_bus"
        assert types[-2] == "phase_exit"

    @pytest.mark.asyncio
    async def test_walk_stop_after(self):
        reg = SubscriberRegistry()
        bus = ThoughtBus(reg)
        phases = set()
        async for event in bus.walk(stop_after="PLAN"):
            if "phase" in event:
                phases.add(event["phase"])

        # Should only walk CLASSIFY, KNOW, PLAN (stop after PLAN inclusive)
        assert "CLASSIFY" in phases
        assert "KNOW" in phases
        assert "PLAN" in phases
        assert "BUILD" not in phases

    @pytest.mark.asyncio
    async def test_subscriber_receives_events(self):
        reg = SubscriberRegistry()

        class TrackingSub:
            def __init__(self):
                self.entered = []
                self.exited = []

            async def on_phase_enter(self, phase, ctx):
                self.entered.append(phase)

            async def on_phase_exit(self, phase, ctx):
                self.exited.append(phase)

        sub = TrackingSub()
        reg.register(PhaseSubscriber(sub, ["BUILD"], name="Tracker"))

        bus = ThoughtBus(reg)
        async for _ in bus.walk():
            pass

        assert "BUILD" in sub.entered
        assert "BUILD" in sub.exited
        assert "CLASSIFY" not in sub.entered  # not subscribed

    @pytest.mark.asyncio
    async def test_context_enrichment(self):
        reg = SubscriberRegistry()

        class EnrichingSub:
            async def on_phase_enter(self, phase, ctx):
                ctx.enrich(risk_level="high", custom_flag=True)

        reg.register(PhaseSubscriber(EnrichingSub(), ["CLASSIFY"], name="Enricher"))

        bus = ThoughtBus(reg)
        async for _ in bus.walk(stop_after="CLASSIFY"):
            pass

        assert bus.context.risk_level == "high"
        assert bus.context.metadata.get("custom_flag") is True


# ---------------------------------------------------------------------------
# Kill-switch
# ---------------------------------------------------------------------------

class TestThoughtBusKillSwitch:
    def test_enabled_by_default(self):
        assert thought_bus_enabled() is True

    def test_disabled_via_env(self, monkeypatch):
        monkeypatch.setenv("ODYSSEUS_THOUGHT_BUS", "off")
        assert thought_bus_enabled() is False

    def test_enabled_via_env(self, monkeypatch):
        monkeypatch.setenv("ODYSSEUS_THOUGHT_BUS", "on")
        assert thought_bus_enabled() is True

    @pytest.mark.asyncio
    async def test_walk_respects_killswitch(self, monkeypatch):
        monkeypatch.setenv("ODYSSEUS_THOUGHT_BUS", "off")
        reg = SubscriberRegistry()
        bus = ThoughtBus(reg)
        events = []
        async for event in bus.walk():
            events.append(event)
        assert len(events) == 1
        assert events[0]["status"] == "disabled"


# ---------------------------------------------------------------------------
# @on_phase decorator
# ---------------------------------------------------------------------------

class TestOnPhaseDecorator:
    def test_decorator_registers_subscriber(self):
        reg = SubscriberRegistry()

        @on_phase("BUILD", priority=10)
        class MyModule:
            def __init__(self):
                self.called = False

            async def on_phase_enter(self, phase, ctx):
                self.called = True

        # The decorator patches __init__, so instantiation triggers registration
        instance = MyModule()

        subs = reg.get("BUILD")
        assert len(subs) == 1
        assert subs[0].name == "MyModule"
        assert subs[0].priority == 10

    def test_decorator_disabled(self):
        reg = SubscriberRegistry()

        @on_phase("BUILD", enabled=False)
        class DisabledModule:
            pass

        instance = DisabledModule()
        subs = reg.get("BUILD")
        assert len(subs) == 0

    def test_decorator_multiple_phases(self):
        reg = SubscriberRegistry()

        @on_phase("BUILD", "QUALITY", priority=30)
        class MultiPhaseModule:
            pass

        instance = MultiPhaseModule()
        assert len(reg.get("BUILD")) == 1
        assert len(reg.get("QUALITY")) == 1
        assert len(reg.get("PLAN")) == 0
