# ADR-0001 — Réécriture complète du moteur en Rust

- **Statut :** accepté
- **Date :** 2026-10-15
- **Provenance :** `[stated]` (choix utilisateur)

## Contexte

Le socle `odysseusAgentOS` est un monolithe Python (1071 fichiers `.py`, couplages forts
autour de `core.database`, shims legacy). La cible « everything is a plugin » exige des
frontières nettes, la performance et la robustesse (sandbox, concurrence, latence de routing).

## Décision

**Réécrire complètement le moteur en Rust**, organisé en workspace de crates autour de
ports/adaptateurs (voir `00-VISION-OMEGA.md` §4). Le socle Python n'est **pas** le moteur
cible : il devient (a) un **catalogue de besoins** et (b) une **source de tests de
comportement** à reproduire.

## Conséquences

- ✅ Performance, sûreté mémoire, concurrence, typage fort, déploiement unique.
- ✅ Frontières de crates = frontières de plugins.
- ⚠️ Coût de migration élevé ; risque de réinventer des comportements déjà réglés.
- ⚠️ Certains adaptateurs (docs, PDF, STT/TTS) peuvent rester des **services** externes
  appelés via port, plutôt que réécrits.

## Alternatives écartées

- **Rust neuf + Python cohabite** : plus sûr, mais laisse une dette de dualité.
- **Rust perf-critique seulement** : ne règle pas le couplage structurel.

## Garde-fous

- On **ne réécrit pas à l'aveugle** : chaque brique Rust naît d'une fiche `RE-*.md` (port,
  contrat, test d'acceptation) et d'un test de parité avec le comportement socle observé.
