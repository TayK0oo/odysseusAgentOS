# Revue de code — FINAL-2 (incréments post-`51a9b545`, commit `f84d4b89`)

> **Auteur :** Reviewer SFD (audit adversarial indépendant) · **Date :** 2026-10-15
> **Périmètre :** diff `51a9b545..f84d4b89` — `omega-router/src/selector.rs`
> (`select_with_local_preference`), `omega-app/src/{lib,store}.rs` (garde plan,
> validation `run_id`), `omega-server/src/pipeline.rs` (record canonique),
> `omega-sandbox/src/approvals.rs` (`prune_denied`), `omega-providers/src/ollama.rs`
> (mapping/timeout), `omega-server/src/routes.rs` + UI (`api.ts`, `App.tsx`),
> cohérence docs (`W3-app-durable.md` §4, `W9-sandbox.md` §6, blueprint §2.1).
> **Commandes exécutées (lecture seule) :** `cargo test --workspace` (247 tests, 0 échec),
> `cargo clippy --workspace --all-targets -- -D warnings` (OK, 0 warning),
> `python3 scripts/check_dep_graph.py` (14 crates, 0 cycle, 0 arête interdite).

---

## §1 Verdict + score

**AJOURNÉ — 6/10.** Travail sérieux (garde plan testée, traversal bouché côté fichier
+ routes, 247 tests verts, clippy propre), mais **2 BLOQUANTS** interdisent de considérer
l'incrément comme clos :

- **B1** : `reqwest::Client::new()` sans timeout (`ollama.rs`, `ollama_catalog.rs`) —
  hang infini possible sur `complete`/`fetch` (disponibilité, fail-open par blocage).
- **B2** : `select_with_local_preference` choisit le meilleur local **sans filtrer
  l'éligibilité** — un local inéligible à fort score masque un local éligible dans la
  marge → décision de routing fausse en multi-candidats (exactement le flux W10 visé).

6 MAJEURS (fuite `Pending`, contexte 32768 non prouvé, aucun test HTTP end-to-end,
`debug_assert` tautologique, blueprint qui mélange réel/cible, `InMemoryEventStore`
sans validation) + 10 MINEURS. Aucun secret hardcodé, aucune injection SQL/XSS/commande
dans le périmètre (le seul flux extérieur → fichier est le `run_id`, désormais validé).

---

## §2 Constats

### BLOQUANT

