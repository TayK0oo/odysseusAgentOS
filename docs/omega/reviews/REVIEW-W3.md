# Revue de code — W3 (assemblage + exécution durable)

> **Auteur :** Reviewer SFD (audit adversarial) · **Date :** 2026-10-15
> **Périmètre :** `omega-app`, `omega-registry`, `omega-ports`, `omega-types`, `omega-config::loader`, `omega-core`, `omega-plugin-api`
> **Commandes exécutées :** `cargo test --workspace` (103 tests OK), `cargo clippy --workspace --all-targets -- -D warnings` (OK), `cargo fmt --all --check` (OK), `python3 scripts/check_dep_graph.py` (10 crates, 0 cycle, 0 arête interdite).

---

## §1 Verdict

**MIS À JOUR (re-vérification indépendante, 2026-10-15) — GO avec réserves. Score 7/10.**

Les 3 BLOQUANTS (B1, B2, B3) et 3 MAJEURS (M1, M2, M3) sont **réellement corrigés** — vérifiés par
de nouveaux assauts exécutés (voir §Résolution) : reprise homonyme, exclusivité de slot cross-voies,
concurrence du `FileEventStore`, diagnostic `TypeMismatch`, tri global d'`active_ids`, et contrat de
câblage des ports.

Il reste **un résidu MAJEUR (N1)** de la même classe de bug (saut silencieux), plus étroit : l'idempotence
est désormais **positionnelle** (`name#index`), donc un **changement de contenu à la même position** entre
deux reprises fait sauter silencieusement l'opération modifiée. S'y ajoutent 3 mineurs (N2 DuplicateId
cross-voie, N3 test de non-régression trop faible, N4 hook `operation_id()` mort dans `App`). Détail en §Résolution.

> Audit initial : **NO-GO, 5/10** (constats §2, repro §3). Correctifs : commits `7713080`, `0e5be01`,
> `f7e184d`, `e910ad3`.

---

## §2 Constats par sévérité

### BLOQUANT

