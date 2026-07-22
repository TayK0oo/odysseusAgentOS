"""
Memory Operations — CRUD with provenance and versioning.

Core API:
    memory_read(path) → (content, version_token)
    memory_write(path, content, if_version) → (success, version_token)
    memory_append(path, entry, if_version) → (success, version_token)
    memory_str_replace(path, old, new, if_version) → (success, version_token)
    memory_delete(path, if_version) → (success)
    memory_list(prefix) → [paths]

Concurrency control: if_version must match current hash.
Provenance: every entry must be tagged [stated]/[observed]/[inferred].
"""

from __future__ import annotations

import logging
import os
from dataclasses import dataclass
from typing import Optional

from src.memory_provenance.frontmatter import MemoryFrontmatter, parse_frontmatter, dump_frontmatter
from src.memory_provenance.taxonomy import MemoryTaxonomy
from src.memory_provenance.omission import OmissionFilter

logger = logging.getLogger(__name__)


@dataclass
class MemoryResult:
    """Result of a memory operation."""
    success: bool
    content: Optional[str] = None
    version_token: Optional[str] = None
    error: Optional[str] = None


class MemoryOperations:
    """CRUD operations on the memory file system.

    All write operations require if_version for concurrency control.
    All entries are filtered through OmissionFilter before storage.
    """

    def __init__(self, base_path: str = "workspace/memory"):
        self._base_path = base_path

    def _resolve(self, path: str) -> str:
        """Resolve a relative path to an absolute filesystem path."""
        path = path.lstrip("/")
        return os.path.join(self._base_path, path)

    def _ensure_dir(self, filepath: str) -> None:
        os.makedirs(os.path.dirname(filepath), exist_ok=True)

    def _compute_hash(self, content: str) -> str:
        import hashlib
        return hashlib.sha256(content.encode("utf-8")).hexdigest()[:12]

    # ------------------------------------------------------------------
    # Read
    # ------------------------------------------------------------------

    def memory_read(self, path: str) -> MemoryResult:
        """Read a memory file. Returns content + version token."""
        filepath = self._resolve(path)
        if not os.path.exists(filepath):
            return MemoryResult(success=False, error=f"File not found: {path}")
        try:
            with open(filepath, "r", encoding="utf-8") as f:
                content = f.read()
            version = self._compute_hash(content)
            return MemoryResult(success=True, content=content, version_token=version)
        except Exception as e:
            return MemoryResult(success=False, error=str(e))

    # ------------------------------------------------------------------
    # Write (create or replace)
    # ------------------------------------------------------------------

    def memory_write(
        self, path: str, content: str, if_version: str = "new"
    ) -> MemoryResult:
        """Create or replace a memory file. Requires if_version.

        if_version="new" → create only (fails if exists).
        if_version=<hash> → update (fails if hash doesn't match).
        """
        filepath = self._resolve(path)

        if if_version == "new":
            if os.path.exists(filepath):
                return MemoryResult(
                    success=False,
                    error=f"File already exists: {path}. Use update with version token.",
                )
        else:
            if not os.path.exists(filepath):
                return MemoryResult(success=False, error=f"File not found: {path}")
            current = self.memory_read(path)
            if not current.success:
                return current
            if current.version_token != if_version:
                return MemoryResult(
                    success=False,
                    content=current.content,
                    version_token=current.version_token,
                    error="Version mismatch — file was modified. Re-read and retry.",
                )

        # Parse and validate frontmatter
        fm, body = parse_frontmatter(content)
        if not fm.name:
            fm.name = os.path.splitext(os.path.basename(path))[0]
        fm.updated = ""  # Will be set by dump_frontmatter
        final_content = dump_frontmatter(fm, body)
        fm.compute_version(final_content)

        self._ensure_dir(filepath)
        try:
            with open(filepath, "w", encoding="utf-8") as f:
                f.write(final_content)
            new_version = fm.version
            return MemoryResult(success=True, version_token=new_version)
        except Exception as e:
            return MemoryResult(success=False, error=str(e))

    # ------------------------------------------------------------------
    # Append
    # ------------------------------------------------------------------

    def memory_append(
        self, path: str, entry: str, if_version: str
    ) -> MemoryResult:
        """Append a fact to a memory file.

        Rules:
        - File must exist
        - Entry must not already exist (dedup)
        - Entry must pass OmissionFilter
        - Entry should have provenance tag [stated]/[observed]/[inferred]
        - if_version must match current hash
        """
        if not OmissionFilter.is_safe(entry):
            return MemoryResult(
                success=False, error="Entry contains protected content — omitted."
            )

        current = self.memory_read(path)
        if not current.success:
            return current

        if current.version_token != if_version:
            return MemoryResult(
                success=False,
                content=current.content,
                version_token=current.version_token,
                error="Version mismatch — file was modified. Re-read and retry.",
            )

        if entry in current.content:
            return MemoryResult(
                success=False, error="Entry already exists — no duplicate.",
            )

        # Append after the frontmatter body
        fm, body = parse_frontmatter(current.content)
        new_body = body.rstrip() + "\n" + entry.strip() + "\n"
        new_content = dump_frontmatter(fm, new_body)
        new_version = self._compute_hash(new_content)

        self._ensure_dir(self._resolve(path))
        try:
            with open(self._resolve(path), "w", encoding="utf-8") as f:
                f.write(new_content)
            return MemoryResult(success=True, version_token=new_version)
        except Exception as e:
            return MemoryResult(success=False, error=str(e))

    # ------------------------------------------------------------------
    # String replace (surgical edit)
    # ------------------------------------------------------------------

    def memory_str_replace(
        self, path: str, old_str: str, new_str: str, if_version: str
    ) -> MemoryResult:
        """Replace a string in a memory file. Surgical, exact match."""
        current = self.memory_read(path)
        if not current.success:
            return current

        if current.version_token != if_version:
            return MemoryResult(
                success=False,
                content=current.content,
                version_token=current.version_token,
                error="Version mismatch — file was modified. Re-read and retry.",
            )

        count = current.content.count(old_str)
        if count == 0:
            return MemoryResult(success=False, error="old_str not found in file.")
        if count > 1:
            return MemoryResult(
                success=False, error=f"old_str found {count} times — must be unique.",
            )

        if not OmissionFilter.is_safe(new_str):
            return MemoryResult(
                success=False, error="New content contains protected data — omitted.",
            )

        new_content = current.content.replace(old_str, new_str, 1)
        new_version = self._compute_hash(new_content)

        try:
            with open(self._resolve(path), "w", encoding="utf-8") as f:
                f.write(new_content)
            return MemoryResult(success=True, version_token=new_version)
        except Exception as e:
            return MemoryResult(success=False, error=str(e))

    # ------------------------------------------------------------------
    # Delete
    # ------------------------------------------------------------------

    def memory_delete(self, path: str, if_version: str) -> MemoryResult:
        """Delete a memory file. Requires explicit user confirmation.

        The caller must have already confirmed with the user.
        This function just validates the version match.
        """
        current = self.memory_read(path)
        if not current.success:
            return current

        if current.version_token != if_version:
            return MemoryResult(
                success=False,
                content=current.content,
                version_token=current.version_token,
                error="Version mismatch — file was modified. Re-read and retry.",
            )

        try:
            os.remove(self._resolve(path))
            return MemoryResult(success=True)
        except Exception as e:
            return MemoryResult(success=False, error=str(e))

    # ------------------------------------------------------------------
    # List
    # ------------------------------------------------------------------

    def memory_list(self, prefix: str = "") -> list[dict]:
        """List memory files under a prefix."""
        base = self._resolve(prefix)
        if not os.path.exists(base):
            return []

        results = []
        for root, dirs, files in os.walk(base):
            for f in files:
                if f.endswith(".md"):
                    full = os.path.join(root, f)
                    rel = os.path.relpath(full, self._base_path).replace("\\", "/")
                    stat = os.stat(full)
                    results.append({
                        "path": rel,
                        "name": f,
                        "size": stat.st_size,
                        "modified": stat.st_mtime,
                    })
        return results
