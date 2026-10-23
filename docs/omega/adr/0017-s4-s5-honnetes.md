# ADR-0017 — S4 et S5 honnêtes : réécriture des faux verts

**Date** : 2026-10-23 · **Statut** : Corrigé par ADR-0017-bis · **Décidé par** : N0 (chef projet)

## Contexte

L'audit G2 (2026-10-23) a montré que :
- `S4.yaml` testait la durabilité (append), pas la mémoire du plan (§2 S4).
- `S5.yaml` était trivial (health + echo), la compaction n'était pas exercée.
- Aucun outil mémoire exposé à l'agent. C4 (injection conditionnelle) absent.

## Décision

1. L'ancien test de durabilité est renommé `S3b` (rien de perdu).
2. `S4.yaml` est réécrit pour tester la mémoire : mot aléatoire planté en
   session A, `kill -9`, session B (nouveau processus, nouvelle conversation)
   retrouve le mot via `/api/run`, entrée taguée `[stated]`, oubli ⇒ 0 trace.
   Négatif : store vide ⇒ B ignore le mot.
3. `S5.yaml` est réécrit pour tester la compaction : conversation > fenêtre,
   faits plantés début et milieu, compaction active, budget respecté, pertes
   tracées. Négatif : compaction off ⇒ dépassement ou perte.
4. G2 repasse en ROUGE sincère (S4 et S5 ECHEC jusqu'à implémentation de
   l'outil mémoire + C4 + C5).

## Conséquences

- `gate.py --phase 2` est ROUGE tant que R1 (outils mémoire), R2 (C4),
  R3 (C5) ne sont pas livrés.
- Les scénarios sont honnêtes : ils échouent si la fonctionnalité est absente.