**B1 — Pas de timeout HTTP : `complete`/`fetch` peuvent pendre indéfiniment.**
`omega-providers/src/ollama.rs:75-81` (`Client::new()`) et
`omega-providers/src/ollama_catalog.rs:64-71` (idem). `reqwest` sans `.timeout()` attend
indéfiniment un serveur qui accepte la connexion sans répondre (Ollama chargé/swap,
pause réseau). `omega-server` n'a pas de timeout propre sur `execute_run` : un POST
`/api/run` peut donc bloquer un worker Axum **pour toujours** (épuisement du pool,
fail-open par blocage — contraire à l'esprit W9 fail-closed).
**Correctif :** `Client::builder().timeout(Duration::from_secs(60)).connect_timeout(...)`
+ test (serveur qui ne répond jamais → erreur typée avant N s ; timeout court en test).

**B2 — `best_local` ignore l'éligibilité : un local inéligible masque le local éligible.**
`omega-router/src/selector.rs:37-46` : la boucle filtre sur `tier == SelfHosted` uniquement,
sans `covers(required_capabilities)` ni `context_length >= required_context`. `self_hosted_first`
(`ollama_catalog.rs:162-165`) re-vérifie l'éligibilité **du seul local retenu** et rend `false`
s'il est inéligible → retour au distant, **alors qu'un autre local éligible dans la marge
aurait dû gagner**.
**Preuve (traçage exact) :** candidats `[L1(ctx 1024, score 0.90), L2(ctx 32768, score 0.72),
R(ctx 128k, score 0.74)]`, `required_context=4000`, marge 0.05. `select` → R (seul éligible
avec L2 parmi les filtrés… cas général). `best_local` = L1 (0.90 > 0.72). `self_hosted_first
(L1,…)` → `false` (contexte insuffisant) → décision R. **Attendu :** L2 (0.72+0.05 ≥ 0.74) gagne.
Le correct est prouvé par le test `self_host_first_*` au niveau règle, mais **pas** au niveau
sélecteur (aucun test multi-locaux d'éligibilité mixte).
**Correctif :** filtrer l'éligibilité dans la boucle (même prédicat que `select`, l.85-90)
avant l'argmax ; ajouter le test mixte ci-dessus. (Constat annexe : à ex-aequo stricts,
le **premier** `SelfHosted` de l'ordre d'entrée gagne — déterministe à ordre fixé, mais
`select_with_local_preference` n'a pas le test de permutation dont `select` dispose, l.493-533.)

### MAJEUR

**M1 — `prune_denied` ne purge que les refus : `Pending` et `Approved` non consommés
s'accumulent sans borne.**
`omega-sandbox/src/approvals.rs:120-130`. `request()` (l.67-81) est non borné, sans TTL ;
`pending()` (l.108-118) n'expire rien ; un `Approved` jamais consommé reste éternellement.
Sur runs longs / backend qui auto-`request` à chaque `Prompt`, croissance mémoire
monotone → DoS lent. W9 §6 affirme « pas d'accumulation silencieuse » — **faux** pour
les deux autres états. **Correctif :** `prune_stale` (TTL sur `Pending`/`Approved`) ou file
bornée avec éviction documentée + test de saturation.

**M2 — Contexte local 32768 codé en dur, non prouvé ; `DEFAULT_LOCAL_CONTEXT` déjà
2× le défaut réel.**
`omega-server/src/pipeline.rs:20-25` (`local_record(model, Some(32768))`) suppose un
`num_ctx=32768` côté Ollama sans jamais l'interroger (`/api/show`) ni l'exiger en config.
Défaut réel d'Ollama : `num_ctx=4096` (pas 8192 comme l'affirme le commentaire
`ollama_catalog.rs:21-25` — « ordre de grandeur du défaut », 2× au-dessus). Pire :
`REQUIRED_CONTEXT=4000` (`config.rs:23`) passe à 96 tokens du défaut réel, et **aucun
garde-fou ne borne la taille du prompt** (`dto.rs:43-62` ne limite pas `content`) :
prompt > fenêtre → troncation/sliding silencieux côté Ollama, résumé « prouvé » sur
entrée tronquée. **Correctif :** `num_ctx` issu de `/api/show` ou config explicite
(fail-closed si inconnu → repli sur `DEFAULT_LOCAL_CONTEXT` documenté comme conservateur
réel ≤4096) + `max_content_len` sur `ApiRunRequest` + test.

**M3 — Aucun test HTTP end-to-end : handlers testés en appel direct uniquement.**
`omega-server/src/routes.rs:159-286` : tous les tests appellent `run(state, Json(req))` /
`trace(state, Path(...))` en direct. Jamais exercés via la stack Axum : routage
(`/api/trace/:run_id`), désérialisation JSON (champs inconnus — cf. m10), statuts réels,
et surtout **décodage `%`-URL**. Le test traversal `Path("..%2Fevil")` (l.232-235) passe
une chaîne **non décodée** (contient `%`, rejetée trivialement) : il ne prouve pas le
comportement réel (selon version Axum/hyper : 404 sur `%2F`, ou décodé en `../evil`
→ 400). La garde `is_valid_run_id` est saine (tout encodage décodé retombe sur `/`, `.`,
`%` — tous rejetés), mais **non prouvée de bout en bout**. **Correctif :** test
d'intégration via `tower::ServiceExt::oneshot` sur `create_router` avec requêtes HTTP
réelles (`/api/trace/..%2Fevil`, `POST /api/run` avec `run_id:"../evil"`, rôle inconnu).

**M4 — `debug_assert_eq!` tautologique et désactivé en release.**
`omega-app/src/lib.rs:181` : `id` (l.180) et `all_ids[index]` (l.127-132) sont dérivés de la
**même formule sur les mêmes données** — l'assert ne peut jamais échouer, et en release
(`debug_assert` compilé à vide) aucun invariant n'est vérifié. Si une refactorisation
future désynchronise les deux calculs, corruption silencieuse des clés d'idempotence.
**Correctif :** supprimer (bruit) **ou** promouvoir en `assert!` sur un vrai invariant
(ex. unicité des ids du plan).

**M5 — Blueprint §2.1 mélange réel et cible sans marquage, sous une note qui revendique
l'« état réel vérifié ».**
`docs/omega/reverse-eng/OMEGA-BLUEPRINT.md:72-100` liste 18 crates dont `omega-memory`,
`omega-tools`, `omega-cli`, `omega-ui-bindings` — **absents** de `omega/crates/`
(14 répertoires ; garde-fou : « 14 crates »). La note ajoutée par cet incrément affirme
que la colonne reflète « l'état réel vérifié (`Cargo.toml`, garde-fou vert) » : le
garde-fou mentionne ces crates en commentaires de règles futures (`check_dep_graph.py:39,80,82`),
pas comme existantes. → Violation RULES-OMEGA §4.2 (doc qui ment par omission).
**Correctif :** colonne `Statut` (`réel`/`cible`) ou scission du tableau.

**M6 — `InMemoryEventStore` ne valide pas le `run_id` ; `execute_run` non plus.**
`omega-app/src/store.rs:37-43` accepte tout (`../evil` → clé HashMap, inoffensif en
mémoire mais incohérent) ; `pipeline.rs:31-35` transmet `run_id` sans contrôle — la
validation n'existe que dans `FileEventStore` + `routes.rs`. Tout appelant direct
d'`App::run`/`execute_run` (binaire futur, tests) contourne la garde.
**Correctif :** valider dans `App::run` (refus typé central) ou documenter que la
validation est une responsabilité du bord (et tester `execute_run` avec `run_id:"../evil"`
sur store fichier → erreur, pas d'écriture).

### MINEUR

**m1 — Test no-op 1-candidat incomplet.** `selector.rs:658-671` ne couvre que le cas
« gagnant déjà local » ; pas le cas « 1 candidat distant » (branche `best_local=None`,
l.47-49), et n'assert que `model_id` (pas identité `score`/`reason` avec `select`).
**Correctif :** 1 test distant-single + `assert_eq!(d, raw)`.

**m2 — Transitions d'état d'approbation non gardées.** `approvals.rs:84-105` :
`deny()` sur un `Approved` en attente de consommation l'annule silencieusement ;
`approve()` sur un `Denied` le ressuscite. TOCTOU opérateur/backend.
**Correctif :** refuser les transitions depuis un état terminal (`false` si déjà
`Approved`/`Denied`, sauf `Pending`) + test.

**m3 — `OllamaProvider` ignore silencieusement `req.model`.** `ollama.rs:99-111` utilise
toujours `self.model`. Cohérent aujourd'hui (pipeline passe `decision.model_id` qui
égale `ollama/<model>`), mais divergence silencieuse si le registre câble un autre
modèle que la décision. **Correctif :** documenter (le provider local sert son modèle
configuré) ou `debug_assert`/warn en cas de divergence.

**m4 — `wire_to_message` : rôle inconnu → `User` silencieux.** `ollama.rs:58-66`
(`_ => Role::User`), incohérent avec la philosophie stricte du bord (`parse_role`,
`dto.rs:31-41`, rejette en 400). Un `role:"developer"` renvoyé par un proxy
OpenAI-compatible serait requalifié en `user` sans trace.
**Correctif :** conserver le rôle brut ou tracer ; au minimum tester le cas.

**m5 — Erreurs HTTP Ollama sans corps.** `ollama.rs:119-124` et `ollama_catalog.rs:187-192`
ne retiennent que le statut (ex. `404 model not found` d'Ollama — le cas le plus fréquent
en local — perd son message). **Correctif :** inclure `body.truncate(200)` comme le fait
déjà l'UI (`api.ts:26`).

**m6 — UI : une 400 saisie est affichée « Hors-ligne ».** `App.tsx:235` (`Banner offline`
sur toute erreur `getTrace`, y compris 400 `run_id` invalide) ; et `Cockpit` (l.62-72)
n'envoie jamais de `run_id` (`postRun(messages)` sans id, `api.ts:40-50`) → rejouabilité
depuis le Cockpit impossible, le panneau Traces démarre sur `live-1` quasi-sûr-absent
(200 vide, issue `unknown` — correct mais déroutant). **Correctif :** distinguer 4xx
(« saisie invalide ») de l'injoignable ; champ `run_id` optionnel dans Cockpit.

**m7 — W9 §6 range du fait sous « Reste ».** Le §6 « Reste (incrément 3) » liste
`prune_denied()` comme « déjà implémenté + testé » — à déplacer en §5 (auto-revue),
et à amender (cf. M1 : le problème d'accumulation n'est que partiellement traité).

**m8 — Re-run non-resume sur `run_id` existant : duplication silencieuse du journal.**
`omega-app/src/lib.rs:165-172` : si `recorded_plan` existe et `!resume`, le plan n'est ni
re-figé ni refusé — toutes les ops se ré-exécutent et appendent des `step.completed` en
double. La reprise ultérieure reste correcte (sémantique d'ensemble), mais la garantie
W3 §4.2 (« exactement une fois ») est violée en volume. **Correctif :** refuser
(`Config` « run déjà existant, utilisez resume ») ou documenter l'idempotence volume.

**m9 — `run()` ne valide pas `run_id` généré vs explicite de façon homogène + compteur
process-local (rappel N3/m3 de REVIEW-W3, toujours ouvert).** `lib.rs:124,305-309` :
`run-N` compteur `AtomicU64` → collision inter-process sur store partagé.
**Correctif :** inchangé depuis REVIEW-W3 — UUID/horodatage.

**m10 — `ApiRunRequest` sans `schema_version` alors que l'UI l'envoie.**
`dto.rs:22-27` vs `api.ts:45` (`schema_version` émis). Toléré (serde ignore l'inconnu),
jamais vérifié : dérive silencieuse possible si le protocole évolue.
**Correctif :** champ `#[serde(default)] schema_version` + contrôle d'écart, ou retrait
côté UI.

---

## §3 Cas reproduits / vérifiés

Exécuté (lecture seule) : `cargo test --workspace` → **247 passés, 0 échec** (13 suites
unitaires + doc-tests à 0) ; `cargo clippy --workspace --all-targets -- -D warnings` →
**0 warning** ; `check_dep_graph.py` → 14 crates, 0 cycle, 0 arête interdite.
`cargo fmt --check` non exécuté (hors mission ; aucun fichier modifié de toute façon).

Traçages adversariaux (relecture exacte, pas de modification) :

| # | Scénario | Résultat établi | Statut |
|---|---|---|---|
| B2 | `[L1(ctx1024,s.90), L2(ctx32768,s.72), R(s.74)]`, requis 4000, marge .05 | `best_local`=L1 → `self_hosted_first`=false → **R gagne, L2 éligible-dans-marge ignoré** | BUG avéré (code) |
| B1 | Ollama accept-connexion-sans-réponse + `POST /api/run` | aucun timeout client ni serveur → **worker bloqué indéfiniment** | BUG avéré (code) |
| M1 | 10⁶ `request()` sans `approve/deny/consume` | `entries` croît sans borne (aucun appel interne à `prune_denied`) | FUITE avérée (code) |
| M4 | `debug_assert_eq!(id, all_ids[index])`, `lib.rs:127-132` vs `:180` | même formule, mêmes données → **ne peut échouer** ; vide en release | Assert mort |
| M5 | `ls omega/crates` (14) vs blueprint §2.1 (18 lignes crate) | `memory/tools/cli/ui-bindings` **absents**, note « réel vérifié » | Écart doc avéré |
| OK | `a/b` vs `a_b` (`store.rs:93-99`) | `a/b` refusé avant tout I/O, `a_b` accepté → **pas de collision** (refus ≠ normalisation) | Non-bug ✓ |
| OK | Ex-aequo `best_local` (`s > bs`, l.43) | premier pris, `>` strict → **déterministe à ordre fixé** | Non-bug (cf. B2-note : pas de test de permutation) |
| OK | Winner introuvable (l.30-33) | inatteignable (`select` ne rend qu'un id issu de `candidates`) — défensif mort mais inoffensif | Non-bug ✓ |
| OK | 1 candidat local (`pipeline.rs:51-54`) | `winner.tier==SelfHosted` → retour `decision` inchangée — **no-op prouvé** (test l.658-671) | Non-bug ✓ |
| OK | Legacy sans `plan.defined` admis (`lib.rs:154-164` + test l.936-962) | reprise sans contrôle — compromis **documenté et testé** | Accepté ✓ |
| OK | `encodeURIComponent` UI (`api.ts:53`) + validation serveur (`routes.rs:139-143`) | toute forme décodée (`/`, `.`, `%`, `..`) rejetée par le charset | Garde saine, mais preuve e2e manquante (M3) |

---

## §4 Tests manquants

1. **Timeout** : serveur muet + `OllamaProvider::complete` / `OllamaCatalogSource::fetch`
   → erreur typée avant deadline (couvre B1).
2. **Multi-locaux d'éligibilité mixte** : L1 inéligible-score-haut + L2 éligible-dans-marge
   → L2 gagne (couvre B2) ; + permutation d'ordre à ex-aequo (déterministe).
3. **Distant-single + identité décision** : 1 candidat distant → `select_with_local_preference
   == select` (`model_id`+`score`+`reason`) (couvre m1).
4. **Intégration HTTP réelle** (`oneshot` sur `create_router`) : traversal encodé, `run_id`
   JSON invalide, rôle inconnu → statuts 400/200 Corps versionnés ; run inconnu → 200
   `unknown` (couvre M3).
5. **Saturation `ApprovalQueue`** : N `request()` sans décision → borne/TTL prouvée
   (couvre M1) ; transitions `approve→deny`, `deny→approve` refusées (couvre m2).
6. **`execute_run` avec `run_id:"../evil"` sur store fichier** → erreur typée, 0 fichier
   créé (couvre M6).
7. **Re-run non-resume sur `run_id` existant** → refus ou journal sans doublons
   (couvre m8).
8. **Prompt > contexte** : contenu démesuré → 400 (couvre M2) ; `num_ctx` réel vs 32768
   documenté/testé via `/api/show` mocké.

---

## §5 Points solides

- **Traversal réellement bouché en profondeur** : charset strict `store.rs:93-99`, refus
  avant I/O + test « sans toucher le disque » (`store.rs:262-274`), validation miroir
  dans les deux handlers (`routes.rs:122-128,139-143`) avec tests négatifs dédiés.
- **Garde plan bien conçue** : `plan.defined` + refus typé (jamais de saut silencieux),
  `last()` via `rev()` correct en append-only, doublons homonymes couverts (`name#index`,
  test `duplicate_operation_names_...`), legacy admis explicitement + testé.
- **Règle `self_hosted_first` exacte et bien testée** au niveau règle (8 tests :
  marge, égalité stricte, contexte insuffisant, capacité manquante, distant inéligible,
  tier non-local) ; docstring qui énonce la règle formellement (`ollama_catalog.rs:140-172`).
- **Qualité d'exécution** : 247 tests verts, clippy `-D warnings` silencieux, erreurs
  typées partout (jamais de `unwrap` sur chemins I/O/réseau des lib), `stream:false`
  imposé et documenté, usage mappé sans perte (`prompt/completion_tokens`).
- **Docs honnêtes là où ça compte** : W3 §4.4 décrit la garde anti-dérive telle
  qu'implémentée ; `is_valid_run_id` documente la menace (`store.rs:87-92**) ; `prune_denied`
  admet son appel manuel (pas de magie).
- **UI robuste par construction** : `encodeURIComponent`, 3 états par panneau, repli
  fixtures explicite, `aria-label`/`role=status|alert` sur formulaires et bandeaux (a11y
  de base présente — à noter : aucun `alt` manquant car aucune image ; pas de strings
  non-i18n bloquants à ce stade mais **aucun wrapper i18n** — dette G2 assumée Phase 1).

---

## Métriques

- Fichiers audités : 11 (`selector.rs`, `ollama_catalog.rs`, `app/{lib,store}.rs`,
  `server/{pipeline,routes,dto,config}.rs`, `sandbox/approvals.rs`, `providers/ollama.rs`,
  `ui/{api.ts,App.tsx}`) + 3 docs (W3, W9, blueprint).
- Bloquants : **2** · Majeurs : **6** · Mineurs : **10**.
- Violations TDD (tests manquants) : 8 (§4).
- Violations sécurité : 1 bloquante (timeout/dispo, B1) ; traversal **OK** ; secrets : 0.
- Violations accessibilité/i18n : 0 bloquante (a11y de base OK ; i18n : dette assumée, pas de wrapper).
- Commandes : `cargo test --workspace` → 247 OK / 0 KO ; clippy `-D warnings` → OK ;
  `check_dep_graph.py` → 14 crates, 0 cycle.

---

## Addendum — écriture concurrente constatée pendant l'audit (hors périmètre)

Pendant la rédaction (arbre propre au début : `git status` vide après `f84d4b89`), deux
fichiers audités ont été **modifiés par un tiers non coordonné, sans commit** :
`omega-providers/src/ollama.rs` (+32/-1 : `DEFAULT_HTTP_TIMEOUT=180s`, `with_timeout()`,
test de construction) et `omega-providers/src/ollama_catalog.rs` (+6/-1 : timeout 30s
dans `new()`). Ces changements **répondent partiellement à B1** mais sont **hors audit**
(non relus, non testés par mes soins — mes 247 tests/clippy portent sur l'état commité).

- Violation process : RULES-OMEGA §5.2 (« un seul agent écrit par arbre ») — même classe
  que l'incident 2026-10-15 cité dans les règles.
- Réserves sur le fond (à vérifier par le Reviewer du correctif) : (a) `.expect()` dans
  un constructeur de lib (panique vs erreur typée — contraire aux conventions du workspace) ;
  (b) aucun test d'expiration réelle (construction seule, zéro réseau — le hang reste non
  prouvé-couvert) ; (c) `fetch()` catalogue toujours sans `connect_timeout` distinct ;
  (d) B2, M1–M6, m1–m10 non adressés par ce diff.
- Recommandation : faire relire/commit ces changements dans une branche/worktree dédié,
  puis re-jouer `cargo test` + clippy avant toute revendication de clôture de B1.
