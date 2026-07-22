"""
Conversation Search with Linguistic Signal Detection (§5.16).

Two search modes:
    conversation_search — full-text keyword search
    recent_chats — temporal window search

Automatic detection of linguistic signals indicating
reference to past conversations.
"""

from src.conversation_search.search import ConversationSearch, SearchResult
from src.conversation_search.signals import LinguisticSignalDetector, SignalType

__all__ = [
    "ConversationSearch", "SearchResult",
    "LinguisticSignalDetector", "SignalType",
]
