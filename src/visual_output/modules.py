"""
Design modules — constraints for each visual output type.

6 modules: diagram, mockup, interactive, chart, art, data_viz.
Each module defines rendering constraints (colors, dimensions, fonts).
Loaded as skills before generation (SFD §5.18.3).
"""

from dataclasses import dataclass, field

MODULES: dict[str, "DesignModule"] = {}


@dataclass
class DesignModule:
    """Constraints and defaults for a visual output module."""
    name: str
    description: str
    default_width: int = 800
    default_height: int = 600
    format: str = "svg"  # svg | html
    colors: dict[str, str] = field(default_factory=dict)
    fonts: list[str] = field(default_factory=lambda: ["system-ui", "sans-serif"])
    max_elements: int = 50  # max SVG/HTML elements for performance

    def __post_init__(self):
        MODULES[self.name] = self


# ---------------------------------------------------------------------------
# 6 canonical design modules (SFD §5.18.3)
# ---------------------------------------------------------------------------

DesignModule(
    name="diagram",
    description="Flow, architecture, and sequence diagrams",
    default_width=900,
    default_height=500,
    colors={"primary": "#3B82F6", "secondary": "#10B981", "bg": "#F9FAFB", "text": "#111827"},
)

DesignModule(
    name="mockup",
    description="UI mockups and wireframes",
    default_width=400,
    default_height=700,
    format="html",
    colors={"primary": "#6366F1", "bg": "#FFFFFF", "border": "#E5E7EB", "text": "#374151"},
    fonts=["Inter", "system-ui"],
)

DesignModule(
    name="interactive",
    description="Interactive widgets, calculators, games",
    default_width=600,
    default_height=400,
    format="html",
    colors={"primary": "#8B5CF6", "accent": "#EC4899", "bg": "#1F2937", "text": "#F9FAFB"},
)

DesignModule(
    name="chart",
    description="Data charts and graphs",
    default_width=700,
    default_height=400,
    colors={"primary": "#F59E0B", "secondary": "#3B82F6", "grid": "#E5E7EB", "text": "#111827"},
)

DesignModule(
    name="art",
    description="Illustrations and creative visual content",
    default_width=800,
    default_height=600,
    colors={"primary": "#EC4899", "secondary": "#8B5CF6", "bg": "#FAFAFA"},
    max_elements=200,
)

DesignModule(
    name="data_viz",
    description="Complex data visualizations",
    default_width=900,
    default_height=500,
    colors={"primary": "#06B6D4", "heat_low": "#DBEAFE", "heat_high": "#1E40AF", "text": "#111827"},
)
