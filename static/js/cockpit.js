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
          const ok = j && (j.status === 'ok' || j.status === 'healthy' || j.ok === true);
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
