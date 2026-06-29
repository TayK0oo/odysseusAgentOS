from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional

router = APIRouter(prefix="/api/autoeval", tags=["autoeval"])

class EvalRequest(BaseModel):
    project_dir: str
    baseline_score: float

@router.post("/evaluate")
async def run_evaluation(req: EvalRequest):
    """Lance l'autoeval sur le projet et retourne le résultat keep/revert."""
    try:
        from src.autoeval_loop import create_from_manifest
        loop = create_from_manifest(req.project_dir)
        if not loop:
            return {"error": "Pas de PROJECT.yaml avec eval_command dans ce répertoire"}

        run = loop.evaluate_last_change(req.baseline_score)
        return {
            "action": run.action_taken,
            "score_before": run.score_before,
            "score_after": run.score_after,
            "improved": run.improved,
            "git_hash": run.git_hash_before,
        }
    except Exception as e:
        return {"error": str(e)}

@router.post("/run-eval-only")
async def run_eval_only(project_dir: str, eval_command: str, metric: str = "pass_rate"):
    """Lance juste l'eval sans keep/revert — pour tester."""
    try:
        from src.autoeval_loop import AutoevalLoop
        loop = AutoevalLoop(project_dir=project_dir, eval_command=eval_command, metric=metric)
        result = loop.run_eval()
        return {"score": result.score, "success": result.success, "output": result.raw_output[:500]}
    except Exception as e:
        return {"error": str(e)}
