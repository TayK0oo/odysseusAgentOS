# ChannelGateway Outbound Round-Trip — Design

**Date:** 2026-07-05
**Milestone:** M3.5 (channel bootstrap) — closing the outbound deferral
**Status:** Implemented (kill-switched OFF by default)

## Problem

The M3.5 channel bootstrap wakes the Discord/Telegram adapters, but the inbound
handler (`src/channel_bootstrap.py::make_inbound_handler`) fired
`event_bus.fire_event("channel_message_<channel>", None)` and discarded the
`InboundMessage`. Because `fire_event(event_type, owner=None)` is trigger-only
(no payload), nothing ran the agent on what was actually said, and nothing
replied to the sender. The two-way round-trip was the missing feature.

Confirmed during exploration: both adapters already populate a live per-message
reply callback — `discord_adapter.py:70` `reply_fn=lambda text: msg.reply(...)`,
`telegram_adapter.py:64` `reply_fn=lambda text: msg.reply_text(...)`. The reply
path exists per message; the handler was throwing it away.

## Decision

**Approach A — direct round-trip via `reply_fn`**, **stateless one-shot** identity.

The inbound handler, when enabled, runs the native one-shot agent on
`message.content` and replies via the message's own `reply_fn` (falling back to
native gateway delivery). The `fire_event` trigger still fires first, unchanged,
as an observability/trigger signal.

Rejected alternatives:
- **B (event + side-channel context store):** reconstructs data `reply_fn`
  already hands us in-scope. Over-engineered.
- **C (full chat-pipeline + per-user session):** heaviest; YAGNI for construction
  phase with no users. The stateless design can grow into it later.

Identity: **stateless one-shot, `owner=None`, no session.** Chosen for the
current construction phase (no users). Structured so a per-sender identity
(`discord:<sender_id>`) can replace `owner=None` later without touching callers.

## Native-first mapping

No new provider/model/endpoint machinery. The agent call routes through
`src/task_endpoint.py::task_llm_call_async(messages, owner=None)` — the shared
background-task LLM candidate chain that already resolves endpoint/model/fallback
from the native `ModelEndpoint` config (same path scheduled tasks use). Reply
delivery reuses the adapter's native `reply_fn`, else the existing
`deliver_outbound` → `ChannelGateway.send` path.

## Kill-switch

New gate `ODYSSEUS_CHANNEL_AGENT_REPLY`, default OFF (mirrors the existing
`_gate()` truthy set `{on,1,true,yes}`). OFF ⇒ inbound handling is byte-identical
to before (fire trigger event only). ON ⇒ additionally run the agent and reply.
Independent of the `ODYSSEUS_INPROCESS_DISCORD/TELEGRAM` adapter gates, so the
round-trip can be enabled/disabled without touching adapter startup.

## Components

- `channel_agent_reply_enabled()` — the gate.
- `run_agent_reply(message, agent_call=None, gateway=None) -> Optional[str]` —
  the round-trip: skip empty content → call agent → reply via `reply_fn` → fall
  back to gateway. Best-effort, returns reply text or None, never raises.
  `agent_call`/`gateway` are injectable for tests (no real LLM or network).
- `make_inbound_handler(fire_event=None, agent_call=None, gateway=None)` — fires
  the trigger event, then calls `run_agent_reply` iff the gate is ON.

## Testing

`tests/test_channel_bootstrap.py` (all fakes, no network/LLM):
- gate default OFF;
- gate OFF ⇒ agent never called, reply untouched, event still fires;
- gate ON ⇒ agent runs on `message.content`, reply delivered via `reply_fn`;
- no `reply_fn` ⇒ falls back to gateway `OutboundMessage` to the sender;
- agent raises ⇒ no crash, no reply;
- empty content ⇒ agent not called.

## Out of scope

Per-user sessions/history, tool access on channel replies (would move to
`stream_agent_loop`), multi-turn context. Deferred until there are real users.
