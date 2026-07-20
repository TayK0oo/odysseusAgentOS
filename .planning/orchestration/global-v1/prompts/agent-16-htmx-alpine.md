# Agent A16: HTMX + Alpine.js — Reactive Frontend

## TASK
Integrate HTMX (BSD, 14KB) and Alpine.js (MIT, 15KB) to simplify the vanilla JS frontend — replacing verbose EventSource/fetch/DOM manipulation with declarative HTML attributes while keeping the existing SPA architecture.

## CONTEXT
- Current: 154 JS files (5.69 MB), vanilla JavaScript. SSE streaming via `EventSource` + custom renderer. Forms via `fetch()` + manual DOM updates.
- HTMX: `hx-sse` replaces EventSource handling. `hx-post` replaces fetch+DOM. Zero JS for basic interactions.
- Alpine.js: `x-data`, `x-show`, `x-on:click` — reactivity without a framework.
- Strategy: Keep existing JS. Add HTMX/Alpine alongside. Migrate incrementally. No rewrite.

## REQUIREMENTS

### 1. Add Libraries
Download to `static/lib/`:
- `htmx.min.js` (14 KB)
- `alpine.min.js` (15 KB)

Add to `static/index.html`:
```html
<script src="/static/lib/htmx.min.js"></script>
<script src="/static/lib/alpine.min.js" defer></script>
```

### 2. SSE Chat Stream (HTMX)
Replace `chatStream.js` EventSource with `hx-sse`:
```html
<div hx-sse="connect:/api/chat/stream?session_id=123">
  <div hx-sse="swap:message">{{ content }}</div>
</div>
```

### 3. Forms (HTMX)
Replace manual fetch+DOM:
```html
<form hx-post="/api/auth/login" hx-target="#auth-status" hx-swap="outerHTML">
  <input name="username" />
  <input name="password" type="password" />
  <button type="submit">Login</button>
</form>
```

### 4. Interactive Components (Alpine.js)
Dropdowns, modals, tabs:
```html
<div x-data="{ open: false }">
  <button @click="open = !open">Menu</button>
  <div x-show="open" @click.outside="open = false">...</div>
</div>
```

### 5. Migration Priority
1. Chat SSE stream → HTMX (highest complexity JS → 0 JS)
2. Forms (login, settings, presets) → HTMX
3. Dropdowns, modals, tabs → Alpine.js
4. Complex logic (calendar, email, cookbook) → keep vanilla JS

### 6. Kill-Switch
No kill-switch needed — HTMX/Alpine are JS libraries. If removed, fallback to existing vanilla JS.

## VERIFICATION
- Chat stream works via `hx-sse` (visually identical)
- Login form submits via HTMX
- Dropdown menu opens/closes via Alpine
- All existing functionality preserved
- JS bundle size: 154 files → ~100 files (target: -40%)

## OUTPUT
- `static/lib/htmx.min.js`, `static/lib/alpine.min.js`
- Modified `static/index.html` (add script tags)
- Modified chat, auth, settings HTML/JS (add HTMX/Alpine attributes)
- Size comparison: before vs after JS payload
