# Live Indicators & Run Cockpit — Design (Sub-project A)

**Date**: 2026-07-07
**Author**: Théo Cornu (+ Claude)
**Status**: Approved design → implementation planning
**Scope**: Sub-project A of the "tout passer par l'UI" initiative. Surfaces backend runtime signals as live UI indicators in the same style as the M4 agent-dispatch badges.

---

## 1. Motivation

The UI-coverage audit (`INVENTAIRE/C-COUVERTURE-UI.md`) found ~38% of backend capabilities are UI blind spots. Many runtime signals (drift score, iteration budget, current phase, autoeval decisions, verifier results, teacher escalation, health) are **computed in the live loop but only logged or gated OFF** — the user never sees them.

This sub-project makes those signals **visible in the UI**, reusing the M4 agent-indicator pattern (commit `2b58fd3`) so the new indicators match the existing style. It is the first of the "cockpit UI" chain; Sub-project B (kill-switches dashboard) will follow to let the user *activate* the dormant modules that feed these indicators.

**Non-goals (YAGNI)**: no historical charts, no metrics persistence, no kill-switch toggles (→ Sub-project B), no changes to orchestration logic (→ Sub-project D).

---

## 2. Existing pattern (must match)

- **SSE emission** (`src/agent_loop.py`): inline `yield f'data: {json.dumps({"type": ...})}\n\n'`. No emit helper. Discrete events queued in `_agent_events` and flushed at round start (`agent_loop.py:~2530`). A frontend timeout-bypass whitelist lists "live" event types (`static/js/chat.js:~1396`).
- **Frontend rendering** (`static/js/chat.js:2336-2367`): a large `else if (json.type === '...')` switch. Inline sanitization via `String(x).replace(/[<>&]/g, '')`. Badges use classes `msg msg-system agent-indicator` + state class `agent-running`/`agent-completed`.
- **CSS** (`static/css/style.min.css:2063-2069`): `.agent-indicator`, `.agent-dot`, `agent-pulse` keyframe, CSS vars `--accent` / `--green`.
- **No canonical SSE event registry** — event types are freeform, discovered by reading handlers.

### Data-source reality (evidence)

| Signal | Location | Currently emitted? |
|---|---|---|
| Drift score (low/med/high) | `agent_loop.py:3654` `Observer.compute_drift_score()` | No — log-only |
| Iteration/budget hard-stop | `agent_loop.py:3157` | Yes — `budget_exceeded` |
| Phase | `agent_loop.py:2548` `PhaseTracker.infer_phase()` | Partial — inside `agent_dispatch` |
| Autoeval keep/revert | `agent_loop.py:3682` `apply_autoeval()` | No — log-only, gated OFF |
| Teacher escalation | `agent_loop.py:3722` `run_teacher_inline()` | Yes — `teacher_takeover` |
| Verifier subagent | `agent_loop.py:1801+` | No — no dedicated event |
| Health / readiness | `/api/health`, `/api/ready` | REST only |

---

## 3. Architecture

Two complementary surfaces:

1. **Run Cockpit** — a persistent chip strip for *continuous* state (phase, drift, iteration budget, health). Fed by one consolidated SSE event `run_status` per round, plus periodic `/api/health` polling.
2. **Inline badges** — *discrete* moment events (autoeval keep/revert, verifier pass/fail, teacher takeover, budget exceeded) rendered as ephemeral system messages in the chat stream, reusing the `agent-indicator` classes.

```
stream_agent_loop (backend)
  ├── end of each round → yield run_status {phase, drift, iters, budget}
  ├── on autoeval decision → yield autoeval_result {decision, reason}
  ├── on verifier finish   → yield verifier_result {status, detail}
  ├── (existing) teacher_takeover, budget_exceeded
        │
        ▼  SSE stream (single connection, existing)
  ┌──────────────────────────────┬─────────────────────────────┐
  │ cockpit.js                   │ chat.js switch (+2 handlers) │
  │  updates #run-cockpit chips  │  renders inline badges       │
  │  polls /api/health           │  reuses .agent-indicator     │
  └──────────────────────────────┴─────────────────────────────┘
```

---

## 4. Backend — SSE protocol changes (`src/agent_loop.py`)

### 4.1 New consolidated event `run_status`
Emitted at the end of each round, after drift/phase/budget are computed:
```json
{
  "type": "run_status",
  "phase": "build",
  "drift": "low",            // "low" | "med" | "high" | null (module off)
  "iters": {"used": 3, "max": 12},
  "budget": {"pct": 42, "tokens": 18450}   // pct of hard-stop consumed; null if unknown
}
```
- Added to the timeout-bypass whitelist in `chat.js`.
- **Honesty rule**: a field is `null` when its module is inactive/not computed. The frontend renders `null` as a muted "—" chip. **No fabricated values** — this is precisely what motivates Sub-project B (turn modules on).

