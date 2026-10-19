# CONVERGENCE — Analyse unifiee de tous les outils (harnais + TikTok)

> Genere le 2026-10-19. Synthese de 51 harnais disséqués (910 fonctionnalités, 104 capacités) + 148 vidéos TikTok (3 comptes) + travail existant odysseusAgentOS.
> Mentalite : "Everything is a plugin" — chaque composant derriere un port + un registre.

---

## 1. Inventaire complet des sources

### 1.1 Harnais disséqués (51)

| Domaine | # harnais | # fonctionnalités | # adoptées | # planifiées |
|---------|-----------|-------------------|------------|--------------|
| agents | 47 | 108 | 1 | 104 |
| automatisation | 31 | 41 | 2 | 39 |
| configuration | 38 | 59 | 14 | 40 |
| contexte | 45 | 118 | 28 | 90 |
| execution | 42 | 110 | 10 | 98 |
| extension | 39 | 93 | 16 | 71 |
| memoire | 38 | 54 | 9 | 45 |
| observabilite | 38 | 62 | 24 | 34 |
| outils | 28 | 85 | 23 | 60 |
| routage | 38 | 54 | 18 | 29 |
| securite | 40 | 83 | 32 | 47 |
| ui | 29 | 43 | 0 | 38 |

**Total : 51 harnais, 910 fonctionnalités (177 adoptées, 695 planifiées, 38 écartées)**

### 1.2 Outils TikTok identifies (3 comptes, 148 vidéos)

| Compte | # vidéos | Outils identifies | Thématique principale |
|--------|----------|-------------------|----------------------|
| @0xloucash | 78 | Claude, ChatGPT, GitHub repos, skills Claude, Arcads, Gamma, Google Stitch, ManyChat, Obsidian, Telegram, MCP servers, etc. | IA multi-outils, productivité, scraping |
| @axelayzeia | 35 | Claude, Jarvis, Claude Code (Ponytail, OmniRoute, AgentSkills, Graphify), Recordly, PlugKit, certifications IA | IA pour dirigeants, prospection, automation |
| @thomasbssh1 | 34 | Ditto, RuView, AgentCraft, Gradium AI, Skill Replica, Founder skill, OmniRoute, JEV, Claude Design, Claude Code | Transformation entreprise IA, hackathon, skills |

**Outils TikTok uniques (28) :** Claude, ChatGPT, Jarvis, Ditto, RuView, AgentCraft, Gradium, Skill Replica, Founder skill, OmniRoute, JEV, Recordly, PlugKit, Ponytail, AgentSkills, Graphify, Certifications IA, Typesafe Console, Apify, MCP servers, Arcads, Gamma, Google Stitch, ManyChat, Obsidian, Telegram, etc.

### 1.3 Travail existant (odysseusAgentOS main branch)

Le repo principal contient :
- `docs/master-ref/10-ARCHITECTURE-MODULAIRE.md` : analyse combinatoire de 68 outils + vision AgentOS
- `docs/master-ref/01-SFD-v3.1.md` : spécification fonctionnelle de référence
- `docs/omega/adr/` : décisions d'architecture
- `docs/omega/research/` : recherche TikTok @0xloucash
- `core/` : moteur Python (atomic_io, database, middleware, etc.)
- `src/` : 100+ fichiers Python (agent_loop, agent_tools, etc.)
- `services/` : services Python
- `routes/` : routes API

---

## 2. Convergence — Outils par catégorie (Everything is a Plugin)

### 2.1 Couche Provider (LLM Routing)

**Sources :** ZenRouter (24 modèles), codebuff (freebuff model ids), refact-ai (provider registre), bloop (openai-compatible + local), nanocoder (MCP client), TikTok (OmniRoute, Claude, ChatGPT)

