# Roadmap: AgentOS — Odysseus

**Project:** AgentOS — Odysseus
**Created:** 2026-06-27
**Total phases:** 15
**Requirements mapped:** 76/76 ✓

> Principe directeur : phases 1→8 = harness fiable. Ne passer à 9→15 (mémoire/governance/UI/design) qu'une fois le socle stable (invariant Constitution n°7 : structure avant autonomie).

---

## Phase 1 — Constitution : Loop + Traces

**Goal:** Encoder les 10 invariants de la Constitution dans la loop et structurer les traces d'exécution — fondation sur laquelle tout le reste repose.

**Requirements:** CONST-01, CONST-02, CONST-03, CONST-04, CONST-05

**Plans:**
1. Risk classifier (catégories read/draft/write/exec/destructif) intégré en entrée de `src/agent_loop.py`
2. Structured traces JSONL (run_id, tool, risk_level, permission_decision, cost, outcome) dans `src/trace_writer.py`
3. Gate d'approbation pour actions destructives (tracée hors-prompt, diff affichée)
4. Agents Markdown `.opencode/agents/` mis à jour avec les 10 invariants

**Success criteria:**
1. Chaque tool call produit une ligne JSONL dans `data/traces/YYYY-MM-DD.jsonl`
2. Une action destructive (rm, drop table, force push) demande confirmation et log la décision
3. Les 10 invariants sont lisibles dans au moins un agent Markdown
4. Le risk_level est visible dans les logs de session

---

## Phase 2 — Model Router LiteLLM

**Goal:** Remplacer la gestion manuelle des providers par LiteLLM et ajouter l'IntentGate + hash-anchored edits.

**Requirements:** ROUT-01, ROUT-02, ROUT-03, ROUT-04, ROUT-05, ROUT-06

**Plans:**
1. LiteLLM comme wrapper unique dans `src/llm_core.py` (pip install litellm, migrate providers)
2. IntentGate : classifier de tâche (quick/deep/utility/vision/code/creative) avant routing
3. Matrice agent→modèle étendue dans `model-routing.json` (catégories + fallback chains OmO)
4. Hash-anchored edits : validation LINE#ID avant application de chaque modification
5. Stage model assignment : consommer les stages GSD (planning/execution/verification) dans le router

**Success criteria:**
1. Tous les providers passent par LiteLLM (`litellm.completion(...)` uniquement)
2. L'IntentGate classe correctement 10 prompts de test (un par catégorie)
3. Un fallback 429 → modèle suivant fonctionne sans intervention
4. Une édition avec hash invalide est rejetée avec message d'erreur clair

---

## Phase 3 — Phase-lock & Serena MCP

**Goal:** Implémenter le phase-lock réel (tools retirés par phase) avec Serena comme moteur d'édition symbolique.

**Requirements:** PERM-01, PERM-02, PERM-03, PERM-04, PERM-05, PERM-06, PERM-07

**Plans:**
1. Serena MCP : service Docker dans docker-compose.yml (`uvx serena start-mcp-server`)
2. ToolRegistry : filtrage de `src/tool_index.py` selon la phase courante
3. `phase-lock.yaml` : mapping déclaratif phase → tools autorisés/retirés
4. Intégration dans `src/agent_loop.py` : phase lue depuis le contexte de session, tools filtrés avant chaque call
5. Tests : vérifier que RESEARCH ne peut pas écrire, VERIFY ne peut pas modifier du code

**Success criteria:**
1. Serena MCP répond sur son socket avec `find_symbol` et `replace_symbol_body`
2. En phase RESEARCH, toute tentative d'écriture de fichier est bloquée avec erreur explicite
3. En phase BUILD, tous les tools sont disponibles
4. Le `phase-lock.yaml` est la seule source de vérité pour le mapping

---

## Phase 4 — Exécution MCP (Scrapling + Supabase + Faker + Kroki)

**Goal:** Brancher les 4 MCP manquants qui complètent la couche d'exécution.

