"""
trace_writer.py

Thread-safe JSONL trace writer for the agent loop.
Writes structured traces to data/traces/YYYY-MM-DD.jsonl.

Each line is a JSON object with:
  ts, run_id, session_id, tool, risk_level, args_summary,
  permission_decision, outcome, cost_tokens, duration_ms
"""

from __future__ import annotations

import contextvars
import json
import logging
import os
import threading
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# run_id — correlation id shared by every trace/metric of a single run.
#
# A "run" = one agent-loop invocation (one chat turn or one orchestrated
# dispatch). It is held in a ContextVar so concurrent requests (each its own
# asyncio task) get an isolated id without threading it through call sites.
# The module-level default keeps pre-scope behaviour byte-compatible: until a
# caller opens a run scope, current_run_id() returns this single process id
# exactly like the old constant did.
# ---------------------------------------------------------------------------
_PROCESS_RUN_ID = str(uuid.uuid4())
_run_id_var: contextvars.ContextVar[str] = contextvars.ContextVar(
    "trace_run_id", default=_PROCESS_RUN_ID
)

# ---------------------------------------------------------------------------
# Thread-safe write lock (per-file)
# ---------------------------------------------------------------------------
_locks: dict[str, threading.Lock] = {}
_locks_mutex = threading.Lock()


def _get_lock(path: str) -> threading.Lock:
    with _locks_mutex:
        if path not in _locks:
            _locks[path] = threading.Lock()
        return _locks[path]


# ---------------------------------------------------------------------------
# Traces directory
# ---------------------------------------------------------------------------

def _traces_dir() -> Path:
    """Return the traces directory, creating it if needed."""
    try:
        from src.constants import DATA_DIR
        base = Path(DATA_DIR)
    except Exception:
        base = Path(__file__).parent.parent / "data"
    traces = base / "traces"
    traces.mkdir(parents=True, exist_ok=True)
    return traces


def _today_file() -> Path:
    """Return today's JSONL trace file path."""
    today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    return _traces_dir() / f"{today}.jsonl"


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def write_trace(
    *,
    tool: str,
    risk_level: str,
    args_summary: str,
    permission_decision: str,  # "auto_approved" | "gate_required" | "approved" | "rejected"
    outcome: str,              # "success" | "error" | "rejected"
    session_id: Optional[str] = None,
    cost_tokens: int = 0,
    duration_ms: int = 0,
    extra: Optional[dict] = None,
) -> None:
    """Write a single trace entry to today's JSONL file.

    Thread-safe. Never raises — errors are logged at WARNING level so they
    never break the agent loop.
    """
    try:
        record = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "run_id": _run_id_var.get(),
            "session_id": session_id or "",
            "tool": tool,
            "risk_level": risk_level,
            "args_summary": args_summary,
            "permission_decision": permission_decision,
            "outcome": outcome,
            "cost_tokens": cost_tokens,
            "duration_ms": duration_ms,
        }
        if extra:
            record.update(extra)

        line = json.dumps(record, ensure_ascii=False) + "\n"
        fpath = str(_today_file())
        lock = _get_lock(fpath)
        with lock:
            with open(fpath, "a", encoding="utf-8") as f:
                f.write(line)
    except Exception as exc:
        logger.warning("trace_writer: failed to write trace for tool=%s: %s", tool, exc)


# ---------------------------------------------------------------------------
# Unified token accounting (M3.X) — authoritative per-run token totals.
#
# Tokens were historically counted in ~5 divergent places (cartography risk #1).
# The single WRITER OF TRUTH is the final metrics dict from
# agent_loop._compute_final_metrics (it lands in chat_messages.meta_data). That
# writer publishes its authoritative per-run totals here, keyed by run_id, so
# secondary writers (e.g. routes.chat_helpers.accumulate_token_usage) can
# RECONCILE against them instead of writing their own numbers.
#
# Kill-switch: ODYSSEUS_UNIFIED_TOKENS, default OFF (mirrors
# orchestrator.phase_tracker.tracker_enabled). When OFF, nobody reads this
# registry and behaviour is byte-identical to before. Best-effort throughout:
# recording/reading must never raise into the agent loop.
# ---------------------------------------------------------------------------
_RUN_TOKENS_CAP = 4096  # bound memory: forget oldest runs beyond this many
_run_tokens: "dict[str, dict]" = {}
_run_tokens_lock = threading.Lock()


def unified_tokens_enabled() -> bool:
    """OFF unless ODYSSEUS_UNIFIED_TOKENS is set to a truthy value."""
    val = os.getenv("ODYSSEUS_UNIFIED_TOKENS", "off").strip().lower()
    return val in {"on", "1", "true", "yes"}


def record_run_tokens(run_id, input_tokens=0, output_tokens=0) -> None:
    """Publish the authoritative token totals for a run (writer of truth).

    Last-write-wins: the final metrics dict holds the run TOTAL, so re-recording
    overwrites rather than accumulating. Never raises.
    """
    try:
        if not run_id:
            return
        entry = {
            "input_tokens": int(input_tokens or 0),
            "output_tokens": int(output_tokens or 0),
        }
        with _run_tokens_lock:
            _run_tokens[str(run_id)] = entry
            if len(_run_tokens) > _RUN_TOKENS_CAP:
                # dict preserves insertion order — drop the oldest.
                for old in list(_run_tokens.keys())[: len(_run_tokens) - _RUN_TOKENS_CAP]:
                    _run_tokens.pop(old, None)
    except Exception as exc:
        logger.warning("trace_writer: failed to record run tokens for run=%s: %s", run_id, exc)


def get_run_tokens(run_id) -> Optional[dict]:
    """Return {'input_tokens', 'output_tokens'} for a run, or None if unknown.

    Never raises.
    """
    try:
        if not run_id:
            return None
        with _run_tokens_lock:
            entry = _run_tokens.get(str(run_id))
        return dict(entry) if entry is not None else None
    except Exception:
        return None


def current_run_id() -> str:
    """Return the run id in effect for the current context."""
    return _run_id_var.get()


def set_run_id(run_id: str) -> str:
    """Set the run id for the current context. Returns the value set."""
    _run_id_var.set(run_id)
    return run_id


def new_run_id() -> str:
    """Start a fresh run: generate a new UUID, make it current, return it."""
    return set_run_id(str(uuid.uuid4()))
