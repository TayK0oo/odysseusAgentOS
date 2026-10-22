# ADR-0016 — La porte G2 devient exécutable : mort du faux vert (P2)

**Date** : 2026-10-23 · **Statut** : Accepté · **Décidé par** : N0 (chef projet)
**Travail** : P2 (ouverture) · **Ferme** : critères G2 de la porte G2
**Ne renverse pas** : ADR-0013 (G1 exécutable), ADR-0015 (S1 canary).

## Contexte

`omega/gates.toml` définissait G2 par un seul critère `kind="ci"` placeholder
(`« porte non outillée »`), qui ne fait pas échouer la porte (`DELEGUE_CI`).
C'est le même défaut que pour G1 avant ADR-0013 : un vert déclaré sans exécution.

Les sources existent déjà : banc d'acceptation (S3), scénarios YAML, moteur
Rust, tests corruption/concurrence (C2), et les unités C3/C4/C5/C6 à venir.

## Décision

1. Phase 2 de `gates.toml` est réécrite en critères exécutables et mesurables,
   chacun avec sa citation du plan (`source` implicite dans la description),
   tous bloquants sauf le critère de latence (seuil ⚙ non gelé) :
   - `G2.a.s3` — S3 vert scripté, 5/5 runs (déjà vert) ;
   - `G2.b.s4` — S4 vert scripté, 5/5 runs (C3) ;
   - `G2.c.s5` — S5 vert scripté, 5/5 runs (C5) ;
   - `G2.d.a3` — A3 crash matrix réduite (déjà vert) ;
   - `G2.e.a1` — A1 conformité par port sur `MemoryStore` (C3) ;
   - `G2.f.corruption` — corruption journal tronqué ;
   - `G2.g.concurrence` — concurrence 2 runs ;
   - `G2.h.cascade` — oubli en cascade ;
   - `G2.i.latence` — latence mémoire p95 (seuil provisoire 500 ms, `kind="ci"`,
     non bloquant jusqu'au gel par ADR « Seuils v1 »).

2. Seuils ⚙ : la latence mémoire (G2) et le rappel après compaction réel (S5)
   sont **provisoires**. Calibration en 5 runs baseline par C3 (latence) et C5
   (rappel), puis gel par ADR « Seuils v1 » avant que G2 ne puisse passer en VERT.

3. La règle porte/code s'applique : ce commit ne touche que `gates.toml` et cet
   ADR ; aucun code jugé n'est modifié.

## Conséquences

- `gate.py --phase 2` devient ROUGE sincère dès aujourd'hui : S4 et S5 sont
  absents (C3/C5 non livrés), les autres critères sont verts ou à livrer.
- Le critère `G2.i.latence` reste `DELEGUE_CI` (non bloquant) jusqu'au gel.
- Toute modification d'un seuil exige un ADR et un commit séparé.