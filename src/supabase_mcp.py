"""
Supabase MCP client — se connecte à @supabase/mcp-server-supabase.
Mode read_only par défaut. Branching activable pour les tests.
"""
import os
import logging
import httpx
from typing import Optional

logger = logging.getLogger(__name__)

SUPABASE_MCP_URL = os.getenv("SUPABASE_MCP_URL", "http://localhost:8900")

async def supabase_query(sql: str, read_only: bool = True) -> dict:
    """Exécute une requête SQL via Supabase MCP. read_only=True par défaut."""
    if not read_only:
        logger.warning("Supabase: écriture activée — utiliser uniquement sur une branche de test")

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.post(
                f"{SUPABASE_MCP_URL}/query",
                json={"sql": sql, "read_only": read_only}
            )
            return resp.json()
    except Exception as e:
        return {"error": str(e), "sql": sql}

async def supabase_create_branch(branch_name: str) -> dict:
    """Crée une branche DB Supabase pour les tests (DB éphémère isolée)."""
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.post(
                f"{SUPABASE_MCP_URL}/branches",
                json={"name": branch_name}
            )
            return resp.json()
    except Exception as e:
        return {"error": str(e)}

SUPABASE_TOOLS = [
    {
        "name": "supabase_query",
        "description": "Exécute une requête SQL en lecture seule sur Supabase. Ne peut pas modifier les données.",
        "parameters": {
            "type": "object",
            "properties": {
                "sql": {"type": "string", "description": "Requête SQL SELECT uniquement"},
            },
            "required": ["sql"]
        }
    },
    {
        "name": "supabase_create_branch",
        "description": "Crée une branche DB Supabase isolée pour les tests (DB éphémère).",
        "parameters": {
            "type": "object",
            "properties": {
                "branch_name": {"type": "string", "description": "Nom de la branche (ex: feature-auth-test)"}
            },
            "required": ["branch_name"]
        }
    }
]
