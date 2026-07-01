# M2-P1 — Orchestrator Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the native business-logic package `src/orchestrator/` that parses `.opencode/agents/*.md` into `AgentSpec` objects, discovers them in a registry, and resolves each agent's model through the existing `ModelRouter` — giving `llm_router.py` its first live call-site.

**Architecture:** Three focused modules. `spec.py` parses one Markdown agent file (YAML frontmatter + body prompt) into an immutable `AgentSpec`. `registry.py` scans the agents directory and exposes lookup. `dispatcher.py` (P1 slice) resolves the model string for a spec via `ModelRouter`. No session-spawning yet — that lands in M2-P2. Everything is pure, in-process, and unit-testable with no network.

**Tech Stack:** Python 3.12, `dataclasses`, `pyyaml` (already in `requirements.txt`), existing `src/llm_router.py::ModelRouter`, pytest.

**Reference:** Design spec `docs/superpowers/specs/2026-07-01-odysseus-integration-milestone2-design.md` (Section 1). Tracking `.planning/INTEGRATION-TRACKING.md` (Bloc A). Vision `.planning/AGENT-OS-ANALYSE-PROFONDE.md` (§A.6, §5.3).

---

## File Structure

- Create: `src/orchestrator/__init__.py` — package exports (`AgentSpec`, `parse_agent_spec`, `AgentRegistry`, `resolve_model`)
- Create: `src/orchestrator/spec.py` — `AgentSpec` dataclass + `parse_agent_spec(path)`
- Create: `src/orchestrator/registry.py` — `AgentRegistry` discovery/lookup
- Create: `src/orchestrator/dispatcher.py` — `resolve_model(spec, router=None)` (P1 slice)
- Create: `tests/test_orchestrator_spec.py`
- Create: `tests/test_orchestrator_registry.py`
- Create: `tests/test_orchestrator_dispatcher.py`

**Ground truth about the agent files (verified 2026-07-01):**
- Most `.opencode/agents/*.md` start with `---` frontmatter containing `name`, `description` (may be a multiline `>` block), and optionally a `tools:` list.
- Two files (`design-extract.md`, `open-design.md`) have **no frontmatter** — they begin directly with `# Title`. The parser MUST handle this by deriving `name` from the filename stem and treating the whole file as the prompt body.
- No file currently sets a `model:` key, so `resolve_model` will normally fall through to the router.

---

## Task 1: `AgentSpec` dataclass + `parse_agent_spec`

**Files:**
- Create: `src/orchestrator/spec.py`
- Test: `tests/test_orchestrator_spec.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_orchestrator_spec.py
"""Tests for src/orchestrator/spec.py — parsing .opencode agent Markdown."""
import textwrap
from src.orchestrator.spec import AgentSpec, parse_agent_spec


def _write(tmp_path, name, content):
    p = tmp_path / name
    p.write_text(textwrap.dedent(content), encoding="utf-8")
    return p


def test_parses_frontmatter_name_and_description(tmp_path):
    p = _write(tmp_path, "gsd-planner.md", """\
        ---
        name: gsd-planner
        description: Creates phase plans from goals.
        ---
        # Role

        The planner does X.
        """)
    spec = parse_agent_spec(p)
    assert isinstance(spec, AgentSpec)
    assert spec.name == "gsd-planner"
    assert spec.description == "Creates phase plans from goals."
    assert "The planner does X." in spec.prompt
    assert spec.prompt.startswith("# Role")
    assert spec.tools == []
    assert spec.model is None
    assert spec.source_path == str(p)


def test_parses_tools_list(tmp_path):
    p = _write(tmp_path, "security-audit.md", """\
        ---
        name: security-audit
        description: STRIDE audit.
        tools:
          - read_file
          - grep
        ---
        # Role
        Audit stuff.
        """)
    spec = parse_agent_spec(p)
    assert spec.tools == ["read_file", "grep"]


def test_multiline_folded_description(tmp_path):
    p = _write(tmp_path, "constitution.md", """\
        ---
        name: constitution
        description: >
          Constitution agent — encodes the invariants.
          Run before any agentic workflow.
        ---
        # Body
        """)
    spec = parse_agent_spec(p)
    assert spec.description.startswith("Constitution agent")
    assert "Run before any agentic workflow." in spec.description


def test_no_frontmatter_derives_name_from_filename(tmp_path):
    p = _write(tmp_path, "open-design.md", """\
        # Open Design Agent

        Génère des artefacts de design depuis un brief.
        """)
    spec = parse_agent_spec(p)
    assert spec.name == "open-design"
    assert spec.description == ""
    assert spec.prompt.startswith("# Open Design Agent")
    assert spec.tools == []


def test_explicit_model_key_is_read(tmp_path):
    p = _write(tmp_path, "x.md", """\
        ---
        name: x
        description: d
        model: openai/kimi-k2.7-code
        ---
        body
        """)
    spec = parse_agent_spec(p)
    assert spec.model == "openai/kimi-k2.7-code"


def test_agentspec_is_frozen(tmp_path):
    p = _write(tmp_path, "x.md", "---\nname: x\ndescription: d\n---\nbody\n")
    spec = parse_agent_spec(p)
    try:
        spec.name = "mutated"
        raised = False
    except Exception:
        raised = True
    assert raised, "AgentSpec must be immutable (frozen dataclass)"
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python -m pytest tests/test_orchestrator_spec.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.orchestrator'`

