# ADR-0028 — Redaction `sk-` (OpenAI API keys)

**Date** : 2026-10-31 · **Statut** : ACCEPTED · **Décidé par** : N0 (chef projet)

## Contexte

`omega-types/src/secret_redact.rs` détectait `sk_live_` (Stripe) mais pas `sk-` (OpenAI). Un secret OpenAI (`sk-...`) n'était pas redacté dans les logs/traces.

## Décision

Ajouter `sk-` comme préfixe détecté dans `detect_at`, avec garde-fou sur la longueur (≥ 20 caractères après `sk-`).

## Justification

- Les clés OpenAI (`sk-...`) sont des secrets à protéger.
- Le pattern `sk-` est un préfixe courant (OpenAI, Anthropic, etc.).
- Le garde-fou (≥ 20 chars) évite les faux positifs sur des mots courts.

## Conséquence

- Modification de `omega-types/src/secret_redact.rs` (3 lignes).
- Test discriminant : `redact_sk_openai_key` (omega-types/tests/secret_redact_sk.rs).
- Pas de changement de seuil ADR-0022 (amélioration de couverture, pas de seuil).
