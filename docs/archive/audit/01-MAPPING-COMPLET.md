# AUDIT — 01 MAPPING COMPLET (Post-Phase 1)

> **Statut :** Phase 1 terminée. 10 subagents parallèles, ~25K lignes de rapports bruts.
> **HEAD :** `bca398a` (branche `dev`), synchronisé avec `origin/dev`.
> **Méthode :** Lecture directe du code via CBM (11749 nodes, 38126 edges). Zéro supposition.

---

## 1. Synthèse exécutive

### 1.1 État global

| Métrique | Valeur | Source |
|---|---|---|
| Fichiers Python source (~) | 250+ | services/ + src/ + routes/ + core/ + mcp_servers/ |
| Fichiers JS frontend | 151 modules, 5.53 MB | `static/js/` |
| Fichiers CSS | 1 unique, 1.22 MB (38K lignes) | `static/style.css` |
| Fichiers de test | 691 (~3622 fonctions, 4351 nodeids) | `tests/` |
| Tests verts | 4196 passed | STATE.md |
| Tests en échec | 124 (70% environnementaux, 30% suspects) | `.pytest_cache/lastfailed` |
| Kill-switches documentés | 19 | `.env.example:199-316` |
| Kill-switches vérifiés OFF | 16/19 (84%) | Zones A/B/C/D/E |
| Kill-switches ON par défaut | 2 — `ODYSSEUS_DESTRUCTIVE_GATE`, `KROKI_ENABLED` | gate.py + docker-compose |
| Services Docker | 11 (dont 4 avec profils) | `docker-compose.yml` |
| Agents `.opencode/` | 12 définis, 0 chargés en live | Zone B |
| Routes FastAPI | 53 `include_router` + 1 `mount` | `app.py` |
| Outils agent | 30 dans TOOL_HANDLERS + 34 dans dispatcher = 64 total | Zone F |
| Serveurs MCP | 7 (5 Python builtin, 1 NPX, 3 externe dont 1 fantôme) | Zone F |

### 1.2 Verdict M3

**M3 est fonctionnellement complet** — tous les sous-milestones M3.0→M3.5 + M3.X sont câblés dans `agent_loop.py` et le package `src/orchestrator/`, **chacun derrière un kill-switch défaut-OFF**. Comportement byte-identical quand tous les kill-switches sont OFF.

**17 call-sites M3 vérifiés** fichier:ligne dans `agent_loop.py:1934-3679` :
- `2534` PhaseTracker, `2543` router_advice, `3562` acontext, `3579` governance, `3597` checkpoint, `3612` observer, `3625` autoeval, `1977` run_id, `3550` channel gateway, `3660` teacher

**3 corrections majeures confirmées** :
1. ✅ RRF (`hybrid_search`) réparé — fonction pure, plus de réflexion buggée
2. ✅ `httpx.post` synchrone hardcodé supprimé — remplacé par `MemoryProviderRegistry`
3. ✅ `GRAPHIFY_URL` éradiqué — délégué au `VectorRAG` natif

---

## 2. Inventaire complet — par zone

### 2.1 Zone A — Backend FastAPI / Decision Engine

**53 routes enregistrées** dans `app.py` (584-824). **Zéro mount mort** — le bug `035d2f6` (routing_routes) est corrigé, `test_app.py::TestAppBoots` existe.

**Fichiers supprimés confirmés** :
- `src/llm_router.py` ❌ absent
- `.planning/stage-model-assignment.yaml` ❌ absent
- `routes/routing_routes.py` ❌ absent
- Re-export `ModelRouter` dans `src/__init__.py` ❌ retiré

**Fichiers conservés délibérément** :
- `model-routing.json` — politique de routing UNIQUE (models/stages/heuristic/fallback_chains), `providers.*` partiellement redondant avec DB

**IDs modèles dans model-routing.json** : deepseek-v4-pro, kimi-k2.6, deepseek-v4-flash, kimi-k2.7-code, qwen3.7-plus, glm-5.2. Le fichier les déclare "testés et confirmés", STATE.md les déclare "fantômes". **Contradiction à résoudre.**

