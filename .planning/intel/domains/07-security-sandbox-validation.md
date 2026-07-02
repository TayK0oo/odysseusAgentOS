# 07 — Security / Sandbox / Validation / Risk (native map)

> Scope: what Odysseus does NATIVELY, so the grafted intelligence layer (destructive
> gate + command_validator + risk_classifier + docker hardening) only adds what's
> missing. All paths verified via CBM + Read. Cite `fichier:ligne`.
> Grafted files = `src/orchestrator/gate.py`, `src/risk_classifier.py`, `src/command_validator.py`.

---

## 1. Native security layers (files : funcs)

### 1a. Prompt-injection hardening — `src/prompt_security.py`
- `untrusted_context_message(label, content)` (`prompt_security.py:60`) wraps retrieved
  docs / web / email / memory / skill text in a hard-coded `UNTRUSTED_CONTEXT_HEADER`
  + guard markers `<<<UNTRUSTED_SOURCE_DATA>>>` (`:26-27`), role forced to `user`,
  `metadata.trusted=false`.
- `_escape_guard_markers` (`:30`) + `_sanitize_label` (`:44`) neutralise marker-breakout
  and CR/LF injection. Data-not-instructions policy string at `:8`.
- **Native, complete.** Grafted layer must NOT duplicate.

### 1b. Outbound URL safety (SSRF) — two modules, both native
- `src/url_safety.py` `check_outbound_url(url, block_private=False, resolver)` (`:48`):
  scheme allowlist http/https (`:26`), always blocks link-local `169.254/16` (cloud
  metadata), multicast/reserved/unspecified (`_classify` `:34`). Opt-in full private
  lockdown via `EMBEDDING_BLOCK_PRIVATE_IPS`. Used for the user-set embedding endpoint.
- `src/url_security.py` `validate_public_http_url()` (`:81`) / `is_public_http_url()`
  (`:74`): stricter — blocks ALL private/loopback/link-local + internal hostname suffixes
  (`_BLOCKED_NETWORKS` `:24`, `_INTERNAL_SUFFIXES` `:16`), fail-closed on DNS. For
  untrusted API-token-supplied outbound URLs.
- **Native, complete** (two tiers by trust level).

### 1c. Tool security & access gating — `src/tool_security.py`
- `NON_ADMIN_BLOCKED_TOOLS` (`:14`) — public/non-admin users cannot call `bash`,
  `python`, `manage_bg_jobs`, file tools, `api_call`, email, model-serve, vault, etc.
- `is_public_blocked_tool()` (`:154`, fail-CLOSED for non-str names / `mcp__*`).
- `owner_is_admin_or_single_user()` (`:169`) — admin OR `AUTH_ENABLED=false`; pre-setup
  window treated as NON-admin (defense-in-depth).
- Plan mode: `PLAN_MODE_READONLY_TOOLS` allowlist (`:66`) + `plan_mode_disabled_tools()`
  inverse-denylist (`:123`), fail-closed backstop `_PLAN_MODE_KNOWN_MUTATORS` (`:104`).
  **bash/python explicitly excluded from plan mode** (`:117`).
- **Native RBAC/mode gating — orthogonal to destructive-content gating.** No overlap.

### 1d. Secret handling
- `src/secret_storage.py` — Fernet symmetric encrypt (`enc:` prefix, idempotent) of
  IMAP/SMTP passwords in SQLite; key at `data/.app_key` mode 0o600 (`_load_or_create_key`
  `:37`, `safe_chmod` `:45`). Threat model: DB exfil, NOT process compromise.
- `src/settings_scrub.py` — `is_secret_key()` (`:39`) + deep `_scrub_value()` (`:48`)
  mask secret-shaped keys before `/api/auth/settings` (auth-exempt) returns to non-admin.
- `risk_classifier.args_summary()` (`:142`) redacts secret keys in trace summaries.
- **Native, complete.**

### 1e. TLS — `src/tls_overrides.py`
- `llm_verify()` — layers operator PEM (`LLM_CA_BUNDLE`) on top of system trust for
  private-CA LLM providers. **No verify=off knob by design.** Scope pinned to 2 call
  sites (`llm_core.py`, `model_routes.py`) by `tests/test_tls_overrides_scope.py`. Native.

---

## 2. Shell execution paths — INVENTORY (the M3.4 core)

