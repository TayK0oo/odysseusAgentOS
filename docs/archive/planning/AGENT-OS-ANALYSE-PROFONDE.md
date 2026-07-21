# AGENT OS — ANALYSE PROFONDE OUTIL PAR OUTIL

> Objectif : disséquer chaque outil (fonctionnement réel + critères fixes), pour construire le meilleur système possible sur base OpenCode.
> Méthode : recherche live (GitHub, docs, deepwiki) à chaque fiche. Stats datées du 17/06/2026.
> Document vivant — construit par batches. Style direct.

## GRILLE FIXE (chaque fiche)

1. **Quoi** — en 1 ligne.
2. **Fonctionnement** — la mécanique interne réelle.
3. **Stats / maturité** — stars, licence, version, activité (vérifié live).
4. **Intégration** — MCP / CLI / Docker / npm-pip / coût tokens / deps.
5. **Force unique** — le pattern qu'on vole.
6. **Limites / risques**.
7. **Décision AgentOS** — intégrer / absorber / ignorer + OÙ précisément.

## PLAN DE BATCHES (60 outils)

| Batch | Contenu | Statut |
|---|---|---|
| **1** | Socle critique (13) : OpenCode, OAC, Trinité CBM/Graphify/Obsidian, Scrapling, Serena, Supabase MCP, OmO, Acontext, autoresearch, Odysseus | ✅ ci-dessous |
| 2 | Exécution+Design+Observ.+Automation (11) : Faker, Kroki, API Toolkit, Crawl4AI, Browser-Harness, Playwright, Design Extract, Open Design, CodeBurn, Decision Engine/n8n, Paperclip | ✅ ci-dessous |
| 3 | Patterns+Mémoire+Canaux (9) : agents-best-practices, HexStrike, HolyClaude, vibecode, Caveman, Obsidian Second Brain, OpenWA, Chatwoot, Build Your Own X | ✅ ci-dessous |
| 4 | Veille (6) + Banque (10) + Ignorés (14) | ✅ ci-dessous |
| 5 | Synthèse système final (mise à jour archi complète, couche Governance incluse) | ✅ ci-dessous |

---

# BATCH 1 — SOCLE CRITIQUE

## 1.0 OpenCode — `anomalyco/opencode` (ex sst/opencode)

1. **Quoi** — Agent de code open-source pour le terminal, provider-agnostic, architecture client/serveur.
2. **Fonctionnement** — Cœur en Go + TS. Un **serveur** héberge la logique d'agent (sessions, tools, LSP, permissions) ; le **TUI** n'est qu'un client parmi d'autres (desktop, mobile, web possibles) via API. Agents définis en **Markdown** dans `.opencode/agents/`. Deux agents natifs : `build` (accès complet) et `plan` (read-only). Sessions persistées (SQLite). LSP intégré pour l'intelligence code. Skills + MCP servers branchables par config.
3. **Stats / maturité** — ~160k⭐, MIT, archivé sous sst→repris par `anomalyco/opencode`, ~13k commits, 900 contributeurs, 7,5M devs/mois. Très mature.
4. **Intégration** — `curl -fsSL https://opencode.ai/install | bash` ou `npm i -g opencode-ai`. Desktop app dispo. Config `.opencode/config.json`. **C'est la base, pas un add-on.**
5. **Force unique** — La **séparation client/serveur** : tu peux mettre n'importe quelle UI devant (c'est ce qu'a fait Odysseus). Provider-agnostic natif = ton routing multi-modèle est déjà permis par le design.
6. **Limites / risques** — "Opinionated" : toute feature de fond passe par leur design process. Tu construis *autour*, pas *dedans*. Le repo a déménagé (sst→anomalyco) → vérifier les forks/plugins ciblant l'ancien chemin.
7. **Décision** — **BASE.** Tout le reste est greffé dessus comme MCP, skill, agent Markdown, ou client.

## 1.1 OAC — `darrenhinde/OpenAgentsControl`

1. **Quoi** — Framework d'agents *plan-first* avec gates d'approbation, conçu pour OpenCode.
2. **Fonctionnement** — Cycle : (1) tu ajoutes ton **contexte** une fois → (2) **ContextScout** découvre les patterns pertinents → (3) l'agent charge TES standards → (4) propose un **plan** → (5) **tu approuves** → (6) implémente en matchant ton projet → (7) ship sans refacto. 6 stages + 7 subagents (task-manager, context-scout, context-manager, coder-agent, test-engineer, code-reviewer, external-scout). Agents = Markdown éditables. Principe **MVI** : ne charger que le nécessaire (8000→750 tokens).
3. **Stats / maturité** — ~3.8-4k⭐, MIT, v0.7.1 (jan 2026), ~215 commits, statut BETA actif. Plugin OpenCode + Claude Code.
4. **Intégration** — `/plugin marketplace add darrenhinde/OpenAgentsControl` puis `/plugin install oac`, ou `install.sh`. Découverte contexte flexible (`.oac`, `.claude/context`, `.opencode/context`). Contexte depuis GitHub/worktrees/fichiers/URLs.
5. **Force unique** — **MVI + Approval Gates + Context-Aware**. L'agent génère du code qui matche ton projet *dès la 1re fois* car il charge tes patterns avant. C'est la discipline anti-gaspillage tokens.
6. **Limites / risques** — Jeune (BETA). 7 subagents = potentielle lourdeur ; ne garder que les utiles. Le "ContextScout" suppose des patterns bien rangés au départ.
7. **Décision** — **ABSORBER la philosophie** (loop plan-first, MVI, gates) dans ta `loop-canonique`. Ne pas forker en entier ; piocher les agents Markdown utiles.

## 1.2 Trinité — Couche Connaissance (interrogée ENSEMBLE)

> Règle : CBM (structure) → Graphify (sémantique) → Obsidian (mémoire) → fusionner. **Checkpoint obligatoire avant toute génération.** Les 3 sont ton stack interne (pas de repo public vérifiable).

### 1.2a CBM — codebase-memory-mcp
1. **Quoi** — Graphe de code always-on, exposé en MCP.
2. **Fonctionnement** — Indexe le code en nodes/edges (868 nodes / 932 edges sur ton projet) via tree-sitter (66+ langages). Requêtes Cypher + full-text + `trace_path`. 14 outils (search_graph, query_graph…). Tourne en permanence (serveur MCP Go).
3. **Stats** — interne, non public. SQLite.
4. **Intégration** — MCP always-on. 100-200 tokens/query. Web UI localhost:9749.
5. **Force unique** — **Navigation structurelle déterministe** : call tracing, dead code, impact analysis — sans LLM, donc fiable et cheap.
6. **Limites** — Code uniquement. Pas d'inférence sémantique, pas de clustering, pas de cross-artefact (docs).
7. **Décision** — **GARDER (Couche 1 always-on).** C'est ta première question sur tout problème de code.

### 1.2b Graphify — `graphifyy` (PyPI)
1. **Quoi** — Knowledge graph multi-input on-demand.
2. **Fonctionnement** — Ingère code (AST), docs, PDF, images (vision), vidéos (Whisper), URLs. **Double extraction** : AST déterministe (gratuit) + sémantique (subagents Claude). Community detection Leiden. Sorties : HTML D3, SVG, JSON, GraphML, Neo4j, vault Obsidian, serveur MCP. Sur ton projet : 2581 nodes / 3698 edges / 293 communautés, compression 8.1x.
3. **Stats** — interne/PyPI v0.4.3.
4. **Intégration** — `/graphify .` on-demand. 500 tokens/query (vs 50k lecture brute).
5. **Force unique** — **Liens implicites code↔docs + clustering sémantique**. Comprend des concepts, pas juste des appels.
6. **Limites** — On-demand (pas temps réel). La partie sémantique coûte des appels LLM.
7. **Décision** — **GARDER (Couche 1 on-demand).** Onboarding projet, archi globale.

### 1.2c Obsidian (lecture directe)
1. **Quoi** — Mémoire persistante humaine-lisible.
2. **Fonctionnement** — Lecture directe de markdown (pas de MCP). Vault `/agentos/{type}/`, 7 templates wf*.md.
3. **Stats** — interne.
4. **Intégration** — Lecture fichiers. 200-500 tokens/query.
5. **Force unique** — **Mémoire décisionnelle** éditable à la main, versionnable Git.
6. **Limites** — Pas de recherche structurée (lecture simple).
7. **Décision** — **GARDER (Couche 1 mémoire).** Reçoit aussi les sorties d'Acontext (voir 1.9).

## 1.3 Scrapling — `D4Vinci/Scrapling`

1. **Quoi** — Framework de web scraping adaptatif, de la requête unique au crawl massif.
2. **Fonctionnement** — 3 fetchers : `Fetcher` (HTTP + impersonation TLS/HTTP3), `StealthyFetcher` (anti-bot, Cloudflare Turnstile natif), `DynamicFetcher` (headless). **Adaptive scraping** : le parser apprend la structure du site et **re-localise automatiquement** tes éléments quand le site change (`relocate`, `find_similar`, auto_save). Spider async : concurrence configurable, throttling par domaine, **pause/resume checkpoint** (Ctrl+C → reprise), **streaming** (`async for item in spider.stream()`), hooks de cycle de vie (on_start/on_close/on_error). Parser ~2ms (1775x BeautifulSoup).
3. **Stats / maturité** — **63.1k⭐ (vérifié)**, BSD-3, 47 releases, v0.4.9 (juin 2026), 92% test coverage. Très actif.
4. **Intégration** — `pip install scrapling[all]` + `scrapling install`. **MCP natif**. **agent-skill prêt** (Claude Code/OpenCode). CLI `scrapling extract` + shell IPython (curl→scrapling). Docker avec browsers. Extraction ciblée pré-LLM (−80-90% tokens).
5. **Force unique** — **L'adaptive scraping** : tes scrapers survivent aux changements de site. Personne d'autre ne fait ça aussi proprement. + pause/resume + streaming = robustesse production.
6. **Limites / risques** — Surface large (beaucoup de modes) → discipline de config. Scraping = zone légale/éthique (respecter robots, ToS).
7. **Décision** — **INTÉGRER (Couche 2, MCP). Brique web unique.** Remplace Crawl4AI + Browser-Harness (gardés en fallback OFF).

## 1.4 Serena — `oraios/serena`

