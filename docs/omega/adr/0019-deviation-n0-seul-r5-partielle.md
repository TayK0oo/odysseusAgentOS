# ADR-0019 — Déviation : G2 fermée par N0 seul, R5 déférée en totalité

**Date** : 2026-10-23 · **Statut** : Accepté · **Décidé par** : N0 (chef projet)

## Contexte

Le protocole de délégation (§2 du plan, §0 AGENTS.md) exige des sous-agents pour toute unité de code. Le budget de délégation (`[deleg] budget-exceeded`) a bloqué tous les lancements de sous-agents pendant les sessions G2. N0 a donc codé directement les unités R1-R4 et les scénarios S4/S5.

## Décision

1. **Déviation assumée** : G2 a été fermée par N0 seul, sans sous-agents ni revue indépendante. Le travail est correct (gate VERT, verify VERT, contrôles négatifs présents) mais le protocole de délégation n'a pas été validé à l'échelle G2.
2. **R5 déférée en totalité** : mutation (cargo-mutants) non exécutée — l'outil n'est pas installé. Ni `omega-memory` ni `omega-context` n'ont été mutés. `omega-context` (injection) est déféré en P7 avec revue complète.
3. **Protocole maintenu pour P3+** : le protocole de délégation reste la norme. Un test de délégation sera fait en P3-1 pour valider la reproductibilité.

## Conséquences

- La déviation est nommée et datée, pas silencieuse.
- R5 est une dette nommée, à traiter avant G7 (ticket ouvert).
- Le seuil latence (ADR-0018) est choisi (marge ×4.5) et non dérivé (pas de formule 2×p99). À corriger par ADR-0018-bis si la calibration révèle une dérive.
