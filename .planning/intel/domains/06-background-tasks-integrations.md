# Domain 06 — Background / Tasks / Scheduling / Integrations / Channels

Native async/eventing/integration surface of Odysseus (fork of OpenCode). Purpose:
tell the grafted "Channel Gateway + event bus" layer what NOT to rebuild.
Citations are `file:line`. Verified via CBM graph + Read on `odysseusAgentOS`.

---

## 1. Native background / scheduler system

**Three distinct background subsystems, all wired in `app.py`:**

### 1a. `TaskScheduler` — the real cron/event task engine (`src/task_scheduler.py`, ~2100 lines)
- Instantiated `app.py:697` `TaskScheduler(session_manager)`; started at `app.py:1122`
  `await task_scheduler.start()` — gated by env `ODYSSEUS_INPROCESS_TASKS` (default on).
- `start()` (`task_scheduler.py:438`): reaps zombie `TaskRun` rows (crash recovery),
  advances overdue `next_run`, dedupes default-assistant crews, then launches
  `_loop()` + `_note_pings_loop()` as asyncio tasks.
- `_loop()` (`:656`): sleeps 10s, then polls `_check_due_tasks()` and sleeps until the
  next `ScheduledTask.next_run` (capped 1–60s). DB-backed, restart-safe.
- Task rows: `ScheduledTask` (SQLAlchemy, `core/database`) with `trigger_type` in
  {`schedule`, `event`}, `trigger_event`, `trigger_count`, `trigger_counter`, `next_run`,
  `status`, `owner`.
- **How a scheduled agent run is triggered:** `_check_due_tasks` → `_execute_task` →
  `_execute_task_locked` → dispatches by kind: `_execute_llm_task` / `_execute_research_task`
  / `_execute_checkin` / `_execute_action` / `_run_agent_loop` (recursively calls the agent
  loop). Concurrency-capped via `_executing` set + `_executing_lock` + model-slot gating
  (`_task_needs_model_slot`). Task chaining with cycle detection (`_has_chain_cycle`,
  `_run_chained`).
- **Result delivery is already multi-channel:** `_deliver_task_result` (`:in_degree 3`)
  fans out to `_deliver_via_email` and `_deliver_via_mcp`, plus in-app `add_notification` /
  `pop_notifications`. This is the closest native analogue to an outbound gateway.
- `run_task_now(task_id, force=)` (`:2077`) is the public manual/event trigger.
- Routes: `routes/task_routes.py` (`setup_task_routes(task_scheduler)`, `app.py:701`),
  `src/task_endpoint.py`, `routes/assistant_routes.py` (`:704`), `routes/note_routes.py` (`:771`).

### 1b. `bg_jobs` + `bg_monitor` — detached shell-command jobs (NOT the scheduler)
- `src/bg_jobs.py`: launches long `bash`/`cmd` commands **detached** (`launch()`), status
  derived from on-disk `.exit` file (restart-safe), bounded by `DEFAULT_MAX_RUNTIME_S=3600`.
  `refresh()`/`pending_followups()`/`mark_followed_up()` drive an idempotent state machine.
- `src/bg_monitor.py`: always-on poller (`POLL_INTERVAL_S=5`), started at `app.py:975-976`
  `start_bg_monitor()`. Drains `pending_followups()` and **auto-continues the agent**
  (`_run_followup` → `stream_agent_loop`) when a job finishes. Purpose: long installs/ffmpeg
  don't block the SSE chat stream.

### 1c. `cleanup_service` — session GC (`src/cleanup_service.py`)
- `archive_inactive_sessions` / delete after `CleanupConfig` windows (7d archive / 14d delete),
  owner-scoped. Exposed via `routes/cleanup_routes.py` (`app.py:645`). Not a self-driving loop
  here — invoked by routes; scheduler housekeeping defaults also touch it.

---

## 2. Native event bus (`src/event_bus.py`)

**Not a generic pub/sub — it is a thin counter-driven trigger for `ScheduledTask`s.**
- `fire_event(event_name, owner)` (`:33`): schedules `_handle_event` on the running loop
  (sync/async safe). `_handle_event` (`:72`) queries `ScheduledTask` where
  `trigger_type=="event"` and `trigger_event==event_name` and `status=="active"`, increments
  `trigger_counter`, and when it hits `trigger_count` calls
  `_task_scheduler.run_task_now(task.id)`. Persists `next_run=utcnow()` first so triggers
  survive a reboot.
- Wiring: `set_task_scheduler(task_scheduler)` at `app.py:698-699`. There is **no
  subscribe() API** — the only "subscriber" is the scheduler; "subscriptions" are
  event-type ScheduledTask rows in the DB.
