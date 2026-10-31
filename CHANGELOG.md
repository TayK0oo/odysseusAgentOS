# Changelog

## [1.0.0-rc1] - 2026-10-31

### Added
- W6 contexte : budget, compaction, tool search, skill index, pinned facts (branchés).
- W7 UI : cockpit, plugin explorer, router dashboard, trace viewer, context inspector.
- W8 traçabilité : event-bus avec provenance, replay, export OTel (sinks locaux).
- 22 crates auditées en mutation-testing (v24-v27).
- ADR-0022 à ADR-0028.

### Security
- F1 : redaction `sk-` (OpenAI keys) — ADR-0028.
- F3 : CI cargo-audit.

### Changed
- 804 tests workspace (par crate).
- 0 faille CRITIQUE cohérence, 0 faille CRITIQUE sécurité.
- Provenance partielle (pas propagée à tous les événements — gap documenté).

### Known limitations
- W6 index non peuplés (tool_index=0, skill_index=0, pinned_facts=0) — peuplement reporté v33.
- OTel export local uniquement (pas de crate opentelemetry).
- Tests E2E non exécutés (playwright non installé).
