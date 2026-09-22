# core/route_loader.py
"""Register all API routes on the FastAPI app.

Reads component managers and services from ``app.state`` so that the
FastAPI entrypoint stays thin (~100 lines).
"""

import logging

from fastapi import FastAPI, Request
from fastapi.responses import RedirectResponse

logger = logging.getLogger(__name__)

HTTPException = None  # deferred import


def register_all_routes(app: FastAPI):
    """Mount every API router and SPA route onto *app*."""
    import os as _os

    global HTTPException
    from fastapi import HTTPException as _HTTPException

    HTTPException = _HTTPException

    # ── helpers ──────────────────────────────────────────────────────
    from src.app_helpers import abs_join, serve_html_with_nonce

    _app = app
    _state = app.state
    _auth_manager = _state.auth_manager
    _session_manager = _state.session_manager
    _memory_manager = _state.memory_manager
    _memory_vector = getattr(_state, "memory_vector", None)
    _upload_handler = _state.upload_handler
    _personal_docs_mgr = _state.personal_docs_manager
    _api_key_manager = _state.api_key_manager
    _preset_manager = _state.preset_manager
    _chat_processor = _state.chat_processor
    _research_handler = _state.research_handler
    _chat_handler = _state.chat_handler
    _model_discovery = _state.model_discovery
    _skills_manager = _state.skills_manager
    _tts_service = _state.tts_service
    _stt_service = _state.stt_service
    _task_scheduler = _state.task_scheduler
    _mcp_manager = _state.mcp_manager
    _webhook_manager = _state.webhook_manager
    _rag_manager = getattr(_state, "rag_manager", None)
    _rag_available = getattr(_state, "rag_available", False)
    _config = _state.config

    from core.constants import (
        BASE_DIR,
        OPENAI_API_KEY,
        REQUEST_TIMEOUT,
        SESSIONS_FILE,
    )

    # ── routers ──────────────────────────────────────────────────────
    # Auth
    from routes.auth_routes import setup_auth_routes

    app.include_router(setup_auth_routes(_auth_manager))

    # Uploads
    from routes.upload_routes import setup_upload_routes

    _upload_router, _upload_cleanup_func = setup_upload_routes(_upload_handler)
    app.include_router(_upload_router)
    app.state.upload_cleanup_func = _upload_cleanup_func

    # Emoji SVG proxy
    from routes.emoji_routes import setup_emoji_routes

    app.include_router(setup_emoji_routes())

    # Sessions
    from routes.session_routes import setup_session_routes

    _session_config = {
        "REQUEST_TIMEOUT": REQUEST_TIMEOUT,
        "OPENAI_API_KEY": OPENAI_API_KEY,
        "SESSIONS_FILE": SESSIONS_FILE,
    }
    app.include_router(setup_session_routes(_session_manager, _session_config, webhook_manager=_webhook_manager))

    # Admin Danger Zone wipes
    from routes.admin_wipe_routes import setup_admin_wipe_routes

    app.include_router(setup_admin_wipe_routes(_session_manager))

    # Memory
    from routes.memory_routes import setup_memory_routes

    _memory_router = setup_memory_routes(_memory_manager, _session_manager, memory_vector=_memory_vector)
    app.include_router(_memory_router)

    # Skills
    from routes.skills_routes import setup_skills_routes

    app.include_router(setup_skills_routes(_skills_manager))

    # Chat
    from routes.chat_routes import setup_chat_routes

    app.include_router(
        setup_chat_routes(
            _session_manager,
            _chat_handler,
            _chat_processor,
            _memory_manager,
            _research_handler,
            _upload_handler,
            memory_vector=_memory_vector,
            webhook_manager=_webhook_manager,
            skills_manager=_skills_manager,
        )
    )

    # Research
    from routes.research_routes import setup_research_routes

    app.include_router(setup_research_routes(_research_handler, session_manager=_session_manager))

    # History
    from routes.history_routes import setup_history_routes

    app.include_router(setup_history_routes(_session_manager))

    # Search
    from routes.search_routes import setup_search_routes

    app.include_router(setup_search_routes(_config))

    # Presets
    from routes.preset_routes import setup_preset_routes

    app.include_router(setup_preset_routes(_preset_manager))

    # Diagnostics
    from routes.diagnostics_routes import setup_diagnostics_routes

    app.include_router(setup_diagnostics_routes(_rag_manager, _rag_available, _research_handler, _memory_vector))

    # Cleanup
    from routes.cleanup_routes import setup_cleanup_routes

    app.include_router(setup_cleanup_routes(_session_manager))

    # Personal docs
    from routes.personal_routes import setup_personal_routes

    app.include_router(setup_personal_routes(_personal_docs_mgr, _rag_manager, _rag_available))

    # Embedding model management
    from routes.embedding_routes import setup_embedding_routes

    app.include_router(setup_embedding_routes())

    # Models
    from routes.model_routes import setup_model_routes

    app.include_router(setup_model_routes(_model_discovery))

    # GitHub Copilot device-flow login
    from routes.copilot_routes import setup_copilot_routes

    app.include_router(setup_copilot_routes())

    # ChatGPT Subscription
    from routes.chatgpt_subscription_routes import setup_chatgpt_subscription_routes

    app.include_router(setup_chatgpt_subscription_routes())

    # TTS
    from routes.tts_routes import setup_tts_routes

    app.include_router(setup_tts_routes(_tts_service))

    # STT
    from routes.stt_routes import setup_stt_routes

    app.include_router(setup_stt_routes(_stt_service))

    # Documents (artifacts/canvas)
    from routes.document_routes import setup_document_routes

    _document_router = setup_document_routes(_session_manager, _upload_handler)
    app.include_router(_document_router)

    # Signatures
    from routes.signature_routes import setup_signature_routes

    app.include_router(setup_signature_routes())

    # Gallery
    from routes.gallery_routes import setup_gallery_routes

    app.include_router(setup_gallery_routes())

    # Editor drafts
    from routes.editor_draft_routes import setup_editor_draft_routes

    app.include_router(setup_editor_draft_routes())

    # Scheduled tasks + event bus
    from routes.task_routes import setup_task_routes

    app.include_router(setup_task_routes(_task_scheduler))

    from routes.assistant_routes import setup_assistant_routes

    app.include_router(setup_assistant_routes(_task_scheduler))

    # Calendar (CalDAV)
    from routes.calendar_routes import setup_calendar_routes

    _calendar_router = setup_calendar_routes()
    app.include_router(_calendar_router)

    # Shell
    from routes.shell_routes import setup_shell_routes

    app.include_router(setup_shell_routes())

    # Cookbook
    from routes.cookbook_routes import setup_cookbook_routes

    app.include_router(setup_cookbook_routes())

    # Workspace
    from routes.workspace_routes import setup_workspace_routes

    app.include_router(setup_workspace_routes())

    # Hardware model fitting
    from routes.hwfit_routes import setup_hwfit_routes

    app.include_router(setup_hwfit_routes())

    # Model A/B Comparison
    from routes.compare_routes import setup_compare_routes

    app.include_router(setup_compare_routes(_session_manager))

    # User Preferences
    from routes.prefs_routes import setup_prefs_routes

    app.include_router(setup_prefs_routes())

    # Backup
    from routes.backup_routes import setup_backup_routes

    app.include_router(setup_backup_routes(_memory_manager, _preset_manager, _skills_manager))

    # Fonts
    from routes.font_routes import setup_font_routes

    app.include_router(setup_font_routes())

    # MCP
    from routes.mcp_routes import setup_mcp_routes

    app.include_router(setup_mcp_routes(_mcp_manager))

    # Webhooks
    from routes.webhook_routes import setup_webhook_routes

    app.include_router(setup_webhook_routes(_webhook_manager, _auth_manager, _session_manager, _api_key_manager))

    # API Tokens
    from routes.api_token_routes import setup_api_token_routes

    app.include_router(setup_api_token_routes())

    # Notes
    from routes.note_routes import setup_note_routes

    app.include_router(setup_note_routes(_task_scheduler))

    # Email
    from routes.email_routes import setup_email_routes

    _email_router_instance = setup_email_routes()
    app.include_router(_email_router_instance)

    # Codex (needs email_router, memory_router, calendar_router, document_router)
    from routes.codex_routes import setup_claude_routes, setup_codex_routes

    app.include_router(
        setup_codex_routes(
            email_router=_email_router_instance,
            memory_router=_memory_router,
            calendar_router=_calendar_router,
            document_router=_document_router,
        )
    )
    app.include_router(setup_claude_routes())

    # Vault
    from routes.vault_routes import setup_vault_routes

    app.include_router(setup_vault_routes())

    # Contacts
    from routes.contacts_routes import setup_contacts_routes

    app.include_router(setup_contacts_routes())

    # Companion
    from companion import setup_companion_routes

    app.include_router(setup_companion_routes())

    # Phase-lock
    from routes.phase_routes import router as phase_router

    app.include_router(phase_router)

    # Channel gateway
    from routes.channel_routes import router as channel_router

    app.include_router(channel_router)

    # Governance
    from routes.governance_routes import router as governance_router

    app.include_router(governance_router)

    # Autoeval loop
    from routes.autoeval_routes import router as autoeval_router

    app.include_router(autoeval_router)

    # Kill-switches dashboard
    from routes.killswitch_routes import router as killswitch_router

    app.include_router(killswitch_router)

    # n8n workflow automation
    from routes.n8n_routes import router as n8n_router

    app.include_router(n8n_router)

    # MCP external tools
    from routes.mcp_tools_routes import router as mcp_tools_router

    app.include_router(mcp_tools_router)

    # Knowledge — Trinité CBM + Graphify + Obsidian
    from routes.knowledge_routes import router as knowledge_router

    app.include_router(knowledge_router)

    # Observer — drift visibility
    from routes.observer_routes import router as observer_router

    app.include_router(observer_router)

    # ── Agent catalog inline router ─────────────────────────────────
    from fastapi import APIRouter as _APIRouter

    _agents = _APIRouter(prefix="/api/agents", tags=["agents"])

    @_agents.get("")
    async def list_agents():
        catalog = getattr(app.state, "agent_catalog", None)
        if catalog is None:
            return {"agents": [], "hint": "Set ODYSSEUS_AGENT_CATALOG=on to enable the agent catalog"}
        return {
            "agents": [
                {"name": s.name, "description": s.description, "model": s.model, "tools": s.tools}
                for s in (catalog.all() if hasattr(catalog, "all") else [])
            ]
        }

    @_agents.post("/dispatch")
    async def dispatch_agent(request: Request):
        try:
            body = await request.json()
        except Exception:
            return {"ok": False, "error": "Invalid JSON body"}

        agent_name = (body or {}).get("agent", "")
        context = (body or {}).get("context", "")
        session_id = str(getattr(request.state, "session_id", "explicit"))

        if not agent_name:
            return {"ok": False, "error": "Missing 'agent' field"}

        try:
            from src.orchestrator.agent_dispatcher import AgentDispatcher

            dispatcher = AgentDispatcher()
            result = await dispatcher.dispatch_explicit(agent_name, session_id, context)
            return {"ok": True, "agent": agent_name, "result": str(result) if result else None}
        except ValueError as ve:
            return {"ok": False, "error": str(ve)}
        except RuntimeError as re:
            return {"ok": False, "error": str(re)}
        except Exception as e:
            logger.warning(f"Agent dispatch failed: {type(e).__name__}: {e}")
            return {"ok": False, "error": f"{type(e).__name__}: {e}"}

    app.include_router(_agents)

    # ── SPA routes ──────────────────────────────────────────────────

    @app.get("/")
    async def serve_index(request: Request):
        static_path = abs_join(BASE_DIR, "static/index.html")
        if _os.path.exists(static_path):
            return serve_html_with_nonce(request, static_path)
        return serve_html_with_nonce(request, abs_join(BASE_DIR, "index.html"))

    @app.get("/notes")
    async def serve_notes(request: Request):
        return await serve_index(request)

    @app.get("/calendar")
    async def serve_calendar(request: Request):
        return await serve_index(request)

    @app.get("/cookbook")
    async def serve_cookbook(request: Request):
        return await serve_index(request)

    @app.get("/email")
    async def serve_email(request: Request):
        return await serve_index(request)

    @app.get("/memory")
    async def serve_memory(request: Request):
        return await serve_index(request)

    @app.get("/gallery")
    async def serve_gallery(request: Request):
        return await serve_index(request)

    @app.get("/tasks")
    async def serve_tasks(request: Request):
        return await serve_index(request)

    @app.get("/library")
    async def serve_library(request: Request):
        return await serve_index(request)

    @app.get("/dashboard")
    async def serve_dashboard(request: Request):
        page = abs_join(BASE_DIR, "static/dashboard.html")
        if not _os.path.isfile(page):
            raise HTTPException(status_code=404, detail="Dashboard page not found")
        return serve_html_with_nonce(request, page)

    @app.get("/backgrounds")
    async def serve_backgrounds(request: Request):
        page = abs_join(BASE_DIR, "static/backgrounds.html")
        if not _os.path.isfile(page):
            raise HTTPException(status_code=404, detail="Background sandbox page not found")
        return serve_html_with_nonce(request, page)

    @app.get("/login")
    async def serve_login(request: Request):
        AUTH_ENABLED = _os.getenv("AUTH_ENABLED", "true").lower() != "false"
        if not AUTH_ENABLED:
            return RedirectResponse(url="/", status_code=302)
        return serve_html_with_nonce(request, abs_join(BASE_DIR, "static/login.html"))

    # ── API utility routes ─────────────────────────────────────────

    @app.get("/api/version")
    async def get_version():
        from core.constants import APP_VERSION

        return {"version": APP_VERSION}

    @app.get("/api/services")
    async def public_service_health():
        """Public endpoint: consolidated health of all 15 Docker services."""
        from src.service_health import collect_service_health

        return await collect_service_health()

    @app.get("/api/health")
    async def health_check() -> dict[str, str]:
        from datetime import UTC, datetime

        return {"status": "healthy", "timestamp": datetime.now(UTC).isoformat()}

    @app.get("/api/ready")
    async def readiness_check():
        from fastapi.responses import JSONResponse as _JSONResponse

        from src.readiness import check_readiness

        result = check_readiness()
        return _JSONResponse(status_code=200 if result.get("ready") else 503, content=result)

    @app.get("/api/runtime")
    async def runtime_info() -> dict[str, object]:
        in_docker = _os.path.exists("/.dockerenv")
        if not in_docker:
            try:
                with open("/proc/1/cgroup", encoding="utf-8", errors="ignore") as fh:
                    cg = fh.read()
                in_docker = any(marker in cg for marker in ("docker", "containerd", "kubepods"))
            except Exception:
                in_docker = False
        ollama_url = (
            _os.getenv("OLLAMA_BASE_URL")
            or _os.getenv("OLLAMA_URL")
            or ("http://host.docker.internal:11434/v1" if in_docker else "http://127.0.0.1:11434/v1")
        )
        return {
            "in_docker": in_docker,
            "ollama_base_url": ollama_url,
        }

    logger.info("All routes registered")