**Requirements:** EXEC-01, EXEC-02, EXEC-03, EXEC-04, EXEC-05

**Plans:**
1. Scrapling MCP : service Docker + intégration dans `src/tool_schemas.py` ; Crawl4AI/Browser-Harness restent OFF
2. Supabase MCP : service Docker en mode read_only par défaut, branching activable par feature
3. Migration partielle SQLite → Postgres pour le state des sessions agents (pas les données utilisateur)
4. Faker.js dans l'environnement de test (`npm i -D @faker-js/faker` dans tests/)
5. Kroki : service Docker (:8000), route `/api/diagram` dans FastAPI

**Success criteria:**
1. `scrapling.extract(url)` retourne du Markdown propre depuis une URL de test
2. Supabase MCP répond en read_only (écriture bloquée sans branching activé)
3. Faker génère des fixtures reproductibles avec `faker.seed(42)` dans un test
4. POST `/api/diagram` avec du Mermaid retourne un SVG valide

---

## Phase 5 — Observations structurées & Budget enforcer

**Goal:** Donner à chaque projet des budgets contraignants et des observations actionnables.

**Requirements:** OBS-01, OBS-02, OBS-03, OBS-04, OBS-05, OBS-06

**Plans:**
1. Schema PROJECT.yaml : objective, done_definition, constraints, budgets (tokens/iter/cost/tools), eval_command, metric
2. `src/budget_enforcer.py` : coupe la loop quand un budget est atteint (hook dans agent_loop.py)
3. Observations structurées après chaque tool call (writer dans src/trace_writer.py)
4. CodeBurn installé (`npm i -g codeburn`) + cron post-session
5. one-shot rate + waste patterns injectés dans `src/observer.py`
6. AgentSeal en CI (`pip install agentseal`, probe mensuelle ou pre-merge)

**Success criteria:**
1. Un projet avec `max_tokens: 10000` s'arrête proprement quand le budget est atteint
2. `codeburn report` produit un rapport avec one-shot rate et liste des waste patterns
3. `agentseal probe` ne détecte aucune vulnérabilité critique sur le projet
4. Les observations JSONL contiennent cost_usd pour chaque tool call

---

## Phase 6 — GSD Workflow Engine

**Goal:** Installer GSD comme moteur de planning et configurer les Pipeline Lists + subagents spécialisés.

**Requirements:** WF-01, WF-02, WF-03, WF-04, WF-05

**Plans:**
1. GSD (gsd-opencode) installé et configuré (ce .planning/ est déjà le début)
2. Pipeline Lists : créer les premiers pipelines WAIT/MOVE/SKIP pour les workflows récurrents
3. Subagents GSD configurés dans `.opencode/agents/` (planner, roadmapper, researcher, executor, verifier, debugger)
4. Wave-based execution : tester le DAG sur la Phase 7 (plans indépendants en parallèle)
5. Stage model assignment : GSD écrit automatiquement la config modèle depuis les stages

**Success criteria:**
1. `/gsd:plan-phase` produit un PLAN.md exploitable sur une phase de test
2. Un Pipeline List `WAIT → MOVE → SKIP` s'exécute automatiquement à la transition de phase
3. Les subagents spawned par GSD utilisent le bon modèle selon le stage
4. Deux plans indépendants tournent en parallèle sans conflit

---

## Phase 7 — Autoeval Loop (Pattern autoresearch)

**Goal:** Implémenter la boucle keep/revert avec eval_command figé et métrique unique.

**Requirements:** EVAL-01, EVAL-02, EVAL-03

**Plans:**
1. `src/autoeval_loop.py` : lit PROJECT.yaml → lance eval_command → compare metric → keep si mieux / revert si pire
2. Intégration dans agent_loop.py : autoeval déclenché après chaque BUILD si eval_command défini
3. Budget iter par run : coupe avant épuisement du budget global, log le score final
4. Test sur un projet pilote avec un eval_command de test (ex: `pytest tests/ -q`, metric = taux de réussite)

