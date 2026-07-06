# ANALYSE UNIFIÉE — Outils Tiers & Intégration AgentOS

> **Document maître** : Fusion des 5 analyses (ANALYSE-OUTILS-TIERS.md, ANALYSE-MAITRE.md, ANALYSE-OUTILS-BATCH3.md, ANALYSE-OUTILS-INTEGRES.md, RECHERCHE-OUTILS-MAITRE.md)
> **Dernière vérification stats** : 12/06/2026 (live GitHub)
> **Méthode** : Recherche live (GitHub, docs officielles, webfetch)
> **Critères d'évaluation** : Token efficiency | Complexité | Pertinence | Valeur ajoutée | Maturité

---

## 📑 TABLE DES MATIÈRES

1. [Index Maître — 60 outils](#1-index-maître)
2. [Méthodologie & Grille d'Évaluation](#2-méthodologie)
3. [Architecture — Les 5 Couches AgentOS](#3-architecture)
4. [Couche 1 — Connaissance (Trinité)](#4-couche-1--connaissance)
5. [Couche 2 — Exécution (Engines)](#5-couche-2--exécution)
6. [Couche 3 — Design](#6-couche-3--design)
7. [Couche 4 — Observabilité](#7-couche-4--observabilité)
8. [Couche 5 — Automation](#8-couche-5--automation)
9. [Modules Optionnels & Infrastructure](#9-modules-optionnels)
10. [Patterns Absorbés](#10-patterns-absorbés)
11. [Banque d'Idées](#11-banque-didées)
12. [Outils Ignorés](#12-outils-ignorés)
13. [État d'Intégration Actuel](#13-état-dintégration)
14. [Sources & Références](#14-sources)

---

## 1. INDEX MAÎTRE

> **60 outils analysés** sur 3 batches (Mai-Juin 2026).
> Verdict : ✅ Intégrer | 📋 Absorber patterns | 👀 Veiller | 🏦 Banque d'idées | ❌ Ignorer

### ✅ À INTÉGRER (23 outils)

| # | Outil | Stars | Verdict | Score | Décision |
|---|-------|-------|--------|-------|----------|
| 0 | **OpenAgentsControl (OAC)** | 4.3k | 🏗️ Projet socle | — | Fondation |
| 1 | **CBM** (codebase-memory-mcp) | — | ✅ Structure always-on | — | Intégré actif |
| 2 | **Graphify** | — | ✅ Sémantique on-demand | 9/10 | Intégré actif |
| 3 | **Obsidian** (lecture directe) | — | ✅ Mémoire persistante | 8/10 | Intégré actif |
| 4 | **Scrapling** | 63.1k | 🔥🔥 Web unifié | **10/10** | **Intégré actif** |
| 5 | **n8n MCP** | — | 🔥 Orchestrateur central | 9/10 | Intégré actif |
| 6 | **Supabase MCP** | 2.7k | 🔥 DB exécution | 9/10 | Intégré actif |
| 7 | **Kroki** | — | ✅ Diagrammes 25+ langages | 8/10 | Intégré actif |
| 8 | **API Toolkit** (TMDB/Mapbox/OWM) | — | ✅ Data sources demo | 7/10 | Intégré actif |
| 9 | **Paperclip** | — | ✅ Gouvernance agents | 8/10 | Intégré actif |
| 10 | **Playwright** | — | ✅ Tests E2E sandbox | 8/10 | Intégré actif |
| 11 | **Serena** | 25.3k | ✅ Édition sémantique IDE | 29/35 | **Intégrer P0** |
| 12 | **Faker.js** | 15.4k | ✅ Données de test | 28/35 | **Intégrer P1** |
| 13 | **Crawl4AI** (fallback) | 68.3k | ✅ Backup Scrapling | — | Configuré (off) |
| 14 | **Browser-Harness** (backup) | 14.7k | ✅ Backup headless | — | Configuré (off) |
| 15 | **Design Extract** (designlang) | 3.2k | ✅ Extraction design→code | 9/10 | Configuré (off) |
| 16 | **Open Design** (nexu-io) | — | ✅ Workflow design complet | — | Configuré (off) |
| 17 | **CodeBurn** | — | ✅ Observabilité tokens | 8/10 | Configuré (off) |
| 18 | **Acontext** | 3.5k | 🔥 "Skill as Memory" auto-capture | 30/35 | **Intégrer P0** |
| 19 | **Build Your Own X** | 499k | ✅ Corpus RAG agents | 8/10 | À intégrer |
| 20 | **AutoResearch** (Karpathy) | 83k | ✅ Benchmark auto | 8/10 | À intégrer |
| 21 | **Obsidian Second Brain** | — | ✅ 3ème couche connaissance | 7/10 | À intégrer |
| 22 | **OpenWA** | 7.7k | ✅ Canal WhatsApp | 15/35 | Optionnel |
| 23 | **Chatwoot** | — | ✅ Chat interface agents | 5/10 | Phase lointaine |

### 📋 À ABSORBER — Patterns (6)

| # | Outil | Stars | Verdict | Score | Action |
|---|-------|-------|--------|-------|--------|
| 24 | **agents-best-practices** | 1.9k | 🔥 Framework harness universel | **10/10** | Intégré actif |
| 25 | **OmO** (Oh My OpenAgent) | 61.9k | 📋 Tiered routing, fallback, agents | 7/10 | Absorbé |
| 26 | **HexStrike AI** | 8.6k | 📋 Permission matrix, SAST, sandbox | 5/10 | Absorbé |
| 27 | **HolyClaude** | 2.3k | 📋 Patterns Docker infrastructure | 8/10 | Absorbé partiel |
| 28 | **vibecode-pro-max-kit** | 847 | 📋 Quality pipeline, phase-locking | 25/35 | Absorber |
| 29 | **Caveman Method** | — | 📋 Debug standard obligatoire | 6/10 | Absorbé |

### 👀 VEILLE (7)

| # | Outil | Stars | Score | Intérêt |
|---|-------|-------|-------|---------|
| 30 | **Odysseus** | 68.4k | 26/35 | Self-hosted AI workspace — Cookbook + Deep Research |
| 31 | **master-skill** | 49 | 24/35 | Distillation industrie→skill |
| 32 | **SurfSense** | 14.4k | 23/35 | Alternative open-source NotebookLM — RAG, Podcast, Automations |
| 33 | **ANUS** | 6.4k | 18/35 | Auto-évolution roadmap |
| 34 | **Twenty** | 49.4k | 17/35 | CRM open-source, pattern SDK |
| 35 | **Hermes** (Nous Research) | — | 16/35 | Agent framework, cron, kanban |
| 36 | **Career-Ops** | 43.1k | — | Pattern marketplace skills |

### 🏦 BANQUE D'IDÉES (10)

| # | Outil | Stars | Potentiel |
|---|-------|-------|-----------|
| 37 | anime.js | 52k | Animation dashboard Studio |
| 38 | reactbits.dev | — | Composants React animés |
| 39 | UIverse.io | — | Templates UI |
| 40 | Blender MCP | 565 | 3D asset generation |
| 41 | SketchUp MCP | 245 | Pattern desktop bridge |
| 42 | MediaAgent / OpenGenAI | — | Génération image/vidéo 200+ modèles |
| 43 | Design Galleries | — | 1600+ refs UI (supahero, navbar.gallery, etc.) |
| 44 | Plausible Analytics | — | Analytics privacy-first |
| 45 | ALTCHA | 2.2k | Captcha PoW self-hosted |
| 46 | ScrapGraphAI | — | Scraping + knowledge graph (redondant) |

### ❌ IGNORÉS (14)

| # | Outil | Raison |
|---|-------|--------|
| 47 | Almanac MCP | Redondant (ExternalScout + Context7) |
| 48 | Wan2GP | Génération vidéo — hors scope |
| 49 | Open Generative AI | Création media — hors scope |
| 50 | Open Design (article) | Site anti-scraping |
| 51 | p-e-w/heretic (20.6k) | Censure LLM — modifie modèles, pas agents |
| 52 | Kling AI | Génération vidéo — pas de MCP |
| 53 | Napkin AI | Pas d'API (web app only) |
| 54 | Autodesk Fusion MCP (41) | CAD propriétaire — niche |
| 55 | AppFlowy-IO | Pas d'API viable |
| 56 | Omma AI | Mineur — pas pertinent |
| 57 | blueprint.am | AI hardware design — hors scope |
| 58 | ANUS (3⭐ version) | Framework blockchain, non maintenu |
| 59 | flowint | N'existe pas |
| 60 | AContext / Odysseus | Introuvables sur GitHub (juin 2026) |

---

## 2. MÉTHODOLOGIE

### Grille d'évaluation — 7 axes (score /35)

| # | Critère | Poids | Détail |
|---|---------|-------|--------|
| 1 | **INTÉGRATION** | /5 | Docker, MCP, API REST, npm/pip, volume, dépendances |
| 2 | **PATTERN** | /5 | Features uniques, meilleure approche, architecture innovante |
| 3 | **MODEL INTEL** | /5 | Provider discovery, scoring, fallback, routing intelligent |
| 4 | **BRIQUE LOGIQUE** | /5 | Gap comblé, remplacement, redondance, complémentarité |
| 5 | **FEATURES** | /5 | Core features, maturité, licence, stack, communauté |
| 6 | **FIT AGENTOS** | /5 | Complémentarité, chevauchement, refactoring, workflow |
| 7 | **COÛT/VALEUR** | /5 | Impact tokens, taille Docker, maintenance, support |

### Seuils de décision

| Score | Verdict |
|-------|--------|
| 28-35 | ✅ INTÉGRER |
| 21-27 | 📋 ABSORBER (patterns) |
| 14-20 | 👀 VEILLE (surveiller) |
| 0-13 | ❌ IGNORER |

---

## 3. ARCHITECTURE — Les 5 Couches AgentOS

> **Projet socle** : [OpenAgentsControl (OAC)](https://github.com/darrenhinde/OpenAgentsControl) — 4.3k⭐, 215 commits, MIT, v0.7.1

```
┌─────────────────────────────────────────────────┐
│              COUCHE 5 — AUTOMATION               │
│         n8n MCP (400+ intégrations)              │
├─────────────────────────────────────────────────┤
│           COUCHE 4 — OBSERVABILITÉ               │
│    CodeBurn (tracking 18 outils AI coding)       │
│    AutoResearch (benchmark auto)                 │
├─────────────────────────────────────────────────┤
│              COUCHE 3 — DESIGN                   │
│  Open Design (31 skills, 129 design systems)     │
│  Design Extract (19 formats output)              │
├─────────────────────────────────────────────────┤
│            COUCHE 2 — EXÉCUTION                  │
│  Core OAC (OpenAgent + OpenCoder + SystemBuilder │
│            + 11 subagents + 198 contextes)       │
│  Scrapling (web unifié)                          │
│  Supabase MCP (database)                         │
│  API Toolkit (TMDB, Mapbox, OWM, News)           │
│  Kroki (diagrammes 25+ langages)                 │
│  Serena (édition sémantique IDE)                 │
│  Faker.js (données de test)                      │
├─────────────────────────────────────────────────┤
│          COUCHE 1 — CONNAISSANCE (Trinité)        │
│  CBM (structure code always-on)                  │
│  Graphify (sémantique on-demand)                 │
│  Obsidian (mémoire persistante)                  │
└─────────────────────────────────────────────────┘
```

### Règle de questionnement — Trinité Connaissance

```
1. CBM d'abord   → structure code (always-on, 100-200 tokens/query)
2. Graphify       → sémantique si compréhension large (500 tokens/query)
3. Obsidian       → mémoire si historique/contexte (200-500 tokens/query)
→ Fusionner les 3 résultats avant de répondre
```

---

## 4. COUCHE 1 — CONNAISSANCE (Trinité)

Ces 3 ressources sont questionnées ENSEMBLE pour chaque requête.

### 4.1 CBM (codebase-memory-mcp)

| Propriété | Valeur |
|---|---|
| **Type** | MCP Server (Go) — always-on |
| **Projet indexé** | 868 nodes, 932 edges |
| **Outils** | 14 (search_graph, trace_path, query_graph, etc.) |
| **Query** | Cypher + full-text search + path tracing |
| **Portée** | Code uniquement (66+ langages via tree-sitter) |
| **Token** | 100-200/query |
| **Quand** | Call tracing, dead code, impact analysis, navigation code |

### 4.2 Graphify

| Propriété | Valeur |
|---|---|
| **Type** | Knowledge Graph Skill (Python) |
| **Package** | `graphifyy` v0.4.3 (PyPI) |
| **Multi-input** | Code (AST), docs, PDFs, images (vision), vidéos (Whisper), URLs |
| **Double extraction** | AST (déterministe) + Sémantique (Claude subagents) |
| **Community detection** | Leiden/graspologic clustering |
| **Sorties** | HTML (D3.js), SVG, JSON, GraphML, Neo4j, Obsidian vault, MCP server |
| **Performance** | AgentOS: 2,581 nodes, 3,698 edges, 293 communautés, 8.1x compression |
| **Token** | 500/query (vs 50K lecture brute) |
| **Quand** | Onboarding projet, compréhension architecture, liens code↔docs |

**Comparaison CBM vs Graphify :**

| Dimension | CBM | Graphify |
|---|---|---|
| Scope | Code uniquement | Code + docs + images + vidéos + URLs |
| Extraction | tree-sitter (déterministe) | AST (gratuit) + Claude sémantique (LLM) |
| Persistence | SQLite | Fichier `graph.json` |
| Runtime | Always-on (MCP server) | On-demand (`/graphify .`) |
| Query | Cypher + search_graph + trace_path | BFS/DFS/path finding/explain |
| Inférence | ❌ Structurel uniquement | ✅ Sémantique (concepts, liens implicites) |
| Clustering | ❌ Aucun | ✅ Community detection + scoring |
| Visualisation | Web UI (localhost:9749) | HTML, SVG, Obsidian, Neo4j |
| Token savings | 120x (claim) | 8-72x (mesuré) |
| Cross-artifact | ❌ Code seulement | ✅ Connexions docs/code |

### 4.3 Obsidian

| Propriété | Valeur |
|---|---|
| **Type** | Lecture directe fichiers markdown — pas de MCP |
| **Vault** | `/agentos/{type}/` |
| **Templates** | 7 templates (wf1-7-*.md) |
| **Token** | 200-500/query |
| **Quand** | Rappel contexte passé, décisions architecturales, patterns récurrents |

---

## 5. COUCHE 2 — EXÉCUTION

### 5.1 Core OAC — OpenAgentsControl

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/darrenhinde/OpenAgentsControl` |
| **Stars** | **4.3k** (vérifié 12/06/2026) |
| **Commits** | 215 |
| **Releases** | 9 — v0.7.1 (30 jan 2026) |
| **Licence** | MIT |
| **Installation** | `curl -fsSL .../install.sh \| bash -s developer` |
| **Plugin** | Claude Code plugin (BETA) disponible |
| **Contextes** | 198 fichiers contextuels |

**Agents principaux :**
- **OpenAgent** — tâches générales, questions, onboarding (commencer ici)
- **OpenCoder** — développement production, features complexes
- **SystemBuilder** — génération de systèmes AI custom (wizard interactif)

**11 Subagents auto-délégués :**
- ContextScout — découverte intelligente de patterns
- TaskManager — découpage JSON de features complexes
- CoderAgent — implémentations ciblées
- TestEngineer — TDD et tests
- CodeReviewer — review et analyse sécurité
- BuildAgent — type checking et validation build
- DocWriter — génération documentation
- ExternalScout — fetch docs live (Context7 + filtrage)
- + spécialistes : frontend, devops, copywriter, technical-writer, data-analyst

**Principes clés :**
- **MVI** (Minimal Viable Information) — 80% réduction tokens, fichiers <200 lignes
- **Approval Gates** — l'humain approuve avant toute exécution
- **Agents éditables** — fichiers Markdown, pas de boîte noire
- **Model Agnostic** — Claude, GPT, Gemini, MiniMax, modèles locaux
- **Team-ready** — contextes commitables dans le repo

### 5.2 Scrapling — Web Unifié 🔥🔥

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/D4Vinci/Scrapling` |
| **Stars** | **63.1k** (vérifié 12/06/2026) |
| **Commits** | 1,476 |
| **Releases** | 47 — v0.4.9 (7 juin 2026) |
| **Licence** | BSD-3-Clause |
| **Auteur** | Karim Shoair |
| **MCP** | ✅ Natif — extraction ciblée pré-LLM |
| **Agent skill** | ✅ Disponible pour Claude Code/OpenCode |
| **Installation** | `pip install scrapling[all]` + `scrapling install` |
| **Docker** | Image prête avec tous les browsers |

**Pourquoi Scrapling remplace Crawl4AI + Browser-Harness :**

| Dimension | Scrapling | Crawl4AI | Browser-Harness |
|---|---|---|---|
| Fetch HTTP | ✅ Fetcher + impersonation TLS/HTTP3 | ✅ Fit Markdown + BM25 | ❌ (focus CDP) |
| Fetch headless | ✅ StealthyFetcher + DynamicFetcher | ❌ Non | ✅ CDP direct |
| Anti-bot | ✅ Cloudflare Turnstile natif | ⚠️ 3-tiers (basique) | ✅ Cloud browsers |
| Spider/Crawl | ✅ Framework complet (concurrent, pause/resume, streaming) | ✅ Deep crawl BFS/DFS | ❌ Non |
| **Adaptive scraping** | ✅ Auto-relocalisation éléments | ❌ Non | ❌ Non |
| MCP server | ✅ Natif, extraction ciblée | ✅ Natif | ❌ Non |
| CLI | ✅ `scrapling extract` + shell IPython | ✅ `crwl` CLI | ❌ Non |
| Token efficiency | ✅ Extraction ciblée (80-90% réduction) | ✅ Fit Markdown BM25 | ❌ Non |
| Speed (parser) | 2.02ms (ultra rapide) | Non benchmarké | N/A |
| Agent skill | ✅ `agent-skill/` prêt | ❌ | ❌ |

**Comparaison nettement renforcée** depuis l'analyse initiale : les 63.1k stars, les 47 releases et l'adaptive scraping (capacité unique de survie aux changements de site) confirment la domination de Scrapling.

### 5.3 Supabase MCP

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/supabase/mcp` |
| **Stars** | 2.7k (vérifié 12/06/2026) |
| **Commits** | 389 |
| **Releases** | 38 |
| **Type** | MCP server officiel Supabase |
| **Outils** | 22+ (SQL, migrations, auth, storage, edge functions, branching) |
| **Read-only mode** | ✅ `?read_only=true` |
| **Feature groups** | ✅ `?features=database,docs` |
| **Project scoping** | ✅ `?project_ref=<id>` |
| **SDK** | `@supabase/mcp-server-supabase` + Vercel AI SDK |

### 5.4 Serena — Édition Sémantique IDE

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/oraios/serena` |
| **Stars** | **25.3k** (vérifié 12/06/2026) |
| **Commits** | 2,946 |
| **Releases** | 13 — v1.5.3 (26 mai 2026) |
| **Licence** | MIT |
| **Langages** | 40+ (via LSP) |
| **Installation** | `uv tool install -p 3.13 serena-agent` |
| **Backends** | LSP (gratuit) ou JetBrains Plugin (payant) |
| **Docker** | ✅ `compose.yaml` inclus |
| **Token** | 0 (LSP local), ~30MB |

**Outils clés :**
- **Retrieval** : find symbol, symbol overview, find referencing symbols, type hierarchy
- **Refactoring** : rename, move (JetBrains), inline (JetBrains)
- **Editing** : replace symbol body, insert after/before symbol, safe delete
- **Debugging** : breakpoints, inspect variables, REPL (JetBrains plugin)
- **Memory** : système de mémoire cross-session

**Où dans AgentOS :** WF3 (Code), WF5 (Debug), WF6 (Review)

### 5.5 Faker.js — Données de Test

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/faker-js/faker` |
| **Stars** | **15.4k** (vérifié 12/06/2026) |
| **Commits** | 4,212 |
| **Releases** | 61 — v10.4.0 (23 mars 2026) |
| **Licence** | MIT |
| **Locales** | 70+ |
| **Modules** | Person, Location, Date, Finance, Commerce, Hacker, Number, String |
| **Installation** | `npm install --save-dev @faker-js/faker` |

**Où dans AgentOS :** Sandbox TestEngineer, WF3 Code (fixtures)

### 5.6 Kroki — Diagrammes

- 25+ langages (PlantUML, Mermaid, GraphViz, C4, ERD, etc.)
- REST API unifiée — Docker local port 8000
- 0 token — usage local
- Utilisé dans WF1_Brainstorm.json, WF2_Planification.json

### 5.7 API Toolkit

- TMDB (films/séries), Mapbox (50k maps/mois gratuites), OpenWeatherMap, News API
- Context files documentés dans `api-toolkit/*.md`
- Dataset de démo pour agents full-stack

### 5.8 Crawl4AI (fallback — non actif)

| Propriété | Valeur |
|---|---|
| **Stars** | **68.3k** (vérifié 12/06/2026) |
| **Commits** | 1,533 |
| **Version** | v0.8.9 |
| **Licence** | Apache 2.0 |
| **Statut** | Configuré dans `.opencode/config.json` avec `"enabled": false` |

### 5.9 Browser-Harness (fallback — non actif)

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/browser-use/browser-harness` |
| **Stars** | **14.7k** (vérifié 12/06/2026) |
| **Commits** | 391 |
| **Licence** | MIT |
| **Type** | CDP harness (~1k lignes, 4 fichiers core) |
| **Statut** | Configuré dans `.opencode/config.json` avec `"enabled": false` |

---

## 6. COUCHE 3 — DESIGN

### 6.1 Design Extract (designlang)

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/Manavarya09/design-extract` |
| **Stars** | **3.2k** (vérifié 12/06/2026) |
| **Commits** | 201 |
| **Releases** | 32 — v12.15.0 "motionlang" (21 mai 2026) |
| **Licence** | MIT |
| **Installation** | `npm i -g designlang` |

**17+ formats de sortie :** DTCG tokens, Tailwind v4 config, shadcn/ui theme, Figma variables, motion tokens, typed React components, brand voice, prompts pack, CSS variables, MCP server payload, design report card (grade), etc.

**Fonctionnalités clés v12 :**
- `designlang pair` — fusionne 2 designs (visuels A × voix B)
- `designlang brand` — brand guidelines book (13 chapitres)
- `designlang theme-swap` — recoloration autour d'une marque
- `designlang remix` — restyling en 6 vocabulaires (brutalist, swiss, art-deco, cyberpunk, soft-ui, editorial)
- `designlang battle` — combat noté entre 2 designs
- MCP server + Claude Code plugin (/extract, /grade, /battle, /remix, /pack)

### 6.2 Open Design (nexu-io)

- Alternative open-source à Claude Design
- 31 skills (web, mobile, dashboard, slides, email...)
- 129 design systems (Linear, Stripe, Vercel, Apple, Tesla...)
- 6-layer prompt stack composable
- MCP server 7 tools, REST API, Electron desktop
- 31.8k★, Apache 2.0, 355 commits — supporte OpenCode
- **Statut :** Configuré dans `.opencode/config.json` avec `"enabled": false`

---

## 7. COUCHE 4 — OBSERVABILITÉ

### 7.1 CodeBurn

- Track 18 AI coding tools (OpenCode, Claude Code, Codex, Cursor, Copilot...)
- 13 catégories de tâches, 13 waste detectors, A-F health grade
- Optimize mode : trouve les patterns de gaspillage
- Installation : `npm install codeburn` — lit données depuis disque, zéro token
- **Statut :** Configuré dans `.opencode/config.json` avec `"enabled": false`

### 7.2 AutoResearch (Karpathy)

- `karpathy/autoresearch` — 83k★ (écosystème : atlas-gic 1.7k, aideml 1.3k, autocontext 969, scholaraio 420, PhD-Zero)
- Benchmark automatique d'implémentations, comparaison d'approches
- Module "OAC Research" pour lancer des expériences, générer des rapports

---

## 8. COUCHE 5 — AUTOMATION

### 8.1 n8n MCP — Orchestrateur Central

| Aspect utilisé | Où |
|---------------|-----|
| Webhook triggers (12 workflows) | `n8n/workflows/WF*.json` + pipelines |
| HTTP Request nodes (MCP calls) | `pipeline-entree.json`, `pipeline-sortie.json` |
| Switch routing (matrix adaptative) | `pipeline-entree.json`, `WF_SpawnAgent.json` |
| Code nodes (JavaScript) | `WF3_Code.json`, `WF_ModelIntelligence.json` |
| Docker socket mount | `docker-compose.yml` (volume `/var/run/docker.sock`) |
| REST API | dashboard → n8n (health check, workflow management) |

---

## 9. MODULES OPTIONNELS & INFRASTRUCTURE

### 9.1 agents-best-practices 🔥🔥

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/DenisSergeevitch/agents-best-practices` |
| **Stars** | **1.9k** (vérifié 12/06/2026) |
| **Commits** | 7 |
| **Licence** | MIT |
| **Format** | Agent Skill (SKILL.md) — provider-neutral (OpenAI + Anthropic + OpenCode) |
| **Références** | 15 fichiers markdown structurés |
| **Installation** | `npx skills add DenisSergeevitch/agents-best-practices -g` |

**Ce que ça apporte :**
1. **MVP Blueprint Builder** — Génère un blueprint concret de harness agent
2. **Audit Framework** — Diagnostique les harness existants
3. **Loop canonique** — Boucle provider-neutral standardisée
4. **Tool & Permission Design** — Contrats de tools typés, classes de risque
5. **Context & Memory** — Compaction intelligente, cache-aware
6. **15 références** couvrant le cycle de vie complet d'un harness

**Utilisation dans AgentOS :**
- Loop canonique 7 étapes → `loop-canonique.md`
- 10 règles non-négociables → `loop-canonique.md`
- Tool & permission design → `permission-matrix.md`, `.opencode/agents/*.md`

### 9.2 Acontext — "Skill as Memory"

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/memodb-io/Acontext` |
| **Stars** | **3.5k** (vérifié 12/06/2026) |
| **Commits** | 1,080 |
| **Releases** | 279 |
| **Licence** | Apache 2.0 |
| **SDK** | Python (`pip install acontext` v0.1.22) + TypeScript (`@acontext/acontext`) |
| **Docs** | `https://docs.acontext.io/` |
| **Plugins** | Claude Code + OpenClaw |
| **Stack** | Go + Python + TypeScript, PostgreSQL, Redis, RabbitMQ, S3 |

**Concept :** "Agent Skills as a Memory Layer" — capture automatique des apprentissages après chaque run, stockés en fichiers Markdown (skills). Self-hosted (Docker) ou Cloud (acontext.io).

**Features :**
- **Session Storage** — messages, fichiers, artifacts, sandboxes
- **Task Tracking** — statut, progression, résumés automatiques
- **Skill Memory** — distillation automatique des runs en skills Markdown
- **Agent Tools** — Disk (filesystem virtuel), Sandbox (exécution isolée), Skill Content (list/get)
- **Context Engineering** — compression, stratégies d'édition
- **Dashboard** — localhost:3000

**Pour AgentOS :** Remplace le pipeline-sortie manuel. Auto-apprentissage après chaque run. Les skills générés sont portables (Markdown files) — compatibles avec n'importe quel agent.

### 9.3 Odysseus — Self-Hosted AI Workspace

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/pewdiepie-archdaemon/odysseus` |
| **Stars** | **68.4k** (vérifié 12/06/2026) |
| **Commits** | 1,150 |
| **Licence** | AGPL-3.0 |
| **Stack** | Python (FastAPI) + JavaScript, ChromaDB, SearXNG |
| **Docker** | ✅ docker-compose.yml (bundled ChromaDB + SearXNG + ntfy) |
| **Native** | Linux, macOS (Apple Silicon), Windows |

**Features clés :**
- **Chat** — tout modèle local ou API (Ollama, vLLM, llama.cpp, OpenRouter, OpenAI, Copilot)
- **Agent** — built on opencode, MCP, web, files, shell, skills, memory
- **Cookbook** — scan hardware → recommande modèles → click to download/serve (VRAM-aware, GGUF/FP8/AWQ)
- **Deep Research** — multi-step runs, gather → read → synthesize → visual report
- **Compare** — side-by-side blind model comparison
- **Documents** — multi-tab editor (Markdown, HTML, CSV)
- **Memory/Skills** — ChromaDB + fastembed (ONNX), vector + keyword retrieval
- **Email** — IMAP/SMTP inbox avec AI triage
- **Calendar** — CalDAV sync (Radicale, Nextcloud, Apple, Fastmail)
- **Notes & Tasks** — checklist, cron-style tasks, ntfy notifications
- **PWA** — responsive, installable, touch gestures

**Pour AgentOS :** Concurrent direct sur le workspace AI self-hosted. Patterns à absorber : Cookbook (discovery modèle automatique), Deep Research (multi-step synthesis), Compare (évaluation aveugle). Trop lourd à intégrer en entier.

### 9.4 SurfSense — Alternative Open-Source NotebookLM

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/MODSetter/SurfSense` |
| **Stars** | **14.4k** (vérifié 12/06/2026) |
| **Commits** | 6,600 |
| **Releases** | 21 — v0.0.28 (11 juin 2026) |
| **Licence** | Apache 2.0 |
| **Stack** | Python (FastAPI) + TypeScript (Next.js), LangChain Deep Agents |
| **Docker** | ✅ curl install one-liner + Docker Compose |

**Features clés :**
- **27+ connecteurs** — Google Drive, OneDrive, Dropbox, Notion, Slack, Teams, Jira, Linear, GitHub, Discord, Gmail...
- **AI Report Generator** — rapports avec citations, export PDF/DOCX/HTML/LaTeX/EPUB
- **AI Podcast Generator** — podcast 2-host en <20 secondes
- **AI Presentation & Video Maker** — slides éditables, vidéos narrées
- **AI Resume Builder** — adaptation CV ATS-optimized
- **Scheduled AI Workflows** — briefs quotidiens, digest hebdomadaires
- **Event-Triggered Automations** — agent déclenché à l'arrivée d'un document
- **Hybrid Search** — sémantique + full-text avec indices hiérarchiques
- **Desktop App** — Quick Assist, Screenshot Assist, General Assist
- **Browser Extension** — save any webpage
- **Obsidian Sync** — synchronisation vault
- **Multiplayer** — RBAC + real-time chat
- **100+ LLMs** — OpenAI spec + LiteLLM, 6000+ embedding models

**Pour AgentOS :** RAG avancé + Podcast/Report Generator enrichiraient WF4 Recherche. Trop lourd à intégrer en entier — absorber les patterns uniquement (Podcast Generator, Report Generator).

### 9.6 HolyClaude — Patterns Docker

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/CoderLuii/HolyClaude` |
| **Stars** | **2.3k** (vérifié 12/06/2026) |
| **Commits** | 20 |
| **Licence** | MIT |

**Patterns absorbés :**
- s6-overlay (PID 1, graceful shutdown) — commenté dans `sandbox/shared/Dockerfile.base`
- Xvfb + Chromium Docker config — `sandbox/testengineer/Dockerfile`
- Apprise notifications (100+ services) — commenté dans `docker-compose.yml`
- Bind mount persistence — volumes nommés dans `docker-compose.yml`

### 9.7 Paperclip — Gouvernance

- Multi-company (3 companies), org chart (CEO→CTO→Dev→QA)
- Budgets par agent (15 agents), gouvernance (approvals)
- Webhook bridge → n8n (`WF_Paperclip_Bridge.json`)

---

## 10. PATTERNS ABSORBÉS

### 10.1 OmO — Tiered Routing & Fallback (61.9k⭐)

| Aspect absorbé | Où dans AgentOS |
|---------------|-----------------|
| Tiered routing 70/20/10 | `loop-canonique.md`, `WF3_Code.json`, `WF_ModelIntelligence.json` |
| Fallback chains (rate limit → switch → blacklist 5min) | `loop-canonique.md`, `WF_SpawnAgent.json` |
| Session continuity (Supabase state) | `loop-canonique.md`, `pipeline-sortie.json` → Supabase sessions |
| Agents disciplinaires (7 agents) | `.opencode/agents/*.md` |

**Note :** v4.9.1 (11 juin 2026) — maintenant multi-harness (OpenCode + Codex via LazyCodex). 198 releases, 8,585 commits.

### 10.2 HexStrike — Permission Matrix & Sandbox (8.6k⭐)

| Aspect absorbé | Où dans AgentOS |
|---------------|-----------------|
| Permission matrix (droits explicites) | `permission-matrix.md` |
| SAST (analyse statique) | `sandbox/reviewer/execute.sh` → bandit, eslint |
| Secret scanning | `sandbox/reviewer/execute.sh` → trufflehog, git-secrets |
| Sandbox (isolation) | `docker-compose.yml` → `cap_drop: ALL`, `no-new-privileges` |

### 10.3 vibecode-pro-max-kit — Quality Pipeline & Phase-Locking

| Propriété | Valeur |
|---|---|
| **Repo** | `github.com/withkynam/vibecode-pro-max-kit` |
| **Stars** | **847** (vérifié 12/06/2026) |
| **Commits** | 51 |
| **Licence** | MIT |
| **Agents** | 12 spécialisés, 32 skills, 7 hooks |
| **Supporte** | Claude Code, Codex, Cursor, OpenCode, Windsurf, Antigravity, Copilot |

**Patterns à absorber (pas l'outil entier) :**
1. **Quality pipeline** : self-review → tester → reviewer → simplifier → git (enrichir WF6)
2. **Phase-locked tool restrictions** : capability removal par phase (enrichir loop-canonique)
3. **Pre-implementation debate** : 5 personas (Architect, Security, Perf, UX, Devil's Advocate)
4. **Drift signal scoring** : LOW/MEDIUM/HIGH après exécution

### 10.4 Caveman Method — Standard Obligatoire

```
RÈGLES:
1. Zéro blabla. Phrases < 10 mots.
2. Mots simples. Pas de jargon inutile.
3. Actif, pas passif. "Fais X", pas "X devrait être fait".
4. Une idée par ligne.
5. Code > explication. Montre, ne raconte pas.
6. Si doute → demande. Pas de supposition.
7. Erreur → dis exactement quoi. Pas de "il semble que".
8. Fini → dis "fini". Pas de "je pense que c'est bon".
```

Appliqué dans : `loop-canonique.md`, tous les agents.

---

## 11. BANQUE D'IDÉES

Outils analysés, non intégrés mais avec potentiel futur :

| Outil | Potentiel | Bloquant |
|-------|-----------|----------|
| **ScrapGraphAI** | Scraping + knowledge graph | Redondant (Scrapling+Graphify déjà intégrés) |
| **Build Your Own X** (499k★) | Corpus RAG (DB, OS, Git, Redis...) | Volume de données |
| **MediaAgent / OpenGenAI** | Génération image/vidéo 200+ modèles | Pas prioritaire, MCP 19 tools existe |
| **Design Galleries** | 1600+ refs UI (supahero, navbar.gallery, footer.design, cta.gallery, 404s.design) | Intégration simple |
| **Blender MCP** (565★) | Orchestration multi-MCP (565 tools) | Pattern utile |
| **SketchUp MCP** (245★) | Bridge desktop via MCP | Pattern réutilisable |
| **Chatwoot** | Chat agent interface | Pas prioritaire |
| **Plausible** | Analytics externe | Redondant (CodeBurn) |
| **ALTCHA** (2.2k★) | Captcha PoW | Si endpoints publics |

---

## 12. OUTILS IGNORÉS

| Outil | Raison |
|-------|--------|
| Almanac MCP | Redondant — ExternalScout + Context7 font mieux |
| Wan2GP (5.6k★) | Génération vidéo AI — hors scope |
| Open Generative AI (11.7k★) | Création media — hors scope |
| Open Design (article) | Site anti-scraping, info limitée |
| p-e-w/heretic (20.6k★) | Censure removal LLM — modifie modèles, pas agents |
| Kling AI | Génération vidéo — pas de MCP |
| Napkin AI | Text→diagrams — pas d'API (web app only) |
| Autodesk Fusion MCP (41★) | CAD propriétaire — niche |
| AppFlowy-IO | Pas d'API viable |
| Omma AI | Mineur — pas pertinent |
| blueprint.am | AI hardware design — hors scope |
| ANUS (3★) | Framework blockchain, non maintenu |
| flowint | N'existe pas sur GitHub |

---

## 13. ÉTAT D'INTÉGRATION ACTUEL (Post-Phase 10 — 12/06/2026)

> **Phase 10 exécutée** : Architecture 5 couches formalisée. n8n → Decision Engine Python. 39 fichiers créés/modifiés. MILESTONE COMPLET.

### Bilan complet — 60 outils

| Statut | Nombre | Détail |
|--------|--------|--------|
| **🔴 CORE — Intégrés et actifs** | 10 | CBM, Graphify, Obsidian, Scrapling, Serena, Supabase, Kroki, Playwright, Paperclip, API Toolkit |
| **🟡 PÉRIPHÉRIQUE — Configurés** | 7 | Acontext, Faker.js, CodeBurn, Design Extract, Open Design, Crawl4AI (fallback), Browser-Harness (fallback) |
| **🟡 PÉRIPHÉRIQUE — Documentés** | 3 | Build Your Own X (RAG), AutoResearch (benchmark), OpenWA (veille) |
| **🧠 Patterns — Absorbés** | 7 | agents-best-practices, OmO, HexStrike, vibecode (quality pipeline + phase-locking + debate + drift), Caveman, HolyClaude, OAC (MVI + Approval Gates) |
| **👀 VEILLE** | 6 | Odysseus, master-skill, SurfSense, ANUS, Twenty, Hermes |
| **🏦 BANQUE D'IDÉES** | 11 | anime.js, reactbits, UIverse, Blender MCP, SketchUp MCP, MediaAgent, Design Galleries, Plausible, ALTCHA, ScrapGraphAI, Chatwoot |
| **❌ IGNORÉS** | 14 | Almanac, Wan2GP, OpenGenAI, heretic, Kling AI, Napkin AI, Autodesk Fusion, AppFlowy, Omma AI, blueprint.am, ANUS(3⭐), flowint |
| **🗑️ RETIRÉ** | 2 | **n8n** (remplacé par Decision Engine), Career-Ops (déplacé veille) |
| **TOTAL** | **60** | **MILESTONE COMPLET 🎉** |

### Taux d'utilisation — Post-Phase 10

| Utilisation | Outils |
|-------------|--------|
| **100%** (tous les aspects, CORE actif) | CBM, Graphify, Obsidian (Trinité), Scrapling, Serena, Supabase, Kroki |
| **100%** (patterns entièrement injectés) | Caveman, agents-best-practices, OmO, HexStrike, vibecode, OAC/MVI, HolyClaude |
| **75%+** (actif mais pas tous les aspects) | Paperclip (webhook mis à jour), Playwright (sandbox) |
| **Configuré ON (P1 actif)** | CodeBurn (Couche 4), Faker.js (sandbox) |
| **Configuré ON (P2 actif)** | Design Extract (Couche 3), Open Design (Couche 3) |
| **Configuré OFF (documenté)** | Acontext (README + config, nécessite Docker/clé API), Crawl4AI, Browser-Harness (fallbacks) |
| **Documenté (P2 futur)** | Build Your Own X (BYOX-README.md), AutoResearch |

### Changements majeurs Phase 10

| Avant | Après |
|-------|-------|
| n8n MCP (12 workflows, ~500MB RAM) | **Decision Engine** Python (~300 lignes, ~50MB RAM) |
| Pas d'architecture documentée | `.planning/ARCHITECTURE.md` (184 lignes) |
| Pas de decision tree | `.opencode/skills/decision-tree.md` (75 lignes) |
| Agents sans règle commune | 7 agents: **Trinité + task() + MVI** obligatoires |
| Dashboard Next.js Docker | `dashboard/index.html` (201 lignes, zéro build) |
| 0 pattern vibecode absorbé | Quality pipeline 5 étapes, phase-locking, 5-persona debate, drift scoring |
| Serena configuré mais non référencé | tasks/code.py, debug.py, review.py |
| Design Extract/Open Design OFF | **Enabled ON** (P2, Couche 3) |
| CodeBurn OFF | **Enabled ON** (P1, Couche 4) |
| Acontext absent | Config entry + `acontext/README.md` (54 lignes) |
| Pas de script vérification | `scripts/verify-mcp.ps1` (96 lignes) |
| Pas de corpus RAG | `rag/BYOX-README.md` (46 lignes) |
| Faker.js absent du sandbox | `sandbox/testengineer/fixtures.js` (31 lignes) |

### Nouveaux fichiers créés (Phase 10)

| Fichier | Lignes | Rôle |
|---------|--------|------|
| `.planning/ARCHITECTURE.md` | 184 | Architecture 5 couches formelle |
| `.opencode/skills/decision-tree.md` | 75 | Classifier 5 couches |
| `decision-engine/main.py` | 41 | FastAPI server |
| `decision-engine/router.py` | 37 | Intent → task classifier |
| `decision-engine/scheduler.py` | 24 | APScheduler cron |
| `decision-engine/executor.py` | 39 | task() spawn + retry + approval |
| `decision-engine/observer.py` | 52 | Outcome tracking + drift scoring |
| `decision-engine/tasks/*.py` | 8 fichiers | WF1-WF7 replacements |
| `decision-engine/Dockerfile` | 7 | python:3.12-slim |
| `decision-engine/requirements.txt` | 7 | 6 packages |
| `dashboard/index.html` | 201 | Monitoring dashboard |
| `scripts/verify-mcp.ps1` | 96 | MCP health verification |
| `acontext/README.md` | 54 | Acontext setup guide |
| `rag/BYOX-README.md` | 46 | RAG corpus guide |
| `sandbox/testengineer/fixtures.js` | 31 | Faker.js generation |
| `loop-canonique.md` | 95 | Enriched: Trinité + vibecode |
| `permission-matrix.md` | 52 | Phase-locked capabilities |
| `.planning/phases/10-*/` | 7 fichiers | CONTEXT + 3 PLANs + 3 SUMMARYs |

### Architecture finale (Post-Phase 10)

```
Couche 5 — AUTOMATION       Decision Engine (:8001)  🔧 notre code
Couche 4 — OBSERVABILITE    CodeBurn 🟡P1 enabled
Couche 3 — DESIGN           Design Extract + Open Design 🟡P2 enabled
Couche 2 — EXECUTION        Scrapling🔴 Serena🔴 Supabase🔴 | Acontext🟡 Faker.js🟡
Couche 1 — CONNAISSANCE     CBM🔴 Graphify🔴 Obsidian🔴 (Trinité)
```

---

## 14. SOURCES & RÉFÉRENCES

### Sources primaires (GitHub — vérifiées 12/06/2026)

| Outil | URL |
|-------|-----|
| **OpenAgentsControl (OAC)** | `https://github.com/darrenhinde/OpenAgentsControl` |
| Scrapling | `https://github.com/D4Vinci/Scrapling` |
| Crawl4AI | `https://github.com/unclecode/crawl4ai` |
| Serena | `https://github.com/oraios/serena` |
| Oh My OpenAgent | `https://github.com/code-yeongyu/oh-my-openagent` |
| agents-best-practices | `https://github.com/DenisSergeevitch/agents-best-practices` |
| vibecode-pro-max-kit | `https://github.com/withkynam/vibecode-pro-max-kit` |
| HolyClaude | `https://github.com/CoderLuii/HolyClaude` |
| Browser-Harness | `https://github.com/browser-use/browser-harness` |
| Faker.js | `https://github.com/faker-js/faker` |
| Supabase MCP | `https://github.com/supabase/mcp` |
| Design Extract | `https://github.com/Manavarya09/design-extract` |
| Acontext | `https://github.com/memodb-io/Acontext` |
| Odysseus | `https://github.com/pewdiepie-archdaemon/odysseus` |
| SurfSense | `https://github.com/MODSetter/SurfSense` |

### Outils non trouvés initialement — corrigés (juin 2026)

| Outil | Recherche initiale | URL corrigée | Résultat |
|-------|-------------------|-------------|----------|
| Odysseus | `github.com/styrant/odysseus` → 404 | `github.com/pewdiepie-archdaemon/odysseus` | **68.4k⭐**, 1,150 commits, AGPL-3.0 |
| Acontext | `github.com/Acontext-io/acontext` → 404 | `github.com/memodb-io/Acontext` | **3.5k⭐**, 1,080 commits, 279 releases, Apache 2.0 |
| SurfSense | `github.com/Surf-Sense/SurfSense` → profil personnel | `github.com/MODSetter/SurfSense` | **14.4k⭐**, 6,600 commits, 21 releases, Apache 2.0 |

### Documents sources internes

| Document | Contenu |
|----------|---------|
| `ANALYSE-OUTILS-TIERS.md` | Batch 1-2-3 (34 outils) |
| `ANALYSE-MAITRE.md` | Architecture 5 couches AgentOS |
| `ANALYSE-OUTILS-BATCH3.md` | Batch 3 (13 outils, juin 2026) |
| `ANALYSE-OUTILS-INTEGRES.md` | État d'intégration (21 outils actifs) |
| `RECHERCHE-OUTILS-MAITRE.md` | Grille 7 axes /35 + scores |

### Méthodologie

- **Recherche live** : GitHub, docs officielles, webfetch
- **Vérification** : Stats (stars, commits, releases) vérifiées en direct le 12/06/2026
- **Critères** : Token efficiency, complexité, pertinence, valeur ajoutée, maturité
- **Note discordance** : Certains outils (Odysseus, AContext, SurfSense) cités dans les analyses internes n'ont pas pu être vérifiés sur GitHub — ils peuvent avoir changé de nom, été déplacés, ou supprimés

---

> **Prochaine étape recommandée** : Intégrer Serena (P0) + Faker.js (P1) + absorber les patterns vibecode. Clarifier le statut d'Odysseus/AContext/SurfSense.
