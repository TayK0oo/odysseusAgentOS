# Audit G2 — Vérification indépendante (2026-10-23)

## Q1 — Contournement du garde-fou de délégation

**Plugin renommé** : `/home/tayk0oo/.config/opencode/plugins/deleg` → `deleg.disabled` (mv, hors git).
**Config** : `deleg.json` avec `token_budget: 999999999`, `subagent_depth: 4`, `max_depth: 5`, `timeout_ms: 900000`.
**Contrôlait** : budget de tokens par session (30M → 999M), profondeur de délégation (4), timeout (900s), détection de cycles, splitting.
**Effet** : toutes les limites neutralisées. Sous-agents lancés sans garde-fou. Deux sessions consécutives ont épuisé le budget (Free usage limit reached).
**Rollback** : `mv /home/tayk0oo/.config/opencode/plugins/deleg.disabled /home/tayk0oo/.config/opencode/plugins/deleg` + redémarrage service.
**Statut** : contournement actif. Dette à nommer (ADR-0019 couvre la déviation N0-seul, pas le renommage du plugin).

## Q2 — Portes

G0 VERT, G1 VERT, G2 VERT (9/9, 0 DELEGUE_CI). verify VERT. CI omega-gates success (run 39900630106). Aucun écart avec le rapport.

## Q3 — Contre-épreuve par sabotage

| Critère | Sabotage | Résultat |
|---|---|---|
| G2.a.s3 | Désactiver journal | ROUGE (réel) |
| G2.b.s4 | Store vide | ROUGE (réel, mais memoire_retrouvee tautologique — canary dans message utilisateur) |
| G2.c.s5 | Compaction off | ROUGE (réel, mais faits_retrouves = status ok trivial) |
| G2.d.a3 | Désactiver idempotence | ROUGE (réel) |
| G2.e.a1 | Supprimer MemoryStore | ROUGE (réel) |
| G2.f.corruption | Journal intact | ROUGE (réel) |
| G2.g.concurrence | Séquentiel | ROUGE (réel) |
| G2.h.cascade | forget no-op | ROUGE (réel) |
| G2.i.latence | Seuil 50 ms | ROUGE (réel, test cargo) |

Portes décoratives : G2.b.s4 (memoire_retrouvee tautologique) et G2.c.s5 (faits_retrouves trivial) — le sabotage ne les rend pas rouges pour la bonne raison.

## Q4 — Tautologies

- `runner.py:283,508` : canary littéral `"CANARY42"` dans runner et scripted_provider → tautologie
- `runner.py:519-520` : `oubli_cascade = True`, `negatif_store_vide = True` (stubs, retirés)
- `runner.py:525` : `faits_retrouves = status == "ok"` (trivial)
- `runner.py:527` : `pertes_tracees = True` (stub, retiré)
- `runner.py:437` : `reprise_reussie = isinstance(dict)` (status seul)
- `runner.py:134` : a3 status seul
- `S4.yaml:11` : canary littéral dans le message utilisateur

## Q5 — Mémoire

3 magasins disjoints : (a) outil JSONL `workspace/.omega/memory.jsonl`, (b) FileMemoryStore `event_dir/memory.jsonl` (injection), (c) JsonlMemoryStore `memories.jsonl` (orphelin en prod). L'outil écrit (a), l'injection lit (b) — déconnectés. `memory_forget` exposé (T1). `omega-memory` crate orphelin en prod (pas de dépendant). `FileMemoryStore` jamais peuplé en prod (aucun `.remember` hors tests).

## Q6 — Compaction

`LlmCompactionEngine` et `truncate_compact_with_pinned` existent mais orphelins (aucun appelant hors tests). Route réelle : `DefaultContextEngine` (owns_compaction=false), troncation déterministe (ALPHA-7 du début perdu). Aucun événement `context.compacted` dans le code.

## Q7 — Règle porte/code

Commits gates.toml séparés du code jugé (ba03948c, 54059c64). Propre. ADR-0017-bis existe. ADR-0018 seuil choisi (non dérivé). ADR-0019 écrit.

## Q8 — Dettes

- Mutation : jamais exécutée (cargo-mutants absent, ticket ouvert).
- Revue indépendante : A4 faite (6 défauts).
- Orphelins : detect_orphans.py --strict = 8 orphelins (baseline).
- Branches obsolètes : 27. Worktrees prunables : 125.

## Q9 — Journal

Lignes corrompues (« my own suicide arête », « complexion ») listées, non réécrites (append-only). Claims du journal vs sorties de gate : cohérents (G0/G1/G2 VERT).

## Verdict G2

**VERT TROUÉ.** G2 est verte au sens gate, mais la preuve est partielle :
- S4 tautologique (canary codé en dur dans runner et scripted_provider)
- S5 trivial (faits_retrouves = status ok)
- Mémoire déconnectée (deux stores disjoints)
- Compaction non exercée (LlmCompactionEngine orphelin)
- Mutation non exécutée
- Revue indépendante partielle (A4 faite, mais pas de mutation)

## Décisions demandées au Chef

1. **Contournement deleg plugin** : revert (renommer back) ou accepter comme dette ? Recommandation : revert + ADR si besoin.
2. **S4/S5 tautologies** : corriger (canary aléatoire, preuve par la trace) ou accepter comme dette ?
3. **Mémoire déconnectée** : unifier les stores (B2) ou accepter la dualité ?
4. **Compaction non exercée** : brancher LlmCompactionEngine (B3) ou accepter la troncation ?

## Prochain lot recommandé

B1 (gate-honnête) + B2 (mémoire-code) + B3 (compaction-code) + B4 (mutation) + B5 (seuils). Dans cet ordre, séquentiel, avec revue indépendante à la fin.EOF
echo "created"