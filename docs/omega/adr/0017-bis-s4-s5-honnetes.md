# ADR-0017-bis — S4 et S5 honnêtes : retrait des stubs et des tautologies

**Date** : 2026-10-23 · **Statut** : Accepté · **Décidé par** : N0 (chef projet)
**Corrige** : ADR-0017 (les scénarios restaient tautologiques après réécriture)

## Contexte

L'audit G2 (2026-10-23) a montré que l'ADR-0017 avait réécrit S4/S5 en
apparence seulement :
- `runner.py` contenait encore des stubs `= True` (`oubli_cascade`,
  `negatif_store_vide`, `pertes_tracees`) et `faits_retrouves = status == "ok"`
  trivial ;
- le canari S4 était codé en dur (`"CANARY42"`) dans le runner, le provider
  scripté et le message utilisateur — preuve circulaire ;
- `provenance_stated` valait `True` sans vérification ;
- S5 ne vérifiait que `status == "ok"`, pas les faits réellement préservés
  par la compaction.

## Décision

1. Suppression de tous les stubs `= True` et des tautologies dans `runner.py`
   (`oubli_cascade`, `negatif_store_vide`, `pertes_tracees`,
   `faits_retrouves = status`).
2. S4 : canari aléatoire généré par le runner et propagé au provider scripté
   (`seq_memory_store_then_recall(canary)`). La preuve exige un
   `memory_recall` réel et réussi, pas un écho du texte de confirmation. La
   provenance est vérifiée dans la trace (`[stated]`).
3. S5 : le faux LLM utilise `seq_long_context_compaction` (faits `ALPHA-7`,
   `BETA-9` plantés début et milieu). `faits_retrouves` exige la présence
   des deux faits dans la réponse ou la trace, pas un `status == "ok"`.
4. G2 redevient ROUGE sincère : `G2.b.s4` et `G2.c.s5` ECHEC jusqu'à
   livraison de B2 (mémoire) et B3 (compaction).

## Conséquences

- `gate.py --phase 2` est ROUGE tant que B2 et B3 ne sont pas livrés.
- Les scénarios échouent si la fonctionnalité est absente (pas de faux vert).
