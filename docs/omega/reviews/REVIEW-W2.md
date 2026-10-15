# Revue de code — W2 (socle « everything is a plugin »)

- **Réviseur :** Reviewer SFD (indépendant, adversarial)
- **Date :** 2026-10-15
- **Périmètre :** `omega/crates/omega-types|ports|config|registry|plugin-api|core|protocol` + `omega/scripts/check_dep_graph.py`
- **Preuves exécutées :**
  - `cargo test --workspace` → **25/25 OK** (0 failure)
  - `cargo clippy --workspace --all-targets -- -D warnings` → **EXIT 0**
  - `python3 scripts/check_dep_graph.py` → `Graphe de dépendances valide (7 crates, 0 cycle, 0 arête interdite)` **EXIT 0**

---

## §1 — Verdict global : **GO avec réserves**

Le socle compile, les tests passent, `clippy -D warnings` est propre, et le DIP est
réellement tenu (`omega-core` ne dépend que de `types`+`ports`, aucun `unwrap`/`panic!`
sur un chemin de bibliothèque). L'exclusivité `Slot` et la quarantaine/repli sont
correctement implémentées dans le cas nominal.

Cependant le **garde-fou de graphe est contournable à la main** (faux négatifs démontrés
§3) et son test unitaire ne vérifie aucune détection (victoire prématurée). Par ailleurs
l'interchangeabilité « par manifeste/config sans recompiler » promise par ADR-0003 n'est
**pas** câblée à l'exécution. Ces points ne cassent pas le build mais minent la confiance
dans les garanties architecturales annoncées. À corriger avant de s'appuyer sur ce socle
pour W3/W4.

---

## §2 — Constats par sévérité

### BLOQUANT

**B1. Le garde-fou de graphe produit des faux négatifs (arêtes interdites et cycles invisibles).**
- `omega/scripts/check_dep_graph.py:108` ne lit que les sections `dependencies` et
  `build-dependencies`. `[dev-dependencies]` et `[target.'cfg(...)'.dependencies]` sont
  **ignorées**.
- `omega/scripts/check_dep_graph.py:110-118` ne reconnaît une dépendance interne que si la
  **clé** (nom local) commence par `omega-`. Un renommage `stealth = { package =
  "omega-registry", workspace = true }` échappe entièrement à la détection.
- Preuve : les 4 reproductions du §3 reviennent toutes `Graphe de dépendances valide
  (…, 0 arête interdite)` alors qu'elles contiennent une arête interdite ou un cycle.
- Correctif : parser `dev-dependencies` et `target.*.dependencies` (a minima les signaler
  comme interdites dans le graphe architecturel) ; résoudre la clé `package =` en priorité
  sur la clé locale ; ajouter un test de détection (voir B2).

**B2. Le test du garde-fou n'a qu'une assertion positive — il ne peut pas prouver la détection.**
- `omega/crates/omega-core/src/lib.rs:495-512` se contente de vérifier `status.success()`.
  Un script qui retournerait toujours `0` (ou qui ne lirait plus rien) passerait le test.
- Correctif : ajouter un test négatif qui génère un workspace temporaire avec une arête
  interdite (ex. `core → registry` en `dev-dependencies`) et asserte l'échec (`exit != 0`).

### MAJEUR

**M1. `quarantine(id)` n'est pas scopée par capacité.**
- `omega/crates/omega-registry/src/lib.rs:95-103` : la boucle parcourt **toutes** les
  capacités et quarantaine toute entrée dont `id` correspond. Un même `id` enregistré sous
  deux capacités (`llm-provider` **et** `tool`) est quarantiné dans les deux. Un échec de
  fournisseur peut donc silencieusement neutraliser un outil homonyme.
- Correctif : scoper par `(Capability, id)`, ou mieux `quarantine(cap, id)`.

