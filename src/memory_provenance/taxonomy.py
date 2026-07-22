"""
Memory taxonomy — defines the file structure for persistent memory.

Taxonomy:
    /profile.md       — stable identity (> 3 months)
    /preferences.md   — behavioral + contextual
    /topics/*.md      — facts by domain
    /areas/*.md       — projects, responsibilities, incidents
    /people/*.md      — relational context
"""

from enum import Enum
from typing import Optional

TAXONOMY_PATHS = {
    "profile": "profile.md",
    "preferences": "preferences.md",
    "topics": "topics",
    "areas": "areas",
    "people": "people",
}


class MemoryTaxonomy:
    """Validates and resolves memory file paths according to taxonomy.

    Cardinal rule: a fact about X goes in X's file, not the file
    that happens to be already open.
    """

    ROOTS = ("profile", "preferences", "topics", "areas", "people")

    @staticmethod
    def resolve(path: str) -> Optional[str]:
        """Resolve a path to its taxonomy root.

        Returns the root name (profile, topics, etc.) or None.
        """
        path = path.lstrip("/").lower()
        for root in MemoryTaxonomy.ROOTS:
            if path.startswith(root):
                return root
        return None

    @staticmethod
    def is_valid(path: str) -> bool:
        """Check if a path conforms to the memory taxonomy."""
        return MemoryTaxonomy.resolve(path) is not None

    @staticmethod
    def suggest_path(fact_domain: str, filename: str) -> str:
        """Suggest the correct path for a fact.

        Example: suggest_path("food", "preferences") → "topics/food.md"
        """
        domain = fact_domain.lower().replace(" ", "-")
        return f"topics/{domain}.md"

    @staticmethod
    def is_durable(subject: str) -> bool:
        """Test: will this still be true in 3 months?

        Applies to /profile.md entries — only store truly stable identity.
        """
        durable_indicators = [
            "role", "job", "company", "location", "name", "language",
            "expertise", "profession", "industry", "team",
        ]
        subject_lower = subject.lower()
        return any(ind in subject_lower for ind in durable_indicators)
