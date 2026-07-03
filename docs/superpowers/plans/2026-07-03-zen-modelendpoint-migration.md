# Zen → ModelEndpoint Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the native `ModelEndpoint` DB row the single source of truth for the OpenCode Zen provider, so the live chat path can enable Zen routing from a registered endpoint (not only from the `OPENCODE_API_KEY` env var), without deleting the UNIQUE routing policy that lives in `model-routing.json`.

**Architecture:** `zen_router` keeps its UNIQUE value — the complexity classifier (`heuristic`) + tier→model mapping (`models`/`stages`) + per-model blacklist. Slice 1 (already shipped) made `_zen_provider_conn` prefer a native `ModelEndpoint` row for the provider's `base_url`/`api_key`, falling back to `model-routing.json` + env. This plan adds: (2) a cached "is a Zen endpoint registered?" check + a **default-OFF** kill-switch that lets a registered endpoint enable Zen injection on the live path; (3) decouples per-candidate enablement from the JSON `enabled` flag when the endpoint is the source; (4) docs. The live chat path stays **byte-identical by default** (env-triggered path unchanged; new endpoint-triggered path is opt-in behind `ODYSSEUS_ZEN_FROM_ENDPOINT`).

**Tech Stack:** Python, SQLAlchemy (`core.database.ModelEndpoint`), pytest (`asyncio_mode=auto`, `def run(coro)` helper), existing kill-switch env-var pattern.

---

## Scope Correction (read before starting)

The intel map (`.planning/intel/INDEX.md`) previously listed "Supprimer `model-routing.json`" as a migration target. **That is wrong and must NOT be done.** `model-routing.json` holds two distinct kinds of content:

- **Redundant (migrate → DB):** `providers.opencode_zen.base_url` + `api_key_env` — duplicates a `ModelEndpoint` row.
- **UNIQUE routing policy (KEEP):** `models` (tier→model_id + fallback), `stages`, `heuristic` (strong/weak signals, word-count + score thresholds), `intent_categories`, `fallback_chains`. The native side has **no** complexity-based routing — deleting these deletes the routing intelligence that makes `zen_router` worth keeping.

**Also explicitly OUT of scope / rejected here:**
- Deleting `routes/routing_routes.py` (`/api/route*`). Its docstring says it is consumed by the dashboard + external agents. Removing it breaks external consumers — that is a separate, consumer-coordinated deprecation, not part of this migration.
- Deleting `model-routing.json`. Only the provider `base_url`/`api_key_env` become optional (Task 3); the file stays for routing policy.

## Current State (post slice 1)

- `src/zen_router.py`:
  - `_resolve_zen_endpoint_row()` → `(base_url, api_key)` or `None` (best-effort; matches a `ModelEndpoint` whose `base_url` contains `opencode.ai`, resolved via `resolve_endpoint_runtime`).
  - `_zen_provider_conn(zen_cfg)` → `(base_url, api_key)`; prefers the native row, else JSON `base_url` + `os.getenv(api_key_env)`.
  - `get_zen_candidate(tier)` and `build_zen_candidates(messages, stage)` both call `_zen_provider_conn`. Both still early-return when `zen_cfg.get("enabled", False)` is falsy.
- `src/llm_core.py` (~line 2453, inside `stream_llm_with_fallback`): injects Zen candidates **only** when `os.getenv("OPENCODE_API_KEY")` is truthy.
- Tests: `tests/test_zen_provider_conn.py` (6 tests) pin the two-argument `_zen_provider_conn` contract.

---

### Task 1: Cached "Zen endpoint registered?" check

**Why:** The live gate (Task 2) must not run a DB query on **every** stream call. Cache the answer with a short TTL so users who never register a Zen endpoint pay at most one query per TTL window.

**Files:**
- Modify: `src/zen_router.py` (add after `_resolve_zen_endpoint_row`)
- Test: `tests/test_zen_endpoint_gate.py` (new)