Two distinct execution surfaces exist:
- **A. Agent tool loop** → `execute_tool_block()` (`tool_execution.py:521`) which runs the
  Constitution risk-classify + destructive-gate block (`:539-583`) BEFORE
  `_execute_tool_block_impl`.
- **B. Task-scheduler actions** → `TaskScheduler._execute_action` (`task_scheduler.py:1107`)
  calls `BUILTIN_ACTIONS[name]` **directly** — bypasses `execute_tool_block` entirely.
- **C. HTTP shell API** → `routes/shell_routes.py` FastAPI endpoints, admin-gated only.

| # | Path (entry → executor) | fichier:ligne | Validated by command_validator? | Covered by destructive gate? |
|---|---|---|---|---|
| 1 | `bash` tool → `BashTool.execute` `create_subprocess_shell` | `agent_tools/subprocess_tools.py:103,108` | **NO** | **YES** — gate in `tool_execution.py:553-575`, `classify_bash` on raw content |
| 2 | `python` tool → `PythonTool.execute` `create_subprocess_exec -I -c` | `subprocess_tools.py:129,134` | **NO** | **YES** — same gate (`_SHELL_TOOLS` incl. `python`) |
| 3 | `bash` `#!bg` background → `bg_jobs.launch` (detached Popen) | `tool_execution.py:822-826` → `bg_jobs.py:133` | **NO** | **YES (indirect)** — gate runs on the full raw content incl. `#!bg` marker before split, `classify_bash` still matches |
| 4 | **`run_script` action** → `_run_subprocess(shell=True)` | `builtin_actions.py:340,348` | **NO** | **NO — BYPASS** (not in `_SHELL_TOOLS`; scheduler calls action directly) |
| 5 | **`run_local` action** → `_run_subprocess(shell=True)` | `builtin_actions.py:352,358` | **NO** | **NO — BYPASS** |
| 6 | `ssh_command` action → `_run_subprocess` (local `bash -c` or ssh) | `builtin_actions.py:313` | **YES** — `validate_and_log` `:319-322` | **NO** (not in `_SHELL_TOOLS`) — only native validator covers it |
| 7 | **`POST /api/shell/exec`** → `_exec_shell` `create_subprocess_shell` | `shell_routes.py:820,434,438` | **NO** | **NO — BYPASS** (admin auth only) |
| 8 | **`POST /api/shell/stream`** → `_create_shell` (pty/tmux/pipe) | `shell_routes.py:834,412` | **NO** | **NO — BYPASS** (admin auth only) |
| 9 | `manage_bg_jobs` (list/output/kill) | `agent_tools/bg_job_tools.py`, `tool_execution.py:858` | n/a | n/a — does NOT launch new commands |
| 10 | Cookbook install/rebuild subprocs (`create_subprocess_exec`, arg-list) | `shell_routes.py:1214,1541,1619,1676` | NO | NO — but arg-list exec (no shell), fixed argv, admin-gated |

**Note on `run_command`:** referenced in `gate._SHELL_TOOLS` (`gate.py:19`) and
`risk_classifier` (`:63,126`) but **no tool named `run_command` exists** in the codebase.
Phantom name — the real agent shell tools are `bash` / `python`.

---

## 3. Native sandbox / docker hardening — `docker-compose.yml`

- odysseus service: `security_opt: no-new-privileges:true` (`:87`), `tmpfs /tmp size=100m`
  (`:90`), runs as PUID/PGID 1000 non-root (`:82`), ports bound to `127.0.0.1` by default (`:5`).
- **`cap_drop: [ALL]` is COMMENTED OUT** (`:88`) — "activé quand compatibilité vérifiée".
  So the odysseus container keeps the full default cap set.
- `read_only: false` (`:89`, needed for `/app/data` writes).
- **Docker socket mounted `/var/run/docker.sock`** (`:26`) + `group_add DOCKER_GID` (`:29`)
  — Cookbook feature; a container-root or shell-exec compromise = **host Docker daemon =
  effective host root**. This is the single largest sandbox weakness.
- Only `searxng` service has real `cap_drop: ALL` + minimal `cap_add` (`:144-150`).
- No seccomp/apparmor profile specified; no AppArmor/seccomp on odysseus.
- SSRF-adjacent: `extra_hosts host.docker.internal:host-gateway` (`:33`) reaches host services.
- No native per-command sandbox (namespaces/firejail); shell tools run in the app container
  with its full FS + docker.sock reach.

---

## 4. Grafted gate/command_validator vs native — OVERLAP VERDICT + remaining bypasses

