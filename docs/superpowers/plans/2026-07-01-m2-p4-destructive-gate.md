# M2-P4 — Real Destructive Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the destructive-action detection at `tool_execution.py:553` from "log only, then execute anyway" into a **real gate** that blocks catastrophic shell commands before they run — without breaking legitimate explicit destructive tools (`delete_file`, `stop_served_model`, …).

**Architecture:** A pure, well-tested function `should_block_destructive(tool_name, tool_args)` in `src/orchestrator/gate.py` returns a block reason string for catastrophic shell commands (detected via the existing `risk_classifier.classify_bash` patterns: `rm -rf`, fork bomb, `mkfs`, `dd if=`, `drop database`, `git push --force`, …) and `None` otherwise. It deliberately does NOT block the explicit destructive TOOLS from `TOOL_RISK_MAP` (those are user-facing operations with their own semantics). A one-line env switch `ODYSSEUS_DESTRUCTIVE_GATE` (default `on`) allows disabling. The gate is wired into the central choke point `execute_tool_block` so it covers more execution paths than the partial `command_validator`.

**Why safe:** Only catastrophic shell patterns are blocked. `delete_file`/`remove_dir`/`stop_served_model` still run (they are explicit user tools). Normal chat is unaffected. The gate returns the same `(desc, {"error":..., "blocked":True})` shape the phase-lock already uses, so the loop handles it identically.

**Tech Stack:** Python 3.12, existing `src/risk_classifier.py` (`classify_bash`, `RiskLevel`), pytest. No network.

**Reference:** Design spec Section 2 (CLASSIFY: "Gate destructif réel (aujourd'hui il ne fait que logger)"). Tracking Bloc A. Current behavior evidence: `src/tool_execution.py:553-560` sets `_permission_decision="gate_required"` but falls through to execution at line 596.

---

## File Structure

- Create: `src/orchestrator/gate.py` — `should_block_destructive()` + `gate_enabled()`
- Modify: `src/tool_execution.py:553-560` — enforce the gate (return blocked result)
- Modify: `src/orchestrator/__init__.py` — export `should_block_destructive`
- Test: `tests/test_orchestrator_gate.py`

---

## Task 1: `should_block_destructive` pure function

**Files:**
- Create: `src/orchestrator/gate.py`
- Test: `tests/test_orchestrator_gate.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_orchestrator_gate.py
"""Tests for src/orchestrator/gate.py — catastrophic command gate."""
import os
import pytest
from src.orchestrator.gate import should_block_destructive, gate_enabled


def test_blocks_rm_rf():
    reason = should_block_destructive("bash", {"content": "rm -rf /"})
    assert reason is not None
    assert "bloqu" in reason.lower() or "block" in reason.lower()


def test_blocks_fork_bomb():
    assert should_block_destructive("bash", {"content": ":(){:|:&};:"}) is not None


def test_blocks_drop_database():
    assert should_block_destructive("python", {"content": "cur.execute('drop database prod')"}) is not None


def test_blocks_dd_and_mkfs():
    assert should_block_destructive("bash", {"content": "dd if=/dev/zero of=/dev/sda"}) is not None
    assert should_block_destructive("run_command", {"command": "mkfs.ext4 /dev/sdb"}) is not None


def test_allows_normal_bash():
    assert should_block_destructive("bash", {"content": "ls -la && pytest"}) is None


def test_does_not_block_explicit_delete_tool():
    # delete_file is DESTRUCTIVE in the tool map, but it is an explicit user tool
    # with its own semantics — the catastrophic-command gate must NOT block it.
    assert should_block_destructive("delete_file", {"path": "notes/tmp.md"}) is None
    assert should_block_destructive("stop_served_model", {"id": "x"}) is None


def test_reads_command_key_too():
    assert should_block_destructive("bash", {"command": "rm -rf /var"}) is not None


def test_gate_can_be_disabled_by_env(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_DESTRUCTIVE_GATE", "off")
    assert gate_enabled() is False
    # When disabled, should_block_destructive still classifies but callers skip it;
    # the function itself is pure and unaffected — enforcement is gated by gate_enabled().
    assert should_block_destructive("bash", {"content": "rm -rf /"}) is not None


def test_gate_enabled_default_true(monkeypatch):
    monkeypatch.delenv("ODYSSEUS_DESTRUCTIVE_GATE", raising=False)
    assert gate_enabled() is True


def test_gate_enabled_explicit_on(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_DESTRUCTIVE_GATE", "on")
    assert gate_enabled() is True
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `python -m pytest tests/test_orchestrator_gate.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.orchestrator.gate'`

