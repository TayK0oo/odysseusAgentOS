"""
MCP Registry — external connector directory.

SFD §5.13.4: search_mcp_registry queries an external directory
when the user names an unconnected service or the task suggests
a service type without a connector.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

# Simulated registry — in production, this queries an external API
_SAMPLE_REGISTRY = [
    {"name": "salesforce-mcp", "description": "Salesforce CRM connector", "category": "crm", "is_third_party": True},
    {"name": "github-mcp", "description": "GitHub repository management", "category": "devtools", "is_third_party": False},
    {"name": "notion-mcp", "description": "Notion workspace connector", "category": "productivity", "is_third_party": True},
    {"name": "slack-mcp", "description": "Slack messaging connector", "category": "communication", "is_third_party": True},
    {"name": "jira-mcp", "description": "Jira issue tracking", "category": "devtools", "is_third_party": True},
    {"name": "postgres-mcp", "description": "PostgreSQL database connector", "category": "database", "is_third_party": False},
    {"name": "stripe-mcp", "description": "Stripe payments connector", "category": "payments", "is_third_party": True},
    {"name": "zendesk-mcp", "description": "Zendesk support connector", "category": "support", "is_third_party": True},
    {"name": "figma-mcp", "description": "Figma design connector", "category": "design", "is_third_party": True},
    {"name": "hubspot-mcp", "description": "HubSpot marketing connector", "category": "marketing", "is_third_party": True},
]


@dataclass
class RegistryEntry:
    name: str
    description: str
    category: str
    is_third_party: bool = True


class MCPRegistry:
    """Queries an external MCP connector directory."""

    def __init__(self, entries: Optional[list[dict]] = None):
        self._entries: list[RegistryEntry] = [
            RegistryEntry(**e) for e in (entries or _SAMPLE_REGISTRY)
        ]

    def search(self, query: str, limit: int = 5) -> list[RegistryEntry]:
        """Search the registry for matching connectors."""
        q = query.lower()
        results = [
            e for e in self._entries
            if q in e.name.lower() or q in e.description.lower() or q in e.category.lower()
        ]
        return results[:limit]

    def by_category(self, category: str) -> list[RegistryEntry]:
        """Get all connectors in a category."""
        return [e for e in self._entries if e.category == category.lower()]

    def third_party_only(self) -> list[RegistryEntry]:
        return [e for e in self._entries if e.is_third_party]


def search_registry(registry: MCPRegistry, query: str) -> list[RegistryEntry]:
    return registry.search(query)
