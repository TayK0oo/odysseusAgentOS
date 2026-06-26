"""
AgentOS Model Routing API — /api/route

Expose le routing intelligent (ZenRouter) comme endpoint REST.
Permet au dashboard et aux agents externes de requêter le bon modèle.

Endpoints:
  POST /api/route          → route + appel LLM complet
  POST /api/route/dry-run  → classification seule (pas d'appel LLM)
  GET  /api/route/config   → dump de model-routing.json
  GET  /api/route/stats    → métriques blacklist / fail counts
"""

import time
import logging
from fastapi import APIRouter, Request
from pydantic import BaseModel
from typing import Optional

from src.zen_router import (
    classify_complexity,
    route_and_call,
    _load_routing_config,
    _zen_blacklist,
    _zen_fail_counts,
)

logger = logging.getLogger(__name__)
router = APIRouter()


class RouteRequest(BaseModel):
    prompt: str
    system: Optional[str] = ""
    stage: Optional[str] = "chat"   # chat | planning | execution | verification | research
    max_tokens: Optional[int] = 2048
    dry_run: Optional[bool] = False


class DryRunRequest(BaseModel):
    prompt: str
    stage: Optional[str] = "chat"


@router.post("/api/route")
async def route_request(body: RouteRequest):
    """
    Route une requête vers le bon modèle + appelle le LLM.
    dry_run=true → classifie seulement, sans appel LLM.
    """
    if body.dry_run:
        tier, score = classify_complexity(body.prompt)
        cfg = _load_routing_config()
        model_cfg = cfg.get("models", {}).get(tier, {})
        return {
            "dry_run": True,
            "tier": tier,
            "score": round(score, 3),
            "model": model_cfg.get("model_id", "?"),
            "stage": body.stage,
        }

    result = await route_and_call(
        prompt=body.prompt,
        system=body.system or "",
        stage=body.stage or "chat",
        max_tokens=body.max_tokens or 2048,
    )
    return result


@router.post("/api/route/dry-run")
async def dry_run(body: DryRunRequest):
    """Classification seule — 0 token LLM."""
    tier, score = classify_complexity(body.prompt)
    cfg = _load_routing_config()
    model_cfg = cfg.get("models", {}).get(tier, {})
    return {
        "tier": tier,
        "score": round(score, 3),
        "model": model_cfg.get("model_id", "?"),
        "stage": body.stage,
        "category": model_cfg.get("category", "?"),
    }


@router.get("/api/route/config")
async def get_routing_config():
    """Dump de model-routing.json (sans les clés API)."""
    cfg = _load_routing_config()
    # Masquer les env vars des clés
    safe = dict(cfg)
    if "providers" in safe:
        safe["providers"] = {
            k: {**v, "api_key_env": v.get("api_key_env", "")}
            for k, v in safe["providers"].items()
        }
    return safe


@router.get("/api/route/stats")
async def get_routing_stats():
    """État du router : blacklist, fail counts, modèles actifs."""
    now = time.time()
    blacklisted = {
        model: round(expiry - now, 1)
        for model, expiry in _zen_blacklist.items()
        if expiry > now
    }
    return {
        "blacklisted_models": blacklisted,
        "fail_counts": dict(_zen_fail_counts),
        "router": "zen_router v1.0",
        "status": "ok" if not blacklisted else "degraded",
    }
