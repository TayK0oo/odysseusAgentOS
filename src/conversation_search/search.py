"""
Conversation search engine (§5.16).

Two modes:
    conversation_search(query, project_id) → full-text keyword search
    recent_chats(project_id, window, page) → temporal window search

Results are isolated by project — no cross-project leakage.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone, timedelta
from typing import Optional


@dataclass
class SearchResult:
    """A single search result from conversation history."""
    session_id: str
    project_id: str
    snippet: str
    timestamp: str
    is_user_message: bool = True  # user vs system
    score: float = 0.0
    metadata: dict = field(default_factory=dict)


class ConversationSearch:
    """Searches conversation history with two modes.

    In production, this would use Meilisearch for full-text search
    and SQLite for temporal queries. The current implementation
    provides the API surface; the backend is pluggable.
    """

    def __init__(self, backend: Optional[str] = None):
        self._backend = backend or "memory"  # "memory", "meilisearch", "sqlite"
        self._store: list[SearchResult] = []

    def index(self, result: SearchResult) -> None:
        """Index a conversation turn for future search."""
        self._store.append(result)

    def conversation_search(
        self,
        query: str,
        project_id: Optional[str] = None,
        limit: int = 10,
    ) -> list[SearchResult]:
        """Full-text keyword search across conversations.

        The query should contain content words (the subject),
        not meta-words ("discuté", "hier").
        """
        query_lower = query.lower()
        results = []
        for r in self._store:
            if project_id and r.project_id != project_id:
                continue
            if query_lower in r.snippet.lower():
                results.append(r)

        # Sort by score descending, then by timestamp
        results.sort(key=lambda r: (-r.score, r.timestamp), reverse=True)
        return results[:limit]

    def recent_chats(
        self,
        project_id: Optional[str] = None,
        window_days: int = 7,
        page: int = 1,
        page_size: int = 20,
    ) -> list[SearchResult]:
        """Temporal window search — conversations from the last N days."""
        cutoff = (datetime.now(timezone.utc) - timedelta(days=window_days)).isoformat()
        results = []
        for r in self._store:
            if project_id and r.project_id != project_id:
                continue
            if r.timestamp >= cutoff:
                results.append(r)

        results.sort(key=lambda r: r.timestamp, reverse=True)
        start = (page - 1) * page_size
        return results[start:start + page_size]

    def search_by_session(self, session_id: str) -> list[SearchResult]:
        """Get all turns from a specific session."""
        return [r for r in self._store if r.session_id == session_id]

    @property
    def count(self) -> int:
        return len(self._store)

    def clear(self, project_id: Optional[str] = None) -> int:
        """Clear stored results, optionally filtered by project."""
        if project_id is None:
            removed = len(self._store)
            self._store.clear()
            return removed
        before = len(self._store)
        self._store = [r for r in self._store if r.project_id != project_id]
        return before - len(self._store)
