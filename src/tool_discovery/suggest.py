"""
Connector suggestion — present, never auto-select.

SFD §5.13.4: suggest_connectors presents results as actionable
suggestions. NEVER auto-select for third-party services — even if
the user is in a hurry, even if the tool is already connected.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from src.tool_discovery.registry import MCPRegistry, RegistryEntry


@dataclass
class ConnectorSuggestion:
    """A suggested connector with actionable presentation."""
    entry: RegistryEntry
    reason: str
    requires_confirmation: bool = True  # Always True for third-party

    def present(self) -> str:
        """Format for user presentation."""
        prefix = "🔌" if not self.entry.is_third_party else "🔌 (tiers)"
        return f"{prefix} **{self.entry.name}** — {self.entry.description} ({self.reason})"


class ConnectorSuggester:
    """Suggests connectors from the MCP registry.

    Golden rule: NEVER auto-select third-party services.
    Always present as suggestions requiring user confirmation.
    """

    def __init__(self, registry: Optional[MCPRegistry] = None):
        self._registry = registry or MCPRegistry()

    def suggest(
        self,
        query: str,
        limit: int = 3,
    ) -> list[ConnectorSuggestion]:
        """Search registry and return actionable suggestions.

        All third-party results require explicit user confirmation.
        """
        entries = self._registry.search(query, limit=limit)
        suggestions = []
        for entry in entries:
            suggestions.append(ConnectorSuggestion(
                entry=entry,
                reason=f"Correspond à votre demande: {query}",
                requires_confirmation=entry.is_third_party,
            ))
        return suggestions

    def suggest_by_category(self, category: str) -> list[ConnectorSuggestion]:
        """Suggest connectors for a task category."""
        entries = self._registry.by_category(category)
        return [
            ConnectorSuggestion(
                entry=e,
                reason=f"Outil pertinent pour la catégorie: {category}",
                requires_confirmation=e.is_third_party,
            )
            for e in entries
        ]

    def auto_connectable(self, suggestions: list[ConnectorSuggestion]) -> list[ConnectorSuggestion]:
        """Filter to only first-party connectors (auto-connectable)."""
        return [s for s in suggestions if not s.entry.is_third_party]


def suggest_connectors(
    query: str,
    registry: Optional[MCPRegistry] = None,
    limit: int = 3,
) -> list[ConnectorSuggestion]:
    """Convenience: suggest connectors for a query."""
    suggester = ConnectorSuggester(registry)
    return suggester.suggest(query, limit=limit)