- [ ] **Step 3: Write the implementation**

```python
# src/orchestrator/spec.py
"""Parse a .opencode agent Markdown file into an immutable AgentSpec.

An agent file is either:
  1. YAML frontmatter delimited by leading '---' lines, followed by a Markdown
     body used as the system prompt; or
  2. A bare Markdown file with no frontmatter — the whole file is the prompt and
     the agent name is derived from the filename stem.

This module has no side effects and no network calls.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional

import yaml


@dataclass(frozen=True)
class AgentSpec:
    """Immutable description of one orchestrated agent."""

    name: str
    description: str = ""
    prompt: str = ""
    tools: List[str] = field(default_factory=list)
    model: Optional[str] = None
    mode: Optional[str] = None
    source_path: str = ""


def _split_frontmatter(text: str) -> tuple[dict, str]:
    """Return (frontmatter_dict, body). If there is no leading '---' block,
    return ({}, text)."""
    if not text.lstrip().startswith("---"):
        return {}, text

    # Normalise leading whitespace/newlines then split on the frontmatter fences.
    stripped = text.lstrip("\n")
    if not stripped.startswith("---"):
        return {}, text

    parts = stripped.split("\n")
    # parts[0] == '---'. Find the closing '---'.
    for idx in range(1, len(parts)):
        if parts[idx].strip() == "---":
            fm_raw = "\n".join(parts[1:idx])
            body = "\n".join(parts[idx + 1:])
            try:
                fm = yaml.safe_load(fm_raw) or {}
            except yaml.YAMLError:
                fm = {}
            if not isinstance(fm, dict):
                fm = {}
            return fm, body.lstrip("\n")
    # No closing fence — treat as bodyless-frontmatter edge case: no frontmatter.
    return {}, text


def parse_agent_spec(path) -> AgentSpec:
    """Parse one agent Markdown file into an AgentSpec."""
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    fm, body = _split_frontmatter(text)

    name = fm.get("name") or p.stem
    description = fm.get("description") or ""
    if isinstance(description, str):
        description = description.strip()

    tools_raw = fm.get("tools") or []
    tools = [str(t).strip() for t in tools_raw] if isinstance(tools_raw, list) else []

    return AgentSpec(
        name=str(name),
        description=str(description),
        prompt=body.strip(),
        tools=tools,
        model=fm.get("model"),
        mode=fm.get("mode"),
        source_path=str(p),
    )
```

Also create the package init (minimal for now; expanded in later tasks):

