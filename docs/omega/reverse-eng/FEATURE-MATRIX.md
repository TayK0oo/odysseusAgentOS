# FEATURE-MATRIX — Matrice consolidée des features (W5)

> **Workstream :** W5 · **Date :** 2026-10-15 · **Branche :** `feat/agentos-omega`
> **Objet :** fusion **dédupliquée** de toutes les features significatives extraites des trois
> fiches de RE (`RE-OPENCLAW-HERMES.md`, `RE-GOOSE-CODEX.md`, `RE-OPENCODE-CLAUDECODE.md`)
> et du paysage (`docs/omega/research/HARNESS-LANDSCAPE.md`, features P1-P14).
> **Garde-fou :** patterns et idées uniquement ; aucune ref inventée (toutes reprises des fiches).

## Légende

| Colonne | Échelle |
|---|---|
| **Valeur Omega (0-3)** | `0` rien · `1` utile · `2` important · `3` fondamental (contribution aux 6 piliers / RULES-OMEGA) |
| **Difficulté Rust (S/M/L)** | `S` ≤ 1 j-agent · `M` 2-5 j · `L` > 5 j / multi-crates |
| **Workstream cible** | `W2` Plugin · `W3` Rust (moteur) · `W4` Routing · `W6` Contexte · `W7` UI · `W8` Traçabilité · `W9` Sandbox · `W10` Économie |
| **Statut** | `reprend` (tel quel, ré-implémenté) · `améliore` (corrige une limite observée) · `écarte` (hors périmètre / fortement encadré) |
| **Impact** | `valeur ÷ difficulté` (S=1, M=2, L=3) — sert au classement final |

**Provenance.** Les features issues des fiches T1-T5 sont `[observed]` (code cité `file:line` dans
les fiches) ; celles issues de **Claude Code** sont `[stated]` (docs publiques) ; les colonnes
**Valeur / Difficulté / Workstream / Statut** sont des jugements Omega `[inferred]`. Les features
de `HARNESS-LANDSCAPE.md` sont `[observed]` (README/docs, URL dans la source).
Refs : `RE-OPENCLAW-HERMES.md` = `RE-OH` · `RE-GOOSE-CODEX.md` = `RE-GC` ·
`RE-OPENCODE-CLAUDECODE.md` = `RE-OC` · `HARNESS-LANDSCAPE.md` = `HL` ·
`MODEL-ROUTING-BENCHMARKS.md` = `MRB`.

---

## 1. Matrice unique (96 features)

