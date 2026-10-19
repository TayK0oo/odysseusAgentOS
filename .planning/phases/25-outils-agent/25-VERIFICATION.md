---
status: passed
verified: 2026-10-19
covered_digest: v3:sha256:a973195156db6c85c4d71cac7acb293c1c653fbd202fb9a4123b3175540ce3d3
covered_files:
  - omega/crates/omega-tools/src/lib.rs
  - omega/crates/omega-tools/src/diff.rs
  - omega/crates/omega-tools/src/glob.rs
  - omega/crates/omega-tools/src/grep.rs
  - omega/crates/omega-tools/src/workspace.rs
  - omega/crates/omega-tools/src/registry.rs
---

# Phase 25 — VERIFICATION

Critères du ROADMAP :

1. Outils d'agent portés en Rust — ✅ crate `omega-tools` créé : port de `src/agent_tools/filesystem_tools.py` (536 lignes Python) en Rust pur.
   - `read_file` / `write_file` / `edit_file` (ancre exacte, refus si ambiguë) / `ls` (répertoires d'abord) / `glob` (** , *, ?) / `grep` (regex + filtre glob) / `get_workspace` (stats par extension + 10 plus gros fichiers).
   - Module `diff` : unifié LCS avec bornes `MAX_DIFF_LINES` + troncature annotée (port de `_unified_diff`).
   - Module `registry` : 7 schémas JSON (FUNCTION_TOOL_SCHEMAS) + `dispatch(tool, args)` + `spec(tool)` implémentant le port `ToolSurface`.
   - Garde-fous : résolution lexicale + refus de sortie du workspace + deny-list secrets (.env, .ssh, *.pem, *.key).
2. Adaptateurs plugin créés — ✅ chaque outil est une fonction pure derrière `ToolCtx` ; `registry::spec` produit un `ToolSpec` (omega-types) conforme au port.
3. Tests passent — ✅ 27 tests verts dans `omega-tools` ; 0 échec sur l'ensemble du workspace.

Note : le remaining `admin_tools` (1 012 l.), `document_tools` (670 l.), `session_tools` (517 l.) restent P3/P2 — non portés, documentés dans AUDIT-ODYSSEUS-PORTAGE.md.