```python
# src/orchestrator/__init__.py
"""Native orchestration layer for Odysseus (Milestone 2)."""
from src.orchestrator.spec import AgentSpec, parse_agent_spec

__all__ = ["AgentSpec", "parse_agent_spec"]
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_orchestrator_spec.py -v`
Expected: PASS (6 passed)

- [ ] **Step 5: Commit**

```bash
git add src/orchestrator/__init__.py src/orchestrator/spec.py tests/test_orchestrator_spec.py
git commit -m "feat(orchestrator): parse .opencode agent md into AgentSpec"
```

---

## Task 2: `AgentRegistry` discovery and lookup

**Files:**
- Create: `src/orchestrator/registry.py`
- Modify: `src/orchestrator/__init__.py`
- Test: `tests/test_orchestrator_registry.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_orchestrator_registry.py
"""Tests for src/orchestrator/registry.py — discovery of agent specs."""
import textwrap
import pytest
from src.orchestrator.registry import AgentRegistry


def _agent(dir_path, filename, name):
    (dir_path / filename).write_text(
        textwrap.dedent(f"""\
            ---
            name: {name}
            description: desc for {name}
            ---
            # Role
            Body of {name}.
            """),
        encoding="utf-8",
    )


def test_discovers_all_agents(tmp_path):
    _agent(tmp_path, "a.md", "alpha")
    _agent(tmp_path, "b.md", "beta")
    reg = AgentRegistry(tmp_path)
    reg.discover()
    assert set(reg.list_names()) == {"alpha", "beta"}


def test_get_returns_spec(tmp_path):
    _agent(tmp_path, "a.md", "alpha")
    reg = AgentRegistry(tmp_path)
    reg.discover()
    spec = reg.get("alpha")
    assert spec is not None
    assert spec.name == "alpha"
    assert "Body of alpha." in spec.prompt


def test_get_unknown_returns_none(tmp_path):
    reg = AgentRegistry(tmp_path)
    reg.discover()
    assert reg.get("nope") is None


def test_missing_directory_yields_empty_registry(tmp_path):
    reg = AgentRegistry(tmp_path / "does-not-exist")
    reg.discover()
    assert reg.list_names() == []


def test_broken_file_is_skipped_not_fatal(tmp_path):
    _agent(tmp_path, "good.md", "good")
    # A file that cannot be read as expected must not crash discovery.
    (tmp_path / "bad.md").write_bytes(b"\xff\xfe not utf-8 \xff")
    reg = AgentRegistry(tmp_path)
    reg.discover()
    assert "good" in reg.list_names()


def test_discover_is_idempotent(tmp_path):
    _agent(tmp_path, "a.md", "alpha")
    reg = AgentRegistry(tmp_path)
    reg.discover()
    reg.discover()
    assert reg.list_names().count("alpha") == 1
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python -m pytest tests/test_orchestrator_registry.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.orchestrator.registry'`

- [ ] **Step 3: Write the implementation**

```python
# src/orchestrator/registry.py
"""Discover .opencode agent specs from a directory and expose lookup.

Discovery is defensive: a single malformed file must never abort the scan.
"""
from __future__ import annotations

import logging
from pathlib import Path
from typing import Dict, List, Optional

from src.orchestrator.spec import AgentSpec, parse_agent_spec

logger = logging.getLogger(__name__)

# Default location relative to project root (parent of src/).
_PROJECT_ROOT = Path(__file__).parent.parent.parent
_DEFAULT_AGENTS_DIR = _PROJECT_ROOT / ".opencode" / "agents"


class AgentRegistry:
    """Loads and indexes AgentSpec objects by name."""

    def __init__(self, agents_dir=None):
        self._dir = Path(agents_dir) if agents_dir is not None else _DEFAULT_AGENTS_DIR
        self._specs: Dict[str, AgentSpec] = {}

    def discover(self) -> "AgentRegistry":
        """(Re)scan the agents directory. Idempotent — replaces the index."""
        specs: Dict[str, AgentSpec] = {}
        if not self._dir.is_dir():
            logger.warning("[AgentRegistry] agents dir not found: %s", self._dir)
            self._specs = specs
            return self

        for md in sorted(self._dir.glob("*.md")):
            try:
                spec = parse_agent_spec(md)
            except Exception as exc:  # never let one file break discovery
                logger.warning("[AgentRegistry] skipping %s: %s", md.name, exc)
                continue
            specs[spec.name] = spec

        self._specs = specs
        logger.info("[AgentRegistry] discovered %d agents from %s", len(specs), self._dir)
        return self

    def get(self, name: str) -> Optional[AgentSpec]:
        return self._specs.get(name)

    def list_names(self) -> List[str]:
        return sorted(self._specs.keys())

    def all(self) -> List[AgentSpec]:
        return [self._specs[n] for n in self.list_names()]
```

