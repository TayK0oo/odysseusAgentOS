# Trinité Knowledge Panel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a read-only "Trinité — Connaissance" admin-card to the System settings panel showing the live health of the three knowledge legs (CBM, VectorRAG, Obsidian) as cockpit chips.

**Architecture:** Reuse the EXISTING `GET /api/knowledge/status` endpoint (no backend changes). Add one HTML card + one JS init function mirroring Sub-projet B's kill-switches card exactly, reusing B's `.ks-*` CSS classes, A's `.cockpit-chip` classes, and B's hardened `ksClean()` sanitizer.

**Tech Stack:** FastAPI (backend, untouched), vanilla JS ES-module (`static/js/admin.js`), FastAPI TestClient + pytest.

---

## Conventions & grounding (read before starting)

- **RTK rule:** EVERY shell command MUST be prefixed with `rtk` (e.g. `rtk python -m pytest`, `rtk git commit`). No exceptions.
- **The endpoint is fixed.** `routes/knowledge_routes.py:164-176` defines `GET /api/knowledge/status`, returning exactly:
  ```json
  {"trinite": {"cbm": "online|offline", "rag": "online|offline",
               "obsidian": "delegated (native checkpoint_tracker + ODYSSEUS_OBSIDIAN_MCP gate)"}}
  ```
  Do NOT modify this endpoint — other consumers use it. This plan only READS it.
- **Honesty rule:** `cbm` and `rag` are real health checks (`online`/`offline`). `obsidian` is a static descriptive string, NOT a health check — its chip is ALWAYS neutral (`chip-muted`, label "delegated"), never green "online". A failed fetch → muted "indisponible", never a guessed state.
- **Reuse, don't recreate:** `.cockpit-chip`, `.chip-ok`, `.chip-muted` (Sub-projet A) and `.ks-row`, `.ks-name`, `.ks-cat-title` (Sub-projet B) already exist. `ksClean()` already exists in `admin.js` (hardened on `<>&"'`). No new CSS, no new sanitizer.

---

### Task 1: Backend contract test (endpoint unchanged)

Locks the `/status` shape the frontend depends on. The endpoint already exists and passes — this test guards its contract so a future refactor can't silently break the panel.

**Files:**
- Create: `tests/test_knowledge_panel.py`

- [ ] **Step 1: Write the contract test**

Create `tests/test_knowledge_panel.py`:

```python
"""Contract test for the Trinité status endpoint consumed by the knowledge panel."""
from fastapi import FastAPI
from fastapi.testclient import TestClient

from routes.knowledge_routes import router


def _client():
    app = FastAPI()
    app.include_router(router)
    return TestClient(app)


def test_status_returns_trinite_three_legs():
    resp = _client().get("/api/knowledge/status")
    assert resp.status_code == 200
    body = resp.json()
    assert "trinite" in body
    legs = body["trinite"]
    # The three legs the frontend renders.
    for leg in ("cbm", "rag", "obsidian"):
        assert leg in legs, f"missing leg: {leg}"
    # cbm/rag are health strings; obsidian is a descriptive string (never a bare "online").
    assert legs["cbm"] in ("online", "offline")
    assert legs["rag"] in ("online", "offline")
    assert isinstance(legs["obsidian"], str) and legs["obsidian"] != "online"
```

- [ ] **Step 2: Run the test to verify it passes**

Run: `rtk python -m pytest tests/test_knowledge_panel.py -v`
Expected: PASS (1 passed). The endpoint already exists, so this is green immediately — it documents and guards the contract. If it FAILS, stop: the endpoint shape drifted from the spec and the plan must be revisited before touching the frontend.

- [ ] **Step 3: Commit**

```bash
rtk git add tests/test_knowledge_panel.py
rtk git commit -m "test(m6): contract test for Trinité /status endpoint"
```

---

### Task 2: System-panel card (HTML)

**Files:**
- Modify: `static/index.html` (System panel `data-settings-panel="system"`, immediately after the kill-switches `admin-card` closing `</div>` at line 2375, before the Data Backup `<div class="admin-card">` at line 2376)

- [ ] **Step 1: Insert the Trinité card**

In `static/index.html`, locate the end of the kill-switches card — the `</div>` at line 2375 that closes `id="settings-killswitches-card"`, immediately followed by `<div class="admin-card">` (Data Backup) at line 2376. Insert this new card between them:

```html
          <div class="admin-card" id="settings-knowledge-card">
            <h2>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="vertical-align:-2px;margin-right:5px;opacity:0.6">
                <path d="M12 3l9 4.5v9L12 21l-9-4.5v-9L12 3z"/><path d="M12 3v18M3 7.5l9 4.5 9-4.5"/>
              </svg>
              Trinité — Connaissance
            </h2>
            <div class="admin-toggle-sub" style="margin-bottom:8px">Sante des trois jambes de connaissance (lecture seule) : CBM (code), VectorRAG (semantique), Obsidian (memoire).</div>
            <div class="settings-system-logs-controls" style="margin-bottom:8px">
              <button type="button" class="admin-btn-sm" id="knowledge-refresh-btn">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
                Refresh
              </button>
            </div>
            <div id="knowledge-container">
              <div class="settings-system-logs-placeholder">Chargement…</div>
            </div>
          </div>