**Success criteria:**
1. L'agent modifie du code, l'eval tourne, le score baisse → git revert automatique
2. Le score monte → commit automatique avec le score en message
3. Le budget d'itérations est respecté (pas de loop infinie)
4. Le PROJECT.yaml d'un projet pilote est complet et fonctionnel

---

## Phase 8 — Mémoire : Acontext Self-hosted

**Goal:** Déployer Acontext self-hosted et brancher le pipeline de distillation run→SKILL.md→Obsidian.

**Requirements:** MEM-01, MEM-02, MEM-03, MEM-04, MEM-05, MEM-06

**Plans:**
1. Acontext self-hosted : service Docker dans docker-compose.yml (:8029, Postgres embarqué)
2. Hook post-session dans `src/agent_loop.py` : envoie la session terminée à Acontext
3. Pipeline distillation : Experience Agent → succès/échec détectés → Skill Agent → SKILL.md créé/mis à jour
4. Sortie SKILL.md vers `.opencode/skills/` (natif OpenCode) ET vault Obsidian (`agentos/skills/`)
5. MCP Obsidian second-brain : service Docker exposant le vault en lecture/écriture Markdown
6. Vérifier que ChromaDB RAG docs coexiste sans conflit

**Success criteria:**
1. Acontext répond sur :8029 et accepte des sessions
2. Après une session de code, un SKILL.md est créé dans .opencode/skills/
3. Le vault Obsidian reçoit l'apprentissage dans agentos/learnings/
4. Le MCP Obsidian répond à `get_note` et `create_note`
5. ChromaDB RAG docs fonctionne toujours normalement

---

## Phase 9 — Governance (Patterns Paperclip)

**Goal:** Implémenter goal-ancestry, budgets granulaires, approval gates, et heartbeat dans le Decision Engine.

**Requirements:** GOV-01, GOV-02, GOV-03, GOV-04, GOV-05, GOV-06

**Plans:**
1. Schema DB : tables `missions`, `goals`, `projects`, `tasks` (hiérarchie goal-ancestry) dans `core/database.py`
2. Budget tracker : suivi token+coût par agent/project/provider/model avec seuils d'alerte configurables
3. Auto-pause : quand un agent atteint 100% de son budget, nouvelles tâches bloquées
4. Approval gates : actions destructives → gate tracée hors-prompt + rollback possible (git ou DB)
5. Heartbeat + "Memento Man" : l'agent se réveille, lit sa checklist, contexte injecté explicitement
6. Dashboard Odysseus : nouvelles vues "Objectifs" et "Budgets" dans l'UI (static/app.js)

**Success criteria:**
1. Une tâche porte la chaîne complète mission→goal→project→task dans sa metadata
2. Un agent à 100% de budget est auto-pausé et le dashboard l'affiche
3. Un agent heartbeat se réveille, lit son contexte, et continue là où il s'est arrêté
4. La vue "Objectifs" dans l'UI affiche états et budgets (pas juste des conversations)

---

## Phase 10 — Channel Gateway

**Goal:** Implémenter l'inbound bus + outbound bus avec adapters Discord, Telegram, et MCP connectors outbound.

**Requirements:** CHAN-01, CHAN-02, CHAN-03, CHAN-04, CHAN-05, CHAN-06

**Plans:**
1. `src/channel_gateway.py` : inbound bus (messages → intent classifier → Decision Engine)
2. `src/channel_gateway.py` : outbound bus (résultats → adapters configurés)
3. Discord adapter (`discord.py`) : bot qui reçoit et dispatche les commandes
4. Telegram adapter (`python-telegram-bot`) : bot officiel, sans risque de ban
5. MCP connectors outbound : Gmail, Google Calendar, Slack, Notion, Linear (activables par flag)
6. Route FastAPI `/api/channels/*` pour la config des adapters

**Success criteria:**
1. Un message Discord déclenche une tâche dans le Decision Engine
2. Le résultat de la tâche est renvoyé au canal Discord d'origine
3. Idem avec Telegram
4. Ajouter un adapter = 1 fichier Python + 1 flag dans config, sans toucher au cœur

