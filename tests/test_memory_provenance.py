"""
Tests for Memory with Provenance (src/memory_provenance/).

Covers: taxonomy, frontmatter, omission filter, CRUD operations,
versioning, concurrency control.
"""

import os
import tempfile
import pytest

from src.memory_provenance.taxonomy import MemoryTaxonomy
from src.memory_provenance.frontmatter import MemoryFrontmatter, parse_frontmatter, dump_frontmatter
from src.memory_provenance.omission import OmissionFilter
from src.memory_provenance.operations import MemoryOperations


# ---------------------------------------------------------------------------
# Taxonomy
# ---------------------------------------------------------------------------

class TestMemoryTaxonomy:
    def test_valid_roots(self):
        assert MemoryTaxonomy.resolve("profile.md") == "profile"
        assert MemoryTaxonomy.resolve("topics/food.md") == "topics"
        assert MemoryTaxonomy.resolve("areas/auth-redesign.md") == "areas"
        assert MemoryTaxonomy.resolve("people/collaborator.md") == "people"

    def test_invalid_path(self):
        assert MemoryTaxonomy.resolve("random/file.txt") is None

    def test_is_durable(self):
        assert MemoryTaxonomy.is_durable("role at company") is True
        assert MemoryTaxonomy.is_durable("favorite color") is False


# ---------------------------------------------------------------------------
# Frontmatter
# ---------------------------------------------------------------------------

class TestFrontmatter:
    SAMPLE = """---
name: food
description: Food preferences
sources: [chat]
version: a1b2c3d4e5f6
retention: personal
---

- [stated] likes sushi
- [observed] orders pizza often
"""

    def test_parse(self):
        fm, body = parse_frontmatter(self.SAMPLE)
        assert fm.name == "food"
        assert fm.description == "Food preferences"
        assert fm.sources == ["chat"]
        assert fm.retention == "personal"
        assert "- [stated] likes sushi" in body
        assert "- [observed] orders pizza often" in body

    def test_no_frontmatter(self):
        fm, body = parse_frontmatter("just content\nno frontmatter")
        assert fm.name == ""
        assert body == "just content\nno frontmatter"

    def test_dump_roundtrip(self):
        fm, body = parse_frontmatter(self.SAMPLE)
        fm.updated = "2026-07-22T00:00:00"
        dumped = dump_frontmatter(fm, body)
        fm2, body2 = parse_frontmatter(dumped)
        assert fm2.name == fm.name
        assert fm2.sources == fm.sources
        assert body2.strip() == body.strip()

    def test_compute_version(self):
        fm = MemoryFrontmatter()
        v = fm.compute_version("test content")
        assert len(v) == 12
        # Same content = same hash
        v2 = fm.compute_version("test content")
        assert v == v2
        # Different content = different hash
        v3 = fm.compute_version("different content")
        assert v != v3


# ---------------------------------------------------------------------------
# Omission filter
# ---------------------------------------------------------------------------

class TestOmissionFilter:
    def test_safe_content(self):
        assert OmissionFilter.is_safe("likes sushi") is True
        assert OmissionFilter.is_safe("works at Acme Corp") is True

    def test_protected_content(self):
        assert OmissionFilter.is_safe("my social security number is") is False
        assert OmissionFilter.is_safe("bank account details") is False
        assert OmissionFilter.is_safe("credit card number") is False

    def test_filter_entries(self):
        entries = [
            "- [stated] likes sushi",
            "- [stated] SSN is 123-45-6789",
            "- [stated] works at Acme",
        ]
        filtered = OmissionFilter.filter_entries(entries)
        assert len(filtered) == 2
        assert "SSN" not in " ".join(filtered)


# ---------------------------------------------------------------------------
# MemoryOperations
# ---------------------------------------------------------------------------