- **Publishers (real usage, CBM inbound trace):** `routes/session_routes.py` (session created),
  `routes/chat_helpers.py::fire_message_event` (message), `routes/document_routes.py` +
  `agent_tools/document_tools.CreateDocumentTool` (document created), `routes/email_routes.py::
  _record_email_received_events`, `routes/memory_routes.py`, `services/memory/memory_extractor`
  + `skill_extractor`, `routes/research_routes.py` + `research_handler`,
  `routes/skills_routes.py`, `routes/history_routes.py`, `agent_tools/session_tools`,
  `ai_interaction.do_manage_memory`, `mcp_servers/email_server`, and the scheduler itself
  (`_execute_research_task`). Broad and already load-bearing.

---

## 3. Native integrations & webhooks

### 3a. Integrations = registered external HTTP APIs (`src/integrations.py`)
- JSON-file store (`INTEGRATIONS_FILE`), secrets encrypted at rest
  (`_encrypt_integration_secrets` / `secret_storage.encrypt`), `0o600` perms.
- CRUD: `add_integration` / `update_integration` / `delete_integration` / `get_integration`
  / `load_integrations`. Presets (`INTEGRATION_PRESETS`, `:25`): miniflux, gitea, linkding,
  homeassistant, ntfy, **discord_webhook**, vaultwarden, freshrss.
- **`execute_api_call(integration_id, method, path, ...)` (`:350`)**: generic authenticated
  httpx request against a registered integration. Auth types: header / bearer / query / basic /
  none. SSRF-ish path validation, response truncation. Surfaced to the agent as the `api_call`
  tool; `get_integrations_prompt()` injects enabled integrations into the system prompt.
- **Note:** `discord_webhook` preset already lets the agent POST to a Discord webhook URL
  today via `execute_api_call` — a native (outbound-only, no bot) Discord path.

### 3b. Webhooks = outgoing HTTP POST on events (`src/webhook_manager.py`)
- `WebhookManager` instantiated `app.py:578`, loop set `app.py:948`, closed `app.py:1199`.
  Router: `routes/webhook_routes.py` (`app.py:761`).
- `ALLOWED_EVENTS` (`:20`) = {`session.created`, `chat.completed`, `chat.message`,
  `webhook.test`}. `Webhook` DB model with per-hook `events` CSV, `secret` (HMAC-SHA256
  signed), `is_active`.
- `fire_and_forget(event, payload)` (`:233`) → `fire()` → `_deliver()` with SSRF guards
  (`_is_private_url`, DNS re-resolution, redirect disabled), signature header
  `X-Odysseus-Signature`, delivery status persisted.
- **Publishers:** `routes/session_routes.py:437` (session.created),
  `routes/chat_helpers.py:382` (chat.message) + `:1161` (chat.completed),
  `routes/webhook_routes.py:387` (chat.completed). A lint test
  (`tests/test_webhook_emitters_use_manager.py`) enforces using the manager.

---

## 4. Native external channels (email / calendar) + teacher escalation

### 4a. Email (inbound + outbound) — `routes/email_pollers.py`, `email_routes.py`, `src/email_thread_parser.py`
- `_auto_summarize_poller` (30-min cadence) scans IMAP, AI-summarizes / auto-replies /
  spam-classifies recent mail; `_scheduled_email_poller` delivers due `scheduled_emails`
  via SMTP. `_start_poller` spawned at app startup, gated by `ODYSSEUS_INPROCESS_POLLERS`.
- This is a **real, working inbound+outbound message channel** with agent processing —
  the pattern the grafted gateway is trying to generalize.

### 4b. Calendar — `src/caldav_sync.py` (+ `caldav_writeback.py`), `routes/calendar_routes.py`
- One-way CalDAV pull → local SQLite (`CalendarCal`, upsert by VEVENT UID), run on calendar
  open and from a periodic scheduler loop; sync lib run via `asyncio.to_thread`.
  Writeback module handles local→remote.

### 4c. Teacher escalation — `src/teacher_escalation.py`
- End-of-turn self-improvement loop for weak/self-hosted student models. `maybe_escalate`
  (fire-and-forget, Tier1 regex + optional Tier2 LLM eval) and `run_teacher_inline`
  (`:558`, live in-stream teacher takeover). Called from `src/agent_loop.py:3559-3560`.
  On teacher success, distills a portable `SKILL.md` via `do_manage_skills`. Gated by
  `teacher_enabled` / `teacher_model` settings. Unrelated to channels; it's an
  agent-quality loop — do not touch.

---

## 5. Grafted `channel_gateway` vs native — overlap verdict

