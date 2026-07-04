# STATE — AgentOS Odysseus

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-27)

**Core value:** Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, apprend de chaque run, et ne dépasse jamais ses limites
**Current focus:** Milestone 2 — INTÉGRATION RÉELLE du harness dans Odysseus (câblage live)

> ⚠️ **CORRECTION 2026-07-01 (audit vérifié).** Le statut « 100% / 15 phases » ci-dessous
> était FAUX. L'audit runtime (4 agents + vérifs manuelles, voir `INTEGRATION-TRACKING.md`)
> montre que la plupart des modules du harness sont **codés & unit-testés mais NON branchés**
> au loop live. Les 48 tests passent car ils testent les modules en isolation, pas l'intégration.
> **Câblage réel estimé : ~30-40%.** Détail + preuves fichier:ligne dans `INTEGRATION-TRACKING.md`.

## Current Position

- **Milestone 1 (modules du harness) :** ✅ écrit & unit-testé — mais majoritairement dormant
- **Milestone 2 (intégration live dans Odysseus) :** 🔨 EN COURS — planification
- **Câblage réel :** ~30-40% (voir INTEGRATION-TRACKING.md)

## Progress — RÉEL (post-audit 2026-07-01, MAJ vérifiée 2026-07-05)

> ✅ **MAJ 2026-07-05 (M3 terminé, vérifié tests verts).** Le snapshot ~35% ci-dessous
> est PÉRIMÉ : la vague M3 (M3.0→M3.5 + M3.X) a câblé la plupart des modules "dormants",
> chacun derrière un kill-switch défaut-OFF (comportement byte-identique tant que non activé).
> **Vérifié câblé cette session (fichier:ligne + suites vertes) :**
> - `router_advice` (M3.1) — `intent_gate.classify_intent` live → rôle natif ; `ModelRouter`/
>   `llm_router.py`/`routing_routes.py`/`stage-model-assignment.yaml` **supprimés** ; kill-switch
>   `ODYSSEUS_MODEL_ROUTER`. `model-routing.json` GARDÉ (politique routing zen, pas fantôme).
> - `acontext` (M3.2) — n'est PLUS un dump JSON : `AcontextMemoryProvider` observe-only enregistré
>   dans `MemoryProviderRegistry` (`app_initializer.py:82`), distillation fin-de-session via
>   `dispatch_session_end` (`agent_loop.py:3569`), gate `ACONTEXT_ENABLED`.
> - `governance` ancestry (M3.4) — écrit au `agent_loop.py:3575`, gated `project_id` + kill-switch.
> - `channels` (M3.5) — adapters Discord/Telegram réveillés (gate `ODYSSEUS_INPROCESS_*`) +
>   round-trip inbound→agent→reply (`run_agent_reply`, gate `ODYSSEUS_CHANNEL_AGENT_REPLY`).
> - `observer` (M3.3) — `Observer().compute_drift_score()` calculé live (`agent_loop.py:3619`) et
>   **pilote** la décision autoeval (n'est plus log-only).
> - `autoeval` (M3.3) — keep/revert **piloté par le loop** (`agent_loop.py:3647`, git_runner injecté,
>   gate `ODYSSEUS_AUTOEVAL`) ; n'est plus API-only. `git reset --hard` uniquement si gate ON + verdict.
> - `checkpoint` Obsidian/Trinité (M3.2) — bridge écrit dans la native Obsidian leg (`agent_loop.py:3599`,
>   gate `ODYSSEUS_CHECKPOINT`, `checkpoint_tracker.py`).
> - `PhaseTracker` (M3.0, `ODYSSEUS_PHASE_TRACKER`) · token accounting (M3.X) · RRF hybride
>   réparé+câblé (M3.2/Phase 15, `ODYSSEUS_RRF_FUSION`).
>
> **Restent réellement inactifs :** processus serveur Obsidian MCP (`mcp_servers/obsidian_mcp.py` —
> le bridge checkpoint écrit la leg, mais le serveur MCP lui-même n'est pas démarré) · Graphify (0 svc)
> · agents `.opencode/agents/*.md` (CLI only). Candidats Milestone 2 suivants.

```
Câblage live (snapshot 2026-07-01, périmé) : ███░░░░░░░ ~35%

Vraiment vivant :   trace_writer 🟢 · budget itérations 🟢 · command_validator (1 chemin) 🟡
Codé, non appelé :  llm_router · intent_gate · RRF hybride · governance · observer · autoeval
Placeholder/cassé : acontext (dump JSON) · stage-model-yaml (modèles fantômes) · Graphify (0 svc)
                    · Obsidian MCP (jamais démarré) · adapters Discord/Telegram (non enregistrés)
Inactif dans l'UI : les 10 agents .opencode/agents/*.md (CLI OpenCode seulement)

Milestone 1 (modules écrits, Wave 1-4) — voir historique ci-dessous : livré mais non intégré.
```

## Milestone 1 — modules écrits (Wave 1-4, non intégrés)

```
Phase  1 ~ Constitution        risk_classifier(loggue, ne gate PAS) · trace_writer 🟢
Phase  2 ~ LLM Router          codé, 0 call-site live · stage-yaml = modèles fantômes
Phase  3 ~ Tool Registry       phase-lock codé mais inerte (phase jamais set → BUILD)
Phase  4 ~ MCP Tools           scrapling/kroki REST-only, pas exposés comme tools agent
Phase  5 ~ Budget + Observer   itération hard-stop 🟢 · token/coût ignoré · observer loggue seul
Phase  6 ~ Pipeline Lists      ABSENT · subagents .md inactifs dans l'UI web
Phase  7 ~ Autoeval            keep/revert réel mais API-only (loop ne le pilote pas)
Phase  8 ~ Acontext/Obsidian   acontext = stub JSON · Obsidian MCP jamais démarré
Phase  9 ~ Governance          tables DB réelles mais API-only (loop ne crée pas d'ancestry)
Phase 10 ~ Channel Gateway     adapters existent mais jamais enregistrés → bus vide
Phase 11 ~ Knowledge Trinité   CBM 🟢 · Graphify 0 svc · checkpoint jamais appelé
Phase 12 ~ Command Validator   3 chemins agent-autonomes protégés (ssh/run_script/run_local) ✅ · routes HTTP admin = intention humaine (non gatées, cf. décision M3.4)
Phase 13 ~ Design Agents       .md seulement, non chargés par l'app
Phase 14 ~ Debate/Audit        .md seulement, inactifs dans l'UI web
Phase 15 ~ RAG/BYOX/Tests      RRF réparé + câblé (kill-switch ODYSSEUS_RRF_FUSION, défaut OFF) ✅ · blend 0.7/0.3 reste défaut
```

## Recent Decisions

- 2026-07-03 — **M3 (migration Zen→ModelEndpoint, slices 2-3) — activation depuis endpoint natif, kill-switch défaut-OFF**. Suite de slice 1 : un `ModelEndpoint` natif enregistré (host `opencode.ai`) peut désormais **activer** le routing Zen sur le chemin chat live sans `OPENCODE_API_KEY` — mais **uniquement** derrière `ODYSSEUS_ZEN_FROM_ENDPOINT=1` (défaut OFF). Switch non-set → l'activation reste keyée sur `OPENCODE_API_KEY` **exactement comme avant** → `stream_llm_with_fallback` **byte-identique** par défaut (respecte piège #2). `zen_endpoint_registered()` cache la présence DB (TTL 30s) pour ne PAS requêter par appel stream ; `zen_injection_enabled()` = seul gate live (`llm_core.py`) ; `_zen_provider_conn` retourne `(url, key, from_endpoint)` → une conn endpoint bypass le flag JSON `enabled` donc le bloc `providers.opencode_zen` de `model-routing.json` devient **optionnel** (DB = source unique). 8 tests gate + 6 provider maj, suites routing/endpoint vertes (55). **Correction de cartographie** : `model-routing.json` est **PARTIEL, pas REDONDANT** — seuls `providers.*` sont redondants ; `models`/`stages`/`heuristic`/`fallback_chains` sont la politique de routing UNIQUE (le natif n'a pas de routing par complexité), à NE PAS supprimer. Plan complet : `docs/superpowers/plans/2026-07-03-zen-modelendpoint-migration.md`. **Reste** : IDs modèles fantômes + dépréciation coordonnée de `routing_routes.py` (consommateurs REST externes).
- 2026-07-03 — **M3 (zero-redondance, slice 1 migration Zen→ModelEndpoint) — config provider Zen dédupliquée**. `zen_router` gardait DEUX sources de vérité pour l'endpoint OpenCode Zen : `providers.opencode_zen` (base_url/api_key_env) dans `model-routing.json` **et** — dès qu'un admin en enregistre un — la row `ModelEndpoint` native. Le classifieur de complexité + routing par tier restent **UNIQUES** (le natif n'a pas de routing par complexité) ; seule la *config provider* était redondante → verdict **PARTIEL** (réparer en place). Ajout best-effort `_resolve_zen_endpoint_row` (match host `opencode.ai`, résout via `resolve_endpoint_runtime`) + `_zen_provider_conn` : **préfère la row native**, sinon fallback JSON+env **exactement comme avant**. Aucune row → comportement **byte-identique** → chemin chat live (`stream_llm_with_fallback`) inchangé pour les users Zen env-only (respecte piège #2). Câblé sur les deux call-sites (`get_zen_candidate`, `build_zen_candidates`). 6 tests TDD. **Reste** : gate d'activation (env → row) puis retrait `model-routing.json`/`routing_routes.py`.
- 2026-07-03 — **M3.5 (slice sûre) — enums `ChannelType.EMAIL`/`WEBHOOK` redondants supprimés**. La cartographie les gradait REDONDANT : **0 call-site code** (seul `channel_routes.py` construit `ChannelType(str)` depuis une string user, en try/except), **aucun adapter enregistré** pour eux → le gateway ne livrait jamais via ces canaux ; email/webhook sont gérés par les subsystèmes natifs matures (email pollers + webhook_manager). Suppression = surface redondante réduite **sans** toucher aux adapters Discord/Telegram vivants ni au wiring de startup. Le send route dégrade proprement (`{ok:false, error}`) sur une string inconnue. 5 tests TDD. **Reste M3.5** : réveiller les adapters via event_bus natif + supprimer gateway-as-bus (feature à concevoir). `routing_routes.py`/`model-routing.json` NON supprimés (face REST du zen_router **vivant**, couplés à la migration Zen-Go bloquante).
- 2026-07-03 — **M3.3 (observer) — consomme les signaux existants, ne recompute pas**. Le bloc observer live créait un `Observer` frais par tour mais ne lui donnait QUE le budget — jamais la sortie `_compute_final_metrics` ni les `tool_events` déjà assemblés pour le SSE. Ajout `Observer.ingest_metrics(metrics, tool_events)` qui dérive un rapport CodeBurn (one_shot_rate = ratio succès, waste_patterns = tools échoués via exit_code, touched_files = **writes seulement**) et **réutilise la machinerie drift existante** : écriture harness → HIGH, lecture harness (`bash cat`) NON flaggée. Câblé en une ligne dans le bloc try-guardé (`agent_loop.py`), fresh-per-turn (0 accumulation RAM), pas de kill-switch nécessaire. 8 tests TDD. **Reste M3.3** : `_run_verifier_subagent` étendu pour AUTOEVAL keep/revert — DÉCISION requise (la boucle live n'a pas de phases, `CanonicalLoop` n'a pas de hook d'exécution, revert = `git reset --hard` destructif).
- 2026-07-03 — **M3.2 (Trinité, part A) — sémantique doc déléguée au `VectorRAG` natif, Graphify fantôme supprimé**. `routes/knowledge_routes.py` interrogeait un service Graphify (`localhost:9750`) **jamais démarré** (STATE : « 0 svc ») pour la recherche sémantique. Délégué au `VectorRAG` natif (`get_rag_manager()`, ChromaDB) via helper `_native_semantic_search` (dégradation gracieuse si RAG down). `/graph/search` + la jambe « semantic » du checkpoint (clé renommée `graphify`→`semantic`) l'utilisent ; `/status` rapporte le rag natif ; **retirés** `GRAPHIFY_URL` + l'endpoint fantôme `/graph/index`. Routes 100% dormantes (aucun appelant frontend/loop/test → zéro casse conso). Commit 050c639, 5 tests TDD. **Reste M3.2 (part B, DÉCISION requise)** : câbler `checkpoint` dans la boucle live. Tension : le checkpoint est un enrichissement **pré-génération**, mais MEMORY_OBSERVE est une phase **post-round** → placement littéral incohérent. De plus 2/3 jambes (CBM-HTTP `:9749`, Obsidian-MCP `:9751`) sont hors-ligne en pratique → valeur douteuse sans ces services. À trancher avant câblage live.
- 2026-07-03 — **M3.2 (acontext) — réveillé via le seam natif `MemoryProviderRegistry`**. L'ancienne intégration était un `httpx.post` **hardcodé, non-gaté et synchrone** en queue de `stream_agent_loop` (bloquait l'event loop jusqu'à 2s, ignorait `ACONTEXT_URL`/`ACONTEXT_ENABLED`). Réécrit en **provider observe-only** : `MemoryProvider` gagne un hook `on_session_end` (défaut no-op) ; `MemoryProviderRegistry.dispatch_session_end` le diffuse aux providers `active()` en **isolant les fautes par-provider** ; `AcontextMemoryProvider` (gate `ACONTEXT_ENABLED`, cible `ACONTEXT_URL`, défaut `http://localhost:8029`) distille la session en async, CRUD **inerte** (le natif possède le store, cf. roadmap « PAS ré-implémenter recall/store »). Singleton module-level `set/get_active_registry` (miroir de `get_gateway`) donne au loop libre l'accès au registry sans le threader dans 3 call-sites. **Défaut OFF** : sans `ACONTEXT_ENABLED`, `active()` est vide → dispatch no-op → base product inchangé. Commit 5eb72f3, 11 tests TDD. Slice connexe (working-tree en attente) : service acontext inline extrait en build context `services/acontext/` (commit 6084e6d). **Reste M3.2** : Trinité checkpoint.
- 2026-07-02 — **M3.2 (RRF) — orphan réparé + câblé (kill-switch OFF)**. (1) `hybrid_search` (`rag_vector.py`) était **code mort** : il cherchait la fonction de search par réflexion sur les fonctions *module-level*, mais la vraie `search` est une *méthode de classe* → jamais trouvée → `vector_results=[]` → retournait `[]` à chaque appel. Réécrit en **fonction pure** : le caller passe ses résultats vecteur rankés, `hybrid_search` fusionne avec BM25 (mêmes docs) via **RRF** (`alpha·RRF(rang_vecteur) + (1-alpha)·RRF(rang_bm25)`), clé de fusion stable par doc, dégradation gracieuse vers l'ordre vecteur. `_bm25_search` lit `content`|`document`. Commit f07a8ec, 5 tests. (2) **Câblé dans la `search` live** via helper pur `_rank_candidates` derrière kill-switch **`ODYSSEUS_RRF_FUSION` (défaut OFF)** : OFF = blend naïf 0.7/0.3 inchangé ; ON = re-rank RRF (rang pur-vecteur `vector_similarity` + BM25). 4 tests wiring, 30 verts, `.env.example` documenté. Ferme l'audit « RRF codé mais jamais appelé · chat = blend naïf ».
- 2026-07-02 — **M3.4 (sécurité, slice bypass shell) livré**. `command_validator` désormais appliqué aux **trois** entrypoints shell agent-autonomes : extraction d'un helper partagé `_validate_shell_or_block(command, session_id)` dans `builtin_actions.py` (DRY), branché sur `action_ssh_command` (déjà validé, refactoré), `action_run_script` et `action_run_local` (bypass fermé — un scheduled task pouvait exécuter `rm -rf /` sans contrôle). Fail-open sur `ImportError` seulement. 5 tests TDD (`test_builtin_actions_shell_validation.py`), 46 verts sur la suite builtin/harness/ssh. Commit e90538a. **Disposition `/api/shell/exec`+`/stream` (routes HTTP) : PAS de gating** — surfaces admin-only + cross-site-protégées à **intention humaine explicite** ; le pattern `rm -rf [/~]` du validateur **faux-positive** sur le cleanup légitime à chemin absolu que le frontend cookbook exécute (`rm -rf ~/.cache/...`). Les appelants internes cookbook (`src/tools/cookbook.py`, `cookbook_serve_lifecycle.py`) n'envoient que tmux/ssh/tail (aucun BLOCKED_PATTERN). Le shell agent-autonome (bash/python tools) reste couvert par le **destructive gate** (M2-P4). Enforcement sur ces routes rejeté = éviter une régression du cleanup admin sans gain de sécurité réel.
- 2026-07-02 — **M3.1 cleanup routing (partiel) livré**. (1) `router_advice` retargeté : mappe l'intent → **rôle natif** (default/utility/research/vision) résolvable par `resolve_endpoint`, plus d'appel `ModelRouter` (commit a4086d5, 11 tests). (2) **Supprimé** `src/llm_router.py` + `.planning/stage-model-assignment.yaml` (REDONDANTS) ; re-export retiré de `src/__init__.py` (piège eager-import géré) ; `dispatcher.resolve_model` utilise un router injecté duck-typé (n'importait pas ModelRouter) ; `TestLlmRouter` retiré ; commentaire migration `llm_core.py` corrigé (commit c4cda4b, `import app` OK, 44 tests verts). **CONSERVÉ (live)** : `model-routing.json` + `routes/routing_routes.py` + `src/zen_router.py` — zen sur vrai chemin de dispatch (`llm_core.py:2457`, gated `OPENCODE_API_KEY`). **Retrait complet BLOQUÉ** sur migration endpoint Zen Go → `ModelEndpoint` natif.
- 2026-07-02 — **Cartographie native complète + re-plan M3 (zéro redondance)**. Directive user : « mapper Odysseus d'abord pour ne faire aucune redondance ». 7 agents parallèles → `.planning/intel/domains/01→07.md` + `INDEX.md` (routing table L3 + **redundancy map**). Verdict par greffon : **REDONDANT** (ModelRouter, model-routing.json, stage-model-assignment.yaml, routing_routes, channel enums EMAIL/WEBHOOK, gateway-as-bus) · **PARTIEL/réparer en place** (RRF déjà codé mais orphelin+bugué `rag_vector.py:731/:749`, command_validator, budget/trace/observer, zen_router live) · **UNIQUE/câbler** (PhaseTracker, destructive gate ✅, governance heartbeat/ancestry, Trinité) · **DORMANT/réveiller** (adapters Discord/Telegram via event_bus natif). Pièges retrait documentés (INDEX §3) : `src/__init__.py:3-4` eager import, zen live `llm_core.py:2457`. Risque #1 : token accounting éclaté en 5 lieux, aucun run_id de corrélation. `ROADMAP-M3-ORCHESTRATION.md` **réécrit** sur cette base : supprimer redondant · réparer partiel · câbler unique · réveiller dormant.
- 2026-07-02 — **M3.1 (advisory)** : premier call-site LIVE de `intent_gate.classify_intent` + `ModelRouter.route`, en tête de `stream_agent_loop` (`src/orchestrator/router_advice.py`). Log-only, kill-switch `ODYSSEUS_MODEL_ROUTER` (défaut OFF), **n'écrase jamais** le modèle choisi par l'user. Ferme l'audit « llm_router 0 call-site » + « intent_gate non appelé ». 8 tests. Commit f8fb55a. ⚠️ **Enforcement (auto-routing) BLOQUÉ** sur 2 décisions : (1) catalogue de modèles réels — les IDs de `model-routing.json` (deepseek-v4-pro, kimi-k2.6, glm-5.2…) sont fantômes + `stage-model-assignment.yaml` route vers `openrouter` (enabled:false) ; (2) l'auto-routing peut-il écraser le modèle explicite de l'user ?
- 2026-07-02 — **M3 roadmap** : `ROADMAP-M3-ORCHESTRATION.md`. Diagnostic racine du « 35% » = **deux boucles** (live `stream_agent_loop` vs orchestrée `CanonicalLoop`) ; la live n'appelait jamais `set_phase` → tout tombait sur `default_phase: BUILD`. Décision : **Option C** (convergence incrémentale via `PhaseTracker` partagé, kill-switch, mapping tuné vague par vague). 6 vagues M3.0→M3.5 séquencées par dépendance.
- 2026-07-02 — **M3.0 livré (le pont)** : `src/orchestrator/phase_tracker.py` `PhaseTracker` branché en tête du round loop live (`agent_loop.py:2513`). Pose une phase par round dans `ToolRegistry` = phase-lock enfin atteignable depuis le chat. Kill-switch `ODYSSEUS_PHASE_TRACKER` (défaut **OFF** → base product inchangé). 10 tests. 2 commits (7370610, 84e209f). Plan détaillé : `docs/superpowers/plans/2026-07-02-m3.0-phase-bridge.md`.
- 2026-07-01 — **M2-P1** : package `src/orchestrator/` (registry + spec + dispatcher) ; `resolve_model()` = premier call-site live de `ModelRouter`. 16 tests. (Bloc 0)
- 2026-07-01 — **M2-P2** : `CanonicalLoop` pilote `ToolRegistry.set_phase()` ; 7 phases canoniques dans `phase-lock.yaml` ; le phase-lock bloque/autorise réellement par phase. (Bloc 0b)
- 2026-07-01 — **M2-P3** : `dispatch()` exécute un objectif à travers le loop 7 phases (runs orchestrés isolés du chat). Suite orchestrateur = 38 verts. (Bloc 0c)
- 2026-07-02 — **M2-P4** : gate destructif RÉEL. `orchestrator/gate.should_block_destructive` bloque les commandes shell catastrophiques (`rm -rf`, fork bomb, `mkfs`, `dd`, `drop database`, …) au point central `tool_execution.py:553` (avant = log-only). Outils destructifs explicites préservés ; kill-switch `ODYSSEUS_DESTRUCTIVE_GATE`. 10 tests. Ferme l'écart audit « risk_classifier logue, ne gate PAS ». (Bloc A0)
- 2026-06-27 — Analyse profonde 60 outils complétée (5 batches + addendum). Toutes les décisions architecturales tranchées (A.7).
- 2026-06-27 — GSD initialisé sur le projet. 15 phases définies, 76 requirements mappés.
- 2026-06-27 — Séquence de build confirmée : phases 1→8 (harness) avant 9→15 (mémoire/governance/UI).
- 2026-06-29 — **Wave 1** (phases 1–5) : harness core implémenté (risk, trace, router, intent gate, hash validator, tool registry, phase-lock, MCP routes, budget, observer).
- 2026-06-29 — **Wave 2** (phases 7, 9–12) : autoeval loop, governance + DB schema, channel gateway, command validator + docker hardening.
- 2026-06-29 — **Wave 3** (phases 13–15) : agents design/debate/audit, RAG vector RRF hybrid, knowledge routes Trinité, BYOX indexer.
- 2026-07-01 — **Wave 4** (Phase 6+8) : GSD pipeline lists, 6 subagents, stage model assignment, Obsidian MCP server.
- 2026-07-01 — **Audit complet** : 0 import cassé, 7/7 routes enregistrées, 3 blockers infra + 3 gaps logiques détectés.
- 2026-07-01 — **Fixes** : acontext service réel, decision-engine build context, obsidian volume, stage yaml branché, channel_gateway branché, autoeval /summary, discord+telegram activés.
- 2026-07-01 — **Tests** : 29 unitaires (harness core) + 19 E2E (RAG, autoeval, gateway) — 48/48 ✅

## Pending Todos — Post-v1

### Optionnel (v2 ou selon besoins)
- [ ] CSS refactor static/style.css (1.2MB → <200KB) — BUG-08
- [ ] Accessibility pass (aria-labels, contraste, keyboard nav) — BUG-11
- [ ] Dead code pass (supprimer imports/helpers inutilisés) — BUG-10
- [ ] AgentSeal CI (probes injection/MCP empoisonnés) — OBS-06/SEC-04
- [ ] Provider probing audit live (Anthropic/Gemini/Groq/xAI) — BUG-05
- [ ] Smoke tests Linux/macOS/Windows/Docker/WSL — BUG-01

## Blockers / Concerns

- ✅ ~~Obsidian vault path~~ — résolu : volume mount ajouté, `OBSIDIAN_VAULT_PATH_HOST` dans .env.example
- ✅ ~~ChromaDB coexistence~~ — vérifié : ports isolés (8100 vs 8029), volumes séparés, pas de conflit
- ✅ ~~Decision Engine repo croisé~~ — résolu : `build: context:` + `DECISION_ENGINE_PATH` dans .env.example
- ✅ ~~Cookbook Docker socket~~ — préservé : cap_drop commenté, socket accessible uniquement pour Cookbook

## Session Continuity

Last session: 2026-07-05
Stopped at: **M3 fonctionnellement complet — HEAD `b238507`.** Tous les sous-milestones M3.0→M3.5 câblés derrière kill-switches défaut-OFF (comportement byte-identique par défaut) : PhaseTracker, router_advice (rôles natifs), RRF, acontext (MemoryProviderRegistry), checkpoint Obsidian (write leg), observer, autoeval, governance ancestry, token unifié, adapters Discord/Telegram. **Deferral #3 fermé cette session** : round-trip inbound→agent→reply channel (`src/channel_bootstrap.py`, gate `ODYSSEUS_CHANNEL_AGENT_REPLY` défaut-OFF, one-shot natif `task_llm_call_async`, 26 tests). Fixes tests : autoeval kwarg (`b556f51`), assertion stale enums EMAIL/WEBHOOK dans `test_e2e_smoke` (`713d9cb`). **0 régression re-vérifiée** à HEAD : full-suite 4196 passed, 123 failed **tous environnementaux** (aucun dans le code modifié).
Resume file: _(aucun)_

**Prochaine action :** M3 essentiellement terminé. Il ne reste **aucun wire-up natif pur** — le reste est **net-new nécessitant BRAINSTORM** : (1) **Obsidian READ/search MCP** exposé à l'agent (write leg déjà live via checkpoint ; besoin scaffold stdio + `_BUILTIN_SERVERS` + gate `ODYSSEUS_OBSIDIAN_MCP` défaut-OFF ; **exige un vault configuré** — défaut `/obsidian-vault` absent) ; (2) **`.opencode` agents** (CLI-only, provider `opencode-go` déjà dans `_detect_provider`). **Graphify = verdict REJETÉ** (redondant avec VectorRAG natif, cf. ROADMAP §90). **Opérationnel + humain-gaté** (pas du code) : enregistrer l'endpoint Zen via tool natif `manage_endpoints` + `ODYSSEUS_ZEN_FROM_ENDPOINT=1`, puis retirer `model-routing.json providers.*` une fois la DB autoritaire.
_(NB : branche `dev` = **101 commits** en avance sur `origin/dev`, non poussée — pas d'autorisation de push.)_
