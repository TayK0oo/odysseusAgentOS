"""
Governance routes — budgets, heartbeats, goal-ancestry.
Phase 9 : AgentOS governance layer.
"""
import logging
from typing import Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from core.database import SessionLocal, Mission, Goal, GoalProject
from src.governance import GovernanceManager

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/governance", tags=["governance"])

# Shared manager instance (uses the app SessionLocal)
_manager = GovernanceManager(db_session_factory=SessionLocal)


# ---- Pydantic models ----

class HeartbeatUpdate(BaseModel):
    task_id: Optional[str] = None
    context_snapshot: dict = {}
    checklist: list = []
    last_action: str = ""
    next_run_in_seconds: int = 300


class CreateTaskRequest(BaseModel):
    project_id: str
    task_name: str
    description: str = ""
    agent_id: str


# ---- Routes ----

@router.get("/budgets/{agent_id}")
async def get_budget_status(agent_id: str, run_id: Optional[str] = None):
    """Statut budget d'un agent. Passe ?run_id=... pour un run spécifique."""
    if run_id:
        status = _manager.get_budget_status(run_id)
        if status is None:
            raise HTTPException(status_code=404, detail=f"run_id '{run_id}' not found")
        return status

    return {
        "agent_id": agent_id,
        "is_paused": _manager.is_agent_paused(agent_id),
        "message": "Pass ?run_id=<id> for per-run details",
    }


@router.post("/heartbeat/{agent_id}")
async def update_heartbeat(agent_id: str, body: HeartbeatUpdate):
    """Met à jour le heartbeat d'un agent (contexte Memento Man)."""
    try:
        _manager.update_heartbeat(
            agent_id=agent_id,
            task_id=body.task_id,
            context_snapshot=body.context_snapshot,
            checklist=body.checklist,
            last_action=body.last_action,
            next_run_in_seconds=body.next_run_in_seconds,
        )
        return {"ok": True, "agent_id": agent_id}
    except Exception as exc:
        logger.exception("Failed to update heartbeat for agent %s", agent_id)
        raise HTTPException(status_code=500, detail=str(exc))


@router.get("/heartbeat/{agent_id}/context")
async def get_heartbeat_context(agent_id: str):
    """Contexte à injecter au réveil de l'agent (pattern Memento Man)."""
    ctx = _manager.get_heartbeat_context(agent_id)
    if ctx is None:
        raise HTTPException(status_code=404, detail=f"No heartbeat found for agent '{agent_id}'")
    return ctx


@router.get("/goals")
async def list_goals():
    """Liste des missions, goals et projets."""
    with SessionLocal() as db:
        missions = db.query(Mission).filter(Mission.active == True).all()
        result = []
        for m in missions:
            m_data = {
                "id": m.id,
                "name": m.name,
                "description": m.description,
                "created_at": m.created_at.isoformat() if m.created_at else None,
                "goals": [],
            }
            for g in m.goals:
                g_data = {
                    "id": g.id,
                    "name": g.name,
                    "status": g.status,
                    "description": g.description,
                    "projects": [
                        {
                            "id": p.id,
                            "name": p.name,
                            "status": p.status,
                            "project_dir": p.project_dir,
                        }
                        for p in g.projects
                    ],
                }
                m_data["goals"].append(g_data)
            result.append(m_data)
        return {"missions": result}


@router.post("/goals/task")
async def create_task(body: CreateTaskRequest):
    """Crée une tâche GoalTask avec ancestry_path complet."""
    result = _manager.create_task_with_ancestry(
        project_id=body.project_id,
        task_name=body.task_name,
        description=body.description,
        agent_id=body.agent_id,
    )
    if not result.get("ok"):
        raise HTTPException(status_code=404, detail=result.get("error", "unknown error"))
    return result
