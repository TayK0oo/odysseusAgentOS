"""
Channel Gateway — Inbound bus + Outbound bus.
Ajouter un canal = ajouter un adapter. Pas toucher au cœur.
"""
import asyncio
import logging
from abc import ABC, abstractmethod
from typing import Optional, Callable, Any
from dataclasses import dataclass
from enum import Enum

logger = logging.getLogger(__name__)

class ChannelType(Enum):
    DISCORD = "discord"
    TELEGRAM = "telegram"
    EMAIL = "email"
    WEBHOOK = "webhook"

@dataclass
class InboundMessage:
    """Message entrant normalisé."""
    channel: ChannelType
    sender_id: str           # ID utilisateur dans le canal
    sender_name: str
    content: str
    raw: dict                # Payload original
    reply_fn: Optional[Callable] = None  # Fonction pour répondre directement

@dataclass
class OutboundMessage:
    """Message sortant normalisé."""
    channel: ChannelType
    recipient_id: str
    content: str
    metadata: dict = None

class ChannelAdapter(ABC):
    """Interface abstraite pour tous les adapters."""

    @property
    @abstractmethod
    def channel_type(self) -> ChannelType:
        ...

    @abstractmethod
    async def send(self, message: OutboundMessage) -> bool:
        """Envoie un message via ce canal. Retourne True si succès."""
        ...

    @abstractmethod
    async def start_listening(self, on_message: Callable[[InboundMessage], Any]) -> None:
        """Démarre l'écoute des messages entrants."""
        ...

class ChannelGateway:
    """
    Bus central : inbound (adapters → handler) + outbound (handler → adapters).
    """

    def __init__(self):
        self._adapters: dict[ChannelType, ChannelAdapter] = {}
        self._inbound_handler: Optional[Callable] = None

    def register_adapter(self, adapter: ChannelAdapter) -> None:
        """Enregistre un adapter. Remplace si déjà présent."""
        self._adapters[adapter.channel_type] = adapter
        logger.info(f"Channel Gateway: adapter {adapter.channel_type.value} enregistré")

    def set_inbound_handler(self, handler: Callable[[InboundMessage], Any]) -> None:
        """Définit le handler pour tous les messages entrants."""
        self._inbound_handler = handler

    async def send(self, message: OutboundMessage) -> bool:
        """Envoie via l'adapter approprié."""
        adapter = self._adapters.get(message.channel)
        if not adapter:
            logger.warning(f"Pas d'adapter pour {message.channel}")
            return False
        return await adapter.send(message)

    async def broadcast(self, content: str, channels: list[ChannelType] = None) -> dict:
        """Broadcast vers plusieurs canaux."""
        targets = channels or list(self._adapters.keys())
        results = {}
        for ch in targets:
            adapter = self._adapters.get(ch)
            if adapter:
                # Broadcast sans recipient_id spécifique — chaque adapter gère son channel par défaut
                msg = OutboundMessage(channel=ch, recipient_id="broadcast", content=content)
                results[ch.value] = await adapter.send(msg)
        return results

    async def start_all(self) -> None:
        """Démarre tous les adapters en écoute."""
        tasks = []
        for adapter in self._adapters.values():
            if self._inbound_handler:
                tasks.append(adapter.start_listening(self._inbound_handler))
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)

# Singleton
_gateway_instance: Optional[ChannelGateway] = None

def get_gateway() -> ChannelGateway:
    global _gateway_instance
    if _gateway_instance is None:
        _gateway_instance = ChannelGateway()
    return _gateway_instance
