"""
SFDSystemWiring — Single integration point for all SFD v3.0 modules.

Wires into the live Odysseus agent loop:
- Memory provenance (dual-write ChromaDB + MD)
- Durable execution (wrap tool calls)
- Preferences (inject into system prompt)
- Visual output (route responses)
- Context manager (compaction trigger)
- Missing principles (P5 divulgation, P6 echecs-regles, P12 contexte, P19 impact)

Called from app startup and agent loop.
"""

from __future__ import annotations

import logging
import os
from typing import Any, Callable, Optional

logger = logging.getLogger(__name__)

_wiring: Optional["SFDSystemWiring"] = None


class SFDSystemWiring:
    """Central wiring hub for all SFD v3.0 modules.

    Initialized once at app startup. Provides hooks for the agent loop.
    """

    def __init__(self):
        self._memory_bridge = None
        self._durable_engine = None
        self._pref_engine = None
        self._output_router = None
        self._context_mgr = None
        self._stats = {
            "facts_stored": 0,
            "facts_omitted": 0,
            "retries_handled": 0,
            "sagas_compensated": 0,
            "visuals_routed": 0,
            "compactions_done": 0,
            "rules_learned": 0,
        }

    # ------------------------------------------------------------------
    # MEMORY PROVENANCE (P16, P17, P18)
    # ------------------------------------------------------------------

    def wire_memory(self, base_path: str = "workspace/memory") -> None:
        from src.memory_provenance.live_bridge import MemoryBridge, set_memory_bridge
        self._memory_bridge = MemoryBridge(base_path=base_path)
        set_memory_bridge(self._memory_bridge)
        logger.info("[SFDSystemWiring] Memory provenance wired (%s)", base_path)

    def remember(self, domain: str, fact: str, tag: str = "[stated]") -> bool:
        if self._memory_bridge:
            ok = self._memory_bridge.remember(domain, fact, tag)
            if ok:
                self._stats["facts_stored"] += 1
            else:
                self._stats["facts_omitted"] += 1
            return ok
        return False

    def recall(self, domain: str) -> list[str]:
        if self._memory_bridge:
            return self._memory_bridge.recall(domain)
        return []

    # ------------------------------------------------------------------
    # DURABLE EXECUTION (P14)
    # ------------------------------------------------------------------

    def wire_durable(self, db_path: str = "data/durable.db") -> None:
        from src.durable_execution.engine import DurableEngine, durable_exec_enabled
        if not durable_exec_enabled():
            return
        import asyncio
        self._durable_engine = DurableEngine(db_path=db_path)
        # Initialize asynchronously
        try:
            loop = asyncio.get_event_loop()
            if loop.is_running():
                asyncio.create_task(self._durable_engine.initialize())
            else:
                asyncio.run(self._durable_engine.initialize())
        except RuntimeError:
            asyncio.run(self._durable_engine.initialize())
        logger.info("[SFDSystemWiring] Durable execution wired (%s)", db_path)

    def wrap_tool_execution(self, tool_name: str, params: dict, executor: Callable[[str, dict], Any]) -> Any:
        """Wrap a tool execution in a durable activity if appropriate.

        Only wraps long-running or non-idempotent actions.
        """
        # Threshold: > 5s expected, or destructive/write operations
        durable_tools = {"file_write", "shell", "deploy", "docker", "database_write"}
        if tool_name not in durable_tools and not self._durable_engine:
            return executor(tool_name, params)

        if self._durable_engine:
            import asyncio, uuid
            wf_id = asyncio.run(self._durable_engine.create_workflow(f"tool:{tool_name}"))
            act = type('obj', (object,), {
                'activity_id': str(uuid.uuid4())[:8],
                'action': tool_name,
                'params': params,
                'retry_policy': type('obj', (object,), {
                    'max_attempts': 3,
                    'backoff': type('obj', (object,), {'value': 'exponential'})(),
                    'initial_delay_ms': 1000,
                    'max_delay_ms': 60000,
                })(),
                'compensation': None,
                'attempt': 0,
            })()
            asyncio.run(self._durable_engine.add_activity(wf_id, act))
            status = asyncio.run(self._durable_engine.start_workflow(wf_id, executor))
            if status == "failed":
                self._stats["sagas_compensated"] += 1
            self._stats["retries_handled"] += 1 if "retry" in str(status).lower() else 0
            return status == "completed"
        return executor(tool_name, params)

    # ------------------------------------------------------------------
    # PREFERENCES (P20)
    # ------------------------------------------------------------------

    def wire_preferences(self) -> None:
        from src.preferences.engine import PreferenceEngine, set_preference_engine
        self._pref_engine = PreferenceEngine()
        set_preference_engine(self._pref_engine)
        logger.info("[SFDSystemWiring] Preferences wired")

    def inject_preferences(self, system_prompt: str) -> str:
        """Inject resolved preferences into the system prompt."""
        if not self._pref_engine:
            return system_prompt
        resolved = self._pref_engine.resolve([])
        if resolved.get("langue"):
            system_prompt += f"\n\n[PREFERENCE] Réponds en {resolved['langue']}."
        if resolved.get("format"):
            system_prompt += f"\n[PREFERENCE] Format: {resolved['format']}."
        if resolved.get("ton"):
            system_prompt += f"\n[PREFERENCE] Ton: {resolved['ton']}."
        return system_prompt

    # ------------------------------------------------------------------
    # VISUAL OUTPUT (P22)
    # ------------------------------------------------------------------

    def wire_visual(self) -> None:
        from src.visual_output.router import OutputRouter, set_output_router
        self._output_router = OutputRouter()
        set_output_router(self._output_router)
        logger.info("[SFDSystemWiring] Visual output wired")

    def route_response(self, request: str, response: str) -> dict:
        """Route a response through the visual decision tree."""
        if not self._output_router:
            return {"mode": "text"}
        decision = self._output_router.decide(request, response)
        if decision.mode.value != "text":
            self._stats["visuals_routed"] += 1
        return {
            "mode": decision.mode.value,
            "module": decision.module,
            "mcp_tool": decision.mcp_tool,
            "reason": decision.reason,
        }

    # ------------------------------------------------------------------
    # CONTEXT MANAGER (P13)
    # ------------------------------------------------------------------

    def wire_context(self, max_tokens: int = 128000) -> None:
        from src.context_manager import ContextManager
        self._context_mgr = ContextManager(max_tokens=max_tokens)
        logger.info("[SFDSystemWiring] Context manager wired (%d tokens)", max_tokens)

    def track_tokens(self, tokens: int) -> None:
        if self._context_mgr:
            self._context_mgr.track_tokens(tokens)

    def should_compact(self) -> bool:
        if self._context_mgr:
            if self._context_mgr.needs_compaction():
                self._stats["compactions_done"] += 1
                return True
        return False

    # ------------------------------------------------------------------
    # MISSING PRINCIPLES
    # ------------------------------------------------------------------

    # P5 — Divulgation progressive: filter context by phase
    def progressive_disclosure(self, phase: str, all_context: list[str]) -> list[str]:
        """Only reveal context relevant to the current phase."""
        phase_filters = {
            "CLASSIFY": ["risk", "constraint", "budget"],
            "KNOW": ["memory", "history", "skill"],
            "PLAN": ["architecture", "pattern", "tool"],
            "BUILD": ["code", "test", "deploy"],
            "QUALITY": ["test", "lint", "security"],
            "AUTOEVAL": ["metric", "success", "failure"],
            "MEMORY_OBSERVE": ["lesson", "pattern", "skill"],
        }
        relevant = phase_filters.get(phase, [])
        if not relevant:
            return all_context[:10]  # Default: limit to 10 entries
        return [c for c in all_context if any(r in c.lower() for r in relevant)][:10]

    # P6 — Échecs répétés → règles: pattern detection in errors
    def learn_from_failures(self, error_message: str) -> Optional[str]:
        """Detect repeated failure patterns and suggest a rule."""
        patterns = {
            "timeout": "Ajouter retry avec backoff exponentiel pour les appels externes",
            "permission denied": "Vérifier les permissions avant d'exécuter",
            "not found": "Vérifier l'existence des fichiers/ressources avant usage",
            "connection refused": "Attendre que le service soit prêt avant d'appeler",
            "out of memory": "Limiter la taille des données chargées en mémoire",
        }
        for pattern, rule in patterns.items():
            if pattern in error_message.lower():
                self._stats["rules_learned"] += 1
                return rule
        return None

    # P12 — Découpage par contexte: guide for sub-agent creation
    def should_create_sub_agent(self, task_context: str, main_context: str) -> bool:
        """Check if a task's context can be truly isolated."""
        # A sub-agent is justified when the task context has < 20% overlap with main context
        task_words = set(task_context.lower().split())
        main_words = set(main_context.lower().split())
        if not task_words:
            return False
        overlap = len(task_words & main_words) / len(task_words)
        return overlap < 0.2  # Low overlap = safe to isolate

    # P19 — Mémoire gagne sa place: impact check before using a fact
    def memory_earns_place(self, fact: str, response: str) -> bool:
        """Check if a memory fact changes the response substance."""
        # Simple heuristic: if the fact's key terms appear in the response, it had impact
        fact_terms = set(fact.lower().split()) - {"the", "a", "an", "is", "are", "de", "le", "la", "un", "une", "et", "est"}
        response_lower = response.lower()
        matches = sum(1 for t in fact_terms if t in response_lower)
        return matches >= 2  # At least 2 key terms from the fact appear in the response

    # ------------------------------------------------------------------
    # Stats
    # ------------------------------------------------------------------

    @property
    def stats(self) -> dict:
        return self._stats

    def summary(self) -> str:
        return (
            f"SFDSystemWiring: {self._stats['facts_stored']} facts stored, "
            f"{self._stats['facts_omitted']} omitted, "
            f"{self._stats['visuals_routed']} visuals, "
            f"{self._stats['compactions_done']} compactions, "
            f"{self._stats['rules_learned']} rules learned"
        )


def get_wiring() -> Optional[SFDSystemWiring]:
    return _wiring


def set_wiring(w: SFDSystemWiring) -> None:
    global _wiring
    _wiring = w


def init_wiring(
    memory_base_path: str = "workspace/memory",
    durable_db_path: str = "data/durable.db",
    context_max_tokens: int = 128000,
) -> SFDSystemWiring:
    """Initialize and wire all SFD v3.0 modules."""
    global _wiring
    w = SFDSystemWiring()

    try:
        w.wire_memory(memory_base_path)
    except Exception as e:
        logger.warning("Memory wiring skipped: %s", e)

    try:
        w.wire_durable(durable_db_path)
    except Exception as e:
        logger.warning("Durable wiring skipped: %s", e)

    try:
        w.wire_preferences()
    except Exception as e:
        logger.warning("Preferences wiring skipped: %s", e)

    try:
        w.wire_visual()
    except Exception as e:
        logger.warning("Visual wiring skipped: %s", e)

    try:
        w.wire_context(context_max_tokens)
    except Exception as e:
        logger.warning("Context wiring skipped: %s", e)

    _wiring = w
    logger.info("[SFDSystemWiring] All modules wired: %s", w.summary())
    return w