- [ ] **Step 1: Write the failing test**

```python
# tests/test_zen_endpoint_gate.py
"""Task 1 — cached endpoint-existence check for the live Zen gate."""
import src.zen_router as zr


def test_registered_true_when_row_present(monkeypatch):
    monkeypatch.setattr(zr, "_resolve_zen_endpoint_row", lambda: ("https://db/zen", "k"))
    zr._reset_zen_endpoint_cache()
    assert zr.zen_endpoint_registered() is True


def test_registered_false_when_no_row(monkeypatch):
    monkeypatch.setattr(zr, "_resolve_zen_endpoint_row", lambda: None)
    zr._reset_zen_endpoint_cache()
    assert zr.zen_endpoint_registered() is False


def test_result_is_cached_within_ttl(monkeypatch):
    calls = {"n": 0}

    def _row():
        calls["n"] += 1
        return ("https://db/zen", "k")

    monkeypatch.setattr(zr, "_resolve_zen_endpoint_row", _row)
    zr._reset_zen_endpoint_cache()
    assert zr.zen_endpoint_registered() is True
    assert zr.zen_endpoint_registered() is True
    assert calls["n"] == 1  # second call served from cache
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python -m pytest tests/test_zen_endpoint_gate.py -q`
Expected: FAIL — `AttributeError: module 'src.zen_router' has no attribute '_reset_zen_endpoint_cache'`

- [ ] **Step 3: Write minimal implementation**

Add to `src/zen_router.py` immediately after `_resolve_zen_endpoint_row`:

```python
_ZEN_ENDPOINT_CACHE = {"present": None, "ts": 0.0}
_ZEN_ENDPOINT_TTL = 30.0


def _reset_zen_endpoint_cache() -> None:
    _ZEN_ENDPOINT_CACHE["present"] = None
    _ZEN_ENDPOINT_CACHE["ts"] = 0.0


def zen_endpoint_registered() -> bool:
    """Cached: is a native opencode.ai ModelEndpoint registered? Used by the
    live gate so a stream call does not hit the DB every time. Best-effort —
    any resolution failure caches False."""
    now = time.time()
    cached = _ZEN_ENDPOINT_CACHE["present"]
    if cached is not None and (now - _ZEN_ENDPOINT_CACHE["ts"]) < _ZEN_ENDPOINT_TTL:
        return cached
    present = _resolve_zen_endpoint_row() is not None
    _ZEN_ENDPOINT_CACHE["present"] = present
    _ZEN_ENDPOINT_CACHE["ts"] = now
    return present
```

- [ ] **Step 4: Run test to verify it passes**

Run: `python -m pytest tests/test_zen_endpoint_gate.py -q`
Expected: PASS (3 passed)

- [ ] **Step 5: Commit**

```bash
rtk git add src/zen_router.py tests/test_zen_endpoint_gate.py
rtk git commit -m "feat(zen): cached zen_endpoint_registered() check for live gate"
```

---

### Task 2: Enablement source flag + kill-switched live gate

**Why:** Slice 1 made the *credentials* come from the DB row, but Zen still only turns on when `OPENCODE_API_KEY` is set. To let a registered endpoint enable Zen (the point of the migration) we must (a) know whether the conn came from the endpoint or JSON, so per-candidate enablement can ignore the JSON `enabled=false` flag for endpoint-sourced conns, and (b) add a **default-OFF** trigger on the live path. Default OFF ⇒ the live chat path is byte-identical for everyone today.

**Files:**
- Modify: `src/zen_router.py` (`_zen_provider_conn`, `get_zen_candidate`, `build_zen_candidates`, add `zen_injection_enabled`)
- Modify: `src/llm_core.py` (the inject block ~line 2453)
- Modify: `tests/test_zen_provider_conn.py` (update to 3-tuple contract)
- Test: `tests/test_zen_endpoint_gate.py` (extend)

- [ ] **Step 1: Write the failing tests**

