"""
Dynamic tool search — load tool definitions on demand.

SFD §5.13.2: when > 15-20 tools, discover and load schemas
at runtime instead of saturating the context window.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class ToolDescriptor:
    """Lightweight tool metadata for discovery."""
    name: str
    description: str
    category: str = "general"
    is_first_party: bool = True
    keywords: list[str] = field(default_factory=list)

    def matches(self, query: str) -> bool:
        """Check if this tool matches a search query."""
        q = query.lower()
        searchable = [self.name.lower(), self.description.lower()] + [k.lower() for k in self.keywords]
        return any(q in s for s in searchable)


class ToolSearch:
    """Discovers and loads tool definitions on demand.

    Maintains a lightweight index of tool metadata.
    Full schemas are loaded only when the tool is selected.
    """

    def __init__(self, max_loaded: int = 15):
        self._index: dict[str, ToolDescriptor] = {}
        self._max_loaded = max_loaded  # threshold before discovery mode kicks in

    def register(self, tool: ToolDescriptor) -> None:
        """Register a tool in the discovery index."""
        self._index[tool.name] = tool

    def register_many(self, tools: list[ToolDescriptor]) -> None:
        for t in tools:
            self.register(t)

    def search(self, query: str, limit: int = 5) -> list[ToolDescriptor]:
        """Search for tools matching a query."""
        results = [t for t in self._index.values() if t.matches(query)]
        results.sort(key=lambda t: (not t.is_first_party, t.name))
        return results[:limit]

    def should_defer_loading(self) -> bool:
        """True when tool count exceeds threshold — use discovery mode."""
        return len(self._index) > self._max_loaded

    def first_party_tools(self) -> list[ToolDescriptor]:
        return [t for t in self._index.values() if t.is_first_party]

    def third_party_tools(self) -> list[ToolDescriptor]:
        return [t for t in self._index.values() if not t.is_first_party]

    @property
    def count(self) -> int:
        return len(self._index)


# Convenience
def search_tools(index: ToolSearch, query: str) -> list[ToolDescriptor]:
    return index.search(query)