1. **Quoi** — Toolkit MCP d'édition/retrieval sémantique au niveau symbole — "l'IDE de ton agent".
2. **Fonctionnement** — Serveur MCP backé par **LSP** (gopls, rust-analyzer…) via une abstraction maison `solidlsp`/`SolidLanguageServer` (30-40+ langages). Outils symboliques : `find_symbol`, `find_referencing_symbols`, `replace_symbol_body`, `insert_before/after_symbol`, `rename_symbol` (**reference-aware** : 20 usages → 20 mis à jour par le LSP). **Tool gating natif** : chaque tool porte un `ToolMarker` (ToolMarkerSymbolicRead, ToolMarkerCanEdit) ; le `ToolRegistry` filtre les tools selon le `SerenaAgentMode`/`context` (ide-assistant, desktop-app). **Mémoire** : à l'onboarding, analyse le projet → écrit des fichiers Markdown dans `.serena/memories/` ; sessions suivantes lisent sélectivement. `project index` pré-remplit le cache de symboles.
3. **Stats / maturité** — **25.3k⭐**, MIT, v1.5.x (2026), ~2900 commits. Très actif.
4. **Intégration** — `uvx --from git+...oraios/serena serena start-mcp-server --context ide-assistant --project $(pwd)`. Docker (`compose.yaml`, transport SSE). **LSP local = 0 token** (mais le LLM client paie ses propres tokens). ~30MB.
5. **Force unique** — **Édition chirurgicale reference-aware** (vs édition ligne-par-ligne fragile) + **tool-gating par mode** = exactement ton "phase-locked tools" mais natif.
6. **Limites / risques** — JetBrains backend = payant (refactor avancé). LSP par langage = deps runtime à installer. Onboarding initial coûte des tokens LLM.
7. **Décision** — **INTÉGRER P0 (Couche 2).** WF Code/Debug/Review. **Bonus** : son `ToolMarker`/mode est le modèle de référence pour ton système de permissions par phase (§ archi sandbox).

## 1.5 Supabase MCP — `supabase/mcp`

1. **Quoi** — Serveur MCP officiel Supabase = DB d'exécution + state.
2. **Fonctionnement** — Expose 22+ outils : SQL, migrations, auth, storage, edge functions, **branching** (DB éphémères par feature). Contrôle via query params : `?read_only=true` (sécurité), `?features=database,docs` (réduire la surface), `?project_ref=<id>` (scoping).
3. **Stats / maturité** — ~2.7k⭐, officiel, ~389 commits, 38 releases. SDK `@supabase/mcp-server-supabase` + Vercel AI SDK.
4. **Intégration** — MCP. Mode read-only recommandé par défaut.
5. **Force unique** — **Read-only + feature-groups + project-scoping** = surface minimale et sûre exposée à l'agent. Branching = sandbox DB par feature.
6. **Limites / risques** — Couplage Supabase. Donner write à un agent sur une vraie DB = risque → read-only par défaut, branching pour les tests.
7. **Décision** — **INTÉGRER (Couche 2).** Persistance projets + **state des sessions agents** (continuité après fallback modèle).

## 1.6 OmO — `code-yeongyu/oh-my-openagent`

