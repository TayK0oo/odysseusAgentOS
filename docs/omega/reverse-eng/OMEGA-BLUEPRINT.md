# OMEGA-BLUEPRINT — Synthèse d'architecture issue du reverse-engineering (W5)

> **Workstream :** W5 (clôture) · **Date :** 2026-10-15 · **Branche :** `feat/agentos-omega`
> **Objet :** transformer les 96 features consolidées (`FEATURE-MATRIX.md`) en une
> **architecture cible** : workspace Rust, catalogue de capacités/ports, boucle d'agent,
> Context Engine, routeur, traçabilité, décisions et ordre de construction.
> **Sources :** `RE-OPENCLAW-HERMES.md`, `RE-GOOSE-CODEX.md`, `RE-OPENCODE-CLAUDECODE.md`,
> `HARNESS-LANDSCAPE.md`, `MODEL-ROUTING-BENCHMARKS.md`, `00-VISION-OMEGA.md`,
> `rules/RULES-OMEGA.md`.
> **Provenance :** architecture `[inferred]` (synthèse de l'auteur à partir de `[observed]`/`[stated]`) ;
> chaque affirmation de mécanisme renvoie à une ref de fiche.

---

## §1 — Les leçons structurantes du RE (ce qui revient partout)

1. **Le noyau ne connaît que des ports ; le plugin est un manifeste + des capacités + deux verbes dans un Scope.**
   OpenClaw décrit le comportement en schéma **lu sans exécuter le code** (`RE-OH:725-735`) ;
   OpenCode réduit l'extension à `transform` (état rejouable) + `hook` (live) + `reload`
   (`RE-OC:68-78`) ; Codex enregistre des traits « contributeurs » dans un builder qui devient
   **immuable** après `build()` (`RE-GC:282-331`). → *Omega : registre de capacités, manifestes
   schema-validés, surface d'extension minuscule.* `[inferred]`

2. **Deux régimes de capacité, et un échec de plugin n'arrête jamais le noyau.**
   Slot **exclusif** (`context-engine`, `memory`) vs **ensemble** (`llm-provider`, `tool`) —
   OpenClaw et ADR-0003 (`RE-OH:271-301`) ; repli déterministe sur `legacy` en cas d'échec
   (`RE-OH:703` ; `HL:158` ; `RE-OC:286`). → *Omega : `slot exclusif | ensemble` + quarantaine.*

3. **La boucle d'agent est une machine d'opérations ordonnées, paramétrée par des traits**
   (`Operation`/`Inference`/`EffectHandler`/`Emitter`), **agnostique du provider et de
   l'extension**, qui répare la conversation après annulation (`RE-GC:89-152`). → *Omega : le
   squelette du noyau ; l'ordre des étapes encode la priorité (hooks avant inférence).*

4. **Le contexte (compaction incluse) est un adaptateur, pas du code dans la boucle.**
   Goose : `CompactingProvider` = décorateur de `Provider` (`RE-GC:240-261`) ; OpenClaw :
   port `ContextEngine` avec `ownsCompaction` (`RE-OH:136-223`) ; OpenCode : transforms/hooks
   de contexte (`RE-OC:302`). → *Omega : toute l'économie de contexte est branchée sur le port
   `ContextEngine`.*

5. **La configuration est à couches, avec provenance et précédence *testable*.**
   Codex : `ConfigLayerSource::precedence() -> i16`, profils nommés, merge récursif, schéma JSON
   généré (`RE-GC:382-398,516-521`). → *Omega : `layer_source` + version exposés sur chaque
   décision (traçabilité gratuite).*

6. **L'économie de contexte est structurelle, pas cosmétique.**
   Tool search BM25 + rarest-token gate (`RE-OH:430-508`), progressive disclosure des skills
   (`RE-OH:780-788`), prompt caching par préfixe ≤4 breakpoints + affinité parent/fork/self
   (`RE-OC:243-246`), budget avant action (`RE-OC:248-252`), isolation par sous-agent
   (`HL:196-200`). → *Omega : 5 leviers de contexte, tous déterministes côté système.*

7. **La frontière Rust ↔ TS est *générée*, jamais maintenue à la main, et le schéma est versionné.**
   Codex `ts-rs` + fixtures (`RE-GC:468-473,579-583`) ; Goose wire types ACP source unique
   (`RE-GC:280`) ; enveloppe `schemaVersion: 1` d'OpenClaw (`RE-OH:248-269`). → *Omega :
   `omega-protocol` génère les types TS de l'UI.*

8. **Tout est événement traçable : métriques nommées, cibles log-only/trace-safe, replay.**
   Codex `codex.<domaine>.<objet>` + séparation PII (`RE-GC:537-541,592`) ; Claude Code
   spans hiérarchiques + gates OTel (`RE-OC:192-201`) ; OpenCode événements durables vs
   éphémères (`RE-OC:123,315`). → *Omega : event-bus durable + provenance + replay.*

---

## §2 — Workspace Rust final d'Omega

**Principe directeur : « le grain de Goose, la discipline de Codex » (`RE-GC:652-676`),
enrichi par les ports d'OpenClaw/Hermes.** Goose donne le bon **grain** (1 crate de types,
1 crate de boucle, 1 crate de contexte, N crates de plugins, 1 crate applicatif) ; Codex donne
la **discipline** (un crate par capacité de premier plan dès qu'elle a une frontière de
crash/perf). Anti-patterns évités : monolithe `core`/`app` (`RE-GC:83,621`) et granularité
extrême (~200 crates, `RE-GC:622`).

