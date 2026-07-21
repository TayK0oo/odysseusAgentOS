# Kill-switches Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose the ~35 `ODYSSEUS_*` kill-switches as a read-only dashboard inside the existing settings "System" panel, in the cockpit visual style.

**Architecture:** A pure registry module (`src/killswitch_registry.py`) enumerates curated switch descriptors and reads their live `os.environ` state with no side effects. A bare `APIRouter` (`routes/killswitch_routes.py`) serves that state at `GET /api/killswitches`, registered in the M3 block of `app.py`. The frontend adds one `admin-card` to the existing System panel, rendered by `initKillswitchesView()` in `static/js/admin.js`, reusing the cockpit chip CSS classes from sub-projet A.

**Tech Stack:** Python 3.14, FastAPI, pytest (`monkeypatch`), vanilla JS ES-modules, existing `data/` cockpit CSS.

**Conventions (verified against the repo):**
- Every shell command is prefixed with `rtk` (mandatory wrapper).
- Route files export a bare `router = APIRouter(prefix=...)`; registered in `app.py` M3 block (~lines 803-824) via `from routes.X import router as X_router` + `app.include_router(X_router)`.
- Handlers are `async`, wrapped in `try/except` returning `{"ok": bool, ...}` / `{"ok": False, "error": str(e)}`.
- Truthy env convention: `str(v).strip().lower() in ("1","true","yes","on")`.
- Frontend sanitization: `String(x).replace(/[<>&]/g,'')` on any injected text (same as sub-projet A).

---

### Task 1: Kill-switch registry module

**Files:**
- Create: `src/killswitch_registry.py`
- Test: `tests/test_killswitch_registry.py`

- [ ] **Step 1: Write the failing tests**

Create `tests/test_killswitch_registry.py`:

```python
import pytest
from src import killswitch_registry as ksr


def test_unset_var_is_default_and_uses_code_default(monkeypatch):
    monkeypatch.delenv("ODYSSEUS_AUTOEVAL", raising=False)
    row = _find(ksr.read_states(), "ODYSSEUS_AUTOEVAL")
    assert row["is_default"] is True
    assert row["raw"] is None
    assert row["effective"] is False  # default "off"


def test_set_on_is_effective_true(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_AUTOEVAL", "on")
    row = _find(ksr.read_states(), "ODYSSEUS_AUTOEVAL")
    assert row["is_default"] is False
    assert row["raw"] == "on"
    assert row["effective"] is True


def test_set_off_is_effective_false(monkeypatch):
    monkeypatch.setenv("ODYSSEUS_LIVE_ORCHESTRATION", "off")
    row = _find(ksr.read_states(), "ODYSSEUS_LIVE_ORCHESTRATION")
    assert row["effective"] is False
    assert row["is_default"] is False


def test_destructive_gate_defaults_on(monkeypatch):
    monkeypatch.delenv("ODYSSEUS_DESTRUCTIVE_GATE", raising=False)
    row = _find(ksr.read_states(), "ODYSSEUS_DESTRUCTIVE_GATE")
    assert row["is_default"] is True
    assert row["effective"] is True  # default "on"


def test_every_descriptor_has_required_keys():
    required = {"name", "env_var", "default", "category", "timing", "desc", "source"}
    for row in ksr.read_states():
        assert required.issubset(row.keys()), row
        assert row["source"], f"empty source for {row['env_var']}"
        assert row["timing"] in ("startup", "runtime")


def test_categories_cover_all_switches():
    cats = set(ksr.categories())
    for row in ksr.read_states():
        assert row["category"] in cats


def _find(rows, env_var):
    for r in rows:
        if r["env_var"] == env_var:
            return r
    raise AssertionError(f"{env_var} not in registry")
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `rtk python -m pytest tests/test_killswitch_registry.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'src.killswitch_registry'`

- [ ] **Step 3: Write the registry module**

Create `src/killswitch_registry.py`:

```python
"""Read-only registry of odysseus kill-switches.

Pure module (no file I/O, no mutation of os.environ). Each descriptor is
curated and cites the exact code location where the switch is read, so the
dashboard stays honest and verifiable. read_states() resolves the live
os.environ value for each switch; a switch whose env var is unset reports
is_default=True and falls back to the code default.
"""
from __future__ import annotations

