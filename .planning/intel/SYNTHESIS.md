# SYNTHESIS — Omega (ingest 2026-10-17)

Source : 36 docs (docs/adr, docs/omega, docs/planning). Voir INGEST-CONFLICTS.md.

## Entrée
- Vision : docs/omega/00-VISION-OMEGA.md (6 piliers, "Everything is a plugin").
- Workstreams : docs/omega/02-WORKSTREAMS.md (W0–W10).
- Mémoire : docs/omega/MEMORY.md (D1→D15, credentials, backlog).
- ADR : docs/adr/* + docs/omega/adr/* (6 ADR).
- Architecture cible : docs/omega/reverse-eng/NOYAU-ULTIME.md (7 règles, Phase A→D).
- Corpus combiné : docs/omega/reverse-eng/OMEGA-COMPLET.md (105 capacités, 32 harnais).

## Locked decisions
D1→D15 (voir MEMORY.md §6), ADR-0001 (Rust), ADR-0002 (UI React/TS), ADR-0003 (plugin/capacités).

## Constraints
Container-first, modèles gratuits par défaut, un écrivain par arbre, déterminisme au système,
provenance [stated]/[observed]/[inferred].

## Next
Phase 10 — NOYAU Phase A (effets réversibles, couches nommées, guard monotone, matrice d'événements).EOF
for f in decisions requirements constraints context; do echo "# ${f^^} — Omega (ingest 2026-10-17)" > .planning/intel/${f}.md; echo "" >> .planning/intel/${f}.md; echo "Voir docs/omega/ et .planning/PROJECT.md, .planning/REQUIREMENTS.md." >> .planning/intel/${f}.md; done
echo "done"
