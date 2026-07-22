"""
Memory with Provenance — Structured file-based memory system.

Taxonomy: /profile.md, /preferences.md, /topics/*.md, /areas/*.md, /people/*.md
Provenance: [stated], [observed], [inferred] tags on every fact
Versioning: content hash (if_version) for concurrency control
Omission: strict rules on what NEVER gets stored
"""

from src.memory_provenance.taxonomy import MemoryTaxonomy, TAXONOMY_PATHS
from src.memory_provenance.frontmatter import MemoryFrontmatter, parse_frontmatter, dump_frontmatter
from src.memory_provenance.operations import MemoryOperations
from src.memory_provenance.omission import OmissionFilter, PROTECTED_CATEGORIES

__all__ = [
    "MemoryTaxonomy",
    "TAXONOMY_PATHS",
    "MemoryFrontmatter",
    "parse_frontmatter",
    "dump_frontmatter",
    "MemoryOperations",
    "OmissionFilter",
    "PROTECTED_CATEGORIES",
]
