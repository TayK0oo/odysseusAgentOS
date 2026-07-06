"""Graphify MCP server — semantic knowledge graph (3rd leg of the Trinité).

Exposes graphify analysis tools via MCP stdio. Serves as the semantic
layer of the Trinité (CBM structure + Graphify semantics + Obsidian memory).

Registered only when ODYSSEUS_GRAPHIFY=on (default OFF), byte-identical
startup otherwise.
"""
import asyncio
import json
import os
import sys
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Graphify client — wraps the graphifyy Python package
# ---------------------------------------------------------------------------

_GRAPHIFY_AVAILABLE = False
try:
    from graphify import Graphify
    _GRAPHIFY_AVAILABLE = True
except ImportError:
    pass


def _get_graphify():
    """Lazy-init Graphify instance. Returns None if package not installed."""
    if not _GRAPHIFY_AVAILABLE:
        return None
    workspace = os.environ.get("GRAPHIFY_WORKSPACE", os.getcwd())
    return Graphify(workspace=workspace)


def _ensure_available():
    """Raise if graphify is not installed."""
    if not _GRAPHIFY_AVAILABLE:
        raise RuntimeError(
            "graphifyy package not installed. Run: pip install graphifyy"
        )


# ---------------------------------------------------------------------------
# Tool implementations
# ---------------------------------------------------------------------------

def analyze_codebase(path: Optional[str] = None, max_depth: int = 5) -> dict:
    """Build a semantic knowledge graph from a codebase.

    Args:
        path: Directory to analyze (default: current workspace).
        max_depth: Max directory depth for analysis.

    Returns:
        Dict with keys: nodes, edges, communities, stats.
    """
    _ensure_available()
    gf = _get_graphify()
    if gf is None:
        return {"error": "Graphify init failed — check GRAPHIFY_WORKSPACE env var."}

    target = path or os.getcwd()
    try:
        result = gf.analyze(target, max_depth=max_depth)
        return {
            "nodes": getattr(result, "node_count", 0),
            "edges": getattr(result, "edge_count", 0),
            "communities": getattr(result, "community_count", 0),
            "stats": getattr(result, "stats", {}),
            "path": target,
        }
    except Exception as exc:
        return {"error": f"Graphify analysis failed: {exc}"}


def search_knowledge(query: str, limit: int = 10) -> dict:
    """Semantic search across the knowledge graph.

    Args:
        query: Natural-language search query.
        limit: Max results (default 10).

    Returns:
        Dict with keys: results (list of matched nodes with scores).
    """
    _ensure_available()
    gf = _get_graphify()
    if gf is None:
        return {"error": "Graphify not initialized — run analyze_codebase first."}

    try:
        results = gf.search(query, top_k=limit)
        return {
            "query": query,
            "results": [
                {"name": r.get("name", ""), "type": r.get("type", ""),
                 "file": r.get("file", ""), "score": r.get("score", 0)}
                for r in (results or [])
            ],
        }
    except Exception as exc:
        return {"error": f"Graphify search failed: {exc}"}


def explain_relationship(source: str, target: str) -> dict:
    """Explain the semantic relationship between two nodes in the graph.

    Args:
        source: Name of the source node.
        target: Name of the target node.

    Returns:
        Dict with keys: path (list of nodes connecting source→target), explanation.
    """
    _ensure_available()
    gf = _get_graphify()
    if gf is None:
        return {"error": "Graphify not initialized."}

    try:
        explanation = gf.explain(source, target)
        return {
            "source": source,
            "target": target,
            "path": explanation.get("path", []),
            "explanation": explanation.get("explanation", ""),
        }
    except Exception as exc:
        return {"error": f"Graphify explain failed: {exc}"}


def get_graph_status() -> dict:
    """Return the current state of the knowledge graph."""
    if not _GRAPHIFY_AVAILABLE:
        return {"available": False, "error": "graphifyy package not installed"}

    gf = _get_graphify()
    if gf is None:
        return {"available": False, "error": "Graphify init failed"}

    try:
        info = gf.info() if hasattr(gf, "info") else {}
        return {
            "available": True,
            "nodes": info.get("nodes", 0),
            "edges": info.get("edges", 0),
            "communities": info.get("communities", 0),
            "workspace": os.environ.get("GRAPHIFY_WORKSPACE", os.getcwd()),
        }
    except Exception as exc:
        return {"available": True, "error": f"Status query failed: {exc}"}


# ---------------------------------------------------------------------------
# MCP tool registry
# ---------------------------------------------------------------------------

TOOLS = {
    "graphify_analyze": analyze_codebase,
    "graphify_search": search_knowledge,
    "graphify_explain": explain_relationship,
    "graphify_status": get_graph_status,
}


# ---------------------------------------------------------------------------
# MCP stdio server
# ---------------------------------------------------------------------------

from mcp.server import Server  # noqa: E402
from mcp.server.stdio import stdio_server  # noqa: E402
from mcp.types import Tool, TextContent  # noqa: E402

server = Server("graphify")


def _text_result(text: str) -> list[TextContent]:
    return [TextContent(type="text", text=text)]


@server.list_tools()
async def list_tools() -> list[Tool]:
    return [
        Tool(
            name="graphify_analyze",
            description="Build a semantic knowledge graph from a codebase directory. "
            "Extracts AST structure + semantic relationships via LLM subagents. "
            "Call this ONCE at session start to index the project.",
            inputSchema={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory to analyze (default: current workspace).",
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Max directory depth (default: 5).",
                        "default": 5,
                    },
                },
            },
        ),
        Tool(
            name="graphify_search",
            description="Semantic search across the knowledge graph. "
            "Finds functions, classes, and concepts by meaning (not just name). "
            "Use for: 'how does X work?', 'what calls Y?', 'find all authentication code'.",
            inputSchema={
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural-language search query.",
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results (default: 10).",
                        "default": 10,
                    },
                },
                "required": ["query"],
            },
        ),
        Tool(
            name="graphify_explain",
            description="Explain the semantic relationship between two nodes in the graph. "
            "Traces the path connecting source→target and returns a natural explanation.",
            inputSchema={
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "Name of the source node.",
                    },
                    "target": {
                        "type": "string",
                        "description": "Name of the target node.",
                    },
                },
                "required": ["source", "target"],
            },
        ),
        Tool(
            name="graphify_status",
            description="Return the current state of the knowledge graph (node count, edge count, communities).",
            inputSchema={
                "type": "object",
                "properties": {},
            },
        ),
    ]


@server.call_tool()
async def call_tool(name: str, arguments: dict) -> list[TextContent]:
    if name == "graphify_analyze":
        result = analyze_codebase(
            path=arguments.get("path"),
            max_depth=arguments.get("max_depth", 5),
        )
    elif name == "graphify_search":
        result = search_knowledge(
            query=arguments.get("query", ""),
            limit=arguments.get("limit", 10),
        )
    elif name == "graphify_explain":
        result = explain_relationship(
            source=arguments.get("source", ""),
            target=arguments.get("target", ""),
        )
    elif name == "graphify_status":
        result = get_graph_status()
    else:
        return _text_result(f"Unknown tool: {name}")
    return _text_result(json.dumps(result, ensure_ascii=False, indent=2))


async def run():
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, server.create_initialization_options())


if __name__ == "__main__":
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    asyncio.run(run())
