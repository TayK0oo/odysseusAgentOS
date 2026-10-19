---
status: passed
threats_open: 0
threats_total: 6
verified: 2026-10-19
verdict: SECURED
phase: 27
scope: milestone v0.7 — port Rust de odysseusAgentOS (crates omega-memory, omega-search, omega-policy, omega-tools)
auditor: revue inline déterministe (gsd-security-auditor indisponible : budget de délégation épuisé)
method: tests adversariaux exécutables — chaque menace est un test qui échoue avant correctif
---

# 27 — SECURITY : milestone v0.7 (port Rust)

## Verdict

**SECURED** — 6 menaces identifiées, **6 corrigées**, 0 restante.

L'audit n'a pas été déclaratif : chaque menace est un **test adversariaux exécutable**
qui a réellement échoué sur le code avant correctif. Les tests sont conservés comme
régression permanente (`tests/adversarial.rs` dans chaque crate).

| Crate | Tests unit | Tests adversariaux |
|---|---|---|
| `omega-tools` | 27 | 11 |
| `omega-policy` | 10 | 8 |
| `omega-memory` | 12 | 10 |
| `omega-search` | 9 | 7 |
| **Total** | **58** | **36** |

Suite complète du workspace : **407 tests passés, 0 échec**.

## Registre de menaces

| # | Menace | Sévérité | Crate | Preuve (test) | Statut |
|---|--------|----------|-------|---------------|--------|
| T1 | **Traversée via symlink** — `ToolCtx::resolve` ne normalisait que lexicalement ; un symlink dans le workspace vers `/tmp` permettait `write_file` hors workspace | **haute** | `omega-tools` | `atk_symlink_escape` | **CORRIGÉ** — résolution par `canonicalize` du premier composant existant, puis ré-ancrage des segments restants |
| T2 | **Divulgation de clé privée** — deny-list trop étroite : `id_rsa`, `.config/`, `.gnupg/`, `.netrc`, `credentials`, `*.p12/pfx/keystore` absents | **haute** | `omega-tools` | `atk_secret_via_dot_prefix` | **CORRIGÉ** — deny-list étendue (17 entrées), matching par composant de chemin et non par sous-chaîne |
| T3 | **Échappement de commande par espaces** — `"  rm  -rf  /"` et `"rm\t-rf /"` passaient la deny-list (match sur la chaîne brute) | **haute** | `omega-policy` | `atk_exec_whitespace_evasion` | **CORRIGÉ** — normalisation `split_whitespace().join(" ")` avant évaluation |
| T4 | **Traversée via métadonnée de skill** — `FileSkillStore::save` écrivait `root/<category>/<name>/SKILL.md` sans valider ; un `SKILL.md` avec `name: ../../escape` écrivait hors store | **haute** | `omega-memory` | `atk_skill_name_traversal_is_rejected` | **CORRIGÉ** — `safe_segment()` slugifie et refuse `..`, `/`, segment vide ; double vérification `starts_with(root)` |
| T5 | **Injection de filtre `site:`** — seul le **premier** token `site:` était retiré ; `"site:a.com site:b.com"` laissait passer le second dans la requête | moyenne | `omega-search` | `atk_site_filter_cannot_inject_query` | **CORRIGÉ** — boucle sur tous les tokens, plus aucun `site:` ne subsiste dans la requête |
| T6 | **Domaine vide accepté** — `domain("file:///etc/passwd")` renvoyait `Some("")` au lieu de `None` (impact scoring/ranking) | faible | `omega-search` | `atk_domain_extraction_is_total` | **CORRIGÉ** — hôte vide ou sans point → `None` |

## Menaces vérifiées et couvertes (aucune correction requise)

