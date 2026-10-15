# Revue de code — W4 inc1 (routeur de modèles)

> **Réviseur :** SFD Reviewer (indépendant, adversarial) · **Date :** 2026-10-15
> **Périmètre :** `omega/crates/omega-router` + `omega/crates/omega-providers`
> (+ contrats `omega-ports`, `omega-types`).
> **Exécutions effectuées :** `cargo test --workspace` (vert, 28 tests router),
> `cargo clippy --workspace --all-targets -- -D warnings` (vert), `python3 scripts/check_dep_graph.py` (valide, 0 cycle / 0 arête interdite).
> **Reproductions :** crate de test isolé sous `/tmp/opencode/advtest` (hors dépôt, aucun fichier du dépôt modifié).

---

## §1 Verdict

**NO-GO — score 5/10.**

Le cœur numérique (scoring, Pareto, sélecteur) est correct, borné et bien testé.
En revanche le **classifieur**, porte d'entrée du routeur, est cassé pour le
français accentué (langue du projet) et produit des **erreurs de routage
dangereuses** (résumé → modèle trivial, maths → trivial, « génère un plan » →
génération média). Le **fallback** re-sélectionne des modèles déjà en échec et
ping-pong ; le **cache** ne couvre ni la version du registre ni l'état de santé ;
et les **ports `ScoringPolicy`/`Selector`/`FallbackPolicy`** vivent dans
l'adaptateur (`omega-router`) au lieu de `omega-ports`, en contradiction avec le
blueprint §3 et RULES-OMEGA §2.2. Ces points sont bloquants pour un livrable dont
l'unique objet est « bien router ».

Les correctifs sont localisés et sans réécriture du noyau numérique (cf. §2).

---

## §2 Constats par sévérité

### BLOQUANT

**B1 — Le classifieur ne reconnaît pas le français accentué → routage dangereux.**
`omega-router/src/classifier.rs:34-49` (regex sur formes non accentuées) +
`omega-types/src/lib.rs:335-341` (`lower()` sans normalisation Unicode : pas de
NFD, pas de strip d'accents). Toute la taxonomie française (`résume`, `corrige`,
`crée`, `équation`, `synthèse`, `génère`, `écris`…) n'est **jamais** matchée car
les motifs stockés sont non accentués.

Preuve (reproduction, cf. §3) :
- `"Résume ce document en trois points"` → `TrivialCompletion` (attendu `Summarization`).
- `"Résous cette équation du second degré"` → `TrivialCompletion` (attendu `Math`).
- `"Résumer ce texte"` → `TrivialCompletion` (le témoin non accentué `"resume ce texte"` → `Summarization`).

Conséquence : une tâche de résumé ou de maths est routée vers un **petit modèle
trivial rapide/bon marché** (profil T1 vitesse), c'est-à-dire le mauvais modèle
pour le travail. Correctif : normaliser (`NFD` + suppression des marques
diacritiques) avant matching, **ou** inclure les formes accentuées dans les regex ;
ajouter des tests d'acceptation accentués.