**Capacités identifiées :**
- Multi-provider routing (openai/anthropic/local/azure)
- Free-first policy (gratuits d'abord, payants en exception)
- Fallback policy (fallback chain par modèle)
- Token budget tracking
- Model scoring (communauté + benchmark)
- Stream multiplexing (switchable stream)
- Prompt cache (prefix cache partagé)

**Port Omega :** `Provider` + `Selector` + `FallbackPolicy` + `ScoringPolicy` + `TaskClassifier`

### 2.2 Couche Contexte (ContextEngine)

**Sources :** CBM (graphe de code), Graphify (graphe sémantique), ChromaDB (vector), Bloop (tree-sitter index + query DSL pest), sweepai (context pruning ripgrep), refact-ai (vecdb + knowledge graph), nanocoder (semantic memory), TikTok (second cerveau, Obsidian)

**Capacités identifiées :**
- Index de code multi-langage (tree-sitter, 14+ langages)
- Recherche sémantique (vector + full-text)
- Context pruning (LLM-guidé + déterministe)
- Compaction double couche (mechanical + model)
- Query DSL (pest parser, planner, execute, ranking)
- Knowledge graph (concepts + frontmatter)
- Windowed file reads
- Project profile auto-génération
- Repo map (structure du dépôt)

**Port Omega :** `ContextEngine` + `CatalogSource` + `MemoryStore`

### 2.3 Couche Mémoire (MemoryStore)

**Sources :** refact-ai (memories frontmatter + dedupe), nanocoder (semantic memory manager), sweepai (snippet store), TikTok (Obsidian second cerveau, mémoire persistante)

**Capacités identifiées :**
- Mémoire persistante (faits + préférences)
- Frontmatter structuré (KnowledgeFrontmatter)
- Deduplication (normalisation tags, preference text)
- Mémoire semantique (index recherche + summarizer)
- Recall + consolidation
- Namespaces (isolation par projet)
- Capture/commit/patch

**Port Omega :** `MemoryStore` + `EventStore`

### 2.4 Couche Outils (ToolSurface + ToolExecutor)

**Sources :** GSD (tool surface), codebuff (40+ tool handlers), nanocoder (tool registry + profiles), Claude Engineer (BaseTool ABC + ToolCreator), Onlook (code tools), TikTok (tools for Claude, plugins, skills)

**Capacités identifiées :**
- Registre de tools avec profils (activation par contexte)
- Tool creator (l'agent crée ses propres outils)
- Handlers spécialisés (apply-patch, spawn-agent, str-replace, browser-logs, etc.)
- Tool profiles (sous-ensembles activés selon contexte)
- Convention de sortie (formatage structuré)
- Parse custom des appels bruts
- Effets réversibles (snapshot + revert)
- LSP integration (diagnostics + format)

**Port Omega :** `ToolSurface` + `ToolExecutor` + `SandboxBackend`

### 2.5 Couche Agents (SubagentRunner)

**Sources :** SuperAGI (workflow steps + wait-permission), codebuff (fan-out multi-agents + prefix-cache), onlook (agent isolation), GSD (subagents planner/executor/verifier), TikTok (agents IA, Jarvis, skills Claude)

**Capacités identifiées :**
- Fan-out multi-agents (code-searcher + workers)
- Héritage prefix-cache (byte-for-byte)
- Wait-for-permission (checkpoint humain)
- Workflow steps persistés (iteration/tool/wait/queue)
- Isolation par organisation
- Sous-agents externes (Claude Code, Codex, ACP)
- Modes d'agent (plan/build/debug)
- Turn-ending rule (re-injection d'outil)

**Port Omega :** `SubagentRunner` + `AgentLoop` + `AgentStore`

### 2.6 Couche Sécurité (ExecPolicy + PermissionPolicy)

**Sources :** GSD (secret-read-guard, write-guard, injection-scanner), onlook (guardrails), refact-ai (privacy blocklist), nanocoder (bash failure capsule + approval policy), TikTok (guardrails code généré)

**Capacités identifiées :**
- Blocklist de fichiers sensibles (.env, *.pem, .ssh, etc.)
- Workspace isolation (paths within worktree)
- Injection scanning (contenu lu)
- Privacy mode (local vs cloud)
- Approval policy (confirmation tierce)
- Bash failure capsule (sortie structurée)
- Guardrails du code généré (similarité/licence)
- cody-ignore (filtrage contexte)

**Port Omega :** `ExecPolicy` + `PermissionPolicy`

### 2.7 Couche Automatisation (Scheduler + EventStore)

**Sources :** SuperAGI (Celery jobs), sweepai (webhook bus), onlook (background tasks), TikTok (ManyChat automation, Zapier alternatives)

**Capacités identifiées :**
- Webhook bus (ticket/PR/comment/check suite)
- Jobs récurrents (cron + queue)
- Event sourcing (journal append-only)
- Replay (rejeu de runs)
- Snapshot/restore (points de retour)
- Multi-PR orchestration

**Port Omega :** `EventStore` + `Scheduler` + `Hook`

### 2.8 Couche UI (UiSurface + UiPanel)

**Sources :** bolt.new (workbench stores), onlook (sandbox browser), TikTok (dashboards, webview, live artifacts)

**Capacités identifiées :**
- Workbench stores par domaine (chat/files/editor/terminal/previews)
- WebContainer in-browser (sandbox isolé)
- Live artifacts (dashboards connectés)
- Webview React (panel IDE)
- Auto-open correspondant (fichier jumeau)
- Streaming UI (event-sourced render)

**Port Omega :** `UiSurface` + `UiPanel`

### 2.9 Couche Routage (Selector + FallbackPolicy)

**Sources :** ZenRouter (24 modèles), codebuff (freebuff model ids), TikTok (OmniRoute, model routing, Claude/ChatGPT routing)

**Capacités identifiées :**
- Multi-provider routing (free-first)
- Fallback chain (par modèle)
- Token budget tracking
- Model scoring (communauté + benchmark)
- Stream multiplexing (switchable stream)
- Prompt cache (prefix cache partagé)
- Task classification (par type de tâche)

**Port Omega :** `Selector` + `FallbackPolicy` + `ScoringPolicy` + `TaskClassifier` + `Provider`

### 2.10 Couche Observabilité (TelemetrySink)

**Sources :** GSD (context monitor, injection scanner), codebuff (browser-logs, evals), refact-ai (stats telemetry), TikTok (dashboards, métriques)

**Capacités identifiées :**
- Telemetry per run (tokens, temps, appels)
- Browser logs capture (preview debug)
- Eval harness (benchmark par agent)
- Stats aggregation (dashboard self-hosted)
- Redaction before emission
- Export borné hors thread UI

**Port Omega :** `TelemetrySink` + `EventStore`

---

## 3. Architecture Plugin — "Everything is a Plugin"

### 3.1 Principe fondamental

> **Toute brique logicielle est un plugin qui se branche sur un port.**
> **Le noyau ne connaît que les ports, jamais les implémentations.**

### 3.2 Ports Omega (16 existants + nouveaux proposés)

| Port | Rôle | Status actuel | Nouveaux adaptateurs proposés |
|------|------|---------------|-------------------------------|
| `Provider` | Abstraction LLM | ✅ Implémenté | OmniRoute, Claude Code, Codex, Gemini, Ollama |
| `ContextEngine` | Moteur de contexte | ✅ Implémenté | Tree-sitter index, vecdb, knowledge graph, context pruning |
| `MemoryStore` | Mémoire persistante | ✅ Implémenté | Frontmatter memories, semantic memory, recall/consolidate |
| `ToolExecutor` | Exécution d'outils | ✅ Implémenté | 40+ handlers, LSP tools, browser tools |
| `SandboxBackend` | Isolation | ✅ Implémenté | WebContainer, E2B, namespace Linux |
| `TaskClassifier` | Classification de tâche | ✅ Implémenté | 2-tier evaluate, role classifier |
| `ScoringPolicy` | Scoring de modèles | ✅ Implémenté | Communauté + benchmark + cost-aware |
| `Selector` | Sélection de modèle | ✅ Implémenté | Free-first, task-aware, fallback |
| `FallbackPolicy` | Politique de fallback | ✅ Implémenté | Chain par modèle, health check |
| `CatalogSource` | Source de catalogue | ✅ Implémenté | Model catalog, tool catalog, skill catalog |
| `TelemetrySink` | Télémétrie | ✅ Implémenté | Per-run metrics, browser logs, evals |
| `EventStore` | Journal d'événements | ✅ Implémenté | Webhook bus, snapshot/restore, replay |
| `Hook` | Hooks événementiels | ✅ Implémenté | PreToolUse, PostToolUse, SessionStart, etc. |
| `ExecPolicy` | Politique d'exécution | ✅ Implémenté | Blocklist, approval, sandbox restrictions |
| `PermissionPolicy` | Permissions | ✅ Implémenté | Trust levels, confirm/deny, file d'attente |
| `SubagentRunner` | Sous-agents | ✅ Implémenté | Local, Claude Code, Codex, ACP |
| `Registry` | Registre de plugins | 🆕 Proposé | Manifest, entry info, slot/ensemble, quarantaine |
| `UiSurface` | Surface UI | 🆕 Proposé | Slots (root/sidebar/main/rightbar/overlay), thème par surface |
| `ConfigSource` | Source de config | 🆕 Proposé | Layers + precedence, profiles, schema validation |
| `SkillStore` | Magasin de skills | 🆕 Proposé | Discover/load, policy, experiences comme savoirs |
| `AgentLoop` | Boucle d'agent | 🆕 Proposé | Chat/step/recover, pipeline d'étapes, repair loop |
| `AgentStore` | Magasin d'agents | 🆕 Proposé | Discover/load/save, roster, presets |
| `WorkflowEngine` | Moteur de workflow | 🆕 Proposé | Build/compile/run, route/send, checkpoint |
| `SandboxBackend` | Backend sandbox | 🆕 Proposé | WebContainer, E2B, namespace Linux, par branche |

### 3.3 Adaptateurs par port

Chaque port a des adaptateurs (implémentations) qui sont des plugins interchangeables :

| Port | Adaptateurs actuels | Adaptateurs proposés (depuis corpus) |
|------|-------------------|--------------------------------------|
| `Provider` | omega-providers (openai/anthropic) | + OmniRoute, Claude Code, Codex, Gemini, Ollama, local (llama) |
| `ContextEngine` | omega-context (basic) | + Tree-sitter index, vecdb, knowledge graph, context pruning, windowed reads |
| `MemoryStore` | omega-app (FileMemoryStore) | + Frontmatter memories, semantic memory, recall/consolidate |
| `ToolExecutor` | omega-app (basic) | + 40+ handlers, LSP tools, browser tools, apply-patch |
| `SandboxBackend` | omega-sandbox (namespace) | + WebContainer, E2B, par branche |
| `ToolSurface` | omega-context (basic) | + Registry, profiles, tool creator, LSP diagnostics |
| `SubagentRunner` | omega-app (LocalSubagentRunner) | + Claude Code, Codex, ACP, fan-out prefix-cache |
| `ExecPolicy` | omega-core (basic) | + Blocklist, approval policy, privacy mode, bash capsule |
| `PermissionPolicy` | omega-core (basic) | + Trust levels, confirm/deny, file d'attente |
| `EventStore` | omega-server (basic) | + Webhook bus, snapshot/restore, replay, artifact journal |
| `TelemetrySink` | omega-otel (basic) | + Per-run metrics, browser logs, evals, redaction |
| `Hook` | omega-core (basic) | + PreToolUse, PostToolUse, SessionStart, injection scan |
| `Selector` | omega-router (basic) | + Free-first, task-aware, fallback chain, scoring |
| `FallbackPolicy` | omega-router (basic) | + Chain par modèle, health check |
| `ScoringPolicy` | omega-router (basic) | + Communauté + benchmark + cost-aware |
| `TaskClassifier` | omega-router (basic) | + 2-tier evaluate, role classifier |

---

## 4. Roadmap — Convergence Architecture

### Milestone v0.4 — "PLUGIN CONVERGENCE"

**Objectif :** Adapter l'architecture complète au modèle "everything is a plugin" en intégrant les capacités extraites du corpus (51 harnais + 148 vidéos TikTok).

### Phase 11: Convergence des outils TikTok
- Extraire et classifier tous les outils des 148 vidéos TikTok
- Créer des adaptateurs plugin pour chaque outil identifié
- Documenter les patterns d'intégration

### Phase 12: Adaptation architecture "everything is a plugin"
- Créer les 8 nouveaux ports (Registry, UiSurface, ConfigSource, SkillStore, AgentLoop, AgentStore, WorkflowEngine, SandboxBackend)
- Implémenter les adaptateurs pour chaque port
- Mettre à jour le registre de plugins

### Phase 13: Intégration du travail existant (odysseusAgentOS)
- Importer les adaptateurs pertinents du repo principal
- Fusionner les architectures (Python + Rust)
- Créer les ponts (bridge) nécessaires

### Phase 14: Validation end-to-end
- Tests d'intégration pour chaque port
- Tests de plugin (install/uninstall/reload)
- Validation sandbox
- Documentation complète

---

## 5. Organisation des docs

### Structure proposée

```
docs/omega/
├── reverse-eng/
│   ├── CONVERGENCE-TOUT.md          # Ce document
│   ├── OMEGA-COMPLET.md            # Corpus combiné (déjà existant)
│   ├── HARNESS-CENSUS.md           # Census des harnais (déjà existant)
│   ├── HARNESS-INTAKE.md           # Intake REA (déjà existant)
│   ├── dissections/                 # 51 fiches dissection
│   └── tiktok/                      # Nouveau : outils TikTok
│       ├── OUTILS-VEILLE-LOUCASH.md
│       ├── OUTILS-VEILLE-AXELAYZEIA.md
│       ├── OUTILS-VEILLE-THOMASBSSH1.md
│       └── VIDEOS-*.json
├── adr/
│   ├── 0010-plugin-convergence.md   # Nouveau ADR
│   └── ...
├── research/
│   ├── rea/                         # Preuves REA
│   └── loucash/                     # Recherche TikTok
└── rules/
    └── RULES-OMEGA.md              # Règles (déjà existant)
```

---

## 6. Prochaines actions immédiates

1. **Créer le dossier `tiktok/`** dans `docs/omega/reverse-eng/` et y déplacer les fichiers TikTok
2. **Créer l'ADR 0010** pour la convergence plugin
3. **Mettre à jour le ROADMAP.md** avec le milestone v0.4
4. **Créer les phases GSD** pour v0.4
5. **Commiter** les changements

