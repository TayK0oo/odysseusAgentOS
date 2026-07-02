"""
Knowledge routes — Orchestration de la Trinité (CBM + VectorRAG natif + Obsidian).
Checkpoint : CBM (structure code) → VectorRAG natif (sémantique doc) → Obsidian.

La recherche sémantique doc est déléguée au ``VectorRAG`` natif (ChromaDB) au
lieu du service Graphify fantôme (jamais démarré) — zéro redondance.
"""
from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional, List, Dict, Any
import httpx
import logging

from src.rag_singleton import get_rag_manager

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/knowledge", tags=["knowledge"])

CBM_URL = "http://localhost:9749"
OBSIDIAN_URL = "http://localhost:9751"  # Obsidian MCP


def _native_semantic_search(query: str, limit: int = 5) -> Dict[str, Any]:
    """Semantic doc search via the native VectorRAG (replaces phantom Graphify)."""
    rag = get_rag_manager()
    if rag is None:
        return {"results": [], "error": "native RAG unavailable"}
    try:
        hits = rag.search(query, k=limit)
        return {"results": hits, "source": "native-vectorrag"}
    except Exception as e:
        return {"results": [], "error": str(e)}

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

# ---- Sémantique doc — VectorRAG natif (remplace Graphify) ----

@router.get("/graph/search")
async def search_graph_semantic(q: str, limit: int = 10):
    """Recherche sémantique doc — déléguée au VectorRAG natif."""
    return _native_semantic_search(q, limit)

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
    results = {"cbm": None, "semantic": None, "obsidian": None, "query": query}

    # 2. Sémantique doc — VectorRAG natif (synchrone, pas de HTTP)
    results["semantic"] = _native_semantic_search(query, 5)

    async with httpx.AsyncClient(timeout=15.0) as client:
        # 1. CBM — structure code
        try:
            r = await client.post(f"{CBM_URL}/search_graph", json={"name_pattern": query, "limit": 5})
            results["cbm"] = r.json()
        except Exception:
            results["cbm"] = {"error": "CBM offline"}

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
    status = {"rag": "online" if get_rag_manager() is not None else "offline"}
    async with httpx.AsyncClient(timeout=3.0) as client:
        for name, url in [("cbm", CBM_URL), ("obsidian", OBSIDIAN_URL)]:
            try:
                await client.get(f"{url}/health")
                status[name] = "online"
            except Exception:
                status[name] = "offline"
    return {"trinite": status}
