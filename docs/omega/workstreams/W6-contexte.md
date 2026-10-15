# W6 — Économie de contexte (`omega-context`)

> **Workstream :** W6 · **Statut :** 🟢 incrément 1 · **Date :** 2026-10-15
> **Sources :** `OMEGA-BLUEPRINT.md` §5, `RE-OPENCLAW-HERMES.md` §3.2.2/§3.2.3/§4.8,
> `W3-app-durable.md` (contrat de câblage des ports).
> **Provenance :** `[observed]` (implémenté) · `[inferred]` (seuils, formats) · `[todo]`.

---

## 1. Objectif

Réduire le coût et la latence des tours d'agent en n'embarquant que le contexte utile.
Cinq leviers (blueprint §5.2), tous **déterministes côté système** — jamais un appel LLM :

1. **Budget avant action** : estimer, vérifier, plafonner ; dépassement = arrêt.
2. **Tool search** : BM25 + gate par le token le plus rare (surface d'outils réduite).
3. **Progressive disclosure** des skills (3 niveaux) : ne charger que ce qui sert.
4. **Compaction** : résumé structuré déterministe, sans modèle.
5. **Câblage** : le moteur et les index sont fournis par un `Contributor` au registre.

## 2. Périmètre incrément 1

| Élément | Rôle |
|---|---|
| `TokenEstimator` | Estimation déterministe `ceil(chars / 4)`, sans LLM |
| `ContextBudget` | `estimate → check → enforce` ; dépassement = `OmegaError::BudgetExceeded` |
| `DefaultContextEngine` | impl `ContextEngine` (`name="default"`, `owns_compaction=false`) ; assemble sous budget |
| `ToolSurfaceIndex` | BM25 simplifié (somme d'IDF) + gate « token le plus rare » + seuil optionnel |
| `SkillIndex` | Progressive disclosure 3 niveaux (`list`/`view`/`asset`) + cache par signature |
| `truncate_compact` | Garde les N derniers messages + marqueur `…[compacted k messages omis]…` |
| `ContextContributor` | Câble `DefaultContextEngine` (`register_dyn`) + index (`register_typed`) |

## 3. Contrats clés

- **`TokenEstimator`** : `estimate(text) = ceil(nb_chars / 4)`, stable (même entrée → même sortie).
- **`ContextBudget { max_tokens, used }`** : `check(additional)` échoue typé si
  `used + additional > max_tokens` ; `enforce` vérifie **puis** consomme (jamais de
  sur-consommation en cas d'échec).
- **`DefaultContextEngine::assemble`** : concatène les messages `System` + non-`System`
  **en respectant le budget** ; tronque les plus anciens **non-`System`** ; le préfixe
  `System` est **toujours préservé** (primauté du système, cf. §4.6). Si le système
  seul dépasse déjà le budget, le dépassement est **explicite** : `AssembleReport.over_budget = true`
  (voie synchrone) et `assemble` renvoie `BudgetExceeded` (voie port). Jamais de
  résultat hors contrat silencieux.
- **`ToolSurfaceIndex::search(query, top_k)`** : tokenisation lowercase + split non-
  alphanum ; IDF lissé BM25 `ln((N+1)/(df+1)) + 1` ; score = somme des IDF des termes de
  la requête présents ; **admission** : l'outil contient le token le plus discriminant de la
  requête (plus haut IDF = `df` minimal, le « mot de l'intention ») **ou** (si `min_score > 0`)
  son score atteint le seuil. Départage du token de gate (égalité de `df`) : **ordre
  lexical croissant**, déterministe et indépendant de la position des tokens. Tri stable :
  score décroissant puis nom croissant.
- **`SkillIndex`** : `list()` = noms + descriptions (niveau 1, corps non chargé) ;
  `view(id)` = corps (niveau 2), **mis en cache par signature** (FNV-1a du corps) ;
  `asset(id, name)` = fichier lié (niveau 3).
- **`truncate_compact(messages, budget)`** : préserve le préfixe `System`, garde le
  suffixe maximal des messages récents qui tient (marqueur inclus), insère le marqueur si
  `k > 0`. Si `budget < système + marqueur`, **aucun marqueur n'est inséré** : le système
  est préservé tel quel (pas de marqueur mensonger). Déterministe, sans LLM.
- **Câblage** : `ContextContributor` enregistre `Arc<dyn ContextEngine>` via
  `register_dyn` (résolvable par `App::context_engine()` — contrat W3), et les index via
  `register_typed` (capacités `tool-surface` / `skill-store`).

## 4. Garanties

1. Estimation, recherche, compaction : **déterministes** (zéro aléa, zéro LLM).
2. Budget : un dépassement produit une erreur **typée** (`BudgetExceeded`), jamais un
   silence ni une sur-consommation.
3. Assemble : le résultat **respecte le budget** **et préserve le système** ; si le
   système seul dépasse le budget, le dépassement est **signalé** (jamais silencieux).
4. Tool search : un outil qui ne partage que des tokens communs avec la requête est
   **exclu** par la gate (cas Hermes « send gmail email »).
5. Compaction : le résultat tient dans le budget (pour un budget ≥ système + marqueur) ;
   sinon le système est préservé **tel quel**, sans marqueur mensonger.

## 5. Tests d'acceptation

| # | Critère | Statut |
|---|---|---|
| (a) | Estimateur stable (même entrée → même sortie) + bornes `ceil` | ✅ |
| (b) | Budget : `check`/`enforce`, dépassement typé, non-sur-consommation | ✅ |
| (c) | Assemble sous budget + système préservé + coupes journalisées | ✅ |
| (d) | Tool search : gate rarest-token (cas Hermes) + seuil + ordre stable | ✅ |
| (e) | Skills 3 niveaux + cache par signature (invalidation au changement) | ✅ |
| (f) | Compaction : marqueur `k`, système/récents préservés, déterminisme | ✅ |
| (g) | `ContextContributor` câble moteur (dyn) + index (typed), budget depuis config | ✅ |

## 6. Décisions

- **`BudgetExceeded` en variante dédiée** d'`OmegaError` (pas `Config`) : l'échec de budget
  est un événement de domaine à part entière, testable par `matches!` sans ambigüité.
- **BM25 simplifié = somme d'IDF** (pas de normalisation tf/longueur k1/b) : suffisant pour
  la gate et le rappel en Phase 1 ; la normalisation complète est un raffinement Phase 2
  (documenté, cf. `RE-OH:482`).
- **Gate stricte par défaut** (`min_score = 0.0`) : reproduction fidèle du comportement
  Hermes ; le « OU dépasse un seuil » s'active explicitement via `with_min_score`.
- **Le `Contributor` vit dans `omega-context`** (catégorie `adapters`), pas dans
  `omega-app` : l'assemblage applicatif reste agnostique. Cela exige d'autoriser
  `adapters → registry`/`plugin-api` dans le garde-fou (aligné sur `OMEGA-BLUEPRINT.md`
  §2.1 qui liste `registry` comme dépendance de `omega-context`).
- **Cache par signature FNV-1a** : déterministe, sans horodatage (pas de dépendance FS en
  Phase 1) ; l'invalidation est testée en remplaçant un skill de même id.

### 6.1 Correctifs post-revue (REVIEW-W6, 2026-10-15)

- **B1 — `TokenEstimator::default()`** : `Default` implémenté manuellement (et non dérivé)
  pour renvoyer l'estimateur canonique `ceil(chars/4)` — la dérive produisait
  `chars_per_token = 0` et faisait paniquer `estimate()` (division par zéro). Test
  `default_ne_panique_pas`.
- **M1 — `assemble` hors budget** : ajout de `AssembleReport.over_budget` (signal
  explicite, voie synchrone) + `assemble` (port) renvoie `BudgetExceeded` quand le système
  seul dépasse le plafond. Primaute du système documentée.
- **M2 — gate rarest-token** : départage déterministe et robuste (plus haut IDF = `df`
  minimal, puis **ordre lexical croissant**), indépendant de la position des tokens dans la
  requête. Le rappel de l'outil « évident » pour des flexions non couvertes (`writing` vs
  `write`) reste une dette **stemming** (cf. §7).
- **M3 — `truncate_compact`** : si `budget < système + marqueur`, aucun marqueur n'est
  inséré (pas de marqueur mensonger) ; le système est préservé tel quel. Test dédié.
- **M4 — blueprint §2.1** : dépendances réelles d'`omega-context` = `types, ports,
  registry, plugin-api` (pas `config`). La table `OMEGA-BLUEPRINT.md` §2.1 (l.83) liste
  encore `config` et omet `plugin-api` → **dette documentaire** à corriger (hors périmètre
  de ce correctif, cf. §8).

## 7. Reste (incrément 2)

- Stemming (Snowball, `rust-stemmers`) pour un rappel lexical supérieur.
- Prompt caching par préfixe (≤4 breakpoints, affinité) — `CTX-04/05`.
- Compaction **structurée** branchée sur `ContextEngine::compact` (résumé + rechargement
  ciblé), décorateur de `Provider` rattrapant `ContextLengthExceeded`.
- Découverte `AGENTS.md` remontante + budget en caractères (mémoire bornée, W6↔W8).

## 8. Dette documentée (non bloquante, incrément 2+)

- **Stemming** : `search("open file for writing")` sur `{file_write, file_read, openssl}`
  renvoie `["openssl"]` (gate = `open`, token rare unique à `openssl`) ; `file_write` n'est
  récupéré que par couverture minimale/stemming. Comportement **déterministe** verrouillé
  par un test de non-régression (`open_file_for_writing_gate_is_deterministic_phase1`).
- **Blueprint §2.1** : corriger la dépendance d'`omega-context` (`config` → `plugin-api`).
- **Config silencieuse** (`contributor.rs`) : `context.max_tokens` non numérique retombe
  sur le défaut sans signal (à logger/typer).
- **i18n du marqueur** : `…[compacted k messages omis]…` est une chaîne française codée en
  dur injectée dans le prompt (à versionner/configurer).
