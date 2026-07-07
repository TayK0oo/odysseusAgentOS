# Live Indicators & Run Cockpit — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surface backend runtime signals (phase, drift, iteration budget, autoeval, verifier) as live UI indicators — a persistent cockpit chip strip plus inline chat badges — matching the M4 agent-indicator style.

**Architecture:** A small pure module (`src/sse_indicators.py`) builds JSON-serializable SSE event dicts. `stream_agent_loop` emits them with one-line `yield`s at existing hook points. The frontend forwards a consolidated `run_status` event to a new `cockpit.js` module (persistent chips) and renders discrete `autoeval_result`/`verifier_result` events as inline badges reusing `.agent-indicator` styling. Honesty rule: fields are `null` → muted "—" chips when a module is gated OFF; no fabricated data.

**Tech Stack:** Python 3 / FastAPI (SSE via generator `yield`), vanilla ES-module JS, CSS with existing `--accent`/`--green` vars. Tests: pytest.

**Reference spec:** `docs/superpowers/specs/2026-07-07-live-indicators-cockpit-design.md`

---

## File Structure

| File | Responsibility |
|---|---|
| `src/sse_indicators.py` (**new**) | Pure functions building SSE event dicts (`run_status`, `autoeval_result`, `verifier_result`) + drift normalization. No I/O. |
| `tests/test_sse_indicators.py` (**new**) | Unit tests for the pure helpers (shape, null handling, JSON-serializable). |
| `src/agent_loop.py` (modify) | Call helpers and `yield` events at round start (`~2576`) and post-loop hooks (`~3658`, `~3691`). |
| `static/js/cockpit.js` (**new**) | Owns `#run-cockpit` DOM; consumes `run_status` payloads + polls `/api/health`. |
| `static/js/chat.js` (modify) | Whitelist `run_status`; forward it to cockpit; +2 inline-badge branches. |
| `static/index.html` (modify) | `#run-cockpit` container + `cockpit.js` script tag. |
| `static/css/style.min.css` (modify) | Cockpit strip + badge-state styles. |

---

## Task 1: Pure SSE-event helper module

**Files:**
- Create: `src/sse_indicators.py`
- Test: `tests/test_sse_indicators.py`

- [ ] **Step 1: Write the failing tests**

```python
# tests/test_sse_indicators.py
import json
from src.sse_indicators import (
    normalize_drift,
    run_status_event,
    autoeval_event,
    verifier_event,
)


def test_normalize_drift_from_enum_like():
    class _D:
        value = "HIGH"
    assert normalize_drift(_D()) == "high"


def test_normalize_drift_from_string():
    assert normalize_drift("Med") == "med"


def test_normalize_drift_none_and_unknown():
    assert normalize_drift(None) is None
    assert normalize_drift("garbage") is None


def test_run_status_event_shape():
    evt = run_status_event(phase="build", drift=None, used=3, max_rounds=12,
                           budget_pct=42, budget_tokens=18450)
    assert evt == {
        "type": "run_status",
        "phase": "build",
        "drift": None,
        "iters": {"used": 3, "max": 12},
        "budget": {"pct": 42, "tokens": 18450},
    }
    # must be JSON-serializable
    json.dumps(evt)


def test_run_status_event_nulls_when_unknown():
    evt = run_status_event(phase=None, drift="low", used=1, max_rounds=8,
                           budget_pct=None, budget_tokens=None)
    assert evt["phase"] is None
    assert evt["budget"] is None
    assert evt["drift"] == "low"


def test_autoeval_event_shape():
    evt = autoeval_event(decision="revert", reason="drift HIGH")
    assert evt == {"type": "autoeval_result", "decision": "revert", "reason": "drift HIGH"}
    json.dumps(evt)


def test_verifier_event_pass_and_fail():
    assert verifier_event(reasons=None) == {
        "type": "verifier_result", "status": "pass", "detail": ""}
    evt = verifier_event(reasons=["missing test", "lint fail", "x", "y"])
    assert evt["status"] == "fail"
    # detail joins at most 3 reasons
    assert evt["detail"] == "missing test; lint fail; x"
    json.dumps(evt)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `rtk python -m pytest tests/test_sse_indicators.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.sse_indicators'`

