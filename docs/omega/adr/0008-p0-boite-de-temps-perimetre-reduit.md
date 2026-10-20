# ADR-0008 — P0 dépasse sa boîte de temps : périmètre réduit, chemin critique maintenu

**Date** : 2026-10-20 · **Statut** : Accepté · **Décidé par** : chef de projet
**Travail** : P0 (tous) · **Règle** : PLAN-MAITRE §10, règle 11

## Constat (observed)

P0 est une phase `[M]` (2 à 3 sessions). Nous en sommes à la **quatrième**
session, et **G0 n'est pas verte** (2 portes sur 5 : G0.b, G0.c).

La règle 11 du plan est explicite : *« au-delà de ×2 sans porte verte : ADR
(réduire le périmètre ou pivoter), jamais d'extension silencieuse »*. Sans
cet ADR, le plan dérape — et le projet s'est déjà reproché cette dérive
(27 phases de cérémonies, 0 agent capable).

État des 11 travaux de P0 au 2026-10-20 :

| Fait | Travail | Reste |
|---|---|---|
| ✅ | P0-2 `measure.py` + `measures_advanced.py` | P0-5 `gates.toml` + `gate.py` |
| ✅ | P0-3a/3b/3c réconciliation doc-truth (ADR-0007) | P0-6 méta-gardes (stub/orphelins/mutation) |
| ✅ | P0-4 banc d'acceptation (S0 PASS, S1 XFAIL nommé) | P0-7 CI (`gate.py --phase 0`) |
| ⚠️ | P0-1 `verify.py` — écrit, **jamais exécuté complet** jusqu'au 2026-10-20 | P0-8 ledger de veille (105 ressources) |
| ❌ | P0-3d `generate_index.py` | P0-9 spike D0 (moteur de référence) |

## Décision

1. **P0 se poursuit**, mais sur son **chemin critique** uniquement :
   **P0-5 → P0-7 → P0-9 → `verify` complet ×2**.
2. **P0-6** (détecteurs de stub, d'orphelins, baseline de mutation) et
   **P0-8** (ledger de veille) sont **décalés en P1 et P7**, sans perte de
   capacité :
   - le détecteur de **stub** passe en **P1-6** : le handler Python
     `{"status": "executed"}` en est l'occasion naturelle, il sera corrigé à
     cet endroit ;
   - le **ledger de veille** passe en **P7** (veille) : il indexe 105
     ressources, il ne ferme aucune porte ; le plan le place lui-même hors du
     chemin critique de G0.
3. **P0-3d** (`generate_index.py`) reste dans P0 mais **n'est pas bloquant**
   pour G0 : `INDEX.md` et `MEMORY.md` portent déjà une bannière « périmé »,
   donc ils ne mentent pas.
4. **Réservation de numérotation** : cet ADR est le **0008**, le **0009** est
   réservé à D0 (le moteur de référence), la porte G0.d pointe sur
   `docs/omega/adr/0009-*.md`.

## Conséquences

- **G0 reste atteignable** : ses quatre portes rouges sont des **outils** ou
  des **décisions** (gates, CI, D0), pas du volume de code.
- **Le plan ne dérive pas** : la réduction de périmètre est écrite, datée,
  motivée — c'est exactement ce que la règle 11 demande.
- **Rien n'est perdu** : les deux travaux décalés restent listés, avec leur
  destination (P1-6, P7). Aucun n'est abandonné.
- **Le prochain agent sait où aller** : la « prochaine action » doit être
  **calculée** (première porte rouge), pas écrite en dur — c'est le correctif
  `measure.py` v3, qui lit `omega/gates.toml`.

## Alternatives rejetées

- **Prolonger P0 sans toucher au périmètre** — interdit : c'est l'extension
  silencieuse que la règle 11 refuse.
- **Abandonner G0.d (D0)** — impossible : ADR-0006 a rescopé P1-1 de `[L]` à
  `[XL]` précisément parce que le choix du moteur décide **où** la boucle
  s'écrit. D0 conditionne P1.
- **Abandonner G0.e (la CI exécute la porte)** — interdit : le plan note
  (risque R22) qu'« une porte que la CI n'exécute pas est décorative ».
- **Déplacer D0 plus tard pour « aller plus vite vers P1 »** — c'est
  exactement le reproche que le plan adresse aux sprints antérieurs.

## Provenance

- [observed] 2 portes vertes sur 5 (`G0.b`, `G0.c`), `STATUS.json` du
  2026-10-19 : `portes.G0..G7 = ROUGE` pour les gates non outillées.
- [observed] Première exécution complète de `verify.py` : `ECHEC(pytest)`,
  cause et décision dans ADR-0011.
- [inferred] Charge estimée du chemin critique restant : 3 à 4 sessions.
