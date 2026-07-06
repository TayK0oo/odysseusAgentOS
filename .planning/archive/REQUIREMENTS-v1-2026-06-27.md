# Requirements: AgentOS — Odysseus

**Defined:** 2026-06-27
**Core Value:** Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, apprend de chaque run, et ne dépasse jamais ses limites

## v1 Requirements

### Constitution (Harness)

- [ ] **CONST-01**: Le harness classifie le niveau de risque de chaque action (read / draft / write / exec / destructif) avant exécution
- [ ] **CONST-02**: Les traces d'exécution sont structurées en JSONL (run_id, tool, risk_level, permission_decision, cost, outcome)
- [ ] **CONST-03**: Toute action à haut risque passe par une gate d'approbation tracée hors-prompt
- [ ] **CONST-04**: Draft ≠ commit — les effets irréversibles nécessitent une confirmation explicite
- [ ] **CONST-05**: Les 10 invariants de la Constitution sont encodés dans chaque agent Markdown

### Routing

- [ ] **ROUT-01**: LiteLLM est l'abstraction unique vers tous les providers (ollama, opencode_zen, openrouter, anthropic, groq)
- [ ] **ROUT-02**: Un IntentGate classifie l'intention avant de router vers le modèle approprié (quick/deep/utility/vision/code/creative)
- [ ] **ROUT-03**: La matrice agent→modèle est déclarative (model-routing.json étendu) avec fallback chains par catégorie
- [ ] **ROUT-04**: Le runtime_fallback gère les erreurs 400/429/500/502/503 avec cooldown et blacklist temporaire
- [ ] **ROUT-05**: Les hash-anchored edits valident chaque modification avant application (LINE#ID)
- [ ] **ROUT-06**: Le stage model assignment consomme les stages définis par GSD (planning/execution/verification)

### Phase-lock & Permissions

- [ ] **PERM-01**: Serena MCP est intégré comme service Docker (édition symbolique LSP-backed)
- [ ] **PERM-02**: Un ToolRegistry filtre tool_index.py selon la phase courante (RESEARCH/INNOVATE/PLAN/BUILD/VERIFY)
- [ ] **PERM-03**: RESEARCH = read-only (aucun write/exec possible)
- [ ] **PERM-04**: PLAN = écriture dans process/ et .planning/ uniquement
- [ ] **PERM-05**: BUILD = accès complet
- [ ] **PERM-06**: VERIFY = read + test seulement (pas d'écriture code)
- [ ] **PERM-07**: Le mapping phase→tools est déclaratif (YAML) et appliqué réellement (tools retirés, pas juste interdits par consigne)

### Exécution MCP

- [ ] **EXEC-01**: Scrapling MCP est intégré comme service Docker (adaptive scraping, spider async, pause/resume)
- [ ] **EXEC-02**: Scrapling remplace SearXNG pour le web scraping ; Crawl4AI et Browser-Harness restent OFF en fallback
- [ ] **EXEC-03**: Supabase MCP est intégré (read_only par défaut, branching pour les tests)
- [ ] **EXEC-04**: Faker.js est disponible dans l'environnement de test (fixtures réalistes seedables)
- [ ] **EXEC-05**: Kroki est disponible comme service Docker (REST → SVG/PNG pour tous les diagrammes)

### Observations & Budgets

- [ ] **OBS-01**: Chaque projet dispose d'un PROJECT.yaml (objective, done_definition, constraints, budgets, eval_command, metric)
- [ ] **OBS-02**: Un budget_enforcer coupe la loop quand max_tokens / max_iterations / max_cost_usd est atteint
- [ ] **OBS-03**: Les observations structurées (JSONL) sont écrites après chaque tool call avec outcome + cost
- [ ] **OBS-04**: CodeBurn est installé et tourne après chaque session (one-shot rate, waste patterns, $↔commits)
- [ ] **OBS-05**: Le one-shot rate et les waste patterns sont injectés dans l'observer du Decision Engine
- [ ] **OBS-06**: AgentSeal tourne en CI (300+ probes : injection, MCP empoisonnés, skills malveillants)

### Workflow (GSD)

- [ ] **WF-01**: GSD (gsd-opencode) est le moteur de planning — .planning/ initialisé, ROADMAP.md maintenu
- [ ] **WF-02**: Les Pipeline Lists (WAIT/MOVE/SKIP) remplacent le decision-tree.yaml esquissé
- [ ] **WF-03**: Les subagents GSD spécialisés sont configurés (planner, roadmapper, researcher, executor, verifier, debugger)
- [ ] **WF-04**: Le wave-based execution groupe les plans par dépendances (DAG parallèle/séquentiel)
- [ ] **WF-05**: Le stage model assignment GSD écrit automatiquement la config modèle

### Autoeval

- [ ] **EVAL-01**: Chaque projet a un eval_command figé (jamais modifié par l'agent) + une métrique unique
- [ ] **EVAL-02**: La boucle keep/revert (pattern autoresearch) reverte automatiquement si la métrique baisse
- [ ] **EVAL-03**: Le budget temps/iter par run est respecté (coupe avant épuisement du budget global)

### Mémoire (Acontext)

- [ ] **MEM-01**: Acontext self-hosted tourne comme service Docker (:8029, Postgres embarqué)
- [ ] **MEM-02**: À la fin de chaque session, le pipeline distille : session → Experience Agent → Skill Agent → SKILL.md
- [ ] **MEM-03**: Les SKILL.md générés alimentent .opencode/skills/ (format natif OpenCode)
- [ ] **MEM-04**: Les apprentissages sont aussi pushés vers le vault Obsidian second-brain (agentos/)
- [ ] **MEM-05**: L'Obsidian second-brain est exposé via MCP custom (lecture/écriture Markdown)
- [ ] **MEM-06**: La mémoire ChromaDB existante (RAG documents) coexiste et n'est pas remplacée

### Governance (Patterns Paperclip)

- [ ] **GOV-01**: La hiérarchie goal-ancestry est implémentée dans core/database.py (tables goals, projects liées mission→goal→project→task)
- [ ] **GOV-02**: Les budgets granulaires par agent/project/provider/model sont trackés avec seuils d'alerte + hard stops
- [ ] **GOV-03**: L'auto-pause à 100% du budget bloque les nouvelles tâches de l'agent concerné
- [ ] **GOV-04**: Les approval gates pour actions destructives sont tracées hors-prompt avec rollback possible
- [ ] **GOV-05**: Le heartbeat scheduling injecte le contexte explicite à chaque réveil d'agent (pattern "Memento Man")
- [ ] **GOV-06**: Le dashboard Odysseus affiche des objectifs/états (pas juste des conversations)

### Channel Gateway

- [ ] **CHAN-01**: Un inbound bus unifié reçoit les messages (Discord / Telegram / Email) → intent classifier → Decision Engine
- [ ] **CHAN-02**: Un outbound bus unifié dispatche les résultats/notifications vers les adapters configurés
- [ ] **CHAN-03**: Un Discord bot adapter est implémenté (discord.py)
- [ ] **CHAN-04**: Un Telegram bot adapter est implémenté (python-telegram-bot, Bot API officielle)
- [ ] **CHAN-05**: Les MCP connectors outbound sont configurables par flag (Gmail, Calendar, Slack, Notion, Linear)
- [ ] **CHAN-06**: Ajouter un canal = ajouter un adapter, sans toucher au cœur

### Knowledge (Trinité)

- [ ] **KNO-01**: CBM (codebase-memory-mcp) est ajouté comme service Docker dans docker-compose.yml
- [ ] **KNO-02**: Graphify est ajouté comme service Docker (on-demand, pour onboarding projet + archi globale)
- [ ] **KNO-03**: Les routes /api/knowledge/* exposent CBM + Graphify dans l'API FastAPI
- [ ] **KNO-04**: L'Obsidian second-brain MCP est exposé dans docker-compose (couche mémoire inter-projets)
- [ ] **KNO-05**: Le Decision Engine (depuis ConfigOpenCodeNew/decision-engine/) est intégré comme service docker-compose
- [ ] **KNO-06**: Le Decision Engine = glue pour Governance + Channel Gateway + observer

### Sandbox & Sécurité

- [ ] **SEC-01**: docker-compose.yml ajoute cap_drop: ALL + no-new-privileges: true sur les services agents (hors Cookbook)
- [ ] **SEC-02**: s6-overlay est utilisé pour la supervision de process dans les conteneurs longue durée (HolyClaude)
- [ ] **SEC-03**: Un command_validator.py valide les commandes bash avant exécution (scope whitelist, patterns dangereux)
- [ ] **SEC-04**: AgentSeal tourne en CI (voir OBS-06)
- [ ] **SEC-05**: L'audit prompt-injection (skills/notes/documents/mémoires comme surfaces non-fiables) est documenté + testé

### Design

- [ ] **DSGN-01**: Design Extract (designlang) est installé et configuré comme MCP server
- [ ] **DSGN-02**: Open Design (nexu-io) est installé et configuré comme MCP server
- [ ] **DSGN-03**: Les 259 skills design (SKILL.md) d'Open Design sont dans .opencode/skills/design/
- [ ] **DSGN-04**: Le drift CI (designlang drift) est configuré pour détecter la dérive de tokens CSS

### Qualité & Débat

- [ ] **QUAL-01**: Un agent Markdown "debate-5-personas" implémente le pattern vibecode (Architect/Security/Perf/UX/Devil → GO/CAUTION/STOP)
- [ ] **QUAL-02**: Un agent Markdown "edge-case-gen" génère des specs de test sur 12 dimensions (pattern vibecode vc-scenario)
- [ ] **QUAL-03**: Un agent Markdown "security-audit" lance STRIDE+OWASP et propose des fixes par sévérité

### RAG avancé

- [ ] **RAG-01**: RRF hybrid search est implémenté dans src/rag_vector.py (BM25 + vecteur fusionnés par Reciprocal Rank Fusion)
- [ ] **RAG-02**: Le corpus Build Your Own X est indexé dans le RAG (rag/BYOX/)
- [ ] **RAG-03**: L'Obsidian folder-watch auto-sync le vault vers le RAG

### Bugs & Refactor amont

- [ ] **BUG-01**: Smoke tests fresh install (Linux + macOS + Windows + Docker + WSL) — couverture minimale
- [ ] **BUG-02**: Cookbook reliability cross-machines (SGLang support cross-platforms)
- [ ] **BUG-03**: Degraded-state reporting (ChromaDB/SearXNG/email/ntfy) — pas de fail silencieux
- [ ] **BUG-04**: Email performance audit (IMAP bottleneck profiling)
- [ ] **BUG-05**: Provider probing audit (Anthropic/Gemini/Groq/xAI/OpenRouter/OpenAI/DeepSeek)
- [ ] **BUG-06**: Agent prompt/context bloat pour petits modèles (4k/8k) — slimmer prompts + tool selection
- [ ] **BUG-07**: Prompt-injection audit (skills/notes/documents/mémoires comme surfaces non-fiables)
- [ ] **BUG-08**: CSS refactor static/style.css (1.2MB → < 200KB)
- [ ] **BUG-09**: Tour core helper (déduplication scaffolding onboarding)
- [ ] **BUG-10**: Dead code pass (routes/flags/états UI stales)
- [ ] **BUG-11**: Accessibility (keyboard navigation, focus states, contrast, reduced motion)

## v2 Requirements

### Canaux optionnels

- **CHAN-V2-01**: WhatsApp via OpenWA (numéro jetable uniquement, risque ban ToS connu)
- **CHAN-V2-02**: Chatwoot front conversationnel multi-canal (si Odysseus ne suffit pas)

### RAG étendu

- **RAG-V2-01**: SurfSense patterns — Report/Podcast generator depuis le RAG
- **RAG-V2-02**: Automations planifiées depuis le RAG (écriture dans Notion/Slack/Linear)

### UI avancée

- **UI-V2-01**: anime.js pour animations dashboard
- **UI-V2-02**: reactbits.dev composants animés
- **UI-V2-03**: Blender MCP (pattern multi-MCP orchestration — optionnel si assets 3D)

### Veille

- **VEILLE-01**: SurfSense RRF + Tools Registry extensible (absorber les patterns quand stable)
- **VEILLE-02**: Twenty CRM (data-model si Agent OS gère des entités métier)
- **VEILLE-03**: Career-Ops marketplace de skills (quand la banque de skills grossit)

## Out of Scope

| Feature | Reason |
|---------|--------|
| n8n | Remplacé par Decision Engine Python — moins lourd, même contrôle |
| ChromaDB comme mémoire agentique | Acontext couvre ce cas ; ChromaDB reste pour RAG docs |
| Arsenal offensif HexStrike (150 outils) | Hors scope dev ; seuls les patterns défensifs |
| Fork complet Paperclip | UI = Odysseus ; patterns absorbés dans Decision Engine |
| Fork complet vibecode | Patterns RIPER-5/phase-lock/drift absorbés en agents Markdown |
| OpenWA/Chatwoot v1 | Risque ban WhatsApp ; Discord/Telegram suffisent |
| Almanac MCP | Redondant avec CBM + Context7 |
| Wan2GP, Open Generative AI, Kling AI | Génération vidéo — hors scope dev |
| p-e-w/heretic | Modifie les modèles, pas les agents — hors scope |
| ScrapGraphAI | Redondant : Scrapling + Graphify couvrent le besoin |
| Plausible Analytics | Redondant avec CodeBurn pour l'interne |
| Napkin AI | Pas d'API ; Kroki couvre le besoin |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONST-01 à CONST-05 | Phase 1 | Pending |
| ROUT-01 à ROUT-06 | Phase 2 | Pending |
| PERM-01 à PERM-07 | Phase 3 | Pending |
| EXEC-01 à EXEC-05 | Phase 4 | Pending |
| OBS-01 à OBS-06 | Phase 5 | Pending |
| WF-01 à WF-05 | Phase 6 | Pending |
| EVAL-01 à EVAL-03 | Phase 7 | Pending |
| MEM-01 à MEM-06 | Phase 8 | Pending |
| GOV-01 à GOV-06 | Phase 9 | Pending |
| CHAN-01 à CHAN-06 | Phase 10 | Pending |
| KNO-01 à KNO-06 | Phase 11 | Pending |
| SEC-01 à SEC-05 | Phase 12 | Pending |
| DSGN-01 à DSGN-04 | Phase 13 | Pending |
| QUAL-01 à QUAL-03 | Phase 14 | Pending |
| RAG-01 à RAG-03 | Phase 15 | Pending |
| BUG-01 à BUG-11 | Phase 15 | Pending |

**Coverage:**
- v1 requirements: 76 total
- Mapped to phases: 76
- Unmapped: 0 ✓

---
*Requirements defined: 2026-06-27*
*Last updated: 2026-06-27 après analyse profonde 60 outils*