- [ ] **Step 3: Write the implementation**

```python
# src/orchestrator/gate.py
"""Catastrophic-command gate (Milestone 2, P4).

The destructive detection in tool_execution.py historically only LOGGED and
then executed anyway. This module provides a real block decision for
catastrophic SHELL commands (rm -rf, fork bomb, mkfs, dd, drop database, ...)
using the existing risk_classifier patterns.

It intentionally does NOT block the explicit destructive TOOLS (delete_file,
remove_dir, stop_served_model, ...) — those are user-facing operations with
their own semantics; blocking them would break the product.
"""
from __future__ import annotations

import os
from typing import Optional

from src.risk_classifier import classify_bash, RiskLevel

_SHELL_TOOLS = {"bash", "python", "run_command"}


def gate_enabled() -> bool:
    """The gate is on unless ODYSSEUS_DESTRUCTIVE_GATE is set to a false-y value."""
    val = os.getenv("ODYSSEUS_DESTRUCTIVE_GATE", "on").strip().lower()
    return val not in {"off", "0", "false", "no"}


def _extract_command(tool_args) -> str:
    if isinstance(tool_args, str):
        return tool_args
    if isinstance(tool_args, dict):
        return str(tool_args.get("command") or tool_args.get("content") or "")
    return ""


def should_block_destructive(tool_name: str, tool_args) -> Optional[str]:
    """Return a block reason for catastrophic shell commands, else None.

    Pure function: does not read env. Enforcement callers should also check
    gate_enabled() to honour the kill-switch.
    """
    if tool_name not in _SHELL_TOOLS:
        return None
    command = _extract_command(tool_args)
    if not command:
        return None
    if classify_bash(command) == RiskLevel.DESTRUCTIVE:
        first = command.split("\n")[0].strip()[:120]
        return f"Commande catastrophique bloquée par le gate destructif : {first!r}"
    return None
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `python -m pytest tests/test_orchestrator_gate.py -v`
Expected: PASS (9 passed)

- [ ] **Step 5: Export it**

```python
# src/orchestrator/__init__.py — add
from src.orchestrator.gate import should_block_destructive, gate_enabled
```
(Add `"should_block_destructive"`, `"gate_enabled"` to `__all__`.)

- [ ] **Step 6: Commit**

```bash
git add src/orchestrator/gate.py src/orchestrator/__init__.py tests/test_orchestrator_gate.py
git commit -m "feat(orchestrator): catastrophic-command destructive gate"
```

---

## Task 2: Enforce the gate in `execute_tool_block`

**Files:**
- Modify: `src/tool_execution.py` (the destructive branch at lines 553-560)

- [ ] **Step 1: Replace the log-only destructive branch**

Current code (`src/tool_execution.py:553-560`):

```python
        if _risk == RiskLevel.DESTRUCTIVE:
            logger.warning(
                "⚠️ ACTION DESTRUCTIVE détectée : tool=%s summary=%s session=%s",
                _tool_name, _summary, session_id,
            )
            _permission_decision = "gate_required"
        else:
            _permission_decision = "auto_approved"
