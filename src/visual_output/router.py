"""
Output routing — decision tree for response modality.

SFD §5.18.1: 3-step decision tree.
Step 0: Is the response purely textual?
Step 1: Does a connected MCP tool handle this category?
Step 2: Is the user explicitly requesting a file?
Step 3: Inline visualizer (default for visual-worthy content).

Pattern + LLM two-stage trigger.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Optional

# Trigger patterns for automatic visualization
_VISUAL_PATTERNS = [
    "montre-moi", "visualise", "diagramme", "graphique",
    "illustre", "schéma", "show me", "visualize", "diagram",
    "chart", "graph", "draw", "render", "plot",
]


class OutputMode(str, Enum):
    TEXT = "text"
    MCP_TOOL = "mcp_tool"
    FILE = "file"
    INLINE_VISUAL = "inline_visual"


@dataclass
class OutputDecision:
    mode: OutputMode
    reason: str
    module: Optional[str] = None  # Design module name if inline_visual
    mcp_tool: Optional[str] = None  # MCP tool name if mcp_tool


class OutputRouter:
    """Routes responses to the appropriate output modality.

    Two-stage trigger: patterns + keywords (fast) → LLM confirmation.
    """

    @staticmethod
    def has_visual_pattern(text: str) -> bool:
        """Fast pattern check for visual triggers."""
        text_lower = text.lower()
        return any(p in text_lower for p in _VISUAL_PATTERNS)

    @staticmethod
    def is_purely_textual(request: str, response: str) -> bool:
        """Check if the response is purely textual (no visual needed)."""
        # Explicit requests always get visuals
        if OutputRouter.has_visual_pattern(request):
            return False
        # Short, conversational responses don't need visuals
        if len(response.split()) < 50 and not OutputRouter.has_visual_pattern(response):
            return True
        return False

    @staticmethod
    def needs_file(request: str) -> bool:
        """Check if user explicitly requests a file."""
        file_signals = [
            "crée un fichier", "sauvegarde", "télécharge",
            "exporte", "create a file", "save", "download",
            "export", ".pdf", ".xlsx", ".pptx", ".docx", ".csv",
        ]
        req_lower = request.lower()
        return any(s in req_lower for s in file_signals)

    @staticmethod
    def detect_module(request: str, response: str) -> Optional[str]:
        """Detect which design module is appropriate."""
        combined = (request + " " + response).lower()
        if any(w in combined for w in ["diagram", "flow", "architecture", "sequence", "mermaid"]):
            return "diagram"
        if any(w in combined for w in ["mockup", "wireframe", "ui", "interface", "screen"]):
            return "mockup"
        if any(w in combined for w in ["chart", "graph", "plot", "data", "statistics"]):
            return "chart"
        if any(w in combined for w in ["interactive", "calculator", "widget", "game"]):
            return "interactive"
        if any(w in combined for w in ["illustration", "art", "creative", "drawing"]):
            return "art"
        if any(w in combined for w in ["data_viz", "dashboard", "heatmap", "treemap"]):
            return "data_viz"
        return "diagram"  # default

    def decide(
        self,
        request: str,
        response: str,
        available_mcp_tools: Optional[list[str]] = None,
        llm_confirms_visual: bool = False,
    ) -> OutputDecision:
        """Route the response to the best output modality.

        Args:
            request: The user's original request.
            response: The generated text response.
            available_mcp_tools: Connected MCP tools that could render visuals.
            llm_confirms_visual: Second-stage LLM confirmation (patterns matched).
        """
        # Step 2: Explicit file request? (check before textual — takes priority)
        if self.needs_file(request):
            return OutputDecision(OutputMode.FILE, "explicit file request detected")

        # Step 0: Purely textual?
        if self.is_purely_textual(request, response) and not llm_confirms_visual:
            return OutputDecision(OutputMode.TEXT, "purely textual response")

        # Step 1: MCP tool?
        if available_mcp_tools:
            # Kroki is the default diagram tool in the stack
            if "kroki" in [t.lower() for t in available_mcp_tools]:
                module = self.detect_module(request, response)
                if module in ("diagram", "chart"):
                    return OutputDecision(
                        OutputMode.MCP_TOOL,
                        "Kroki handles diagrams/charts",
                        mcp_tool="kroki",
                    )

        # Step 3: Inline visual?
        if self.has_visual_pattern(request) or llm_confirms_visual:
            module = self.detect_module(request, response)
            return OutputDecision(
                OutputMode.INLINE_VISUAL,
                f"visual content detected → {module}",
                module=module,
            )

        return OutputDecision(OutputMode.TEXT, "no visual needed")


def route_output(
    request: str,
    response: str,
    available_mcp_tools: Optional[list[str]] = None,
    llm_confirms: bool = False,
) -> OutputDecision:
    """Convenience function for output routing."""
    router = OutputRouter()
    return router.decide(request, response, available_mcp_tools, llm_confirms)