Update the package exports:

```python
# src/orchestrator/__init__.py
"""Native orchestration layer for Odysseus (Milestone 2)."""
from src.orchestrator.spec import AgentSpec, parse_agent_spec
from src.orchestrator.registry import AgentRegistry

__all__ = ["AgentSpec", "parse_agent_spec", "AgentRegistry"]
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_orchestrator_registry.py -v`
Expected: PASS (6 passed)

- [ ] **Step 5: Verify against the real agents directory**

Run: `python -c "from src.orchestrator.registry import AgentRegistry; r=AgentRegistry().discover(); print(r.list_names())"`
Expected: a list including `constitution`, `gsd-planner`, `security-audit`, `open-design`, `design-extract` (12 names total).

- [ ] **Step 6: Commit**

```bash
git add src/orchestrator/registry.py src/orchestrator/__init__.py tests/test_orchestrator_registry.py
git commit -m "feat(orchestrator): AgentRegistry discovers .opencode agents"
```

---

## Task 3: `resolve_model` — first live ModelRouter call-site

**Files:**
- Create: `src/orchestrator/dispatcher.py`
- Modify: `src/orchestrator/__init__.py`
- Test: `tests/test_orchestrator_dispatcher.py`

Rationale: `INTEGRATION-TRACKING.md` records that `ModelRouter` has **zero live call-sites**. This task creates the first one. `resolve_model` returns the agent's explicit `model` if the spec declares one; otherwise it asks the router to route by stage/intent. The router is injected so the test needs no network and no config file.

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_orchestrator_dispatcher.py
"""Tests for src/orchestrator/dispatcher.py::resolve_model."""
from src.orchestrator.spec import AgentSpec
from src.orchestrator.dispatcher import resolve_model


class _FakeRouter:
    def __init__(self, answer="openai/deepseek-v4-flash"):
        self.answer = answer
        self.calls = []

    def route(self, intent_category, stage=None):
        self.calls.append((intent_category, stage))
        return self.answer


def test_explicit_spec_model_wins_and_router_not_called():
    spec = AgentSpec(name="x", model="openai/kimi-k2.7-code")
    router = _FakeRouter()
    result = resolve_model(spec, router=router, stage="execution")
    assert result == "openai/kimi-k2.7-code"
    assert router.calls == []  # explicit model short-circuits the router


def test_falls_back_to_router_when_no_model():
    spec = AgentSpec(name="x", model=None)
    router = _FakeRouter(answer="openai/glm-5.2")
    result = resolve_model(spec, router=router, stage="planning")
    assert result == "openai/glm-5.2"
    assert router.calls == [("utility", "planning")]


def test_router_stage_defaults_to_none():
    spec = AgentSpec(name="x", model=None)
    router = _FakeRouter()
    resolve_model(spec, router=router)
    assert router.calls == [("utility", None)]


def test_no_router_and_no_model_returns_none():
    spec = AgentSpec(name="x", model=None)
    assert resolve_model(spec, router=None) is None
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python -m pytest tests/test_orchestrator_dispatcher.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.orchestrator.dispatcher'`

- [ ] **Step 3: Write the implementation**

