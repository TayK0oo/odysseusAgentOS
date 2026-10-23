# ADR-0020 — Magasin mémoire unique (S4 vrai)

**Date** : 2026-10-23 · **Statut** : Accepté · **Décidé par** : N0 (chef projet)

## Contexte

Trois magasins mémoire disjoints existaient : (a) outil JSONL `omega-tools` (`.omega/memory.jsonl` dans le workspace jail), (b) `FileMemoryStore` (`event_dir/memory.jsonl`, injection C4), (c) `JsonlMemoryStore` (provenance, BM25, vecteur, TTL, forget — orphelin en prod). Un fait écrit par l'outil n'était jamais lu par l'injection.

## Décision

Un seul magasin : **(c) `JsonlMemoryStore` derrière le port `MemoryStore`**, construit **une fois** dans `omega-server/src/state.rs`, partagé par les outils `memory_store`/`memory_recall`/`memory_forget` **et** l'injection C4. Les stores (a) et (b) sont supprimés ou ramenés à de minces adaptateurs. Provenance `[stated]/[observed]/[inferred]` dans le trait. Pas de `memory_forget` exposé avant cette décision (maintenant exposé).

## Conséquences

- `omega-tools` lit/écrit via le port `MemoryStore`, pas son propre JSONL.
- `FileMemoryStore` (b) supprimé.
- `JsonlMemoryStore` (c) devient le store de production.
- L'injection lit le même store que l'outil → S4 honnête devient possible.
- Un seul chemin de persistance → plus de dualité.