- [ ] **Step 3: Write the implementation**

```python
# src/sse_indicators.py
"""Pure builders for live-indicator SSE events.

No I/O, no side effects — each function returns a JSON-serializable dict
shaped for the frontend cockpit/badge handlers. A field is None when its
source module is inactive/unknown; the frontend renders None as a muted
"—" chip (never fabricates a value).
"""
from __future__ import annotations

from typing import Any, Optional

_DRIFT_LEVELS = {"low", "med", "high"}


def normalize_drift(drift: Any) -> Optional[str]:
    """Coerce a DriftLevel enum / string into 'low'|'med'|'high' or None."""
    if drift is None:
        return None
    raw = getattr(drift, "value", drift)
    text = str(raw).strip().lower()
    return text if text in _DRIFT_LEVELS else None


def run_status_event(
    *,
    phase: Optional[str],
    drift: Any,
    used: int,
    max_rounds: int,
    budget_pct: Optional[int],
    budget_tokens: Optional[int],
) -> dict:
    """Consolidated per-round status for the persistent cockpit strip."""
    budget = None
    if budget_pct is not None or budget_tokens is not None:
        budget = {"pct": budget_pct, "tokens": budget_tokens}
    return {
        "type": "run_status",
        "phase": phase,
        "drift": normalize_drift(drift),
        "iters": {"used": used, "max": max_rounds},
        "budget": budget,
    }


def autoeval_event(*, decision: str, reason: str = "") -> dict:
    """Discrete autoeval keep/revert decision → inline badge."""
    return {"type": "autoeval_result", "decision": decision, "reason": reason}


def verifier_event(*, reasons: Any) -> dict:
    """Discrete verifier verdict → inline badge. Empty reasons == pass."""
    if not reasons:
        return {"type": "verifier_result", "status": "pass", "detail": ""}
    if isinstance(reasons, (list, tuple)):
        detail = "; ".join(str(r) for r in reasons[:3])
    else:
        detail = str(reasons)
    return {"type": "verifier_result", "status": "fail", "detail": detail}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `rtk python -m pytest tests/test_sse_indicators.py -v`
Expected: PASS (7 passed)

- [ ] **Step 5: Commit**

```bash
rtk git add src/sse_indicators.py tests/test_sse_indicators.py
rtk git commit -m "feat(m4): pure SSE-event builders for live indicators"
```

---

## Task 2: Emit the events from `stream_agent_loop`

**Files:**
- Modify: `src/agent_loop.py` (round-start `~2571-2576`; post-loop `~3654-3691`)
- Test: `tests/test_sse_indicators.py` (add integration-shape guard)

Context: `run_status` fires once per round at round start (phase + iters + budget known there; drift only known post-loop, so early rounds carry `drift=None`). One final `run_status` fires after drift is computed. `autoeval_result` and `verifier_result` fire in the post-loop hook block. All emissions are wrapped so they never break the loop (match existing `try/except pass` style).

- [ ] **Step 1: Add the import near the top of `src/agent_loop.py`**

Find the existing block of `from src....` imports at the top of the file and add:

```python
from src.sse_indicators import (
    run_status_event,
    autoeval_event,
    verifier_event,
)
```

- [ ] **Step 2: Track last-known drift + budget helper (module-local, before the round loop)**

Immediately before the `for round_num in range(1, max_rounds + 1):` line (currently `agent_loop.py:2528`), add:

```python
    _last_drift = None  # updated post-loop; carried into per-round run_status

    def _budget_snapshot():
        """Best-effort (pct, tokens) from the enforcer; (None, None) if unknown."""
        if not _budget_enforcer:
            return None, None
        try:
            rep = _budget_enforcer.get_usage_report()
            return rep.get("pct"), rep.get("tokens")
        except Exception:
            return None, None
