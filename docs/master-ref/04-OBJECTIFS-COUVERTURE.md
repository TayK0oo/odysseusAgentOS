# Agent OS — Objectifs & Couverture

> **Fusion de :** `PROJECT-2026-06-27.md` + `03-MATRICE-COUVERTURE.md`
> **Date :** 2026-07-27

---

## Core Value

**Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, apprend de chaque run, et ne dépasse jamais ses limites — sans intervention humaine constante.**

Workspace AI self-hosté complet bâti sur OpenCode (fork Odysseus), transformé en système d'agents autonomes de niveau production. Architecture déterministe multi-couches (Connaissance → Exécution → Automatisation → Gouvernance) capable de faire tourner des agents longue durée, bornés par des budgets et des objectifs, avec une mémoire d'apprentissage continue.

---

## Architecture — 5 Couches

```
CONNAISSANCE (Trinité)    → CBM + Graphify + Obsidian
EXÉCUTION                 → Agents, MCP, Sandbox, Routing
DESIGN                    → Extract + Open Design
OBSERVABILITÉ             → Traces, CodeBurn, AutoEval
GOUVERNANCE               → Budgets, Goal-Ancestry, Heartbeat
```

---

## Objectif Final — 8 Axes (Audit de couverture)

### Axe 1 — Orchestration lisible 🟡 57%

> Lancement de plans entiers avec subagents, délégation et spécialisation des tâches au bon agent avec les bons outils.

| Sous-capacité | Couverture |
|---|---|
| Lancement de plans par subagents | 🟡 60% — `CanonicalLoop` 7 phases codé mais non branché au chat live |
| Spécialisation par outil | 🟡 50% — 12 agents `.opencode/` non chargés en live |
| Délégation orchestrée | 🟢 80% — `resolve_model()` fonctionnel |
| Phase-lock par round | 🟡 40% — `PHASE_TRACKER` OFF par défaut |

### Axe 2 — Modularité totale 🟢 82%

> Chaque brique interchangeable sauf le cœur (app.py, agent_loop.py, llm_core.py).

| Sous-capacité | Couverture |
|---|---|
| Services interchangeables | 🟢 85% — 12 dossiers `services/` |
| Routes plug-and-play | 🟢 90% — 53 `include_router` |
| Adapters canaux | 🟢 80% — Discord, Telegram kill-switchés |
| MCP servers interchangeables | 🟡 70% — 7 serveurs, 1 cassé |
| Providers LLM | 🟢 85% — `ModelEndpoint` natif |

### Axe 3 — Routing intelligent des modèles 🟡 50%

> Sélection automatique du bon modèle pour chaque agent en fonction de la tâche.

| Sous-capacité | Couverture |
|---|---|
| Heuristic classifier (tiers) | 🟢 80% — `classify_complexity()` dans ZenRouter |
| Model-to-tier mapping | 🟢 80% — `model-routing.json` |
| Fallback chain | 🟢 80% — `stream_llm_with_fallback()` |
| Per-agent model override | 🟡 20% — Les agents `.opencode/` ont `model:` mais pas utilisés live |
| Blacklist + cooldown | 🟢 100% — Implémenté dans ZenRouter |

### Axe 4 — Connaissance (Trinité) 🟡 53%

> CBM (code graph) + Graphify (concept graph) + Obsidian (second brain).

| Sous-capacité | Couverture |
|---|---|
| CBM intégré | 🟢 90% — MCP server, 11 749 nœuds |
| Graphify intégré | 🟡 30% — MCP server mais non branché dans le pipeline |
| Obsidian MCP | 🟡 10% — Kill-switch OFF, non configuré |
| Trinité dans l'UI | 🔴 0% — Pas de dashboard unifié |

### Axe 5 — Exécution sandboxée 🟢 75%

> Toute exécution de code est isolée, validée, tracée.

| Sous-capacité | Couverture |
|---|---|
| Docker sandbox | 🟢 100% — Profil sécurité dans docker-compose |
| Command validator | 🟢 80% — `tool_security.py`, `command_validator.py` |
| Destructive gate | 🟢 100% — ON par défaut |
| gVisor runtime | 🟡 30% — Configuré mais non activé |

### Axe 6 — Gouvernance 🟡 38%

> Budgets granulaires, goal-ancestry, approval gates, heartbeat.

| Sous-capacité | Couverture |
|---|---|
| Budgets par projet | 🟢 85% — `budget_enforcer.py` opérationnel |
| Goal-ancestry (mission→goal→project→task) | 🟡 30% — Tables SQLAlchemy existent, pas créées en live |
| Approval + rollback | 🟡 20% — Codé dans `saga.py`, non intégré |
| Heartbeat scheduling | 🔴 0% — Non implémenté |

### Axe 7 — Apprentissage continu 🟡 45%

> Distillation automatique des runs en skills, mémoire persistante.

| Sous-capacité | Couverture |
|---|---|
| Distillation run→SKILL.md | 🟡 30% — Acontext externe, hook présent |
| Skills auto-générés | 🟡 40% — `SkillsManager` + audit pipeline |
| Mémoire avec provenance | 🟢 100% — `[stated]/[observed]/[inferred]` (migré vers OpenCode plugins) |
| Boucle keep/revert | 🟡 20% — `autoeval.py` existe, OFF |

### Axe 8 — UI cockpit 🟢 70%

> Tableau de bord live : phase, health, drift, budget, agents.

| Sous-capacité | Couverture |
|---|---|
| Phase bar (C-K-P-B-Q-A-M) | 🟢 100% — cockpit.js, live |
| Health indicator | 🟢 100% — /api/health polling |
| Drift indicator | 🟡 50% — SSE events existent, pas toujours émis |
| Budget indicator | 🟡 30% — Chip présent, données non injectées |
| Agent status | 🟡 40% — Agent indicators dans le chat |

---

## Score Global : 🟡 59%

---

## Exigences Fondatrices (PROJECT.md)

### ✅ Validées (12)
FastAPI orchestrateur, LLM routing avec fallback, MCP servers, Deep Research, Task scheduler, Context compactor, CalDAV/Email, PWA UI, Cookbook, Auth, Claude/Codex integrations, ChromaDB RAG

### 🔨 Actives (28)
Loop canonique + invariants, Phase-lock, Serena MCP, Scrapling MCP, Observations JSONL, CodeBurn, GSD workflow, PROJECT.yaml + autoeval, Acontext, Governance, Channel Gateway, Trinité UI, Decision Engine, Sandbox durci, Design Extract/Open Design, Debate 5 personas, Autoeval loop, RRF hybrid search

### 📋 Planifiées (7)
Trinité branchée dans l'UI, Graphify pipeline, Obsidian second-brain, Heartbeat, Budgets granulaires live, Goal-ancestry live, Multi-agent activé live

---

## Séquence de Build (Constitution)

```
loop manuel → tools → permissions → observations → budgets → tracing
→ planning → context/memory → compaction → skills/connectors
→ goal loop → subagents
```

Ne jamais sauter d'étape.