**File:** `src/channel_gateway.py` (single file; adapters in `src/adapters/{discord,telegram}_adapter.py`).
Design: abstract `ChannelAdapter` (send + start_listening), `ChannelGateway` singleton
(`get_gateway()`) with `register_adapter`, `set_inbound_handler`, `send`, `broadcast`, `start_all`.
Router `routes/channel_routes.py` registered at `app.py:811-812` → `POST /api/channels/send`,
`/broadcast`, `GET /status`.

**Registration reality (CBM + grep):**
- `register_adapter(...)`: **called ONLY in tests** (`tests/test_e2e_smoke.py:207`). Never in
  `app.py` or any startup path. → **No adapter is ever instantiated or registered at runtime.**
- `set_inbound_handler(...)`: **defined, never called anywhere** (not even tests). → the entire
  **inbound path is dead** — `start_all()` gathers nothing and there's no handler to route to.
- `start_all()`: **never called.**
- `get_gateway()` production callers: only `routes/channel_routes.py` (REST wrappers) and
  `src/agent_loop.py:3515-3521` (broadcasts `full_response` — but guarded by
  `if _gateway._adapters:`, which is always empty, so it's a no-op today).
- `DiscordAdapter` / `TelegramAdapter` nodes are **not even in the CBM graph** (query returned
  0 rows) — freshly grafted, unindexed, and unreferenced outside their own module.

**Verdict per capability:**
| Grafted piece | vs native | Verdict |
|---|---|---|
| `ChannelGateway` outbound `send`/`broadcast` | `TaskScheduler._deliver_task_result` (email+MCP+notif), `WebhookManager`, `integrations.execute_api_call` (incl. `discord_webhook` preset) | **PARTIAL** — native already delivers to email/MCP/webhook/Discord-webhook; gateway adds a uniform abstraction but no new reachable transport yet |
| Discord/Telegram adapters | none native (bot-based two-way) | **DORMANT** — genuinely new capability BUT never registered/started; 100% inert |
| `set_inbound_handler` inbound bus | `email_pollers` (working inbound channel), `event_bus.fire_event` | **DORMANT + PARTIAL** — inbound concept is unique for Discord/Telegram, but wiring is absent; email already proves the inbound-agent pattern |
| `ChannelType.EMAIL` / `WEBHOOK` enum members | full native email pollers + webhook_manager | **REDUNDANT** — these enum arms duplicate mature native subsystems |
| Event fan-out (gateway as bus) | `event_bus.py` + `ScheduledTask(trigger_type=event)` | **REDUNDANT for eventing** — native event bus already publishes session/message/doc/email/memory/research/skill events and triggers agent runs |

**Bottom line:** The gateway's event-bus ambition is REDUNDANT (native `event_bus` +
scheduler already do event→agent-run). Its outbound abstraction is PARTIAL (native delivers
to email/MCP/webhook/Discord-webhook). Only genuinely UNIQUE bit = **two-way Discord/Telegram
bot channels**, and those adapters are **DORMANT** (defined, never registered, no inbound
handler, not started). Graft should: keep only the Discord/Telegram adapters, wire them to the
**native `event_bus.fire_event`** (inbound) and **`TaskScheduler` delivery / `execute_api_call`**
(outbound) rather than a parallel bus.

---

## 6. Risks / notes

- **Dead code shipping:** `channel_routes` is mounted (`app.py:811`) so `/api/channels/*` is
  live but always returns `{adapters: [], count: 0}` — a misleading "feature present" surface.
- **No startup wiring for adapters:** any real use needs `register_adapter` + `set_inbound_handler`
  + `start_all` added to `_startup_event` (`app.py:945`), gated like `ODYSSEUS_INPROCESS_*`.
  `discord.py` / telegram libs are optional imports (adapters degrade to warnings).
- **Two overlapping "event" concepts:** native `event_bus` (counter→ScheduledTask) is DB-backed
  and owner-scoped; the gateway has no persistence/ownership. Introducing a second bus risks
  split-brain eventing. Prefer extending `ALLOWED_EVENTS` / `event_bus` publishers.
- **Owner/tenancy:** native subsystems are strictly owner-scoped (`_resolve_event_owner`,
  webhook/integration/task rows carry `owner`). The grafted gateway is global/singleton with
  no owner concept — a multi-user leak risk if wired naively.
- **Outbound duplication:** `TaskScheduler._deliver_via_mcp/_deliver_via_email` +
  `discord_webhook` integration already cover most "notify me elsewhere" needs; the gateway's
  `broadcast` in `agent_loop.py:3521` would fire on EVERY agent turn if adapters were registered
  — likely unintended spam. Needs explicit opt-in per session/task.
- Verify at runtime with `GET /api/channels/status` (expect empty) and grep confirms
  `set_inbound_handler`/`start_all`/`register_adapter` have zero non-test callers.