| ID | Feature | Source(s) (harnais) | Contrat/port clé (résumé) | Valeur Omega (0-3) | Difficulté Rust (S/M/L) | Workstream cible (W2..W10) | Statut (reprend/améliore/écarte) | Refs (fichier:ligne ou URL) |
|---|---|---|---|---|---|---|---|---|
| CTX-01 | Context Engine enfichable : cycle de vie `ingest → assemble → compact → afterTurn` (+ `bootstrap`, `maintain`), flag `ownsCompaction`, projection `thread_bootstrap` + `epoch` | OpenClaw, OpenCode, Goose | `trait ContextEngine { ingest/assemble/compact/afterTurn/maintain/dispose }` + `ContextEngineInfo.ownsCompaction` + allowlist `acceptedHostParams` | 3 | L | W6 | reprend | RE-OH:136-223,695-713; RE-OC:302; RE-GC:488-493 |
| CTX-02 | Compaction en **décorateur de Provider** (rattrape `ContextLengthExceeded` et retente) + seuils doubles (0.8 relatif / 225k absolu) + suppression milieu→extérieur | Goose, Claude Code | `trait CompactionModel`, `trait TokenEstimator`, `CompactingProvider` | 3 | M | W6 | reprend | RE-GC:240-261,488-493; HL:190-194 |
| CTX-03 | Compaction **structurée** + rechargement ciblé (résumé structuré, ≤N fichiers relus, skills réinjectées ≤5000 tok, anti-thrash) | Claude Code, OpenCode | résumé structuré (`Objective/Requirements/Work State/Next Move`), `SHRINK_STEPS`, `keep`/`buffer` | 3 | M | W6 | reprend | RE-OC:134-141,306; HL:57 |
| CTX-04 | Prompt caching par préfixe : ≤4 breakpoints, ordre canonique `tools → system → messages`, TTL 5m/1h, affinité `parent/fork/self` | OpenCode, Claude Code | `trait PromptCachePolicy { plan(request)->[Breakpoint]; affinity(session)->CacheKey }` | 3 | M | W6 | reprend | RE-OC:243-246,307; HL:190-194 |
| CTX-05 | Télémétrie **prompt-cache typée** + codes de rupture (`historyRewrite/compaction/pruning/systemPrompt/tools/…`) | OpenClaw | `ContextEnginePromptCacheInfo` + `ChangeCode` | 3 | M | W6 | reprend | RE-OH:229-246,681 |
| CTX-06 | Découverte `AGENTS.md` remontante (racine par marqueurs, concat racine→cwd, override local, plafond octets, project trust) | Codex | `DEFAULT_AGENTS_MD_FILENAME`, `AGENTS.override.md`, `find_nearest_ancestor_with_markers` | 3 | S | W6 | reprend | RE-GC:400-414,555-559 |
| CTX-07 | Budget de contexte **avant action** : `estimate → plafond → arrêt` (dépassement = arrêt, pas de dérive) | OpenCode, Goose, Hermes | `trait ContextBudget { estimate(request)->tokens; compact(trigger,keep,buffer)->Outcome }` | 3 | S | W6 | améliore | RE-OC:248-252; rules/RULES-OMEGA.md:58 |
| CTX-08 | Spill / résumé des sorties verbeuses + trim des vieux résultats d'outils (head/tail + chemin) | Goose, Hermes, OpenClaw, Claude Code | `output_spill`, `tool_output.max_bytes`, trim du milieu | 2 | S | W6 | reprend | HL:267; RE-OH:44 |
| CTX-09 | Progressive disclosure des skills (3 niveaux : index → corps → `references/`/`templates/`) | Hermes, Claude Code, Codex, OpenCode | `skills_list`/`skill_view`; descriptions ≤1536 car. | 3 | S | W6 | reprend | RE-OH:780-788; RE-OC:162-169; RE-GC:549-553 |
| CTX-10 | Tool search : 3 passerelles + **BM25 + stemming + rarest-token gate + coverage** + listing byte-stable budgeté | Hermes, Claude Code, OpenClaw, Codex | outils `tool_search`/`tool_describe`/`tool_call`; `search_catalog()` | 3 | M | W6 | reprend | RE-OH:430-508,766-778; RE-OC:153-160 |
| CTX-11 | Repo map / index structurel **déterministe** (tree-sitter + ranking de graphe, tronqué au budget) | Aider, Plandex | `--map-tokens`; PageRank fichiers=noeuds | 2 | M | W6 | reprend | HL:91,265 |
| CTX-12 | Règles **path-scoped** chargées à l'usage (activation par glob sur le fichier touché) | Claude Code | `.claude/rules/` + `paths:` | 2 | S | W6 | reprend | HL:271,55 |
| CTX-13 | Recherche code déterministe locale (Instant Grep / grep indexé, pas d'upload) | Cursor | index local + regex/word-boundary | 2 | M | W6 | reprend | HL:23,74 |
| PLG-01 | Manifeste déclaratif schema-validé + **policy-par-code** (lu **sans exécuter** le code du plugin) | OpenClaw | `openclaw.plugin.json {configSchema, activation, contracts}`, `definePluginEntry` | 3 | M | W2 | reprend | RE-OH:725-735 |
| PLG-02 | **Slots** : une seule implémentation active par capacité exclusive (+ owner implicite, repli `legacy`) | OpenClaw | `PluginSlotKey`, `resolveSlotSelection`, `applyExclusiveSlotSelection` | 3 | S | W2 | reprend | RE-OH:271-301,715-723 |
| PLG-03 | Registre de plugins multi-source (`builtin/package/local/sdk`) + capacités par surface + `update/check` + état `failed` non fatal | OpenCode | `Source`/`Features`/`State`; `plugin.list/check/update` | 3 | M | W2 | reprend | RE-OC:46-66,303 |
| PLG-04 | Registre de **contributeurs typés par cycle de vie** (builder → registre immuable, défauts vides, chaîne de responsabilité) | Codex | `ExtensionRegistryBuilder`/`ExtensionRegistry`; ~15 traits réduits à ~8-10 | 3 | L | W2 | reprend/améliore | RE-GC:282-331,509-514 |
| PLG-05 | Registre runtime **à constructeurs** + `ProviderType` | Goose | `ProviderConstructor = Arc<dyn Fn(..)->BoxFuture<Result<Arc<dyn Provider>>>>` | 3 | S | W2 | reprend | RE-GC:199-209 |
| PLG-06 | Registre **ordonné et déterministe** (IndexMap + priorité explicite) au lieu d'un `HashMap` | Goose (amélioré) | ordre déterministe des contributeurs | 2 | S | W2 | améliore | RE-GC:623 |
| PLG-07 | **Deux verbes d'extension** : `transform` (état rejouable) + `hook` (live) + `reload` | OpenCode | `transform(domain, cb)` / `hook(domain, event, cb)` / `reload` | 3 | M | W2 | reprend | RE-OC:68-78 |
| PLG-08 | **Superposition par Scope** (dernière inscription gagne ; fermeture révèle la précédente) | OpenCode | `Registration` + `Scope`; loi d'overlay | 3 | M | W2 | reprend | RE-OC:82-87 |
| PLG-09 | Découverte **idempotente** (empreinte sha256 + cache d'échec d'import + watch) | OpenCode | `createPluginSources(watch)` | 2 | S | W2 | reprend | RE-OC:62 |
| PLG-10 | Surface d'enregistrement ~60 `register*` / 179 extensions | OpenClaw | `plugin-api.types.ts` `register*` | 2 | L | W2 | écarte (coeur réduit ~8-10) | RE-OH:347-366,881 |
| PLG-11 | **Plugin override policy** (opt-in opérateur pour écraser un outil natif) | Hermes | `_PluginOverridePolicy`, `allow_tool_override` | 2 | S | W2 | reprend | RE-OH:834-840 |
| PLG-12 | **Isolation d'échec** : un plugin peut `failed(cause)` sans arrêter le noyau ; quarantaine + repli déterministe | OpenClaw, OpenCode | `State.failed(cause, ref)`; repli `legacy` | 3 | S | W2 | reprend | RE-OH:703; HL:158; RE-OC:286 |
| PLG-13 | Feature staging & cycle de vie (`UnderDevelopment → Experimental → Stable → Deprecated → Removed` + registre) | Codex | `enum Stage`, `FEATURES` | 2 | S | W2 | reprend | RE-GC:591-595 |
| PLG-14 | Injection de dépendances **sans service locator** (par constructeur + `Option<Arc<dyn Trait>>`) | Goose, Codex | `OmegaRegistry<T: Port>`, `PortContext` | 2 | S | W2 | reprend | RE-GC:445-453 |
| PLG-15 | Marketplace de plugins (catalogue + scopes + namespacing) | Claude Code | `marketplace.json`; scopes `user/project/local` | 2 | M | W2 | écarte/encadre (résolveur seul) | RE-OC:179-184,351 |
| PLG-16 | Distribution personnalisée (re-branding + preset providers/extensions sans fork) | Goose | `config.yaml` + `goose serve` (ACP) | 2 | S | W2 | reprend | RE-GC:585-589 |
| PLG-17 | **Trust & provenance des plugins** + policy d'installation (allow/deny, quarantaine, manifeste signé) | OpenClaw, Claude Code, Codex | `security.installPolicy`, `plugin.json` | 3 | M | W2 | améliore | HL:124,160; RE-OC:345 |
| ENG-01 | Boucle d'agent abstraite : étapes `Operation`/`Inference` **ordonnées** (ordre = priorité), effets différés, réparation post-annulation, meta par message | Goose | `trait Operation`, `trait Inference`, `trait EffectHandler`, `Emitter`, `RunStatus` | 3 | M | W3 | reprend | RE-GC:89-152,481-486,652-676 |
| ENG-02 | Port Provider (méthode primaire `stream`, `complete` par défaut, `manages_own_context`, `retry_config`, capacités) | Goose, Codex | `trait Provider` / `trait ModelProvider` | 3 | M | W3 | reprend | RE-GC:154-197,333-357 |
| ENG-03 | Tool executor & **exposure** (`spec`/`handle` séparés ; `Direct/Deferred/CodeModeOnly/Hidden` ; `search_info`) | Codex | `trait ToolExecutor { tool_name, spec, exposure, search_info, handle }` | 3 | M | W3 | reprend | RE-GC:333-359,567-571 |
| ENG-04 | Registre d'outils (`schema`, `check_fn`, timeouts, `dynamic_schema_overrides`, cap de taille de résultat) | Hermes | `ToolEntry`, `ToolRegistry.get_definitions` | 3 | M | W3 | reprend | RE-OH:510-537 |
| ENG-05 | Hooks typés de cycle de vie (sémantique **Gate/Modify/Observe/Claim** + budgets + matcher + décision structurée abortable) | OpenClaw, Claude Code, Codex, Goose | `HookResult{FailedContinue,FailedAbort}`, `HookEvent`, `api.on` | 3 | M | W3 | reprend | RE-OH:368-389,737-746; RE-GC:361-380,543-547 |
| ENG-06 | Hooks de sous-agents (`prepareSubagentSpawn` + `rollback`, `contextMode isolated/fork`, `onSubagentEnded`) | OpenClaw | `prepareSubagentSpawn`, `onSubagentEnded` | 3 | M | W3 | reprend | RE-OH:757-764 |
| ENG-07 | Config à couches + profils + `precedence()` **testable** + schéma JSON + `deny_unknown_fields` | Codex, OpenClaw, Goose | `ConfigLayerSource::precedence`, `merge_toml_values`, `ConfigProfile` | 3 | M | W3 | reprend | RE-GC:382-398,516-521; RE-OH:248-269 |
| ENG-08 | Erreurs de port **typées** (`thiserror` ; récupérable/fatale ; connues des décorateurs) | Goose, Codex | `ProviderError::ContextLengthExceeded`, `CodexErr`, `map_api_error` | 2 | S | W3 | reprend | RE-GC:428-434 |
| ENG-09 | **Annulation coopérative** (`CancellationToken` propagé partout, jamais d'`abort` brutal) | Goose, Codex | `OrCancelExt::or_cancel`, `Emitter.cancel_token` | 3 | S | W3 | reprend | RE-GC:436-443 |
| ENG-10 | Frontière **protocole source unique** + exports TS générés (`ts-rs`) + fixtures de schéma | Codex, Goose | `protocol`/`app-server-protocol`; `export.rs` | 2 | M | W3/W7 | reprend | RE-GC:468-473,579-583 |
| ENG-11 | Extensions **MCP** (client, transports stdio/HTTP/builtin, cache tools versionné, préfixage `ext__tool`, `${VAR}`) | Goose | `McpClientTrait`, `ExtensionManager` | 3 | M | W3 | reprend | RE-GC:221-238,502-507 |
| ENG-12 | Interop **MCP serveur + ACP** (agents/harnais externes traités comme plugins) | landscape, Goose, OpenClaw | serveur ACP; adaptateurs de harnais | 2 | M | W3 | améliore | HL:152,127; RE-GC:280 |
| ENG-13 | Agents déclaratifs (frontmatter YAML + permissions + `mode subagent/primary`, découverte multi-source) | OpenCode, Claude Code | `schema/agent.ts {id, model, permissions, mode}` | 3 | S | W3 | reprend | RE-OC:92,304 |
| ENG-14 | Sous-agents **déclarés porteurs de leur config** (+ built-ins, profondeur limitée, foreground/background) | Codex, Claude Code, Hermes | `agent-roles` TOML, `TaskConfig`, outil `subagent` | 3 | M | W3 | reprend | RE-GC:561-565; RE-OC:107,171-177 |
| ENG-15 | Skills store moteur (découverte, catalogue, **sélection dynamique**, invocation implicite, alias/host-roots) | Codex, OpenCode | `dynamic_skill_selector`, `detect_implicit_skill_invocation` | 2 | M | W3 | reprend | RE-GC:549-553 |
| ENG-16 | Plan/Act + agent de planification (mode read-only ; rappel de mode **ré-apposé près de la queue** pour préserver le cache) | OpenCode, Cline, Windsurf | agent `plan` read-only; hook `session.context` | 2 | S | W3 | reprend | RE-OC:106; HL:208-212 |
| ENG-17 | Code Mode (le modèle appelle les outils via du **code sandboxé** ; schémas rendus en types) | OpenCode, smolagents | `codemode/tool-schema.ts`, `MAX_RENDER_DEPTH=8` | 2 | L | W3 | écarte/diffère | RE-OC:122,314; RE-GC:633 |
| ENG-18 | Hooks **JS in-process** (« mods ») | Claude Code | hooks JS embarqués | 1 | L | W3 | écarte (hors noyau Rust) | RE-OC:182,318,350 |
| MEM-01 | Mémoire **bornée** (caractères) + **snapshot gelé** au chargement (préserve le prefix-cache) | Hermes, Claude Code, Codex | `MemoryStore.format_for_system_prompt` (snapshot figé ≠ état vivant) | 3 | M | W6 | reprend | RE-OH:539-575,790-797 |
| MEM-02 | Table mémoire **transactionnelle** (`add/replace/patch/remove/apply_batch` all-or-nothing + garde-fous drift/injection/patch ambigu) | Hermes | `MemoryStore.{add,replace,patch,remove,apply_batch}` | 3 | S | W6 | reprend | RE-OH:539-575 |
| MEM-03 | Contrat mémoire enfichable + **autorisation/origine des hits** | OpenClaw | `MemoryPluginRuntime.authorizeSearchHits`, `classifyWorkspaceMemoryPaths` | 2 | M | W6 | reprend | RE-OH:303-345 |
| MEM-04 | `session_search` **FTS5** (forme unique, 4 modes inférés, dédup lineage, démotion source, bornes ISO/relatives) | Hermes | `session_search()`; schéma FTS5 + trigram | 2 | M | W6 | reprend | RE-OH:577-597,799-805 |
| MEM-05 | Memories locales **générées** (pipeline d'écriture 2 phases sandboxé, citations, budget) | Codex | `memories/write/{phase1,phase2,guard}` | 2 | M | W6 | reprend | RE-GC:573-577 |
| MEM-06 | Boucle d'**auto-amélioration** (background review post-tour + curator d'inactivité ; archive **jamais** supprimer, skills pinned) | Hermes | `prepare_background_review_run`, `maybe_run_curator` | 2 | M | W6 | reprend (opt-in, budget plafonné) | RE-OH:644-666,825-832 |
| MEM-07 | Mémoire **cross-harness** partagée | gptme | mémoire partagée inter-harnais | 1 | M | W6 | reprend | HL:33 |
| RTR-01 | Registre de capacités modèles (par **variante d'effort** : qualités par axe, coût, vitesse, TTFT, contexte, capacités, santé) | MODEL-ROUTING | `ModelRecord`; `trait ModelRegistry { records(q); health(id) }` | 3 | M | W4 | reprend | MRB:282-286,524-556 |
| RTR-02 | Providers **déclaratifs JSON** + `KeyResolver` injecté | Goose | `include_dir!`, `expose_declarative_provider!`, `definitions/*.json` | 3 | S | W4 | reprend | RE-GC:211-219,495-500 |
| RTR-03 | Catalogue communautaire **rejouable** (config/auth comme transforms) | OpenCode | catalog transforms | 3 | M | W4 | reprend | RE-OC:124,312 |
| RTR-04 | Sources catalogue en **adaptateurs** (OpenRouter, models.dev, Ollama, HF, `/v1/models`) → `ModelRecord` | MODEL-ROUTING | `trait CatalogSource { fetch()->Vec<RawModel> }` | 3 | M | W4 | reprend | MRB:295-311,383 |
| RTR-05 | **Classifieur de tâche déterministe** (T1..T14 ; regex + longueur + pièces ; **zéro LLM**) | MODEL-ROUTING | `trait TaskClassifier { classify(req)->TaskType }` | 3 | S | W4 | reprend | MRB:231-260,385 |
| RTR-06 | Moteur de **scoring composite** multi-axes + profils de pondération par type de tâche | MODEL-ROUTING | `trait ScoringPolicy { weights(t); score(t,m) }` | 3 | M | W4 | reprend | MRB:156-192,384 |
| RTR-07 | Sélecteur + **filtres de capacités dures** + frontière de Pareto (qualité/prix) + tie-break coût/latence/self-host | MODEL-ROUTING | `trait Selector { pick(t, candidates)->Decision }` | 3 | M | W4 | reprend | MRB:194-203,387 |
| RTR-08 | Politique de **fallback** (même tier → tier sup. → self-hosted → file/backoff) + circuit breaker (half-open) | MODEL-ROUTING, OpenClaw, Hermes | `trait FallbackPolicy { next(failed, state) }` | 3 | M | W4 | reprend | MRB:360-368,388 |
| RTR-09 | Boucle de feedback **bandit** (UCB1/Thompson ; reward = qualité − λ·coût − μ·latence) | MODEL-ROUTING | `trait FeedbackStore { record(ev); priors(m) }` | 2 | M | W4 | reprend | MRB:369-375,390 |
| RTR-10 | **Cache de décision** de routage (clé `hash(tâche, profil, version registre)` + invalidation sur changement de santé) | MODEL-ROUTING | cache de décision | 2 | S | W4 | reprend | MRB:358 |
| RTR-11 | Scoring **communautaire multi-axes** daté et sourcé (AA, ECI, SWE-bench, Terminal-Bench, Aider, LiveBench, HF, LMArena) | MODEL-ROUTING | benchmarks datés par source | 3 | M | W4 | reprend | MRB:24-93,397-489 |
| RTR-12 | **Garde budgétaire** (plafond par appel et par session ; refus/escalade contrôlée) | MODEL-ROUTING, RULES | contrainte dure budget (best-score Pareto-compatible) | 3 | S | W4 | reprend | MRB:201-202; rules/RULES-OMEGA.md:58 |
| RTR-13 | **Changement de modèle en cours de tâche** sans markup | Kilo | switch mid-task | 2 | S | W4 | reprend | HL:25,203 |
| RTR-14 | **Harness emulation** (profils de harnais par modèle : `claude-code/native/swe-agent/minimal`) | Open Interpreter | `/harness` profils | 2 | M | W4 | reprend | HL:48 |
| OBS-01 | **Event-bus** : événements **durables** vs **éphémères** + manifeste durable | OpenCode | `durable()`/`ephemeral()`; `event-manifest` | 3 | M | W8 | reprend | RE-OC:123,315 |
| OBS-02 | **OpenTelemetry nommé** + cibles `log-only` vs `trace-safe` (contrôle PII) + gates de redaction fail-closed | Codex, Claude Code, Goose | `otel/metrics/names.rs` (`codex.<domaine>.<objet>`), `OTEL_LOG_*` | 3 | M | W8 | reprend | RE-GC:537-541,592; RE-OC:192-201,313 |
| OBS-03 | **Provenance native** (`[stated]/[observed]/[inferred]`) + source sur tout artefact/événement | RULES/Omega | tag de provenance obligatoire | 3 | S | W8 | améliore | rules/RULES-OMEGA.md:67-71 |
| OBS-04 | **Replay** d'une session depuis les traces (sans ré-exécuter les effets) | Omega vision | journal append-only + replay déterministe | 3 | L | W8 | reprend/améliore | 00-VISION-OMEGA.md:190 |
| OBS-05 | Traçabilité de la **décision de routage** (modèle, effort, provider, coût, latence, issue) | MODEL-ROUTING | trace de décision | 3 | S | W8 | reprend | MRB:377-379 |
| OBS-06 | **Métadonnées de message** notées par opération (`set_message_meta`) pour reconstruction déterministe | Goose | `set_message_meta`/`message_meta` | 2 | S | W8 | reprend | RE-GC:152 |
| OBS-07 | **Provenance de la couche de config** (`layer_source` + version) exposée sur chaque décision | Codex (amélioré) | `ConfigLayer {name, version, disabled_reason}` | 2 | S | W8 | améliore | RE-GC:386-389,625 |
| SUB-01 | **Sous-agents isolés parallèles** (contexte propre, seul un résumé remonte, profondeur limitée, foreground/background) | Claude Code, Codex, Hermes, Cursor | `spawn/resume`; contexte isolé | 3 | M | W3 | reprend | HL:196-200; RE-OC:309 |
| SUB-02 | Isolation par **git worktree** du sous-agent (`isolation: worktree`) | Claude Code, Zed | `isolation: worktree` | 3 | L | W9 | reprend | RE-OC:174,309; HL:216 |
| SUB-03 | **Équipe persistante** (coordinateur + spécialistes, état conservé entre sessions) | Cline | `cline --team-name` | 2 | M | W3 | reprend | HL:84 |
| SUB-04 | **Orchestration durable** + human-in-the-loop (journal append-only, replay déterministe, `interrupts`) | LangGraph, CrewAI | durable execution; interrupts | 3 | L | W8 | reprend | HL:238-242 |
| SBX-01 | Port d'exécution **multi-backend** (local/docker/ssh/singularity/modal/daytona/vercel) + spawn-per-call + snapshot de session | Hermes | `BaseEnvironment`; `TERMINAL_ENV`; backends plugin | 3 | L | W9 | reprend | RE-OH:393-428,807-814 |
| SBX-02 | **Politique de sandbox typée** (read-only/workspace-write/danger-full-access/external, `writable_roots`, network, exclusions) | Codex | `SandboxMode`/`SandboxPolicy`; `WritableRoot` | 3 | M | W9 | reprend | RE-GC:523-528 |
| SBX-03 | Backends Linux (**Landlock+seccomp**, bubblewrap) + réécriture d'`argv` | Codex | `SandboxManager::transform` | 3 | L | W9 | reprend (Linux-first) | RE-GC:524-528,632 |
| SBX-04 | **Approbations granulaires** (`UnlessTrusted/OnRequest/Granular/Never` + sous-types) + chaîne de décision | Codex | `AskForApproval`, `ApprovalReviewContributor.decide` | 3 | M |W9 | reprend | RE-GC:525,523-528 |
| SBX-05 | **Execpolicy** (DSL de règles préfixes/réseau `Allow/Prompt/Forbidden`, amendable) | Codex | `PolicyParser`, `Policy::evaluate`, `Rule` | 2 | M | W9 | reprend | RE-GC:530-535 |
| SBX-06 | **Policy/permissions** `allow/ask/deny` (provider/tool/integration ; règles ordonnées) | OpenCode | `Permission.Ruleset`, `Permission.evaluate` | 3 | S | W9 | reprend | RE-OC:97-98,311 |
| SBX-07 | **Checkpoints / worktrees / rollback** (snapshot + revert nommé, aires protégées, `/rewind` vers préfixe caché) | Zed, Windsurf, Claude Code, Plandex, Cline | checkpoints; sandbox de diff | 3 | L | W9 | reprend | RE-OC:316; HL:214-218 |
| SBX-08 | **Trust de projet / entrée non fiable** (policy par code ; projet non-trusté court-circuite la lecture) | Codex, Claude Code, OpenClaw | project trust; `is_untrusted()` | 3 | M | W9 | reprend | RE-GC:414; HL:160 |
| SBX-09 | Sécurité **supply-chain** (pins exacts, veille worm, identité de plugin) | Hermes | `pyproject` pins; plugin identity | 1 | S | W9 | reprend | RE-OH:839 |
| ECO-01 | **Déterminisme au système** (parser/scoring/index/diff = scripts/outils, **jamais** un tour LLM) | Aider, Cursor, Hermes, OpenClaw | règle d'architecture | 3 | S | W10 | reprend | HL:164,266; rules/RULES-OMEGA.md:55-57 |
| ECO-02 | **Self-hosted first + souveraineté** (sujet bloqué → modèle self-hosted) | RULES/Omega, Hermes | fallback self-hosted | 3 | M | W10 | reprend | 00-VISION-OMEGA.md:194-199; rules/RULES-OMEGA.md:61-63 |
| ECO-03 | **Comptabilité de coût** (tokens par type, cache read/write, coût par run, plafond) | Codex, Claude Code | `codex.turn.cost_microusd`, `cache_read_tokens` | 3 | S | W10 | reprend | RE-GC:537-541; RE-OC:197 |
| ECO-04 | **Cron + scheduler sans démon** + job schema (champs *authored* vs état scheduler) | Hermes, Goose, Cline | `SchedulerTrait`, `JOB_DEFINITION_FIELDS` | 2 | M | W10 | reprend | RE-OH:599-615,816-823 |
| ECO-05 | **Gateway messagerie** + registre de plateformes (path plugin, zéro modification du core) | Hermes, OpenClaw | `PlatformRegistry.register/create_adapter` | 2 | M | W10 | reprend | RE-OH:617-642 |
| ECO-06 | **Automations déclenchées** (schedule + webhooks/événements, sessions distantes) | OpenHands, Cline, Claude Code | automations trigger | 2 | M | W10 | reprend | HL:244-248 |
| UI-01 | **UI interchangeable** par protocole (TUI/desktop/web + serveur ACP/client) | Goose, OpenCode, OpenHands, Zed | `goose serve` (ACP); SDK/client | 2 | M | W7 | reprend | RE-GC:585-589; HL:111,136 |

---

## 2. Correspondance avec les features du paysage (P1-P14)

Les 14 features prioritaires de `HARNESS-LANDSCAPE.md` §4 sont couvertes par la matrice
(une feature = une ligne) :

| Paysage | Se décompose en |
|---|---|
| P1 Context Engine enfichable | `CTX-01` (+ `CTX-02/03/04`) |
| P2 Progressive disclosure skills | `CTX-09` (+ `ENG-15`) |
| P3 Tool Search | `CTX-10` (+ `ENG-03`) |
| P4 Compaction + prefix caching | `CTX-02`, `CTX-03`, `CTX-04`, `CTX-05` |
| P5 Sous-agents isolés parallèles | `SUB-01` (+ `ENG-06`, `ENG-14`, `SUB-02`) |
| P6 Registre multi-modèles + routing | `RTR-01..RTR-12` |
| P7 Plan/Act + agent de planification | `ENG-16` |
| P8 Checkpoints / worktrees / rollback | `SBX-07`, `SUB-02` |
| P9 Exécution sandboxée multi-backend | `SBX-01`, `SBX-02`, `SBX-03` |
| P10 Hooks de cycle de vie | `ENG-05`, `ENG-06` |
| P11 Mémoire bornée + FTS5 | `MEM-01`, `MEM-04` |
| P12 Orchestration durable + HITL | `SUB-04` |
| P13 Planification/cron + canaux | `ECO-04`, `ECO-05`, `ECO-06` |
| P14 Observabilité unifiée (OTel) | `OBS-02` (+ `OBS-01`) |

---

## 3. Classement priorisé (impact = valeur ÷ difficulté)

**Règle de tri :** impact décroissant ; à impact égal, **valeur** décroissante, puis
**prérequis de construction** (W2 → W3 → W4 → W6 → W8 → W9 → W10), puis difficulté croissante.
`S=1, M=2, L=3`.

| Impact | Nb | Features (dans l'ordre de construction) |
|---|---|---|
| **3.00** (v3/S) | 17 | `PLG-02`, `PLG-05`, `PLG-12`, `ENG-09`, `RTR-05`, `RTR-02`, `CTX-09`, `CTX-07`, `CTX-06`, `ENG-13`, `MEM-02`, `RTR-12`, `SBX-06`, `OBS-03`, `OBS-05`, `ECO-01`, `ECO-03` |
| **2.00** (v2/S) | 14 | `PLG-06`, `PLG-09`, `PLG-11`, `PLG-13`, `PLG-14`, `PLG-16`, `ENG-08`, `ENG-16`, `CTX-08`, `CTX-12`, `RTR-10`, `RTR-13`, `OBS-06`, `OBS-07` |
| **1.50** (v3/M) | 34 | `PLG-01`, `PLG-03`, `PLG-07`, `PLG-08`, `PLG-17`, `ENG-01`, `ENG-02`, `ENG-03`, `ENG-04`, `ENG-05`, `ENG-06`, `ENG-07`, `ENG-11`, `ENG-14`, `SUB-01`, `RTR-01`, `RTR-03`, `RTR-04`, `RTR-06`, `RTR-07`, `RTR-08`, `RTR-11`, `CTX-02`, `CTX-03`, `CTX-04`, `CTX-05`, `CTX-10`, `MEM-01`, `OBS-01`, `OBS-02`, `SBX-02`, `SBX-04`, `SBX-08`, `ECO-02` |
| **1.00** (v3/L, v2/M, v1/S) | 27 | `PLG-04`, `CTX-01`, `SUB-02`, `SUB-04`, `SBX-01`, `SBX-03`, `SBX-07`, `OBS-04`, `PLG-15`, `ENG-10`, `ENG-12`, `ENG-15`, `CTX-11`, `CTX-13`, `MEM-03`, `MEM-04`, `MEM-05`, `MEM-06`, `RTR-09`, `RTR-14`, `SBX-05`, `SUB-03`, `ECO-04`, `ECO-05`, `ECO-06`, `UI-01`, `SBX-09` |
| **0.67** (v2/L) | 2 | `PLG-10`, `ENG-17` |
| **0.50** (v1/M) | 1 | `MEM-07` |
| **0.33** (v1/L) | 1 | `ENG-18` |

### Top 10 (à attaquer en premier)

| # | ID | Feature | Impact | Pourquoi maintenant |
|---|---|---|---|---|
| 1 | `PLG-02` | Slots (exclusivité par capacité) | 3.00 | Prérequis de tout le modèle plugin ; `S`. |
| 2 | `PLG-12` | Isolation d'échec + repli legacy | 3.00 | Robustesse gratuite ; empêche qu'un plugin tue l'agent. |
| 3 | `PLG-05` | Registre runtime à constructeurs | 3.00 | Base des providers ; table testable/remplaçable à chaud. |
| 4 | `ENG-09` | Annulation coopérative | 3.00 | Contrat transverse de toute la boucle ; `S`. |
| 5 | `RTR-05` | Classifieur de tâche déterministe | 3.00 | Aucun LLM, testable ; porte la bascule vitesse↔réflexion. |
| 6 | `RTR-02` | Providers déclaratifs JSON | 3.00 | Ajouter un provider = config, pas du Rust. |
| 7 | `CTX-09` | Progressive disclosure des skills | 3.00 | Économie de contexte immédiate ; `S`. |
| 8 | `CTX-07` | Budget de contexte (estimate→plafond→arrêt) | 3.00 | Rend l'économie **mesurable** (pilier 5). |
| 9 | `ECO-01` | Déterminisme au système | 3.00 | Règle fondatrice : le déterministe n'est pas un tour LLM. |
| 10 | `OBS-03` | Provenance native sur tout artefact | 3.00 | Rend la traçabilité exploitable ; `S`. |

> Les 7 autres features à impact 3.00 (`CTX-06`, `ENG-13`, `MEM-02`, `RTR-12`, `SBX-06`,
> `OBS-05`, `ECO-03`) suivent immédiatement dans le même palier.

---

## 4. Quick wins (difficulté **S** + valeur **3**)

17 features livrables en ≤ 1 jour-agent chacune, à fort levier :

| ID | Feature | Workstream |
|---|---|---|
| `PLG-02` | Slots (une implémentation exclusive par capacité) | W2 |
| `PLG-05` | Registre runtime à constructeurs | W2 |
| `PLG-12` | Isolation d'échec + repli legacy | W2 |
| `ENG-09` | Annulation coopérative | W3 |
| `ENG-13` | Agents déclaratifs | W3 |
| `CTX-06` | Découverte `AGENTS.md` remontante | W6 |
| `CTX-07` | Budget de contexte | W6 |
| `CTX-09` | Progressive disclosure des skills | W6 |
| `MEM-02` | Table mémoire transactionnelle | W6 |
| `RTR-02` | Providers déclaratifs JSON | W4 |
| `RTR-05` | Classifieur de tâche déterministe | W4 |
| `RTR-12` | Garde budgétaire | W4 |
| `SBX-06` | Policy/permissions allow-ask-deny | W9 |
| `OBS-03` | Provenance native | W8 |
| `OBS-05` | Traçabilité de la décision de routage | W8 |
| `ECO-01` | Déterminisme au système | W10 |
| `ECO-03` | Comptabilité de coût | W10 |

---

*Matrice consolidée au 2026-10-15. Version machine-lisible : `docs/omega/reverse-eng/features.json`.
Synthèse d'architecture : `docs/omega/reverse-eng/OMEGA-BLUEPRINT.md`.*
