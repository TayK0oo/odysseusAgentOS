# Agent A5: Tailwind CSS — Design System Migration

## TASK
Replace the monolithic 37,499-line `static/style.css` (1.22 MB) with a Tailwind CSS JIT-generated stylesheet targeting <200 KB while preserving the existing dark theme design.

## CONTEXT
- Current: Single `static/style.css` = 37,499 lines, 1.22 MB. No design system, ad-hoc classes.
- Target: <200 KB with Tailwind JIT (only generates used classes)
- Must preserve: dark theme default, Fira Code monospace, existing component styles
- Approach: Incremental — Tailwind coexists with existing CSS during transition

## REQUIREMENTS

### 1. Setup
- Install Tailwind CSS standalone CLI (no Node.js required at runtime)
- Create `tailwind.config.js`:
```js
module.exports = {
  content: ["./static/**/*.{html,js}"],
  darkMode: 'class',
  theme: {
    extend: {
      colors: { /* Odysseus palette */ },
      fontFamily: { mono: ['Fira Code', 'monospace'] }
    }
  }
}
```

### 2. CSS Refactor
- Create `static/tailwind.css` with `@tailwind base/components/utilities`
- Add `@apply` for existing component classes (buttons, cards, inputs, modals)
- Build: `npx tailwindcss -i static/tailwind.css -o static/style.min.css --minify`
- Target: <200 KB (current: 1,220 KB)

### 3. Migration Strategy
Phase 1: Add Tailwind output alongside existing CSS (both loaded)
Phase 2: Port 20% of components to Tailwind classes
Phase 3: Port 50%, start removing legacy CSS
Phase 4: Port 80%, legacy CSS optional
Phase 5: Full Tailwind, legacy CSS removed

Start with Phase 1 only in this agent task.

### 4. Package.json
```json
{
  "scripts": {
    "css:build": "npx tailwindcss -i static/tailwind.css -o static/style.min.css --minify",
    "css:watch": "npx tailwindcss -i static/tailwind.css -o static/style.min.css --watch"
  }
}
```

### 5. Kill-Switch
No kill-switch needed — CSS is static. Legacy CSS stays as fallback.

## VERIFICATION
- `npm run css:build` generates `style.min.css` < 200 KB
- Visual check: dark theme intact, components look identical
- Existing CSS still loaded as fallback

## OUTPUT
- `tailwind.config.js`
- `static/tailwind.css`
- `static/style.min.css` (generated)
- Size comparison: before vs after
