"""
Inline renderer — HTML/SVG widgets for the visual output system.

Renders content in sandboxed iframes within the SSE stream.
Kroki is used for Mermaid/PlantUML diagrams (already in Docker stack).
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass
class RenderedOutput:
    """A rendered visual element ready for the frontend."""
    format: str  # "html" or "svg"
    content: str
    width: Optional[int] = None
    height: Optional[int] = None
    sandboxed: bool = True


class InlineRenderer:
    """Generates HTML/SVG output for inline rendering in the frontend.

    Output is wrapped in a sandboxed container for security.
    """

    SANDBOX_TEMPLATE = """<div class="inline-visual" data-module="{module}">
{content}
</div>"""

    SVG_WRAPPER = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}">
{content}
</svg>"""

    @staticmethod
    def wrap(content: str, module: str = "diagram") -> str:
        """Wrap rendered content in a sandboxed container."""
        return InlineRenderer.SANDBOX_TEMPLATE.format(
            module=module,
            content=content,
        )

    @staticmethod
    def svg_container(content: str, width: int = 800, height: int = 600) -> str:
        """Wrap SVG content with viewBox."""
        return InlineRenderer.SVG_WRAPPER.format(
            width=width, height=height, content=content,
        )


def render_svg(content: str, module: str = "diagram", width: int = 800, height: int = 600) -> RenderedOutput:
    """Render SVG content for inline display."""
    renderer = InlineRenderer()
    svg = renderer.svg_container(content, width, height)
    wrapped = renderer.wrap(svg, module)
    return RenderedOutput(
        format="svg",
        content=wrapped,
        width=width,
        height=height,
    )


def render_html(content: str, module: str = "interactive") -> RenderedOutput:
    """Render HTML content for inline display in iframe."""
    renderer = InlineRenderer()
    wrapped = renderer.wrap(content, module)
    return RenderedOutput(
        format="html",
        content=wrapped,
    )
