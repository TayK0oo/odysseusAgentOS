# Phase 1: NOYAU Phase A — squelette - Context

**Gathered:** 2026-10-17
**Status:** Ready for planning
**Mode:** Auto-generated (infrastructure phase)

<domain>
## Phase Boundary

Effets réversibles dans le registre (register → Registration avec dispose()), couches nommées dans la config (ConfigLayerSource + provenance de valeur), guard monotone (PermissionPolicy : composition = intersection, jamais union), matrice d'événements générée + test de cohérence.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion
All implementation choices are at Claude's discretion — pure infrastructure phase. Use ROADMAP phase goal, success criteria, and codebase conventions to guide decisions.

</decisions>

<code_context>
## Existing Code Insights

Reusable assets: omega-registry (Registry/RegistryBuilder), omega-config (loader), omega-ports (PermissionPolicy?), omega-core (events).
Established patterns: Rust workspace crates, tests in-crate.
Integration points: omega-registry register/register_typed/register_dyn, omega-config loader, permission policy in ports.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — infrastructure phase.

</specifics>

<deferred>
## Deferred Ideas

None — infrastructure phase.

</deferred>
