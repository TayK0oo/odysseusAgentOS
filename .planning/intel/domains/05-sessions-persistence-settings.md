# Domain 05 — Sessions / Persistence / Settings / Config

Native Odysseus (forked OpenCode) persistence & config map, so the grafted
"intelligence layer" (governance / trace / observer) never re-implements what
already exists. All persistence is **SQLite via SQLAlchemy** (single `app.db`)
plus a handful of JSON config files under `DATA_DIR`.

Engine + models: `core/database.py`. DB URL default
`sqlite:///{DATA_DIR}/app.db` (`_default_database_url` L41; override via
`DATABASE_URL` env). `PRAGMA foreign_keys=ON` (L71). One declarative `Base`;
`init_db()` creates every table at import (L2476). Column-level encryption via
`EncryptedText` TypeDecorator (L79) → `src/secret_storage.py` (Fernet, key at
`data/.app_key`).

---

## 1. Native session / message model & persistence

**Files:** `core/session_manager.py` (SessionManager, business logic + DB),
`core/models.py` (in-memory `Session`/`ChatMessage` dataclasses + singleton
accessors), `core/database.py` (ORM), `routes/session_routes.py`,
`routes/history_routes.py`, `src/session_search.py`, `src/session_actions.py`.

**Tables (`core/database.py`):**
- `sessions` (L104 `Session`): `id, name, endpoint_url, model, owner, rag,
  archived, folder, headers(JSON), created_at/updated_at(TimestampMixin),
  last_accessed, last_message_at, is_important, message_count,
  total_input_tokens, total_output_tokens, mode('agent'|'chat'|'research'),
  crew_member_id`. Indexes `ix_sessions_active`, `ix_sessions_search`.
- `chat_messages` (L183 `ChatMessage`): `id, session_id(FK→sessions CASCADE),
  role, content(Text), meta_data(col "metadata", JSON string — holds
  **per-message metrics**, `_db_id`, `timestamp`), timestamp`. Index
  `ix_messages_session_time`.

**Lifecycle (SessionManager):** `load_sessions` (L70, boots metadata-only,
top-100 by last_accessed), `get_session` (L363, hydrates messages on demand),
`_db_to_session`/`_db_to_session_meta` (L127/L103), `create_session` (L459),
`delete_session` (L505, detaches docs, cascades messages), `archive_session`
(L567), `update_session_name` (L547), `mark_important` (L587),
`truncate_messages` (L267), `sync_session_metadata` (L386). `save_sessions`
(L621) is a **no-op** (DB is authoritative). In-memory cache = `self.sessions`.
Singleton wired via `set_session_manager_instance`; also injected into
`src/ai_interaction.py` and `src/assistant_log.py`.

**Message persist:** `add_message` (L190) → `_persist_message` (L209): writes a
`chat_messages` row, serializes `message.metadata` to `meta_data` JSON, stamps
`last_accessed` + `last_message_at`, updates `message_count`. Multimodal content
(list) is JSON-serialized into the Text column.

**Transcript search:** `src/session_search.py` uses **SQLite FTS5** over
`chat_messages` (`_has_fts_table` L116) with context-window snippets — shared by
UI search and the agent search tool.

**Detached streaming:** `src/agent_runs.py` is NATIVE (English docstrings) — an
**in-memory** SSE replay-buffer manager (`_RUNS` dict, `_EVICT_GRACE_S=180`) that
keeps an agent/chat stream alive server-side after the SSE client disconnects.
It does **not** persist metrics; it is NOT the grafted layer despite its name.

---

## 2. Native settings / prefs system

**File:** `src/settings.py` (single source of truth). Paths in
`src/constants.py`: `DATA_DIR` (`ODYSSEUS_DATA_DIR`), `SETTINGS_FILE`
(settings.json), `FEATURES_FILE` (features.json), `USER_PREFS_FILE`
(user_prefs.json), `AUTH_FILE`, `MEMORY_FILE`, `PRESETS_FILE`.

- Global settings: `load_settings`/`save_settings` (L213/231, atomic write via
  `core/atomic_io.atomic_write_json`), `get_setting` (L238),
  `is_setting_overridden` (L243). 2 s TTL cache (L20). `DEFAULT_SETTINGS` (L31)
  is a large registry: model routing (`default_endpoint_id/model` +
  `*_fallbacks`, `utility_*`, `research_*`, `task_*`, `vision_*`, `image_*`),
  agent budgets (`agent_max_rounds`, `agent_input_token_budget/_hard_max`,
  `agent_max_tool_calls`, `agent_stream_timeout_seconds`), search providers,
  TTS/STT, reminders, keybinds.
- Feature flags: `load_features`/`save_features` (L298/316), `DEFAULT_FEATURES`
  (L199: web_search, deep_research, memory, rag, gallery, …).
- Per-user prefs: `get_user_setting(key, owner)` (L277) resolves from
  `routes/prefs_routes._load_for_user(owner)` (user_prefs.json) first, falling
  back to global — gated to whitelist `_PER_USER_KEYS` (L265: vision/image/
  default/utility/research model+endpoint).
- Personas/presets: `src/preset_manager.py` (`PresetManager.DEFAULT_PRESETS`
  L9: code_analyze/brainstorm/reason/… temperature+system_prompt, persisted to
  presets.json); `src/reminder_personas.py`. DB `crew_members` table also holds
  personas.

