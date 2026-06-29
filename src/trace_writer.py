"""
trace_writer.py

Thread-safe JSONL trace writer for the agent loop.
Writes structured traces to data/traces/YYYY-MM-DD.jsonl.

Each line is a JSON object with:
  ts, run_id, session_id, tool, risk_level, args_summary,
  permission_decision, outcome, cost_tokens, duration_ms
"""

from __future__ import annotations

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
# Global run_id — one UUID per process start
# ---------------------------------------------------------------------------
_RUN_ID = str(uuid.uuid4())

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
            "run_id": _RUN_ID,
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


def current_run_id() -> str:
    """Return the current process-level run UUID."""
    return _RUN_ID