**M2. `resolve::<T>()` confond « mauvais type » et « aucune implémentation » — `TypeMismatch` est du code mort.**
- `omega/crates/omega-registry/src/lib.rs:119-130` : si `downcast::<T>()` échoue pour toutes
  les entrées actives, on renvoie `NoImplementation`, jamais `TypeMismatch`
  (`omega-types/src/error.rs:28`, variante **jamais construite**). Résultat : enregistrer un
  `Provider` sous `Capability::ContextEngine` puis `resolve::<ContextEngine>` renvoie
  « aucune implémentation » — diagnostic trompeur qui masque une erreur de câblage.
- Correctif : renvoyer `TypeMismatch(cap)` quand une entrée existe mais qu'aucune ne
  `downcast` ; ajouter un test.

**M3. Aucun lien type ↔ capacité à l'enregistrement (registre effectivement non typé).**
- `omega/crates/omega-registry/src/lib.rs:65-92` accepte n'importe quel
  `Arc<dyn Any + Send + Sync>` pour n'importe quel `Manifest.capability`. Le type concret
  n'est validé qu'au `resolve` (et encore, cf. M2). Rien n'empêche un manifeste
  `ContextEngine` porteur d'une instance `Provider`. `TypeMismatch` (dead code) était
  probablement destiné à ça.
- Correctif : soit un `register_typed<T: Port>(cap, Arc<T>)` par capacité, soit une
  vérification au `register`. Documenté en simplification Phase 1 (W2 §6), mais à
  re-serrer car c'est le cœur de la promesse « un contrat, N implémentations ».