**Services/** : 9 vivants (memory, search, research, shell, tts, stt, youtube, hwfit, docs), 2 dormants (faces stub vide, acontext service externe).

**Doublon code détecté** : `_AGENT_PREAMBLE` et `_AGENT_RULES` définis 2 fois dans `agent_loop.py` (lignes 62-111 écrasées par 175-200).

### 2.2 Zone B — Orchestration & Agents

**7 phases canoniques** (CLASSIFY→KNOW→PLAN→BUILD→QUALITY→AUTOEVAL→MEMORY_OBSERVE) dans `phases.py` et `loop.py`. `CanonicalLoop` **non branché au live** — `PhaseTracker` injecte une phase simplifiée (toujours BUILD sauf plan_mode).

**3 phases YAML orphelines** (RESEARCH, INNOVATE, VERIFY) dans `phase-lock.yaml` — définies mais absentes de l'Enum Python.

**12 agents `.opencode/agents/*.md`** : tous définis, parsables, testés via `AgentRegistry.discover()`, mais **aucun chargé dynamiquement par le code live**. Le mécanisme est prêt mais non branché.

**Gate destructif ON par défaut** (seul kill-switch actif) — bloque bash/python avec patterns catastrophiques dans `tool_execution.py:563`.

**3 call-sites `_validate_shell_or_block`** confirmés dans `builtin_actions.py:340,359,374` (ssh, run_script, run_local).

**Documents fantômes** : `loop-canonique.md` et `permission-matrix.md` inexistants.

### 2.3 Zone C — Mémoire, RAG, Connaissance

**3 corrections M3.2 confirmées** :
1. ✅ RRF réparé + câblé (`rag_vector.py:781`, gate `ODYSSEUS_RRF_FUSION` OFF)
2. ✅ `httpx.post` synchrone supprimé → `MemoryProviderRegistry.dispatch_session_end()` dans `agent_loop.py:3569`
3. ✅ GRAPHIFY_URL éradiqué → `_native_semantic_search` utilise `get_rag_manager()`

**Routes Trinité dormantes** : `/api/knowledge/*` sans appelant frontend/loop/test.

**⚠ Problème critique** : `/api/knowledge/memory/*` proxy vers `localhost:9751` qui n'existe pas — `obsidian_mcp.py` est un serveur stdio MCP, pas HTTP. Endpoint cassé structurellement.

**⚠ Problème** : `mcp_servers/obsidian_mcp.py` n'implémente PAS le protocole MCP stdio (utilise Flask HTTP), mais `builtin_mcp.py:109` l'enregistre comme serveur stdio. Incompatibilité silencieuse.

**services/memory/** : 9 modules, 7 vivants, 2 wrappers fantômes (5 lignes chacun).

### 2.4 Zone D — Canaux

**ChannelType** : seulement DISCORD + TELEGRAM. EMAIL/WEBHOOK supprimés et vérifiés par tests.

**3 kill-switches OFF** : `ODYSSEUS_INPROCESS_DISCORD`, `ODYSSEUS_INPROCESS_TELEGRAM`, `ODYSSEUS_CHANNEL_AGENT_REPLY`. Tous défaut-OFF → comportement byte-identical.

**Round-trip inbound→agent→reply** : `run_agent_reply()` (`channel_bootstrap.py:68-126`) utilise `task_llm_call_async` natif. Testé (26 tests). Best-effort, jamais lève.

**Adapters Discord/Telegram** : existent, lazy-load (import dans la fonction), mais dormant sans kill-switch.

**Email** : sous-système complètement indépendant (routes + pollers + MCP server). Mature et actif.

**⚠ Dette** : `src/adapters/__init__.py` = 1 ligne de commentaire (placeholder vide).

### 2.5 Zone E — Observabilité

**Pipeline d'observabilité complet** : run_id ContextVar → trace_writer → budget_enforcer → observer → autoeval. 5 étapes chaînées dans `agent_loop.py:1972-3623`.

**⚠ Problème** : `Observer` instancié neuf à chaque run — pas de mémoire inter-run. `_codeburn_reports` ne contient jamais plus d'un rapport → drift score systématiquement LOW (sauf harness touché dans le run courant).

**⚠ Problème** : `trace_writer.py` accepte `cost_tokens` mais le call-site dans `tool_execution.py` ne le passe jamais → toutes les traces ont `cost_tokens: 0`.

**⚠ Code mort** : 6 entrées fantômes dans `TOOL_RISK_MAP` (`create_file`, `list_dir`, `search_files`, `get_symbol`, `pipeline`, `edit_image`) — outils qui n'existent pas ou ont été renommés.

**⚠ Méthodes non utilisées** : `BudgetEnforcer.consume_tool_call()`, `auto_pause_if_exceeded()`, `Observer.get_summary()`.

**Unified tokens** : intégré dans `trace_writer.py:125-200` (pas de fichier séparé). Registre borné à 4096 entrées, last-write-wins.

### 2.6 Zone F — Outils & MCP

**64 outils agents** : 30 dans TOOL_HANDLERS + 34 dispatchés directement dans `tool_execution.py`. 80+ branches elif — dispatcher monolithique fragile.

**7 serveurs MCP** :
- 4 Python builtin (image_gen, memory, rag, email) — vivants
- 1 NPX builtin (Playwright) — vivant
- 1 optionnel (obsidian) — **cassé** (pas de protocole MCP stdio)
- 2 externes : scrapling (vivant, docker), supabase (**fantôme** — pas de service docker)

**⚠ Critique** : Supabase MCP défini dans `mcp_manager.py:40-46` + `supabase_mcp.py` (64 lignes) mais **aucun service docker-compose**. Service inopérant.

**⚠ Critique** : `obsidian_mcp.py` utilise Flask HTTP, pas MCP stdio. `builtin_mcp.py` le lance comme stdio → échec silencieux.

**Tools fantômes dans phase-lock.yaml** : `run_command`, `create_file`, `delete_file` — jamais dispatchés.

**Double routage MCP/TOOL_HANDLERS** : `bash`, `python`, `read_file`, `write_file`, `web_search`, `web_fetch`, `generate_image` ont deux implémentations (MCP prioritaire, TOOL_HANDLERS fallback).

### 2.7 Zone G — Frontend Dashboard

**SPA vanilla JS** : 182 fichiers, 11.48 MB, sans framework ni bundler.

**⚠ Dette critique** : `style.css` = 1.22 MB (38,346 lignes), cible STATE.md <200KB (BUG-08).

**⚠ Dette critique** : Pas de build/bundler — 151 modules ES6 servis bruts, pas de tree-shaking, minification, code splitting.

**⚠ Dette** : Librairies lourdes sans lazy-loading (xlsx.full.min.js ~1.4MB).

**⚠ Dette** : `app.js` = 4085 lignes (orchestrateur monolithique), `slashCommands.js` = 5500+ lignes.

**Points positifs** : PWA fonctionnelle, Service Worker bien conçu, GZip compression, CSP nonce, API complète bien intégrée.

**Pas de dashboard séparé** — tout est dans le SPA unique. Pas de Next.js, React ou autre framework.

### 2.8 Zone H — Infrastructure & Docker

**11 services docker-compose** : odysseus, chromadb, searxng (épinglé), ntfy, kroki, kroki-mermaid, serena-mcp, acontext, scrapling-mcp, codebase-memory, decision-engine.

**⚠ Critique** : `cap_drop: [ALL]` commenté sur `odysseus` depuis la création (ligne 88). 9/11 services sans restriction de sécurité.

**⚠ Critique** : Standalone GPU files (`docker-compose.gpu-*.yml`) désynchronisés — 7 services manquants + sécurité absente vs le compose principal.

**⚠ Problème** : `decision-engine` build context pointe vers `../ConfigOpenCodeNew/decision-engine` (chemin externe, Windows hardcodé dans `.env.example:316`).

**⚠ Problème** : `chromadb:latest` non épinglé (seul `searxng` l'est).

**Services fantômes** : Supabase MCP (pas de container), LiteLLM proxy (jamais dockerisé).

**3/11 services avec healthcheck** seulement (searxng, kroki, scrapling).

### 2.9 Zone I — Tests

**691 fichiers, ~3622 fonctions, 4351 nodeids**. 4196 passed, 124 failed.

**124 échecs** : ~70% environnementaux (Node.js absent, symlinks Windows, plateforme-spécifique), ~30% suspects (code_nav_tools: ripgrep absent, JS tests).

**Zones critiques sans tests directs** :
- `core/auth.py`, `core/database.py`, `core/models.py` — 0 test
- `routes/chat_routes.py`, `routes/session_routes.py`, `routes/auth_routes.py` — 0 test route

**Points positifs** : Helpers sophistiqués (import_state, db_stubs, sqlite_db, embedding_lanes), taxonomie automatique, smoke test boot (`TestAppBoots`), 88 fichiers avec mocks.

**Split inachevé** : seul `tests/cli/` (28 fichiers) migré, 651 fichiers restent en flat `tests/`.

### 2.10 Zone J — Documentation

**36 fichiers inventoriés**. 7 à archiver, 8 écarts doc↔doc, 4 doublons, 6 documents fantômes.

**⚠ Critique** : `STATE.md:140` affirme "101 commits en avance sur origin/dev" — **faux**, `git rev-list --count` = 0.

**⚠ Critique** : Conflit Graphify non résolu — la vision le positionne comme leg Trinité, M3 le REJETTE comme redondant. Objectif final #4 exige "Trinité câblée cohéremment" → impossible sans Graphify (ou remplacement).

**⚠ Doublon sévère** : `docs/ANALYSE-OUTILS-UNIFIE.md` (836 lignes, ConfigOpenCodeNew) vs `.planning/AGENT-OS-ANALYSE-PROFONDE.md` (456+ lignes) — deux analyses des mêmes 60 outils, conclusions divergentes.

**⚠ Documents à archiver** : `INTEGRATION-TRACKING.md`, `REQUIREMENTS.md`, `PROJECT.md`, `pipeline-lists/MOVE.md`, `pipeline-lists/WAIT.md`, `docs/memory-pipeline.md`, `ANALYSE-OUTILS-UNIFIE.md`.

**Documents les plus fiables** : `ROADMAP-M3-ORCHESTRATION.md`, `STATE.md` (après correction), `.planning/intel/INDEX.md` + `domains/*.md`, `docs/audit/00-BASELINE.md`.

---

## 3. Graphe de dépendances de haut niveau

```
┌─────────────────────────────────────────────────────────────────┐
│                        app.py (FastAPI)                          │
│  53 include_router + 1 mount + _startup_event                    │
└──────────────────────────┬──────────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                  ▼
┌─────────────────┐ ┌──────────────┐ ┌──────────────────┐
│  src/agent_loop │ │  routes/*.py │ │  mcp_servers/*   │
│  stream loop    │ │  59 fichiers  │ │  5 serveurs       │
│  ~3679 lignes   │ │  API REST     │ │  stdio MCP        │
└────────┬────────┘ └──────┬───────┘ └────────┬─────────┘
         │                 │                   │
    ┌────┼────────────┬────┼───────┬───────────┼──────┐
    ▼    ▼            ▼    ▼       ▼           ▼      ▼
┌──────┐┌──────────┐┌────────┐┌──────────┐┌───────────┐┌─────────┐
│llm   ││orchestrator││observer││channel   ││memory     ││tools    │
│core  ││(11 modules)││trace   ││gateway   ││provider   ││registry │
│2517L ││M3 hub     ││budget  ││bootstrap ││rag_vector ││execution│
└──┬───┘└──────────┘└───┬────┘└─────┬────┘└─────┬─────┘└────┬────┘
   │                     │           │           │            │
   ▼                     ▼           ▼           ▼            ▼
┌──────────────────────────────────────────────────────────────────┐
│                        services/ (12 dossiers)                    │
│  memory/ search/ research/ shell/ tts/ stt/ youtube/ hwfit/      │
│  docs/ faces/ acontext/                                          │
└──────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│                     core/ (11 modules)                            │
│  database.py auth.py models.py middleware.py session_manager.py   │
└──────────────────────────────────────────────────────────────────┘
```

### Dépendances critiques inter-zones

| Source → Cible | Type | Intensité |
|---|---|---|
| `agent_loop.py` → `orchestrator/*` | CALLS (6 kill-switches) | 🔴 Forte |
| `agent_loop.py` → `llm_core.py` | CALLS (stream) | 🔴 Critique |
| `agent_loop.py` → `observer.py` | CALLS (drift) | 🟡 Conditionnel |
| `tool_execution.py` → `mcp_manager.py` | CALLS (dispatch MCP) | 🔴 Forte |
| `memory_provider.py` → `services/acontext/` | HTTP (async) | 🟡 Gated |
| `channel_bootstrap.py` → `adapters/` | CALLS (lazy import) | 🟡 Gated |
| `rag_vector.py` → `chromadb` | HTTP (singleton) | 🟡 Conditionnel |
| `builtin_mcp.py` → `mcp_servers/*.py` | SUBPROCESS (stdio) | 🔴 Forte |
| `app.py` → `routes/*.py` | INCLUDE_ROUTER (53x) | 🔴 Forte |
| `app.py` → `static/` | MOUNT (StaticFiles) | 🔴 Forte |

---

## 4. Tableau de bord — Risques consolidés (Top 15)

| # | Risque | Zone | Sévérité | Détail |
|---|---|---|---|---|
| R1 | `obsidian_mcp.py` pas un vrai serveur MCP stdio | C, F | **CRITIQUE** | Flask HTTP ≠ MCP stdio. `builtin_mcp.py` le lance comme stdio → échec silencieux |
| R2 | Supabase MCP sans service docker | F, H | **CRITIQUE** | Code client + 64 lignes, config serveur, mais aucun conteneur |
| R3 | `cap_drop: [ALL]` commenté sur odysseus | H | **CRITIQUE** | Conteneur principal sans restriction capacités |
| R4 | CSS 1.22MB monolithique | G | **CRITIQUE** | 38K lignes, cible <200KB, dette BUG-08 non résolue |
| R5 | `STATE.md:140` — 101 commits ahead (faux) | J | **CRITIQUE** | Information erronée, risque push non autorisé |
| R6 | Observer sans mémoire inter-run | E | **HAUTE** | Nouveau Observer par run → drift jamais cumulatif |
| R7 | Standalone GPU files désynchronisés | H | **HAUTE** | 7 services + sécurité manquants vs compose principal |
| R8 | Routes Obsidian HTTP fantômes | C | **HAUTE** | `/api/knowledge/memory/*` proxy vers port inexistant |
| R9 | Tests core/auth, core/database sans couverture | I | **HAUTE** | Modules critiques 0% testés directement |
| R10 | 12 agents `.opencode/` non chargés en live | B | **HAUTE** | Définis, parsables, testés — mais jamais branchés |
| R11 | Conflit Graphify (vision vs M3) | J | **HAUTE** | Trinité incomplète sans 3ème leg |
| R12 | IDs modèles fantômes dans model-routing.json | A | **MOYENNE** | Contradiction fichier vs STATE.md |
| R13 | `TOOL_RISK_MAP` avec outils fantômes | E | **MOYENNE** | 6 entrées vers outils inexistants/renommés |
| R14 | Dispatcher monolithique 80+ elif | F | **MOYENNE** | Fragile, pas de pattern plugin |
| R15 | `chromadb:latest` non épinglé | H | **MOYENNE** | Risque mise à jour cassante |

---

## 5. Classification — Code mort, zombie, fantôme

| Élément | Zone | Nature | Action |
|---|---|---|---|
| `services/faces/__init__.py` (1 ligne) | A | Stub vide | SUPPRIMER |
| `src/adapters/__init__.py` (1 ligne) | D | Placeholder vide | SUPPRIMER |
| `services/memory/memory.py` (5 lignes) | C | Wrapper fantôme | GARDER (compat) |
| `services/memory/memory_vector.py` (5 lignes) | C | Wrapper fantôme | GARDER (compat) |
| `TOOL_RISK_MAP` entrées fantômes (6) | E | Outils inexistants | NETTOYER |
| `phase-lock.yaml` phases orphelines (3) | B | Jamais activables | NETTOYER |
| `phase-lock.yaml` outils fantômes (3) | F | `run_command` etc. | NETTOYER |
| `loop-canonique.md` | B | Document fantôme | CRÉER |
| `permission-matrix.md` | B | Document fantôme | CRÉER |
| `PROJECT.yaml` | J | Jamais créé | CRÉER ou ABANDONNER |
| Supabase MCP (code + config) | F, H | Service absent | SUPPRIMER ou DÉPLOYER |
| `docs/memory-pipeline.md` | J | Aspirationnel, 38 lignes réelles | ARCHIVER |
| `_AGENT_PREAMBLE` dupliqué | A | 1ère déf écrasée | NETTOYER |
| `BudgetEnforcer.consume_tool_call()` | E | Méthode jamais appelée | NETTOYER |
| `Observer.get_summary()` | E | Méthode jamais appelée | NETTOYER |

---

## 6. Prochaine étape

Phase 2 : produire `docs/audit/02-PLAN-NETTOYAGE.md` classant chaque élément candidat en SUPPRIMER / ARCHIVER / GARDER / FUSIONNER / CRÉER, avec justification traçable fichier:ligne.

**Rappel garde-fous :**
- Aucune suppression sans GO explicite de l'utilisateur
- Tout travail destructif dans un worktree `audit/table-rase`
- Smoke test de boot après chaque commit destructif
- Docs obsolètes → `docs/archive/`, pas détruites

— Fin Phase 1 / 01-MAPPING-COMPLET.
