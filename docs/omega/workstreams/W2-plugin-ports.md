# W2 — Socle « everything is a plugin » (workspace Rust)

> **Workstream :** W2 · **Statut :** 🟢 en cours · **Date :** 2026-10-15
> **Sources :** `ADR-0003`, `OMEGA-BLUEPRINT.md` §2-§3, `FEATURE-MATRIX.md`.
> **Provenance :** `[observed]` (implémenté) · `[inferred]` · `[todo]`.

---

## 1. Objectif

Poser le **socle Rust** qui rend *remplaçable toute brique* : des **ports** (traits),
un **registre de capacités** (deux régimes), des **manifestes** déclaratifs, une
**quarantaine** (l'échec d'un plugin n'arrête pas le noyau), et une **config à couches**.

## 2. Périmètre Phase 1

Crates créés dans cette étape :

| Crate | Rôle | Dépend de |
|---|---|---|
| `omega-types` | Types purs + erreurs de domaine | — |
| `omega-ports` | Traits/ports uniquement | `types` |
| `omega-config` | Config à couches + `precedence()` | `types` |
| `omega-registry` | Registres, slots (slot/ensemble), manifestes, quarantaine | `types` |
| `omega-plugin-api` | Contributeurs, fabriques, chargement de manifestes | `types`, `registry` |
| `omega-core` | Boucle d'agent agnostique : `Operation`, `Inference`, `Effect`, event-bus, annulation | `types`, `ports` |
| `omega-protocol` | Wire types serde (JSON) server→UI, `schema_version` | `types` |

Crates suivants (Phase 1 complète, à venir) : `omega-context`, `omega-tools`, `omega-providers`,
`omega-app`, `omega-cli`.

## 3. Modèle

- **Capacité** (`Capability`) : besoin nommé, jamais une marque.
- **Régime** (`Regime`) : `Slot` (1 implé active) ou `Ensemble` (N).
- **Manifeste** (`Manifest`) : `{id, capability, version, priority}`.
- **Registre** : résolution déterministe (priorité desc, puis id asc), `resolve::<T>()`.
- **Quarantaine** : `quarantine(cap, id)` → repli sur l'implémentation suivante (**scopée par capacité**).
- **Lien type↔capacité** : `register_typed::<T>(manifest, Arc<T>)` mémorise `TypeId::of::<T>()` ; `TypeMismatch` si le type résolu ne correspond pas.
- **Config** : 4 couches `defaults < user < projet < session` (JSON en Phase 1 ; TOML + schéma JSON différés).

## 4. Tests d'acceptation (exigés)

| # | Critère | Statut |
|---|---|---|
| (a) | Exclusivité slot : 2 impls non arbitrées → `SlotConflict` | ✅ |
| (b) | Ensemble : N impls coexistent | ✅ |
| (c) | Isolation d'échec : quarantaine → repli sur la suivante | ✅ |
| (d) | Précédence config : user < projet < session | ✅ |
| (e) | Object-safety : usage des ports via `Arc<dyn Port>` | ✅ |
| (f) | Zéro cycle de dépendances + arêtes interdites | ✅ (`omega/scripts/check_dep_graph.py` + `--self-test` + test négatif) |
| (g) | `omega-core` : ordre des opérations respecté | ✅ |
| (h) | `omega-core` : annulation coopérative → `Cancelled`, pas d'op suivante | ✅ |
| (h') | `omega-core` : annulation **intra-étape** (`tokio::select!` biased) | ✅ |
| (i) | `omega-core` : event-bus journalise chaque étape | ✅ |
| (j) | `omega-core` : échec isolé vs critique | ✅ |
| (k) | `omega-protocol` : round-trip serde des wire types | ✅ |
| (l) | Quarantaine scopée : `quarantine(cap, id)` n'affecte pas un autre `cap` | ✅ |
| (m) | `TypeMismatch` distinct de `NoImplementation` | ✅ |
| (n) | `register_typed` lie type ↔ capacité | ✅ |
| (o) | `PluginFactory` câblé : `install` → `instantiate` → `register_typed` | ✅ |
| (p) | `unregister` / `replace` pour `Slot` | ✅ |
| (q) | Détection de doublon `(cap, id)` à l'enregistrement | ✅ |

## 5. Commandes

```bash
cd omega
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
python3 omega/scripts/check_dep_graph.py --self-test
```

## 6. Simplifications assumées (Phase 1)

- Le registre stocke des instances type-effacees (`Arc<dyn Any + Send + Sync>`) mais
  mémorise le `TypeId` attendu via `register_typed::<T>()`. Des sous-registres typés par
  capacité viendront si le besoin se confirme.
- `Provider::complete` est la forme par défaut ; le **streaming** sera un port dédié (Phase 2).
- Pas encore de chargement dynamique (`.so`/WASM) : l'`omega-plugin-api` pose les contrats.
- `omega-config` est volontairement **JSON-only** en Phase 1 (merge de couches + précédence).
  Le parsing TOML récursif, `deny_unknown_fields` et le générateur de schéma JSON
  (`schemars`) sont différés à la Phase 2 (cf. `OMEGA-BLUEPRINT.md` §2.1).
