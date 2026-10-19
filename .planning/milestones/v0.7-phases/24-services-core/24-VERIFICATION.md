---
status: passed
verified: 2026-10-19
covered_digest: v3:sha256:5fed9fdde95a01cf0d6fbff395daf9996d7bbeba32337f2e2279c0662b7c159e
covered_files:
  - omega/crates/omega-memory/src/lib.rs
  - omega/crates/omega-memory/src/skill.rs
  - omega/crates/omega-search/src/lib.rs
  - omega/crates/omega-policy/src/lib.rs
---

# Phase 24 — VERIFICATION

Critères du ROADMAP :

1. Services core portés en Rust — ✅ 3 crates créés :
   - `omega-memory` (12 tests) : port de `services/memory/` — JsonlMemoryStore (`MemoryStore`) + FileSkillStore (`SkillStore`, SKILL.md frontmatter/sections/sidecar usage).
   - `omega-search` (9 tests) : port de `services/search/` — enhance_query (site:, multi-part, entités), recency_score, rank_results, cache_duration.
   - `omega-policy` (10 tests) : port de `src/budget_enforcer.py` + garde OPA/secret — BudgetEnforcer (tokens/coût/itérations/tool-calls, seuil d'alerte 80 %), PathRulePermissionPolicy (deny .env/.ssh/*.pem, ask .planning/), CommandPrefixExecPolicy (rm -rf /, curl|sh détecté par segmentation de pipeline).
2. Adaptateurs plugin créés — ✅ chaque implémente le trait `omega-ports` correspondant (object-safe vérifié par les tests).
3. Tests passent — ✅ cargo build workspace OK ; 31 nouveaux tests verts ; 45 suites de tests vertes au total.

Notes :
- Écriture atomique JSONL (tmp + rename)忠实 au pattern `core/atomic_io.py`
- Frontmatter YAML minimal : parse/emit + sections When to Use / Procedure / Pitfalls / Verification
- P0剩余 : observabilité (langfuse/otel) traitée par omega-otel existant