---

## Phase 11 — Trinité + Decision Engine intégrés

**Goal:** Brancher CBM, Graphify, Obsidian second-brain dans docker-compose, et intégrer le Decision Engine.

**Requirements:** KNO-01, KNO-02, KNO-03, KNO-04, KNO-05, KNO-06

**Plans:**
1. CBM (codebase-memory-mcp) : service Docker dans docker-compose.yml, routes `/api/knowledge/code/*`
2. Graphify : service Docker (on-demand), routes `/api/knowledge/graph/*`
3. Obsidian second-brain MCP : service Docker (voir Phase 8, MEM-05)
4. Routes FastAPI `/api/knowledge/*` qui orchestrent CBM + Graphify + Obsidian
5. Decision Engine depuis `ConfigOpenCodeNew/decision-engine/` : service Docker dans docker-compose.yml
6. Decision Engine = glue : Governance (Phase 9) + Channel Gateway (Phase 10) + observer (Phase 5)

**Success criteria:**
1. `search_graph("agent_loop")` retourne les nodes CBM du projet depuis l'UI
2. `graphify .` indexe le projet et expose le graphe sémantique
3. Le Decision Engine reçoit les events de Governance + Channel Gateway
4. La Trinité est interrogée en checkpoint obligatoire avant toute génération (CBM → Graphify → Obsidian)

---

## Phase 12 — Sandbox durci

**Goal:** Hardener Docker, valider les commandes bash, et déployer AgentSeal en CI.

**Requirements:** SEC-01, SEC-02, SEC-03, SEC-04, SEC-05

**Plans:**
1. `docker-compose.yml` : `cap_drop: ALL` + `no-new-privileges: true` sur services agents (hors Cookbook)
2. s6-overlay dans les Dockerfiles des conteneurs longue durée (HolyClaude patterns)
3. `src/command_validator.py` : scope whitelist + patterns dangereux bloqués avant exec bash
4. AgentSeal CI : probe hebdomadaire + pre-merge sur branches feature
5. Documentation de l'audit prompt-injection + tests automatisés sur les surfaces (skills/notes/docs/mémoires)

**Success criteria:**
1. `docker inspect` confirme cap_drop=ALL sur le conteneur agent
2. Une commande bash hors-whitelist est bloquée avec message d'erreur
3. AgentSeal probe passe sans vulnérabilités critiques
4. Les surfaces non-fiables (skills/notes/docs) sont marquées `[UNTRUSTED]` dans les traces

---

## Phase 13 — Design (Design Extract + Open Design)

**Goal:** Intégrer les deux MCP design et configurer les 259 skills design.

**Requirements:** DSGN-01, DSGN-02, DSGN-03, DSGN-04

**Plans:**
1. Design Extract (designlang) : `npm i -g designlang`, MCP server (`designlang mcp`) dans docker-compose.yml
2. Open Design (nexu-io) : `curl ...install.sh | sh -s opencode`, MCP server configuré
3. 259 skills design depuis Open Design → `.opencode/skills/design/` (SKILL.md par design system)
4. Drift CI : `designlang drift` configuré comme check GitHub Actions (anti-dérive tokens CSS)

**Success criteria:**
1. `designlang extract https://example.com` produit des tokens DTCG valides
2. Open Design génère un composant React depuis un brief texte
3. Les skills design sont listés dans `.opencode/skills/design/`
4. Le drift check échoue si les tokens CSS dérivent de plus de 5%

---

## Phase 14 — Qualité : Debate + Edge-cases + Audit sécurité

**Goal:** Implémenter les 3 agents de qualité Markdown (debate, edge-case gen, security audit).

**Requirements:** QUAL-01, QUAL-02, QUAL-03