**M4. Interchangeabilité par manifeste/config non câblée (ADR-0003 non tenu à l'exécution).**
- `omega/crates/omega-plugin-api/src/lib.rs:37-40` : `PluginFactory` est **déclaré mais
  jamais utilisé** — aucun registre de fabriques, aucun appel à `instantiate`.
- `omega/crates/omega-plugin-api/src/lib.rs:58-75` : `manifests_from_json` ne produit que
  des `Manifest` (descripteurs), **pas** des instances.
- Conséquence : remplacer `ContextEngine` exige d'écrire du Rust (`registry.register(...,
  Arc::new(MyEngine))`) et de **recompiler** au moins le crate d'assemblage. Le noyau
  (`omega-core`) n'est pas à recompiler (DIP OK), mais « remplaçable par manifeste/config »
  (ADR-0003, RULES-OMEGA §1.4) n'est pas implémenté. Atténuation : W2 §6 l'assume
  explicitement (« pas encore de chargement dynamique »). Correctif : lier manifeste →
  fabrique → instance (registre de fabriques par `id`/capacité) en Phase 1.

### MINEUR

**m1. Pas de `remove`/`replace` pour un `Slot` occupé.**
- `omega/crates/omega-registry/src/lib.rs` : seules mutations = `register` et `quarantine`.
  Le commentaire ligne 63 (« retirer/remplacer la précédente explicitement ») décrit un
  `remove` **qui n'existe pas**. Le seul moyen de remplacer un slot est de quarantiner
  l'ancien (laissant une entrée fantôme). Correctif : ajouter `unregister(cap, id)` /
  `replace`.

**m2. Annulation coopérative non intra-étape — écart avec blueprint §4.**
- `omega/crates/omega-core/src/lib.rs:254-259` : `check()` est vérifié **avant** chaque
  étape, jamais **pendant** (`select! biased` du blueprint §4 absent). Une opération longue
  ne peut pas être interrompue en vol. Le critère W2 (h) est satisfait (pas d'étape
  suivante), mais la sémantique « court-circuite réellement » reste grossière.

**m3. `CancellationToken::check` réutilise `PluginFailed` pour une annulation.**
- `omega/crates/omega-core/src/lib.rs:94-103` : renvoie
  `OmegaError::PluginFailed("kernel", "annulation demandee")`. Une annulation n'est pas une
  défaillance de plugin ; il manque une variante `Cancelled`. Nit sémantique, mais le type
  d'erreur traverse potentiellement les frontières.

**m4. `omega-config` est en deçà du blueprint §2.1.**
- Aucun parsing TOML, aucun `schemars`, aucun `deny_unknown_fields` dans tout
  `omega/crates/omega-config/` (grep vide). Le blueprint promettait « merge TOML récursif +
  générateur de schéma JSON ». On n'a qu'un empilement de `serde_json::Value`. Acceptable
  en Phase 1, mais non documenté dans W2 §6 (à expliciter ou à implémenter).

**m5. `LayeredConfig::get` refait le merge complet à chaque appel.**
- `omega/crates/omega-config/src/lib.rs:75-90` : `get` appelle `merged()` qui clone toutes
  les couches et re-trie. O(n log n) par lecture. Sans importance à ce stade, mais à
  memoïser si le nombre de couches/lignes grandit.

### NIT

**n1. Portabilité du garde-fou : `Path.glob` requiert Python ≥ 3.13.**
- `omega/scripts/check_dep_graph.py:96` (`root.glob`) n'existe qu'en 3.13+ alors que
  `tomllib` (ligne 14) est 3.11+. Sur 3.11/3.12 le script (et donc le test B1) lèverait
  `AttributeError`, faisant échouer `cargo test` pour une mauvaise raison. (Environnement
  de revue : Python 3.14.7.)

**n2. Le graphe est indexé par nom de répertoire, pas par `[package] name`.**
- `omega/scripts/check_dep_graph.py:97` (`cargo.parent.name`). Si un crate est renommé via
  `name =` dans son `Cargo.toml`, la clé diverge et les arêtes sont mal classées
  (faux positifs « arête interdite » sur des crates externes nommés `omega-*`, cf. §3).

**n3. Faux positifs potentiels sur des crates externes préfixés `omega-`.**
- `omega/scripts/check_dep_graph.py:110-118` : toute dépendance dont la clé commence par
  `omega-` et est une simple chaîne (`"1"`) est traitée comme interne, puis classée
  `other` (catégorie sans cible autorisée, ligne 33) → « arête interdite » erronée.

**n4. « Immuable en usage » n'est pas garanti par le système de types.**
- `omega/crates/omega-registry/src/lib.rs:160` : `build()` rend un `Registry` dont
  `register`/`quarantine` sont publics (`&mut self`). Le « gel » n'est qu'une convention
  (le builder est consommé), contournable. Bénin mais trompeur.

**n5. Typos/commentaires.** `omega/crates/omega-core/src/lib.rs:148` « Émetteur
asynchche » → « asynchrone ». Commentaires en français avec quelques accents (« chaîne »,
« déjà ») — autorisé par les règles (accents OK en prose), sans impact.

---

## §3 — Cas adversariaux testés (résultats observés)

Tous reproduits dans un workspace temporaire `/tmp/opencode/depguard_repro` sans modifier
le dépôt.

| # | Cas | Résultat observé | Verdict |
|---|---|---|---|
| 1 | `omega-core` dépend de `omega-registry` via `[dev-dependencies]` (arête interdite `core→registry`) | `Graphe de dépendances valide (3 crates, 0 cycle, 0 arête interdite)` — EXIT 0 | **FAUX NÉGATIF** |
| 2 | `omega-core` → `omega-registry` **renommé** `stealth = { package = "omega-registry", workspace = true }` | `valide … 0 arête interdite` — EXIT 0 | **FAUX NÉGATIF** |
| 3 | `omega-core` → `omega-registry` via `[target.'cfg(unix)'.dependencies]` | `valide … 0 arête interdite` — EXIT 0 | **FAUX NÉGATIF** |
| 4 | **Cycle** `omega-types ⇄ omega-core` via `[dev-dependencies]` de `omega-types` | `valide … 0 cycle` — EXIT 0 | **FAUX NÉGATIF (cycle manqué)** |

**Exclusivité `Slot` :** testée par lecture + raisonnement. La garde
(`omega-registry/src/lib.rs:73-77`, `find(|e| !e.quarantined)`) **tient** : impossible
d'obtenir deux implémentations *actives* (non quarantainées) d'une capacité `Slot`. Le
contournement n'existe pas pour l'exclusivité elle-même — les vraies failles sont M1
(scope de quarantaine) et M2/M3 (type ↔ capacité non lié). Reproduction conceptuelle du
contournement de quarantaine :

```text
register(Manifest{id:"x", capability:LlmProvider}, Arc<ProviderX>)
register(Manifest{id:"x", capability:Tool},         Arc<ToolX>)     // même id, autre capacité
quarantine("x")   // => ProviderX ET ToolX neutralisés (M1)
```

**Interchangeabilité :** vérifiée par grep — `PluginFactory::instantiate` n'a **aucun**
site d'appel ; `manifests_from_json` ne fabrique que des descripteurs. Impossible de
remplacer `ContextEngine` par simple manifeste/config sans écrire de Rust et recompiler
l'assemblage (M4).

---

## §4 — Couverture manquante (tests à ajouter)

1. **Garde-fou (négatif)** : générer un workspace avec arête interdite via
   `dev-dependencies` / `target.*.dependencies` / renommage `package=` et asserter
   `exit != 0` (compense B1/B2).
2. **Garde-fou (cycle)** : cycle via `dev-dependencies` détecté comme tel.
3. **Quarantaine scopée** : `quarantine` d'un `id` sur une capacité ne doit **pas**
   neutraliser le même `id` sur une autre capacité (M1).
4. **Type mismatch** : enregistrer `Provider` sous `ContextEngine`, `resolve::<ContextEngine>`
   doit renvoyer `TypeMismatch` (pas `NoImplementation`) (M2).
5. **Slot + repli** : quarantiner le propriétaire d'un `Slot` puis ré-enregistrer un
   successeur → le nouveau est résolu, l'ancien reste inactif (chemin jamais testé).
6. **Annulation intra-étape** (si `select!`/token vérifié dans `run`) : opération longue
   annulée en cours (m2).
7. **Config** : `deny_unknown_fields` / schéma si la promesse blueprint est retenue ;
   sinon documenter la simplification (m4).

---

## §5 — Ce qui est solide

- **DIP tenu** : `omega-core` ne référence que `omega-types`+`omega-ports`
  (`omega-core/Cargo.toml:9-11`), aucun adaptateur importé, aucun `unwrap`/`panic!`/
  `expect` hors modules de test.
- **ISP** : 10 ports fins et spécialisés, object-safe démontré
  (`omega-ports/src/lib.rs:119-124`).
- **Exclusivité `Slot` réellement appliquée** et testée (`slot_rejects_second_owner`).
- **Quarantaine + repli déterministe** sur `Ensemble` testé
  (`failure_is_isolated_and_falls_back`) ; résolution ordonnée priorité desc puis id asc
  (`IndexMap` + tri stable).
- **Boucle d'agent** : ordre respecté, isolation d'échec vs `fail_fast` distinctes et
  testées, event-bus journalise chaque étape (`run.started`/`step.*`/`run.*`).
- **Annulation coopérative** stoppe bien l'étape suivante (test `cancellation_stops_before_next_step`).
- **Config à couches** avec précédence testée indépendamment de l'ordre d'insertion.
- **Protocole** : round-trip serde de chaque DTO + `schema_version` versionné.
- **Qualité outillage** : `clippy -D warnings` propre, tests verts, 25 cas exécutés.

---

## Métriques

- Fichiers audités : 12 (`7 × lib.rs`, `4 × error/types`, `1 × check_dep_graph.py`) + 7 `Cargo.toml`
- Crates : 7 · Tests exécutés : 25 (tous OK)
- Violations SOLID : 0 (DIP/ISP/SRP respectés ; OCP partiel cf. M4)
- Violations TDD : 2 (B2 test non-significatif ; garde-fou sans cas négatif)
- Violations sécurité : 0 (pas de secret, pas d'injection ; surface réseau absente en Phase 1)
- Violations accessibilité : N/A (pas d'UI dans ce périmètre)
- Violations i18n : N/A (pas de strings visibles utilisateur)
- Faux négatifs du garde-fou de graphe : **4/4 reproduits** (§3)
- Code mort : 1 variante (`OmegaError::TypeMismatch`) + 1 trait non utilisé (`PluginFactory`)

---

## §6 — Résolution (2026-10-15)

Tous les constats BLOQUANTS et MAJEURS ont été corrigés dans `omega/**` ; les MINEURS
bon marché (`unregister`/`replace`, typo, `Cancelled`) ont également été traités.
Le garde-fou `--self-test` et le test Rust `dependency_graph_guard_self_test_passes`
garantissent que les faux négatifs ne régresseront pas.

| Constats | Corrections clés |
|---|---|
| B1/B2 | `check_dep_graph.py` parse `dev-dependencies`, `build-dependencies`, `target.*.*` ; résout `package =`, `workspace = true`, `path =` ; indexe par `[package] name` ; `--self-test` avec 4 cas adversariaux |
| M1 | `quarantine(cap, id)` scopée par capacité |
| M2/M3 | `register_typed::<T>()` mémorise `TypeId::of::<T>()` ; `resolve` renvoie `TypeMismatch` si le type ne correspond |
| M4 | `PluginFactory` générique + `install(factory, ctx)` → `instantiate` → `register_typed` ; `FactoryContributor` pour `compose` |
| m1 | `unregister(cap, id)` et `replace`/`replace_typed` |
| m2/m3 | `tokio::select!` biased dans `AgentLoop::run` ; `OmegaError::Cancelled` |
| m4 | simplification documentée dans `W2-plugin-ports.md` §6 (JSON-only Phase 1) |
| n1/n2/n3 | `Path.glob` supprimé, lecture de `[package] name`, externalisation des crates non-membres |
| n5 | typo « asynchche » corrigée |

Commandes de validation : `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `python3 omega/scripts/check_dep_graph.py --self-test`.

---

## §7 — Résolution — re-vérification indépendante (Reviewer, 2026-10-15)

Re-vérification **indépendante** des commits `7853bf4`, `5e9abcc`, `7eeec2e`, `7d06c32`,
`808951b`, `12c58d9` (≠ auto-certification de l'exécuteur au §6). Preuves exécutées :

- `cargo test --workspace` → **33/33 OK** (types 0, ports 2, config 3, registry 10, plugin-api 5, core 8, protocol 5)
- `cargo clippy --workspace --all-targets -- -D warnings` → **EXIT 0**
- `cargo fmt --all --check` → **EXIT 0**
- `python3 omega/scripts/check_dep_graph.py --self-test` → **4/4 cas adversariaux détectés**
- `python3 omega/scripts/check_dep_graph.py` → `Graphe de dépendances valide (7 crates, 0 cycle, 0 arête interdite)`

### Verdict par constat

| Constat | Statut | Preuve |
|---|---|---|
| **B1** (faux négatifs garde-fou) | **CORRIGÉ** | `check_dep_graph.py:188-223` parse `dev-dependencies`/`build-dependencies`/`target.*.*` ; `_resolve_dep_name` résout `package=`, `workspace=true`, `path=` ; `--self-test` 4/4 OK. Mes 4 reproductions §3 échouent désormais (EXIT 1). |
| **B2** (test victoire prématurée) | **CORRIGÉ** | `check_dep_graph.py:325-418` (`self_test()` construit des fixtures invalides et exige EXIT≠0) + test Rust `omega-core/src/lib.rs:574-592` (`dependency_graph_guard_self_test_passes`). |
| **M1** (quarantaine non scopée) | **CORRIGÉ** | `omega-registry/src/lib.rs:164-172` `quarantine(cap, id)` ; test `quarantine_is_scoped_by_capability` (l.409) : quarantiner `x` sous `LlmProvider` ne neutralise pas `x` sous `Tool`. |
| **M2** (TypeMismatch mort) | **CORRIGÉ** | `omega-registry/src/lib.rs:193-213` : `resolve` renvoie `TypeMismatch` quand des entrées actives existent mais d'un autre `TypeId` ; test `type_mismatch_is_reported` (l.434). |
| **M3** (lien type↔capacité) | **PARTIEL** | `register_typed` mémorise `TypeId::of::<T>()` — cela lie le type *d'enregistrement* au type *de résolution* (M2 réglé). **Mais** aucune borne ne lie `T` au *trait* du port de la capacité : `register_typed::<FakeProvider>(Manifest{cap:ContextEngine}, …)` puis `resolve::<FakeProvider>(ContextEngine)` **réussit et renvoie un Provider cohérent**. Le type-erasure Phase 1 (documenté W2 §6) ne garantit pas la conformité trait↔capacité. Suivi Phase 2 (sous-registres typés). |
| **M4** (interchangeabilité) | **CORRIGÉ (Phase 1)** | `omega-plugin-api/src/lib.rs:44-59` : `PluginFactory::Instance` + `install()` → `instantiate` → `register_typed` ; `FactoryContributor` pour `compose`. Test `install_factory_registers_typed_context_engine` (l.185). Le noyau n'est pas à recompiler. Limite assumée : la fabrique reste du Rust compilé (pas de chargement dynamique `.so`/WASM) — W2 §6 l'assume. |
| **m1** (remove/replace) | **CORRIGÉ** | `unregister`/`replace`/`replace_typed` (`omega-registry/src/lib.rs:130-158`) + tests `unregister_removes_entry`, `slot_replace_after_quarantine`. → **mais** cf. N2 (non-atomicité). |
| **m2** (annulation intra-étape) | **CORRIGÉ** | `omega-core/src/lib.rs:289-300` : `tokio::select! { biased; _ = token.cancelled() => … r = fut => r }` ; test `cancellation_interrupts_long_operation` (l.451) interrompt une op de 60 s. |
| **m3** (`Cancelled` manquant) | **CORRIGÉ** | `omega-types/src/error.rs:28` `OmegaError::Cancelled` ; `CancellationToken::check` le renvoie ; branche dédiée `omega-core/src/lib.rs:309-312`. |
| **m4** (config < blueprint) | **CORRIGÉ** | Documenté : `W2-plugin-ports.md:40` et `:82-84` (« JSON-only Phase 1, TOML/schemars différés Phase 2 »). Code inchangé, assumé. |
| **n1** (`Path.glob` 3.13+) | **CORRIGÉ** | `discover_crates` utilise `base.iterdir()` (`check_dep_graph.py:117`), plus de `Path.glob`. Compatible 3.11+. |
| **n2** (clé = nom de dossier) | **CORRIGÉ** | `read_package_name()` (`check_dep_graph.py:97-101`) lit `[package] name`. |
| **n3** (faux positifs crates externes `omega-*`) | **CORRIGÉ** | `_resolve_dep_name` retourne `None` pour toute clé absente de `crates` (l.167/169/182/185). |
| **n4** (immutabilité non typée) | **NON CORRIGÉ** | `Registry` expose toujours `register`/`unregister`/`replace`/`quarantine` publics (`&mut self`) ; le « gel » reste conventionnel. NIT, sans impact. |
| **n5** (typo) | **CORRIGÉ** | `omega-core/src/lib.rs:165` « Émetteur asynchrone ». |

### Nouveaux constats (reproduits)

**N1 — Garde-fou : dépendance `path` vers un crate NON-membre invisible (faux négatif résiduel).**
`_resolve_dep_name` ne reconnaît une cible `path` que si `name in crates`, où `crates` est
construit uniquement depuis `[workspace] members`. Une arête interdite vers un crate présent
dans le dépôt mais **hors `members`** est ignorée. Repro (workspace `/tmp/opencode/depguard_v2`, cas B) :

```toml
# omega-core/Cargo.toml (membre) — cible hors members :
omega-registry = { path = "../hidden/omega-registry" }
# hidden/omega-registry/Cargo.toml  -> name = "omega-registry"  (NON membre)
```

Résultat : `Graphe de dépendances valide (2 crates, 0 cycle, 0 arête interdite)` — **EXIT 0**
alors que `core → registry` est interdit. Sévérité : **MINEUR** (disposition de workspace
inhabituelle, mais c'est exactement le type de faille que le garde-fou doit neutraliser).
Correctif : si un `path` pointe sous la racine du workspace mais que la cible n'est pas un
membre découvert, **refuser/alerter** plutôt que traiter comme externe.

**N2 — `replace`/`replace_typed` non atomiques.**
`replace` fait `unregister(cap, id)` puis `register(...)` (`omega-registry/src/lib.rs:136-146`).
Si `register` échoue (ex. `SlotConflict` parce qu'un autre `id` actif existe, ou `DuplicateId`),
l'entrée `id` a **déjà été retirée** : l'ancienne implémentation est perdue sans être remplacée.
Sévérité : **MINEUR**. Correctif : vérifier les préconditions avant de retirer, ou
register-then-rollback.

**N3 — `register_typed` n'empêche pas un port incohérent (résidu de M3).**
Complément du statut M3 ci-dessus : l'invariant « le type implémente le port de la capacité »
n'est vérifié nulle part. Aucune borne de trait sur `register_typed::<T>` ni sur
`PluginFactory::Instance`. Une fabrique peut déclarer `capability = ContextEngine` et
instancier un type sans rapport ; `resolve` le restitue sans erreur. Documenté Phase 2,
mais à sécuriser (sous-registres typés par capacité, ou borne `T: ContextEnginePort` par mapping).

### Cas adversariaux re-testés (garde-fou)

| Cas | Statut |
|---|---|
| §3-1 dev-dependency interdite | **détecté** (self-test + repro manuel) |
| §3-2 renommage `package=` | **détecté** (self-test) |
| §3-3 `target.*.dependencies` | **détecté** (self-test) |
| §3-4 cycle via dev-deps | **détecté** (self-test) |
| Nouveau A : `[workspace.dependencies]` renommée (`stealth = { package="omega-registry", workspace=true }`) | **détecté** (EXIT 1) |
| Nouveau B : arête vers crate non-membre (path hors members) | **NON détecté** (EXIT 0) → N1 |
| Nouveau C : `optional = true` | **détecté** (EXIT 1, `optional` ne contourne pas) |
| Nouveau D : cycle via crates `other` (`omega-foo ⇄ omega-bar`) | **détecté** (EXIT 1, cycle + arêtes) |

### Verdict mis à jour

**GO** — score **9/10** (précédent : GO avec réserves 7/10).

Les deux BLOQUANTS et les MAJEURS sont réellement résolus (preuves ci-dessus), et les 4
faux négatifs d'origine sont désormais couverts par un auto-test qui ne peut pas régresser
silencieusement. Restent trois suivis **non bloquants** : N1 (garde-fou aveugle aux
dépendances `path` hors `members`), N2 (`replace` non atomique), N3 (résidu M3 : pas de
lien trait↔capacité — assumé Phase 2). Aucun d'eux ne remet en cause la mise en service du
socle pour W3/W4, mais N1 mérite correction avant d'ajouter les crates adaptateurs
(`omega-context`, `omega-providers`, etc.) qui vivront hors `crates/` si `plugins/*` n'est
pas immédiatement membre du workspace.
