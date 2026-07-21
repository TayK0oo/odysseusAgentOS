# AUDIT — 00 BASELINE (Prétendu vs À vérifier)

> **Statut :** Phase 0 de la mission « Rescan total & Table rase », HEAD `bca398a` (branche `dev`).
> **Règle zéro (rappel).** Le milestone M3 est déclaré fonctionnellement complet dans `STATE.md`.
> Ce document ne l'endosse PAS : il capture ce que les docs *prétendent* avoir livré, et pour
> chaque item enregistre **exactement quoi vérifier** en Phase 1, avec fichier + ligne cibles.
> Un test unitaire vert ne prouve rien à ce stade — seule la lecture directe du chemin live fait foi.
> **Preuve du besoin :** le fix `035d2f6` (« import app cassé depuis `b4c56e4` ») et son test smoke
> boot subprocess (`37a511c`, `tests/test_app.py::TestAppBoots`) ont exposé que 48+ tests unitaires
> peuvent passer alors qu'`import app` explose en `ModuleNotFoundError`. Zéro confiance justifiée.

---

## 1. Snapshot du dépôt à HEAD

| Item | Valeur | Source |
|---|---|---|
| HEAD | `bca398a` — `docs(env): document all M3 kill-switches; drop dead OBSIDIAN_MCP_ENABLED` | `git log -1` |
| Branche | `dev` | `git status` (« dev...origin/dev », clean) |
| Commits ahead / behind origin/dev | 0 / 0 | `git rev-list --count HEAD ^origin/dev` = 0 |
| Discrepancy détectée | `STATE.md:140` prétend « **101 commits en avance sur `origin/dev`**, non poussée » — HEAD est **en fait synchronisé** (`0 ahead`). Soit push effectué depuis, soit la ligne est stale. **À revalider avant tout push destructif.** |
| Racine | `C:\Users\ttmdu\Documents\GitHub\odysseusAgentOS` | environnement |
| Arborescence racine (dirs, 21) | `.github/`, `.opencode/`, `.planning/`, `companion/`, `config/`, `core/`, `data/`, `docker/`, `docs/`, `integrations/`, `licenses/`, `logs/`, `mcp_servers/`, `routes/`, `scripts/`, `services/`, `specs/`, `src/`, `static/`, `tests/`, `tools/` | `ls C:/…/odysseusAgentOS` |
| Fichier majeur | `app.py` — 53.7 KB à la racine (point d'entrée FastAPI) | `ls` |
| CBM index | `nodes=11749`, `edges=38126` (933 Files, 5230 Functions, 1564 Methods, 733 Routes, 528 Classes) | `list_projects()` + `get_architecture()` |
| Doc-vérité vision (**hors dépôt**) | `C:\Users\ttmdu\Documents\GitHub\ConfigOpenCodeNew\docs\ANALYSE-OUTILS-UNIFIE.md` (836 lignes, 60 outils, 5 couches) | lecture directe |
| ADR présents dans CBM | oui (`adr_present: true`, hook Context Intel) | hook startup |

### 1.1 Commits M3 clés (log -50 filtré)

Récupérés via `git log --oneline -50`. Ordonnés chronologiquement descendants.

| SHA | Sujet | Prétention |
|---|---|---|
| `bca398a` | `docs(env): document all M3 kill-switches; drop dead OBSIDIAN_MCP_ENABLED` | Documente les 12 kill-switches M3 dans `.env.example` |
| `5860629` | `docs: record Obsidian read-tools shipped + correct false "0 dangling ref"` | Obsidian read/search MCP shipped |
| `37a511c` | `test(app): add subprocess boot smoke test for import app` | Boot smoke test dans `tests/test_app.py` |
| `ef23b78` | `feat(obsidian): expose read/search vault tools via stdio MCP (gated OFF)` | scaffold stdio + list_notes/get_note/search_notes |
| `035d2f6` | `fix(app): remove dead routing_routes mount left by b4c56e4` | Fix bug d'`import app` |
| `713d9cb` | `fix(test): smoke test asserted removed ChannelType.EMAIL/WEBHOOK` | Fix régression tests |
| `b556f51` | `fix(test): AutoevalLoop.evaluate_last_change kwarg is score_baseline` | Correction kwarg |
| `4ee786b` | `feat(channels): inbound->agent->reply round-trip (kill-switched OFF)` | M3.5 round-trip channel |
| `b4c56e4` | `refactor(M3.1): remove redundant /api/route* wrapper (routing_routes.py)` | Suppression wrapper REST |
| `94f7cb2` | `feat(M3.X): unified token accounting behind kill-switch` | M3.X comptabilité tokens unifiée |
| `a6a82ad` | `feat(M3.2): Trinité checkpoint bridge persisted at MEMORY_OBSERVE` | Checkpoint Obsidian bridge |
| `786c6de` | `feat(channels): M3.5 wake dormant Discord/Telegram adapters in-process` | Adapters réveillés |
| `f96858d` | `feat(M3.3): AUTOEVAL keep/revert verifier gated behind kill-switch` | Autoeval gated |
| `abe389a` | `feat(governance): make ancestry live via kill-switched loop bridge (M3.4)` | Governance ancestry live |
| `18c37f9` | `feat(traces): per-run correlation id shared across traces, budgets, metrics` | run_id de corrélation |
| `48a79ee` | `fix(zen): replace phantom deepseek-v4-flash-free fallback with confirmed…` | Fix ID modèle fantôme |
| `f95a120` | `feat(mcp): wake Scrapling as a real MCP server (Bloc F)` | Scrapling MCP live |
| `586ba3d` | `feat(tools): wire render_diagram Kroki agent tool (Bloc F)` | Kroki agent tool |
| `88ab463` | `feat(zen): kill-switched endpoint-triggered injection (default OFF)` | Zen→ModelEndpoint slice 2-3 |
| `7408fbf` | `refactor(zen): dedup Zen provider config via native ModelEndpoint` | Zen→ModelEndpoint slice 1 |
| `009ab23` | `feat(observer): consume metrics + tool_events for live drift score …` | Observer.ingest_metrics |
| `5eb72f3` | `feat(M3.2): wake acontext via native MemoryProviderRegistry seam` | Acontext observe-only |

---

## 2. Objectif final (rappel utilisateur — 8 axes)

Transcrit ici pour que Phase 3 puisse le croiser à la matrice de couverture :

1. **Orchestration lisible** : lancement de plans entiers par subagents, spécialisation par outil.
2. **Modularité totale** : chaque brique interchangeable sauf le cœur.
3. **Routing intelligent des modèles** : sélection auto par tâche via `ModelEndpoint` natif + profils (`anthropic ×2` / `openrouter`).
4. **Mémoire & contexte** : Trinité (CBM structure + Graphify sémantique + Obsidian mémoire) câblée cohéremment.
5. **Observabilité complète** : correlation-id par run, traces, budgets, tout visible.
6. **Adaptativité par type de projet** : code first, extensible aux autres domaines.
7. **Auto-évolution** : auto-évaluation des failles, R&D auto, veille dans un second cerveau Obsidian.
8. **Contraintes d'ingénierie** : native-first, kill-switch défaut-OFF (byte-identique tant que OFF), self-hosting complet via Docker Compose.

---

## 3. Prétentions M3 (source: `STATE.md`, `ROADMAP-M3-ORCHESTRATION.md`, `INTEGRATION-TRACKING.md`)

Chaque ligne = prétention textuelle des docs internes + **ce qu'il faudra vérifier en Phase 1**.

### 3.1 M3.0 — PhaseTracker (le pont)

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| `stream_agent_loop` porte une phase par round via `PhaseTracker` partagé → `set_phase` | `ROADMAP-M3:65`, `STATE.md:96` | Lire `src/orchestrator/phase_tracker.py` (existence + logique), lire `src/agent_loop.py:2513` (call-site prétendu) et confirmer que `set_phase` est réellement invoqué à chaque round loop. Kill-switch `ODYSSEUS_PHASE_TRACKER` **défaut OFF** vérifié dans `.env.example:207`. |
| 10 tests dédiés | commits `7370610`, `84e209f` | Trouver `tests/test_orchestrator_phase_tracker.py` (ou équivalent), compter les tests. |
| Kill-switch OFF ⇒ base product byte-identique | `ROADMAP-M3:65` invariant | Confirmer que sans la variable env, `phase_tracker` retourne no-op (grep pour l'appel). |

### 3.2 M3.1 — Cleanup routing

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| `src/llm_router.py`, `.planning/stage-model-assignment.yaml` **supprimés** | `ROADMAP-M3:197` | `git log --diff-filter=D --name-only` doit lister les 2 fichiers. Absence à HEAD confirmée par `ls`. |
| Re-export `src/__init__.py` nettoyé (piège INDEX §3.1) | `STATE.md:92`, `INDEX.md §3.1` | Lire `src/__init__.py` — plus de `from .llm_router import ModelRouter`. |
| `routing_routes.py` + son test **supprimés** | `ROADMAP-M3:197` (« correction 2026-07-05 ») | Absence à HEAD à vérifier ; mais `app.py:803` avait un mount mort → **bug d'import** persistant jusqu'à `035d2f6`. Confirmer que le mount est effectivement retiré à HEAD. |
| `router_advice` retargeté rôles natifs (`default/utility/research/vision`) | `STATE.md:92`, `ROADMAP-M3:73` | Lire `src/orchestrator/router_advice.py` et vérifier l'absence d'appel à `ModelRouter.route`. Confirmer les rôles listés. |
| **Non supprimé (conservé délibérément)** : `model-routing.json` = politique routing UNIQUE (models/stages/heuristic/fallback_chains). | `ROADMAP-M3:84`, `STATE.md:30` | Confirmer la présence du fichier + inventorier les IDs de modèles (fantômes ou réels ?). |
| Bug bloquant `import app` levé par la suppression `b4c56e4` + non-attrapé | `STATE.md:137`, `ROADMAP-M3:197` | Vérifier `tests/test_app.py::TestAppBoots` (smoke boot subprocess `37a511c`). |

### 3.3 M3.2 — Trinité (Connaissance)

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| RRF (`hybrid_search`) réparé + câblé dans `search` live derrière `ODYSSEUS_RRF_FUSION` OFF | `STATE.md:90`, `ROADMAP-M3:199` | Lire `src/rag_vector.py:731` (fonction pure prétendue) + `rag_vector.py:378` (call-site). Vérifier kill-switch. |
| Acontext = MemoryProvider observe-only via `MemoryProviderRegistry` | `STATE.md:89`, `agent_loop.py:3569` (prétendu call-site `dispatch_session_end`) | Lire le module `memory_provider.py`, confirmer `AcontextMemoryProvider` + `dispatch_session_end` sur `agent_loop.py:3569`. Confirmer suppression du `httpx.post` synchrone hardcodé. |
| Graphify fantôme **supprimé** (route/graphify + `GRAPHIFY_URL`) | `STATE.md:88` | `grep graphify` dans `src/`, `routes/` → 0 hit attendu ; `.env.example` sans `GRAPHIFY_URL`. |
| Sémantique doc déléguée à `VectorRAG` natif via `_native_semantic_search` | `STATE.md:88` | Trouver le helper dans `routes/knowledge_routes.py` et confirmer qu'il appelle `get_rag_manager().search`. |
| Trinité checkpoint : bridge kill-switché `ODYSSEUS_CHECKPOINT` OFF, persisté au MEMORY_OBSERVE post-round via `create_note` Obsidian primitive | `STATE.md:42`, `ROADMAP-M3:199` | Lire `src/orchestrator/checkpoint_tracker.py` + `agent_loop.py:3599` (call-site). Vérifier best-effort try-guarded. |

### 3.4 M3.3 — Qualité (Observer + Autoeval)

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| `Observer.ingest_metrics(metrics, tool_events)` consomme le SSE metrics, ne recompute pas | `STATE.md:38`, `ROADMAP-M3:100` | Lire `src/observer.py` (ou équivalent) — confirmer `ingest_metrics`. Confirmer câblage dans `agent_loop.py:3619` (call-site prétendu). |
| Autoeval keep/revert piloté par le loop derrière `ODYSSEUS_AUTOEVAL` OFF, `git_runner` injectable | `STATE.md:40`, `ROADMAP-M3:200` | Lire `src/orchestrator/autoeval.py`, chercher `decide_keep_or_revert`, `apply_autoeval`. `agent_loop.py:3647` (call-site). Confirmer que revert = `git reset --hard` gated. |
| 19 tests autoeval | commit `f96858d` | Trouver `tests/test_orchestrator_autoeval.py`. |

### 3.5 M3.4 — Sécurité & Governance

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| Bypass shell corrigés : `action_run_script` + `action_run_local` + `action_ssh_command` passent par `_validate_shell_or_block(command, session_id)` | `STATE.md:91` (commit `e90538a`) | Lire `src/builtin_actions.py` — confirmer le helper `_validate_shell_or_block` + les 3 call-sites. |
| Governance ancestry live via kill-switch `ODYSSEUS_GOVERNANCE_ANCESTRY` OFF, no-op sans `project_id` | `STATE.md:35`, `ROADMAP-M3:201` | Lire `src/orchestrator/ancestry_tracker.py` + `agent_loop.py:3575`. Vérifier réutilisation de `create_task_with_ancestry` natif. |
| Fusion des 2 listes de patterns (`command_validator.BLOCKED_PATTERNS` vs `risk_classifier.DESTRUCTIVE_PATTERNS`) **rejetée** (2 politiques distinctes) | `ROADMAP-M3:118`, `STATE.md:91` | Confirmer les deux listes toujours présentes ; rappeler dans le rapport final que la "duplication" est intentionnelle. |
| Nom fantôme `run_command` retiré des 3 guards | `ROADMAP-M3:115` | Grep pour `run_command` dans `orchestrator/gate.py`, `risk_classifier.py`, `tool_registry.py`. |

### 3.6 M3.5 — Canaux

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| Enums `ChannelType.EMAIL` / `WEBHOOK` **supprimés** (0 caller) | `STATE.md:86` | Grep sur l'énum. |
| Adapters Discord/Telegram réveillés in-process via `ODYSSEUS_INPROCESS_DISCORD` / `ODYSSEUS_INPROCESS_TELEGRAM` OFF | `STATE.md:203`, `ROADMAP-M3:203` | Lire `src/channel_bootstrap.py`, confirmer `bootstrap_channels`, `make_inbound_handler`, `deliver_outbound`. Vérifier câblage dans `app.py::_startup_event` → `_startup_channel_adapters`. |
| Round-trip inbound→agent→reply derrière `ODYSSEUS_CHANNEL_AGENT_REPLY` OFF | `STATE.md:132`, `ROADMAP-M3:140` | Lire `run_agent_reply` (one-shot natif via `task_llm_call_async`). Vérifier fallback gateway. |

### 3.7 M3.X — Comptabilité tokens unifiée

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| `run_id` de corrélation ContextVar partagé traces↔budgets↔metrics | commit `18c37f9` | Grep sur `run_id` ContextVar. |
| `unified_tokens_enabled` / `record_run_tokens` / `get_run_tokens` registre borné run-keyed | `ROADMAP-M3:202` | Localiser les fonctions et vérifier le kill-switch `ODYSSEUS_UNIFIED_TOKENS` OFF. |
| 12 tests | commit `94f7cb2` | Trouver le fichier de tests. |

### 3.8 Obsidian READ/search MCP

| Prétention | Preuve documentaire | À vérifier en Phase 1 |
|---|---|---|
| Scaffold stdio ajouté à `mcp_servers/obsidian_mcp.py` (Server/list_tools/call_tool) | `STATE.md:135` (commit `ef23b78`) | Lire `mcp_servers/obsidian_mcp.py` — confirmer les décorateurs MCP. |

| Expose `list_notes`, `get_note`, `search_notes` (pas `create_note` — WRITE = checkpoint natif) | `STATE.md:135` | Compter les tools exposés. |
| Enregistré via `_optional_python_servers()` derrière `ODYSSEUS_OBSIDIAN_MCP` OFF | `STATE.md:135`, `.env.example:265` | Lire le lieu d'enregistrement. |
| **Legacy `OBSIDIAN_MCP_ENABLED` supprimé** (commit `bca398a`) | `.env.example:302-303` | Grep pour toute référence résiduelle. |
| 7 tests | commit `ef23b78` | Localiser le fichier de tests. |

---

## 4. Kill-switches déclarés (source: `.env.example`)

Inventaire brut extrait de `.env.example:199-266`. **Chaque switch doit être vérifié en Phase 1** : (a) le code qui le lit existe, (b) OFF ⇒ code path original inchangé.

| Variable | Défaut | Ligne `.env.example` | Portée revendiquée |
|---|---|---|---|
| `ODYSSEUS_DESTRUCTIVE_GATE` | `on` | 203 | Bloque `rm -rf /` etc. au point central `tool_execution.py:553` (M2-P4). |
| `ODYSSEUS_PHASE_TRACKER` | `off` | 207 | Pose une phase par round dans le loop live (M3.0). |
| `ODYSSEUS_GOVERNANCE_ANCESTRY` | `off` | 213 | Écrit une `GoalTask` avec ancestry au MEMORY_OBSERVE (M3.4). |
| `ODYSSEUS_CHECKPOINT` | `off` | 219 | Écrit checkpoint Obsidian post-round (M3.2). |
| `ODYSSEUS_MODEL_ROUTER` | `off` | 224 | Advisory log-only du routing (M3.1). |
| `ODYSSEUS_RRF_FUSION` | `off` | 229 | RRF re-rank au lieu du blend 0.7/0.3 (M3.2). |
| `ODYSSEUS_ZEN_FROM_ENDPOINT` | `0` | 235 | Active Zen depuis `ModelEndpoint` natif si aucune clé env (M3). |
| `ODYSSEUS_AUTOEVAL` | `off` | 241 | Autorise `git reset --hard` sur verdict verifier (M3.3). |
| `ODYSSEUS_UNIFIED_TOKENS` | `off` | 246 | Réconciliation tokens par run_id (M3.X). |
| `ODYSSEUS_INPROCESS_DISCORD` | `off` | 251 | Réveille adapter Discord in-process (M3.5). |
| `ODYSSEUS_INPROCESS_TELEGRAM` | `off` | 252 | Réveille adapter Telegram in-process (M3.5). |
| `ODYSSEUS_CHANNEL_AGENT_REPLY` | `off` | 258 | Reply auto sur inbound channel (M3.5). |
| `ODYSSEUS_OBSIDIAN_MCP` | `off` | 265 | Enregistre le stdio Obsidian MCP (list/get/search). |
| `ACONTEXT_ENABLED` | `false` | 313 | Active le provider observe-only (M3.2). |
| `SCRAPLING_ENABLED` | `false` | 286 | Active la connexion MCP Scrapling. |
| `SUPABASE_MCP_ENABLED` | `false` | 290 | Active MCP Supabase. |
| `KROKI_ENABLED` | `true` | 293 | Kroki diagrammes (par défaut ON — anomalie vs invariant « OFF par défaut »). |

**⚠ À vérifier — anomalie kill-switch KROKI :** `KROKI_ENABLED=true` par défaut viole l'invariant « toute nouvelle capacité gatée OFF ». Motivation à confirmer (`STATE.md` traite Kroki comme livraison E2E validée `32ef455`).

**⚠ Legacy retiré (commit `bca398a`) :** `OBSIDIAN_MCP_ENABLED` (jamais wired). Ne doit plus apparaître dans le code — Phase 1 grep.

---

## 5. Écarts doc ↔ doc déjà spottés (Phase 0)

Sans même lire le code : plusieurs prétentions internes se contredisent. À trancher en Phase 1 (le code fait foi).

| # | Écart | Sources contradictoires | Résolution attendue Phase 1 |
|---|---|---|---|
| 1 | 101 commits ahead of origin/dev vs 0 ahead | `STATE.md:140` vs `git rev-list --count` = 0 | Confirmer via `git status -sb` + comparaison des SHAs distants. Corriger STATE.md si stale. |
| 2 | Le snapshot « ~35% câblé » de `STATE.md:51` est marqué **périmé** ligne 24 par la MAJ 2026-07-05 | interne `STATE.md` | Aucune action code — noter que STATE est en couches horodatées. |
| 3 | `INTEGRATION-TRACKING.md:47` prétend `command_validator` bloque « sur 1 chemin ». `STATE.md:91` prétend les 3 chemins agent couverts + `_validate_shell_or_block` helper. | `INTEGRATION-TRACKING.md:47` (2026-07-01) vs `STATE.md:91` (2026-07-02) | Grep pour `_validate_shell_or_block` dans `src/builtin_actions.py`. |
| 4 | `INTEGRATION-TRACKING.md:82` traite `stage-model-assignment.yaml` de « FANTÔME modèles » ; `ROADMAP-M3:197` prétend qu'il a été **supprimé** | interne | Confirmer l'absence du fichier + du re-export `src/__init__.py`. |
| 5 | Graphify : `AGENT-OS-ANALYSE-PROFONDE` positionne Graphify comme 2ᵉ leg de la Trinité ; `STATE.md:47` prétend Graphify = **rejeté** (verdict REDONDANT avec VectorRAG). `ANALYSE-OUTILS-UNIFIE.md:70` positionne Graphify comme **intégré actif** avec score 9/10. | vision (ConfigOpenCodeNew) vs interne | Décision architecturale à re-valider avec l'utilisateur au master plan. **Sans Graphify → point 4 de l'objectif final « Trinité » n'est **pas** rempli par CBM+Obsidian seuls.** |
| 6 | `.opencode/agents/*.md` : `INTEGRATION-TRACKING.md:49-52` = **inactifs dans l'UI web Odysseus**. `STATE.md:132` liste les agents `.opencode` comme dernier net-new à brainstormer. `ANALYSE-OUTILS-UNIFIE.md:278` prétend « 11 Subagents auto-délégués ». | interne vs vision | Phase 1 grep pour tout référence Python à `.opencode/agents`. Grand écart à confirmer. |
| 7 | Decision Engine : `.env.example:316` pointe `DECISION_ENGINE_PATH=/c/Users/ttmdu/Documents/GitHub/ConfigOpenCodeNew/decision-engine` (repo cross). `PROJECT.md:45` liste « Decision Engine intégré (depuis ConfigOpenCodeNew/decision-engine/) » comme non-actif. `ANALYSE-OUTILS-UNIFIE.md:701-745` documente Decision Engine comme livré en Phase 10 de ConfigOpenCodeNew. | interne + vision | Vérifier l'existence + le montage docker-compose de Decision Engine dans **ce** repo. |

---

## 6. Zones à rescanner (Phase 1 — dispatch parallèle)

Le rescan Phase 1 s'articulera sur **10 zones** (adaptées à l'arbo réelle constatée). Chaque zone reçoit un subagent Explore + un format de retour strict (voir mission-prompt § Phase 1).

| Zone | Périmètre initial | Kill-switches en jeu |
|---|---|---|
| **A** — Backend FastAPI / Decision engine | `app.py`, `src/orchestrator/`, `routes/`, `src/agent_loop.py`, `src/llm_core.py`, `src/zen_router.py`, `src/orchestrator/router_advice.py`, `services/` | `ODYSSEUS_MODEL_ROUTER`, `ODYSSEUS_ZEN_FROM_ENDPOINT`, `ODYSSEUS_DESTRUCTIVE_GATE` |
| **B** — Orchestration & agents | `src/orchestrator/*`, `.opencode/agents/*.md`, `src/orchestrator/loop.py` (CanonicalLoop), `phase_tracker.py`, `autoeval.py`, `ancestry_tracker.py`, `checkpoint_tracker.py`, `verifier`, `dispatcher.py`, `spec.py`, `registry.py` | `ODYSSEUS_PHASE_TRACKER`, `ODYSSEUS_AUTOEVAL`, `ODYSSEUS_GOVERNANCE_ANCESTRY`, `ODYSSEUS_CHECKPOINT` |
| **C** — Mémoire, RAG, Connaissance | `src/rag_vector.py`, `src/memory_provider.py`, `services/acontext/`, `mcp_servers/obsidian_mcp.py`, `src/context_compactor.py`, `routes/knowledge_routes.py` | `ODYSSEUS_RRF_FUSION`, `ODYSSEUS_CHECKPOINT`, `ODYSSEUS_OBSIDIAN_MCP`, `ACONTEXT_ENABLED` |
| **D** — Canaux (Discord/Telegram/email/webhooks) | `src/channel_gateway.py`, `src/channel_bootstrap.py`, `src/adapters/discord_adapter.py`, `src/adapters/telegram_adapter.py`, email pollers, `src/webhook_manager.py`, `routes/channel_routes.py` | `ODYSSEUS_INPROCESS_DISCORD`, `ODYSSEUS_INPROCESS_TELEGRAM`, `ODYSSEUS_CHANNEL_AGENT_REPLY` |
| **E** — Observabilité | `src/observer.py`, `src/trace_writer.py`, `src/budget_enforcer.py`, `src/unified_tokens.py` (si présent), `src/orchestrator/gate.py`, `agent_runs`/`task_runs` schema | `ODYSSEUS_UNIFIED_TOKENS`, `ODYSSEUS_DESTRUCTIVE_GATE` |
| **F** — Outils & MCP | `src/tool_registry.py`, `src/tool_execution.py`, `src/agent_tools/*`, `mcp_servers/*`, `src/mcp_manager.py`, `docker-compose.yml` services `scrapling`/`kroki`/`kroki-mermaid`/`supabase`, `render_diagram` | `SCRAPLING_ENABLED`, `KROKI_ENABLED`, `SUPABASE_MCP_ENABLED`, `ODYSSEUS_OBSIDIAN_MCP` |
| **G** — Frontend Next.js dashboard | `static/` (dashboard actuel), `dashboard/` (si absent, à noter), templates HTML, integrations UI | (aucun kill-switch direct) |
| **H** — Infra & Docker | `docker-compose.yml`, `docker-compose.gpu-*.yml`, `docker/*`, `Dockerfile`, `docker/gpu.nvidia.yml`, `docker/gpu.amd.yml`, `.env.example`, LiteLLM proxy (si présent), Supabase | tous kill-switches config env |
| **I** — Tests | `tests/`, y compris les 123 « échecs environnementaux » à valider comme réellement environnementaux | — |
| **J** — Docs | `docs/`, `README.md`, `ROADMAP.md`, `SECURITY.md`, `THREAT_MODEL.md`, `.planning/`, `docs/superpowers/plans/*`, `docs/superpowers/specs/*` | — |

Chaque subagent produira une section fichier-par-fichier au format imposé :

```
## Zone X — <nom>
### Inventaire fichier par fichier
| Fichier | Rôle réel (lu, pas supposé) | État (vivant/mort/zombie/douteux) | Kill-switch associé | Dépendances entrantes/sortantes |
### Fonctionnalités extraites
### Écarts doc vs code
### Code mort / doublons / fantômes détectés
### Risques & dettes
```

---

## 7. Ce que ce document N'endosse PAS

Pour éviter tout endossement implicite :

- **Aucun** des 12 kill-switches M3 n'est vérifié comme réellement lu par le code (Phase 1).
- **Aucun** des 4196 « tests verts » n'est réputé prouver l'intégration (les tests unitaires isolent — le fix `035d2f6` en est la preuve).
- **Aucun** call-site prétendu (`agent_loop.py:2513`, `:3569`, `:3575`, `:3599`, `:3619`, `:3647`, etc.) n'est confirmé — ils seront lus un par un.
- **Aucune** claim de suppression (`llm_router.py`, `stage-model-assignment.yaml`, `routing_routes.py`, enums `ChannelType.EMAIL`/`WEBHOOK`) n'est vérifiée à ce stade.
- **La position sur Graphify** (rejeté ? intégré ? redondant ?) est en conflit interne/vision et sera arbitrée en Phase 3 (matrice de couverture) puis Phase 4 (master plan).

---

## 8. Prochaine étape

Lancer la Phase 1 : 10 subagents Explore parallèles, un par zone (§6). Chacun produit une carte fichier-par-fichier au format imposé. La fusion produira `docs/audit/01-MAPPING-COMPLET.md` avec un graphe de dépendances de haut niveau (via `trace_path` / `query_graph` CBM).

Rappels garde-fous permanents :

1. Native-first — toute proposition d'ajout doit prouver que le natif ne le fait pas déjà.
2. Kill-switch OFF par défaut — byte-identique tant que le flag est OFF.
3. Aucune action destructive hors du worktree `audit/table-rase`.
4. Découpage en vagues si le contexte sature (Phase 1 sera vraisemblablement 2-3 vagues).
5. Bug bloquant type `import app` cassé : `systematic-debugging` + fix sur branche séparée + test.

— Fin Phase 0.
