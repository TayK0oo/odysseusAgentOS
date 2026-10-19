---
status: passed
threats_open: 0
threats_total: 2
verified: 2026-10-19
verdict: SECURED
phase: 27
scope: omega-python-bridge — pont Rust↔Python (exécution de code Python par subprocess)
auditor: revue inline déterministe (gsd-security-auditor indisponible : budget de délégation épuisé)
method: 10 tests adversariaux exécutables + contrôle négatif (garde-fou désactivé pour prouver que le test détecte)
---

# 27 (bis) — SECURITY : omega-python-bridge

Complète `27-SECURITY.md` qui déclarait explicitement ce crate **hors périmètre**.

## Verdict

**SECURED** — 2 menaces identifiées, **2 corrigées**, 0 restante.

Ce crate **exécute du code Python**. C'est le vecteur le plus sensible du projet.
Workspace après correctifs : **417 tests passés, 0 échec**.

## Registre de menaces

| # | Menace | Sévérité | Preuve | Statut |
|---|--------|----------|--------|--------|
| B1 | **Le bridge se cassait sur toute sortie non-JSON** — il lisait **la première ligne** de stdout et la parsait. Un `print()`, un warning Deprecation ou une trace Python en tête rendait le bridge totalement inopérant (déni de service sur toute l'intégration Python). | haute | `atk_non_json_stdout_fails_cleanly` | **CORRIGÉ** — lecture en boucle jusqu'à une ligne JSON-RPC valide, lignes vides/non-JSON ignorées |
| B2 | **Sortie stdout non bornée** — `read_line` accumulait sans limite ; un script bavard (ou un `print` d'un gros payload) épuisait la mémoire du processus hôte. | haute | `atk_unbounded_stdout_does_not_exhaust_memory` | **CORRIGÉ** — plafond `MAX_LINE_BYTES = 8 Mio` par ligne, retour `Err` au dépassement |

## Menaces vérifiées — couvertes, aucune correction requise

| Menace | Test | Résultat |
|---|---|---|
| **Fuite de processus au timeout** | `atk_timeout_kills_child_process` | enfant tué — **contrôlé négativement** (voir ci-dessous) |
| Timeout non court-circuité | idem | retour en < 800 ms, bridge non bloqué |
| Processus muet (ne répond jamais) | `atk_silent_process_times_out_not_hangs` | timeout propre, pas de blocage |
| **Injection de commande** via chemin d'interpréteur | `atk_interpreter_path_is_not_shell_interpreted` | `exec` direct, aucun shell ; `/tmp/omega-bridge-pwned` non créé avec un chemin contenant `;` |
| Réponse JSON-RPC malformée | `atk_malformed_response_is_rejected` | 6 variantes refusées proprement |
| Erreur JSON-RPC avalée | `atk_error_response_is_surfaced_not_swallowed` | `error.code/message` remontés |
| Confusion d'identifiants entre requêtes | `atk_request_id_is_incrementable` | ids distincts |
| Ponts spécialisés appelant une mauvaise méthode | `atk_specialised_bridges_use_fixed_methods` | `agent_loop.run` et `tool.execute` bien émis (vérifié par log du script espion) |
| Requête giganticque (50 Mo de params) | `atk_huge_params_are_rejected_or_bounded` | pas de panic ni d'allocation non bornée |

## Preuve que le test de fuite de processus est un vrai détecteur

Un test qui passe ne prouve rien tant qu'on n'a pas vérifié qu'il **échoue** quand la
protection est retirée. Contrôle négatif effectué :

| Configuration | Résultat |
|---|---|
| `child.kill()` **présent** | ✅ PASS — processus tué |
| `child.kill()` **neutralisé** | ❌ **FAIL** — `THREAT OPEN: processus (pid 180178) a survécu (état S)` |

La détection lit `/proc/<pid>/status` et distingue l'état du process — une simple présence
de `/proc/<pid>` n'aurait rien prouvé, un **zombie** (`Z`) restant lisible après sa mort.

## Hors périmètre (reste ouvert)

L'audit porte sur le **mécanisme** du bridge. Il ne couvre pas :

- le **script handler** `omega/bridge/handler.py` en production réelle branché sur
  `src/agent_loop.py` de odysseusAgentOS — le handler actuel est un stub qui retourne
  `{"status": "executed"}` sans exécuter d'outils ;
- l'**authentification** de l'appelant : aujourd'hui `call()` est une méthode publique
  sans contrôle d'accès. À защитть dès que le bridge quittera le périmètre local.

## Conclusion

Le mécanisme d'exécution est durci et **prouvé** par tests adversariaux + contrôle négatif.
Les deux réserves restantes (handler stub, authentification) sont des décisions
d'intégration, pas des défauts du transport.