**B1 — Idempotence clé sur `name()` : collision → opération silencieusement sautée.** ✅ **CORRIGÉ**
`omega-app/src/lib.rs:133-138` saute une opération si `completed.contains(op.name())` ;
`completed_operation_ids` (`lib.rs:257-264`) collecte les `operation_id` des événements
`step.completed`. L'identifiant est `op.name()` (`omega-core/src/lib.rs:286-289`).
→ Deux opérations nommées « infer » (le **cas nominal** d'une boucle agent) : si un crash
intervient après la 1re et avant la 2e, la reprise saute **les deux**.
Le champ `RunInput` documente « name() doit être unique » (`lib.rs:170-172`) **sans le
vérifier**.
**Preuve (reproduction exacte, voir §3) :** 2 ops `"dup"`, journal = 1 `step.completed("dup")`
→ reprise `executed=0 skipped=2 trace=[]`.
**Correctif :** refuser les noms dupliqués à `RunInput::new`/`run` (`DuplicateId`) **ou**
rendre la clé d'idempotence positionnelle (`seq`/index) au lieu de `name()`.

**B2 — Exclusivité de slot contournable entre voies `register_typed` et `register_dyn`.** ✅ **CORRIGÉ**
`omega-registry/src/lib.rs:66-69` stocke deux `IndexMap` indépendants ; le contrôle de slot
est fait séparément dans `register_inner` (`lib.rs:89-93`, sur `self.entries`) et dans
`register_dyn` (`lib.rs:157-161`, sur `self.trait_entries`). Aucun contrôle croisé.
→ Un `Capability::EventStore` (régime `Slot`) accepte **deux** propriétaires (un typed + un
dyn), violant « une seule implémentation active par run » (`OMEGA-BLUEPRINT.md` §3).
**Preuve :** `register_typed(EventStore)=ok` **et** `register_dyn(EventStore)=ok`,
`active_ids(EventStore) = ["typed-store","dyn-store"]`.
**Correctif :** unifier le contrôle d'exclusivité sur l'union des deux voies (ou une seule
structure d'entrées).

**B3 — `FileEventStore::append` non synchronisé (read-modify-write).** ✅ **CORRIGÉ**
`omega-app/src/store.rs:85-127` : `append` relit tout le fichier (`fs::read`, ligne 90-97),
recalcule `seq` = nombre de `\n` (lignes 99-102), puis écrit. **Aucun verrou** (contrairement
à `InMemoryEventStore` qui a un `Mutex`, `store.rs:23`). `App` est `Send + Sync` et `run()`
async concurrent → deux runs sur le même `run_id` courent en parallèle.
**Preuve :** 8 threads × 50 appends → 400 événements lus, mais **125 `seq` dupliqués** (la
garantie « `seq` monotone par run » du port `omega-ports/src/lib.rs:134-135` est rompue).
**Correctif :** `Mutex<()>` (ou verrou de fichier) autour de read+append ; idéalement
attribuer `seq` via `O_APPEND` + comptage atomique.

### MAJEUR

**M1 — `resolve_dyn` sous mauvais type → `NoImplementation` (diagnostic trompeur).** ✅ **CORRIGÉ**
`omega-registry/src/lib.rs:178-193` : si `downcast_ref::<Arc<T>>` échoue sur toutes les
entrées actives, on retourne `NoImplementation` au lieu de `TypeMismatch`. La voie typée, elle,
distingue bien (`resolve`, `lib.rs:294-314`).
**Preuve :** `resolve_dyn::<dyn ContextEngine>` sur un `Arc<dyn Provider>` enregistré sous
`ContextEngine` → `NoImplementation(ContextEngine)` ; même scénario en typed →
`TypeMismatch(ContextEngine)`.
**Correctif :** en `resolve_dyn`, si des entrées actives existent mais aucune ne downcast,
retourner `TypeMismatch(cap)`.

**M2 — `active_ids` combine les deux voies mais casse l'ordre de priorité global.** ✅ **CORRIGÉ**
`omega-registry/src/lib.rs:267-286` : ids typed (triés), puis ids trait **concaténés après**.
Une entrée dyn `priority=100` apparaît **après** une entrée typed `priority=0`. La doc
« par ordre de priorité » est donc fausse dès que les deux voies coexistent.
**Correctif :** fusionner puis trier globalement (priorité desc, id asc).

**M3 — Incohérence de câblage typed/dyn : un port enregistré via `PluginFactory` est invisible
pour `App`.** ✅ **CORRIGÉ (documenté)**
`App::event_store()`/`provider()`/`context_engine()` résolvent **uniquement** via `resolve_dyn`
(`omega-app/src/lib.rs:61-82`) ; or `install`/`PluginFactory` enregistrent via `register_typed`
(`omega-plugin-api/src/lib.rs:51-59`). Un contributeur légitime qui installe un `EventStore`
par fabrique sera introuvable (`NoImplementation`) par `App`, alors que `active_ids` le liste.
**Correctif :** soit `App` résout les deux voies, soit documenter/interdire le `register_typed`
pour les capacités résolues en dyn. **Option retenue : documenter** — les capacités en port
(`llm-provider`, `context-engine`, `event-store`) **doivent** être enregistrées via
`register_dyn` (cf. `W3-app-durable.md` §6, règle de câblage des ports). Rendre `App` robuste
aux deux voies exigerait une table de coercition type → trait object (alourdissement non
justifié, YAGNI) : la voie typed reste dédiée aux capacités concrètes `Sized`.

### MINEUR

> **Hors périmètre de cette passe de correction** (mission : BLOQUANTS + MAJEURS). m1–m4
> restent ouverts et seront traités dans une passe dédiée.

**m1 — Collision scalaire/table dans `from_env` (perte de valeur, non-déterminisme).**
`omega-config/src/loader.rs:96-112` : `set_path` fait `return` silencieux si un segment
intermédiaire n'est pas un objet. `from_env` itère `std::env::vars()` **sans tri**
(`loader.rs:58`). `OMEGA_FOO=1` + `OMEGA_FOO__BAR=2` → une des deux valeurs est perdue selon
l'ordre d'itération (voir §3). **Correctif :** trier les clés ; détecter la collision et
lever une erreur typée plutôt que d'écraser/ignorer.

**m2 — Pas de `fsync` : durabilité limitée à un crash process, pas à une coupure machine.**
`omega-app/src/store.rs:122-124` : `write_all` + `flush` (buffer OS), pas `sync_all`. La doc
« durable » (`store.rs:56-60`) est sur-prometteuse. **Correctif :** `sync_all` (ou documenter
la limite) si l'objectif est l'exécution durable face à coupure.

**m3 — `run_id` auto-généré non global, collision inter-process possible.**
`omega-app/src/lib.rs:266-270` : compteur `AtomicU64` process-local. Deux process distincts
sur un `FileEventStore` partagé peuvent générer `run-1` et mélanger leurs journaux.
**Correctif :** id horodaté/UUID, ou exigence d'un `run_id` explicite pour la reprise.

**m4 — `build()` valide `provider`+`context_engine` mais pas `event_store`.**
`omega-app/src/lib.rs:248-252` : `event-store` absent → `build` OK, échec au `run`. Acceptable
selon le doc W3, mais incohérent (capacité requise par `run` non validée tôt).

---

## §3 Cas adversariaux reproduits

Tous via crate scratch externe (chemin `path=` sur les crates du workspace, dépôt non modifié).

| # | Scénario | Résultat observé | Conclusion |
|---|---|---|---|
| 1 | 2 ops nommées `"dup"`, journal = 1 `step.completed("dup")`, `run(resume=true)` | `executed=0 skipped=2 trace=[]` | 2e op **jamais exécutée** mais sautée (perte de travail) |
| 2 | `register_typed` + `register_dyn` sur `Capability::EventStore` (Slot) | les deux OK ; `active_ids=["typed-store","dyn-store"]` | exclusivité de slot contournée |
| 3 | `register_dyn(Arc<dyn Provider>)` sous `ContextEngine`, puis `resolve_dyn::<ContextEngine>` | `NoImplementation(ContextEngine)` | mauvais type non détecté (vs `TypeMismatch` en typed) |
| 4 | `OMEGA_ADV_FOO=1` + `OMEGA_ADV_FOO__BAR=2` → `from_env` | `merged foo = Number(1)` (la table `foo.bar` est perdue) | collision scalaire/table, non-déterministe |
| 5 | `FileEventStore`, 8 threads × 50 appends même `run_id` | 400 lus, **125 `seq` dupliqués** | `seq` non monotone sous concurrence |

Cas corrects (vérifiés, non-bugs) :
- `step.completed` absent mais `step.started` présent → l'opération **est ré-exécutée**
  (at-least-once, sens sûr) : `completed_operation_ids` ne retient que `step.completed`.
- Ligne tronquée finale ignorée puis réparée à l'append suivant
  (`file_store_tolerates_truncated_last_line`, comportement conforme).
- Append **synchronement** avant l'étape suivante : `StoreBackedBus::publish`
  (`omega-app/src/lib.rs:299-315`) écrit dans le store avant de rendre la main au `AgentLoop`.
- Quarantaine **appliquée aux deux voies** (`quarantine` itère `entries` et `trait_entries`,
  `omega-registry/src/lib.rs:249-264`).

---

## §4 Tests manquants

1. **Deux opérations de même `name()`** → doit être refusé (`DuplicateId`) ou rejoué par
   position, jamais sautées silencieusement (couvre B1).
2. **Slot croisé typed/dyn** : `register_typed` puis `register_dyn` sur le même slot → doit
   lever `SlotConflict` (couvre B2).
3. **Concurrence `FileEventStore`** : N threads appendant au même `run_id` → `seq` strictement
   croissant et sans trou (couvre B3).
4. **`resolve_dyn` type erroné** → `TypeMismatch` (pas `NoImplementation`) (couvre M1).
5. **Collision `from_env` scalaire/table** → erreur typée ou merge déterministe (couvre m1).
6. **Fabrique (`install`) puis résolution par `App`** : un `EventStore` installé via
   `register_typed` doit être résolvable par `App::event_store()` (couvre M3).

---

## §5 Points solides

- **Architecture conforme** : `Event` et `DurableEvent` dans `omega-types` (feuille) ;
  `EventStore` (`omega-ports`) ne dépend que de `omega_types`
  (`omega-ports/src/lib.rs:8-12`). Garde-fou `check_dep_graph.py` → 10 crates, 0 cycle,
  0 arête interdite (dont les cas dev-deps/rename/target détectés en `--self-test`).
- **Résolution toujours par le registre** : aucun fournisseur en dur dans `App`
  (`resolve_dyn` partout, `omega-app/src/lib.rs:61-82`) ; quarantaine respectée
  (test `respects_quarantine`).
- **Qualité** : 103 tests verts, clippy `-D warnings` et fmt irréprochables ; SOLID/DRY
  respectés (ports fins, `Contributor`/`PluginFactory`, `RegistryBuilder` gelé).
- **Reprise déterministe** (noms uniques) démontrée par `resume_is_deterministic`.
- **Tolérance au crash d'écriture** du journal correctement pensée et testée.
- **Voie typée robuste** : `TypeMismatch`, `DuplicateId`, `SlotConflict`, `replace` atomique
  (rollback sur échec), quarantaine scopée par capacité — tous testés.

---

---

## §Résolution — re-vérification (indépendante)

Commandes re-exécutées : `cargo test --workspace` (109 tests OK), `cargo clippy --workspace
--all-targets -- -D warnings` (OK), `cargo fmt --all --check` (OK), `python3 scripts/check_dep_graph.py`
(10 crates, 0 cycle, 0 arête interdite). Assauts via crate scratch externe `path=` (dépôt intact).

| Constat | Statut | Preuve (reproduction exécutée) |
|---|---|---|
| **B1** — deux homonymes + crash entre les deux + reprise | **CORRIGÉ** | run1 `Failed exec=2` ; run2 resume `Ok exec=1 skip=1` ; `trace=["dup","dup"]` → chacune exécutée **exactement une fois** |
| **B2** — slot cross-voies (typed→dyn **et** dyn→typed) | **CORRIGÉ** | les deux ordres → `SlotConflict(EventStore, _)` ; `Ensemble` → les deux coexistent (`active=["x","y"]`) |
| **B3** — `FileEventStore` concurrent (8 threads × 50) | **CORRIGÉ** | 400 lus, **0 `seq` dupliqué**, `seq == 0..399` (unique + monotone) |
| **M1** — `resolve_dyn` type présent mais mauvais / absent | **CORRIGÉ** | mauvais type → `TypeMismatch(ContextEngine)` ; absent → `NoImplementation(EventStore)` |
| **M2** — `active_ids` tri global typed+dyn | **CORRIGÉ** | `["hi"(prio 100, dyn), "mid"(5, typed), "lo"(0, typed)]` |
| **M3** — contrat « ports = `register_dyn` » | **CORRIGÉ (documenté)** | règle explicite dans `W3-app-durable.md` §6 |

### Nouveaux constats

**N1 — MAJEUR. Idempotence positionnelle → saut silencieux si le contenu change à même position.**
`omega-app/src/lib.rs:134-143` : l'id est `format!("{}#{}", op.name(), index)` ; il ne capture **pas le
contenu** de l'opération. Un run complète l'op à l'index *k* (`name#k` au journal) ; une reprise réutilisant
le même `run_id` avec une opération **de même nom, à la même position, mais de contenu différent** est
**sautée sans s'exécuter**.
**Preuve (repro exécutée) :** run1 `[a,b]` complet ; reprise `[a'(autre contenu), b]` → `executed=0 skipped=2
trace=[]` — `a'` jamais exécutée. Atteignable via l'API publique : `assemble_operation`/`infer_operation`
ont des noms **fixes** (`assemble`/`infer`) ; un « retry avec tweak » (nouveau prompt/modèle, même `run_id`,
`resume=true`) saute l'inférence modifiée.
**Correctif :** inclure un condensat du contenu dans la clé, ou refuser la reprise quand le plan diffère, ou
documenter explicitement l'invariante « plan identique entre deux reprises ».

**N2 — MINEUR. `DuplicateId` non appliqué cross-voie sur `Ensemble`.**
`omega-registry/src/lib.rs:104-110` (typed) et `177-183` (dyn) vérifient l'unicité d'id **dans leur propre
liste** ; `active_owner` ne protège que les `Slot`. `register_typed(id="dup")` + `register_dyn(id="dup")` sur
`LlmProvider` (Ensemble) → les deux OK, `active_ids=["dup","dup"]`.
**Correctif :** vérifier l'unicité d'id sur l'union des deux voies.

**N3 — MINEUR (TDD). Le test de non-régression B1 est trop faible (victoire prématurée).**
`omega-app/src/lib.rs:857-895` (`duplicate_operation_names_each_run_once_and_resume`) exécute les deux
homonymes **jusqu'au bout** avant la reprise : il **passe aussi sur l'ancien code buggé** (le saut ne se
produit que si le crash survient *entre* les deux). Le vrai test de régression est le crash entre homonymes
(couvert par la repro B1 de §Résolution, qui passe). **Correctif :** ajouter le cas « crash après le 1er
homonyme, le 2e jamais exécuté ».

**N4 — MINEUR. `Operation::operation_id()` (ajouté en `7713080`) est mort dans le chemin `App`.**
`TaggedOperation` (`omega-app/src/lib.rs:349-367`) surcharge `name()` (→ `name#index`) mais pas
`operation_id()` ; `App::run` calcule l'id à partir de `op.name()` **avant** l'enveloppement. Le hook
`operation_id()` du trait n'est donc jamais consulté par `App` (deux mécanismes pour une même clé, l'un
redondant). Cosmétique : documenter ou supprimer l'un des deux.

### Verdict mis à jour

**GO avec réserves — 7/10.** Les bloquants sont levés avec preuves exécutées. N1 est un résidu de la même
classe (saut silencieux) mais exige un changement de plan entre deux reprises (précondition plus étroite).
À traiter en incrément 2 : clé enrichie du contenu, ou garde « plan identique » avant reprise.

## Métriques

- Fichiers audités : 9 (`omega-app/{lib,store}.rs`, `omega-ports/lib.rs`,
  `omega-registry/lib.rs`, `omega-types/{lib,error}.rs`, `omega-config/{lib,loader}.rs`,
  `omega-core/lib.rs`, `omega-plugin-api/lib.rs`).
- Violations SOLID/encapsulation : 2 (B2 slot croisé, M3 câblage deux-voies).
- Violations TDD (tests manquants) : 6 (§4).
- Violations sécurité (OWASP) : 0 critique (pas de secrets hardcodés ; le parsing env→JSON
  est une surface de confusion, pas d'injection exploitable dans ce périmètre).
- Violations accessibilité : 0 (hors périmètre, pas d'UI dans l'incrément 1).
- Violations i18n : 0 (pas de chaîne visible utilisateur ; erreurs de domaine FR pour logs).