```

- [ ] **Step 2: Verify the markup is present**

Run: `rtk python -c "import pathlib; h=pathlib.Path('static/index.html').read_text(encoding='utf-8'); print('card', 'settings-knowledge-card' in h); print('container', 'knowledge-container' in h); print('refresh', 'knowledge-refresh-btn' in h)"`
Expected: three `True` lines.

- [ ] **Step 3: Commit**

```bash
rtk git add static/index.html
rtk git commit -m "feat(m6): Trinité knowledge admin-card in System settings panel"
```

---

### Task 3: Frontend render logic (JS)

**Files:**
- Modify: `static/js/admin.js` (add `initKnowledgeView()` immediately after the kill-switches block, before the `INIT & REFRESH` comment banner; wire it into the `inits` array in `initAll()` right after `initKillswitchesView`)

- [ ] **Step 1: Add the render function**

In `static/js/admin.js`, find the `INIT & REFRESH` comment banner (the line `   INIT & REFRESH`). Immediately BEFORE that banner's opening `/* ═══` line, insert:

```javascript
/* ═══════════════════════════════════════════
   TRINITÉ KNOWLEDGE PANEL (read-only)
   ═══════════════════════════════════════════ */
const KNOW_LEG_META = {
  cbm: { name: 'CBM', desc: 'Graphe de code (fonctions/classes/routes), port 9749.' },
  rag: { name: 'VectorRAG', desc: 'Recherche sémantique documentaire (ChromaDB).' },
  obsidian: { name: 'Obsidian', desc: 'Mémoire persistante (notes markdown), déléguée.' },
};

function renderKnowledge(data) {
  const container = el('knowledge-container');
  if (!container) return;
  const legs = data && data.trinite;
  if (!legs || typeof legs !== 'object') {
    container.innerHTML = '<div class="settings-system-logs-placeholder">Indisponible.</div>';
    return;
  }
  let html = '<div class="ks-cat-title">Trinité</div>';
  for (const key of ['cbm', 'rag', 'obsidian']) {
    const meta = KNOW_LEG_META[key];
    const raw = legs[key];
    let chipCls, label;
    if (key === 'obsidian') {
      // Descriptive string, never a health boolean → always neutral.
      chipCls = 'chip-muted';
      label = 'delegated';
    } else if (raw === 'online') {
      chipCls = 'chip-ok';
      label = 'online';
    } else if (raw === 'offline') {
      chipCls = 'chip-muted';
      label = 'offline';
    } else {
      chipCls = 'chip-muted';
      label = '?';
    }
    html += '<div class="ks-row" title="' + ksClean(meta.desc) + '">' +
      '<span class="ks-name">' + ksClean(meta.name) +
      ' <code>' + ksClean(key) + '</code></span>' +
      '<span class="cockpit-chip ' + chipCls + '">' +
      '<span class="chip-val">' + ksClean(label) + '</span></span>' +
      '</div>';
  }
  container.innerHTML = html;
}

function loadKnowledge() {
  const container = el('knowledge-container');
  fetch('/api/knowledge/status', { credentials: 'same-origin' })
    .then(function (r) { return r.ok ? r.json() : null; })
    .then(renderKnowledge)
    .catch(function () {
      if (container) container.innerHTML =
        '<div class="settings-system-logs-placeholder">Indisponible.</div>';
    });
}

function initKnowledgeView() {
  const btn = el('knowledge-refresh-btn');
  if (btn) btn.addEventListener('click', loadKnowledge);
  loadKnowledge();
}

