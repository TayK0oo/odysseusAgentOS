# ADR-0018 — Seuils v1 : latence mémoire

**Date** : 2026-10-23 · **Statut** : Accepté · **Décidé par** : N0 (chef projet)

## Contexte

G2.i.latence utilisait un seuil provisoire de 1000 ms (non gelé). Le plan §9 exige une calibration sur 5 runs baseline puis gel par ADR.

## Mesure

`cargo test -p omega-memory latency_memory_p95_under_threshold` — 5 runs de `recall` sur 1000 souvenirs JSONL. p95 mesuré : **11 ms** (machine de dev, release non optimisé).

## Décision

Seuil gelé : **latence p95 ≤ 50 ms** sur 1000 souvenirs. Marge ×4.5 sur la mesure observée (11 ms). Toute modification du seuil exige un ADR.

## Conséquences

- `gates.toml` G2.i.latence : seuil 50 ms, `bloquant = true`.
- `latency_memory_p95_under_threshold` : assert `p95 < 50`.