```

Replace with (keep the logging, ADD real enforcement for catastrophic shell commands):

```python
        if _risk == RiskLevel.DESTRUCTIVE:
            logger.warning(
                "⚠️ ACTION DESTRUCTIVE détectée : tool=%s summary=%s session=%s",
                _tool_name, _summary, session_id,
            )
            _permission_decision = "gate_required"
            # M2-P4: real gate — block catastrophic SHELL commands (rm -rf, fork
            # bomb, mkfs, dd, drop database, ...) before they execute. Explicit
            # destructive TOOLS (delete_file, ...) are NOT blocked here.
            try:
                from src.orchestrator.gate import should_block_destructive, gate_enabled
                if gate_enabled():
                    _block_reason = should_block_destructive(_tool_name, _tool_args)
                    if _block_reason:
                        logger.error(
                            "🛑 GATE DESTRUCTIF : %s (session=%s)", _block_reason, session_id
                        )
                        return (
                            "blocked by destructive gate",
                            {"error": f"🛑 GATE DESTRUCTIF : {_block_reason}", "blocked": True},
                        )
            except Exception:
                pass  # never break the loop on a gate error
        else:
            _permission_decision = "auto_approved"
```

Note: `_tool_args` here is `{"content": _content}` (set at line 548), which
`should_block_destructive` reads via the `content` key — matching the test
`test_blocks_rm_rf`.

- [ ] **Step 2: Verify the destructive gate blocks in a focused runtime check**

Run:
```bash
python -c "
from src.orchestrator.gate import should_block_destructive
print('rm -rf:', should_block_destructive('bash', {'content':'rm -rf /'}))
print('ls:', should_block_destructive('bash', {'content':'ls'}))
print('delete_file:', should_block_destructive('delete_file', {'path':'x'}))
"
```
Expected:
- `rm -rf:` a non-None block reason
- `ls:` `None`
- `delete_file:` `None`

- [ ] **Step 3: Run the existing tool_execution tests to ensure no regression**

Run: `python -m pytest tests/ -k "tool_execution or tool_exec or execute_tool" -q`
Expected: PASS (no regressions). If no such tests exist, run `python -c "import src.tool_execution"` to confirm the module still imports.

- [ ] **Step 4: Commit**

```bash
git add src/tool_execution.py
git commit -m "feat(tool-exec): enforce destructive gate for catastrophic shell commands"
```

---

## Task 3: Update tracking doc

**Files:**
- Modify: `.planning/INTEGRATION-TRACKING.md`

- [ ] **Step 1:** In Bloc A (or a new "Bloc A0 — Gate destructif réel (M2-P4)"), record: catastrophic
shell commands now BLOCKED at the central `execute_tool_block` choke point (was log-only). Note the
scope (shell patterns only; explicit delete tools unaffected) and the `ODYSSEUS_DESTRUCTIVE_GATE`
kill-switch. This closes the audit gap "risk_classifier logue, ne gate PAS".

- [ ] **Step 2: Commit**

```bash
git add .planning/INTEGRATION-TRACKING.md
git commit -m "docs(tracking): M2-P4 real destructive gate delivered"
```

---

## Self-Review

**Spec coverage (Section 2, CLASSIFY real gate):** `should_block_destructive` + enforcement at
`execute_tool_block` → Tasks 1–2 ✅.

**Placeholder scan:** No TBD/TODO; all code complete. ✅

**Type consistency:** `should_block_destructive(tool_name, tool_args) -> Optional[str]` and
`gate_enabled() -> bool` used identically in tests and in the tool_execution wiring. Reuses
`classify_bash`/`RiskLevel` from `risk_classifier.py` unchanged. Block-result shape
`(str, {"error":..., "blocked": True})` matches the existing phase-lock block path (tool_execution.py:584). ✅

**Scope:** One pure module + one focused edit at a proven choke point; guarded by try/except and an env
kill-switch; explicit delete tools preserved. Safe, tested, non-breaking. ✅
