"""SFD §5.18 — Modalités de sortie et visualisation.

Arbre de décision 3 étapes :
  Étape 0 — La réponse nécessite-t-elle autre chose que du texte ?
  Étape 1 — Un outil MCP connecté gère-t-il cette catégorie ?
  Étape 2 — L'utilisateur demande-t-il un fichier ?
  Étape 3 — Visualiseur inline (défaut)

Gated behind ODYSSEUS_OUTPUT_ROUTER kill-switch (default OFF).
"""

from __future__ import annotations

import logging
import os
import re
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Literal, Optional

logger = logging.getLogger(__name__)

# ─── Kill-switch ────────────────────────────────────────────────────────

def output_router_enabled() -> bool:
    val = os.getenv("ODYSSEUS_OUTPUT_ROUTER", "off").strip().lower()
    return val in {"on", "1", "true", "yes"}


# ─── Types ──────────────────────────────────────────────────────────────

class OutputMode(str, Enum):
    TEXT_ONLY = "text_only"
    TEXT_INLINE = "text_inline"
    MCP_TOOL = "mcp_tool"
    FILE_DOWNLOAD = "file_download"
    VISUAL_INLINE = "visual_inline"


class DesignModule(str, Enum):
    DIAGRAM = "diagram"
    MOCKUP = "mockup"
    INTERACTIVE = "interactive"
    CHART = "chart"
    ART = "art"
    DATA_VIZ = "data_viz"


@dataclass
class OutputDecision:
    mode: OutputMode
    module: Optional[DesignModule] = None
    mcp_tool: Optional[str] = None
    file_extension: Optional[str] = None
    reason: str = ""


# ─── Triggers ───────────────────────────────────────────────────────────

# Mots-clés qui déclenchent une visualisation
VISUAL_TRIGGERS: Dict[DesignModule, List[str]] = {
    DesignModule.DIAGRAM: [
        "diagramme", "diagram", "schéma", "schema", "flowchart",
        "organigramme", "flux", "flow", "architecture", "séquence",
        "sequence", "graphe", "graph",
    ],
    DesignModule.CHART: [
        "graphique", "chart", "courbe", "plot", "histogramme",
        "pie chart", "bar chart", "camembert", "données", "data",
        "statistiques", "statistics", "comparaison", "comparison",
    ],
    DesignModule.MOCKUP: [
        "maquette", "mockup", "wireframe", "interface", "UI",
        "formulaire", "form", "écran", "screen", "page",
        "landing page", "app",
    ],
    DesignModule.DATA_VIZ: [
        "visualisation", "visualization", "carte", "map",
        "heatmap", "treemap", "sankey", "network",
    ],
}

# Signaux de demande de fichier
FILE_SIGNALS: List[str] = [
    "crée un fichier", "cree un fichier", "sauvegarde", "sauvegarder",
    "télécharge", "telecharge", "exporte", "exporter", "download",
    ".pdf", ".png", ".svg", ".html", ".csv", ".json", ".xlsx",
]

# Catégories → modules de design
CATEGORY_TO_MODULE: Dict[str, DesignModule] = {
    "architecture": DesignModule.DIAGRAM,
    "flux": DesignModule.DIAGRAM,
    "diagramme": DesignModule.DIAGRAM,
    "graphique": DesignModule.CHART,
    "données": DesignModule.CHART,
    "data": DesignModule.CHART,
    "interface": DesignModule.MOCKUP,
    "ui": DesignModule.MOCKUP,
    "maquette": DesignModule.MOCKUP,
}


# ─── Router ─────────────────────────────────────────────────────────────

