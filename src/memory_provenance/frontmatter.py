"""
YAML frontmatter parsing for memory files.

Format:
    ---
    name: <slug>
    description: <one line>
    sources: [chat, claude-code, api]
    aliases: [alt name]
    version: <12-char content hash>
    retention: public | internal | personal
    updated: <ISO datetime>
    ---

    - [stated] fact stated by the user
    - [observed] fact observed by the system
    - [inferred] fact inferred with confidence
"""

from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional

_FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n", re.DOTALL)


@dataclass
class MemoryFrontmatter:
    """Parsed frontmatter from a memory file."""
    name: str = ""
    description: str = ""
    sources: list[str] = field(default_factory=list)
    aliases: list[str] = field(default_factory=list)
    version: str = ""
    retention: str = "internal"
    updated: str = ""

    def compute_version(self, content: str) -> str:
        """Compute SHA-256 content hash (12 chars) for versioning."""
        h = hashlib.sha256(content.encode("utf-8")).hexdigest()
        self.version = h[:12]
        return self.version


def parse_frontmatter(raw: str) -> tuple[MemoryFrontmatter, str]:
    """Parse YAML frontmatter from raw file content.

    Returns (frontmatter, body_content).
    """
    import yaml

    fm = MemoryFrontmatter()
    body = raw

    match = _FRONTMATTER_RE.match(raw)
    if match:
        try:
            data = yaml.safe_load(match.group(1)) or {}
            fm = MemoryFrontmatter(
                name=data.get("name", ""),
                description=data.get("description", ""),
                sources=data.get("sources", []),
                aliases=data.get("aliases", []),
                version=data.get("version", ""),
                retention=data.get("retention", "internal"),
                updated=data.get("updated", ""),
            )
        except Exception:
            pass  # Invalid YAML — return defaults
        body = raw[match.end():]

    return fm, body.strip()


def dump_frontmatter(fm: MemoryFrontmatter, body: str) -> str:
    """Serialize frontmatter + body to a complete file string."""
    import yaml

    data = {
        "name": fm.name,
        "description": fm.description,
        "sources": fm.sources,
        "aliases": fm.aliases,
        "version": fm.version,
        "retention": fm.retention,
        "updated": fm.updated or datetime.now(timezone.utc).isoformat(),
    }
    # Remove empty values
    data = {k: v for k, v in data.items() if v}

    yaml_str = yaml.dump(data, default_flow_style=False, allow_unicode=True, sort_keys=False)
    return f"---\n{yaml_str}---\n\n{body.strip()}\n"