import os
from typing import Any, Optional

_TRUTHY = ("1", "true", "yes", "on")


def _truthy(value: Optional[str]) -> bool:
    return str(value).strip().lower() in _TRUTHY if value is not None else False


# Curated set — grounded in code (rg 'os\\.(environ\\.get|getenv)\\("ODYSSEUS_')
# plus the agent/channel gates. Config params (paths, ids, models) are excluded.
_SWITCHES: list[dict[str, Any]] = [
    # ── Orchestration ──
    {"name": "Live orchestration", "env_var": "ODYSSEUS_LIVE_ORCHESTRATION",
     "default": "off", "category": "Orchestration", "timing": "runtime",
     "desc": "CanonicalLoop 7 phases.", "source": "src/agent_loop.py:2493"},
    {"name": "Phase tracker", "env_var": "ODYSSEUS_PHASE_TRACKER",
     "default": "off", "category": "Orchestration", "timing": "runtime",
     "desc": "Infere la phase par round et la pousse au ToolRegistry.",
     "source": "src/orchestrator/phase_tracker.py:25"},
    {"name": "Model router", "env_var": "ODYSSEUS_MODEL_ROUTER",
     "default": "off", "category": "Orchestration", "timing": "runtime",
     "desc": "Routing advisory log-only (n'ecrase pas le modele user).",
     "source": "src/orchestrator/router_advice.py:42"},
    {"name": "Destructive gate", "env_var": "ODYSSEUS_DESTRUCTIVE_GATE",
     "default": "on", "category": "Orchestration", "timing": "runtime",
     "desc": "Garde-fou operations destructrices (ON par defaut).",
     "source": "src/orchestrator/gate.py:26"},
    # ── Governance / Memory ──
    {"name": "Autoeval", "env_var": "ODYSSEUS_AUTOEVAL",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Decision keep/revert post-build.",
     "source": "src/orchestrator/autoeval.py:44"},
    {"name": "Autoevolve", "env_var": "ODYSSEUS_AUTOEVOLVE",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Recherche d'amelioration apres drift HIGH + revert.",
     "source": "src/orchestrator/autoevolve.py:18"},
    {"name": "CodeBurn", "env_var": "ODYSSEUS_CODEBURN",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Rapport one-shot rate / waste / cout.",
     "source": "src/orchestrator/codeburn_runner.py:26"},
    {"name": "Governance ancestry", "env_var": "ODYSSEUS_GOVERNANCE_ANCESTRY",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Ecrit GoalTask avec ancestry mission->goal->project->task.",
     "source": "src/orchestrator/ancestry_tracker.py:26"},
    {"name": "Checkpoint", "env_var": "ODYSSEUS_CHECKPOINT",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Persiste un checkpoint post-round dans Obsidian.",
     "source": "src/orchestrator/checkpoint_tracker.py:39"},
    {"name": "Unified tokens", "env_var": "ODYSSEUS_UNIFIED_TOKENS",
     "default": "off", "category": "Governance/Memory", "timing": "runtime",
     "desc": "Comptage tokens unifie avec run_id de correlation.",
     "source": "src/trace_writer.py:146"},
    # ── RAG ──
    {"name": "RRF fusion", "env_var": "ODYSSEUS_RRF_FUSION",
     "default": "off", "category": "RAG", "timing": "runtime",
     "desc": "Fusion vecteur+BM25 par Reciprocal Rank Fusion.",
     "source": "src/rag_vector.py:50"},
    # ── MCP / Services ──
    {"name": "Disable MCP", "env_var": "ODYSSEUS_DISABLE_MCP",
     "default": "off", "category": "MCP/Services", "timing": "startup",
     "desc": "Coupe tous les serveurs MCP built-in (lu au boot).",
     "source": "src/builtin_mcp.py:89"},
    {"name": "Obsidian MCP", "env_var": "ODYSSEUS_OBSIDIAN_MCP",
     "default": "off", "category": "MCP/Services", "timing": "runtime",
     "desc": "Active le serveur MCP Obsidian.",
     "source": "src/builtin_mcp.py:98"},
    {"name": "Graphify", "env_var": "ODYSSEUS_GRAPHIFY",
     "default": "off", "category": "MCP/Services", "timing": "runtime",
     "desc": "Active le serveur MCP Graphify.",
     "source": "src/builtin_mcp.py:107"},
    {"name": "Zen from endpoint", "env_var": "ODYSSEUS_ZEN_FROM_ENDPOINT",
     "default": "off", "category": "MCP/Services", "timing": "runtime",
     "desc": "Route Zen via un endpoint enregistre.",
     "source": "src/zen_router.py:131"},
    # ── Channels ──
    {"name": "In-process Discord", "env_var": "ODYSSEUS_INPROCESS_DISCORD",
     "default": "off", "category": "Channels", "timing": "startup",
     "desc": "Bootstrap du bot Discord in-process.",
     "source": "src/channel_bootstrap.py:45"},
    {"name": "In-process Telegram", "env_var": "ODYSSEUS_INPROCESS_TELEGRAM",
     "default": "off", "category": "Channels", "timing": "startup",
     "desc": "Bootstrap du bot Telegram in-process.",
     "source": "src/channel_bootstrap.py:50"},
    {"name": "Channel agent reply", "env_var": "ODYSSEUS_CHANNEL_AGENT_REPLY",
     "default": "off", "category": "Channels", "timing": "runtime",
     "desc": "Reponse auto de l'agent sur les channels.",
     "source": "src/channel_bootstrap.py:59"},
    # ── Agents ──
    {"name": "Agent catalog", "env_var": "ODYSSEUS_AGENT_CATALOG",
     "default": "off", "category": "Agents", "timing": "startup",
     "desc": "Charge le catalogue d'agents .opencode au boot.",
     "source": "app.py:1254"},
]

