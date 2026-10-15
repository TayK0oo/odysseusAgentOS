# MEMORY-OMEGA — Mémoire persistante de la session

> Mémoire durable, versionnée dans Git. **Source de vérité** pour reprendre la session
> à froid. Mettre à jour à chaque décision / découverte. Tags de provenance :
> `[stated]` (dit par l'utilisateur) · `[observed]` (vérifié) · `[inferred]` (déduit) ·
> `[todo]` (à faire) · `[blocked]` (en attente).

---

## 1. Identité de la session

| Champ | Valeur |
|---|---|
| Branche de travail | `feat/agentos-omega` `[observed]` |
| Branche de base | `feat/inventaire-global-v1` `[observed]` |
| **Répertoire de travail** | **`/home/tayk0oo/Projets/odysseusAgentOS-omega`** (worktree isolé) `[observed]` |
| Dépôt principal | `/home/tayk0oo/Projets/odysseusAgentOS` (reste sur `feat/inventaire-global-v1`) `[observed]` |
| Dépôt distant | `github.com/TayK0oo/odysseusAgentOS` (fork de `arcahyadi/odysseus`) `[observed]` |
| Date d'ouverture | 2026-10-15 `[observed]` |
| Modèle orchestrateur | DeepSeek V4.1 Flash (opencode-go) `[stated]` |

## 2. Constat du dépôt de base (état au fork) `[observed]`

- 1071 fichiers `.py`, 136 modules `src/`, 63 routes, ~503 endpoints.
- SFD v3.1 : 25 principes · 22 modules · 21 UC · 21 NF (`docs/master-ref/01-SFD-v3.1.md`).
- Couverture code : **6/22 principes actifs**, **6/19 UC actifs**. Environnement non exécutable (pas de venv).
- Modularité : god node `core.database` (64 deps), `route_loader` non dynamique (58 imports), 71 shims legacy.
- Vision existante : 5 couches (CONNAISSANCE → EXÉCUTION → DESIGN → OBSERVABILITÉ → GOUVERNANCE),
  8 axes, Constitution en 13 étapes (`docs/VISION-INTEGRATION.md`).
- Config OpenCode : plugins TS `@agentos/sfd-*`, agents `sfd-orchestrator/executor/planner/reviewer`.
- UI : `static/` (HTML/CSS/JS) — pas de React/TS.

## 3. ⚠️ Concurrence détectée (isolation en place) `[observed]`

- Un **agent concurrent** écrivait dans le répertoire principal pendant cette session
  (refactor « MOD-5 AppContext » sur 30 fichiers `src/*.py` + `src/app_context.py` +
  `tests/test_sprint5_mod5_appcontext.py`, écrits entre 00:42 et 00:47).
- **Cause probable :** une autre session OpenCode travaillant sur le socle.
- **Action :** travail Omega isolé dans un `git worktree` dédié ; répertoire principal
  restitué à `feat/inventaire-global-v1` avec ses modifications intactes.
- **Conséquence :** ne jamais opérer Omega depuis le dépôt principal.

## 4. Recherche — corpus TikTok `@0xloucash` `[observed]`

- Profil : **Loucash**, 235K abonnés. Bio : « Openclaw / Tech / Polymarket ».
- **78 vidéos** énumérées (`yt-dlp`), ≈2,96 M vues cumulées, période ~2026-03 → 2026-10.
- Fichiers : `docs/omega/research/loucash/` (`videos.json`, `videos_enriched.json`,
  `classified.json`) + vault `obsidian-vault/loucash/` (78 notes + 15 MOC + `_INDEX.md`).
- Top catégories : claude-tooling (10), skills-plugins (9), web-design (9), ai-agents (8),
  automation (6), business-monetization (5), free-alternatives (5), github-repos (5)…
- Pistes clés pour Omega : Hermès (vidéo #25), 1 plugin = 68 agents/300 skills (#14),
  system prompt Claude fuité (#30), repo de reverse-engineering (#3), contournement des
  limites Claude Code via modèles OSS (#27), mémoire + skill auto-évolutive (#26),
  second cerveau Obsidian (#50), alternatives gratuites installées par prompt (#5).

## 5. États de l'art produits `[observed]`

| Doc | Contenu |
|---|---|
| `docs/omega/research/HARNESS-LANDSCAPE.md` | 25 harnais, patterns plugins, **top-10 features à reverse-engineer** |
| `docs/omega/research/MODEL-ROUTING-BENCHMARKS.md` | Sources de benchmarks, composite de score, taxonomie de tâches, architecture routeur |

## 6. Décisions (log)

| # | Décision | Date | Statut | ADR |
|---|---|---|---|---|
| D1 | Nouvelle branche dédiée à la vision « ultime » | 2026-10-15 | ✅ | — |
| D2 | Nom de branche `feat/agentos-omega` | 2026-10-15 | ✅ | — |
| D3 | Ancrage doc `docs/omega/` ; règles `.opencode/rules/` | 2026-10-15 | ✅ | — |
| D4 | Recherche classée dans `docs/omega/research/` + vault Obsidian | 2026-10-15 | ✅ | — |
| D5 | **Moteur : réécriture complète en Rust** | 2026-10-15 | ✅ `[stated]` | ADR-0001 (à écrire) |
| D6 | **UI : nouvelle app React/TS à côté** (pas de refonte incrémentale) | 2026-10-15 | ✅ `[stated]` | ADR-0002 (à écrire) |
| D7 | **Premier chantier de code : W5 reverse-engineering** | 2026-10-15 | ✅ `[stated]` | — |
| D8 | Isolation du travail Omega via git worktree (concurrence) | 2026-10-15 | ✅ `[observed]` | — |
| D9 | Correction `REVIEW-W2` : garde-fou durci + quarantaine scopée + type↔capacité + factory câblée + annulation intra-étape | 2026-10-15 | ✅ `[observed]` | ADR-0003 |
| D10 | Réconciliation dette W4 : `TaskType`/`RoutingRequest`/`Capabilities` dans `omega-types`, port `TaskClassifier` dans `omega-ports`, `omega-router` implémente et ré-exporte | 2026-10-15 | ✅ `[observed]` | — |
| D11 | Correction REVIEW-W4 : normalisation Unicode (B1), priorité du classifieur (B2), fallback sans ping-pong (M1), cache versionné (M2), ports `ScoringPolicy`/`Selector`/`FallbackPolicy` dans `omega-ports` (M3), M4 documenté comme non-objectif | 2026-10-15 | ✅ `[observed]` | — |
| D12 | W3 moteur + exécution durable : `omega-app`, `EventStore` (mem+fichier JSONL), reprise idempotente, loader config — 103 tests | 2026-10-15 | ✅ `[observed]` | — |
| D13 | Correction REVIEW-W3 : ids uniques par opération (B1), exclusivité slot cross-voies (B2), `FileEventStore` synchronisé (B3), `TypeMismatch` dyn (M1), tri global `active_ids` (M2), contrat ports=dyn documenté (M3) — 109 tests | 2026-10-15 | ✅ `[observed]` | — |
| D14 | W6 incrément 1 : crate `omega-context` (estimateur, budget, moteur par défaut, tool search BM25 + rarest-token gate, skills 3 niveaux, compaction) + variante `OmegaError::BudgetExceeded` + garde-fou `adapters → registry/plugin-api` — 29 tests | 2026-10-15 | ✅ `[observed]` | — |
| D15 | Correction REVIEW-W6 : `Default` de `TokenEstimator` sûr (B1), `over_budget` + `BudgetExceeded` sur `assemble` (M1), départage lexical de la gate (M2), pas de marqueur mensonger en compaction (M3), M4 (blueprint §2.1) documenté en dette — 39 tests `omega-context` | 2026-10-15 | ✅ `[observed]` | — |

## 7. Credentials & accès

Annoncés par l'utilisateur comme fournissables `[stated]` — à déposer dans un `.env`
**non versionné** (jamais dans Git). Statut : `[blocked]` tant que non fournis.

| Ressource | Variable | Statut | Usage |
|---|---|---|---|
| OpenCode ZenRouter | `OPENCODE_API_KEY` | ✅ déposé + validé `[observed]` | Routage moteur principal |
| GitHub | `GITHUB_TOKEN` | ✅ déposé + validé `[observed]` | Clones, reverse-eng, rate-limit |
| HuggingFace | `HF_TOKEN` | ✅ déposé + validé `[observed]` | Benchmarks, modèles |
| TikTok (session) | cookies `sessionid` | `[todo]` non fourni | Transcription / scraping profond |
| Compute élastique | `MODAL_TOKEN` / SSH | `[todo]` non fourni | Exécution à la demande |
| Clés LLM multi-fournisseurs | `OPENAI/ANTHROPIC/…` | `[todo]` non fourni (abonnement Go utilisé) | Registre routeur |

## 8. Prochaines étapes (backlog vivant)

- [x] Enrichir les 78 vidéos (dates, counts, hashtags)
- [x] Classification riche + vault Obsidian
- [x] Paysage des harnais + benchmarks/routing
- [x] ADR-0001 (Rust) · ADR-0002 (UI React) · ADR-0003 (architecture plugin)
- [x] `00-VISION-OMEGA.md` (vision cible)
- [x] **W5 — fiches RE** : `RE-OPENCLAW-HERMES.md` (975 l.) · `RE-GOOSE-CODEX.md` (695 l.) · `RE-OPENCODE-CLAUDECODE.md` (396 l.)
- [x] **W5 — consolidation** : `FEATURE-MATRIX.md` (96 features) · `OMEGA-BLUEPRINT.md` (18 crates) · `features.json`
- [x] **W2 — incrément 1** : workspace `omega/` + crates `types/ports/config/registry/plugin-api`, 13 tests verts, clippy propre
- [x] **W2 — incrément 2** : crates `omega-core` (boucle d'agent) + `omega-protocol` (wire types), `omega/scripts/check_dep_graph.py` (graphe + garde-fou), 25 tests verts, clippy/fmt propre
- [x] **W2 — revue indépendante** : `REVIEW-W2.md` (B1/B2/M1-M4/m1-m5/n1-n5) corrigée — 33 tests verts, `--self-test` OK
- [x] **W4 inc1 — routeur logique pure** : `omega-providers` + `omega-router` (classifieur déterministe, scoring, Pareto, fallback, cache) — 86 tests, ports réconciliés
- [x] **W4 — revue + correctifs + re-vérification** : `REVIEW-W4.md` (NO-GO 5/10 → GO avec réserves 7/10, N1/N2/N3 corrigés)
- [x] **W3 inc1 — moteur + exécution durable** : `omega-app` (App, run/reprise idempotente), `EventStore` (mem + JSONL), loader config — 109 tests
- [x] **W3 — revue + correctifs + re-vérification** : `REVIEW-W3.md` (NO-GO 5/10 → GO avec réserves 7/10, B1/B2/B3/M1/M2/M3 corrigés, N1-N4 notés)
- [x] **W6 — incrément 1** : crate `omega-context` (estimateur, budget, `DefaultContextEngine`, tool search BM25 + rarest-token gate, skills 3 niveaux, compaction) + `ContextContributor` — 29 tests
- [x] **W6 — revue + correctifs + re-vérification** : `REVIEW-W6.md` (GO 8/10, B1/M1/M3 corrigés, M2/M4 partiels documentés, **N1 TOCTOU corrigé** : `assemble_sync` retourne `(résultat, rapport)` + test de concurrence) — 40 tests context, **149 workspace**
- [x] **W8 inc1 — traçabilité** : `omega-otel` (provenance, `MetricName`, `StdoutSink`/`FileSink`, `TraceReplayer`, `RunSummary`, `OtelContributor`)
- [x] **W9 inc1 — sandbox** : `ExecDecision`/`ExecPolicy`, `RuleBasedExecPolicy` (deny-by-default), `LocalSandboxBackend` (timeout + kill), `SandboxContributor` — **169 tests workspace**, clippy/fmt/graphe verts (13 crates)
- [ ] **W8/W9 — revue indépendante** (en attente quota Go)
- [x] **W8/W9 — correctifs B1/B2/B3 + majeurs** : garde anti-composition (`has_unquoted_metachars`, Allow+metachars→Prompt), kill de groupe (`process_group` + `kill -9 -pgid`, test petits-fils), replay fidèle (`ignored_events`, `outcome_conflict`, premier marqueur gagne), `FileSink.emit_errors`, `MetricName` (`-`, 3-8 segments) — **176 tests**, clippy/fmt/graphe verts
- [x] **W8/W9 — re-vérification** : `REVIEW-W8-W9.md` (**GO conditionnel 8/10**, B1/B2/B3 corrigés vérifiés par 20+ cas, N1 `setsid` documenté, **N3 corrigé** : `app→router` autorisé dans le garde-fou, fmt/clippy exemple) — **180 tests**
- [x] **W2 inc3 — provider réel** : `OllamaProvider` + exemple `live_ollama` (**LIVE-OK** : T2→ollama/qwen2.5-coder:7b, 2/2 étapes, journal 6 événements, coût nul)
- [x] **W7 inc1 — scaffold UI** : `omega/ui` (React/TS + Vite, 5 panneaux, `protocol.ts` miroir, `tsc` vert)
- [x] **W9 inc2 — parse strict + file d'approbations** : casse normalisée, `strict()` (substitutions→Forbidden), `ApprovalQueue` à usage unique, backend `with_approvals` (mergé, paragraphe parallèle)
- [x] **W10 inc1 — catalogue local + self-host-first** : `OllamaCatalogSource` (`/api/tags`→records), `local_record` (coût 0, tier SelfHosted), règle `score_local + 0.05 >= score_distant` (mergé, paragraphe parallèle)
- [x] Branche publiée sur `origin/feat/agentos-omega`
- [x] **W7 inc2 — omega-server + UI branchée** (mergé, paragraphe parallèle) : axum (health, plugins, run, trace), UI sur `/api` (loading/erreur + repli fixtures), **LIVE vérifié** : `POST /api/run`→T14→local 0.685→2/2 OK→replay cohérent
- [x] **Revue indépendante `REVIEW-FINAL-2` (AJOURNÉ 6/10) + correction optimale** : B1 timeouts HTTP (180 s provider / 30 s catalogue + test d'expiration réelle), B2 éligibilité dans la préférence locale (+ test), `InMemoryEventStore` validé, `remove`/`len`/`is_empty` file, test traversal décodé, `debug_assert` retiré, blueprint §2.1 (crates cibles vs existants), note `OLLAMA_NUM_CTX`, exemple à id unique — **251 tests**, tout vert
  - Commits : `63827d87` (correctifs, committé par l'utilisateur depuis l'arbre de travail) + `04ae01fa` (complément : import de test, id unique exemple, mémoire). `[observed]`
- [x] **W4 inc3 — circuit breaker à états** : Closed→Open→HalfOpen→Closed, `record_success` (M4 implémenté), horloge injectable, sonde unique par run (mergé, paragraphe parallèle)
- [x] **W6 inc2 — stemming déterministe** : règles R0–R7 commentées, appliqué docs+requêtes, `file_write` rappelé via seuil (gate stricte inchangée, prouvé) (mergé, paragraphe parallèle)
- [x] Décomposition W0-W10 (`docs/omega/02-WORKSTREAMS.md`)
- [x] Credentials déposés et validés (`.env`, non versionné)
- [x] Subagents : Go épuisé puis `opencode/*` payants épuisés → **routage 100% Free** (`muse-spark-1.3-contributor-free`, `mimo-v2.6-flash-free`, replis `step-5`/`space-bunny`)
- [x] Parallélisme propre : 1 worktree + 1 branche par agent (`omega-par/*`, `feat/omega-*`), zones de fichiers disjointes, `MEMORY.md` consolidée par l'orchestrateur
- [x] **Bascule provider `opencode` (défaut)** : quota Go épuisé → sonde OK sur `opencode/mimo-v2.6-flash-free` ; MAIS modèles payants `opencode/*` aussi à sec (`Insufficient account funds`) → **routage 100% Free** : `opencode/muse-spark-1.3-contributor-free` (revues), `opencode/mimo-v2.6-flash-free` (exécution), replis `step-5-preview-free`, `space-bunny-free`
- [x] Self-hosting local : `ollama pull qwen2.5-coder:7b` (4,7 Go) + inférence vérifiée via API (`/api/generate` OK) — amorce W10 `[observed]`

### Dettes architecturales résolues

- [x] W4 — doublon `TaskType` (`omega-types` vs `omega-router`) et `TaskClassifier`
  défini dans `omega-router` au lieu de `omega-ports` : réconcilié ; source unique
  `omega-types`, port dans `omega-ports`, implémentation dans `omega-router`.
  Preuves : `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo fmt --all --check`, `python3 omega/scripts/check_dep_graph.py` et `--self-test`
  tous verts (`[observed]`).
- [x] REVIEW-W4 — B1/B2/M1/M2/M3 corrigés ; M4 documenté comme non-objectif.
  - `ModelRecord`/`QualityAxes`/`ModelTier` déplacés dans `omega-types` ;
  - ports `ScoringPolicy`/`Selector`/`FallbackPolicy` définis dans `omega-ports` ;
  - `RouterPolicy` orphelin supprimé ;
  - classifieur normalisé NFD + strip marks et priorité explicite ;
  - fallback mémorise les modèles déjà essayés (pas de ping-pong) ;
  - cache inclut `registry_version`.
  Preuves : 45 tests router passants, clippy/fmt/dep graph verts (`[observed]`).

### Clones de travail (nettoyables)
`/tmp/opencode/re/` : codex (131M), goose (646M), hermes-agent (344M), openclaw (939M), opencode (230M).

### Incident d'orchestration (2026-10-15) `[observed]`
Deux exécuteurs subagents ont écrit **dans le même arbre** de travail (`omega/`) en parallèle :
l'exécuteur « N1/N2 » a supprimé les artefacts en cours de l'exécuteur « W4 » (`omega-router/`)
pour débloquer ses commandes. Le fichier `W4-routing.md` est susceptible d'avoir été affecté.
**Règle ajoutée** (RULES-OMEGA §5.2) : un seul agent en écriture par arbre ; parallélisme =
un worktree git par agent.

## 9. Pointeurs utiles

| Sujet | Fichier |
|---|---|
| Règles session | `docs/omega/rules/RULES-OMEGA.md` |
| Vision cible | `docs/omega/00-VISION-OMEGA.md` |
| Workstreams | `docs/omega/02-WORKSTREAMS.md` |
| Reverse-engineering | `docs/omega/reverse-eng/` |
| Recherche | `docs/omega/research/` |
| ADR | `docs/omega/adr/` |
| Vision existante | `docs/VISION-INTEGRATION.md` |
| SFD de référence | `docs/master-ref/01-SFD-v3.1.md` |