```

- [ ] **Step 2: Wire it into initAll**

In `static/js/admin.js`, find the `inits` array in `initAll()`. It currently reads:

```javascript
    initCalDAV, initBackup, initDangerZone, initTokenForm, initLogsView,
    initKillswitchesView,
    () => settingsModule.initIntegrations()
```

Change the `initKillswitchesView,` line to add `initKnowledgeView`:

```javascript
    initCalDAV, initBackup, initDangerZone, initTokenForm, initLogsView,
    initKillswitchesView, initKnowledgeView,
    () => settingsModule.initIntegrations()
```

- [ ] **Step 3: Verify the JS parses (no syntax error)**

Run: `rtk node --check static/js/admin.js && echo OK || echo "node unavailable — skip"`
Expected: `OK` if node is installed; otherwise `node unavailable — skip` (acceptable — the ES-module is loaded by the browser at runtime, and the change is a localized function addition).

- [ ] **Step 4: Verify the wiring landed**

Run: `rtk python -c "import re,pathlib; js=pathlib.Path('static/js/admin.js').read_text(encoding='utf-8'); print('initfn', 'function initKnowledgeView' in js); m=re.search(r'const inits = \[(.*?)\]', js, re.S); print('wired', 'initKnowledgeView' in (m.group(1) if m else ''))"`
Expected: two `True` lines.

- [ ] **Step 5: Commit**

```bash
rtk git add static/js/admin.js
rtk git commit -m "feat(m6): initKnowledgeView renders Trinité leg health as cockpit chips"
```

---

### Task 4: End-to-end verification

**Files:** none (verification only)

- [ ] **Step 1: Run the new + neighboring test set**

Run: `rtk python -m pytest tests/test_knowledge_panel.py tests/test_killswitch_registry.py tests/test_killswitch_routes.py -q`
Expected: PASS (8 passed total: 1 knowledge contract + 6 registry + 1 route).

- [ ] **Step 2: Boot smoke**

Run: `rtk python -c "import app; print('boot OK')"`
Expected: prints `boot OK` (boots without import error).

- [ ] **Step 3: Live status route response**

Run:
```bash
rtk python -c "
from fastapi import FastAPI
from fastapi.testclient import TestClient
from routes.knowledge_routes import router
a = FastAPI(); a.include_router(router)
r = TestClient(a).get('/api/knowledge/status').json()
legs = r['trinite']
print('legs:', sorted(legs.keys()))
print('obsidian not online:', legs['obsidian'] != 'online')
"
```
Expected: `legs: ['cbm', 'obsidian', 'rag']` and `obsidian not online: True`.

- [ ] **Step 4: Verify served assets contain the wiring**

Run: `rtk python -c "import pathlib; h=pathlib.Path('static/index.html').read_text(encoding='utf-8'); js=pathlib.Path('static/js/admin.js').read_text(encoding='utf-8'); print('card', 'settings-knowledge-card' in h); print('initfn', 'initKnowledgeView' in js); print('reuses_ksClean', 'ksClean(' in js)"`
Expected: three `True` lines (confirms card present, init function present, sanitizer reused).

- [ ] **Step 5: Confirm no backend endpoint was modified (read-only guarantee)**

Run: `rtk git diff 91c23b9 HEAD -- routes/knowledge_routes.py`
Expected: EMPTY output (the knowledge endpoint was not touched by this sub-project). If non-empty, revert the change to `routes/knowledge_routes.py` — the spec forbids modifying it.

- [ ] **Step 6: Final commit (if any verification tweaks were needed)**

```bash
rtk git add -A
rtk git commit -m "test(m6): verify Trinité knowledge panel wiring end-to-end" || echo "nothing to commit"
```

---

## Notes for the implementer

- **Do NOT modify `routes/knowledge_routes.py`.** This iteration reuses the existing `/api/knowledge/status` verbatim. Task 4 Step 5 enforces this.
- **Obsidian is never "online".** It is a delegated leg with no health check; its chip is always `chip-muted` labelled "delegated". This is the honesty rule — rendering it green would fabricate a health state.
- **Reuse, don't recreate:** `.cockpit-chip/.chip-ok/.chip-muted` (A), `.ks-row/.ks-name/.ks-cat-title` (B), and `ksClean()` (B) all already exist. No new CSS file changes, no new sanitizer. If you find yourself adding a `.ks-*` rule, stop — it should already exist from Sub-projet B.
- **No polling.** Like the kill-switches card, the panel fetches once on init and re-fetches on the Refresh button. Trinité health is quasi-static.
