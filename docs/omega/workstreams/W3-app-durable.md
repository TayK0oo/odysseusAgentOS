# W3 — Moteur & exécution durable (`omega-app`)

> **Workstream :** W3 · **Statut :** 🟢 incrément 1 · **Date :** 2026-10-15
> **Sources :** `OMEGA-BLUEPRINT.md` §2/§4/§7, `00-VISION-OMEGA.md` §7-8.
> **Provenance :** `[observed]` (implémenté) · `[inferred]` · `[todo]`.

---

## 1. Objectif

Assembler l'application (registre + ports câblés) et offrir une **exécution durable** :
un run interrompu **reprend sans ré-exécuter** les opérations déjà complétées.

## 2. Périmètre incrément 1

| Élément | Rôle |
|---|---|
| `omega-types::Event` | Événement de domaine (`name`, `payload`, `durable`) — partagé bus + journal |
| `omega-types::DurableEvent` | Enregistrement de journal (`run_id`, `seq`, `name`, `operation_id`, `payload`) |
| `omega-ports::EventStore` | Port du journal durable (`append -> seq`, `read_run`) |
| `omega-app::store` | `InMemoryEventStore` (tests) + `FileEventStore` (JSONL append-only) |
| `omega-app` | `AppBuilder`/`App` : composition, câblage, `run` durable + reprise |
| `omega-config::loader` | Config multi-sources (defaults + `omega.toml` + env) |

## 3. Contrats clés

- **`EventStore`** (slot exclusif) : `append(event) -> seq` monotone par run ; `read_run(run_id)`.
- **Idempotence de reprise** : une opération est « complétée » **si et seulement si** le journal
  contient un événement `step.completed` portant son `operation_id` (= `Operation::name()`).
  → `App::run(resume=true)` **saute** les opérations déjà complétées.
- **`FileEventStore`** : un fichier JSONL par run ; **tolère une dernière ligne tronquée**
  (crash en cours d'écriture) — ignorée à la lecture, réparée à l'append suivant.
- **Résolution par le registre** : `App` résout `Provider` / `ContextEngine` / `EventStore`
  depuis le registre (`register_dyn`/`resolve_dyn` pour les trait objects) — **jamais en dur**.

## 4. Garanties

1. Un run coupé au step *k* reprend au step *k+1* et se termine (test `crash_after_k_steps_...`).
2. Chaque opération s'exécute **exactement une fois** (idempotence par `operation_id`).
3. La reprise est **déterministe** (reprise répétée → même résultat, 0 ré-exécution).
4. Une capacité absente → erreur **typée** (`NoImplementation`) au build ou au run.
5. La quarantaine s'applique aussi aux trait objects (repli sur l'implémentation suivante).

## 5. Tests d'acceptation

| # | Critère | Statut |
|---|---|---|
| (a) | Composition depuis 2 contributeurs + manifestes → résolution provider/context-engine | ✅ |
| (b) | Run complet avec stubs → journal écrit + câblage effectif | ✅ |
| (c) | Crash après k étapes → reprise sans ré-exécuter, terminaison `Ok` | ✅ |
| (d) | Capacité manquante → erreur typée | ✅ |
| (e) | Reprise déterministe | ✅ |
| (f) | `FileEventStore` : persistance entre instances + tolérance ligne tronquée | ✅ |

## 6. Décisions

- `Event` vit dans `omega-types` (pas dans `omega-core`) pour que le port `EventStore`
  (`omega-ports`) puisse l'utiliser sans dépendre du noyau.
- Le registre a **deux voies** : `register_typed`/`resolve::<T>` (concret, `TypeId` vérifié)
  et `register_dyn`/`resolve_dyn::<T>` (trait objects). `App` utilise la seconde.
- `StoreBackedBus` fait le pont bus → store : chaque événement `durable` est append
  **synchronement** avant l'étape suivante (ordre + durabilité garantis).

## 7. Reste (incrément 2)

- Fournisseur **réel** (HTTP OpenAI-compatible) branché via `omega-providers`.
- `omega-server` (API HTTP/WS pour l'UI) + `omega-protocol` (DTO) — prévu W7.
- Politique de compaction branchée sur le `ContextEngine` (W6).