```

- [ ] **Step 3: Emit per-round `run_status` after the budget check**

Immediately AFTER the budget-check block that ends at `agent_loop.py:2576` (the `break` line inside `if not _budget_check["ok"]`), and before `round_response = ""` (2578), add:

```python
        # Live cockpit: consolidated per-round status (never breaks the loop)
        try:
            _bp, _bt = _budget_snapshot()
            _phase_val = _current_phase.value if _current_phase is not None else None
            yield f'data: {json.dumps(run_status_event(phase=_phase_val, drift=_last_drift, used=round_num, max_rounds=max_rounds, budget_pct=_bp, budget_tokens=_bt))}\n\n'
        except Exception:
            pass
```

- [ ] **Step 4: After drift is computed, record it and emit a final `run_status`**

In the OBSERVER block, immediately after `drift = _observer.compute_drift_score()` (currently `agent_loop.py:3654`) and its `if drift == DriftLevel.HIGH` log, extend the block so drift is captured and re-emitted. Replace the `except Exception: drift = None` tail (3657-3658) with:

```python
        _last_drift = drift
    except Exception:
        drift = None
        _last_drift = None
    # Live cockpit: final status carrying the computed drift
    try:
        _bp, _bt = _budget_snapshot()
        yield f'data: {json.dumps(run_status_event(phase=None, drift=_last_drift, used=max_rounds, max_rounds=max_rounds, budget_pct=_bp, budget_tokens=_bt))}\n\n'
    except Exception:
        pass
```

- [ ] **Step 5: Emit `verifier_result` and `autoeval_result` in the autoeval hook**

In the AUTOEVAL block, after the `_ae = apply_autoeval(...)` call and its `if _ae.enabled and _ae.decision == "revert":` log (currently ends `agent_loop.py:3691`), add inside the same `try`:

```python
        # Live badge: verifier verdict (derived from the reasons the verifier left)
        try:
            yield f'data: {json.dumps(verifier_event(reasons=locals().get("_verifier_last_reasons")))}\n\n'
        except Exception:
            pass
        # Live badge: autoeval decision (only meaningful when the module ran)
        if getattr(_ae, "enabled", False):
            try:
                _reason = _ae.error or (f"drift {getattr(_last_drift, 'value', _last_drift)}" if _ae.decision == "revert" else "checks passed")
                yield f'data: {json.dumps(autoeval_event(decision=_ae.decision, reason=str(_reason)))}\n\n'
            except Exception:
                pass
```

- [ ] **Step 6: Add a JSON-serializable guard test**

Append to `tests/test_sse_indicators.py`:

```python
def test_all_events_are_sse_line_safe():
    # SSE frames are newline-delimited; payloads must not contain raw newlines
    for evt in (
        run_status_event(phase="build", drift="high", used=2, max_rounds=5,
                         budget_pct=90, budget_tokens=1000),
        autoeval_event(decision="keep", reason="ok"),
        verifier_event(reasons=["boom"]),
    ):
        line = json.dumps(evt)
        assert "\n" not in line
