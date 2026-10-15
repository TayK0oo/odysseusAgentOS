# Revue de code — W6 inc1 (économie de contexte)

> **Réviseur :** SFD Reviewer (indépendant, adversarial) · **Date :** 2026-10-15
> **Périmètre :** `omega/crates/omega-context/src/*` + `omega-types/src/error.rs`
> (`BudgetExceeded`) + `omega/scripts/check_dep_graph.py` (règle `adapters` élargie).
> **Exécutions effectuées :** `cargo test --workspace` (vert, 29 tests `omega-context`),
> `cargo clippy --workspace --all-targets -- -D warnings` (vert, exit 0),
> `cargo fmt --all --check` (vert), `python3 scripts/check_dep_graph.py` (valide, 11 crates, 0 cycle / 0 arête interdite).
> **Reproductions :** crate isolé sous `/tmp/opencode/repro` (hors dépôt, aucun fichier du dépôt modifié).

---

## §1 Verdict

**GO avec réserves — score 7/10.**

Les cinq leviers sont implémentés, **déterministes** (zéro LLM), et les garanties
centrales tiennent : estimateur stable, budget typé sans sur-consommation, système
préservé hors cas dégénéré, gate rarest-token (cas Hermes) couverte, disclosure 3
niveaux sans fuite du corps, cache par signature invalidé, déterminisme de
compaction. Câblage `register_dyn`/`resolve_dyn` conforme au contrat W3, graphe de
dépendances sain (0 cycle, 0 arête interdite), clippy/fmt propres.

Trois réserves substantielles : (1) un **pied-dans-la-porte public** — `TokenEstimator`
dérive `Default` qui produit une instance dont `estimate()` **panique** sur tout
texte non vide ; (2) `assemble` viole **silencieusement** le budget quand le seul
préfixe système dépasse déjà le plafond (contrat §4.3 non tenu, aucun signal) ;
(3) la gate rarest-token, à égalité d'IDF, peut choisir un token de bruit et **rater
l'outil évident** (reproduit). Corrections localisées, sans réécriture (cf. §2).

---

## §2 Constats par sévérité

### BLOQUANT (DOIT)

**B1 — `TokenEstimator` dérive `Default` → instance invalide → panic division par zéro.**
`omega-context/src/token.rs:13-16` (`#[derive(Default)]` sur un struct portant
`chars_per_token: u32`) + `token.rs:26-33` (`chars.div_ceil(self.chars_per_token as usize)`).
Le `Default` dérivé fixe `chars_per_token = 0` ; `estimate("x")` appelle
`1usize.div_ceil(0)` → `panic!("attempt to divide by zero")`. Prouvé : `rustc`
isole confirme le panic, et `TokenEstimator::default().estimate("")` rend `0` (chemin
vide) masquant le piège. `default()` n'est pas encore appelé dans le dépôt (seul
`::new()` l'est), mais le type est **réexporté** (`lib.rs:30`) et son `Default` est
public. Correctif : `impl Default for TokenEstimator { fn default() -> Self { Self::new() } }`
(ou supprimer la dérive et ne garder que `new()`).

### DEVRAIT

**M1 — `assemble` rend un résultat hors budget, sans signal, quand le système dépasse déjà.**
`omega-context/src/engine.rs:76-95`. `assemble_sync` préserve toujours le préfixe
système mais ne vérifie jamais `system_tokens <= max_tokens`. Reproduit :
`DefaultContextEngine::new(5)` + un seul `System` de 10 tokens → résultat de 10
tokens, `dropped=0`, `report.estimated_tokens=10 > 5`, retour `Ok` (jamais
`BudgetExceeded`). Le contrat W6 §4.3 (« le résultat respecte le budget **et**
préserve le système ») est incohérent dans ce cas ; contrairement à `truncate_compact`
(§4.6), `assemble` ne documente aucune exception. Correctif : documenter la primauté
du système (comme §4.6) **et** exposer le dépassement (champ `over_budget: bool`
dans `AssembleReport`, ou renvoyer `BudgetExceeded`), + test dédié.

