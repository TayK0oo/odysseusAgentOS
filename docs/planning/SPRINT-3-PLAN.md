# Sprint 3 — Plan : consommer les résultats (le backlog de câblage)

> Date : 2026-09-26 · Branche `feat/inventaire-global-v1` · Né de `SPRINT-1B-RECETTE.md`
> Déclencheur : la re-dérivation des 41 statuts (`../traceability/02-PRINCIPES-UC.md`)
> Baseline : **10 ACTIF · 28 PARTIEL · 1 DORMANT · 2 ABSENT** — 4 792 PASS

## 1. Le constat, en une phrase

**Le Palier 0 a allumé 14 modules ; 10 d'entre eux calculent un résultat que personne ne consomme.**

Ce n'est pas un problème d'activation. Activer de nouveaux switchs ne fera plus monter la colonne *Activation* : le verrou est désormais le câblage, pas la configuration.

| Niveau | Maîtrisé | Preuve |
|---|---|---|
| 1 — le module existe | ✅ 22/22 principes | inventaire `00-CODE-INVENTORY.md` |
| 2 — le switch est `on` | ✅ 14/14 P0 | registre, testé par `tests/test_killswitch_registry.py` |
| 3 — le résultat agit | ❌ **10/22** | `02-PRINCIPES-UC.md` §5 |

## 2. Périmètre — 10 items, un geste chacun

Chaque ligne est un geste **borné et vérifiable**. Aucun n'est une refonte. Ordre = impact sur le score d'activation, puis risque.

| # | P/UC | Le résultat est… | Geste | Preuve de fin |
|---|---|---|---|---|
| 1 | **P17** | persisté malgré `PROTECTED` | faire **bloquer** la branche au lieu de `logger.info` (`agent_loop.py:4356-4357`) | un message classé `protected` n'apparaît **pas** dans `data/classification_index.json` |
| 2 | **P20** | résolu puis loggé | injecter `language`/`tone`/`format` dans le prompt (`agent_loop.py:4282-4289`) | la réponse du stub est en français quand `language=fr` |
| 3 | **P22** | pur, décision ignorée | donner un effet de bord à `OutputDecision` (`agent_loop.py:4331-4339`) | une demande de diagramme **écrit un fichier** |
| 4 | **P19** | calculé sur une entrée vide | passer une vraie `hypothetical_response`, relier `should_store` à un store (`agent_loop.py:4431`) | `score > 0` sur un tour réel |
| 5 | **UC-05** | la phase API est écrasée | empêcher `on_round_start` de réimposer `BUILD` (`agent_loop.py:2934` → `phase_tracker.py:60`) | `POST /api/phase/set` + un tour ⇒ la phase **tient** |
| 6 | **P16** | `add_observed` en no-op | créer `topics/agent-output.md` (ou gérer son absence) | `[observed]` et `[inferred]` apparaissent dans `data/memory-fs/` |
| 7 | **P5 + P21** | résumé calculé, outils non restreints | injecter `allowed_tools` dans les schémas d'outils (`progressive_disclosure.py:131` n'est appelé que depuis le module : `:141`, `:146`) | le modèle ne reçoit plus les outils hors phase |
| 8 | **P14 + UC-11** | `create_workflow` a 0 appelant | persister un workflow et résoudre le chemin du vault | un run interrompu puis relancé **reprend** |
| 9 | **UC-12** | traces écrites, jamais lues | exposer une lecture des traces (route d'audit) | `GET /api/audit/traces` rend ce qu'`autoeval` a décidé |
| 10 | **P18** | jeton lu et réémis dans le même appel | faire circuler la version entre deux appels distincts | une écriture concurrente est **rejetée** |

## 3. Ce qui est **exclu**, et pourquoi

- **UC-01** (perte d'ACTIF) — bloqué sur une dépendance externe : `OPENCODE_API_KEY` absente de `.env`. Le projet utilise **OpenCode Zen**. *Escalade Chef : fourniture de la clé, ou décision explicite de travailler sans flux LLM nominal.*
- **UC-06 / UC-07** (ABSENT) — worktree et merge : fonctionnalité entière, pas du câblage. Sprint séparé.
- **P8, P12** — `PLANNING_ENGINE` et `AGENT_CATALOG` sont `off` et hors P0/P1/P2 déjà validés. Remettre un switch `off` à `on` est une **décision de politique**, pas du câblage : *escalade Chef au même titre que FND-4.*
- **UC-10** (DORMANT) — même nature : les 12 switches agents sont `off` par conception (démultiplication coûteuse). Ne pas le casser.
- **`ODYSSEUS_AUTOEVAL_ALLOW_RESET`** — reste `OFF`. Un `git reset --hard` sur un arbre non commité est une perte de travail ; ce n'est pas un item de backlog.

## 4. Discipline de fin de sprint

Pour chaque item, dans cet ordre — sans exception :

1. **test qui échoue d'abord**, sur la preuve de fin ci-dessus (pas sur l'implémentation) ;
2. le geste ;
3. le test au vert ;
4. **mise à jour du statut** dans `02-PRINCIPES-UC.md` avec la nouvelle citation `file:line` ;
5. `python tools/gen_killswitch_tables.py` — la doc ne se fait pas a la main.

> Un item dont la preuve de fin est « le switch est `on` » n'est pas un item terminé. C'est un item du Palier 0.

## 5. Critère de sortie du sprint

Le score n'est pas une cible ; c'est un **indicateur**. Ce qui compte :

- **0 principe** dont l'unique action est un `logger.info` sur un résultat calculé ;
- chaque ligne de `02-PRINCIPES-UC.md` §5 résolue ou explicitement reportée avec sa raison ;
- suite verte, lint ≤ base connue (3 708), `gen_killswitch_tables.py --check` vert.

Si l'*Activation* ne bouge pas après ces 10 items, c'est que le modèle de statut est faux — pas que le travail est incomplet. Le vérifier avant d'ajouter du travail.