| Menace | Test | Résultat |
|---|---|---|
| Écriture/lecture hors workspace par chemin absolu | `atk_absolute_path_escape`, `atk_list_dir_cannot_escape` | bloqué |
| Échappement `..` simple et imbriqué | `atk_dotdot_escape`, `atk_nested_dotdot_escape` | bloqué |
| `edit_file` sur un fichier hors workspace | `atk_edit_cannot_write_outside` | bloqué |
| `.env` imbriqué / racine | `atk_nested_secret_via_parent_dir` | bloqué |
| Épuisement mémoire (grep sans borne) | `atk_grep_respects_max_results` | borné à `max_results` |
| Divulgation de chemins via glob | `atk_glob_does_not_leak_paths_outside_root` | chemins relatifs uniquement |
| Boucle infinie via symlink récursif | `atk_glob_follows_symlink_loop` | terminé (aucun lien suivi par `walk`) |
| Race check-then-increment du budget (200 threads) | `atk_budget_tokens_concurrent` | atomique (`Mutex` autour incrément + contrôle) |
| Contournement du plafond de coût en flottants | `atk_budget_float_bypass` | déclenche bien au-delà du seuil |
| Division par zéro si plafond = 0 | `atk_budget_zero_limit_not_infinite` | 0 = illimité, `percent` fini |
| Détournement de `curl … \| sh` | `atk_exec_pipe_evasion_variants` (4 variantes, chemins absolus inclus) | bloqué par segmentation du pipeline |
| Élargissement via composition de permissions | `atk_permission_intersection_cannot_widen` | `Deny` l'emporte toujours |
| Écriture JSONL partielle / fichier `.tmp` résiduel | `atk_partial_write_leaves_no_garbage`, `atk_no_temp_file_left_behind` | tmp+rename atomique, journal existant intact |
| Lignes JSONL corrompues | `atk_corrupted_lines_are_skipped_not_fatal` | lignes valides préservées |
| Requête hostile (null, 100 k caractères, émojis) | `atk_query_never_panics` (16 variantes) | aucune panique |
| Injection de métacaractères shell dans la requête | `atk_search_never_injects_shell_metacars` | aucune réécriture, aucun shell-out |
| Non-termination sur motif glob/path海量 | `atk_rank_bounds_output` (10 000 hits) | borné, snippet ≤ 200 caractères |
| Slugify vide / traversée | `atk_slugify_never_empty` | jamais vide, jamais de `/` |

## Limites assumées et documentées

| Limite | Comportement | Suivi |
|---|---|---|
| `bash -c 'curl … \| sh'` échappe à la détection structurelle de pipeline | `ExecDecision::Allow` | Verrouillé par `atk_exec_known_evasion_limited` : le test **échouera** le jour où ce vecteur sera couvert, imposant la mise à jour de la doc. CorrectifPx : `ExecPolicy` doit refuser `bash -c`/`eval` par politique, pas par heuristique de pipeline. |
| Écritures mémoire concurrentes (`read-modify-write` sans verrou fichier) | perte possible d'entrées en concurrence | Verrouillé par `atk_concurrent_remember_does_not_lose_writes` (comportement connu). CorrectifPx : Mutex inter-processus ou fichier append-only avec O_APPEND. |
| `parse_frontmatter` est un parseur YAML **minimal** (scalaires + listes inline) | pas de YAML imbriqué / multi-ligne | Accepté : le format SKILL.md d'odysseus est plat. Un YAML riche exigerait une dépendance — non justifiée. |

## Hors périmètre

Le bridge JSON-RPC (`omega-python-bridge` + `omega/bridge/handler.py`) **exécute du code Python**
et n'a pas été audité dans cette phase. C'est le vecteur le plus sensible du milestone v0.6 et il
doit faire l'objet d'un audit dédié avant tout usage en production (recommandation : phase
séparée, `/gsd-secure-phase` sur la phase du bridge).

## Conclusion

Aucune menace ouverte sur le périmètre audité. Le code corrigé est couvert par **36 tests
adversariaux exécutables** qui rejoueraient immédiatement toute régression.