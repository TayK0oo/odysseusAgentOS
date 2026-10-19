---
status: passed
verified: 2026-10-18
covered_digest: v3:sha256:9a4a8d97ba63be7f22e950eac45352d7419c7e3c33d1d7c869976472e7f48158
covered_files:
  - omega/crates/omega-harness/data/apps/refact-ai.json
  - omega/crates/omega-harness/data/apps/nanocoder.json
  - omega/crates/omega-harness/data/apps/sourcegraph-cody.json
---

# Phase 8 verification — Intake vague 3

## Success criteria

1. **4 fiches/decision validees** : refact-ai (10 fct, AGPL-3.0), nanocoder (10 fct), sourcegraph-cody (8 fct) **dépôt résolu** `cody-public-snapshot` (archivé, 3800 etoiles) note + census patche ; **mentat : dépôt supprimé (404 permanent, verifie 2026-10-18 par API + web) — écart documenté**, census note rempli.
2. **0 harnais OSS silencieux** : chaque entrée du census est disséquée ou documentée (écart mentat tracé dans `scripts/harness_census.py` output + JSON).
