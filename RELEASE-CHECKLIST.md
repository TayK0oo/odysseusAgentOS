# Release Checklist v1.0.0-rc1

## Pré-requis
- [x] HEAD commité, working tree propre
- [x] `cargo test --workspace` : 0 failed (804 tests)
- [x] `cargo clippy --workspace --all-targets` : 0 warning (hors engine.rs:248:14)
- [x] `cargo fmt --all -- --check` : OK
- [x] `npm run check` (UI) : OK (tsc --noEmit)
- [x] ADRs 0022-0028 tous ACCEPTED
- [x] W6 index branchés (tool_index, skill_index, pinned_facts) — peuplement reporté v33
- [x] W8 provenance documentée (partielle)
- [x] F1 redact sk- testé
- [x] F3 cargo-audit CI configuré
- [x] 22/22 crates auditées mutation-testing

## Tag
- [x] `git tag -a v1.0.0-rc1 -m "Release candidate 1 — Omega v1.0.0-rc1"`