**B2 — Priorité des mots-clés → classe dangereuse.**
`classifier.rs:81-116` : l'ordre `re_embedding → re_media → re_agentic → … →
re_reasoning` fait qu'un mot « piège » d'un regex amont écrase la vraie intention.
- `re_media` (l.34) contient `image|photo|genere` et est testé **avant**
  `re_reasoning` (l.49) qui contient `plan|analyse|architecte` :
  `"genere un plan d'architecture pour la migration"` → `MediaGeneration`
  (attendu `LongReasoning`) — routé vers un générateur image/vidéo.
  `"analyse cette image en détail"` → `MediaGeneration` (attendu `Vision`) —
  confond « analyser une image » (entrée multimodale) avec « générer une image ».
- `re_implementation` (l.45, `build`) testé avant `re_edit` (l.47, `fix`) :
  `"fix the build"` → `Implementation` (attendu `TargetedEdit`).

Correctif : réordonner la priorité (raisonnement/vision avant media ; édition
avant implémentation) ou introduire des règles de désambiguïsation
(`plan|stratégie|architecture` ⇒ T6 avant le media), et tester ces collisions.

### MAJEUR

**M1 — Fallback re-sélectionne des modèles déjà en échec (ping-pong), escalade de tier jamais atteinte.**
`omega-router/src/fallback.rs:71-130`. `next_fallback` n'exclut que
`failed.model_id` (l.75) et ne mémorise **pas** l'ensemble des modèles déjà
essayés. Un modèle échoué une fois (compteur < `max_failures`, donc encore
`is_available`) est re-sélectionné à l'étape « même tier ». Reproduction
(`max_failures=3`, deux modèles `Standard` m1/m2, cf. §3) :

```
step0: m1 -> m2 ; step1: m2 -> m1 ; step2: m1 -> m2 ; step3: m2 -> m1
step4: m1 -> m2 (m1 devient indisponible) ; step5: m2 -> None
```

Le ping-pong m1↔m2 dure 5 tours ; l'escalade vers un tier supérieur **ne se produit
jamais** tant qu'un même-tier reste « disponible ». L'escalade est *finie*
(elle s'arrête à l'épuisement) mais inefficace. Correctif : propager un ensemble
« déjà essayés » (ou exclure tout modèle dont `failure_count>0`), et tester
l'épuisement + l'escalade réelle.

**M2 — Cache : clé insuffisante, décision périmée possible.**
`omega-router/src/cache.rs:24-36`. La clé = `hash(TaskType, capacités, contexte, texte)`.
Elle **n'inclut pas** la version du registre/catalogue ni l'état de santé des
modèles, contrairement au blueprint §6.1 (« clé = hash(tâche, profil, **état santé
registre**) »). Après un changement de catalogue (prix, ajout/retrait de modèle) ou
de santé, `get()` retourne la décision **obsolète**. Invalidation uniquement
manuelle (`clear`/`remove`), aucun couplage événementiel. Correctif : ajouter une
`registry_version`/`health_version` à la clé, ou invalider sur tout changement de
registre ; ajouter un test « catalogue changé ⇒ miss ».

**M3 — Ports `ScoringPolicy`/`Selector`/`FallbackPolicy` définis dans l'adaptateur, pas dans `omega-ports`.**
`omega-router/src/scoring.rs:37`, `selector.rs:19`, `fallback.rs:12`. Le blueprint §3
(« router-policy : `ScoringPolicy` + `Selector` + `FallbackPolicy` », régime slot) et
RULES-OMEGA §2.2 (DIP : le noyau dépend d'abstractions, ports dans la crate `ports`)
exigent ces traits dans `omega-ports`. À la place, `omega-ports` expose un trait
`RouterPolicy` (l.57-61) **orphelin** (aucun implémenteur, aucun consommateur) et de
signature incohérente (`async fn choose(...) -> ModelId` ≠ `Selector::select`).
Seul `TaskClassifier` a été correctement relocalisé (cf. W4-routing.md:35-38).
Correctif : déplacer les 3 traits dans `omega-ports` (ISP, non-async), implémenter/
supprimer `RouterPolicy`, ré-exporter depuis `omega-router`.

**M4 — Compteur d'échecs jamais remis à zéro (aucun signal de succès, aucun half-open).**
`omega-router/src/fallback.rs:52-58`. `record_failure` incrémente ; il n'existe
**aucun** `record_success`/reset/cooldown. Un échec transitoire (blip réseau)
blackliste **définitivement** le modèle une fois `max_failures` atteint.
*Nota :* documenté comme non-objectif de l'incrément (W4-routing.md:108 « pas de
half-open ») — pas un bug caché, mais un manque à tracer en ADR avant l'incrément 3.

### MINEUR

**m1 — `HeuristicTaskClassifier::new()` recompile 10 regex à chaque construction** (`classifier.rs:32-51`) ; `Default` → `new()` l'aggrave. Utiliser `OnceLock`/`LazyLock`. (Slot construit une fois : impact faible.)
**m2 — `RoutingRequest::lower()` alloue une `String` à chaque `classify`** (`omega-types/src/lib.rs:335`). Préférer une recherche insensible à la casse sans allocation.
**m3 — `score()` recalcule min/max de normalisation par comparaison dans `max_by`** (`scoring.rs:253-258` + `selector.rs:83`) → O(n²·log n). Négligeable aux tailles actuelles.
**m4 — `pick_best` (fallback) sans tie-break** (`fallback.rs:141-153`) : en cas d'égalité stricte de score, résultat dépendant de l'ordre d'itération, contrairement au tie-break complet du sélecteur. Déterministe *si* l'entrée est ordonnée, mais fragile.
**m5 — `DefaultHasher` (SipHash clé fixe) non garanti stable entre versions Rust** (`cache.rs:3-5`) : clé reproductible intra-processus, mais non persistable/versionnable. Acceptable pour un cache mémoire.

### NIT

**n1 — `unwrap()` sur `Regex::new`** (`classifier.rs:34-49`) : justifié (motifs constants ne peuvent échouer), mais réévalué à l'exécution.
**n2 — `.expect("pareto non vide")`** (`selector.rs:105`) : justifié (frontière non vide si `eligible` non vide). OK.
**n3 — Messages d'erreur `RouterError` et `reason` en dur** (`selector.rs:31-34, 107-116` ; `fallback.rs:151`) : chaînes diagnostic (non UI), hors périmètre i18n, mais à centraliser à terme.

---

## §3 Cas adversariaux reproduits

Reproduction exécutée via `cargo run` (crate isolé `/tmp/opencode/advtest`), sorties réelles.

### 3.1 Classifieur — français accentué non reconnu

| Entrée | Sortie observée | Attendu | Sévérité |
|---|---|---|---|
| `Résume ce document en trois points` | `TrivialCompletion` | `Summarization` | BLOQUANT |
| `Résous cette équation du second degré` | `TrivialCompletion` | `Math` | BLOQUANT |
| `Résumer ce texte` | `TrivialCompletion` | `Summarization` | BLOQUANT |
| `Résumé du rapport` | `TrivialCompletion` | `Summarization` | BLOQUANT |
| `synthèse de ce document` | `TrivialCompletion` | `Summarization`/`Research` | BLOQUANT |
| `resume ce texte` (témoin non accentué) | `Summarization` | `Summarization` | — (contrôle) |

### 3.2 Classifieur — collisions de mots-clés (priorité)

| Entrée | Sortie observée | Attendu | Sévérité |
|---|---|---|---|
| `genere un plan d'architecture pour la migration` | `MediaGeneration` | `LongReasoning` | BLOQUANT |
| `analyse cette image en détail` | `MediaGeneration` | `Vision` | BLOQUANT |
| `fix the build` | `Implementation` | `TargetedEdit` | BLOQUANT |
| `génère un plan d'architecture…` (accentué) | `LongReasoning` | — | (illustre l'asymétrie B1/B2) |

### 3.3 Fallback — ping-pong sur modèles déjà échoués

Configuration : `max_failures=3`, candidats `m1`, `m2` (tier `Standard`, même coût/vitesse,
`m2` chat=80 > `m1` chat=70). Boucle `record_failure` + `next_fallback` :

```
step0: failed=m1 -> next=m2   (m1 dispo, m2 dispo)
step1: failed=m2 -> next=m1   (m1 dispo, m2 dispo)   ← re-sélection de m1 déjà échoué
step2: failed=m1 -> next=m2   (m1 dispo, m2 dispo)   ← re-sélection de m2 déjà échoué
step3: failed=m2 -> next=m1   (m1 dispo, m2 dispo)
step4: failed=m1 -> next=m2   (m1 INDISPO, m2 dispo)
step5: failed=m2 -> None      (épuisement)
```

### 3.4 Scoring — bornes vérifiées (non cassé)

- Un seul candidat `(coût=0, tok/s=0)` : `score = 0.800000` — pas de NaN/div-zero, borné.
- Inversion de coût correcte : `cheap` (coût 0) score T1 `0.890000` > `pricy` (coût 100/200) `0.690000` — coût plus bas ⇒ meilleur score. ✓
- `weights` normalisés (`is_normalized() == true`) pour les 14 profils. ✓

---

## §4 Tests manquants

1. **Classifieur** : cas accentués français (`résume`, `corrige`, `crée`, `équation`, `synthèse`, `génère`, `écris`) ; collisions de priorité (`genere un plan`, `analyse cette image`, `fix the build`) ; normalisation Unicode.
2. **Sélecteur** : `NoEligibleModel` (seul `EmptyRegistry` est testé — `selector.rs:361-370`) ; exclusion d'un modèle **dominé** sur la frontière ; tie-break complet (coût croissant, vitesse décroissante, préférence self-hosted) ; déterminisme sur liste permutée.
3. **Fallback** : ping-pong / re-sélection d'un modèle déjà échoué ; épuisement complet → `None` ; escalade réelle de tier (même-tier → supérieur → self-hosted) ; cas « failed.model_id absent du catalogue ».
4. **Cache** : décision périmée après changement de catalogue/santé (version de registre) ; collision de clé.
5. **Scoring** : candidat unique / coût=0 / `tokens_per_second`=0 / candidats identiques (déjà partiellement vérifiés en §3.4, à figer en tests unitaires).
6. **Ports** : object-safety `dyn TaskClassifier` (symétrique du test `Provider` existant, `omega-ports/src/lib.rs:125-130`).

---

## §5 Points solides

- **Score composite borné et robuste** : normalisations log-bornees, retour `1.0` sur candidat unique (`scoring.rs:289-291`) et sur valeurs identiques (`:295, :315`), aucune division par zéro avec coût=0 ou `tok/s`=0. Inversion de coût **correcte** (`inverse_log_normalize`, `:305-322`).
- **Poids sommant à 1** pour les 14 profils, et **testé** (`weights_sum_to_one`, `scoring.rs:351-376`).
- **Sélection solide** : filtres durs (contexte + capacités, `selector.rs:63-65`), frontière de Pareto correcte (jamais de modèle dominé, `:128-154`), erreur typée `RouterError` (pas d'`anyhow`), tie-break **déterministe et complet** (`:83-104`).
- **Déterminisme** : même entrée → même sortie vérifié (§3.4, 3 runs identiques) ; `DefaultHasher` à clé fixe ⇒ clé de cache reproductible.
- **`TaskType` source unique** (`omega-types`), pas de doublon (`task.rs` = simple ré-export) ; `TaskClassifier` bien relocalisé en port.
- **Graphe de dépendances valide** (0 cycle, 0 arête interdite) ; **clippy `-D warnings` vert** ; 28 tests router passants.
- **Commentaires orientés POURQUOI**, français sans accents dans les identifiants, conformes aux conventions.

---

## Métriques

- Fichiers audités : 11 (7 `omega-router` + 2 `omega-providers` + `omega-ports` + `omega-types`).
- Violations SOLID : 1 (DIP — ports dans l'adaptateur, M3).
- Violations TDD : 2 (tests manquants B1/B2 et M1/M2 non couverts).
- Violations sécurité : 0 (aucun secret, aucune entrée réseau — logique pure).
- Violations accessibilité : 0 (hors périmètre, aucun artefact UI).
- Violations i18n : 0 bloquante (chaînes diagnostic en dur, n3).
- Tests passants : 28/28 (router), clippy `-D warnings` : 0 warning.