# 12 phase/explicit agent switches (agent_dispatcher.py). All default off,
# runtime-read via AgentDispatcher.dispatch_for_phase.
_AGENT_SWITCHES = [
    ("Agent: constitution", "ODYSSEUS_AGENT_CONSTITUTION"),
    ("Agent: gsd-planner", "ODYSSEUS_AGENT_PLANNER"),
    ("Agent: gsd-researcher", "ODYSSEUS_AGENT_RESEARCHER"),
    ("Agent: gsd-executor", "ODYSSEUS_AGENT_EXECUTOR"),
    ("Agent: gsd-debugger", "ODYSSEUS_AGENT_DEBUGGER"),
    ("Agent: debate-5-personas", "ODYSSEUS_AGENT_DEBATE"),
    ("Agent: security-audit", "ODYSSEUS_AGENT_SECURITY"),
    ("Agent: gsd-verifier", "ODYSSEUS_AGENT_VERIFIER"),
    ("Agent: edge-case-gen", "ODYSSEUS_AGENT_EDGECASE"),
    ("Agent: gsd-roadmapper", "ODYSSEUS_AGENT_ROADMAPPER"),
    ("Agent: design-extract", "ODYSSEUS_AGENT_DESIGN_EXTRACT"),
    ("Agent: open-design", "ODYSSEUS_AGENT_OPEN_DESIGN"),
]
for _label, _var in _AGENT_SWITCHES:
    _SWITCHES.append({
        "name": _label, "env_var": _var, "default": "off",
        "category": "Agents", "timing": "runtime",
        "desc": "Dispatch auto de l'agent par phase canonique.",
        "source": "src/orchestrator/agent_dispatcher.py",
    })

_CATEGORY_ORDER = [
    "Orchestration", "Governance/Memory", "RAG",
    "MCP/Services", "Channels", "Agents",
]


def categories() -> list[str]:
    """Stable display order of switch categories."""
    return list(_CATEGORY_ORDER)


def read_states() -> list[dict[str, Any]]:
    """Return each descriptor enriched with live env state.

    Adds raw (env value or None), is_default (True if unset), and effective
    (resolved bool). Best-effort per descriptor: never raises.
    """
    rows: list[dict[str, Any]] = []
    for d in _SWITCHES:
        raw = os.environ.get(d["env_var"])
        is_default = raw is None
        effective = _truthy(d["default"]) if is_default else _truthy(raw)
        rows.append({**d, "raw": raw, "is_default": is_default, "effective": effective})
    return rows
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `rtk python -m pytest tests/test_killswitch_registry.py -v`
Expected: PASS (6 passed)

