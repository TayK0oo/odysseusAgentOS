# Domain 03 — Tools System (Odysseus NATIVE capabilities)

> Map of the NATIVE tool system so the grafted intelligence layer never duplicates
> registration, gating, or policy. Project: `odysseusAgentOS`.
> CBM id: `C-Users-ttmdu-Documents-GitHub-odysseusAgentOS`.
> Grafted modules flagged **[GRAFT]**.

---

## 1. Native tool registration & schema flow (files:funcs)

Three parallel registries, kept in parity by a test:

| Concern | File | Symbol |
|---|---|---|
| Function-call schemas (sent to model) | `src/tool_schemas.py:23` | `FUNCTION_TOOL_SCHEMAS = [...]` list of `{"function": {"name", ...}}` |
| Model-facing one-line descriptions / routing | `src/tool_index.py:70+` | `TOOL_DESCRIPTIONS` dict; `get_tool_index()` `src/tool_index.py:595`, `reset_tool_index()` `:617` |
| Function-call → internal tool block | `src/tool_schemas.py:1213` | `function_call_to_tool_block()` (maps provider tool_call JSON → `block.tool_type` + `content`) |
| Native/XML/code parsing → blocks | `src/tool_parsing.py` | `parse_tool_blocks()` `:739`, `_parse_tool_call_block()` `:459`, `_parse_tool_code_block()` `:682`, `_parse_xml_direct_tool()` `:550` |
| Arg parsing (single source) | `src/tool_utils.py:44` | `_parse_tool_args()` |
| Known-name reconciliation | `src/tool_policy.py:145` | `known_tool_names()` |

