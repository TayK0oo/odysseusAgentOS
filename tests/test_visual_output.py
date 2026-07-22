"""
Tests for Visual Output system (src/visual_output/).

Covers: output router decision tree, patterns, renderer, design modules.
"""

import pytest

from src.visual_output.router import OutputRouter, OutputMode, route_output, OutputDecision
from src.visual_output.renderer import InlineRenderer, render_svg, render_html, RenderedOutput
from src.visual_output.modules import DesignModule, MODULES


# ---------------------------------------------------------------------------
# OutputRouter
# ---------------------------------------------------------------------------

class TestOutputRouter:
    def setup_method(self):
        self.router = OutputRouter()

    def test_purely_textual_short_response(self):
        decision = self.router.decide("hello", "Hi there! How can I help you?")
        assert decision.mode == OutputMode.TEXT

    def test_visual_pattern_triggers_inline(self):
        decision = self.router.decide(
            "montre-moi un diagramme du système",
            "Here is the system architecture with 3 components...",
        )
        assert decision.mode == OutputMode.INLINE_VISUAL
        assert decision.module == "diagram"

    def test_explicit_file_request(self):
        decision = self.router.decide(
            "crée un fichier PDF du rapport",
            "Report content...",
        )
        assert decision.mode == OutputMode.FILE

    def test_mcp_tool_kroki(self):
        decision = self.router.decide(
            "visualise le flux de données",
            "Data flows from A to B to C...",
            available_mcp_tools=["kroki", "filesystem"],
        )
        assert decision.mode == OutputMode.MCP_TOOL
        assert decision.mcp_tool == "kroki"

    def test_detect_module_diagram(self):
        assert self.router.detect_module("draw a flowchart", "flow...") == "diagram"
        assert self.router.detect_module("architecture diagram", "arch...") == "diagram"

    def test_detect_module_mockup(self):
        assert self.router.detect_module("create a wireframe", "ui...") == "mockup"
        assert self.router.detect_module("design the interface", "screen...") == "mockup"

    def test_detect_module_chart(self):
        assert self.router.detect_module("plot the data", "statistics...") == "chart"

    def test_llm_confirms_overrides_text(self):
        decision = self.router.decide(
            "how are you",
            "I'm doing well, thanks!",
            llm_confirms_visual=True,
        )
        assert decision.mode == OutputMode.INLINE_VISUAL

    def test_has_visual_pattern(self):
        assert OutputRouter.has_visual_pattern("montre-moi un graphique") is True
        assert OutputRouter.has_visual_pattern("show me a diagram") is True
        assert OutputRouter.has_visual_pattern("hello world") is False

    def test_needs_file(self):
        assert OutputRouter.needs_file("crée un fichier .pdf") is True
        assert OutputRouter.needs_file("exporte les données") is True
        assert OutputRouter.needs_file("explique moi") is False

    def test_convenience_function(self):
        decision = route_output("montre-moi un diagramme", "System has 3 layers...")
        assert decision.mode == OutputMode.INLINE_VISUAL


# ---------------------------------------------------------------------------
# Renderer
# ---------------------------------------------------------------------------

class TestInlineRenderer:
    def test_wrap(self):
        wrapped = InlineRenderer.wrap("<svg>...</svg>", module="diagram")
        assert "inline-visual" in wrapped
        assert 'data-module="diagram"' in wrapped
        assert "<svg>...</svg>" in wrapped

    def test_svg_container(self):
        svg = InlineRenderer.svg_container("<circle/>", width=400, height=300)
        assert 'viewBox="0 0 400 300"' in svg
        assert "<circle/>" in svg

    def test_render_svg(self):
        output = render_svg("<rect/>", module="diagram")
        assert output.format == "svg"
        assert output.sandboxed is True
        assert "inline-visual" in output.content

    def test_render_html(self):
        output = render_html("<button>Click</button>", module="interactive")
        assert output.format == "html"
        assert "inline-visual" in output.content
        assert "<button>Click</button>" in output.content


# ---------------------------------------------------------------------------
# Design Modules
# ---------------------------------------------------------------------------

class TestDesignModules:
    def test_all_modules_registered(self):
        expected = {"diagram", "mockup", "interactive", "chart", "art", "data_viz"}
        assert set(MODULES.keys()) == expected

    def test_module_has_colors(self):
        for name, mod in MODULES.items():
            assert len(mod.colors) >= 2, f"{name} needs at least 2 colors"

    def test_module_has_fonts(self):
        for name, mod in MODULES.items():
            assert len(mod.fonts) >= 1, f"{name} needs at least 1 font"

    def test_diagram_defaults(self):
        mod = MODULES["diagram"]
        assert mod.default_width == 900
        assert mod.format == "svg"

    def test_interactive_is_html(self):
        mod = MODULES["interactive"]
        assert mod.format == "html"

    def test_art_has_more_elements(self):
        mod = MODULES["art"]
        assert mod.max_elements == 200
