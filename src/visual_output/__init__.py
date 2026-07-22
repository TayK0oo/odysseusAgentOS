"""
Visual Output System — multimodal response routing.

SFD §5.18: Decision tree (MCP tool → file → inline visualizer → text).
Renderer: HTML/SVG inline + Kroki for diagrams.
Design modules: diagram, mockup, interactive, chart, art, data_viz.

Each module = a skill loaded before generation.
"""

from src.visual_output.router import OutputRouter, OutputMode, route_output
from src.visual_output.renderer import InlineRenderer, render_svg, render_html
from src.visual_output.modules import DesignModule, MODULES

__all__ = [
    "OutputRouter",
    "OutputMode",
    "route_output",
    "InlineRenderer",
    "render_svg",
    "render_html",
    "DesignModule",
    "MODULES",
]
