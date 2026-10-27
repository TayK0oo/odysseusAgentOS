# ADR-0022 — Inventaire des seuils v1 (G2 / G3)

**Date** : 2026-10-27 · **Statut** : Accepté · **Décidé par** : N0 (chef projet)

## Contexte

Les seuils qui conditionnent le passage en VERT de G2/G3 sont dispersés dans `gates.toml` et dans des constantes de code. Aucun document ne dit, pour chacun, s'il vient d'une **mesure**, d'un **choix** ou d'une **norme**. Cet ADR fait l'inventaire et tague la provenance de chaque valeur, sans en changer aucune.

## Inventaire

| Seuil | Valeur | Où | Provenance |
|---|---|---|---|
| Latence mémoire p95 (`G2.i.latence`) | `p95 < 50 ms` — 1000 souvenirs, 5 runs `recall`, gate bloquant | `omega/gates.toml:236`, test `latency_memory_p95_under_threshold` | **choisi** (ADR-0018 ; formule écrite mais inévaluable, ADR-0021) |
| Budget d'injection mémoire | enveloppe ≤ `max_tokens` du moteur, **8000** par défaut ; un fait coûte `ceil(chars/4)` tokens | `omega-context/src/injection.rs`, `engine.rs:assemble_with_injection` | **choix** (défaut de config, aucune calibration) |
| `RECALL_LIMIT` (outil `memory_recall`) | 20 résultats max | `omega/crates/omega-tools/src/memory.rs:24` | **choix** (borne de sortie d'outil) |
| `recall_limit` (injection) | 8 faits candidats avant filtre pertinence/dedup/budget | `omega/crates/omega-context/src/injection.rs:74` | **choix** |
| Budget de contexte | 8000 tokens (`DEFAULT_MAX_TOKENS`), pas de surcharge d'environnement | `omega/crates/omega-server/src/config.rs:20,97` | **choix** (défaut, pas de mesure) |
| Budget d'exécutions d'outils | 8 par run (`DEFAULT_TOOL_BUDGET`), surchargeable `OMEGA_TOOL_BUDGET` | `omega/crates/omega-server/src/config.rs:27,168` | **choix** (garde-fou anti-boucle, pas de mesure) |
| `G3` | **aucun seuil numérique** — critère `kind = "ci"`, `bloquant = false` | `omega/gates.toml:248` | n/a (porte non outillée) |

## Constat

- **Aucun seuil v1 n'est dérivé d'une mesure.** Un seul adossement à une observation existe (latence, ADR-0018 : marge ×4.5 sur 11 ms) et il n'est pas reproductible par formule ; les autres sont des bornes de sûreté.
- **Aucun seuil ne vient d'une norme externe** : rien dans G2/G3 n'est imposé par une RFC, un SLA ou un standard de place. La colonne « norme » de cet inventaire est donc vide, et c'est un fait, pas un oubli.
- G3 n'oppose aucun chiffre : la seule porte de la phase 3 est un placeholder non outillé, donc non bloquant.

## Conséquences

- Toute modification d'une valeur du tableau exige un ADR (règle ADR-0018 étendue à l'ensemble).
- Le chemin pour sortir du « choisi » est unique et déjà écrit : ADR-0021 (latence), puis mêmes formule + mesure pour budget d'injection et budget de contexte si l'on veut les dériver.
- Aucun fichier de code, `gates.toml` ou scénario d'acceptation n'est modifié par cet ADR : il ne fait que nommer l'existant.