### Overlap
- **`command_validator.py` vs `risk_classifier.classify_bash`:** two SEPARATE destructive
  pattern lists that partially overlap (rm -rf, mkfs, dd, drop database, git push --force,
  fork bomb) but diverge:
  - `command_validator` uniquely BLOCKS: `curl … | bash`, `wget -O- | sh`, `echo <secret>`,
    `kill -9 1`, `killall -9`, DROP/TRUNCATE with test-table exceptions; has a WARNING tier
    (`sudo`, `eval`, `exec`, chmod 777) and a SAFE_PREFIXES whitelist (`command_validator.py:20-63`).
  - `risk_classifier.DESTRUCTIVE_PATTERNS` uniquely covers: `DELETE FROM`, `shred`, `wipefs`,
    `> /dev/sd*`, `chmod -R 777`, `format c:` (`risk_classifier.py:85-105`).
  - **Neither is a superset.** They are used in different places and are NOT wired together.
- The gate (`gate.should_block_destructive` `:36`) correctly covers agent `bash`/`python`
  (incl. `#!bg`). `command_validator` only actually runs inside `action_ssh_command`.
- Native tool_security RBAC/plan-mode is orthogonal — no duplication risk.

### CONCRETE remaining bypasses (M3.4 gap list — order by severity)
1. **`POST /api/shell/exec` + `/api/shell/stream`** (`shell_routes.py:820,834`) — arbitrary
   shell, admin-auth only, ZERO content validation and ZERO destructive gate. Highest-value
   bypass: a compromised/misused admin session or CSRF-ish path runs `rm -rf /`, and with
   docker.sock mounted → host takeover.
2. **`run_script` action** (`builtin_actions.py:340`) — scheduled tasks run arbitrary shell
   via `subprocess.run(shell=True)`, no validator, no gate. A task the agent creates (or an
   imported task) executes unchecked.
3. **`run_local` action** (`builtin_actions.py:352`) — same as run_script, no host arg.
4. **`ssh_command` remote branch** — `validate_and_log` runs, but the remote `ssh host cmd`
   path (`:335`) still ships the command to a remote box; validator patterns are host-agnostic
   so it's checked, but WARNING-tier items (sudo/eval) are allowed through.
5. **`command_validator` never guards the agent `bash`/`python` tools** — only the gate does,
   so the richer validator ruleset (curl|bash, secret-echo, kill -9 1) is NOT enforced on the
   primary agent shell surface. Inverse gap: the gate's DELETE FROM / shred / wipefs patterns
   are NOT enforced on `ssh_command`.
6. **`cap_drop: ALL` disabled** + **docker.sock mounted** — even a gated command that slips a
   novel destructive pattern has full container caps and host-daemon reach.

### What the grafted layer should add (not duplicate)
- Route **`run_script` / `run_local` / all `/api/shell/*` endpoints** through a single shared
  validator+gate (merge the two pattern lists into one source of truth).
- Add `run_script`/`run_local` to `gate._SHELL_TOOLS` OR gate at the action layer
  (`task_scheduler._execute_action` / `_run_subprocess`).
- Merge `command_validator` WARNING/BLOCK patterns into `risk_classifier` (or vice-versa) so
  one canonical destructive set covers every surface.
- Enable `cap_drop: [ALL]` + minimal cap_add; drop or proxy docker.sock; add seccomp profile.

---

## 5. Risks / notes
- The gate is kill-switchable via `ODYSSEUS_DESTRUCTIVE_GATE=off` (`gate.py:22`) — a false-y
  env fully disables destructive blocking for agent bash/python. Document as an operational risk.
- Gate + classifier are regex-only → trivially bypassable by obfuscation (`r''m -rf`, base64,
  `$IFS`, variable indirection, `bash -c "$(...)"`). Not a real sandbox; treat as best-effort.
- Native path-confinement exists: `agent_cwd()` / `_active_workspace` bind subprocess cwd to the
  per-turn workspace (`tool_execution.py:532-534,611`) — but shell can still `cd` / use abs paths.
- `command_validator` SAFE_PREFIXES uses `startswith` → `ls; rm -rf /` is whitelisted by the
  `ls ` prefix and never reaches the block check (`command_validator.py:76-78`). Real bug.
- `ssh_command` on Windows falls back to `shell=True` with the raw string (`builtin_actions.py:333`).
- `python` tool uses `-I` (isolated) but still full network + FS in-container.
