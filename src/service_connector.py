"""
Service Connector — Wires external MCP/Docker services for SFD skills.

Connects:
    - Kroki (port 8700) → diagrams, charts
    - Meilisearch (port 7700) → conversation search
    - LangFuse (port 3000) → observability tracing
    - ChromaDB (port 8100) → vector memory
    - ntfy (port 8091) → notifications
"""

import os, json, logging, httpx
from typing import Optional

logger = logging.getLogger(__name__)

SERVICES = {
    "kroki": {"port": 8700, "url": "http://127.0.0.1:8700", "format": "svg"},
    "meilisearch": {"port": 7700, "url": "http://127.0.0.1:7700"},
    "langfuse": {"port": 3000, "url": "http://127.0.0.1:3000"},
    "chromadb": {"port": 8100, "url": "http://127.0.0.1:8100"},
    "ntfy": {"port": 8091, "url": "http://127.0.0.1:8091"},
}

class ServiceConnector:
    """Checks and connects to Docker/MCP services required by SFD skills."""

    @staticmethod
    def check(service_name: str) -> bool:
        """Check if a service is reachable."""
        if service_name not in SERVICES:
            return False
        url = SERVICES[service_name]["url"]
        try:
            r = httpx.get(f"{url}/", timeout=2)
            return r.status_code < 500
        except Exception:
            return False

    @staticmethod
    def status() -> dict:
        """Get status of all services."""
        return {name: ServiceConnector.check(name) for name in SERVICES}

    # Kroki — diagram generation (Mermaid, PlantUML, GraphViz)
    @staticmethod
    def render_diagram(diagram_text: str, diagram_type: str = "mermaid") -> Optional[str]:
        """Render a diagram via Kroki. Returns SVG string or None."""
        url = f"{SERVICES['kroki']['url']}/{diagram_type}/svg"
        try:
            r = httpx.post(url, content=diagram_text.encode(), timeout=10)
            if r.status_code == 200:
                return r.text
        except Exception as e:
            logger.warning("Kroki render failed: %s", e)
        return None

    # Meilisearch — full-text conversation search
    @staticmethod
    def search_conversations(query: str, index: str = "conversations", limit: int = 10) -> list:
        """Search conversations via Meilisearch."""
        url = f"{SERVICES['meilisearch']['url']}/indexes/{index}/search"
        try:
            r = httpx.post(url, json={"q": query, "limit": limit}, timeout=5)
            if r.status_code == 200:
                return r.json().get("hits", [])
        except Exception as e:
            logger.warning("Meilisearch search failed: %s", e)
        return []

    @staticmethod
    def summary() -> str:
        """Human-readable status of all services."""
        status = ServiceConnector.status()
        lines = []
        for name, ok in status.items():
            icon = "OK" if ok else "OFF"
            lines.append(f"  {name}: {icon} ({SERVICES[name]['url']})")
        return "\n".join(lines)