### 2.1 Liste des crates (18 + plugins + ui)

> **Note de maintenance (2026-10-15) :** la colonne « Dépendances internes » reflète
> l'état **réel vérifié** (`Cargo.toml`, garde-fou `check_dep_graph.py` vert) —
> les adaptateurs dépendent aussi de `registry`/`plugin-api` pour leurs
> `Contributor`s (leçon du câblage `omega-context`, revue W6).

| Crate | Responsabilité | Dépendances internes |
|---|---|---|
| `omega-types` | **Types purs** : `Message`, `Conversation`, `ModelConfig`, `Usage`, IDs, **erreurs de domaine**. Zéro logique applicative, zéro dépendance interne. | — (feuille) |
| `omega-ports` | **Traits/ports fins uniquement** (ISP) : `Provider`, `ContextEngine`, `MemoryStore`, `ToolExecutor`, `SandboxBackend`, `RouterPolicy`, `CatalogSource`, `TelemetrySink`, `SubagentRunner`, `Hook`, `PermissionPolicy`… Aucune implémentation. | `types` |
| `omega-config` | Config **à couches** + profils + `precedence()` testable + merge TOML récursif + générateur de **schéma JSON** (`schemars`, `deny_unknown_fields`). | `types` |
| `omega-protocol` | **Wire types source unique** (JSON-RPC/ACP) → exports **TS générés** (`ts-rs`) + fixtures de schéma. | `types` |
| `omega-core` | **Boucle d'agent** : `Operation`/`Inference`/`Effect`/`Emitter`, `EffectHandler`, `RunStatus`, **event-bus interne**, **annulation coopérative**. Ne connaît ni provider ni extension. | `types`, `ports` |
| `omega-registry` | **Registres ordonnés et déterministes** (IndexMap + priorité) + **slots** (exclusif/ensemble) + manifestes + DI par constructeur + quarantaine/repli. | `types`, `ports`, `config` |
| `omega-plugin-api` | **Traits contributeurs** de cycle de vie (builder → registre immuable, défauts vides, chaîne de responsabilité) ; chargeur de manifestes + policy d'installation. | `types`, `ports`, `registry` |
| `omega-context` | **Context Engine** (port + `ownsCompaction`), compaction (décorateur + structurée), prompt caching, **tool search BM25**, progressive disclosure des skills, découverte `AGENTS.md`, budget de contexte. | `types`, `ports`, `registry`, `plugin-api` |
| `omega-memory` | **Mémoire bornée + snapshot gelé**, table transactionnelle, **FTS5 session search**, mémoire auto générée. | `types`, `ports`, `config` |
| `omega-tools` | **Registre d'outils** + tool executor/exposure (`spec`/`handle` séparés) + index de recherche. | `types`, `ports`, `config` |
| `omega-providers` | **Providers déclaratifs JSON** + adaptateurs par API + **catalog sources** (OpenRouter, models.dev, Ollama, HF, `/v1/models`). | `types`, `ports` |
| `omega-router` | **Registre de modèles**, classifieur de tâche déterministe, scoring composite, sélecteur/Pareto (+ préférence self-hosted), fallback + circuit breaker, feedback bandit, cache de décision. | `types`, `ports`, `providers` |
| `omega-sandbox` | **Politiques de sandbox typées** + backends Linux (Landlock+seccomp, bubblewrap) + execpolicy + approbations + checkpoints/worktrees. | `types`, `ports`, `registry`, `plugin-api` |
| `omega-otel` | **Télémétrie nommée** + cibles log-only/trace-safe + **provenance** + event store durable + **replay**. | `types`, `ports`, `registry`, `plugin-api` |
| `omega-app` | **Assemble** : enregistre les plugins/capacités, câble les adaptateurs, expose le daemon/runtime. | `types`, `ports`, `config`, `registry`, `plugin-api`, `core` |
| `omega-server` | **Adaptateur de bord** : API HTTP/WS pour l'UI (schéma versionné). | `app`, `protocol` (+ `types`, `ports`, `registry`, `plugin-api`, `core`, `context`, `otel`, `router`, `providers` pour le câblage) |
| `omega-cli` | Binaire CLI. | `app` |
| `omega-ui-bindings` | Frontière Rust↔React/TS (générée depuis `protocol`). | `protocol` |
| `plugins/` | Un crate par capacité (ex. `harness-*`, `skills`, `hooks`, `memories`, `subagents`, `tools-*`). | `ports` (+ `types`, `registry`) — **jamais** `core`/`app` |
| `ui/` | App React/TS (Vite + TS). | `server` (via protocole typé) |

### 2.2 Sens des dépendances (zéro cycle)

