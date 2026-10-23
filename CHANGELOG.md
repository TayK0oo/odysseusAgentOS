# Changelog — Odysseus Agent OS

All notable changes follow [SemVer](https://semver.org/) and
[Keep a Changelog](https://keepachangelog.com/) conventions.

## [v1.1.0] — 2026-10-23

### Added
- `omega-context`: PinnedFacts module for durable facts surviving compaction
  - `PinnedFact` struct with id, content, category, priority, timestamps
  - `PinnedFacts` thread-safe storage with upsert, get, remove, list, list_by_category
  - Integration via `ContextContributor::with_pinned_facts()` registering `Capability::PinnedFacts`
- `omega-context`: LlmCompactionEngine wiring via `ContextContributor::with_llm_compaction(provider)`
  - Wraps `DefaultContextEngine` (layer 1: deterministic truncation) with LLM summary layer
  - Fallback to truncation-only when provider fails (fail-closed)
  - `owns_compaction = true` for LlmCompactionEngine, `false` for DefaultContextEngine
- `omega-types`: `Capability::PinnedFacts` (Slot regime) for registry resolution
- `omega-adapters`: 5 compaction wiring tests verifying LlmCompactionEngine and PinnedFacts integration

### Changed
- `ContextContributor` now optionally registers `PinnedFacts` and `LlmCompactionEngine`
- Default context engine remains `DefaultContextEngine` when no provider provided

## [v1.0.0] — 2026-08-06

### Added
- Architecture Decision Records (`docs/adr/`) for modular design
- Risk register (`docs/governance/risk-register.md`) — operational risk catalog
- RACI matrix (`docs/governance/raci-matrix.md`) — role/responsibility assignments
- SLA template (`docs/governance/sla.md`) — service level agreements
- Runbooks (`docs/operations/runbooks/`) — operational procedures
- RFC template (`docs/governance/rfc-template.md`) — engineering proposals
- User personas (`docs/governance/personas.md`) — target audience definitions
- Post-mortem template (`docs/governance/post-mortem-template.md`) — incident retrospectives
- Deprecation policy (`docs/governance/deprecation-policy.md`) — lifecycle management
- Bus factor analysis (`docs/governance/bus-factor.md`) — key-person risk assessment
- `sfd-visual` package — visual output generation for SFD pipeline
- `explore.md` agent — codebase exploration automation
- Semantic versioning tags — `v1.0.0` tag applied to HEAD
- Provenance column on `memories` table — `stated | observed | inferred` tracking
- P5 — Progressive Disclosure (`src/progressive_disclosure.py`):
  `DisclosureLevel` enum (MINIMAL/STANDARD/FULL), phase+risk gating,
  `get_allowed_tools()` per level, gated behind `ODYSSEUS_PROGRESSIVE_DISCLOSURE`
- P19 — Memory Impact Verification (`src/memory_impact.py`):
  `MemoryImpactVerifier` with `evaluate_impact()`, `should_store()`,
  n-gram Jaccard similarity scoring, gated behind `ODYSSEUS_MEMORY_IMPACT`
- Kill-switch registry entries for P5 and P19 in `src/killswitch_registry.py`
- M6.8 and M6.9 hooks in `archive/legacy/agent_loop.py`

### Changed
- Architecture audit: suppression of 4 dead files
  - `src/zen_router.py` (354 lines)
  - `src/preferences.py` (292 lines)
  - `src/content_security.py` (281 lines)
  - `src/durable_execution.py` (281 lines)

### Fixed
- `sfd-eventbus` plugin restored — `@opencode-ai/plugin` + `@agentos/sfd-eventbus`
  added to `.opencode/package.json`
- Missing npm dependencies installed

### Security
- Triple-checked imports before dead code removal
- Memory provenance column ensures audit trail for all stored facts
