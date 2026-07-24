# Plan de Migration — Odysseus → OpenCode Core

**Date :** 2026-07-24 | **Statut :** Plan approuvé

## Architecture Cible

```
┌─────────────────────────────────────────────────────────────┐
│              ODYSSEUS (UI Pure — Python/FastAPI)             │
│                                                              │
│  Chat UI  │ Cockpit  │ Settings  │ Dashboard  │ Email/Cal   │
│  ─────────┼──────────┼───────────┼────────────┼──────────── │
│  FastAPI  │ JS/CSS   │ Config    │ Killswitch │ Routes      │
│  routes   │ cockpit  │ YAML      │ Registry   │ existantes  │
└───────────┴──────────┴───────────┴────────────┴─────────────┘
        │                                              │
        │ WebSocket/SSE                                │ API REST
        ▼                                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    OP ENCODE CLI                             │
│                                                              │
│  ZenRouter  │ Agents 16  │ MCP 8  │ Skills  │ Worktrees     │
│  ───────────┼────────────┼────────┼─────────┼────────────── │
│  deepseek   │ @sfd-orch  │ Kroki  │ sfd-    │ git worktree  │
│  minimax    │ @executor  │ Chroma │ pipeline│ native         │
│  kimi       │ @planner   │ Scrapl │ decision│                │
│                        │ @reviewer  │ Email  │ tree    │                │
│                                                              │
│  NOS PLUGINS (7):                                            │
│  ─────────────                                               │
│  opencode-sfd-memory    → [stated]/[observed] + MD files     │
│  opencode-sfd-durable   → retry/saga/signals                 │
│  opencode-sfd-prefs     → 5-level resolution + guardrails    │
│  opencode-sfd-visual    → Kroki diagrams + SVG inline        │
│  opencode-sfd-classify  → 5 retention levels                 │
│  opencode-sfd-security  → injection guard + output filter    │
│  opencode-sfd-discovery → tool_search + registry + suggest   │
│                                                              │
│  CUSTOM TOOLS (2):                                           │
│  ───────────────                                             │
│  sfd-phase    → 7-phase state machine                        │
│  sfd-memory   → CRUD with provenance                         │
└─────────────────────────────────────────────────────────────┘
        │                                              │
        ▼                                              ▼
┌─────────────────────────────────────────────────────────────┐
│              SERVICES DOCKER (inchangés)                     │
│                                                              │
│  ChromaDB  │ Kroki  │ Meilisearch  │ ntfy  │ SearXNG        │
│  Scrapling │ Serena │ LangFuse     │ OPA   │ PostgreSQL     │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1 — Fondations OpenCode (1-2 jours)

### 1.1 Setup OpenCode comme moteur principal
- [ ] Remplacer `OPENCODE_API_KEY` par le CLI local (supprimer l'appel HTTP direct)
- [ ] Configurer `opencode.json` avec tous les modèles
- [ ] Migrer les 12 agents `.opencode/agents/` existants vers le format OpenCode natif
- [ ] Ajouter nos 4 agents SFD (@sfd-orchestrator, @planner, @executor, @reviewer)

### 1.2 Bridge Odysseus ↔ OpenCode
- [ ] Remplacer `stream_agent_loop()` par un appel au CLI OpenCode
- [ ] Mode: Odysseus envoie le message → OpenCode CLI traite → réponse streamée → Odysseus affiche
- [ ] Garder le cockpit SSE (phase_enter/exit events depuis nos plugins)
- [ ] Garder les routes FastAPI existantes

**Fichiers à créer :**
- `opencode.json` — config globale
- `src/opencode_bridge.py` — bridge FastAPI ↔ OpenCode CLI
- `.opencode/agents/*.md` — déjà fait

**Fichiers à supprimer :**
- `src/llm_core.py` → OpenCode ZenRouter natif
- `src/zen_router.py` → OpenCode ZenRouter natif
- `src/endpoint_resolver.py` → OpenCode natif

---

## Phase 2 — Plugins SFD (3-5 jours)

### 2.1 Plugin: Mémoire avec Provenance
- [ ] `opencode-sfd-memory` — npm package ou plugin local
- [ ] Hook `session.created` → initialise le MD file store
- [ ] Hook `tool.execute.after` → après chaque `write`, taguer [stated]/[observed]
- [ ] Custom tool `sfd-memory` → CRUD operations
- [ ] OmissionFilter intégré → bloque SSN, santé, religion...
- [ ] Dual-write: ChromaDB (via API HTTP) + MD files

### 2.2 Plugin: Exécution Durable
- [ ] `opencode-sfd-durable` — npm package ou plugin local
- [ ] Hook `tool.execute.before` → wrap dans SQLite workflow si action longue
- [ ] Retry policy: exponential backoff, max 3 attempts
- [ ] Saga: compensation en ordre inverse
- [ ] Signal d'approbation humaine → via cockpit Odysseus

### 2.3 Plugin: Préférences
- [ ] `opencode-sfd-prefs` — plugin local
- [ ] Hook `session.created` → inject preferences dans le system prompt
- [ ] Résolution 5 niveaux: request > always > userStyle > selective > default
- [ ] Guardrails: bloque "always agree", "never disagree"...

### 2.4 Plugin: Sortie Visuelle
- [ ] `opencode-sfd-visual` — plugin local
- [ ] Custom tool `sfd-render` → appelle Kroki, retourne SVG
- [ ] Arbre de décision: MCP tool → file → inline visualizer → text
- [ ] Supports Mermaid, PlantUML, GraphViz via Kroki

### 2.5 Plugin: Classification
- [ ] `opencode-sfd-classify` — plugin local
- [ ] Hook `tool.execute.before` → vérifie la classification des données écrites
- [ ] 5 niveaux: PUBLIC, INTERNAL, PERSONAL, SENSITIVE, PROTECTED
- [ ] Droit à l'oubli: custom tool `sfd-forget`

### 2.6 Plugin: Sécurité Contenus
- [ ] `opencode-sfd-security` — plugin local
- [ ] Hook `tool.execute.before` (read) → vérifie pas d'injection dans les fichiers mémoire
- [ ] Hook `session.updated` → filtre la sortie pour contenu prohibé
- [ ] Bloque: violence, copyright, personnes réelles, désinformation

### 2.7 Plugin: Découverte Outils
- [ ] `opencode-sfd-discovery` — plugin local
- [ ] Custom tool `sfd-discover` → cherche dans le registre MCP
- [ ] Suggestion de connecteurs (first-party vs third-party)
- [ ] Distinction: third-party → toujours demander confirmation

---

## Phase 3 — Odysseus UI Adaptation (2-3 jours)

### 3.1 Cockpit
- [ ] Le cockpit reçoit les événements de phase depuis nos plugins (via WebSocket)
- [ ] La phase bar C-K-P-B-Q-A-M reste inchangée
- [ ] Les chips (drift, iters, budget, health) restent inchangés

### 3.2 Chat UI
- [ ] Le flux SSE passe par `opencode_bridge.py` au lieu de `stream_agent_loop`
- [ ] Les tool blocks (BASH, WRITE_FILE...) sont rendus comme avant
- [ ] Les agent indicators (⏳, ✓, ✗) sont rendus comme avant

### 3.3 Settings
- [ ] Kill-switch dashboard: nos switches SFD (8) migrent dans `opencode.json` permissions
- [ ] Model config: géré par OpenCode natif
- [ ] Skills management: déjà natif OpenCode

---

## Phase 4 — Tests & Validation (2-3 jours)

### 4.1 Tests Plugins
- [ ] Test unitaire de chaque plugin (Jest/Bun test)
- [ ] Test d'intégration: orchestrateur → planner → executor → reviewer
- [ ] Test de parallélisme: 3 executors simultanés

### 4.2 Tests End-to-End
- [ ] UC-01: Lancer un projet complet (build a backup system)
- [ ] UC-02: Interrompre et reprendre
- [ ] UC-10: Multi-agent avec modèles différents
- [ ] UC-16: Sortie visuelle (diagramme)

### 4.3 Performance
- [ ] Mesurer latence: OpenCode CLI vs ancien stream_agent_loop
- [ ] Mesurer tokens: multi-agent via OpenCode vs notre ancien système

---

## Phase 5 — Nettoyage (1 jour)

### Fichiers à SUPPRIMER (remplacés par OpenCode natif)
```
src/thought_bus/          → @sfd-orchestrator + sfd-phase tool
src/agent_pipeline.py     → sfd-decision-tree skill
src/full_system.py        → @sfd-orchestrator agent
src/mode_detector.py      → OpenCode Build/Plan Tab switch
src/plugin_system.py      → OpenCode plugins natifs
src/worktree_support.py   → OpenCode worktrees natifs
src/llm_core.py           → OpenCode ZenRouter natif
src/zen_router.py         → OpenCode ZenRouter natif
src/endpoint_resolver.py  → OpenCode natif
```

### Fichiers à GARDER (Odysseus UI + services)
```
app.py                    → FastAPI app
routes/chat_routes.py     → chat SSE (modifié pour bridge OpenCode)
routes/*.py               → toutes les routes UI
static/                   → tout le frontend
src/sfd_wiring.py         → adapté pour initialiser les plugins
src/service_connector.py  → inchangé
src/killswitch_registry.py→ adapté pour opencode.json permissions
src/orchestrator/         → agents .opencode/ (conservés)
skills/                   → migré vers .opencode/skills/
docker-compose.yml        → inchangé
scripts/test-e2e.sh       → adapté pour OpenCode CLI
.github/workflows/        → adapté
```

### Code à ADAPTER
```
src/sfd_wiring.py         → initialise les plugins OpenCode au lieu des modules Python
src/agent_instructions.py → devient un skill .opencode/skills/
src/agent_pipeline.py     → devient sfd-decision-tree skill
routes/chat_routes.py     → bridge vers OpenCode CLI au lieu de stream_agent_loop
```

---

## Récapitulatif

| Métrique | Avant | Après |
|----------|-------|-------|
| Lignes de code Python | ~3,500 (SFD) | ~500 (bridge + UI) |
| Lignes de code TS/JS | 0 | ~1,500 (plugins + tools) |
| Agents | 12 .opencode/ (manuel) | 16 agents (natif OpenCode) |
| Phases | ThoughtBus custom | sfd-phase tool + plugin hooks |
| Modèles par agent | Non (tous même modèle) | Oui (executor=m3, reviewer=deepseek) |
| Parallélisme | Manuel (asyncio.gather) | Natif (Task tool) |
| Plugins | Notre système custom | Natif OpenCode JS/TS |
| Worktrees | Notre implémentation | Natif OpenCode |
| Compaction | Manuel | Natif OpenCode |
| Maintenance | 11 modules à maintenir | 7 plugins + 1 bridge |
| Dépendances | 433 packages Python | 433 packages (inchangé) + OpenCode CLI |

**Gain net :** ~3,000 lignes de Python supprimées, fonctionnalités natives OpenCode utilisées au lieu de réimplémentations, architecture plus simple et plus robuste.
