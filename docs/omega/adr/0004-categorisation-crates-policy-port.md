# ADR-0004 — Catégorisation des 22 crates et statut de `Policy`

**Date** : 2026-10-19 · **Statut** : Accepté · **Décidé par** : chef de projet
**Contexte déclencheur** : `check_dep_graph.py` échec — 10 arêtes interdites (P0-1, baser `verify` sur ces faits).

## Contexte

`STATUS.json` mesure 22 crates. `check_dep_graph.py` les classe en
`types/ports/config/protocol/core/registry/plugin-api/router/adapters/plugins/app/app-edge`
et interdit, pour un graphe hexagonal sain, qu'une crate non catégorisée (`other`)
dépende de qui que ce soit. Le premier état mesuré trouvait 10 violations.

## Enquête (observed, pas inferred)

| Crate concernée | Dépendances réelles | Constat |
|---|---|---|
| `omega-policy` | `types`, `ports` | importe **déjà** `omega_ports::{ExecPolicy, PermissionPolicy}` — le trait est un port depuis l'origine |
| `omega-tools` | `types`, `ports`, `policy` | déclare `omega-policy` dans `Cargo.toml` mais **0 référence dans le code** — dépendance morte |
| `omega-config` | `types`, `ports` | implémente le port `ConfigSource` (`lib.rs:184`) — c'est un adaptateur de câblage |
| `omega-search`, `omega-memory`, `omega-python-bridge`, `omega-adapters` | `types`, `ports` | implémentent `MemoryStore`/`SkillStore`/moteurs/pont JSON-RPC/`Default*` |

## Décision

1. **`Policy` est un port — confirmé.** Les traits `ExecPolicy`, `PermissionPolicy`,
   `IntersectionPermissionPolicy` **restent dans `omega-ports`** (ils y sont déjà ;
   rien à extraire). Les impls concrètes (`BudgetEnforcer`, `PathRulePermissionPolicy`,
   `CommandPrefixExecPolicy`) restent dans `omega-policy`, catégorisée **adapters**.
2. **Recatégorisation en `adapters`** (dans `check_dep_graph.py`) :
   `omega-policy`, `omega-search`, `omega-memory`, `omega-python-bridge`,
   `omega-adapters`. Justification commune : chacune implémente des ports face à
   un système extérieur (FS, index de recherche, moteur de mémoire, runtime Python).
3. **La règle `config` est élargie à `{types, ports}`** : `omega-config` implémente
   `ConfigSource`. Une crate de câblage peut référencer des traits ; elle ne peut
   toujours pas dépendre de `core` ni d'`adapters`.
4. **La dépendance morte `omega-tools → omega-policy` est supprimée** — commit séparé
   (`fix:`), conforme à la règle porte/code (un commit fonctionnel ≠ un commit de porte).
5. **Rejeté** : assouplir `adapters → core`, toker `policy` en `ports` (aplatir la
   catégorie), ou réécrire les crates — aucune nécessaire après enquête.

## Alternatives envisagées

- *(extraire un trait `Policy` vers `omega-ports`)* — **superflue** : les traits y
  sont déjà ; l'enquête a évité un refactor inutile (principe L2 : vérifier avant
  d'écrire).
- *(récategoriser `omega-policy` en `ports`)* — rejetée : la crate contient de la
  logique métier concrète (plafonds, préfixes, permissions), pas des traits seuls.

## Conséquences

- `dep_graph` mesuré **vert** après recatégorisation + suppression de l'arête morte
  (`Graphe de dependances valide (22 crates, 0 cycle, 0 arete interdite)`).
- `verify` (P0-1) perd son seul rouge prévisible `dep_graph` ; les rouges
  `ruff/pytest/fmt/clippy/test` restent à mesurer — c'est la baseline P0.
- Un futur adaptateur d'exécution devra implémenter les ports de `omega-ports`,
  pas dépendre des structs de `omega-policy` (contrainte nouvelle, tout gain).