- Parity is enforced: `tests/test_tool_index_schema_parity.py` asserts every schema tool has an index description.
- Some tools are **XML-only** (never in `FUNCTION_TOOL_SCHEMAS`): e.g. `manage_notes`, `generate_image`, `ask_user`, `update_plan`. This gap is why plan-mode gating has a static backstop (§3).
- Newer tools migrated to an **agent_tools registry** (`src/agent_tools/*`, `TOOL_HANDLERS`) dispatched via `_document_tool_dispatch` / `_direct_fallback` (issue #3629): `chat_with_model`, `ask_teacher`, `list_models`, `create_session`, `list_sessions`, `send_to_session`, `manage_session`, admin `manage_*`.

## 2. Native execution choke point + where grafts hook in

Single entrypoint, 17 callers: **`execute_tool_block()`** `src/tool_execution.py:521` (thin wrapper) → **`_execute_tool_block_impl()`** `src/tool_execution.py:550` (dispatch).

Order inside the wrapper `execute_tool_block` (521→547) and its post-classification block (~550 region):

1. **Constitution risk classification** (native-adjacent): `classify_tool()` + `_args_summary()` from `src/risk_classifier.py`; writes JSONL trace via `src/trace_writer.py`.
2. **[GRAFT] Destructive gate** `src/tool_execution.py:~558`: if `RiskLevel.DESTRUCTIVE`, calls `src/orchestrator/gate.should_block_destructive()` (guarded by `gate_enabled()`). Blocks catastrophic **shell** (`bash`/`python`/`run_command`) via `classify_bash()`; returns `{"blocked": True}`. Explicit destructive *tools* (`delete_file`…) NOT blocked here. Wrapped in try/except — never breaks the loop.
3. **[GRAFT] Phase-lock** `src/tool_execution.py:~583`: `ToolRegistry.get_instance().is_tool_allowed(tool_type, session_id, args)`. On deny returns `"blocked by phase-lock [PHASE]"`. `ImportError`/`Exception` → pass (fail-open).
4. Workspace bind (`_active_workspace.set(workspace)`) → delegate to `_execute_tool_block_impl`.
5. Post-exec trace write (duration, outcome).

Inside **`_execute_tool_block_impl`** (native gating, in order, `:550`+):
- misformatted-JSON-in-```python``` detection → helpful error.
- `disabled_tools` denylist reject.
- `tool_policy.blocks(tool)` (guide-only policy) reject.
- `_ADMIN_TOOLS` + `not _owner_is_admin(owner)` reject (`:293`, `_owner_is_admin` `:308`).
- `is_public_blocked_tool(tool)` + non-admin reject (`src/tool_execution.py:725`).
- then per-tool dispatch (MCP map / direct fallback / registry / do_* impls).

## 3. Native policy/security vs grafted phase-lock — overlap verdict

Native layers (`src/tool_security.py`):
- **`NON_ADMIN_BLOCKED_TOOLS`** `:14` (37 tools) → `blocked_tools_for_owner(owner)` `:199` returns these for non-admin/non-single-user. Owner-based, deployment-wide.
- **`PLAN_MODE_READONLY_TOOLS`** `:66` (allowlist, ~21 read-only tools) → `plan_mode_disabled_tools()` `:123` inverts it against `FUNCTION_TOOL_SCHEMAS ∪ _PLAN_MODE_KNOWN_MUTATORS` `:104` (static fail-closed backstop). MCP dropped entirely in plan mode.
- **`is_public_blocked_tool()`** `:154`, `owner_is_admin_or_single_user()`.
- `disabled_tools` (user/global per-request denylist, DB-backed `_migrate_add_disabled_tools`).
- `ToolPolicy` (`src/tool_policy.py`) guide-only `blocks()` denylist.
- MCP-level: `McpManager.plan_mode_blocked_mcp()` `src/mcp_manager.py:588`, `mcp_tool_is_readonly()` `:105`.

Grafted **phase-lock** (`src/tool_registry.py`): per-session `PhaseContext`, YAML-driven (`config/phase-lock.yaml`). Phases RESEARCH/INNOVATE/PLAN/BUILD/VERIFY + custom CLASSIFY/KNOW/QUALITY/AUTOEVAL/MEMORY_OBSERVE. Adds `write_restriction.allowed_paths` (PLAN) and `exec_restriction.allowed_commands_patterns` (VERIFY/QUALITY test-only) — capabilities the native layers lack.

**Overlap verdict:** the *mechanism* overlaps (both produce a per-call allow/deny on tool name). Native gating is **owner/permission + mode (plan)** driven, deployment-static; phase-lock is **workflow-phase** driven, session-dynamic, path/command-pattern aware. Different axes. Overlap only on the coarse "block bash/write in a restricted context" — but native has no notion of RESEARCH/VERIFY phases and no per-path/per-command allowlists.

## 4. Catalog of native built-in tools (name → purpose), by category

**Shell / compute**
- `bash` — run shell commands (install, git, builds); `#!bg` marker → detached job.
- `python` — execute Python for compute/parsing.
- `manage_bg_jobs` — list/output/kill detached `#!bg` bash jobs.

**Filesystem (workspace-confined)**
- `read_file` — read file, optional offset/limit.
- `write_file` — create / full-rewrite file on disk.
- `edit_file` — exact-string replacement edit on disk (diff shown).
- `grep` — ripgrep content search (file:line:match).
- `glob` — find files by glob, newest first.
- `ls` — list directory entries.
- `get_workspace` — return active workspace absolute path.

**Web**
- `web_search` — single web lookup for a fact/current info.
- `web_fetch` — fetch text content of a named URL.

**Editor documents (panel)**
- `create_document`, `update_document` (full rewrite), `edit_document` (find/replace), `suggest_document` (review), `manage_documents` (list/read/delete/page).

**AI / model interaction**
- `chat_with_model`, `ask_teacher`, `pipeline` (multi-step), `list_models`, `generate_image`, `edit_image`.

**Sessions / chats**
- `create_session`, `list_sessions`, `send_to_session`, `manage_session`, `search_chats`.

**Knowledge / memory / notes**
- `manage_memory` (facts about USER), `manage_skills`, `manage_notes` (Keep-style + reminders), `manage_contact` (CardDAV), `resolve_contact`, `manage_calendar`.

**Deep research**
- `trigger_research` (start job), `manage_research` (list/read/delete saved).

**Email**
- `list_email_accounts`, `list_emails`, `read_email`, `send_email`, `reply_to_email`, `archive_email`, `delete_email`, `mark_email_read`, `bulk_email`.

**Tasks / scheduling**
- `manage_tasks` (cron tasks).

**Integrations / generic API**
- `api_call` (named integration: Home Assistant, Miniflux, Gitea…), `app_api` (loopback to allowed internal `/api/*`).

**Cookbook / model serving [admin]**
- `download_model`, `serve_model`, `serve_preset`, `stop_served_model`, `cancel_download`, `adopt_served_model`, `tail_serve_output`, `list_served_models`, `list_downloads`, `search_hf_models`, `list_cached_models`, `list_serve_presets`, `list_cookbook_servers`.

**Admin / config**
- `manage_endpoints`, `manage_mcp`, `manage_webhooks`, `manage_tokens`, `manage_settings` (also toggles tools on/off).

**Vault (secrets)**
- `vault_search`, `vault_get`, `vault_unlock`.

**UI-control / turn markers (no subprocess)**
- `ask_user` (multiple-choice, ENDS turn), `update_plan` (writes active plan), `ui_control` (panels, mode/model/theme, toggle tools).

**MCP-routed** (`_MCP_TOOL_MAP` `src/tool_execution.py:317`): `bash`, `python`, `read_file`, `write_file`, `web_search`, `web_fetch`, `generate_image` route through the MCP manager rather than direct impls.

**Built-in MCP servers** (`src/builtin_mcp.py:72` `register_builtin_servers` `:92`): Python — `image_gen`, `memory`, `rag`, `email`; NPX — `builtin_browser` (`@playwright/mcp`). Kill-switch `ODYSSEUS_DISABLE_MCP`.

## 5. Redundancy verdict for grafted tool modules

| Grafted module | Verdict | Rationale |
|---|---|---|
| `src/tool_registry.py` phase-lock (`is_tool_allowed`) | **PARTIAL** | Mechanism (name-based allow/deny) overlaps native `disabled_tools` + plan-mode. But phase axis (RESEARCH/INNOVATE/PLAN/VERIFY/…), per-session state, path-scoped writes, and test-command allowlists are **unique**. Do not re-implement name blocking; DO reuse native `disabled_tools`/`blocked_tools_for_owner` for owner concerns instead of duplicating them in YAML. |
| `src/orchestrator/gate.should_block_destructive` | **UNIQUE-COMPLEMENT** | Native `classify_tool`/risk classifier only **logged** destructive shell then executed; this graft is the first real *block* on catastrophic `bash`/`python`/`run_command`. Deliberately scoped to shell (not `delete_file` tools). No native equivalent enforces this. Keep. |
| Constitution trace (`risk_classifier` + `trace_writer`) | UNIQUE-COMPLEMENT | Observability/audit layer; no native trace of per-tool risk/outcome. Keep. |

**Net:** the grafted layer should NOT add its own generic tool-name denylist or owner/admin gating — those exist natively (§3) and run inside `_execute_tool_block_impl`. It should keep only the phase-workflow lock (path/command-scoped) and the destructive shell gate, which have no native counterpart.

## 6. Risks / notes

- **Fail-open phase-lock:** `src/tool_execution.py:~583` swallows all exceptions (`except Exception: pass`). If `config/phase-lock.yaml` is malformed or `ToolRegistry` errors, gating silently disables — opposite of native plan-mode which fails **closed**. Consider aligning fail-mode.
- **Two arg-parse paths:** phase-lock re-parses `block.content` as JSON (`src/tool_registry.py` args extraction in `_execute_tool_block_impl` region) separate from native `_parse_tool_args` (`src/tool_utils.py:44`). Non-JSON / plain-string tool payloads (e.g. `update_plan`, `ask_user` raw string) yield empty args → `write_restriction`/`exec_restriction` checks silently skipped.
- **Phase gate uses `block.tool_type`; destructive gate uses `_tool_name`** (same source) — consistent, but the phase check only inspects `command`/`cmd`/`path`/`file_path` keys; a tool using a different arg name for a path escapes `write_restriction`.
- **Singleton config load once:** `ToolRegistry.get_instance()` caches config at first use; editing `phase-lock.yaml` at runtime needs a process restart (no reload path).
- **Ordering:** grafted gate + phase-lock run in `execute_tool_block` (521) BEFORE native `disabled_tools`/owner/admin checks in `_execute_tool_block_impl` (550). A phase-lock ALLOW does not bypass native admin/owner denials — good (defense in depth), but means a tool can pass phase-lock and still be blocked natively (correct, just note for debugging "why blocked").
- **`config/phase-lock.yaml` default_phase = BUILD** = full access; a session with no `set_phase` call is unrestricted by the graft. The graft is opt-in per session.