- [ ] **Step 5: Commit**

```bash
rtk git add src/killswitch_registry.py tests/test_killswitch_registry.py
rtk git commit -m "feat(m5): kill-switch registry — pure read-only env state module"
```

---

### Task 2: API route

**Files:**
- Create: `routes/killswitch_routes.py`
- Modify: `app.py` (M3 block, after the autoeval router registration ~line 816)
- Test: `tests/test_killswitch_routes.py`

- [ ] **Step 1: Write the failing test**

Create `tests/test_killswitch_routes.py`:

```python
from fastapi import FastAPI
from fastapi.testclient import TestClient

from routes.killswitch_routes import router


def _client():
    app = FastAPI()
    app.include_router(router)
    return TestClient(app)


def test_list_killswitches_ok():
    resp = _client().get("/api/killswitches")
    assert resp.status_code == 200
    body = resp.json()
    assert body["ok"] is True
    assert isinstance(body["switches"], list) and body["switches"]
    assert "Orchestration" in body["categories"]
    first = body["switches"][0]
    assert {"env_var", "effective", "is_default", "category"}.issubset(first)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `rtk python -m pytest tests/test_killswitch_routes.py -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'routes.killswitch_routes'`

- [ ] **Step 3: Write the route module**

Create `routes/killswitch_routes.py`:

```python
"""Kill-switches dashboard route — read-only state of ODYSSEUS_* switches."""
from fastapi import APIRouter

from src.killswitch_registry import read_states, categories

router = APIRouter(prefix="/api/killswitches", tags=["killswitches"])


@router.get("")
async def list_killswitches():
    """Retourne l'etat reel de tous les kill-switches (lecture seule)."""
    try:
        return {"ok": True, "categories": categories(), "switches": read_states()}
    except Exception as e:
        return {"ok": False, "error": str(e)}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `rtk python -m pytest tests/test_killswitch_routes.py -v`
Expected: PASS (1 passed)

- [ ] **Step 5: Register the router in app.py**

In `app.py`, find the autoeval registration block (~line 814-816):

```python
# ========= AUTOEVAL LOOP (Phase 7 — autoresearch pattern) =========
from routes.autoeval_routes import router as autoeval_router
app.include_router(autoeval_router)
```

Insert immediately after it:

```python
# ========= KILL-SWITCHES DASHBOARD (Sub-projet B — lecture seule) =========
from routes.killswitch_routes import router as killswitch_router
app.include_router(killswitch_router)
```

- [ ] **Step 6: Verify app boots and route is wired**

Run: `rtk python -c "import app"`
Expected: no import error (boot logs end normally).

Run: `rtk python -c "from app import app; print([r.path for r in app.routes if 'killswitch' in r.path])"`
Expected: `['/api/killswitches']`

- [ ] **Step 7: Commit**

```bash
rtk git add routes/killswitch_routes.py tests/test_killswitch_routes.py app.py
rtk git commit -m "feat(m5): GET /api/killswitches route + M3 registration"
```

---

### Task 3: System-panel card (HTML)

**Files:**
- Modify: `static/index.html` (System panel `data-settings-panel="system"`, after the Terminal Logs `admin-card` closing `</div>` at line 2357)

- [ ] **Step 1: Insert the kill-switches card**

In `static/index.html`, locate the end of the Terminal Logs card (line 2357, the `</div>` that closes `id="settings-system-logs-card"`, immediately before `<div class="admin-card">` for Data Backup at line 2358). Insert this new card between them:

```html
          <div class="admin-card" id="settings-killswitches-card">
            <h2>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="vertical-align:-2px;margin-right:5px;opacity:0.6">
                <path d="M12 2v6m0 0a4 4 0 1 0 0 8 4 4 0 0 0 0-8z"/><line x1="12" y1="16" x2="12" y2="22"/>
              </svg>
              Kill-switches
            </h2>
            <div class="admin-toggle-sub" style="margin-bottom:8px">Etat reel des modules dormants (lecture seule). "defaut" = valeur si la variable d'environnement n'est pas definie.</div>
            <div class="settings-system-logs-controls" style="margin-bottom:8px">
              <button type="button" class="admin-btn-sm" id="killswitches-refresh-btn">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
                Refresh
              </button>
            </div>
            <div id="killswitches-container">
              <div class="settings-system-logs-placeholder">Chargement…</div>
            </div>
          </div>
```