**Endpoints/models are DB-stored, NOT in settings.json:** `model_endpoints`
(L361, admin-configured, `api_key` EncryptedText, cached/pinned/hidden models,
`supports_tools`, `owner`), `provider_auth_sessions` (L397, encrypted OAuth
tokens), `mcp_servers` (L411, `oauth_tokens` EncryptedText). settings.json only
stores which endpoint_id/model is *selected* per role.

**Secrets:** `src/secret_storage.py` (Fernet `encrypt`/`decrypt`, `enc:` prefix,
idempotent, key `data/.app_key` 0o600); `EncryptedText` columns:
model_endpoints.api_key, provider_auth_sessions.access/refresh_token,
mcp_servers.oauth_tokens, email_accounts.imap/smtp_password, signatures.
`src/api_key_manager.py`, `src/settings_scrub.py` handle key redaction.

---

## 3. Native metrics / agent-run logging

- **Per-message metrics** are the native store: token counts + timing live in
  `chat_messages.meta_data` (JSON) and are rolled up onto
  `sessions.total_input_tokens/total_output_tokens/message_count`.
- **Scheduled-task runs:** `task_runs` table (L651): `task_id(FK), started_at,
  finished_at, status(running|success|error), result, error, tokens_used,
  steps(JSON tool-call log), model`. Backref `ScheduledTask.runs`. This is the
  native per-run execution record (tokens + tool-call steps).
- `scheduled_tasks` (L573): full task/trigger/cron config + `run_count`.
- `src/agent_runs.py` — see §1: in-memory SSE only, no persistence.
- `src/assistant_log.py` — `log_to_assistant` is a **legacy no-op** (L41);
  activity moved to Tasks/notifications. Do not build on it.

---

## 4. Grafted governance / trace / observer vs native — verdict

Grafted tables in `core/database.py` (French comments, ancestry, L2360-2470):
`missions, goals, goal_projects, goal_tasks(ancestry_path), agent_budgets,
agent_heartbeats`. All wired (governance_routes, agent_loop, tool_execution).

| Module | What it adds | Native equivalent | Verdict |
|---|---|---|---|
| `src/governance.py` GovernanceManager — **budgets** (`agent_budgets`: max_tokens/cost/iterations, auto-pause, `consume_budget`) | Native `settings.agent_*` caps are *config only* (soft trim / round cap), no per-run persisted spend counter with auto-pause. `task_runs.tokens_used` records but doesn't enforce. | **PARTIAL** — enforcement+persistence is new; token accounting overlaps `task_runs`/message metrics (dedupe the source of truth). |
| `src/governance.py` — **heartbeat** (`agent_heartbeats`: context_snapshot, checklist, next_run_at) | Native has none — scheduler (`scheduled_tasks.next_run`) is time-based, not agent-resumable state. | **UNIQUE** — but note conceptual overlap with scheduler's next_run scheduling. |
| `src/governance.py` — **goal ancestry** (`missions/goals/goal_projects/goal_tasks`) | Native has NO project/goal hierarchy. `sessions.folder`, `crew_members`, `scheduled_tasks` are flat. | **UNIQUE**. |
| `src/governance.py` — **approval gates** (`request_approval`, log-only) | Native `settings.agent_email_confirm` + `scheduled_emails` draft-staging is a real per-action approval flow for email. Governance gate is generic but only logs (no persistence/queue). | **PARTIAL** — generic gate is new; email-approval pattern already exists natively, reuse it. |
| `src/trace_writer.py` — **JSONL tool traces** (`data/traces/YYYY-MM-DD.jsonl`: ts, run_id, session_id, tool, risk_level, permission_decision, outcome, cost_tokens, duration_ms) | Native records tool-call steps in `task_runs.steps` (JSON) and message meta_data — but only for scheduled tasks / assistant turns, in-DB, not a uniform per-tool audit trail. | **PARTIAL** — the flat per-tool audit-log granularity + risk/permission fields are new; cost_tokens/session_id duplicate what message meta_data already has. |
| `src/observer.py` — **drift score** (aggregates budget + CodeBurn + harness-touch, in-memory only) | No native equivalent. | **UNIQUE** (ephemeral, not persisted). |

---

## 5. Risks / notes

- **Token-accounting duplication (highest risk):** tokens live in THREE places —
  `chat_messages.meta_data`, `sessions.total_*_tokens`, `task_runs.tokens_used`,
  and now `agent_budgets.tokens_used` + trace `cost_tokens`. Pick ONE writer of
  truth (message metrics) and have governance/trace *read* it, not re-count.
- **`run_id` fragmentation:** trace_writer uses a **process-level** `_RUN_ID`
  (one UUID per server start, L28), governance `run_id` is caller-supplied,
  `agent_runs` keys by `session_id`, `task_runs.id` is per execution. No shared
  correlation id — cross-referencing traces↔budgets↔task_runs is currently
  impossible. Define a single run/correlation id if the intelligence layer needs
  to join them.
- **Grafted tables use `uuid4` PK defaults** (L2362) while native tables expect
  caller-supplied string ids — keep consistent to avoid FK surprises.
- Governance approval gate and Observer drift are **not persisted** (log / RAM
  only) — any dashboard/audit requirement will need a new table; don't assume
  they survive restart.
- `save_sessions`/`assistant_log` are no-ops — do not hook persistence there.
- Endpoints/models/secrets are DB+encrypted, NOT settings.json — grafted config
  should follow the same pattern (DB row + `EncryptedText`), not new JSON files.
