"""M3.1 — coordinated deprecation of /api/route*.

The routing endpoints are REDONDANT with native model routing (zen_router +
ModelEndpoint), but external REST consumers still call them, so the plan is a
COORDINATED deprecation (signal now, remove later) — never a blunt removal.
These tests pin the non-breaking signal: every /api/route* response carries an
RFC 8594 `Deprecation: true` header and a `Warning` notice, while the response
body/behaviour is unchanged.
"""
from fastapi import FastAPI
from fastapi.testclient import TestClient

from routes.routing_routes import router


def _client():
    app = FastAPI()
    app.include_router(router)
    return TestClient(app)


def test_dry_run_endpoint_marked_deprecated():
    r = _client().post("/api/route/dry-run", json={"prompt": "hello"})
    assert r.status_code == 200
    assert r.headers.get("Deprecation") == "true"
    assert "299" in r.headers.get("Warning", "")
    # Behaviour unchanged: still returns a tier classification.
    assert "tier" in r.json()


def test_stats_endpoint_marked_deprecated():
    r = _client().get("/api/route/stats")
    assert r.status_code == 200
    assert r.headers.get("Deprecation") == "true"


def test_config_endpoint_marked_deprecated():
    r = _client().get("/api/route/config")
    assert r.status_code == 200
    assert r.headers.get("Deprecation") == "true"


def test_route_dry_run_flag_marked_deprecated():
    r = _client().post("/api/route", json={"prompt": "hi", "dry_run": True})
    assert r.status_code == 200
    assert r.headers.get("Deprecation") == "true"
    assert r.json().get("dry_run") is True