class TestMemoryOperations:
    @pytest.fixture
    def ops(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            ops = MemoryOperations(base_path=tmpdir)
            yield ops

    def test_read_nonexistent(self, ops):
        result = ops.memory_read("profile.md")
        assert result.success is False

    def test_write_and_read(self, ops):
        content = """---
name: test
description: Test file
---
- [stated] test fact
"""
        result = ops.memory_write("topics/test.md", content, if_version="new")
        assert result.success is True
        assert result.version_token is not None

        # Read back
        read = ops.memory_read("topics/test.md")
        assert read.success is True
        assert "test fact" in read.content
        assert read.version_token == result.version_token

    def test_write_already_exists(self, ops):
        ops.memory_write("topics/test.md", "---\nname: t\n---\ncontent", "new")
        result = ops.memory_write("topics/test.md", "---\nname: t\n---\ncontent2", "new")
        assert result.success is False
        assert "already exists" in result.error

    def test_append(self, ops):
        ops.memory_write(
            "topics/food.md",
            "---\nname: food\n---\n- [stated] likes sushi\n",
            "new",
        )
        read = ops.memory_read("topics/food.md")
        result = ops.memory_append(
            "topics/food.md",
            "- [stated] prefers tea",
            read.version_token,
        )
        assert result.success is True

        read2 = ops.memory_read("topics/food.md")
        assert "prefers tea" in read2.content

    def test_append_version_mismatch(self, ops):
        ops.memory_write(
            "topics/food.md",
            "---\nname: food\n---\n- [stated] likes sushi\n",
            "new",
        )
        result = ops.memory_append(
            "topics/food.md",
            "- [stated] prefers tea",
            "wrong_version_token",
        )
        assert result.success is False
        assert "Version mismatch" in result.error

    def test_append_duplicate(self, ops):
        ops.memory_write(
            "topics/food.md",
            "---\nname: food\n---\n- [stated] likes sushi\n",
            "new",
        )
        read = ops.memory_read("topics/food.md")
        result = ops.memory_append(
            "topics/food.md",
            "- [stated] likes sushi",
            read.version_token,
        )
        assert result.success is False
        assert "already exists" in result.error

    def test_append_protected_content(self, ops):
        ops.memory_write(
            "topics/test.md",
            "---\nname: test\n---\n- [stated] ok\n",
            "new",
        )
        read = ops.memory_read("topics/test.md")
        result = ops.memory_append(
            "topics/test.md",
            "- [stated] my social security number is",
            read.version_token,
        )
        assert result.success is False
        assert "protected" in result.error

    def test_str_replace(self, ops):
        ops.memory_write(
            "topics/test.md",
            "---\nname: test\n---\n- [stated] likes coffee\n",
            "new",
        )
        read = ops.memory_read("topics/test.md")
        result = ops.memory_str_replace(
            "topics/test.md",
            "coffee", "tea",
            read.version_token,
        )
        assert result.success is True

        read2 = ops.memory_read("topics/test.md")
        assert "likes tea" in read2.content
        assert "coffee" not in read2.content

    def test_delete(self, ops):
        ops.memory_write(
            "topics/test.md",
            "---\nname: test\n---\ncontent\n",
            "new",
        )
        read = ops.memory_read("topics/test.md")
        result = ops.memory_delete("topics/test.md", read.version_token)
        assert result.success is True
        assert ops.memory_read("topics/test.md").success is False

    def test_list(self, ops):
        ops.memory_write("topics/a.md", "---\nname: a\n---\ncontent\n", "new")
        ops.memory_write("topics/b.md", "---\nname: b\n---\ncontent\n", "new")
        ops.memory_write("areas/proj.md", "---\nname: proj\n---\ncontent\n", "new")

        all_files = ops.memory_list()
        assert len(all_files) == 3

        topics = ops.memory_list("topics")
        assert len(topics) == 2

    def test_concurrency_control(self, ops):
        """Simulate two agents writing concurrently."""
        ops.memory_write("topics/food.md", "---\nname: food\n---\n- [stated] likes sushi\n", "new")

        # Agent A reads
        read_a = ops.memory_read("topics/food.md")
        # Agent B reads
        read_b = ops.memory_read("topics/food.md")
        assert read_a.version_token == read_b.version_token

        # Agent A writes first
        result_a = ops.memory_append("topics/food.md", "- [stated] likes ramen", read_a.version_token)
        assert result_a.success is True

        # Agent B tries to write with old token
        result_b = ops.memory_append("topics/food.md", "- [stated] likes udon", read_b.version_token)
        assert result_b.success is False
        assert "Version mismatch" in result_b.error
        # Agent B receives the new version token
        assert result_b.version_token is not None
        assert result_b.version_token != read_b.version_token
