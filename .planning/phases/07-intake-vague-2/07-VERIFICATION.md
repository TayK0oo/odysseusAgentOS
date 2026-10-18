---
status: passed
verified: 2026-10-18
covered_digest: v3:sha256:d6771ec30c19b22854cf563643f1039abcfced4e31790a9b95b7749aee76589c
covered_files:
  - omega/crates/omega-harness/data/apps/superagi.json
  - omega/crates/omega-harness/data/apps/bolt-new.json
  - omega/crates/omega-harness/data/apps/codebuff.json
  - omega/crates/omega-harness/data/apps/claude-engineer.json
  - omega/crates/omega-harness/data/apps/bloop.json
  - omega/crates/omega-harness/data/apps/sweep.json
---

# Phase 7 — VERIFICATION

Critères du ROADMAP :

1. 6 fiches dissections/<id>.md validées par validate_dissections.py — ✅ (0 ref non résolue, script).
2. Chaque fiche propose un port/adaptateur + preuves fichier:ligne — ✅ (JSON source unique, 6/6).
3. Preuves REA attachées — ✅ (intake summaries docs/omega/research/rea/evidence/intake-<id>.summary.json, 6/6).

Fiches : superagi (9 fct), bolt-new (7), codebuff (8), claude-engineer (8), bloop (6), sweep (7). D8=v3:sha256:9a4a8d97ba63be7f22e950eac45352d7419c7e3c33d1d7c869976472e7f48158
cat > .planning/phases/08-intake-vague-3/08-VERIFICATION.md <<EOF
---
status: passed
verified: 2026-10-18
covered_digest: 
covered_files:
  - omega/crates/omega-harness/data/apps/refact-ai.json
  - omega/crates/omega-harness/data/apps/nanocoder.json
  - omega/crates/omega-harness/data/apps/sourcegraph-cody.json
---

# Phase 8 — VERIFICATION

Critères du ROADMAP :

1. Fiches validées + dépôts réels résolus et notés — ✅ : refact-ai (10 fct, AGPL-3.0), nanocoder (10 fct), sourcegraph-cody (8 fct, dépôt résolu cody-public-snapshot archivé 3800★, census patché).
2. 0 harnais OSS silencieux — ✅ : mentat dépôt supprimé (404 permanent vérifié API+curl, 2026-10-18), écart documenté dans census (note + stars=0).