```
 L0  omega-types
 L1  omega-ports ─┐   omega-config ─┐   omega-protocol
 L2  omega-core ──┤   omega-registry┴──┐   omega-plugin-api
 L3  omega-context · omega-memory · omega-tools · omega-providers · omega-sandbox · omega-otel
 L4  omega-router
 L5  omega-app
 L6  omega-server · omega-cli · omega-ui-bindings
      plugins/ ──► ports (+ types, registry)        ui/ ──► server
```

**Règles (garde-fous CI, à encoder dans les `Cargo.toml` + un test de graphe) :**

- `ports → types` uniquement ; `core → ports` ; **jamais** `core → adaptateur`.
- Un crate d'**adaptateur** (`context`, `memory`, `tools`, `providers`, `sandbox`, `otel`)
  dépend de `ports`/`types`/`config`, **jamais** de `core` ni de `app`.
- `plugins → ports` (jamais l'inverse). `app → tout` ; `cli/server/ui-bindings → app/protocol`.
- **Zéro cycle** : vérifié par un test qui construit le graphe de dépendances (cf. rule
  « zéro dépendance circulaire », `RULES-OMEGA.md:26-27`).

### 2.3 Phasage du workspace

- **Phase 1 (noyau, ~11 crates)** : `types`, `ports`, `config`, `protocol`, `core`, `registry`,
  `plugin-api`, `context`, `tools`, `providers`, `app`, `cli`.
- **Phase 2 (fracture justifiée par frontière de crash/perf)** : `memory` (SQLite/FTS5),
  `router` (I/O catalogues), `sandbox` (backends lourds), `otel` (I/O télémétrie),
  `server` + `ui-bindings` (bord UI).

> **Réconciliation avec `00-VISION-OMEGA.md` §4.1** : `omega-runtime` → `omega-core` ;
> `omega-plugins` → `omega-registry` + `omega-plugin-api` ; `omega-telemetry` → `omega-otel` ;
> `omega-memory`/`omega-tools`/`omega-router`/`omega-context` conservés ; `omega-server` conservé.
> *(`[inferred]` — la fusion `registre`/`plugin-api` et le renommage `otel` suivent la
> leçon n°1 du RE.)*

---

## §3 — Catalogue des capacités & ports

**Régime** : *slot exclusif* = 1 implémentation active par run (résolue par config, owner
implicite + repli déterministe) ; *ensemble* = N implémentations concurrentes
(cf. ADR-0003, `RE-OH:271-301`). Un plugin déclare une **capacité**, jamais une marque
(`RULES-OMEGA.md:30-31`).

| Capacité | Port (trait Rust / interface TS) | Régime | Implémentations prévues |
|---|---|---|---|
| `llm-provider` | `Provider` (`stream` primaire, `complete` par défaut, `manages_own_context`, `retry_config`, capacités) | ensemble | anthropic, openai, google, bedrock/vertex, openrouter, **ollama/llama.cpp (self-host)**, providers **déclaratifs JSON** |
| `catalog-source` | `CatalogSource { fetch()->Vec<RawModel> }` | ensemble | openrouter, models.dev, ollama, huggingface, `/v1/models` provider, litellm |
| `model-registry` | `ModelRegistry { records(q); health(id) }` | **slot** | registre local souverain (SQLite/JSON, cache versionné) |
| `task-classifier` | `TaskClassifier { classify(req)->TaskType }` | **slot** | heuristique **déterministe** (T1..T14) ; pas de LLM |
| `router-policy` | `ScoringPolicy` + `Selector` + `FallbackPolicy` | **slot** | règles déterministes (défaut), bandit UCB1/Thompson, self-host-first |
| `router-feedback` | `FeedbackStore { record(ev); priors(m) }` | ensemble | store local (priors par point de service) |
| `context-engine` | `ContextEngine { bootstrap/ingest/assemble/compact/afterTurn/maintain }` + `ownsCompaction` | **slot** | `omega-context-default`, `LegacyContextEngine` (compat 100 %), engines tiers |
| `compaction-model` | `CompactionModel` + `TokenEstimator` | ensemble | petit modèle rapide (via routeur), estimateur déterministe (chars/tokens) |
| `prompt-cache-policy` | `PromptCachePolicy { plan(req)->[Breakpoint]; affinity(session) }` | **slot** | `anthropic-style` (≤4 breakpoints, TTL 5m/1h), `no-op` (providers sans hints) |
| `skill-store` | `SkillStore { discover(sources); load(id) }` | ensemble | FS `~/.agents/skills`, bundle embarqué, URL/`index.json` |
| `memory-store` | `MemoryStore { recall, add/replace/patch/remove, apply_batch }` | **slot** | fichiers bornés (`MEMORY.md`/`USER.md`), embeddings, mémoire générée |
| `memory-search` | `MemorySearch { query(...) }` | ensemble | **FTS5** (rusqlite), recherche vectorielle |
| `tool` | `ToolExecutor { tool_name, spec, exposure, search_info, handle }` | ensemble | natifs (shell, apply_patch, file_search, web_fetch), bridge **MCP**, plugins (WASM/natif) |
| `tool-surface` | `ToolSurface { index()->[ToolStub]; hydrate(q)->[ToolSchema] }` | ensemble | **BM25 + rarest-token gate** (défaut), passthrough (tout en avant) |
| `sandbox-backend` | `SandboxBackend { execute, fetch_file, cleanup }` | ensemble | local, docker, ssh, singularity, modal, daytona, vercel ; landlock+seccomp, bubblewrap |
| `execpolicy` | `ExecPolicy { evaluate(cmd)->Decision }` | **slot** | DSL de règles préfixes/réseau (`Allow/Prompt/Forbidden`) |
| `permission-policy` | `PermissionPolicy { evaluate(action, resource, scope)->Effect }` | **slot** | `Ruleset` ordonné `allow/ask/deny` (provider/tool/integration) |
| `approval-review` | `ApprovalReviewContributor { decide(input)->Option<Decision> }` | ensemble (chaîne de responsabilité) | config granulaire, auto-review |
| `hook` | `Hook { name, func }` → `HookResult{FailedContinue,FailedAbort}` | ensemble | `command`, `http`, `mcp_tool`, `prompt`, `agent` ; modes sync/async, scopes thread/turn |
| `subagent-runner` | `SubagentRunner { spawn(spec)->ChildHandle; resume(id,msg) }` | ensemble | in-process (contexte isolé), **worktree-isolated**, ACP externe |
| `scheduler-job` | `Scheduler { add/run_now/kill/list }` | ensemble | cron sans démon, automations événementielles |
| `channel` | `PlatformAdapter { check_fn, create_adapter }` | ensemble | telegram, discord, slack, signal, whatsapp, webhook |
| `telemetry-sink` | `TelemetrySink { emit(metric/event/span) }` | ensemble | OTLP, stdout, fichier local, no-op |
| `event-store` | `EventStore { append; replay }` | **slot** (durable) | journal append-only local (durable) + bus éphémère |
| `provenance-sink` | `ProvenanceSink { tag(fact, kind, source) }` | ensemble | tags `[stated]/[observed]/[inferred]` + source |
| `ui-panel` | `UiPanel` (descripteur) | ensemble | cockpit, plugin explorer, router dashboard, trace viewer, context inspector |
| `config-source` | `ConfigSource { layers()->[ConfigLayer] }` | ensemble | defaults, user, projet, session (3-4 couches, pas 9) |
| `harness-adapter` | `Provider`/`HarnessAdapter` | ensemble | plugins `harness-*` (claude-code, codex, gemini) — **jamais dans le noyau** |

---

## §4 — Boucle d'agent

**Modèle retenu (`ENG-01`, `RE-GC:89-152`).** Une machine à états générique sur la *session*
et l'*effet*, pilotée par une **liste ordonnée d'`Operation`** :

```text
run():  load(session) → step() → apply_effects() → … jusqu'à Yielded | Cancelled → finalize()
step(): parcourt les Opérations dans l'ordre ; première `Applied` gagne ;
        agrège outils + prompt de TOUTES les étapes dans l'InferenceInput ;
        honore un CancellationToken (select! biased).
finalize(): répare la conversation après annulation (réponses d'erreur synthétiques).
```

- **Ordre = priorité** : hooks avant inférence, contexte avant prompt, etc. (`RE-GC:150`).
- **Effets différés** appliqués par un `EffectHandler` ; `Emitter` = canal mpsc + token
  d'annulation (`RE-GC:131-146`).
- **Chaque opération note dans le message ce qu'elle a fait** (`set_message_meta`) pour que le
  pipeline reconstruit depuis la conversation persistée aboutisse à la même décision
  (`OBS-06`, `RE-GC:152`).
- **Compilation WASM** possible (`async_trait(?Send)`, `RE-GC:151`) : cœur partageable
  natif ↔ navigateur (option, non prioritaire).

**Plan/Act (`ENG-16`).** Un mode `plan` **read-only** (edit `deny`) est un agent déclaratif
(`ENG-13`) ; son rappel de mode est **ré-apposé/retiré près de la queue** du contexte pour
préserver le préfixe de cache (`RE-OC:106`). Le « planning agent » en tâche de fond
(Windsurf, `HL:210`) est un sous-agent différé (Phase 2).

**Sous-agents (`SUB-01`, `ENG-14`, `ENG-06`).** Chaque sous-agent a **sa propre fenêtre** et
**sa propre config** (`agent-roles` Codex, `RE-GC:561-565`) ; seul un **résumé** remonte au
parent (`HL:196-200`). Cycle de vie par hooks dédiés : `prepareSubagentSpawn` (+ `rollback`),
`contextMode: isolated | fork`, `onSubagentEnded(reason)` (`RE-OH:757-764`). Isolation
**worktree** = `SUB-02` (W9).

**Hooks typés (`ENG-05`, `ENG-06`).** Sémantique explicite, reprise d'OpenClaw + Codex :

| Sémantique | Effet | Hooks (exemples) |
|---|---|---|
| **Gate** | bloquer / exiger approbation | `before_tool_call`, `before_agent_run`, `turn_start_admission` |
| **Modify** | réécrire params / ajouter contexte / forcer un tour | `before_model_resolve`, `before_prompt_build`, `before_agent_finalize`, `before_tool_call` |
| **Observe** | journaliser (résultat, erreur, durée) | `after_tool_call`, `before/after_compaction`, `session_start/end`, `token_usage` |
| **Claim** | court-circuiter / fournir une réponse | `before_agent_reply`, `before_dispatch`, `decide_approval` |

`HookResult::{FailedContinue, FailedAbort}` distingue « continuer la chaîne » de « tout
arrêter » (`RE-GC:361-380`) ; budgets par hook (`before_prompt_build` 15 s, compaction 30 s,
`RE-OH:368-389`).

---

## §5 — Context Engine & économie de contexte

### 5.1 Cycle de vie (port `ContextEngine`, `CTX-01`)

```text
bootstrap ─► ingest / ingestBatch ─► assemble ─► (compact) ─► afterTurn ─► maintain ─► dispose
   │                                    │            │                                   ▲
   └── prepareSubagentSpawn / onSubagentEnded ───────┴───────────────────────────────────┘
```

- **`assemble`** renvoie `{ messages, estimatedTokens, systemPromptAddition?, contextProjection? }` ;
  la projection `thread_bootstrap` + `epoch` injecte le contexte **une fois par époque** sur les
  backends à thread persistant (gain de cache, `RE-OH:196-198`).
- **`ownsCompaction = true`** : l'hôte vérifie **chaque** requête sortante à partir de l'usage
  mesuré du run ; `estimatedTokens` de l'engine **ne contrôle pas** l'admission
  (`RE-OH:219-223`). Sinon l'hôte compacte lui-même.
- **Allowlist** : l'hôte ne transmet que les paramètres listés dans `acceptedHostParams`
  (`RE-OH:225-227`).
- **Compat 100 %** via `LegacyContextEngine` (ingest no-op, assemble pass-through, compact
  délégué) — c'est le repli déterministe de `PLG-12`.

### 5.2 Les 5 leviers de l'économie de contexte (tous déterministes → « système »)

1. **Tool search** (`CTX-10`, `RE-OH:430-508`) : 3 passerelles remplacent les schémas différables ;
   **BM25 + stemming + rarest-token gate + couverture minimale** ; listing **byte-stable**
   (ordre déterministe → préfixe cachable). Budget de listing = `min(threshold_pct% du contexte,
   listing_max_tokens)` ; le budget **ne contrôle pas** l'activation.
2. **Progressive disclosure des skills** (`CTX-09`, `RE-OH:780-788`) : index {nom, description}
   → corps → fichiers `references/` ; descriptions tronquées (≤1536 car.).
3. **Prompt caching par préfixe** (`CTX-04`, `RE-OC:243-246`) : ≤4 breakpoints, ordre
   `tools → system → messages`, TTL 5m/1h, affinité `parent ?? fork ?? session` ; télémétrie de
   rupture typée (`CTX-05`).
4. **Compaction** (`CTX-02/03`) : décorateur de Provider qui rattrape `ContextLengthExceeded` +
   seuils doubles (0.8 / 225k) + résumé **structuré** + rechargement ciblé (≤N fichiers, skills
   réinjectées ≤5000 tok) + anti-thrash.
5. **Isolation & budget** (`SUB-01`, `CTX-07`) : le travail bruyant va dans un sous-agent ; seul
   un résumé remonte ; l'optimisation de contexte ne charge que ce qui sert — mêmes principes que
   l'optimiseur documenté par OpenCode (`RE-OC:352`).

**Budget avant action** : chaque appel est estimé, tracé, plafonné ; dépassement = arrêt
(`RULES-OMEGA.md:58`). **Sorties verbeuses** : spill head/tail + chemin, trim des vieux
résultats (`CTX-08`, `HL:267`).

---

## §6 — Routeur de modèles (capacité `router-policy`)

Source détaillée : `docs/omega/research/MODEL-ROUTING-BENCHMARKS.md` (§2 score composite,
§3 taxonomie T1-T14, §4 métriques, §5 catalogues, §6 architecture + ports).

**Intégration comme capacités** (aucun fournisseur en dur, `RULES-OMEGA.md:97`):

```
CatalogSource (ensemble)  ──sync──►  ModelRegistry (slot)  ──►  ScoringPolicy/Selector/Fallback (slot)
   openrouter · models.dev                ModelRecord*            règles déterministes (défaut)
   ollama · HF · /v1/models           (modèle × effort × provider)   bandit UCB1/Thompson · self-host-first
                                            ▲
   [Tâche] ─► TaskClassifier (déterministe) ─► profil de pondération (T1..T14) ─► Score(t,m)
                 ▲                                                                    │
        FeedbackStore (priors) ◄── télémétrie ◄── ProviderAdapter (llm-provider) ◄───┘
```

- **Classifieur 100 % déterministe** (regex + longueur + pièces + mots-clés) : **un LLM ne
  décide pas du routage** (coût, latence, non-déterminisme ; `MRB:250-255`).
- **Scoring relatif** (percentile dans le registre), **multi-axes** (raison/ code/ agentique/
  knowledge/ chat/ vision) ; **par variante d'effort** (`MRB:282-286`).
- **Fallback** : même tier → tier supérieur → **self-hosted** → file/backoff ; circuit breaker
  half-open (`MRB:360-368`).
- **Feedback bandit** : `reward = q_obs − λ·coût_norm − μ·latence_norm` (`MRB:369-375`).
- **Garde budgétaire** : contrainte dure ; si le meilleur score dépasse le plafond, on retombe
  sur le meilleur score **Pareto-compatible** (`MRB:201-202`, `RTR-12`).

---

## §7 — Traçabilité & télémétrie

- **Event-bus** (`OBS-01`) : événements **durables** (journal append-only, manifeste) vs
  **éphémères** (UI live) (`RE-OC:123,315`). Le journal durable alimente le **replay** (`OBS-04`).
- **Provenance native** (`OBS-03`) : tout fait/artefact porte `[stated]/[observed]/[inferred]` +
  source (`RULES-OMEGA.md:67-71`) — **amélioration** que ni OpenClaw ni Hermes n'imposent
  (`RE-OH:874-875`).
- **OTel nommé** (`OBS-02`) : convention de nommage `omega.<domaine>.<objet>`
  (cf. `codex.api_request`, `codex.turn.token_usage`, `codex.tool.call`,
  `codex.multi_agent.spawn.phase.duration_ms`, `codex.hooks.run`, `RE-GC:537-541`) ;
  **cibles** `log-only` vs `trace-safe` (contrôle PII) ; gates `OTEL_LOG_*` et **redaction
  fail-closed** (`RE-OC:192-201`) ; destination **locale/self-hosted par défaut** (amélioration,
  `RE-OC:347`).
- **Traçabilité de la décision de routage** (`OBS-05`) : modèle, effort, provider, coût, latence,
  token/cache, issue (`MRB:377-379`).
- **Provenance de la couche de config** (`OBS-07`) : `layer_source` + version sur chaque décision
  (amélioration Codex, `RE-GC:625`).
- **Télémétrie de cache** (`CTX-05`) : diagnostic typé des ruptures (`ChangeCode`) pour rendre la
  stabilité de préfixe **mesurable** (`RE-OH:229-246`).

---

## §8 — Décisions adoptées / améliorées / écartées

### 8.1 Adoptées (reprises, ré-implémentées proprement)

| Élément | Source | Justification économique |
|---|---|---|
| Port `ContextEngine` + `ownsCompaction` + `LegacyContextEngine` | OpenClaw (`RE-OH:695-713`) | Contrat fin, cycle de vie explicite, repli gratuit. |
| Slots (exclusivité par capacité) | OpenClaw (`RE-OH:715-723`) | Une seule implémentation payée à la fois ; config > code. |
| Manifeste validé, lu **sans exécuter** le code | OpenClaw (`RE-OH:725-735`) | Contrat avant chargement → sécurité supply-chain. |
| Hooks typés Gate/Modify/Observe/Claim + budgets | OpenClaw (`RE-OH:737-746`) | Gouvernance déterministe sans « demander au modèle ». |
| Tool Search BM25 + rarest-token gate + coverage | Hermes (`RE-OH:766-778`) | Réduit la surface de tokens sans perdre le recall ; zéro LLM. |
| Progressive disclosure skills (3 niveaux) | Hermes (`RE-OH:780-788`) | Économie de contexte massive ; scan FS déterministe. |
| Mémoire bornée + snapshot gelé | Hermes (`RE-OH:790-797`) | Préserve le prefix-cache ; budget en caractères. |
| Boucle `Operation`/`Inference` + `EffectHandler`/`Emitter` | Goose (`RE-GC:652-676`) | Squelette du noyau, testable sans réseau. |
| Compaction = décorateur de Provider | Goose (`RE-GC:240-261`) | Ne pollue pas la boucle ; rattrape l'overflow. |
| Providers déclaratifs JSON + `KeyResolver` | Goose (`RE-GC:211-219`) | Ajouter un provider = config. |
| Contributeurs typés, builder → registre immuable | Codex (`RE-GC:282-331`) | « Everything is a plugin » réduit à ~8-10 traits. |
| Config à couches + `precedence()` + schéma JSON | Codex (`RE-GC:516-521`) | Provenance et précédence explicites. |
| Politique de sandbox/approbation **typée** | Codex (`RE-GC:523-528`) | Le *modèle de politique* vaut plus que le code OS. |
| Execpolicy (DSL déterministe) | Codex (`RE-GC:530-535`) | « Le déterministe n'est pas une tâche d'agent ». |
| OTel nommé + log-only/trace-safe | Codex (`RE-GC:537-541`) | Économie & traçabilité mesurées. |
| Découverte `AGENTS.md` remontante + trust | Codex (`RE-GC:555-559`) | Contexte projet éprouvé, plafonné. |
| Sous-agents porteurs de leur config | Codex (`RE-GC:561-565`) | Isolation propre ; seul un résumé remonte. |
| Tool executor `{spec, exposure, search_info, handle}` | Codex (`RE-GC:567-571`) | Infra de tool search directement réutilisable. |
| Protocole source unique → exports TS | Codex (`RE-GC:579-583`) | Frontière générée, pas maintenue à la main. |
| `transform` + `hook` + Scope superposable | OpenCode (`RE-OC:68-87`) | Désactivation propre, zéro « undo ». |
| Compaction structurée + rechargement ciblé | Claude Code (`RE-OC:134-141`) | Remplacement exact de la conversation verbatim. |
| Prompt caching par préfixe (≤4, ordre canonique, affinité) | OpenCode + Claude Code (`RE-OC:243-246`) | Levier n°1 d'économie ($ et latence). |
| Policy/permissions allow/ask/deny | OpenCode (`RE-OC:311`) | Gouvernance déterministe. |
| 7 backends d'exécution derrière un port | Hermes (`RE-OH:807-814`) | Self-host et « run anywhere ». |
| Orchestration durable + HITL | LangGraph (`HL:238-242`) | Reprise exacte ; candidat microservice (crash-boundary). |

### 8.2 Améliorées

| # | Sujet | Limite observée | Amélioration Omega |
|---|---|---|---|
| A1 | Monolithe applicatif | `goose` gros ; `core` Codex hub chargé (`RE-GC:621`) | `omega-core` ne dépend que de `types` + `ports` ; découpage strict. |
| A2 | Granularité extrême | ~200 crates (`RE-GC:622`) | Démarrer à ~11, fracturer **quand une frontière le justifie** (Phase 2). |
| A3 | Registre `HashMap` + closure non ordonné | Goose (`RE-GC:623`) | `IndexMap` + priorité explicite (déterminisme). |
| A4 | Erreurs `anyhow` aux frontières | perte de typage plugin (`RE-GC:624`) | Erreur de port **typée** obligatoire ; `anyhow` réservé au câblage binaire. |
| A5 | Traçabilité partielle | `layer_source` non exposé (`RE-GC:625`) | `OBS-07` : provenance de couche + version sur chaque décision. |
| A6 | Providers couplés à des harnais externes | Goose `claude_code.rs` (`RE-GC:626`) | Ces harnais = plugins `harness-*`, jamais dans le noyau. |
| A7 | Compaction non mesurée en qualité | mesure des tokens seulement (`RE-GC:627`) | Score de **rétention** déterministe avant/après. |
| A8 | Coût LLM non plafonné chez Hermes | `background_review` après **chaque** tour (`RE-OH:871-873`) | `MEM-06` **opt-in** avec plafond de budget. |
| A9 | Lexique éclaté (engine/slot vs backend/search) | OpenClaw vs Hermes (`RE-OH:865-866`) | Lexique unifié **capacité + port + adaptateur**. |
| A10 | Provenance absente | ni OpenClaw ni Hermes (`RE-OH:874-875`) | Tag `[stated]/[observed]/[inferred]` partout (`OBS-03`). |
| A11 | Fournisseurs en dur (« liste par défaut ») | `Provider.ID` OpenCode (`RE-OC:345`) | Manifeste **signé** + scoring communautaire ; le marketplace n'est qu'un résolveur. |
| A12 | Types Effect + JSON Schema non versionnés | OpenCode (`RE-OC:346`) | Traits Rust + schéma `serde`/JSON Schema **versionné**. |
| A13 | Télémétrie fournisseur | Anthropic (Claude Code) (`RE-OC:352`) | OTel repris, destination locale par défaut, redaction **fail-closed**. |
| A14 | Sensibilité au modèle d'un harnais | `Terminal-Bench` dépend de l'agent (`MRB:73-76`) | Comparer harnais et modèle séparément + harnais maison (`lm-eval`). |

### 8.3 Écartées (ou fortement encadrées)

| Écarté | Source | Raison |
|---|---|---|
| Surface `register*` complète (~60 méthodes) | OpenClaw (`RE-OH:881`) | Trop large ; cœur ~8-10 traits + extensibilisation par contrat. |
| Fork LLM automatique après chaque tour | Hermes (`RE-OH:882`) | Coût non plafonné ; contraire à « le coût va au système ». |
| `PluginKind` à 2 valeurs | OpenClaw (`RE-OH:883`) | Omega a besoin de plus de capacités (`routing`, `sandbox`…). |
| Monolithe Python « flat » | Hermes (`RE-OH:884`) | Idées conservées, structure non. |
| 9 couches de config enterprise/MDM | Codex (`RE-GC:630`) | Hors périmètre perso ; garder 3-4 couches. |
| Marketplace de plugins distant | Codex (`RE-GC:631`) | Complexité inutile à court terme. |
| Backends Windows (RestrictedToken, MXC) | Codex (`RE-GC:632`) | Linux-first : Landlock/seccomp + bubblewrap. |
| V8/deno code-mode complet | Codex (`RE-GC:633`) | Trop lourd ; diférer (`ENG-17`). |
| Binding uniffi multi-langage | Goose (`RE-GC:634`) | ACP + JSON-RPC d'abord. |
| Live voice (websocket voix) | Goose (`RE-GC:635`) | Hors mission. |
| Runtime JS in-process (« mods ») | Claude Code (`RE-OC:350`) | Hors noyau Rust ; WASM sandboxé si besoin (`ENG-18`). |
| Namespacing imposé / marketplace propriétaire | Claude Code (`RE-OC:351`) | Le marketplace n'est qu'un résolveur. |
| Contenu du system prompt fuité | corpus TikTok (`RE-OC:354`) | Propriétaire : **mécanismes** seulement, aucun contenu. |

---

## §9 — Mapping features → workstreams & ordre de construction

### 9.1 Distribution (96 features)

| Workstream | Nb | Features |
|---|---|---|
| **W2** Plugin (ports, registre, manifestes, slots) | 17 | `PLG-01..PLG-17` |
| **W3** Rust (moteur : boucle, outils, hooks, config, protocole, MCP/ACP, sous-agents) | 19 | `ENG-01..ENG-18`, `SUB-01`, `SUB-03` |
| **W4** Routing (registre modèles, catalogues, scoring, fallback) | 14 | `RTR-01..RTR-14` |
| **W6** Contexte (Context Engine, compaction, caching, tool search, mémoire, FTS5) | 20 | `CTX-01..CTX-13`, `MEM-01..MEM-07` |
| **W7** UI (UI interchangeable, types TS générés) | 1 (+1 partagé) | `UI-01` (+ `ENG-10`) |
| **W8** Traçabilité (event-bus, OTel, provenance, replay, durable) | 8 | `OBS-01..OBS-07`, `SUB-04` |
| **W9** Sandbox (multi-backend, execpolicy, approbations, worktrees) | 10 | `SBX-01..SBX-09`, `SUB-02` |
| **W10** Économie (déterminisme au système, coût, self-host, cron/gateway) | 6 | `ECO-01..ECO-06` |

### 9.2 Ordre de construction (chemin critique)

```
W5 (RE — clos) ─► W2 Plugin ─► W3 Moteur + W4 Routing ─► W6 Contexte ─► W8 Traçabilité ─► W7 UI ─► W9 Sandbox ─► W10 Économie
```

1. **W2 — Plugin** (prérequis de tout) : `PLG-02` (slots), `PLG-05` (registre à constructeurs),
   `PLG-12` (isolation d'échec), puis `PLG-01`/`PLG-03`/`PLG-04`/`PLG-07`/`PLG-08`/`PLG-17`.
   *Port + contrat + test d'acceptation avant tout code (`RULES-OMEGA.md:80-81`).*
2. **W3 — Moteur** : `ENG-09` (annulation) → `ENG-01` (boucle) → `ENG-02` (provider) →
   `ENG-03/04` (outils) → `ENG-05/06` (hooks) → `ENG-07` (config) → `ENG-11` (MCP) →
   `ENG-13/14` (agents/sous-agents) → `ENG-10` (protocole/TS).
3. **W4 — Routing** : `RTR-05` (classifieur) + `RTR-02` (providers déclaratifs) →
   `RTR-01/04/11` (registre/catalogues/scoring) → `RTR-06/07/08` (score/sélecteur/fallback) →
   `RTR-12` (budget) → `RTR-09/10` (bandit/cache).
4. **W6 — Contexte** : `CTX-09` (disclosure) + `CTX-07` (budget) + `CTX-06` (AGENTS.md) →
   `CTX-01` (engine) → `CTX-02/03` (compaction) → `CTX-04/05` (caching) → `CTX-10` (tool search) →
   `MEM-01/02` → `MEM-04` (FTS5) → `CTX-08/11/12/13`.
5. **W8 — Traçabilité** : `OBS-03` (provenance) + `OBS-05` (décision de routage) →
   `OBS-01/02` (event-bus/OTel) → `OBS-04` (replay) + `SUB-04` (durable).
6. **W7 — UI** : cockpit / plugin explorer / router dashboard / trace viewer / context inspector,
   types TS générés (`ENG-10`).
7. **W9 — Sandbox** : `SBX-06` (permissions) → `SBX-02/04` (politiques/approbations) →
   `SBX-01/03` (backends) → `SBX-05` (execpolicy) → `SBX-08` (trust) → `SBX-07` (checkpoints) +
   `SUB-02` (worktree).
8. **W10 — Économie** : `ECO-01` (déterminisme au système) + `ECO-03` (coût) → `ECO-02`
   (self-host) → `ECO-04/05/06` (cron/gateway/automations).

### 9.3 Quick wins d'entrée (S + valeur 3)

`PLG-02`, `PLG-05`, `PLG-12`, `ENG-09`, `ENG-13`, `CTX-06`, `CTX-07`, `CTX-09`, `MEM-02`,
`RTR-02`, `RTR-05`, `RTR-12`, `SBX-06`, `OBS-03`, `OBS-05`, `ECO-01`, `ECO-03`
(détail : `FEATURE-MATRIX.md` §4).

---

*Synthèse produite au 2026-10-15 (W5). Matrice source : `docs/omega/reverse-eng/FEATURE-MATRIX.md`.
Version machine-lisible : `docs/omega/reverse-eng/features.json`. Toute évolution d'architecture
passe par un ADR (`docs/omega/adr/`).*