1. **Quoi** — Harness d'agents multi-modèles pour OpenCode/Codex, optimisé "tokenmaxxing".
2. **Fonctionnement** — **Routing par agent** : chaque agent a un `model` natif + une **fallback chain** ordonnée, hardcodée par agent (`model-requirements.ts`) — pas de liste globale. **runtime_fallback** : sur erreur (`[400,429,503,529]`), retry → modèle suivant, `cooldown_seconds`, `max_fallback_attempts`, blacklist temporaire. **Catégories** (quick/deep/utility/vision) → pools. 11 agents (Sisyphus, Hephaestus, Oracle, Atlas, Momus…). **Exécution parallèle** (research+impl+verif simultanés). **Hash-anchored edits** (LINE#ID) : valide chaque edit avant application (Grok 6.7%→68.3% succès). **IntentGate** : filtre l'intention avant de lancer. `ulw-loop` (ultrawork durable goal execution). Multi-harness (OpenCode + Codex via LazyCodex).
3. **Stats / maturité** — **~42k⭐** (le doc disait 61.9k → gonflé), très actif (8500+ commits, ~200 releases), v4.9.x.
4. **Intégration** — `bunx oh-my-opencode install` (interactif ou `--no-tui`). Config `~/.config/opencode/oh-my-openagent.json`. Schema JSON fourni.
5. **Force unique** — **Le système de routing/fallback le plus abouti de l'écosystème OpenCode.** C'est la référence directe pour ton "routing intelligent configurable". + hash-anchored edits + IntentGate.
6. **Limites / risques** — Chains hardcodées en TS → moins déclaratif que ce que tu veux (tu veux JSON configurable). Marketing agressif ("Anthropic a bloqué OpenCode à cause de nous") → ignorer le narratif, prendre la technique. Lourd si tout installé.
7. **Décision** — **ABSORBER les patterns** (matrice agent→modèle, runtime_fallback, catégories, IntentGate, hash-edits) dans ton **Model Router** déclaratif (JSON) branché sur LiteLLM. Ne pas dépendre du repo entier. Note : ton "Sisyphus" descend de leur agent Sisyphus.

## 1.7 Acontext — `memodb-io/Acontext`

1. **Quoi** — "Agent Skills as a Memory Layer" : transforme ce que l'agent a *fait* en skills Markdown réutilisables.
2. **Fonctionnement** — Flow : `Session messages → task complete/failed → Distillation → Skill Agent → Update Skills`. Apprend des **résultats de tâche** (ce qui a marché/échoué → SOPs + warnings), pas juste du chat. Mémoire = **fichiers Markdown (schéma SKILL.md)**, **pas d'embeddings**. Récupération = **progressive disclosure** par tool use (`get_skill`, `get_skill_file`) + reasoning, **pas de top-k sémantique**. Toi tu définis le schéma (1 fichier/contact, 1/projet…). Workflow **"wait-for-user-confirmation"** : valider les patterns appris avant sauvegarde. Stockage Postgres (session/files/artifacts/sandboxes).
3. **Stats / maturité** — ~3.5k⭐, Apache-2.0, 1080 commits, **279 releases** (très itératif). SDK Python + TS. Plugins Claude Code + OpenClaw.
4. **Intégration** — Self-host `curl -fsSL https://install.acontext.io | sh` (Docker : Postgres+Redis+RabbitMQ+S3) ou cloud. Dashboard localhost:3000. Marche avec Claude/OpenAI/LangGraph/Agno/Vercel AI SDK.
5. **Force unique** — **Distillation des outcomes en skills lisibles/portables**, sans vector DB opaque. Boucle d'apprentissage continue. Skills = mêmes fichiers que tes skills OpenCode → **pas de format à part**.
6. **Limites / risques** — **Stack lourd** (Postgres+Redis+RabbitMQ+S3) pour ce que ça fait. Évaluer une V1 allégée : distillation maison → écrit directement dans le vault Obsidian + skills `.opencode/skills/`.
7. **Décision** — **INTÉGRER (Couche mémoire)** — mais commencer par le **pattern** (distillation outcome→Markdown) avant d'installer tout le stack. La sortie alimente ton **second-brain Obsidian** (lien Batch 3).

## 1.8 autoresearch — `karpathy/autoresearch`

1. **Quoi** — Agent qui lance des expériences ML en boucle autonome (overnight) sur 1 GPU.
2. **Fonctionnement** — Le **pattern** (≈630 lignes) : tu fournis (a) un **fichier cible**, (b) un **`eval.sh` figé** (l'agent ne le modifie JAMAIS), (c) **1 métrique**. L'agent édite le code, lance, score, **garde si mieux / revert si pire**, relit les logs d'erreur, re-tente, boucle. Budget temps fixe (5 min/run) → résultats comparables. ~12 exp/h, ~100/nuit. Self-contained (PyTorch + qq packages).
3. **Stats / maturité** — **~59k⭐** (doc disait 83k → gonflé), MIT. Écosystème de forks (bilevel-autoresearch, autoresearch-automl…).
4. **Intégration** — Repo = spécifique nanochat/ML. **Ce qui t'intéresse = le pattern**, pas le code. Adaptable à *tout objectif mesurable* (cf. forks "domain-agnostic").
5. **Force unique** — **Boucle d'auto-optimisation bornée** : eval figé + métrique + keep/revert. Si tu peux lancer une commande et lire un nombre, ça marche. C'est le squelette exact de ton "loop avec objectif + limites par projet".
6. **Limites / risques** — Tel quel = mono-GPU/ML. Le revert suppose un versioning propre (git). Métrique unique = peut sur-optimiser un proxy.
7. **Décision** — **ABSORBER le pattern** dans ton **Project Manifest** (`eval_command` figé + `metric` + `max_iterations`/`token_budget`). Cœur de l'auto-évaluation par projet.

## 1.9 Odysseus — `pewdiepie-archdaemon/odysseus`

1. **Quoi** — Workspace AI self-hosted complet, **bâti sur OpenCode** (par Felix Kjellberg / PewDiePie).
2. **Fonctionnement** — Docker compose (UI sur :7000). Modules : **Chat+Agents** (local/API, MCP, files, shell, skills, memory) ; **Cookbook** (scan hardware → reco modèles VRAM-aware → download/serve, 270+ modèles, GGUF/FP8/AWQ) ; **Deep Research** (multi-step : gather→read→synthesize→report) ; **Compare** (A/B aveugle) ; **Documents** (éditeur multi-onglets MD/HTML/CSV) ; **Email** IMAP/SMTP + triage ; **Calendar** CalDAV ; **Notes/Tasks** cron + ntfy ; **Memory** ChromaDB + fastembed. **PWA** installable. Adapte du code OpenCode (crédité dans ACKNOWLEDGMENTS).
3. **Stats / maturité** — ~72k⭐, **MIT** (le doc disait AGPL → FAUX, donc forkable librement), ~1150 commits, branches dev/main.
4. **Intégration** — `git clone … && docker compose up -d --build`. Natif Linux/macOS(Apple Silicon)/Windows.
5. **Force unique** — **C'est déjà "OpenCode + UI workspace + PWA" prêt.** Le Cookbook VRAM-aware est parfait pour ton infra Ollama locale. Te fait gagner des mois sur l'UI.
6. **Limites / risques** — Gros codebase tiers (dette si fork). ChromaDB pour la mémoire (≠ ta Trinité) → à réconcilier. Sécurité : outils puissants (shell/files/email) → garder auth, ne pas exposer les ports.
7. **Décision** — **CANDIDAT BASE UI (reclassé de Veille).** Reco : **forker pour l'UI/workspace**, remplacer sa mémoire ChromaDB par ta Trinité, brancher ton Decision Engine + Model Router par-dessus. À trancher vs UI maison.

---

## NOTES TRANSVERSES — ce que le Batch 1 impose à l'archi

- **Tool-gating** : Serena prouve que filtrer les tools par mode est natif et fiable → adopter ce modèle (ToolMarker) pour TES permissions par phase, au lieu de réinventer.
- **Mémoire = Markdown partout** : Serena (`.serena/memories/`), Acontext (SKILL.md), Obsidian, skills OpenCode → **un seul format**, versionnable Git, pas d'embeddings opaques. Cohérence forte. ChromaDB d'Odysseus est l'exception à neutraliser.
- **Loop bornée** : autoresearch (eval figé+métrique+keep/revert) + OAC (plan-first+gates) + OmO (ulw-loop durable) → fusionner en une loop canonique unique.
- **Routing** : OmO (chains+fallback) + LiteLLM (abstraction providers) + ton `.bat` toggle → un `model-routing.json` déclaratif.
- **UI** : Odysseus rend l'UI quasi "gratuite" — décision structurante n°1.

---

# BATCH 2 — EXÉCUTION / DESIGN / OBSERVABILITÉ / AUTOMATION

## 2.1 Faker.js — `faker-js/faker`

1. **Quoi** — Générateur de données factices réalistes.
2. **Fonctionnement** — Modules (Person, Location, Date, Finance, Commerce, Internet, Hacker, Number, String…) produisant des valeurs plausibles. **Seedable** (`faker.seed(123)`) → données déterministes/reproductibles, crucial pour des tests stables. 70+ locales.
3. **Stats / maturité** — 15.4k⭐, MIT, v10.x, ~4200 commits. Standard de l'écosystème JS.
4. **Intégration** — `npm i -D @faker-js/faker`. Pas de MCP (lib appelée par le code de test). 0 token.
5. **Force unique** — **Fixtures réalistes + seedables** → tests E2E/unitaires non flaky, démos crédibles.
6. **Limites** — JS/TS only (équivalents Python : Faker py). Données factices ≠ cas limites réels.
7. **Décision** — **INTÉGRER P1.** Sandbox TestEngineer (`sandbox/testengineer/fixtures.js`), étape BUILD de la loop (outil forcé avec Playwright).

## 2.2 Kroki

1. **Quoi** — Passerelle REST unifiée vers 25+ langages de diagrammes.
2. **Fonctionnement** — Tu POST du texte (PlantUML, Mermaid, GraphViz, C4, ERD, BPMN…) → Kroki rend SVG/PNG. Encodage deflate+base64 dans l'URL pour du GET. Self-host Docker (port 8000).
3. **Stats / maturité** — Projet établi, conteneurs officiels.
4. **Intégration** — Docker local, REST. **0 token** (rendu local).
5. **Force unique** — **Un seul endpoint pour tous les formats** → l'agent écrit du texte, Kroki dessine. Pas de dépendance par langage.
6. **Limites** — Rendu only (pas d'édition interactive). Image lourde si tous les moteurs.
7. **Décision** — **INTÉGRER (Couche 2).** Étapes Brainstorm/Plan (archi, flux, ERD). Complète Graphify (qui sort déjà du HTML/SVG).

## 2.3 API Toolkit (TMDB / Mapbox / OpenWeatherMap / News)

1. **Quoi** — Jeu d'APIs réelles documentées pour nourrir des projets full-stack de démo.
2. **Fonctionnement** — Fichiers de contexte (`api-toolkit/*.md`) décrivant endpoints/clés/quotas. TMDB (films/séries), Mapbox (50k maps/mois gratuit), OWM (météo), News API.
3. **Stats** — agrégat, pas un repo unique.
4. **Intégration** — Context files + clés API. Appels HTTP standards.
5. **Force unique** — **Dataset prêt** pour tester l'agent sur du vrai (cartes, médias, météo) sans monter un backend.
6. **Limites** — Quotas gratuits limités. Pas une brique structurelle, juste des données.
7. **Décision** — **GARDER (data sources démo).** Utile pour bench/QA d'agents full-stack. Faible priorité.

## 2.4 Crawl4AI — `unclecode/crawl4ai` (fallback)

1. **Quoi** — Crawler/scraper orienté LLM (markdown propre).
2. **Fonctionnement** — "Fit Markdown" + filtrage BM25 (extrait le contenu pertinent), deep crawl BFS/DFS, MCP natif, CLI `crwl`. Anti-bot 3 niveaux (basique).
3. **Stats / maturité** — ~68k⭐, Apache-2.0, v0.8.x. Très populaire.
4. **Intégration** — pip + Docker. MCP. Configuré `enabled:false` chez toi.
5. **Force unique** — **Fit Markdown + BM25** = sortie nickel pour RAG. Mais Scrapling couvre + (adaptive, anti-bot avancé, spider).
6. **Limites** — Pas d'adaptive scraping, anti-bot plus faible que Scrapling.
7. **Décision** — **GARDER FALLBACK (OFF).** Backup de Scrapling. N'activer que si Scrapling échoue sur un cas précis.

## 2.5 Browser-Harness — `browser-use/browser-harness` (fallback)

1. **Quoi** — Harness CDP minimal (~1k lignes, 4 fichiers core).
2. **Fonctionnement** — Pilotage direct via Chrome DevTools Protocol, accès à des cloud browsers. Pas de spider, pas de fetch HTTP — focus contrôle navigateur bas niveau.
3. **Stats / maturité** — ~14.7k⭐, MIT.
4. **Intégration** — Configuré `enabled:false` chez toi.
5. **Force unique** — **CDP direct + cloud browsers** pour les cas headless très récalcitrants.
6. **Limites** — Surface étroite. Redondant avec Scrapling (DynamicFetcher) + Playwright pour la plupart des cas.
7. **Décision** — **GARDER FALLBACK (OFF).** Dernier recours headless.

## 2.6 Playwright

1. **Quoi** — Automatisation navigateur multi-moteurs (Chromium/Firefox/WebKit).
2. **Fonctionnement** — API d'automatisation (clics, navigation, assertions, screenshots, traces), auto-wait, contextes isolés, headless/headed. Test runner intégré.
3. **Stats / maturité** — Microsoft, standard industriel, ultra mature.
4. **Intégration** — `npm i -D @playwright/test`. MCP Playwright dispo. Tourne dans le sandbox.
5. **Force unique** — **Tests E2E fiables + isolation contexte** = validation réelle dans la loop (étape QUALITY PIPELINE).
6. **Limites** — Browsers lourds (Docker Xvfb). Tests E2E = plus lents que unitaires.
7. **Décision** — **INTÉGRER (Couche 2, sandbox test).** Étape test de la loop. Sert aussi de moteur à Design Extract.

## 2.7 Design Extract (designlang) — `Manavarya09/design-extract`

1. **Quoi** — Extrait le **design system complet** de n'importe quelle URL, en une commande.
2. **Fonctionnement** — Pointe un **navigateur headless (Playwright)** sur l'URL → lit le design **calculé depuis le DOM rendu** (couleurs, spacing, typo, motion, shadows, anatomie, états hover/focus/active, responsive sur 4 breakpoints). Émet **17+ fichiers** : DTCG tokens (primitive/semantic/composite), Tailwind v4, shadcn theme, Figma vars, motion tokens, composants React typés, prompt pack (v0/Cursor/Claude). **CSS health audit + grade A-F + WCAG remediation**. Multi-plateforme (web/iOS SwiftUI/Android Compose/Flutter/WordPress). Verbes : `extract`, `grade`, `clone` (→ app Next.js), `apply` (écrit les tokens dans un projet), `drift` (bot CI anti-dérive de tokens), `remix` (6 vocabulaires), `theme-swap` (recolor OKLCH), `brand` (book 13 chapitres), `pair`, `battle`, `mcp`.
3. **Stats / maturité** — ~3.2k⭐, MIT, v12.x, Node 20+, Playwright. Très actif.
4. **Intégration** — `npm i -g designlang` ou `npx designlang <url>`. **MCP server** (`designlang mcp`) + plugin Claude Code (slash commands). Agent skill (`npx skills add Manavarya09/design-extract`). `--smart` route vers un LLM OpenAI-compatible si besoin.
5. **Force unique** — **Capture un design depuis une URL vivante** (vs Figma/tokens manuels) → tokens prêts à coder. `drift` garde ton projet aligné en CI.
6. **Limites** — Lecture DOM = qualité variable selon le site (confidence affichée). Sortie = point de départ, pas un design system final.
7. **Décision** — **INTÉGRER (Couche 3, ON).** Très pertinent pour tes restylings (360 POD, glassmorphism). Pair parfait avec Open Design (2.8) : Extract *aspire* un design, Open Design *en génère* un.

## 2.8 Open Design — `nexu-io/open-design`

1. **Quoi** — Alternative open-source à Claude Design, **tournant SUR ton agent** (OpenCode inclus).
2. **Fonctionnement** — App desktop locale + daemon. Détecte ton CLI agent sur le PATH (Claude Code/OpenCode/Codex/Cursor…) et l'utilise comme **moteur de design**. Boucle agent-native : *discover brief → lock direction → stream artifact → critique (Critique Theater) → deliver*. 259+ skills (SKILL.md) + 142+ design systems (DESIGN.md, importés d'awesome-design-md). Génère web/desktop/mobile prototypes, dashboards, decks, images, vidéos, HyperFrames. Preview iframe sandboxée, export HTML/PDF/PPTX/MP4. **MCP bidirectionnel** : ship son propre MCP server ET consomme des MCP externes (OAuth managé). BYOK partout.
3. **Stats / maturité** — nexu-io, Apache-2.0/MIT, v0.9.x, des milliers de PRs (le doc le sous-comptait : 259 skills vs 31, 142 systems vs 129). Très actif.
4. **Intégration** — `curl -fsSL https://open-design.ai/install.sh | sh -s opencode` (s'installe DANS l'agent), ou app desktop, ou MCP server. Utilisable sans GUI.
5. **Force unique** — **Le layer design natif d'OpenCode**, avec design systems en fichiers Markdown (cohérent avec ta stratégie "tout en Markdown"). Critique Theater = self-review design intégrée.
6. **Limites** — Surface énorme (peut faire doublon avec Design Extract sur certains verbes). Desktop app = poids ; mais mode MCP/skill suffit.
7. **Décision** — **INTÉGRER (Couche 3, ON).** Moteur de génération design. + ses **DESIGN.md** alimentent ta banque de design systems. Complémentaire de Design Extract.

## 2.9 CodeBurn — `getagentseal/codeburn`

1. **Quoi** — Observabilité tokens/coût/perf des agents de code, 100% local.
2. **Fonctionnement** — Lit les **logs de session sur disque** (JSONL/SQLite) — pas de proxy, pas de clé API. Classifier déterministe **13 catégories** (Coding/Debug/Refactor/Test…) sans LLM. **One-shot rate** (% d'edits réussis sans retry). `codeburn optimize` scanne ~11 **patterns de gaspillage** (fichiers relus en boucle, ratio read:edit faible, bash output non capé, **MCP servers inutilisés payant leur schema**, CLAUDE.md gonflé) → **fixes copier-coller + estimation tokens/$ économisés**. Grade A-F, ghost agents, corrèle dépense ↔ commits git (productif/reverté/abandonné). Prix via LiteLLM (cache 24h). Export CSV/JSON.
3. **Stats / maturité** — AgentSeal, MIT, Node 20+. TUI Ink. Supporte Claude Code/Codex/Cursor/**OpenCode**/Pi/Copilot.
4. **Intégration** — `npm i -g codeburn`. 0 token (lecture disque). Jumeau **AgentSeal** (`pip install agentseal`) = sécurité agents (300+ probes injection, MCP empoisonnés, skills malveillants).
5. **Force unique** — **One-shot rate + corrélation commits** = signaux parfaits à injecter dans ton drift score et ta loop auto-éval. Détecte les MCP inutiles qui coûtent des tokens à vide.
6. **Limites** — Lecture a posteriori (pas temps réel). Copilot = tracking partiel.
7. **Décision** — **INTÉGRER (Couche 4, ON).** Boucler `one-shot rate` + waste patterns dans l'observer. **Bonus : ajouter AgentSeal** à la couche sécurité/sandbox.

## 2.10 Decision Engine (ton code) / n8n MCP (retiré)

1. **Quoi** — Cerveau d'orchestration Python (remplace n8n).
2. **Fonctionnement** — FastAPI (:8001). `router.py` (intent→task), `scheduler.py` (APScheduler cron), `executor.py` (spawn task() + retry + approval), `observer.py` (outcome + drift). ~300 lignes vs n8n ~500MB RAM. WF1-7 = scripts Python.
3. **Stats** — interne (Phase 10).
4. **Intégration** — Docker (python:3.12-slim), 6 packages. Dashboard → REST.
5. **Force unique** — **Léger, déterministe, dans TON langage** — pas de boîte noire no-code. Tu contrôles l'arbre de décision.
6. **Limites** — À enrichir : decision-tree déclaratif (YAML), forced_tools vérifiés, budgets (← Paperclip). Pas d'intégrations SaaS clé-en-main (ce que n8n offrait).
7. **Décision** — **GARDER & ÉTENDRE (Couche 5).** Y greffer : decision-tree.yaml, Model Router, budgets Paperclip, heartbeat scheduling. **n8n = ignorer** sauf besoin ponctuel d'intégration SaaS no-code.

## 2.11 Paperclip — `agencyenterprise/paperclip-ai`

1. **Quoi** — Control plane organisationnel pour équipes d'agents ("zero-human company"). La couche AU-DESSUS des runtimes.
2. **Fonctionnement** — Node.js server + React UI. **Org chart** (CEO→CTO→Dev→QA, rôles, reporting lines). **Budgets par agent** : à 100% d'utilisation → **auto-pause + blocage des nouvelles tâches** (anti-runaway). **Task checkout + budget enforcement atomiques** (pas de double-work). **Governance board-of-directors** : un agent ne peut pas en recruter un autre / exécuter une stratégie majeure sans approbation humaine ; **config versionnée + rollback** (baked-in, non désactivable). **Heartbeat scheduling** : l'agent se réveille, check sa queue, agit ; **état persistant entre heartbeats** (reprend le contexte, ne repart pas de zéro). **Goal-ancestry** : chaque tâche porte toute sa lignée d'objectifs (le "pourquoi"). **Multi-company isolé**. **BYOA** : tout runtime recevant un heartbeat (Claude Code, OpenCode, Codex, Python, shell, webhook) via adapters. Templates de société export/import (secret scrubbing).
3. **Stats / maturité** — ~42k⭐, open-source, créé par @dotta (mars 2026), croissance explosive. Self-host.
4. **Intégration** — Node server + React UI, self-host. Adapters vers runtimes. SKILLS.md pour injection contexte runtime.
5. **Force unique** — **C'est la couche Governance qui te manque** : budgets par agent avec auto-pause (= tes "limites par projet"), goal-ancestry (= tes "objectifs clairs"), approval gates + rollback, heartbeat cron. Tout ce que ton Decision Engine doit faire côté gouvernance, Paperclip le formalise déjà.
6. **Limites / risques** — Modélise des "entreprises" (peut être over-kill si tu veux juste orchestrer des projets). React UI = peut chevaucher Odysseus. Sécurité : empiler de l'autonomie sur OpenClaw/agents = surface d'attaque (cf. AgentSeal/sandbox).
7. **Décision** — **ABSORBER les patterns dans le Decision Engine** (budgets auto-pause, goal-ancestry, approval+rollback, heartbeat) plutôt que forker l'UI entière (Odysseus tient l'UI). OU forker si tu veux le control-plane multi-projets prêt. **Nouvelle Couche transverse : GOVERNANCE.**

---

## NOTES TRANSVERSES — apports du Batch 2

- **Nouvelle couche : GOVERNANCE** (Paperclip) — transverse, au-dessus de l'exécution : budgets/agent + auto-pause, goal-ancestry, approval+rollback, heartbeat. Répond direct à tes "objectifs + limites par projet".
- **Design = paire Extract↔Generate** : Design Extract aspire un design system depuis une URL (tokens) ; Open Design en génère des artefacts. Les deux parlent Markdown (DTCG / DESIGN.md) → cohérent avec "tout en Markdown".
- **Observabilité actionnable** : CodeBurn fournit one-shot rate + waste patterns + corrélation commits → signaux directs pour l'observer/drift score. Détecte aussi les **MCP inutiles** (utile vu ta modularité : désactiver ce qui coûte à vide).
- **Sécurité** : AgentSeal (jumeau CodeBurn) entre dans la couche sandbox aux côtés de HexStrike (Batch 3).
- **Web = Scrapling seul en première ligne** ; Crawl4AI + Browser-Harness restent OFF en secours. Pas de redondance active.
- **n8n définitivement écarté** au profit du Decision Engine, sauf besoin SaaS no-code ponctuel.

---

# BATCH 3 — PATTERNS / MÉMOIRE / CANAUX

## 3.1 agents-best-practices — `DenisSergeevitch/agents-best-practices`

1. **Quoi** — Agent Skill provider-neutral = la **charte d'ingénierie de harness**.
2. **Fonctionnement** — Définit le **harness** comme la couche déterministe qui enveloppe le LLM : *le modèle propose, le harness valide/autorise/exécute/loggue*. **Loop canonique** : `instructions → context builder → model call → tool proposal → validation → permission decision → execution OU approval pause → observation → next step / final`. **Séquence de construction** (dans cet ordre, n'ajouter la suite qu'après fiabilité du socle) : `loop manuel → tools → permissions → observations structurées → budgets → tracing → planning → context/memory → compaction → skills/connectors → goal loop → subagents`. Skill = SKILL.md + 15 références (mvp-blueprint, architecture, coding-agents, security-evals-observability…).
3. **Stats / maturité** — ~1.9k⭐, MIT, format Agent Skill (Codex/Claude/OpenCode).
4. **Intégration** — `npx skills add DenisSergeevitch/agents-best-practices -g`. C'est de la connaissance, pas un runtime.
5. **Force unique** — **10 principes non-négociables** : (1) le risque change la loop (read/draft/write/comm/financier/destructif/privilégié = chemins de permission différents) ; (2) draft ≠ commit (effets à haut risque → trace d'approbation hors prompt) ; (3) le contexte se construit, ne se déverse pas (retrieve juste assez, étiqueter les trust boundaries, préserver l'état à travers la compaction) ; (4) le travail long a des **budgets** (step/time/token/cost/tool-call) ; (5) progressive disclosure des skills/connectors ; (6) les **échecs répétés deviennent des features** du harness (validators/tools/docs/evals/policies) ; (7) la plupart des échecs ≠ manque d'autonomie ; (8) le plan de workflow **n'est pas une policy de confiance** (passe les mêmes gates) ; (9) **évaluer le harness, pas que le modèle** ; (10) **humans ON the loop** (maintiennent le harness) > humans IN the loop (revoient chaque output). Schéma de trace riche (run_id, permission decisions, compaction boundaries, cost…).
6. **Limites** — C'est un guide, pas du code. À traduire en implémentation.
7. **Décision** — **ABSORBER — c'est la CONSTITUTION d'Agent OS.** Ta `loop-canonique.md` et ta séquence de build dérivent directement de ce doc. Tout le reste (Paperclip budgets, Serena permissions, autoresearch failures→features, CodeBurn eval-the-harness) en est une instanciation.

## 3.2 HexStrike AI — `0x4m4/hexstrike-ai`

1. **Quoi** — Framework MCP de **sécurité offensive** (pentest autonome).
2. **Fonctionnement** — Serveur MCP exposant **150+ outils de sécurité** (nmap, sqlmap, ffuf, nuclei, hashcat, ghidra, radare2…) à des agents, + 12 agents spécialisés (TechnologyDetector, IntelligentDecisionEngine, BugBountyWorkflowManager, CVEIntelligenceManager, AIExploitGenerator…). **Garde-fous défensifs** : validation de commandes (anti command-injection), scope validation, tool whitelist, rate limiting, audit logging, safe mode (non-destructif), API key optionnelle.
3. **Stats / maturité** — ~9k⭐, plusieurs forks. Usage = "authorized testing only".
4. **Intégration** — MCP stdio (`hexstrike_mcp.py`). Requiert l'arsenal de pentest installé.
5. **Force unique (pour nous)** — **L'échafaudage de contrôle**, pas l'arsenal : command validation, scope validation, whitelist, rate limit, audit log, safe mode. C'est un modèle de sandbox MCP discipliné.
6. **Limites / risques** — **C'est de l'offensif.** Intégrer les 150 outils = inutile pour ton usage (dev) et dangereux. Surface légale/éthique (autorisation écrite obligatoire).
7. **Décision** — **ABSORBER UNIQUEMENT les patterns de contrôle** (validation commandes + scope + whitelist + audit + safe mode) dans ta couche sandbox, aux côtés d'AgentSeal. **NE PAS** intégrer l'arsenal offensif.

## 3.3 HolyClaude — `CoderLuii/HolyClaude`

1. **Quoi** — Collection de patterns d'infrastructure Docker pour agents.
2. **Fonctionnement** — Briques réutilisables : **s6-overlay** (PID 1, supervision de process, graceful shutdown), **Xvfb + Chromium** (navigateur headless en conteneur), **Apprise** (notifications vers 100+ services), bind-mount persistence (volumes nommés).
3. **Stats / maturité** — ~2.3k⭐, MIT, ~20 commits (petit mais ciblé).
4. **Intégration** — Patterns à copier dans tes Dockerfiles (`sandbox/shared/Dockerfile.base`, `sandbox/testengineer/Dockerfile`, `docker-compose.yml`).
5. **Force unique** — **s6-overlay** (process management propre dans un conteneur) + **Apprise** (1 lib pour toutes les notifs) = robustesse opérationnelle.
6. **Limites** — Pas un produit, juste des recettes Docker.
7. **Décision** — **ABSORBER (Docker du sandbox).** s6-overlay pour les conteneurs longue durée, Xvfb+Chromium pour le TestEngineer, Apprise pour les notifs (alternative : ntfy d'Odysseus).

## 3.4 vibecode-pro-max-kit — `withkynam/vibecode-pro-max-kit`

1. **Quoi** — Harness de code **spec-driven** (méthodologie RIPER-5) avec mémoire auto-améliorante.
2. **Fonctionnement** — **RIPER-5** = Research → Innovate → Plan → Execute → Review, chaque phase gated. **Phase-locked tool restrictions** (vrai retrait de capacité, pas une consigne) : RESEARCH = read-only, INNOVATE = pas de Bash, PLAN = écrit seulement dans `process/`. **Drift signal scoring** post-exécution : LOW (light touch) / MEDIUM (changements significatifs) / HIGH (fichiers harness/protocole touchés). **5-Persona Pre-Implementation Debate** (`vc-predict`) : Architect, Security, Performance, UX, Devil's Advocate → verdict **GO / CAUTION / STOP** avant d'écrire une ligne. **12-Dimension Edge Case Generator** (`vc-scenario`) → test specs. **STRIDE+OWASP audit** (`vc-security`) auto-fix par sévérité. **Smart auto-routing** (intent depuis NL, 6 niveaux de précédence, 1 question max). Skill discovery auto (scan des 32 skills).
3. **Stats / maturité** — ~847⭐ (jeune), MIT. 12 agents, 32 skills, 7 hooks. Claude Code/Codex/Cursor/**OpenCode**/Windsurf/Copilot.
4. **Intégration** — Install 30s sur n'importe quel codebase. Agents/skills Markdown.
5. **Force unique** — **3 patterns en or pour tes exigences** : phase-locked tools (= "outils forcés/interdits par phase"), drift scoring (= signal de re-loop), debate 5 personas (= checkpoint qualité avant code).
6. **Limites** — Jeune, petite communauté. RIPER-5 = lourd pour des micro-tâches (router les triviales hors pipeline complet).
7. **Décision** — **ABSORBER (loop + qualité).** Phase-lock → ta couche permissions (avec Serena ToolMarker). Drift scoring → observer. Debate 5 personas → étape optionnelle de la loop sur projets complexes. Édge-case generator → étape test.

## 3.5 Caveman Method

1. **Quoi** — Standard d'écriture/communication des agents.
2. **Fonctionnement** — Règles : zéro blabla (<10 mots/phrase), mots simples, voix active ("Fais X"), une idée/ligne, code > explication, si doute → demande, erreur → dire exactement quoi, fini → dire "fini".
3. **Stats** — pattern, pas de repo.
4. **Intégration** — Injecté dans le system prompt de chaque agent.
5. **Force unique** — **Réduit les tokens de sortie** et force la clarté/actionnabilité. Anti-hallucination de politesse ("il semble que").
6. **Limites** — Trop sec pour de la doc destinée aux humains → réserver aux échanges agent↔agent et logs.
7. **Décision** — **ABSORBER (standard de sortie).** Dans tous les agents. Désactiver pour les livrables humains (doc, rapports).

## 3.6 Obsidian Second Brain (ton projet — dual-vault MCP)

1. **Quoi** — 3ᵉ couche de connaissance : mémoire **personnelle & inter-projets** (distincte de CBM=code et Graphify=sémantique projet).
2. **Fonctionnement** — Double vault (ex. perso / technique) exposé via MCP custom. Reçoit les apprentissages distillés (sortie d'Acontext) + décisions archi + patterns récurrents. Lecture/écriture markdown, versionnable Git.
3. **Stats** — interne.
4. **Intégration** — MCP custom (le tien). Format Markdown (cohérent avec toute la stack).
5. **Force unique** — **Mémoire qui traverse les projets** : Agent OS se souvient de ce qui a marché ailleurs. Boucle Acontext → vault → relecture au prochain projet.
6. **Limites / risques** — Lecture brute coûte des tokens si le vault grossit → indexer (Graphify peut sortir vers Obsidian) ou résumer.
7. **Décision** — **INTÉGRER (Couche 1, 3ᵉ niveau).** Branche de sortie d'Acontext (3.x Batch 1). Pierre angulaire de la "mémorisation complète".

## 3.7 OpenWA — `open-wa/wa-automate-nodejs`

1. **Quoi** — Toolkit qui transforme un compte WhatsApp en API / bot / **MCP server pour agents**.
2. **Fonctionnement** — Pilote WhatsApp Web (Selenium/navigateur). Expose : Easy API (no-code), SocketClient, embedded runtime, plugin host, **MCP server**. Webhook bridge vers CRM/helpdesk. Multi-session. Intègre nativement **Chatwoot** + Node-RED.
3. **Stats / maturité** — ~3.6k⭐ (nodejs ; le doc disait 7.7k → gonflé), TS, v5 alpha / v4 stable, actif (juin 2026). Licence Hippocratic/Do-No-Harm.
4. **Intégration** — `npm i -g @open-wa/wa-automate`. MCP server pour brancher l'agent. Docker recommandé.
5. **Force unique** — **Canal WhatsApp** pour piloter Agent OS / recevoir des notifs / déclencher des tâches depuis ton téléphone.
6. **Limites / risques** — **Non officiel, contre les ToS WhatsApp → risque de ban du numéro.** Clé payante ($5/mo) pour numéros inconnus. Selenium = fragile.
7. **Décision** — **OPTIONNEL (canal, faible priorité).** Si tu veux un canal mobile, l'email/PWA d'Odysseus est plus sûr. N'activer WhatsApp que sur un numéro jetable, en connaissant le risque.

## 3.8 Chatwoot

1. **Quoi** — Plateforme open-source d'engagement client / helpdesk omnicanal.
2. **Fonctionnement** — Inbox unifiée (web chat, email, WhatsApp via OpenWA, réseaux), agents humains + bots, conversations assignables, API. Rails + Vue, self-host Docker.
3. **Stats / maturité** — Produit mature, large communauté.
4. **Intégration** — Self-host. Se branche à OpenWA comme front de conversation.
5. **Force unique** — **Front conversationnel multi-canal auditable** si Agent OS doit gérer du support.
6. **Limites** — Lourd. Redondant avec le **Chat d'Odysseus** si tu forkes.
7. **Décision** — **IGNORER si fork Odysseus.** Sinon, candidat front conversationnel. Phase lointaine.

## 3.9 Build Your Own X — `codecrafters-io/build-your-own-x`

1. **Quoi** — Corpus géant de tutoriels "recrée X from scratch" (DB, OS, Git, Redis, navigateur, compilateur…).
2. **Fonctionnement** — Liste curée de guides pas-à-pas, par techno. Pas de code unique — un index de ressources.
3. **Stats / maturité** — l'un des repos les plus étoilés de GitHub (~300k+⭐), très maintenu.
4. **Intégration** — À ingérer comme **corpus RAG** (`rag/BYOX-README.md` déjà esquissé chez toi).
5. **Force unique** — **Connaissance d'implémentation profonde** : quand l'agent doit construire un système non-trivial, ce corpus donne des plans éprouvés.
6. **Limites** — Volume. À filtrer/embedder (ou laisser Graphify l'indexer) plutôt que tout charger.
7. **Décision** — **INTÉGRER (corpus RAG, P2).** Source de référence pour les projets "from scratch". Indexer, pas charger brut.

---

## NOTES TRANSVERSES — apports du Batch 3

- **La constitution est fixée** : agents-best-practices fournit la loop canonique + la séquence de build + les 10 principes. **Tout Agent OS doit s'y conformer.** Les autres outils en sont des instanciations concrètes :
  - budgets → Paperclip · permissions par phase → vibecode phase-lock + Serena ToolMarker · échecs→features → Acontext + autoresearch · eval-the-harness → CodeBurn · trace riche → observer.
- **Phase-lock = convergence** : 3 sources indépendantes (Serena ToolMarker, vibecode RESEARCH/INNOVATE/PLAN, agents-best-practices "risk changes the loop") disent la même chose → **c'est un invariant d'architecture**, pas une option.
- **Sécurité = couche dédiée** : AgentSeal (probes) + HexStrike (validation commandes/scope/whitelist/audit/safe-mode, sans l'offensif) + HolyClaude (isolation Docker) → **couche SANDBOX/SECURITY** consolidée.
- **Mémoire = pipeline complet** : run → Acontext distille → SKILL.md + **Obsidian second-brain** → relecture inter-projets. Plus CBM/Graphify pour le code. Tout en Markdown.
- **Canaux = via Odysseus de préférence** (PWA/email) ; OpenWA/Chatwoot seulement si besoin spécifique, avec leurs risques.

---

# BATCH 4 — VEILLE / BANQUE / IGNORÉS (condensé)

## 4A. VEILLE — fiches courtes (surveiller, intégration future possible)

### 4A.1 SurfSense — `MODSetter/SurfSense`
- **Quoi/Fonctionnement** : alternative open-source NotebookLM/Perplexity/Glean. FastAPI + Next.js 16. **Hybrid search RRF** (vector + full-text), **Deep Agent Factory** (LangGraph) + **Tools Registry extensible**, **pipeline podcast** (2-host <20s), connecteurs 25+ via Celery + meta-scheduler, **Watch Local Folder (auto-sync vault Obsidian)**, automations planifiées/event-triggered (écrit dans Notion/Slack/Linear), RBAC + collab temps réel (Electric SQL). Exports cités PDF/DOCX/LaTeX/EPUB.
- **Stats** : ~14.4k⭐, Apache-2.0, très actif (v0.0.x, juin 2026).
- **Force** : le WF Recherche clé-en-main + Obsidian folder-watch (relie ton second-brain).
- **Limite** : lourd (Celery/Postgres/Next.js), redondant avec Deep Research d'Odysseus.
- **Décision** : **ABSORBER les patterns** (RRF hybrid search, Tools Registry, Report/Podcast generator, Obsidian watch) dans le WF Recherche. Ne pas intégrer en entier.

### 4A.2 master-skill (~49⭐)
- Distillation de connaissance industrie → skill Markdown. Niche, petit.
- **Décision** : **VEILLE.** Concept déjà couvert par Acontext (distillation). Surveiller.

### 4A.3 ANUS (~6.4k, l'agent — pas la version blockchain)
- Framework d'agents avec auto-évolution de roadmap.
- **Décision** : **VEILLE.** Idée d'auto-évolution intéressante mais redondante avec autoresearch + Acontext.

### 4A.4 Twenty (~49.4k) — `twentyhq/twenty`
- CRM open-source moderne (alt Salesforce). Data-model flexible, SDK, métadonnées custom.
- **Force/Décision** : **VEILLE (pattern).** Si Agent OS gère un jour des entités métier (clients/projets), son modèle de données custom + SDK est une bonne référence. Pas d'intégration directe.

### 4A.5 Hermes (Nous Research)
- Framework d'agents : cron, kanban, orchestration.
- **Décision** : **VEILLE.** Patterns d'orchestration ; ton Decision Engine + Paperclip couvrent déjà cron/kanban/budgets.

### 4A.6 Career-Ops (~43.1k)
- Pattern de **marketplace de skills** (découverte/partage).
- **Décision** : **VEILLE.** Pertinent quand ta banque de skills (Acontext/Open Design) grossira → besoin d'un registre/marketplace. Surveiller.

## 4B. BANQUE D'IDÉES — 1-2 lignes (potentiel futur, non prioritaire)

| Outil | Potentiel | Quand l'activer |
|---|---|---|
| **anime.js** (~52k) | Animations JS performantes | Habillage dashboard/Studio UI |
| **reactbits.dev** | Composants React animés prêts | UI du fork Odysseus / Studio |
| **UIverse.io** | Templates UI communautaires | Prototypage rapide d'UI |
| **Design Galleries** (1600+ refs) | Réfs UI (supahero, navbar.gallery, footer.design, 404s.design…) | Inspiration pour Open Design / Design Extract |
| **Blender MCP** (~565) | Génération 3D + **pattern orchestration multi-MCP** | Si besoin d'assets 3D ; étudier son pattern multi-MCP |
| **SketchUp MCP** (~245) | **Pattern bridge desktop↔MCP** réutilisable | Modèle pour brancher une app desktop |
| **MediaAgent / OpenGenAI** | Génération image/vidéo (200+ modèles) | Si Agent OS doit produire des médias |
| **Plausible Analytics** | Analytics privacy-first | Si Agent OS expose des endpoints publics (redondant avec CodeBurn en interne) |
| **ALTCHA** (~2.2k) | Captcha PoW self-hosted | Si endpoints publics à protéger |
| **ScrapGraphAI** | Scraping + knowledge graph | **Redondant** (Scrapling + Graphify le font déjà) — garder en idée seulement |

## 4C. IGNORÉS — une ligne justifiée chacun

| Outil | Raison du rejet |
|---|---|
| Almanac MCP | Redondant — ExternalScout + Context7 font mieux. |
| Wan2GP (~5.6k) | Génération vidéo IA — hors scope dev. |
| Open Generative AI (~11.7k) | Création média — hors scope. |
| **Open Design (article)** | Site **anti-scraping** — à ne PAS confondre avec `nexu-io/open-design` (intégré, §2.8). Rien à récupérer. |
| p-e-w/heretic (~20.6k) | Retrait de censure LLM — **modifie les modèles, pas les agents**. Hors scope. |
| Kling AI | Génération vidéo — pas de MCP. |
| Napkin AI | Text→diagrams — pas d'API (web app only). Kroki couvre le besoin. |
| Autodesk Fusion MCP (~41) | CAD propriétaire — niche. |
| AppFlowy-IO | Pas d'API viable pour orchestration. |
| Omma AI | Mineur, non pertinent. |
| blueprint.am | AI hardware design — hors scope. |
| ANUS (3⭐) | Framework blockchain non maintenu (≠ l'ANUS agent en veille). |
| flowint | N'existe pas sur GitHub. |
| **AContext / Odysseus "introuvables"** | **Résolu** : les deux ont été retrouvés et reclassés (Acontext §1.7 intégré, Odysseus §1.9 candidat UI). L'entrée "introuvable" du doc est obsolète. |

---

## NOTES TRANSVERSES — apports du Batch 4

- **Rien de bloquant ne sort de la veille/banque** : le socle (Batch 1-3) couvre déjà les besoins. La veille sert surtout d'**options futures** : SurfSense (patterns recherche), Twenty (data-model si entités métier), Career-Ops (marketplace de skills quand la banque grossit).
- **SurfSense = seule veille à patterns absorbables tout de suite** : RRF hybrid search + Report/Podcast + Obsidian watch → enrichit le WF Recherche sans intégration lourde.
- **Banque = couche cosmétique/extension** (anime.js, reactbits, galleries) pour l'UI, + 2 patterns réutilisables (Blender multi-MCP, SketchUp desktop-bridge).
- **Confirmation des doublons** : ScrapGraphAI (Scrapling+Graphify), Plausible (CodeBurn), Napkin (Kroki) — ne pas réintroduire.
- **Nettoyage** : l'entrée "AContext/Odysseus introuvables" du doc source est périmée — corrigée.

**Couverture : 60/60 outils analysés.** (Batch 1 : 13 · Batch 2 : 11 · Batch 3 : 9 · Batch 4 : 27 [6 veille + 10 banque + 14 ignorés, dont l'entrée doublon "introuvables"].)

> **Suite : Batch 4** [terminé ci-dessus]

---

# BATCH 5 — SYNTHÈSE SYSTÈME FINAL

> Objectif tenu : un système AI-driven ultra-modulaire sur base OpenCode, prenant le meilleur des 60 outils. Voici le blueprint consolidé.

## 5.1 La CONSTITUTION (invariants non-négociables)

Source : `agents-best-practices`. **Tout Agent OS s'y conforme.**

> Le modèle **propose**. Le harness **valide / autorise / exécute / loggue**. Le plan n'est jamais une policy de confiance.

**Loop canonique** :
`instructions → context builder → model call → tool proposal → validation → permission decision → execution OU approval pause → observation → next step / final`

**Séquence de construction** (ne jamais sauter d'étape, n'ajouter la suivante qu'après fiabilité) :
`loop manuel → tools → permissions → observations structurées → budgets → tracing → planning → context/memory → compaction → skills/connectors → goal loop → subagents`

**10 invariants** :
1. Le risque change la loop (read/draft/write/comm/$/destructif/privilégié → permissions différentes).
2. Draft ≠ commit (effets à haut risque → approbation tracée hors prompt).
3. Le contexte se construit (MVI), ne se déverse pas. Trust boundaries étiquetées.
4. Budgets obligatoires (step/time/token/cost/tool-call) **par projet**.
5. Progressive disclosure (noms+desc d'abord, charge le détail au besoin).
6. Les échecs répétés deviennent des features (validator/tool/doc/eval/policy).
7. La plupart des échecs ≠ manque d'autonomie → ajouter de la structure, pas de l'autonomie.
8. Le plan de workflow passe les mêmes gates que toute action.
9. Évaluer le harness, pas que le modèle.
10. Humans **on** the loop (maintiennent le harness) > humans **in** the loop.

## 5.2 ARCHITECTURE FINALE (couches + 2 transverses)

```
┌──────────────────────────────────────────────────────────────────┐
│  CLIENTS    TUI OpenCode · Desktop · PWA (fork Odysseus) · (WhatsApp opt.)│
├──────────────────────────────────────────────────────────────────┤
│ ┌─ GOVERNANCE (transverse) ── Paperclip patterns ───────────────┐ │
│ │ budgets/agent+auto-pause · goal-ancestry · approval+rollback ·  │ │
│ │ heartbeat scheduling · multi-projet isolé                       │ │
│ └────────────────────────────────────────────────────────────────┘ │
│ ┌─ SANDBOX / SECURITY (transverse) ─────────────────────────────┐  │
│ │ Docker cap_drop+no-new-priv (HolyClaude s6) · phase-lock        │  │
│ │ (Serena ToolMarker + vibecode) · cmd/scope validation (HexStrike)│ │
│ │ · AgentSeal probes · SAST/secrets                               │  │
│ └────────────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────┤
│ C5 AUTOMATION   Decision Engine (FastAPI) : router · scheduler ·    │
│                 executor · observer + decision-tree.yaml            │
│ ──────────────  ROUTING : Model Router (OmO patterns) + LiteLLM     │
├──────────────────────────────────────────────────────────────────┤
│ C4 OBSERVABILITÉ  CodeBurn (one-shot rate, waste, $↔commits) +      │
│                   drift scoring (vibecode) + traces (constitution)  │
├──────────────────────────────────────────────────────────────────┤
│ C3 DESIGN        Design Extract (URL→tokens) + Open Design (génère) │
├──────────────────────────────────────────────────────────────────┤
│ C2 EXÉCUTION     OpenCode core · Serena (édition) · Scrapling (web) │
│                  · Supabase (DB/state) · Faker · Kroki · Playwright │
├──────────────────────────────────────────────────────────────────┤
│ C1 CONNAISSANCE  Trinité: CBM(code always-on)·Graphify(sémantique)· │
│                  Obsidian(mémoire) + second-brain MCP               │
│ ──────────────   MÉMOIRE : Acontext (run→SKILL.md→vault, en boucle) │
└──────────────────────────────────────────────────────────────────┘
```

## 5.3 LA LOOP CANONIQUE UNIFIÉE (le cœur exécutable)

Fusion de : agents-best-practices (charpente) + OAC (plan-first/gates/MVI) + vibecode (RIPER-5/phase-lock/drift/debate) + autoresearch (eval figé) + Acontext (mémoire). Pilotée par `decision-tree.yaml`.

| Phase | Outils FORCÉS | Tools RETIRÉS (phase-lock) | Gate |
|---|---|---|---|
| 0. CLASSIFY | Decision Engine router | write, exec | — |
| 1. KNOW (checkpoint) | CBM + Graphify + Obsidian + Acontext recall | write, exec | — |
| 2. PLAN (MVI) | plan-first, écrit dans `process/` | exec, write code | — |
| 3. DEBATE (si complexe) | 5 personas (Architect/Sec/Perf/UX/Devil) → GO/CAUTION/STOP | write, exec | **STOP bloque** |
| 4. APPROVE | Governance (Paperclip approval) | — | **humain si destructif** |
| 5. BUILD | Serena (édition) + Faker (fixtures) | — | budget check |
| 6. QUALITY | Playwright (E2E) + SAST/secrets + simplify | write hors-scope | tests verts |
| 7. AUTOEVAL | eval figé + métrique (keep/revert, autoresearch) | — | done OU budget épuisé |
| 8. MEMORY | Acontext distille → SKILL.md + second-brain Obsidian | — | — |
| 9. OBSERVE | CodeBurn + drift score (LOW/MED/HIGH) | — | HIGH → re-loop/escalade |

Règles : les **forced_tools** sont vérifiés par l'`observer` (étape échoue si l'outil imposé n'a pas été appelé). Les **budgets** (token/iter/temps) viennent du `PROJECT.yaml` et coupent via Governance (auto-pause à 100%).

## 5.4 LES 7 EXIGENCES → réponse finale consolidée

1. **Routing intelligent configurable** → **Model Router** (`model-routing.json` : matrice agent→modèle + fallback chains + runtime_fallback + catégories quick/deep/utility, patterns OmO) sur **LiteLLM** (abstraction Groq/Ollama/Anthropic). Ton `.bat` toggle = un profil de plus.
2. **Sandbox** → couche transverse : Docker durci (cap_drop/no-new-priv, s6-overlay HolyClaude) + **phase-lock réel** (Serena ToolMarker + vibecode) + validation cmd/scope (HexStrike défensif) + AgentSeal.
3. **Workflows + decision tree cross-workflow** → **Decision Engine** + `decision-tree.yaml` déclaratif (enchaîne les WF, porte les forced_tools/checkpoints) → le modèle exécute des étapes atomiques, l'arbre porte l'orchestration.
4. **Checkpoints + outils forcés** → loop §5.3 : KNOW/QUALITY/AUTOEVAL/MEMORY obligatoires ; forced_tools vérifiés par l'observer ; phase-lock retire les tools hors phase.
5. **Loops à objectif + limites par projet** → `PROJECT.yaml` (objective + done_definition mesurable + constraints + **budgets** + `eval_command` figé + metric) ; boucle keep/revert (autoresearch) ; coupe par Governance.
6. **Intégration facile d'outils** → **MCP-first** : tout outil = MCP server + skill Markdown, activable par flag dans `config.json` ; grille 7 axes /35 avant ajout ; CodeBurn détecte les MCP inutiles à désactiver.
7. **Auto-éval + mémoire complète** → boucle A (autoeval : eval figé + drift + signaux CodeBurn) ; boucle B (mémoire : Acontext distille run→SKILL.md→ second-brain Obsidian + ré-index CBM/Graphify). Tout en Markdown, versionné Git.

## 5.5 STACK CONCRET — qui fait quoi

| Slot | Outil retenu | Mode |
|---|---|---|
| Base/serveur | OpenCode | intégrer (socle) |
| UI/workspace | **fork Odysseus** (MIT) | intégrer (à trancher) |
| Loop/philosophie | agents-best-practices + OAC | absorber (constitution) |
| Routing | Model Router (OmO) + LiteLLM | absorber→implémenter |
| Connaissance code | CBM + Graphify | intégrer (Trinité) |
| Mémoire | Acontext + Obsidian (+second-brain) | intégrer (pattern d'abord) |
| Édition code | Serena | intégrer P0 |
| Web | Scrapling | intégrer (Crawl4AI/Browser-Harness OFF) |
| DB/state | Supabase MCP | intégrer |
| Tests | Playwright + Faker | intégrer |
| Diagrammes | Kroki | intégrer |
| Design | Design Extract + Open Design | intégrer ON |
| Observabilité | CodeBurn (+AgentSeal sécurité) | intégrer ON |
| Orchestration | Decision Engine + decision-tree.yaml | étendre (maison) |
| Governance | Paperclip patterns | absorber→Decision Engine |
| Sandbox | HexStrike (défensif) + HolyClaude + AgentSeal | absorber |
| Qualité/phase | vibecode (RIPER-5/phase-lock/drift/debate) | absorber |
| Auto-optim | autoresearch (pattern eval figé) | absorber |
| Recherche (futur) | SurfSense (patterns) | veille→absorber P2 |
| RAG | Build Your Own X | intégrer P2 |
| Canaux (opt.) | OpenWA/Chatwoot | optionnel (via Odysseus de préférence) |

## 5.6 ORDRE DE BUILD RÉVISÉ (conforme à la séquence de la constitution)

| # | Étape | Couvre exigence | Effort |
|---|---|---|---|
| 1 | **Loop manuelle fiable** + 10 invariants dans chaque agent | constitution | M |
| 2 | **Model Router** (`model-routing.json`) sur LiteLLM | #1 routing | M |
| 3 | **Permissions + phase-lock** (Serena ToolMarker + vibecode) | #2 sandbox, #4 | M |
| 4 | **Serena P0 + Scrapling + Supabase** (MCP) | #6 exécution | M |
| 5 | **Observations structurées + budgets** (observer + PROJECT.yaml budgets) | #4 #5 | M |
| 6 | **CodeBurn** (one-shot rate, waste) bouclé dans l'observer | #7 obs | S |
| 7 | **decision-tree.yaml** + forced_tools vérifiés | #3 #4 | M |
| 8 | **PROJECT.yaml + autoeval loop** (eval figé/keep-revert) | #5 #7 | M |
| 9 | **Mémoire** : pattern Acontext (distill→Markdown→Obsidian) | #7 | L |
| 10 | **Governance** : budgets auto-pause + goal-ancestry + heartbeat (Paperclip patterns dans Decision Engine) | #5 gouvernance | L |
| 11 | **Sandbox durci** : Docker cap_drop/s6 + cmd validation + AgentSeal | #2 | M |
| 12 | **Design ON** : Design Extract + Open Design | design | S |
| 13 | **Debate 5 personas + edge-case gen** (vibecode) | qualité | S |
| 14 | **UI** : fork Odysseus, brancher Trinité+Router+Decision Engine | UI | L |
| 15 | **RAG (BYOX) + SurfSense patterns** | recherche | L |

> Principe : étapes 1→8 d'abord (le harness fiable). N'ajouter mémoire/governance/UI/RAG (9→15) qu'une fois le socle stable — invariant n°7 (structure avant autonomie).

## 5.7 DÉCISIONS OUVERTES (à trancher)

1. **UI : fork Odysseus vs maison.** Fork = mois gagnés (PWA, email, Cookbook VRAM-aware idéal Ollama) mais dette d'un gros codebase + neutraliser son ChromaDB (au profit de ta Trinité). Reco : fork, traité comme un client de plus sur ton serveur OpenCode.
2. **Acontext : stack complet vs pattern léger.** Postgres+Redis+RabbitMQ+S3 = lourd. Reco : commencer par le **pattern** (distillation maison → Markdown → Obsidian), migrer vers Acontext full si le besoin se confirme.
3. **Paperclip : absorber vs forker.** Absorber les patterns dans le Decision Engine = contrôle + légèreté. Forker = control-plane multi-projets prêt mais chevauche Odysseus. Reco : absorber.
4. **Profondeur du phase-lock.** Serena le fait nativement pour le code ; à étendre à TOUS les tools via le ToolRegistry/mode. Décider du mapping phase→tools exact.
5. **Canaux.** WhatsApp (OpenWA) = pratique mais risque de ban. Reco : s'en tenir à PWA/email d'Odysseus sauf besoin réel.

## 5.8 PRINCIPES DIRECTEURS (ce que l'analyse des 60 outils a prouvé)

- **Tout en Markdown** : agents, skills, mémoire, design systems, permissions. Versionnable, lisible, portable. Pas d'embeddings opaques (seule exception à neutraliser : ChromaDB d'Odysseus).
- **MCP-first** : chaque capacité = un serveur branchable par flag. La modularité vient de là.
- **Le harness > le modèle** : la fiabilité vient de la structure déterministe autour du LLM, pas d'un modèle plus gros (invariant n°9).
- **Phase-lock = invariant** (3 sources convergentes) : les tools sont vraiment retirés/accordés par phase.
- **Boucles bornées** : aucun projet ne tourne sans objective + done_definition + budgets.
- **Mémoire = boucle fermée** : chaque run nourrit la connaissance réutilisée au suivant.
- **Construire dans l'ordre** : socle fiable d'abord ; autonomie/mémoire/UI ensuite.

**FIN — 60/60 outils analysés, architecture consolidée, ordre de build défini.**

---

# ADDENDUM — GSD + arbitrages finaux (recherche live)

## A.1 GSD — "Get Shit Done" par TÂCHES — `gsd-build/get-shit-done` (+ fork OpenCode `rokicool/gsd-opencode`)

1. **Quoi** — Moteur de workflow **spec-driven** pour agents : transforme une intention en logiciel livré sans perte de cohérence ("context rot").
2. **Fonctionnement** — Découpe le travail en **phases à frontières d'exécution atomiques**. Chaque phase = **fenêtre de contexte fraîche + plan détaillé + état persistant**. Cycle : *Define project → Discuss phase → Plan → Execute (worktree Git isolé) → Verify (tests/audit/approbation) → next*. **Orchestrateur mince** qui spawn des subagents spécialisés (gsd-planner, gsd-roadmapper, gsd-plan-checker, gsd-phase-researcher, gsd-project-researcher, gsd-research-synthesizer, gsd-codebase-mapper, Executor, Verifier, Debugger), chacun avec son propre contexte 200k. **XML Task Formatting** (tâches atomiques + étapes de vérification). **Wave-Based Execution** (groupe les plans par dépendances → DAG parallèle/séquentiel). **Pipeline Lists** (inspiré du coprocesseur Copper de l'Amiga) : programmes de workflow **déclaratifs** (WAIT / MOVE / SKIP) synchronisés aux événements du cycle GSD, **pré-compilés au planning**, exécutés automatiquement aux transitions de phase — *l'IA ne décide pas quel skill charger, le pipeline le fait* (modes : `sprite` ~200 tokens, `offload` hors-contexte). **Stage-based model assignment** : on assigne les modèles à 3 stages (planning/execution/verification) via profils, pas par agent ; GSD écrit `opencode.json`. **Commits atomiques revertables** (git bisect trouve la tâche fautive). Config dans `.planning/config.json`.
3. **Stats / maturité** — repo très discuté (TÂCHES, org `gsd-build`), SDK TS `@gsd-build/sdk`, support Claude Code/Codex/OpenCode/Windsurf/Cursor, GSD v2 = routing auto.
4. **Intégration** — Fork **`gsd-opencode`** natif OpenCode (commandes `/gsd-new-project`, `/gsd-set-profile`…). `.planning/` = **ta structure actuelle** → tu utilises déjà GSD.
5. **Force unique** — **La structuration de plan + délimitation de limites + isolation de contexte par phase** = exactement ce que tu cherches. Pipeline Lists = orchestration déclarative pré-compilée (bien plus mûr que mon `decision-tree.yaml`). Stage model assignment = pré-intègre le routing.
6. **Limites / risques** — Orienté dev/code (à élargir pour des projets non-code). Opinionated sur la structure `.planning/`. v1 routing manuel (v2 auto).
7. **Décision** — **INTÉGRER comme MOTEUR DE WORKFLOW/PLANNING (nouvelle couche cœur).** Le **gsd-planner / gsd-roadmapper** = ton "agent de planification spécialisé". Ses **Pipeline Lists remplacent** le `decision-tree.yaml` que j'avais esquissé. Son **stage model assignment** se marie au Model Router. Base = fork `gsd-opencode`.

## A.2 Acontext — verdict "indispensable ?"

- **Ce qui est dur à refaire** : le moteur de **distillation automatique continue** (Experience Agent surveille les sessions → détecte succès/échec → LLM distille ce-qui-a-marché / échoué / préférences → Skill Agent décide créer/mettre à jour un SKILL.md) + **Task Agent** (suit état/progrès/préférences en live). Reconstruire ça proprement = gros effort, faible valeur ajoutée.
- **Ce qui est trivial / sans lock-in** : la sortie = **Markdown SKILL.md** (git diff/grep/mount), export ZIP, Apache-2.0, marche avec n'importe quel framework. Pas d'embeddings.
- **Poids** : cloud = `pip install acontext` + clé (trivial, mais data externe). Self-hosted = serveur sur `:8029` avec **Postgres embarqué auto-créé** (plus léger qu'annoncé ; pas de setup DB externe obligatoire).
- **VERDICT : INDISPENSABLE → INTÉGRATION COMPLÈTE (self-hosted).** La mémorisation/auto-apprentissage est un objectif de 1er rang du projet ; le moteur de distillation est précisément le cœur dur. Lock-in quasi nul. Sortie branchée sur le **second-brain Obsidian**. (On abandonne l'option "pattern léger d'abord".)

## A.3 Paperclip — ce qu'on ABSORBE (gestion de projet)

Source d'inspiration majeure pour le PM. On **absorbe les patterns** (pas de fork — l'UI = Odysseus). À porter dans la couche Governance + la structure de projet :

1. **Hiérarchie goal-ancestry** : `mission → goal → project → task`. Chaque tâche trace son "pourquoi" jusqu'à la mission. → **C'est la colonne vertébrale de la structuration de projet.** Le `PROJECT.yaml` devient cette chaîne.
2. **Budgets granulaires** : tracking token+coût par company/agent/project/goal/issue/provider/**model** ; policies à **seuils d'alerte + hard stops** (auto-pause). → tes "limites par projet", au niveau le plus fin.
3. **Governance + rollback** : approval gates appliqués, changements de config **versionnés + rollback** sûr.
4. **Heartbeat + "Memento Man"** : l'agent se réveille, lit sa checklist, agit ; **injection de contexte explicite à chaque réveil** (l'agent est amnésique). → résout l'état entre runs.
5. **"Manage goals, not terminals"** : le dashboard suit des **objectifs/états**, pas des process. → modèle mental de l'UI de pilotage.
- *(Vu en passant : Vibe Kanban = board kanban open-source pour tâches d'agents dev — alternative d'UI PM si Odysseus ne suffit pas. Veille.)*

## A.4 Phase-lock — décision (je tranche)

**Mécanisme = Serena** (ToolMarker + ToolRegistry filtré par mode/contexte), **étendu à TOUS les tools** (pas que l'édition de code). **Politique = mapping de vibecode** (RESEARCH=read-only, INNOVATE=no exec, PLAN=écrit `process/`/`.planning/` seulement, BUILD=full, VERIFY=read+test). Serena fournit le moteur natif fiable ; on déclare le mapping phase→tools dans la config. **Pas de réinvention.**

## A.5 Canaux — décision (communiquer avec le système + outils tiers)

Objectif : (a) parler à Agent OS depuis des chats, (b) un max d'outils de communication avec des tiers. Architecture = **Channel Gateway** (un bus entrée/sortie, plusieurs adapters) :

- **Chat (piloter le système)** — priorité aux **API officielles sans risque de ban** :
  - **Discord** (tu gères déjà des serveurs Discord → adapter naturel, bot officiel).
  - **Telegram** (Bot API officielle, simple, fiable).
  - **Email** (IMAP/SMTP natif d'Odysseus).
  - **WhatsApp via OpenWA** — *optionnel*, **risque ToS/ban**, numéro jetable seulement.
- **Communication avec des tiers (outbound)** — via **MCP connectors** (tu en as déjà : Gmail, Google Calendar, Google Drive) + à ajouter selon besoin : Slack, Discord webhooks, SMTP, Notion, Linear. Le Channel Gateway expose une API unique ; chaque tiers = un adapter MCP activable par flag.
- **Principe** : un seul **inbound bus** (messages → intent → Decision Engine) et un seul **outbound bus** (notifications/résultats → adapters). Ajouter un canal = ajouter un adapter, pas toucher au cœur.

## A.6 ARCHITECTURE RÉVISÉE (intégrant GSD + Channel Gateway + Governance Paperclip)

```
CLIENTS + CHANNEL GATEWAY
  TUI · Desktop · PWA (Odysseus fork) · Discord · Telegram · Email · WhatsApp(opt)
  inbound bus → intent ;  outbound bus → adapters MCP tiers (Gmail/Cal/Slack/Notion…)
        │
GOVERNANCE (transverse, patterns Paperclip)
  mission→goal→project→task · budgets granulaires (seuils+hard stop) · approval+rollback · heartbeat
        │
WORKFLOW / PLANNING ENGINE  ← NOUVELLE COUCHE CŒUR
  GSD (gsd-opencode) : phases à contexte frais · Pipeline Lists (WAIT/MOVE/SKIP) ·
  wave/DAG · subagents (planner/roadmapper/researcher/executor/verifier/debugger) ·
  stage model assignment ──┐
        │                   └──→ ROUTING : Model Router (OmO) + LiteLLM
AUTOMATION RUNTIME
  Decision Engine (host + glue Governance + Channel Gateway + observer)
        │
OBSERVABILITÉ  CodeBurn (one-shot/waste/$↔commits) + drift + traces
DESIGN         Design Extract + Open Design
EXÉCUTION      OpenCode core · Serena (édition + phase-lock) · Scrapling · Supabase · Faker · Kroki · Playwright
SANDBOX/SEC    Docker cap_drop/s6 · phase-lock (Serena) · cmd/scope validation (HexStrike défensif) · AgentSeal
CONNAISSANCE   Trinité CBM·Graphify·Obsidian(+second-brain)
MÉMOIRE        Acontext (self-hosted) : run→distille→SKILL.md→second-brain, en boucle
```

## A.7 DÉCISIONS — état final (toutes tranchées)

| Décision | Tranché |
|---|---|
| Base UI | **Fork Odysseus** (s'approprier la codebase), neutraliser ChromaDB → Trinité |
| Workflow/Planning | **GSD (gsd-opencode)** = moteur cœur ; Pipeline Lists remplacent decision-tree.yaml |
| Mémoire | **Acontext full self-hosted** (indispensable) |
| Gestion de projet | **Absorber Paperclip** (goal-ancestry + budgets + governance + heartbeat) |
| Phase-lock | **Serena** (moteur) + mapping vibecode (politique), étendu à tous les tools |
| Routing | Model Router (OmO) + LiteLLM, alimenté par les stages GSD |
| Canaux | **Channel Gateway** : Discord + Telegram + Email (sûrs), WhatsApp opt., tiers via MCP |
| Orchestration runtime | Decision Engine = host/glue (Governance + Gateway + observer) |

**FIN ADDENDUM.**