@dataclass
class OutputRouter:
    """Routeur de modalité de sortie selon l'arbre de décision SFD §5.18."""

    connected_mcp_tools: Dict[str, str] = field(default_factory=dict)
    # {category: mcp_tool_name}

    def route(self, request: str, response_text: str) -> OutputDecision:
        """Détermine la modalité de sortie optimale.

        Étapes dans l'ordre, s'arrête à la première correspondance.
        """
        request_lower = request.lower()

        # Étape 0 : réponse purement textuelle ?
        if self._is_purely_textual(request):
            return OutputDecision(mode=OutputMode.TEXT_ONLY, reason="purely textual request")

        # Étape 1 : outil MCP connecté ?
        category = self._detect_category(request_lower)
        if category and category in self.connected_mcp_tools:
            return OutputDecision(
                mode=OutputMode.MCP_TOOL,
                mcp_tool=self.connected_mcp_tools[category],
                reason=f"MCP tool available for category '{category}'",
            )

        # Étape 2 : demande de fichier ?
        if self._is_file_request(request_lower):
            ext = self._detect_file_extension(request_lower)
            return OutputDecision(
                mode=OutputMode.FILE_DOWNLOAD,
                file_extension=ext,
                reason="explicit file request detected",
            )

        # Étape 3 : visualiseur inline ?
        module = self._detect_design_module(request_lower)
        if module and self._content_merits_visualization(response_text):
            return OutputDecision(
                mode=OutputMode.VISUAL_INLINE,
                module=module,
                reason=f"content merits {module.value} visualization",
            )

        # Défaut : texte
        return OutputDecision(mode=OutputMode.TEXT_ONLY, reason="no visual trigger detected")

    def _is_purely_textual(self, request: str) -> bool:
        """Vérifie si la demande est purement textuelle."""
        textual_patterns = [
            r"^(dis|dit|répète|repete|quoi|comment|pourquoi|quand|où|qui|quel)",
            r"^(explique|décris|decris|résume|resume|liste|donne)",
            r"\?$",
        ]
        for pattern in textual_patterns:
            if re.search(pattern, request, re.IGNORECASE):
                # Vérifier qu'aucun trigger visuel n'est présent
                for triggers in VISUAL_TRIGGERS.values():
                    for trigger in triggers:
                        if trigger in request.lower():
                            return False
                return True
        return False

    def _detect_category(self, request: str) -> Optional[str]:
        """Détecte la catégorie de la demande."""
        for category, keywords in VISUAL_TRIGGERS.items():
            for keyword in keywords:
                if keyword in request:
                    return category.value
        return None

    def _is_file_request(self, request: str) -> bool:
        """Détecte une demande explicite de fichier."""
        for signal in FILE_SIGNALS:
            if signal in request:
                return True
        return False

    def _detect_file_extension(self, request: str) -> Optional[str]:
        """Détecte l'extension de fichier demandée."""
        extensions = [".pdf", ".png", ".svg", ".html", ".csv", ".json", ".xlsx", ".md", ".py"]
        for ext in extensions:
            if ext in request:
                return ext
        return None

    def _detect_design_module(self, request: str) -> Optional[DesignModule]:
        """Détecte le module de design approprié."""
        for category, triggers in VISUAL_TRIGGERS.items():
            for trigger in triggers:
                if trigger in request:
                    return category
        return None

    def _content_merits_visualization(self, content: str) -> bool:
        """Vérifie si le contenu mérite une visualisation."""
        # Contenu avec structure spatiale/séquentielle/systémique
        structural_signals = [
            r"├|└|│|─",  # ASCII tree
            r"graph\s+(TD|LR|RL|BT)",  # Mermaid
            r"sequenceDiagram",
            r"flowchart",
            r"```(mermaid|dot|graphviz)",
            r"\d+\s*(étapes|steps|phases|stages)",
            r"(tableau|table)\s+(comparatif|de\s+comparaison)",
        ]
        for signal in structural_signals:
            if re.search(signal, content, re.IGNORECASE):
                return True
        return False


# ─── Singleton ───────────────────────────────────────────────────────────

_router: Optional[OutputRouter] = None


def get_output_router() -> OutputRouter:
    global _router
    if _router is None:
        _router = OutputRouter()
    return _router