```python
# src/orchestrator/dispatcher.py
"""Model resolution for an agent spec (Milestone 2, P1 slice).

This is the first live call-site of src/llm_router.py::ModelRouter. Actual
sub-session spawning lands in M2-P2; here we only resolve WHICH model an agent
should run on.
"""
from __future__ import annotations

import logging
from typing import Optional

from src.orchestrator.spec import AgentSpec

logger = logging.getLogger(__name__)


def resolve_model(spec: AgentSpec, router=None, stage: Optional[str] = None) -> Optional[str]:
    """Resolve the LiteLLM model string for an agent.

    Priority:
      1. spec.model (explicit override in the agent frontmatter)
      2. router.route(intent_category="utility", stage=stage)
      3. None (caller falls back to the session's default model)
    """
    if spec.model:
        logger.debug("[dispatcher] %s uses explicit model %s", spec.name, spec.model)
        return spec.model

    if router is not None:
        model = router.route(intent_category="utility", stage=stage)
        logger.info("[dispatcher] %s routed to %s (stage=%s)", spec.name, model, stage)
        return model

    logger.debug("[dispatcher] %s has no model and no router; returning None", spec.name)
    return None
```

Update the package exports:

```python
# src/orchestrator/__init__.py
"""Native orchestration layer for Odysseus (Milestone 2)."""
from src.orchestrator.spec import AgentSpec, parse_agent_spec
from src.orchestrator.registry import AgentRegistry
from src.orchestrator.dispatcher import resolve_model

__all__ = ["AgentSpec", "parse_agent_spec", "AgentRegistry", "resolve_model"]
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_orchestrator_dispatcher.py -v`
Expected: PASS (4 passed)

- [ ] **Step 5: Run the full orchestrator suite together**

Run: `python -m pytest tests/test_orchestrator_spec.py tests/test_orchestrator_registry.py tests/test_orchestrator_dispatcher.py -v`
Expected: PASS (16 passed)

- [ ] **Step 6: Commit**

```bash
git add src/orchestrator/dispatcher.py src/orchestrator/__init__.py tests/test_orchestrator_dispatcher.py
git commit -m "feat(orchestrator): resolve_model as first live ModelRouter call-site"
```

---

## Task 4: Update tracking doc — Bloc A partial

**Files:**
- Modify: `.planning/INTEGRATION-TRACKING.md`

- [ ] **Step 1: Mark the P1 deliverables in the tracker**

In `.planning/INTEGRATION-TRACKING.md`, under the Bloc A section of the "Tracker d'avancement", check the boxes for: orchestrator package created, `.opencode` agents discovered natively, and `resolve_model` wired to `ModelRouter`. Add a one-line note: "M2-P1 livré — registry+spec+dispatcher, 16 tests verts, premier call-site live du router (dispatcher.resolve_model)."

Do NOT mark the router 🟢 globally yet — it is only reached via `resolve_model`, not yet from the live loop. That upgrade happens in M2-P2/P3 when the loop calls the dispatcher.

- [ ] **Step 2: Commit**

```bash
git add .planning/INTEGRATION-TRACKING.md
git commit -m "docs(tracking): M2-P1 orchestrator foundation delivered"
```

---

## Self-Review

**Spec coverage (Section 1 of the design):**
- `registry.py` (discover/lookup) → Task 2 ✅
- `spec.py` (parse .md → AgentSpec) → Task 1 ✅
- `dispatcher.py` (model resolution, first router call-site) → Task 3 ✅
- `loop.py` / `phases.py` → deferred to M2-P2 (Section 2 of the design), intentionally out of this plan's scope.

**Placeholder scan:** No TBD/TODO; every code step contains complete code. ✅

**Type consistency:** `AgentSpec` fields (`name`, `description`, `prompt`, `tools`, `model`, `mode`, `source_path`) are defined once in Task 1 and used identically in Tasks 2–3. `resolve_model(spec, router, stage)` signature matches its tests. `AgentRegistry` methods (`discover`, `get`, `list_names`, `all`) match their tests. ✅

**Scope:** Self-contained, pure-Python, no network. Produces working, tested software (the parsing + discovery + model-resolution foundation) that later phases build on. ✅
