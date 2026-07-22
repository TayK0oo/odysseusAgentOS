"""
Multi-Agent Decision Engine (§5.1.2).

Scores tasks against 3 criteria to decide single vs multi-agent:
    1. Context protection — subtask produces >1000 irrelevant tokens
    2. Parallelization — truly independent subtasks, no shared state
    3. Specialization — >15 tools or conflicting behavioral instructions

SFD NF-10: no multi-agent without documented justification.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Optional


class AgentMode(str, Enum):
    SINGLE = "single_agent"
    MULTI = "multi_agent"


@dataclass
class DecompositionDecision:
    mode: AgentMode
    reason: str
    criteria_met: list[str]  # which criteria were triggered
    token_overhead_estimate: float = 1.0  # multiplier (1.0 = no overhead)
    recommended_agents: Optional[list[str]] = None


@dataclass
class TaskProfile:
    """Measurable characteristics of a task for decomposition analysis."""
    estimated_output_tokens: int = 500
    irrelevant_token_ratio: float = 0.0  # 0.0-1.0
    is_parallelizable: bool = False
    has_independent_subtasks: bool = False
    tool_count: int = 5
    has_conflicting_instructions: bool = False
    domain_count: int = 1  # number of unrelated tool domains


class MultiAgentDecisionEngine:
    """Decides whether a task should use single or multi-agent mode.

    Default: single agent. Multi-agent only if one of the 3 criteria
    is met with measurable evidence (§5.1.2).

    Context protection: subtask > 1000 tokens of irrelevant content
    Parallelization: truly independent, no state sharing
    Specialization: > 15 tools or conflicting behavioral consignes
    """

    CONTEXT_THRESHOLD_TOKENS = 1000
    TOOL_THRESHOLD = 15

    def evaluate(self, task: TaskProfile) -> DecompositionDecision:
        """Evaluate a task and decide single vs multi-agent.

        Returns DecompositionDecision with the mode and justification.
        """
        criteria_met: list[str] = []
        reasons: list[str] = []

        # Criterion 1: Context protection
        irrelevant_tokens = int(task.estimated_output_tokens * task.irrelevant_token_ratio)
        if irrelevant_tokens > self.CONTEXT_THRESHOLD_TOKENS:
            criteria_met.append("context_protection")
            reasons.append(
                f"Subtask produces ~{irrelevant_tokens} irrelevant tokens "
                f"(threshold: {self.CONTEXT_THRESHOLD_TOKENS})"
            )

        # Criterion 2: Parallelization
        if task.is_parallelizable and task.has_independent_subtasks:
            criteria_met.append("parallelization")
            reasons.append("Task decomposes into truly independent subtasks")

        # Criterion 3: Specialization
        if task.tool_count > self.TOOL_THRESHOLD:
            criteria_met.append("specialization")
            reasons.append(
                f"Agent handles {task.tool_count} tools "
                f"(threshold: {self.TOOL_THRESHOLD})"
            )

        if task.has_conflicting_instructions or task.domain_count > 2:
            if "specialization" not in criteria_met:
                criteria_met.append("specialization")
            reasons.append(
                f"Conflicting instructions across {task.domain_count} unrelated domains"
            )

        if criteria_met:
            overhead = 3.0 if len(criteria_met) >= 2 else 2.0
            return DecompositionDecision(
                mode=AgentMode.MULTI,
                reason=" | ".join(reasons),
                criteria_met=criteria_met,
                token_overhead_estimate=overhead,
                recommended_agents=self._suggest_agents(criteria_met),
            )

        return DecompositionDecision(
            mode=AgentMode.SINGLE,
            reason="No decomposition criteria met — single agent sufficient",
            criteria_met=[],
            token_overhead_estimate=1.0,
        )

    def _suggest_agents(self, criteria_met: list[str]) -> list[str]:
        """Suggest agent types based on which criteria were met."""
        agents = []
        if "context_protection" in criteria_met:
            agents.append("context-isolated-worker")
        if "parallelization" in criteria_met:
            agents.append("parallel-worker")  
        if "specialization" in criteria_met:
            agents.append("domain-specialist")
        return agents

    @staticmethod
    def measure_overhead(
        single_agent_tokens: int,
        multi_agent_tokens: int,
    ) -> float:
        """Calculate the actual token multiplier from observed data.

        Typical: 3-10x for multi-agent, up to 15x for orchestrator-workers.
        """
        if single_agent_tokens == 0:
            return 0.0
        return multi_agent_tokens / single_agent_tokens
