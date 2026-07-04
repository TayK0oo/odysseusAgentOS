"""channel_bootstrap.py — wake the dormant Discord/Telegram adapters (M3.5).

The Discord/Telegram adapters (src/adapters/*) and the ChannelGateway
(src/channel_gateway.py) already exist but have zero in-process callers, so the
gateway never runs live. This module is the single, kill-switched wake-up point
that `_startup_event` calls:

    - inbound  : adapter -> gateway inbound handler -> event_bus.fire_event(...)
    - outbound : OutboundMessage -> gateway.send(...) (native delivery)

SAFETY: each channel is OFF by default behind its own kill-switch
(ODYSSEUS_INPROCESS_DISCORD / ODYSSEUS_INPROCESS_TELEGRAM), mirroring
src/orchestrator/phase_tracker.tracker_enabled(). With both OFF (the default),
bootstrap_channels() registers nothing and starts nothing, so startup behaviour
is byte-identical to today. Everything here is best-effort: a broken adapter,
bus, or bot can never crash the app.
"""
from __future__ import annotations

import logging
import os
from typing import Callable, Optional

from src.channel_gateway import (
    ChannelGateway,
    ChannelType,
    InboundMessage,
    OutboundMessage,
    get_gateway,
)

logger = logging.getLogger(__name__)

_TRUTHY = {"on", "1", "true", "yes"}


def _gate(env_name: str) -> bool:
    """OFF unless ``env_name`` is set to a truthy value (mirror tracker_enabled)."""
    val = os.getenv(env_name, "off").strip().lower()
    return val in _TRUTHY


def discord_inprocess_enabled() -> bool:
    """OFF unless ODYSSEUS_INPROCESS_DISCORD is truthy."""
    return _gate("ODYSSEUS_INPROCESS_DISCORD")


def telegram_inprocess_enabled() -> bool:
    """OFF unless ODYSSEUS_INPROCESS_TELEGRAM is truthy."""
    return _gate("ODYSSEUS_INPROCESS_TELEGRAM")


def make_inbound_handler(fire_event: Optional[Callable] = None) -> Callable:
    """Build the inbound handler: channel message -> event_bus.fire_event(...).

    The event name is ``channel_message_<channel>`` (e.g. channel_message_discord),
    which event-triggered scheduled tasks can subscribe to. Best-effort: any error
    (bus down, bad payload) is swallowed so an inbound message never crashes the
    adapter's listen loop.
    """
    if fire_event is None:
        from src.event_bus import fire_event as _fire_event
        fire_event = _fire_event

    async def _handler(message: InboundMessage) -> None:
        try:
            event_name = f"channel_message_{message.channel.value}"
            fire_event(event_name, None)
        except Exception:
            logger.exception("Inbound channel message handling failed (best-effort)")

    return _handler


async def deliver_outbound(
    message: OutboundMessage, gateway: Optional[ChannelGateway] = None
) -> bool:
    """Deliver an OutboundMessage via native gateway delivery. Best-effort.

    Routes through ChannelGateway.send(), which dispatches to the registered
    adapter for the message's channel. Returns False (never raises) on any error
    or when no adapter is registered.
    """
    gw = gateway or get_gateway()
    try:
        return await gw.send(message)
    except Exception:
        logger.exception("Outbound channel delivery failed (best-effort)")
        return False


def _default_adapter_factory(channel: ChannelType):
    """Build the real adapter for ``channel`` from environment configuration.

    Kept separate so tests can inject a fake factory and never import discord.py
    / python-telegram-bot or open a real connection.
    """
    if channel is ChannelType.DISCORD:
        from src.adapters.discord_adapter import DiscordAdapter
        chan_id = os.getenv("DISCORD_DEFAULT_CHANNEL_ID")
        return DiscordAdapter(
            bot_token=os.getenv("DISCORD_BOT_TOKEN"),
            default_channel_id=int(chan_id) if chan_id else None,
        )
    if channel is ChannelType.TELEGRAM:
        from src.adapters.telegram_adapter import TelegramAdapter
        chat_id = os.getenv("TELEGRAM_DEFAULT_CHAT_ID")
        return TelegramAdapter(
            bot_token=os.getenv("TELEGRAM_BOT_TOKEN"),
            default_chat_id=int(chat_id) if chat_id else None,
        )
    raise ValueError(f"No adapter factory for channel {channel!r}")


async def bootstrap_channels(
    gateway: Optional[ChannelGateway] = None,
    adapter_factory: Optional[Callable[[ChannelType], object]] = None,
    fire_event: Optional[Callable] = None,
) -> list[ChannelType]:
    """Wake the gated in-process channel adapters. Returns the list started.

    For each channel whose kill-switch is ON: build its adapter, register it on
    the gateway, wire the inbound handler, then start_all() so it begins
    listening. With every gate OFF (the default) this registers nothing, starts
    nothing, and returns []. Fully best-effort — never raises into startup.
    """
    gw = gateway or get_gateway()
    factory = adapter_factory or _default_adapter_factory

    wanted: list[ChannelType] = []
    if discord_inprocess_enabled():
        wanted.append(ChannelType.DISCORD)
    if telegram_inprocess_enabled():
        wanted.append(ChannelType.TELEGRAM)

    if not wanted:
        return []

    started: list[ChannelType] = []
    for channel in wanted:
        try:
            adapter = factory(channel)
            gw.register_adapter(adapter)
            started.append(channel)
        except Exception:
            logger.exception("Failed to register %s adapter (best-effort)", channel)

    if not started:
        return []

    try:
        gw.set_inbound_handler(make_inbound_handler(fire_event=fire_event))
        await gw.start_all()
    except Exception:
        logger.exception("Failed to start channel adapters (best-effort)")

    logger.info(
        "In-process channel adapters started: %s",
        ", ".join(c.value for c in started),
    )
    return started