- [ ] **Step 2: Verify the markup is present and balanced**

Run: `rtk python -c "import pathlib,re; h=pathlib.Path('static/index.html').read_text(encoding='utf-8'); print('card' , 'settings-killswitches-card' in h); print('container', 'killswitches-container' in h); print('refresh', 'killswitches-refresh-btn' in h)"`
Expected: three `True` lines.

- [ ] **Step 3: Commit**

```bash
rtk git add static/index.html
rtk git commit -m "feat(m5): kill-switches admin-card in System settings panel"
```

---

### Task 4: Frontend render logic (JS)

**Files:**
- Modify: `static/js/admin.js` (add `initKillswitchesView()`; add it to the `inits` array in `initAll()` at line 3029)

- [ ] **Step 1: Add the render function**

In `static/js/admin.js`, immediately before the `INIT & REFRESH` comment banner (line 3022), insert:

```javascript
/* ═══════════════════════════════════════════
   KILL-SWITCHES DASHBOARD (read-only)
   ═══════════════════════════════════════════ */
function ksClean(s) { return String(s == null ? '' : s).replace(/[<>&]/g, ''); }

function renderKillswitches(data) {
  const container = el('killswitches-container');
  if (!container) return;
  if (!data || data.ok !== true || !Array.isArray(data.switches)) {
    container.innerHTML = '<div class="settings-system-logs-placeholder">Indisponible.</div>';
    return;
  }
  const byCat = {};
  for (const sw of data.switches) {
    (byCat[sw.category] = byCat[sw.category] || []).push(sw);
  }
  let html = '';
  for (const cat of data.categories) {
    const rows = byCat[cat];
    if (!rows || !rows.length) continue;
    html += '<div class="ks-cat-title">' + ksClean(cat) + '</div>';
    for (const sw of rows) {
      const on = sw.effective === true;
      // Destructive gate ON = protection active (good); OFF = warn.
      let chipCls = on ? 'chip-ok' : 'chip-muted';
      if (sw.env_var === 'ODYSSEUS_DESTRUCTIVE_GATE') chipCls = on ? 'chip-ok' : 'chip-warn';
      const state = on ? 'ON' : 'OFF';
      const badges =
        (sw.is_default ? '<span class="ks-badge">defaut</span>' : '') +
        (sw.timing === 'startup' ? '<span class="ks-badge">startup</span>' : '');
      html += '<div class="ks-row" title="' + ksClean(sw.desc) + ' — ' + ksClean(sw.source) + '">' +
        '<span class="ks-name">' + ksClean(sw.name) +
        ' <code>' + ksClean(sw.env_var) + '</code>' + badges + '</span>' +
        '<span class="cockpit-chip ' + chipCls + '">' +
        '<span class="chip-val">' + state + '</span></span>' +
        '</div>';
    }
  }
  container.innerHTML = html || '<div class="settings-system-logs-placeholder">Aucun switch.</div>';
}

function loadKillswitches() {
  const container = el('killswitches-container');
  fetch('/api/killswitches', { credentials: 'same-origin' })
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(renderKillswitches)
    .catch(function () {
      if (container) container.innerHTML =
        '<div class="settings-system-logs-placeholder">Indisponible.</div>';
    });
}

function initKillswitchesView() {
  const btn = el('killswitches-refresh-btn');
  if (btn) btn.addEventListener('click', loadKillswitches);
  loadKillswitches();
}
```

- [ ] **Step 2: Wire it into initAll**

In `static/js/admin.js`, find the `inits` array in `initAll()` (line 3027-3031):

```javascript
  const inits = [
    initSignupToggle, initShareDefaultsToggle, initAddUser, initEndpointForm, initMcpForm,
    initCalDAV, initBackup, initDangerZone, initTokenForm, initLogsView,
    () => settingsModule.initIntegrations()
  ];
```

Change the `initLogsView` line to add `initKillswitchesView`:

```javascript
  const inits = [
    initSignupToggle, initShareDefaultsToggle, initAddUser, initEndpointForm, initMcpForm,
    initCalDAV, initBackup, initDangerZone, initTokenForm, initLogsView, initKillswitchesView,
    () => settingsModule.initIntegrations()
  ];
```

- [ ] **Step 3: Verify the JS parses (no syntax error)**

Run: `rtk node --check static/js/admin.js && echo OK || echo "node unavailable — skip"`
Expected: `OK` if node is installed; otherwise `node unavailable — skip` (acceptable — the ES-module is loaded by the browser at runtime, and the change is a localized function addition).

- [ ] **Step 4: Commit**

```bash
rtk git add static/js/admin.js
rtk git commit -m "feat(m5): initKillswitchesView renders switch state as cockpit chips"
```

---

### Task 5: Styles

**Files:**
- Modify: `static/css/style.min.css` (append at end of file, alongside the sub-projet A cockpit rules)

- [ ] **Step 1: Append the kill-switch layout rules**

At the very end of `static/css/style.min.css`, append:

```css
.ks-cat-title{font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.04em;opacity:.55;margin:10px 0 4px}
.ks-row{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:4px 0;border-bottom:1px solid color-mix(in srgb,var(--fg) 8%,transparent)}
.ks-name{font-size:12px;display:flex;align-items:center;gap:6px;flex-wrap:wrap}
.ks-name code{font-size:11px;opacity:.7}
.ks-badge{font-size:9px;text-transform:uppercase;letter-spacing:.03em;padding:1px 5px;border-radius:6px;background:color-mix(in srgb,var(--fg) 12%,transparent);opacity:.75}
```

- [ ] **Step 2: Verify the rules landed**

Run: `rtk python -c "import pathlib; c=pathlib.Path('static/css/style.min.css').read_text(encoding='utf-8'); print('ks-row', '.ks-row' in c); print('ks-badge', '.ks-badge' in c)"`
Expected: two `True` lines.

- [ ] **Step 3: Commit**

```bash
rtk git add static/css/style.min.css
rtk git commit -m "feat(m5): kill-switch dashboard layout styles"
```

---

### Task 6: End-to-end verification

**Files:** none (verification only)

- [ ] **Step 1: Run the full new test set**

Run: `rtk python -m pytest tests/test_killswitch_registry.py tests/test_killswitch_routes.py -v`
Expected: PASS (7 passed total).

- [ ] **Step 2: Boot smoke + live route response**

Run: `rtk python -c "import app"`
Expected: boots without import error.

- [ ] **Step 3: Verify the served assets contain the wiring**

Run: `rtk python -c "import pathlib; h=pathlib.Path('static/index.html').read_text(encoding='utf-8'); js=pathlib.Path('static/js/admin.js').read_text(encoding='utf-8'); print('card', 'settings-killswitches-card' in h); print('initfn', 'initKillswitchesView' in js); print('wired', 'initLogsView, initKillswitchesView' in js)"`
Expected: three `True` lines.

- [ ] **Step 4: Confirm no os.environ mutation (honesty/no-side-effect guard)**

Run: `rtk python -c "import pathlib; src=pathlib.Path('src/killswitch_registry.py').read_text(encoding='utf-8'); assert 'os.environ[' not in src and 'setenv' not in src and 'os.environ.pop' not in src; print('no mutation: OK')"`
Expected: `no mutation: OK`

- [ ] **Step 5: Final commit (if any verification tweaks were needed)**

```bash
rtk git add -A
rtk git commit -m "test(m5): verify kill-switches dashboard wiring end-to-end" || echo "nothing to commit"
```

---

## Notes for the implementer

- **Do NOT add a toggle.** This iteration is strictly read-only. Writing to `.env` or `os.environ` is explicitly out of scope (spec §"Hors périmètre").
- **Honesty rule:** never fabricate a value. Unset var → `(default)` badge + code default. A read failure → muted chip, never a guessed state.
- **Reuse, don't recreate:** the `.cockpit-chip`, `.chip-ok`, `.chip-muted`, `.chip-warn` classes already exist from sub-projet A. Only the `.ks-*` layout classes are new.
- **Playwright is not available** in this environment; browser verification is server-side (asset content + route response) only. State that limitation honestly when reporting completion rather than claiming a full UI click-through.