**Plans:**
1. Agent Markdown `.opencode/agents/debate-5-personas.md` : pattern vibecode — Architect/Security/Perf/UX/Devil → verdict GO/CAUTION/STOP
2. Agent Markdown `.opencode/agents/edge-case-gen.md` : 12 dimensions → specs de test
3. Agent Markdown `.opencode/agents/security-audit.md` : STRIDE+OWASP, auto-fix par sévérité
4. Intégration dans la loop canonique : debate avant BUILD si complexité élevée, security-audit avant VERIFY

**Success criteria:**
1. Le debate produit un verdict GO/CAUTION/STOP pour une feature de test
2. L'edge-case gen produit ≥ 12 scénarios de test pour une feature donnée
3. Le security-audit détecte une injection SQL intentionnellement introduite dans le code de test
4. Les 3 agents sont invocables depuis l'UI Odysseus (slash commands ou boutons)

---

## Phase 15 — RAG avancé + Bugs amont

**Goal:** RRF hybrid search, BYOX corpus, Obsidian folder-watch, et résoudre les bugs prioritaires du ROADMAP amont.

**Requirements:** RAG-01, RAG-02, RAG-03, BUG-01 à BUG-11

**Plans:**
1. RRF hybrid search dans `src/rag_vector.py` (rank_bm25 + vecteur, fusion par Reciprocal Rank Fusion)
2. BYOX corpus indexé dans `rag/BYOX/` (filtré, pas brut)
3. Obsidian folder-watch : auto-sync vault vers RAG au changement
4. Smoke tests fresh install (script CI pour Linux + macOS + Windows + Docker + WSL)
5. Degraded-state reporting (ChromaDB/SearXNG/email/ntfy → erreurs explicites dans l'UI)
6. Agent prompt/context bloat : profiling + slimmer prompts pour modèles 4k/8k
7. CSS refactor `static/style.css` (< 200KB)
8. Dead code pass + accessibility pass

**Success criteria:**
1. RRF retourne des résultats plus précis que vecteur seul sur 5 requêtes de test
2. Un guide "Build Your Own Redis" est trouvable dans le RAG
3. Smoke test passe sur les 5 environnements cibles
4. Le CSS final est < 200KB et l'UI passe le test d'accessibilité WCAG AA
5. Un modèle 4k (ex: phi-3-mini) peut traiter une requête simple sans dépasser son contexte

---

## Summary

| # | Phase | Priorité | Requirements | Effort |
|---|-------|----------|--------------|--------|
| 1 | Constitution : Loop + Traces | P0 — Harness | CONST-01→05 | M |
| 2 | Model Router LiteLLM | P0 — Harness | ROUT-01→06 | M |
| 3 | Phase-lock & Serena MCP | P0 — Harness | PERM-01→07 | M |
| 4 | Exécution MCP (Scrapling/Supabase/Faker/Kroki) | P1 — Exécution | EXEC-01→05 | M |
| 5 | Observations & Budget enforcer | P1 — Exécution | OBS-01→06 | M |
| 6 | GSD Workflow Engine | P1 — Exécution | WF-01→05 | M |
| 7 | Autoeval Loop | P1 — Exécution | EVAL-01→03 | S |
| 8 | Mémoire : Acontext Self-hosted | P1 — Mémoire | MEM-01→06 | L |
| 9 | Governance (Paperclip patterns) | P2 — Gouvernance | GOV-01→06 | L |
| 10 | Channel Gateway | P2 — Canaux | CHAN-01→06 | M |
| 11 | Trinité + Decision Engine intégrés | P2 — Connaissance | KNO-01→06 | L |
| 12 | Sandbox durci | P2 — Sécurité | SEC-01→05 | M |
| 13 | Design (Design Extract + Open Design) | P3 — Design | DSGN-01→04 | S |
| 14 | Qualité : Debate + Edge-cases + Audit | P3 — Qualité | QUAL-01→03 | S |
| 15 | RAG avancé + Bugs amont | P3 — Finalisation | RAG-01→03, BUG-01→11 | L |

**Légende effort :** S = 1-3 jours · M = 1 semaine · L = 2+ semaines

---
*Created: 2026-06-27*
*Requirements: 76/76 mapped ✓*