Append to `tests/test_zen_endpoint_gate.py`:

```python
def test_injection_enabled_by_env_key(monkeypatch):
    monkeypatch.setenv("OPENCODE_API_KEY", "env-key")
    monkeypatch.delenv("ODYSSEUS_ZEN_FROM_ENDPOINT", raising=False)
    assert zr.zen_injection_enabled() is True


def test_injection_disabled_by_default_without_env(monkeypatch):
    monkeypatch.delenv("OPENCODE_API_KEY", raising=False)
    monkeypatch.delenv("ODYSSEUS_ZEN_FROM_ENDPOINT", raising=False)
    monkeypatch.setattr(zr, "zen_endpoint_registered", lambda: True)
    # endpoint registered but kill-switch OFF → still disabled (byte-identical)
    assert zr.zen_injection_enabled() is False


def test_injection_enabled_by_endpoint_when_killswitch_on(monkeypatch):
    monkeypatch.delenv("OPENCODE_API_KEY", raising=False)
    monkeypatch.setenv("ODYSSEUS_ZEN_FROM_ENDPOINT", "1")
    monkeypatch.setattr(zr, "zen_endpoint_registered", lambda: True)
    assert zr.zen_injection_enabled() is True


def test_endpoint_conn_bypasses_json_enabled_false(monkeypatch):
    cfg = {
        "providers": {"opencode_zen": {"enabled": False, "base_url": "https://json/zen", "api_key_env": "OPENCODE_API_KEY"}},
        "stages": {"chat": "standard"},
        "models": {"standard": {"model_id": "m-standard", "fallback": []}},
    }
    monkeypatch.setattr(zr, "_load_routing_config", lambda: cfg)
    monkeypatch.setattr(zr, "_resolve_zen_endpoint_row", lambda: ("https://db/zen", "db-key"))
    monkeypatch.setattr(zr, "classify_complexity", lambda p: ("standard", 0.5))
    cands = zr.build_zen_candidates([{"role": "user", "content": "hi"}], stage="chat")
    assert cands and cands[0][0] == "https://db/zen"  # enabled=false ignored: endpoint is the source
```

Update the two-tuple unpacks in `tests/test_zen_provider_conn.py` to the new 3-tuple `(url, key, from_endpoint)`:

```python
def test_conn_falls_back_to_json_and_env(monkeypatch):
    monkeypatch.setattr(zr, "_resolve_zen_endpoint_row", lambda: None)
    monkeypatch.setenv("OPENCODE_API_KEY", "env-key")
    url, key, from_endpoint = zr._zen_provider_conn(
        {"base_url": "https://json/zen", "api_key_env": "OPENCODE_API_KEY"}
    )
    assert url == "https://json/zen"
    assert key == "env-key"
    assert from_endpoint is False


def test_conn_prefers_native_endpoint_row(monkeypatch):
    monkeypatch.setattr(zr, "_resolve_zen_endpoint_row", lambda: ("https://db/zen", "db-key"))
    monkeypatch.setenv("OPENCODE_API_KEY", "env-key")
    url, key, from_endpoint = zr._zen_provider_conn(
        {"base_url": "https://json/zen", "api_key_env": "OPENCODE_API_KEY"}
    )
    assert url == "https://db/zen"
    assert key == "db-key"
    assert from_endpoint is True
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `python -m pytest tests/test_zen_endpoint_gate.py tests/test_zen_provider_conn.py -q`
Expected: FAIL — `zen_injection_enabled` missing; `_zen_provider_conn` returns a 2-tuple (ValueError: not enough values to unpack).

- [ ] **Step 3: Write minimal implementation**

In `src/zen_router.py`, change `_zen_provider_conn` to return a 3-tuple:

```python
def _zen_provider_conn(zen_cfg: dict):
    """(base_url, api_key, from_endpoint). Prefers a native ModelEndpoint row
    (single source of truth); falls back to model-routing.json + env. The
    third element tells callers whether the conn came from a registered
    endpoint, so per-candidate enablement can bypass the JSON `enabled` flag."""
    row = _resolve_zen_endpoint_row()
    if row:
        return row[0], row[1], True
    base_url = zen_cfg.get("base_url", "https://opencode.ai/zen/v1/chat/completions")
    api_key = os.getenv(zen_cfg.get("api_key_env", "OPENCODE_API_KEY"), "")
    return base_url, api_key, False


