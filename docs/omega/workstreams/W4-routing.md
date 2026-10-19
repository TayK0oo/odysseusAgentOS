# W4 — Routing (routeur de modèles)

> **Workstream :** W4 · **Increment :** 1 (logique pure, zéro réseau) + 3 (circuit breaker à états, M4)
> **Date :** 2026-10-15 · **Branche :** `feat/agentos-omega` (+ `feat/omega-w4-cb` pour l'inc3)

## Objectif de l'incrément

Livrer le **noyau déterministe du routeur** : classification de tache, scoring composite,
sélection Pareto, fallback + circuit breaker, et cache de décision — sans aucun appel
réseau ni LLM. Les données de catalogue sont en dur dans les tests.

## Livrables

1. **`docs/omega/workstreams/W4-routing.md`** (ce document) — spec et décisions.
2. **`omega/crates/omega-providers/`** — types catalogue (`ModelRecord`, `QualityAxes`,
   `Capabilities`, `ModelTier`).
3. **`omega/crates/omega-router/`** — logique de routing (`TaskClassifier`,
    `ScoringPolicy`, `Selector`, `FallbackPolicy`, `DecisionCache`) et tests.

## Décisions d'architecture

### Où placer les types

- `ModelRecord`, `QualityAxes` et `ModelTier` sont des **types purs** ; leur source
  unique est désormais **`omega-types`** (déplacement depuis `omega-providers`
  lors de la correction de REVIEW-W4 M3). `omega-providers` les ré-exporte pour
  compatibilité.
- `TaskType` T1..T14, `RoutingRequest`, `Attachment`/`AttachmentKind`, `Capabilities`,
  `RouterDecision`, `RouterError` et `Weights` sont des **types purs** ; ils vivent
  dans **`omega-types`** (source unique, aucune dépendance interne).
- Les ports `TaskClassifier`, `ScoringPolicy`, `Selector` et `FallbackPolicy` sont
  des **traits** ; ils vivent dans **`omega-ports`** (`OMEGA-BLUEPRINT.md` §3,
  capacités `task-classifier` et `router-policy`).
- `omega-router` implémente ces ports (`HeuristicTaskClassifier`,
  `CompositeScoringPolicy`, `ParetoSelector`, `TierFallbackPolicy`) et consomme
  les types depuis `omega-types`, conformément au graphe de dépendances
  `router → {types, ports, providers}`.
- Le trait `RouterPolicy` orphelin et incohérent (`async fn choose(...) -> ModelId`)
  a été supprimé de `omega-ports` ; il est remplacé par les trois ports fins
  `ScoringPolicy` + `Selector` + `FallbackPolicy` (ISP).

### Taxonomie T1..T14

Mapping conforme à `MODEL-ROUTING-BENCHMARKS.md` §3 :

| Code | Variante `omega-router` | Signal dominant |
|---|---|---|
| T1 | `TrivialCompletion` | texte court (< 8 mots), salutations |
| T2 | `Summarization` | « résume », « liste », « convertis » |
| T3 | `TargetedEdit` | patch, corrige, 1 fichier |
| T4 | `Refactor` | refactorise, plusieurs fichiers |
| T5 | `Implementation` | crée, implémente, scaffolding |
| T6 | `LongReasoning` | conçoit, architecte, stratégie, long |
| T7 | `Math` | formules, preuves, maths/sciences |
| T8 | `Research` | cherche, compare, sources |
| T9 | `AgenticTerminal` | shell, scripts, CI, terminal |
| T10 | `Vision` | image en entrée |
| T11 | `Audio` | fichier audio |
| T12 | `Embedding` | indexation, similarité, embedding |
| T13 | `MediaGeneration` | génère image/vidéo/audio/TTS |
| T14 | `Chat` | défaut, conversation généraliste |

### Priorité du classifieur (REVUE-W4 B2 + N1)

L'ordre de matching est explicite et testé :

1. **Modalités d'entrée** (pièces jointes image/audio/video) — priorité absolue.
2. **Vision implicite** : analyse/description/explication d'une image/photo.
3. **Génération média** : (verbe de génération média **ET** nom média) **OU**
   verbe de dessin intrinsèquement visuel (`dessine`, `esquisse`, `croquis`).
4. **Intentions fortes** : raisonnement long (plan, stratégie, architecture,
   concevoir, analyser, raisonnement, design) > mathématiques > recherche.
5. **Transformation / édition** : implementation > refactor > édition ciblee >
   résumé/reformatage.
6. **Capacités spécialisées** : embedding, agentique.
7. **Longueur** : > 40 mots → raisonnement long ; < 8 mots → trivial.
8. **Défaut** : conversation généraliste (`Chat`).

Cette priorité garantit les cas adversariaux de la revue :
- `"genere un plan d'architecture"` → `LongReasoning` (verbe `genere` sans nom média :
  pas de génération média ; intention forte avant media).
- `"genere une image d'un plan d'architecture"` → `MediaGeneration` (verbe `genere`
  **et** nom `image` : la génération média prime sur le sujet « plan/architecture »).
- `"dessine le plan de la maison"` → `MediaGeneration` (verbe de dessin : visuel
  intrinsèque).
- `"analyse cette image"` → `Vision` (vision implicite avant reasoning).
- `"fix the build"` → `TargetedEdit` (édition avant implémentation).

### Normalisation Unicode (REVUE-W4 B1)

Le texte est normalisé en **NFD + suppression des marques diacritiques** avant
matching. Cela permet de matcher les formes accentuées du français (`Résume`,
`équation`, `génère`...) avec les motifs non accentués du classifieur.

La dépendance `unicode-normalization` est ajoutée au `Cargo.toml` de
`omega-router` uniquement ; `omega-types` reste sans dépendance Unicode.

### Score composite

```text
Score(t,m) = Σ_axes w_axe(t) · axe_norm(m)
           + w_speed(t) · speed_norm(m)
           + w_cost(t)  · cost_norm(m)
           + w_context(t) · context_norm(t,m)
```

- `axe_norm` : valeur 0..100 ramenée à [0,1].
- `speed_norm` : log-normalisation dans l'ensemble des candidats.
- `cost_norm` : log-normalisation **inverse** dans l'ensemble des candidats
  (coût faible → 1).
- `context_norm` : `min(ctx(m) / ctx_requis, 1)`.
- Les poids `w_*(t)` sont par `TaskType` et somment à 1.

Profils de poids retenus :

| Profil | reasoning | code | agentic | knowledge | chat | vision | speed | cost | context |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| T1 | 0 | 0 | 0 | 0.05 | **0.15** | 0 | **0.50** | **0.20** | 0.10 |
| T2 | 0 | 0 | 0 | 0.10 | 0.10 | 0 | 0.35 | **0.30** | 0.15 |
| T3 | 0.05 | **0.40** | 0 | 0.05 | 0.10 | 0 | 0.15 | 0.15 | 0.10 |
| T4 | 0.15 | **0.30** | 0.10 | 0.10 | 0.10 | 0 | 0.05 | 0.10 | 0.10 |
| T5 | 0.20 | **0.30** | **0.15** | 0.05 | 0.10 | 0 | 0.05 | 0.10 | 0.05 |
| T6 | **0.50** | 0.10 | 0 | 0.20 | 0 | 0 | 0.05 | 0.05 | 0.10 |
| T7 | **0.50** | 0.10 | 0 | 0.10 | 0.10 | 0 | 0.05 | 0.05 | 0.10 |
| T8 | 0.15 | 0 | 0.10 | **0.35** | 0.05 | 0 | 0.05 | 0.10 | **0.20** |
| T9 | 0.05 | 0.15 | **0.40** | 0.10 | 0.05 | 0 | 0.10 | 0.10 | 0.05 |
| T10 | 0 | 0 | 0 | 0 | 0.10 | **0.60** | 0.10 | 0.15 | 0.05 |
| T11 | 0 | 0 | 0 | **0.20** | 0.20 | 0 | 0.25 | **0.35** | 0.00 |
| T12 | 0 | 0 | 0 | 0 | 0 | 0 | 0.15 | **0.55** | 0.30 |
| T13 | 0.10 | 0.10 | 0 | 0.10 | 0 | 0.40 | 0.10 | 0.20 | 0.00 |
| T14 | 0.10 | 0.05 | 0.05 | 0.25 | **0.25** | 0 | 0.15 | 0.10 | 0.05 |

### Sélection

1. **Filtres durs** : capacités requises (`function_calling`, `multimodal`) et
   `context_length >= ctx_requis`.
2. **Frontière de Pareto** sur (`Q_eff`, coût) : garde les modèles non dominés.
3. **Argmax du score composite** sur la frontière.
4. **Tie-break** : coût croissant, puis vitesse décroissante, puis préférence self-hosted.

### Fallback (REVUE-W4 M1 + N2, inc3 M4)

- Compteur d'echecs consecutifs par modele ; apres `max_failures` le modele
  est marque indisponible.
- L'etat circuit-breaker persiste sur l'instance ; l'ensemble des modeles
  deja essayes (`tried`) est **par run** : porte par l'appelant via
  `next_fallback(..., tried: &mut HashSet<String>)` et reinitialise a chaque
  requete. Il ne fuit donc pas d'une requete a l'autre.
- La politique ne re-selectionne jamais un modele deja essaye dans le run,
  evitant le ping-pong.
- Echelle : meme tier → tier superieur → `SelfHosted`.
- Increment 3 : le compteur simple devient une **machine a etats**
  (voir § « Circuit breaker a etats ») ; le reset sur succes (M4) est
  implemente via `record_success` (methode inherente : le trait
  `FallbackPolicy` d'`omega-ports` est hors perimetre de l'increment).

### Circuit breaker a etats (increment 3, REVUE-W4 M4)

Machine a etats par modele, implementee dans
`omega/crates/omega-router/src/fallback.rs` (seul fichier de code touche) :

```text
Closed --(N echecs consecutifs)--> Open --(delai ecoule)--> HalfOpen
  ^                                  |                        |
  |----(record_success, reset)--------+----(sonde echouee)----+
  +----(sonde reussie : record_success)-----------------------+
```

- **Parametres** : `max_failures` = N (configurable via `new`, defaut
  `DEFAULT_MAX_FAILURES = 3`, sature a >= 1) ; `open_timeout` (configurable
  via `with_open_timeout`, defaut `DEFAULT_OPEN_TIMEOUT = 60 s`).
- **Horloge injectable** : `with_clock(Fn() -> Instant)` ; l'horloge systeme
  (`Instant::now`) est le defaut, une horloge manuelle (`Arc<Mutex<Instant>>`
  avancee a la main) rend les tests deterministes sans `sleep`. Le passage
  `Open` → `HalfOpen` est derive paresseusement a la lecture : `is_available`
  (`&self`, impose par le trait) ne mute jamais.
- **`record_success`** (methode inherente, le trait `FallbackPolicy` etant
  hors perimetre) : remet le compteur a zero et repasse `Closed` — y compris
  apres une sonde `HalfOpen` reussie. Sans effet sur un modele inconnu.
- **`is_available`** : faux si `Open`, vrai si `Closed` ou `HalfOpen`.
- **`record_failure`** : en `Closed`, incremente et ouvre au seuil N ; en
  `Open` avant la fin du delai, ne prolonge pas l'ouverture (garde
  `opened_at` d'origine) ; en `HalfOpen` (delai ecoule), re-ouvre et le delai
  repart de zero.
- **`next_fallback`** : les candidats `Open` sont exclus ; les `Closed`
  sont preferes (escalade meme tier → tier superieur → `SelfHosted`).
  Une sonde `HalfOpen` n'est proposee que si aucun `Closed` n'est eligible,
  a raison d'**une seule sonde par run** : la proposition insere le marqueur
  interne `__omega_half_open_probe__` dans `tried`, et toute sonde
  supplementaire avec le meme `tried` retourne `None`. Un `tried` frais
  (contrat par run) re-autorise la sonde.
- **Etat observable** : `state(model_id) -> CircuitState`
  (`Closed`/`Open`/`HalfOpen`, inconnu = `Closed`) ; `failure_count`,
  `max_failures()`, `open_timeout()` exposes pour tests et telemetrie future.

### Cache (REVUE-W4 M2)

- Clé déterministe : hash de `(TaskType, ctx_requis, capacités requises, texte,
  registry_version)`.
- `registry_version` représente la version du catalogue / état de santé. Un
  changement de catalogue produit une clé différente et donc un `miss`.
- Invalidation manuelle (`clear`) ou par clé (`remove`) toujours disponible.

## Tests d'acceptation

1. **T1 → rapide/bon marché** ; **T5 → élite**.
2. **Filtre dur** : un modèle avec `context_length` insuffisant est exclu.
3. **Fallback** : après `max_failures` échecs, le routeur propose un autre modèle.
4. **Fallback sans ping-pong** : un modèle déjà essayé n'est pas re-sélectionné.
5. **Escalade de tier** : épuisement du tier courant → tier supérieur / self-hosted.
6. **Cache** : `hit` sur requête identique, `miss` sur requête différente ou
   `registry_version` différente.
7. **Déterminisme** : même entrée → même décision.
8. **Français accentué** : `Résume`, `équation`, `génère`… sont reconnus.
9. **Collisions de priorité** : `genere un plan d'architecture` → `LongReasoning`,
   `analyse cette image` → `Vision`, `fix the build` → `TargetedEdit`,
   `genere une image d'une stratégie marketing` → `MediaGeneration`,
   `dessine le plan de la maison` → `MediaGeneration`.
10. **Fallback par run** : deux requêtes consécutives sur la même instance
     obtiennent chacune le fallback complet depuis zéro (pas de fuite de `tried`).
11. **Circuit breaker inc3** : `Closed` → `Open` apres N echecs (N configurable,
    defaut 3) → `HalfOpen` apres le delai (defaut 60 s) → `Closed` si la sonde
    reussit, `Open` (delai reparti de zero) si elle echoue.
12. **Reset sur succes (M4)** : `record_success` remet le compteur a zero ;
    apres reset, il faut a nouveau N echecs consecutifs pour ouvrir.
13. **Sonde unique par run** : `next_fallback` exclut les modeles `Open`,
    prefere les `Closed` (meme a score inferieur) et n'autorise qu'une seule
    sonde `HalfOpen` par `tried` ; un `tried` frais re-autorise la sonde.
14. **Determinisme** : horloge injectee, aucun `sleep` dans les tests.

## Non-objectifs de l'incrément

- Pas de synchronisation catalogue (W4 incrément 2).
- Pas de feedback bandit (W4 incrément 3).
- ~~Pas de half-open circuit breaker ni de remise à zéro du compteur d'échecs
  (ADR + incrément 3)~~ → **fait en increment 3** (machine a etats +
  `record_success`, REVUE-W4 M4 clos ; sans ADR dedie, le § « Circuit breaker
  a etats » ci-dessus tient lieu de spec).
- Pas de binding TS / protocole (W7).
- Lemmatisation partielle : la forme impérative `ecris` (→ `Implementation`) est
  couverte (REVUE-W4 N3), mais les verbes génériques `faire`/`fais` restent
  **dette** : trop polysémiques pour un classement sûr sans casser l'existant.

## Provenance

- Taxonomie et formules : `[inferred]` depuis `MODEL-ROUTING-BENCHMARKS.md` §2-3.
- Placement des crates : `[inferred]` depuis `OMEGA-BLUEPRINT.md` §2.1-2.2.
- Contrainte « zéro réseau » : `[stated]` par la mission.
- Correction REVIEW-W4 : `[observed]` — `cargo test --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo fmt --all --check`, `python3 omega/scripts/check_dep_graph.py` et
  `--self-test` tous verts.