**M2 — Gate rarest-token : l'égalité d'IDF résolue « premier token de la requête » peut rater l'outil évident.**
`omega-context/src/tool_surface.rs:100-116` (`gate_token`) + `119-148` (`search`).
Reproduit (crate `/tmp/opencode/repro`) :
`search("open file for writing", 10)` sur `{file_write, file_read, openssl}` → `["openssl"]`.
`open` (df=1, présent **uniquement** dans `openssl`) et `writing` (df=1, dans
`file_write`) ont le même IDF ; le tie-break choisit `open` (position 0) → la gate
`contains_gate` exclut `file_write`/`file_read` pourtant évidents. Correctif (Phase 1) :
documenter explicitement la fragilité et couvrir le cas par un test ; Phase 2 : couverture
minimale / stemming (déjà planifié, W6 §7) réduira l'impact.

**M3 — `truncate_compact` : budget < système seul → le marqueur est quand même inséré (contredit §4.6).**
`omega-context/src/compaction.rs:34-55`. Reproduit : `truncate_compact([System 10 tok, User 2 tok], 5)`
→ `[System, "…[compacted 1 messages omis]…"]` = **18 tokens** (pas « le système
préservé tel quel » promis par W6 §4.6). §3 (« insère le marqueur si k>0 ») est suivi,
§4.6 non. Correctif : trancher — soit ne pas insérer le marqueur quand
`system_tokens + marker > budget` (préserve « tel quel »), soit reformuler §4.6.

**M4 — Dérive blueprint : `omega-context` dépend de `plugin-api`, pas de `config`.**
`omega-context/Cargo.toml` (deps réelles : `types, ports, registry, plugin-api`) vs
`OMEGA-BLUEPRINT.md` §2.1 l.83 (`types, ports, config, registry`). L'élargissement
`adapters → registry/plugin-api` (`check_dep_graph.py:42`) est **justifié et minimal**
(le `Contributor` doit vivre côté adaptateur pour garder `omega-app` agnostique, W6 §6),
mais la table du blueprint n'est plus la source de vérité. Correctif : mettre à jour la
table §2.1 et/ou poser un ADR dédié à l'arête `adapters → plugin-api`.

### AMÉLIORATIONS (mineur)

- **Perf** : `ToolSurfaceIndex::add` reconstruit df/idf en O(N) à chaque ajout →
  `from_docs` en O(N²) (`tool_surface.rs:73-77,79-96`) ; `search` fait `Vec::contains`
  linéaire par terme. Acceptable Phase 1 (petit N), à noter pour l'indexation.
- **DRY** : extraction du préfixe système dupliquée (`engine.rs:76-81` vs
  `compaction.rs:26-31`) ; `ContextBudget.used` jamais utilisé par le moteur
  (`engine.rs:38` garde un `budget` dont seul `max_tokens()` est lu) + double
  `TokenEstimator` (moteur et budget).
- **Config silencieuse** : `resolve_max_tokens` (`contributor.rs:61-67`) retombe sur le
  défaut sans signal si `context.max_tokens` est non numérique (`as_u64` → `None`).
- **i18n/protocole** : le marqueur `…[compacted N messages omis]…` est une chaîne
  française codée en dur, injectée dans le prompt ; à terme versionnable/configurable.

---

## §3 Cas adversariaux reproduits (crate `/tmp/opencode/repro`, hors dépôt)

1. `TokenEstimator::default().estimate("")` → `0` (masque) ; `estimate("x")` → **panic**
   division par zéro (via `1usize.div_ceil(0)`).
2. `ToolSurfaceIndex::search("open file for writing")` sur
   `{file_write("write data to a file"), file_read("read data from a file"), openssl("open ssl certificate")}`
   → **`["openssl"]`** : l'outil évident `file_write` est exclu par la gate.
3. `DefaultContextEngine::new(5).assemble_sync([System 10 tokens])` → retour `Ok`, 10
   tokens rendus (hors budget), `report.estimated_tokens=10`, `dropped=0`, aucun drapeau.
4. `truncate_compact([System 10 tok, User 2 tok], 5)` → `[System, marqueur]` = 18
   tokens (marqueur inséré alors que le système seul dépasse le budget).

---

## §4 Tests manquants

