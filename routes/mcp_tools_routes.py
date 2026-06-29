"""Routes pour les MCP tools externes (Scrapling, Supabase, Kroki)."""
from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional, List

router = APIRouter(prefix="/api/tools", tags=["mcp-tools"])

class ScrapeRequest(BaseModel):
    url: str
    mode: str = "basic"
    output_format: str = "markdown"

class SpiderRequest(BaseModel):
    start_url: str
    max_pages: int = 10
    follow_patterns: List[str] = []

class DiagramRequest(BaseModel):
    content: str
    diagram_type: str = "mermaid"  # mermaid, plantuml, graphviz, etc.
    output_format: str = "svg"

@router.post("/scrape")
async def scrape_url(req: ScrapeRequest):
    """Scrapling fetch — scrape une URL avec anti-bot."""
    try:
        import httpx
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.post(
                "http://localhost:8800/extract",
                json={"url": req.url, "mode": req.mode, "format": req.output_format}
            )
            return resp.json()
    except Exception as e:
        return {"error": str(e), "hint": "Démarrer avec: docker compose --profile scrapling up -d"}

@router.post("/spider")
async def spider_url(req: SpiderRequest):
    """Scrapling spider — crawl multi-pages."""
    try:
        import httpx
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(
                "http://localhost:8800/spider",
                json={"url": req.start_url, "max_pages": req.max_pages, "patterns": req.follow_patterns}
            )
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

@router.post("/diagram")
async def render_diagram(req: DiagramRequest):
    """Kroki — render un diagramme texte en SVG/PNG."""
    try:
        import httpx, base64, zlib
        # Encodage Kroki : deflate + base64url
        compressed = zlib.compress(req.content.encode(), level=9)
        encoded = base64.urlsafe_b64encode(compressed).decode()

        url = f"http://localhost:8700/{req.diagram_type}/{req.output_format}/{encoded}"
        resp = httpx.get(url, timeout=15.0)

        if req.output_format == "svg":
            return {"svg": resp.text, "url": url}
        else:
            return {"image_b64": base64.b64encode(resp.content).decode(), "url": url}
    except Exception as e:
        return {"error": str(e), "hint": "Kroki démarre automatiquement avec docker compose up"}

@router.get("/available")
async def list_available_tools():
    """Liste les MCP tools disponibles et leur statut."""
    import httpx
    tools_status = {}
    services = {
        "scrapling": "http://localhost:8800/health",
        "kroki": "http://localhost:8700/health",
        "supabase": "http://localhost:8900/health",
        "serena": "http://localhost:8765/health",
    }
    for name, health_url in services.items():
        try:
            r = httpx.get(health_url, timeout=2.0)
            tools_status[name] = "online" if r.status_code < 400 else "error"
        except Exception:
            tools_status[name] = "offline"
    return {"tools": tools_status}