```

- [ ] **Step 7: Run tests + full agent-loop suite for regressions**

Run: `rtk python -m pytest tests/test_sse_indicators.py tests/test_agent_loop.py -v`
Expected: PASS (all existing `test_agent_loop.py` tests still pass; new guard passes)

- [ ] **Step 8: Verify the app still boots (import smoke)**

Run: `rtk python -c "import app"`
Expected: no exception (exit code 0)

- [ ] **Step 9: Commit**

```bash
rtk git add src/agent_loop.py tests/test_sse_indicators.py
rtk git commit -m "feat(m4): emit run_status/autoeval/verifier SSE events from agent loop"
```

---

## Task 3: Frontend — whitelist + inline badges (`chat.js`)

**Files:**
- Modify: `static/js/chat.js` (whitelist `1396`; new branches after `2367`)

- [ ] **Step 1: Add `run_status` to the timeout-bypass whitelist**

At `static/js/chat.js:1396`, the long `if (json.delta || json.type === 'agent_prep' || ...)` condition. Add `run_status` so it keeps the stream alive:

```javascript
              if (json.delta || json.type === 'agent_prep' || json.type === 'tool_start' || json.type === 'tool_output' || json.type === 'tool_progress' || json.type === 'agent_step' || json.type === 'agent_dispatch' || json.type === 'run_status' || json.type === 'doc_stream_open' || json.type === 'doc_stream_delta' || json.type === 'research_progress') {
```

- [ ] **Step 2: Add three handler branches after the `agent_dispatch` block**

Immediately after the `agent_dispatch` block closes at `static/js/chat.js:2367` (the line `}` before `} else if (json.type === 'agent_step') {` at 2369), insert:

```javascript
              } else if (json.type === 'run_status') {
                // Persistent cockpit strip (continuous state). Never blocks chat.
                if (window.cockpit && typeof window.cockpit.update === 'function') {
                  window.cockpit.update(json);
                }

              } else if (json.type === 'autoeval_result') {
                if (_isBg) continue;
                const box = document.getElementById('chat-history');
                if (box) {
                  const decision = json.decision === 'revert' ? 'revert' : 'keep';
                  const reason = String(json.reason || '').replace(/[<>&]/g, '');
                  const el = document.createElement('div');
                  el.className = 'msg msg-system agent-indicator autoeval-' + decision;
                  el.innerHTML = '<span class="agent-dot"></span> autoeval: ' + decision +
                    ' <span class="agent-phase">' + reason + '</span>';
                  box.appendChild(el);
                  box.scrollTop = box.scrollHeight;
                  setTimeout(() => { if (el.parentNode) el.remove(); }, 6000);
                }

              } else if (json.type === 'verifier_result') {
                if (_isBg) continue;
                const box = document.getElementById('chat-history');
                if (box) {
                  const status = json.status === 'fail' ? 'fail' : 'pass';
                  const detail = String(json.detail || '').replace(/[<>&]/g, '');
                  const el = document.createElement('div');
                  el.className = 'msg msg-system agent-indicator verifier-' + status;
                  el.innerHTML = '<span class="agent-dot"></span> verifier: ' + status +
                    ' <span class="agent-phase">' + detail + '</span>';
                  box.appendChild(el);
                  box.scrollTop = box.scrollHeight;
                  setTimeout(() => { if (el.parentNode) el.remove(); }, 6000);
                }
```

- [ ] **Step 3: Verify no syntax errors**

Run: `rtk node --check static/js/chat.js`
Expected: no output (exit code 0)

- [ ] **Step 4: Commit**

```bash
rtk git add static/js/chat.js
rtk git commit -m "feat(m4): chat.js handlers for run_status + autoeval/verifier badges"
```

---

## Task 4: Cockpit module + DOM wiring

**Files:**
- Create: `static/js/cockpit.js`
- Modify: `static/index.html` (container near the in-chat model picker `~1018-1019`; script tag)

- [ ] **Step 1: Write the cockpit module**

```javascript
// static/js/cockpit.js
// Persistent "run cockpit": continuous runtime state as chips.
// Inputs: run_status SSE payloads (via window.cockpit.update) + /api/health poll.
// Output: contents of #run-cockpit. No dependency on chat internals.
(function () {
  'use strict';
  const MUTED = '\u2014'; // em dash "—"

  function esc(s) { return String(s == null ? '' : s).replace(/[<>&]/g, ''); }

  function driftClass(level) {
    if (level === 'high') return 'chip-bad';
    if (level === 'med') return 'chip-warn';
    if (level === 'low') return 'chip-ok';
    return 'chip-muted';
  }

  function setChip(id, label, value, cls) {
    const el = document.getElementById(id);
    if (!el) return;
    el.className = 'cockpit-chip ' + (cls || 'chip-muted');
    el.innerHTML = '<span class="chip-label">' + esc(label) + '</span>' +
      '<span class="chip-val">' + esc(value == null ? MUTED : value) + '</span>';
  }

  const cockpit = {
    update: function (s) {
      if (!s) return;
      setChip('cockpit-phase', 'phase', s.phase || null,
        s.phase ? 'chip-ok' : 'chip-muted');
      setChip('cockpit-drift', 'drift', s.drift || null, driftClass(s.drift));
      if (s.iters && typeof s.iters.used === 'number') {
        setChip('cockpit-iters', 'iters', s.iters.used + '/' + s.iters.max, 'chip-ok');
      }
      if (s.budget && s.budget.pct != null) {
        const cls = s.budget.pct >= 85 ? 'chip-bad' : (s.budget.pct >= 60 ? 'chip-warn' : 'chip-ok');
        setChip('cockpit-budget', 'budget', s.budget.pct + '%', cls);
      } else {
        setChip('cockpit-budget', 'budget', null, 'chip-muted');
      }
    },
    pollHealth: function () {
      fetch('/api/health', { credentials: 'same-origin' })
        .then(function (r) { return r.ok ? r.json() : null; })
        .then(function (j) {
          const ok = j && (j.status === 'ok' || j.ok === true);
          setChip('cockpit-health', 'health', ok ? 'ok' : 'down', ok ? 'chip-ok' : 'chip-bad');
        })
        .catch(function () { setChip('cockpit-health', 'health', '?', 'chip-muted'); });
    },
  };

  window.cockpit = cockpit;
  document.addEventListener('DOMContentLoaded', function () {
    cockpit.pollHealth();
    setInterval(cockpit.pollHealth, 15000);
  });
})();
```

- [ ] **Step 2: Add the cockpit container to index.html**

Immediately after the in-chat model picker wrap opening at `static/index.html:1019` (`<div class="model-picker-wrap" id="model-picker-wrap">` region), add a sibling strip right after that closing `</div>` for the picker wrap. Insert this block:

```html
        <div id="run-cockpit" class="run-cockpit" aria-label="Run status">
          <span id="cockpit-phase" class="cockpit-chip chip-muted"><span class="chip-label">phase</span><span class="chip-val">—</span></span>
          <span id="cockpit-drift" class="cockpit-chip chip-muted"><span class="chip-label">drift</span><span class="chip-val">—</span></span>
          <span id="cockpit-iters" class="cockpit-chip chip-muted"><span class="chip-label">iters</span><span class="chip-val">—</span></span>
          <span id="cockpit-budget" class="cockpit-chip chip-muted"><span class="chip-label">budget</span><span class="chip-val">—</span></span>
          <span id="cockpit-health" class="cockpit-chip chip-muted"><span class="chip-label">health</span><span class="chip-val">—</span></span>
        </div>
```

- [ ] **Step 3: Add the script tag**

Find the existing `<script src="/static/js/chat.js"` tag in `static/index.html` (use `rtk grep -n 'js/chat.js' static/index.html` to locate it) and add immediately BEFORE it:

```html
    <script src="/static/js/cockpit.js"></script>
```

- [ ] **Step 4: Verify cockpit.js syntax**

Run: `rtk node --check static/js/cockpit.js`
Expected: no output (exit code 0)

- [ ] **Step 5: Commit**

```bash
rtk git add static/js/cockpit.js static/index.html
rtk git commit -m "feat(m4): run cockpit module + DOM container"
```

---

## Task 5: Styling

**Files:**
- Modify: `static/css/style.min.css` (append readable rules at end of file)

- [ ] **Step 1: Append cockpit + badge styles**

Append to the end of `static/css/style.min.css`:

```css
.run-cockpit{display:flex;flex-wrap:wrap;gap:6px;align-items:center;margin:4px 0 6px 8px;font-size:.72em}
.cockpit-chip{display:inline-flex;align-items:center;gap:5px;padding:2px 8px;border-radius:10px;border:1px solid var(--border);line-height:1.4}
.cockpit-chip .chip-label{opacity:.55;text-transform:uppercase;letter-spacing:.04em;font-size:.85em}
.cockpit-chip .chip-val{font-weight:600}
.cockpit-chip.chip-muted{opacity:.5}
.cockpit-chip.chip-ok .chip-val{color:var(--green,#50fa7b)}
.cockpit-chip.chip-warn .chip-val{color:var(--warning,#f1fa8c)}
.cockpit-chip.chip-bad .chip-val{color:var(--accent,#ff5555)}
.agent-indicator.autoeval-revert .agent-dot,.agent-indicator.verifier-fail .agent-dot{background:var(--accent,#ff5555)}
.agent-indicator.autoeval-keep .agent-dot,.agent-indicator.verifier-pass .agent-dot{background:var(--green,#50fa7b)}
```

- [ ] **Step 2: Commit**

```bash
rtk git add static/css/style.min.css
rtk git commit -m "feat(m4): cockpit + indicator badge styles"
```

---

## Task 6: Browser verification (Playwright MCP)

**Files:** none (manual verification with `mcp__playwright__*` tools per CLAUDE.md)

- [ ] **Step 1: Start the app**

Run: `rtk python -m uvicorn app:app --port 8000` (background)
Expected: server listening on :8000

- [ ] **Step 2: Load and log in**

Use `browser_navigate` to `http://localhost:8000`, `browser_snapshot`. Log in if the login page appears.

- [ ] **Step 3: Verify cockpit renders muted by default**

`browser_snapshot`. Confirm `#run-cockpit` shows 5 chips; with all M3 modules OFF (default), `drift`/`budget` show "—" (muted) and `health` resolves to "ok". Confirm NO fabricated drift/budget values.

- [ ] **Step 4: Drive a chat turn and watch live updates**

Send a chat message via the UI. During streaming, confirm via `browser_snapshot` that `phase` and `iters` chips update (phase from the live loop, `iters` counting up). Confirm no console errors via `browser_console_messages`.

- [ ] **Step 5: Confirm inline badges are wired (visual smoke)**

Confirm the `autoeval_result`/`verifier_result` handlers exist and don't error even though the modules are OFF (no badge is expected when OFF — that is correct). Manually inject a test event to confirm rendering: run in `browser_evaluate`:
```javascript
window.cockpit.update({phase:'quality',drift:'high',iters:{used:4,max:12},budget:{pct:88,tokens:9000}});
```
Confirm chips flip: `drift`=high (red), `budget`=88% (red), `phase`=quality, `iters`=4/12.

- [ ] **Step 6: Stop the server + final commit if any fixes were made**

Stop uvicorn. If Steps 3-5 required fixes, commit them:
```bash
rtk git add -A
rtk git commit -m "fix(m4): cockpit verification adjustments"
```

---

## Self-Review

**Spec coverage:**
- Consolidated `run_status` event → Task 2 Steps 3-4. ✓
- Discrete `autoeval_result`/`verifier_result` → Task 2 Step 5, Task 3 Step 2. ✓
- Reused `teacher_takeover`/`budget_exceeded` → already emitted (spec §4.3); no new work, out of scope for badges beyond existing rendering. ✓
- Cockpit strip (phase/drift/budget/health) → Task 4. ✓
- Honesty rule (null → "—") → `run_status_event` returns None fields (Task 1); `cockpit.update` renders MUTED (Task 4); verified Task 6 Step 3. ✓
- Inline badges reuse `.agent-indicator` + inline sanitization → Task 3 Step 2. ✓
- CSS reuses `--accent`/`--green` + warning var → Task 5. ✓
- Testing (payload shape + browser) → Tasks 1, 2, 6. ✓

**Placeholder scan:** No TBD/TODO; every code step has complete code. ✓

**Type consistency:** `run_status_event`/`autoeval_event`/`verifier_event`/`normalize_drift` signatures identical across Tasks 1-2. Event `type` strings (`run_status`, `autoeval_result`, `verifier_result`) match between backend emit (Task 2) and frontend handlers (Task 3). Chip element IDs (`cockpit-phase/drift/iters/budget/health`) match between index.html (Task 4 Step 2) and cockpit.js `setChip` calls (Task 4 Step 1). ✓

**Note for executor:** Line numbers reference commit `2821daa` state; if drifted, locate by the quoted anchor code rather than the number.
