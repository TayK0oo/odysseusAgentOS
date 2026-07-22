"""
Unified Context Manager (§5.2).

Coordinates the 3 context management techniques:
    1. Compaction — auto-summarize when near token limit
    2. Structured notepad — externalize plan/decisions to persistent storage
    3. Isolation — filter context per sub-agent role

This facade ties together compaction, budget, filter, and notepad.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class ContextBudget:
    """Token budget for a conversation window."""
    max_tokens: int = 128000
    used_tokens: int = 0
    compaction_threshold: float = 0.80  # trigger at 80% usage
    last_compaction_at: Optional[str] = None
    compaction_count: int = 0

    @property
    def usage_ratio(self) -> float:
        if self.max_tokens == 0:
            return 0.0
        return self.used_tokens / self.max_tokens

    @property
    def should_compact(self) -> bool:
        return self.usage_ratio >= self.compaction_threshold

    @property
    def remaining(self) -> int:
        return max(0, self.max_tokens - self.used_tokens)


@dataclass
class ContextFilter:
    """Filters context entries by relevance to the current task."""
    max_entries: int = 10  # quota
    phase: Optional[str] = None
    role: Optional[str] = None

    def filter(self, entries: list[dict], query_embedding: Optional[list[float]] = None) -> list[dict]:
        """Filter and limit context entries.

        In production: semantic similarity + phase filter + quota.
        """
        filtered = entries
        if self.phase:
            filtered = [e for e in filtered if e.get("phase") in (None, self.phase)]
        return filtered[:self.max_entries]


class ContextManager:
    """Unified facade for context management.

    Coordinates compaction, filtering, notepad, and thread tracking.
    Works with the Thought Bus — subscribes to KNOW and PLAN phases.
    """

    def __init__(self, max_tokens: int = 128000):
        self.budget = ContextBudget(max_tokens=max_tokens)
        self.filter = ContextFilter()
        self._notepad: list[dict] = []  # structured decisions
        self._threads: dict[str, list[str]] = {}  # session_id → [decision_ids]

    # ------------------------------------------------------------------
    # Compaction
    # ------------------------------------------------------------------

    def track_tokens(self, tokens: int) -> None:
        self.budget.used_tokens += tokens

    def needs_compaction(self) -> bool:
        return self.budget.should_compact

    async def compact(
        self,
        history: list[dict],
        summarize_fn=None,
    ) -> list[dict]:
        """Compact conversation history — keep decisions, summarize the rest."""
        from datetime import datetime, timezone
        
        self.budget.last_compaction_at = datetime.now(timezone.utc).isoformat()
        self.budget.compaction_count += 1
        self.budget.used_tokens = 0  # reset after compaction

        if summarize_fn is None:
            # Default: keep the last 5 turns, summarize earlier turns
            if len(history) > 5:
                early = history[:-5]
                recent = history[-5:]
                summary = f"[{len(early)} earlier turns summarized]"
                return [{"role": "system", "content": summary}] + recent
            return history

        return await summarize_fn(history)

    # ------------------------------------------------------------------
    # Notepad
    # ------------------------------------------------------------------

    def note_decision(self, session_id: str, decision: str, context: dict = None) -> str:
        """Record a decision in the structured notepad."""
        from datetime import datetime, timezone
        import uuid

        decision_id = str(uuid.uuid4())[:8]
        entry = {
            "id": decision_id,
            "decision": decision,
            "context": context or {},
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
        self._notepad.append(entry)

        if session_id not in self._threads:
            self._threads[session_id] = []
        self._threads[session_id].append(decision_id)

        return decision_id

    def get_decisions(self, session_id: Optional[str] = None) -> list[dict]:
        """Get recorded decisions, optionally filtered by session."""
        if session_id is None:
            return self._notepad
        thread_ids = self._threads.get(session_id, [])
        return [d for d in self._notepad if d["id"] in thread_ids]

    # ------------------------------------------------------------------
    # Thread tracking
    # ------------------------------------------------------------------

    def get_thread(self, session_id: str) -> list[str]:
        """Get the decision thread for a session."""
        return self._threads.get(session_id, [])

    # ------------------------------------------------------------------
    # Context assembly
    # ------------------------------------------------------------------

    def build_context(
        self,
        task: str,
        phase: str,
        memory_entries: list[dict],
        preferences: Optional[dict] = None,
        decisions: Optional[list[dict]] = None,
    ) -> dict:
        """Build the context payload for an agent invocation.

        Returns a structured dict with filtered context, ready to inject
        into the LLM prompt.
        """
        # Filter memory entries by phase and quota
        self.filter.phase = phase
        relevant_memory = self.filter.filter(memory_entries)

        context = {
            "task": task,
            "phase": phase,
            "memory": relevant_memory,
            "preferences": preferences or {},
            "decisions": decisions or self._notepad[-5:],  # last 5 decisions
            "tokens_used": self.budget.used_tokens,
            "tokens_remaining": self.budget.remaining,
            "should_compact": self.budget.should_compact,
        }

        return context
