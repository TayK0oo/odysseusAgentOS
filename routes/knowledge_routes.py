"""
Knowledge routes — Orchestration de la Trinité (CBM + Graphify + Obsidian).
Checkpoint obligatoire avant toute génération : CBM → Graphify → Obsidian.
"""
from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional, List, Dict, Any
import httpx
import logging

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/knowledge", tags=["knowledge"])

CBM_URL = "http://localhost:9749"
GRAPHIFY_URL = "http://localhost:9750"  # Graphify sur ce port (configurable)
OBSIDIAN_URL = "http://localhost:9751"  # Obsidian MCP

# ---- CBM — Structure code ----

@router.get("/code/search")
async def search_code_graph(q: str, limit: int = 10):
    """CBM search_graph — trouve des fonctions/classes/routes par nom."""
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.post(f"{CBM_URL}/search_graph", json={"name_pattern": q, "limit": limit})
            return resp.json()
    except Exception as e:
        return {"error": str(e), "hint": "Démarrer avec: docker compose --profile knowledge up -d"}

@router.get("/code/trace")
async def trace_code_path(fn: str, mode: str = "calls"):
    """CBM trace_path — trace les appels/dépendances d'une fonction."""
    try:
        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.post(f"{CBM_URL}/trace_path", json={"function_name": fn, "mode": mode})
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

@router.get("/code/snippet")
async def get_code_snippet(qn: str):
    """CBM get_code_snippet — source d'une fonction par qualified name."""
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.post(f"{CBM_URL}/get_code_snippet", json={"qualified_name": qn})
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

@router.get("/code/architecture")
async def get_architecture(aspects: str = "all"):
    """CBM get_architecture — vue d'ensemble du projet."""
    try:
        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.post(f"{CBM_URL}/get_architecture", json={"aspects": aspects.split(",")})
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

# ---- Graphify — Sémantique ----

@router.post("/graph/index")
async def index_project(project_path: str = "."):
    """Graphify — indexe le projet (on-demand). Lance l'analyse sémantique."""
    try:
        async with httpx.AsyncClient(timeout=300.0) as client:
            resp = await client.post(f"{GRAPHIFY_URL}/index", json={"path": project_path})
            return resp.json()
    except Exception as e:
        return {"error": str(e), "hint": "Démarrer avec: docker compose --profile knowledge up -d"}

@router.get("/graph/search")
async def search_graph_semantic(q: str, limit: int = 10):
    """Graphify — recherche sémantique dans le graphe de connaissances."""
    try:
        async with httpx.AsyncClient(timeout=15.0) as client:
            resp = await client.get(f"{GRAPHIFY_URL}/search", params={"q": q, "limit": limit})
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

# ---- Obsidian — Mémoire inter-projets ----

@router.get("/memory/notes")
async def list_notes(path: str = "agentos/"):
    """Obsidian MCP — liste les notes du second-brain."""
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.get(f"{OBSIDIAN_URL}/notes", params={"path": path})
            return resp.json()
    except Exception as e:
        return {"error": str(e), "hint": "Configurer le vault Obsidian dans .env"}

@router.get("/memory/note/{note_path:path}")
async def get_note(note_path: str):
    """Obsidian MCP — lit une note du second-brain."""
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.get(f"{OBSIDIAN_URL}/notes/{note_path}")
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

@router.post("/memory/note")
async def create_note(path: str, content: str, tags: List[str] = []):
    """Obsidian MCP — crée/met à jour une note dans le second-brain."""
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.post(f"{OBSIDIAN_URL}/notes", json={"path": path, "content": content, "tags": tags})
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

# ---- Checkpoint Trinité — utilisé avant génération ----

@router.post("/checkpoint")
async def trinite_checkpoint(query: str, context_type: str = "code"):
    """
    Checkpoint Trinité : interroge CBM + Graphify + Obsidian et fusionne.
    Appelé obligatoirement avant toute génération de code.
    Retourne un contexte enrichi.
    """
    results = {"cbm": None, "graphify": None, "obsidian": None, "query": query}

    async with httpx.AsyncClient(timeout=15.0) as client:
        # 1. CBM — structure code
        try:
            r = await client.post(f"{CBM_URL}/search_graph", json={"name_pattern": query, "limit": 5})
            results["cbm"] = r.json()
        except Exception:
            results["cbm"] = {"error": "CBM offline"}

        # 2. Graphify — sémantique
        try:
            r = await client.get(f"{GRAPHIFY_URL}/search", params={"q": query, "limit": 5})
            results["graphify"] = r.json()
        except Exception:
            results["graphify"] = {"error": "Graphify offline"}

        # 3. Obsidian — mémoire
        try:
            r = await client.get(f"{OBSIDIAN_URL}/search", params={"q": query})
            results["obsidian"] = r.json()
        except Exception:
            results["obsidian"] = {"error": "Obsidian offline"}

    return results

@router.get("/status")
async def knowledge_status():
    """Statut de tous les composants de la Trinité."""
    async with httpx.AsyncClient(timeout=3.0) as client:
        status = {}
        for name, url in [("cbm", CBM_URL), ("graphify", GRAPHIFY_URL), ("obsidian", OBSIDIAN_URL)]:
            try:
                r = await client.get(f"{url}/health")
                status[name] = "online"
            except Exception:
                status[name] = "offline"
    return {"trinite": status}
