"""Acontext Memory Service — distillation de sessions agent."""
import json
import pathlib
import uvicorn
from fastapi import FastAPI
from datetime import datetime

app = FastAPI(title="Acontext Memory Service")
DATA = pathlib.Path("/data/acontext")
DATA.mkdir(parents=True, exist_ok=True)


@app.post("/api/sessions/end")
async def session_end(payload: dict = None):
    ts = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
    (DATA / f"session_{ts}.json").write_text(json.dumps(payload or {}, indent=2))
    return {"status": "distilled", "ts": ts}


@app.post("/api/sessions/complete")
async def session_complete(payload: dict = None):
    return await session_end(payload)


@app.get("/health")
async def health():
    sessions = list(DATA.glob("session_*.json"))
    return {"status": "ok", "service": "acontext", "sessions_stored": len(sessions)}


@app.get("/api/sessions")
async def list_sessions():
    sessions = sorted(DATA.glob("session_*.json"))
    return {"sessions": [s.name for s in sessions[-20:]]}


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8029)