def zen_injection_enabled() -> bool:
    """Should the live path inject Zen candidates? Env key = today's behavior
    (unchanged). A registered endpoint only enables Zen when the default-OFF
    kill-switch ODYSSEUS_ZEN_FROM_ENDPOINT=1 is set — so the live chat path is
    byte-identical by default."""
    if os.getenv("OPENCODE_API_KEY"):
        return True
    if os.getenv("ODYSSEUS_ZEN_FROM_ENDPOINT") == "1" and zen_endpoint_registered():
        return True
    return False
```

Update `get_zen_candidate` — replace the enablement + conn section:

```python
    zen_cfg = providers.get("opencode_zen", {})
    zen_url, api_key, from_endpoint = _zen_provider_conn(zen_cfg)
    if not api_key:
        return None
    if not from_endpoint and not zen_cfg.get("enabled", False):
        return None
```

(delete the earlier standalone `if not zen_cfg.get("enabled", False): return None` guard so this single block governs enablement).

Update `build_zen_candidates` similarly — replace its enablement + conn section:

```python
    zen_cfg = providers.get("opencode_zen", {})
    zen_url, api_key, from_endpoint = _zen_provider_conn(zen_cfg)
    if not api_key:
        return []
    if not from_endpoint and not zen_cfg.get("enabled", False):
        return []
```

(delete its earlier `if not zen_cfg.get("enabled", False): return []` guard.)

In `src/llm_core.py`, replace the inject block:

```python
    zen_cands: list = []
    try:
        from src.zen_router import zen_injection_enabled, build_zen_candidates
        if zen_injection_enabled():
            stage = kwargs.pop("_agentos_stage", "chat")
            zen_cands = build_zen_candidates(messages, stage=stage)
    except Exception as _ze:
        logger.debug(f"[ZenRouter] inject skipped: {_ze}")
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `python -m pytest tests/test_zen_endpoint_gate.py tests/test_zen_provider_conn.py -q`
Expected: PASS (all)

- [ ] **Step 5: Regression — routing + endpoint suites**

Run: `python -m pytest tests/test_orchestrator_router_advice.py tests/test_resolve_endpoint_fallbacks.py tests/test_endpoint_resolver.py -q`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
rtk git add src/zen_router.py src/llm_core.py tests/test_zen_provider_conn.py tests/test_zen_endpoint_gate.py
rtk git commit -m "feat(zen): kill-switched endpoint-triggered injection (default OFF)"
```

---

### Task 3: Make the JSON provider block optional (dedup the redundant config)

**Why:** With Task 2, an endpoint-sourced conn no longer needs `providers.opencode_zen` in `model-routing.json` at all — the DB row supplies url + key + enablement. Prove the file's provider block is now optional so it can later be trimmed to routing policy only. This is the actual redundancy removal; keep it a *test that pins behavior*, not a file deletion.

**Files:**
- Test: `tests/test_zen_endpoint_gate.py` (extend)

- [ ] **Step 1: Write the failing test**

```python
def test_candidates_work_with_no_provider_block(monkeypatch):
    # model-routing.json trimmed to routing policy ONLY (no providers block)
    cfg = {
        "stages": {"chat": "standard"},
        "models": {"standard": {"model_id": "m-standard", "fallback": []}},
    }
    monkeypatch.setattr(zr, "_load_routing_config", lambda: cfg)
    monkeypatch.setattr(zr, "_resolve_zen_endpoint_row", lambda: ("https://db/zen", "db-key"))
    monkeypatch.setattr(zr, "classify_complexity", lambda p: ("standard", 0.5))
    cands = zr.build_zen_candidates([{"role": "user", "content": "hi"}], stage="chat")
    assert cands and cands[0][0] == "https://db/zen" and cands[0][1] == "m-standard"