### 4.2 New discrete events
- `autoeval_result` → `{type, decision: "keep"|"revert", reason}` emitted where `apply_autoeval()` decides (only when module active).
- `verifier_result` → `{type, status: "pass"|"fail", detail}` emitted after the verifier subagent completes.

### 4.3 Reused existing events
- `teacher_takeover`, `budget_exceeded` — already emitted; frontend just needs badge rendering parity.

### 4.4 Constraints
- Emissions are additive `yield`s only; **no restructuring of `stream_agent_loop`**.
- Every new payload is JSON-serializable and small.
- No new event fires when its source module is gated OFF (except `run_status`, which fires every round with `null` fields for inactive signals).

---

## 5. Frontend

### 5.1 Cockpit (`static/js/cockpit.js` — new module)
- Owns the `#run-cockpit` DOM strip, inserted under the existing model-name bar.
- Subscribes to the **existing** SSE stream (no second connection): a small hook in `chat.js` forwards `run_status` events to `cockpit.update(payload)`, or `cockpit.js` exposes a handler the chat switch calls.
- Chips: **Phase**, **Drift** (color: green/amber/red via `--green`/warning/`--accent`), **Budget** (`used/max` iters + token pct bar), **Health** (dot from `/api/health` polled every 15s).
- `null`/inactive → muted "—" chip with a tooltip "module inactif" linking mentally to Sub-project B.
- Single responsibility; inputs = SSE payloads + `/api/health`; output = cockpit DOM. Independently testable.

### 5.2 Inline badges (`static/js/chat.js`)
- Add two `else if` branches: `autoeval_result`, `verifier_result`.
- Render with classes `msg msg-system agent-indicator` + a state class (`autoeval-keep`/`autoeval-revert`, `verifier-pass`/`verifier-fail`), icon + sanitized text via `String(x).replace(/[<>&]/g, '')`.
- Ensure `teacher_takeover`/`budget_exceeded` also render a badge (parity), if not already visible.

### 5.3 CSS (`static/css/style.min.css`)
- `#run-cockpit` layout (flex strip, chips), reusing `--accent`/`--green`; add a warning var (amber) for drift-med/budget-low.
- Badge state classes matching `.agent-indicator` sizing/opacity conventions.

---

## 6. Error handling & edge cases

- **SSE field missing/malformed** → cockpit renders "—" chip, never throws (defensive read with defaults).
- **`/api/health` unreachable** → health dot goes grey ("unknown"), poll retries next interval.
- **Module OFF** → `null` fields → muted chips; discrete events simply never arrive (no error).
- **XSS** → all interpolated strings sanitized with the same inline replace used by M4.
- **Rapid rounds** → cockpit updates are idempotent (replace chip contents, no accumulation); inline badges auto-remove after a timeout like M4 (`setTimeout … remove()`).

---

## 7. Testing

- **Backend**: unit-test the shape of `run_status` / `autoeval_result` / `verifier_result` payloads (JSON-serializable, correct fields, `null` when module off). Assert `stream_agent_loop` still passes existing tests (no regressions in the 4196-test suite).
- **Frontend**: manual/browser verification via Playwright MCP — drive a chat run, assert cockpit chips populate and inline badges render/disappear. Verify muted "—" when modules OFF.
- **Honesty check**: with all M3 modules OFF (default), cockpit shows phase (from live loop) + "—" for drift/autoeval/verifier; no fabricated values.

---

## 8. File touch-list (estimate)

| File | Change |
|---|---|
| `src/agent_loop.py` | +3 `yield` emitters (`run_status`, `autoeval_result`, `verifier_result`) |
| `static/js/chat.js` | +1 whitelist entry, +2 switch branches, forward `run_status` to cockpit |
| `static/js/cockpit.js` | **new** — cockpit module |
| `static/index.html` | +`#run-cockpit` container + `cockpit.js` script tag |
| `static/css/style.min.css` | cockpit + badge state styles |
| tests | payload-shape tests |

---

## 9. Follow-on (out of scope here)

- **Sub-project B**: kill-switches dashboard to activate the modules that light up the `null` chips.
- **Sub-project C**: Trinité knowledge panel.
- **D/E**: orchestrator consolidation, ghost cleanup.
