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

Crates suivants (Phase 1 complète, à venir) : `omega-protocol`, `omega-core`, `omega-context`,
`omega-tools`, `omega-providers`, `omega-app`, `omega-cli`.

## 3. Modèle

- **Capacité** (`Capability`) : besoin nommé, jamais une marque.
- **Régime** (`Regime`) : `Slot` (1 implé active) ou `Ensemble` (N).
- **Manifeste** (`Manifest`) : `{id, capability, version, priority}`.
- **Registre** : résolution déterministe (priorité desc, puis id asc), `resolve::<T>()`.
- **Quarantaine** : `quarantine(id)` → repli sur l'implémentation suivante.
- **Config** : 4 couches `defaults < user < projet < session`.

## 4. Tests d'acceptation (exigés)

| # | Critère | Statut |
|---|---|---|
| (a) | Exclusivité slot : 2 impls non arbitrées → `SlotConflict` | ✅ |
| (b) | Ensemble : N impls coexistent | ✅ |
| (c) | Isolation d'échec : quarantaine → repli sur la suivante | ✅ |
| (d) | Précédence config : user < projet < session | ✅ |
| (e) | Object-safety : usage des ports via `Arc<dyn Port>` | ✅ |
| (f) | Zéro cycle de dépendances | `[todo]` (test de graphe à ajouter) |

## 5. Commandes

```bash
cd omega
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## 6. Simplifications assumées (Phase 1)

- Le registre **type-efface** les instances (`Arc<dyn Any + Send + Sync>`) et expose
  `resolve::<T>()`. Des sous-registres typés par capacité viendront si le besoin se confirme.
- `Provider::complete` est la forme par défaut ; le **streaming** sera un port dédié (Phase 2).
- Pas encore de chargement dynamique (`.so`/WASM) : l'`omega-plugin-api` pose les contrats.