```

- [ ] **Step 2: Run test to verify it fails or passes**

Run: `python -m pytest tests/test_zen_endpoint_gate.py::test_candidates_work_with_no_provider_block -q`
Expected: If it FAILS with `KeyError`/`AttributeError`, `build_zen_candidates` still assumes a `providers` block — fix in Step 3. If it PASSES, Task 2's code already tolerates a missing block (the `providers.get("opencode_zen", {})` default handles it) — proceed to Step 4.

- [ ] **Step 3: Fix only if Step 2 failed**

Ensure `build_zen_candidates` reads `providers = cfg.get("providers", {})` and `zen_cfg = providers.get("opencode_zen", {})` (both already `.get` with defaults). No further change expected; the endpoint conn bypasses `enabled`.

- [ ] **Step 4: Commit**

```bash
rtk git add tests/test_zen_endpoint_gate.py
rtk git commit -m "test(zen): provider block in model-routing.json is now optional"
```

---

### Task 4: Docs + planning update

**Files:**
- Modify: `.env.example` (document `ODYSSEUS_ZEN_FROM_ENDPOINT`)
- Modify: `.planning/intel/INDEX.md`, `.planning/STATE.md`

- [ ] **Step 1: Document the kill-switch in `.env.example`**

Add near the other `ODYSSEUS_*` flags:

```bash
# Enable Zen routing from a registered native ModelEndpoint (host opencode.ai)
# instead of only from OPENCODE_API_KEY. Default OFF → chat path unchanged.
ODYSSEUS_ZEN_FROM_ENDPOINT=0
```

- [ ] **Step 2: Update intel INDEX + STATE**

In `.planning/intel/INDEX.md`: correct the `model-routing.json` row — it is **PARTIEL**, not REDONDANT: only the `providers.*` blocks are redundant; `models`/`stages`/`heuristic`/`fallback_chains` are UNIQUE routing policy to keep. Mark the Zen migration slice 2/3 done.

In `.planning/STATE.md`: add a Recent Decisions entry (dated 2026-07-03) summarizing the kill-switched endpoint-triggered injection + the model-routing.json scope correction.

- [ ] **Step 3: Commit**

```bash
rtk git add .env.example .planning/intel/INDEX.md .planning/STATE.md
rtk git commit -m "docs(zen): document ODYSSEUS_ZEN_FROM_ENDPOINT + correct model-routing.json verdict"
```

---

## Self-Review

- **Spec coverage:** Task 1 (cached check) → gate cost; Task 2 (flag + gate + 3-tuple conn) → endpoint-triggered enablement, default-OFF; Task 3 → provider-block optional (dedup proven); Task 4 → docs + intel correction. The scope correction (keep routing policy, don't delete `model-routing.json`/`routing_routes.py`) is stated up front and enforced by Task 3 keeping the file.
- **Type consistency:** `_zen_provider_conn` returns `(url, key, from_endpoint)` in Task 2 and every caller (`get_zen_candidate`, `build_zen_candidates`) plus the updated `tests/test_zen_provider_conn.py` unpack three values. `zen_endpoint_registered()`/`zen_injection_enabled()` names are used identically in `zen_router.py`, `llm_core.py`, and tests.
- **Placeholder scan:** none — every code step shows full code and every run step shows the exact command + expected result.
- **Risk:** the only live-path edit is the `llm_core.py` inject block; with `ODYSSEUS_ZEN_FROM_ENDPOINT` unset and behavior keyed on `OPENCODE_API_KEY` exactly as before, the env-triggered path is byte-identical. Verified by `test_injection_enabled_by_env_key` + `test_injection_disabled_by_default_without_env`.