- **Estimateur** : `TokenEstimator::default()` ne doit pas paniquer (test de non-régression) ;
  entrée multi-octets/CJK (émoji, `你好…`) et très longue (pas d'overflow) ; rôle du
  ratio documenté comme heuristique (pas une garantie de borne réelle).
- **Budget** : `enforce` à la frontière exacte (`used+additional == max`) ; `max_tokens=0`.
- **Assemble** : **système seul ≥ budget** (dépassement signalé/documenté) ; message
  `System` intercalé (traité comme non-système → documenté) ; liste vide ; `budget=0`.
- **Tool search** : tie-break adversaire (M2) — gate = token de bruit → l'outil évident
  est raté ; `top_k=0` ; requête vide ; requête à tokens dupliqués ; assertion de
  déterminisme (deux appels identiques → même ordre).
- **Skills** : remplacement par un corps **identique** (pas de re-render, `render_count`
  stable) ; `list()` après `add` (résumé reflète le nouveau corps) — la non-fuite du
  corps est déjà garantie par l'absence de champ `body` dans `SkillSummary`.
- **Compaction** : budget < système seul (M3) ; marqueur vs contenu utilisateur
  identique (ambiguïté) ; budget exactement égal au total (frontière).
- **Garde-fou** : `check_dep_graph.py` n'a **aucun cas positif** validant que
  `adapters → registry`/`plugin-api` passe (seuls des cas négatifs) — ajouter un
  fixture « accepté ».

---

## §5 Points solides

- **Déterminisme réel** : estimateur, budget, assemble, gate, cache, compaction — aucun
  `rand`, aucun `HashMap` dans les chemins de tri, tri stable explicite
  (`tool_surface.rs:146` score puis nom).
- **Budget typé correct** : `check` non mutatif + `enforce` « vérifie puis consomme »,
  `saturating_add` contre l'overflow, non-sur-consommation testée (`budget.rs:94-100`).
- **`BudgetExceeded` en variante dédiée** d'`OmegaError` (`error.rs:35-40`) : événement
  de domaine testable par `matches!`, conforme à la décision W6 §6.
- **Gate Hermes fidèle** : l'exclusion des outils « token commun seulement » est
  réellement couverte (`tool_surface.rs:160-175`).
- **Réservation pire-cas du marqueur** (`compaction.rs:35`) : borne supérieure par le
  plus grand `omitted`, garantit `résultat ≤ budget` sur le chemin normal.
- **Disclosure sans fuite** : `SkillSummary` n'embarque pas `body` ; cache par
  signature FNV-1a correct (constantes vérifiées) et invalidé au remplacement
  (`skills.rs:193-204`).
- **Architecture propre** : `omega-context` ne dépend ni de `core` ni de `app` ;
  `ContextContributor` enregistre via `register_dyn` résolu par `App::context_engine()`
  (contrat W3 validé, `contributor.rs:122-144`). Garde-fou élargi **minimalement** et
  avec commentaire de justification (`check_dep_graph.py:39-42`).

---

## §6 Résolution (exécuteur SFD, 2026-10-15)

> **Statut final :** B1 corrigé, M1/M2/M3 corrigés, M4 documenté en dette. 39 tests
> `omega-context` (dont 10 nouveaux), clippy/fmt/dep-graph verts.

| Constat | Correctif | Vérification |
|---|---|---|
| **B1** `TokenEstimator::default()` panique (division par zéro) | `impl Default` manuel → `Self::new()` (canonique `ceil(chars/4)`), dérive retirée | `token::tests::default_ne_panique_pas` + `estimate_counts_chars_not_bytes` + `estimate_long_input_does_not_overflow` |
| **M1** `assemble` hors budget silencieux (système seul > plafond) | `AssembleReport.over_budget` (signal sync) + `assemble` renvoie `BudgetExceeded` | `engine::tests::assemble_system_alone_over_budget_is_signaled` + `assemble_returns_typed_error_when_system_alone_over_budget` |
| **M2** gate rarest-token tie-break « premier token » fragile | départage déterministe : plus haut IDF (`df` minimal) puis **ordre lexical croissant**, indépendant de la position | `tool_surface::tests::gate_tie_break_is_lexical_and_position_independent` + `open_file_for_writing_gate_is_deterministic_phase1` (repro §3 verrouillée) |
| **M3** `truncate_compact` insère un marqueur mensonger si `budget < système` | retour système « tel quel » sans marqueur quand `système + marqueur > budget` | `compaction::tests::system_alone_over_budget_preserves_system_without_lying_marker` |
| **M4** blueprint §2.1 (`omega-context → config`) incohérent | documenté en dette (dépendance réelle `types, ports, registry, plugin-api`) dans `W6-contexte.md` §6.1/§8 — table blueprint hors périmètre du correctif | — |

Réserves résiduelles (dette, cf. `W6-contexte.md` §8) : rappel lexical sans stemming
(M2, inc2), table blueprint §2.1 (M4), config silencieuse (`contributor.rs`), i18n du
marqueur de compaction.
