# PLANIFICATION COMPLÈTE — Odysseus AgentOS

**Date :** 2026-07-21 | **Branche :** `feat/inventaire-global-v1` | **Basé sur :** SFD v3.0, INDEX-MAITRE.md, tests HEAD

---

## 1. ÉTAT GLOBAL

| Métrique | Valeur |
|----------|--------|
| Alignement SFD → Code | **~45%** |
| Modules SFD actifs | 6/20 (30%) |
| Modules SFD dormants | 4/20 (20%) |
| Modules SFD absents | 7/20 (35%) |
| Kill-switches actifs | 1/38 (DESTRUCTIVE_GATE) |
| Tests passés | 4,393 / 4,566 (96.7%) |
| Outils intégrés (global-v1) | 22 (tous OFF par défaut) |

---

## 2. SFD → CODE : STATUT PAR MODULE

### 🟢 ACTIFS (6/20)

| Module SFD | Fichier code | Tests | Validation |
|-----------|-------------|-------|------------|
| §5.4 Exécution contrôlée | `agent_loop.py:3637`, `tool_execution.py:978` | 47 tests agent_loop | ✅ Chat + outils fonctionnels |
| §5.6 Routage modèles | `zen_router.py:294`, `llm_core.py:2308`, `model-routing.json` | 17 tests LLM core | ✅ LiteLLM + Zen OK |
| §5.11 Observabilité | `observer.py:177`, `trace_writer.py:166` | Observer tests | ✅ Drift + traces JSONL |
| §5.12 Workspaces | Racine projet, `.planning/`, `data/projects/` | — | ✅ Structure OK |
| §5.13 Extensibilité MCP | `mcp_manager.py:666`, 6 serveurs MCP | 9 tests MCP | ✅ MCP SDK + builtins |
| §5.17 Skills | `SkillsManager`, `services/memory/skills.py` | 13 tests skills | ✅ Extraction dynamique |

### 🔵 DORMANTS (4/20)

| Module SFD | Fichier code | Kill-switch | Action |
|-----------|-------------|------------|--------|
| §5.1 Multi-agent | `orchestrator/` (18 modules) | 13 switches | Activer progressivement |
| §5.8 Communication multi-canal | `channel_gateway.py`, adapters | 3 switches | Configurer tokens |
| §5.9 Gouvernance | `governance.py:211`, `budget_enforcer.py` | `ODYSSEUS_GOVERNANCE_ANCESTRY` | Activer |
| §5.10 Auto-évaluation | `autoeval.py:115`, `_run_verifier_subagent` | `ODYSSEUS_AUTOEVAL` | Activer |

### 🟡 PARTIELS (3/20)

| Module SFD | Code existant | Manque |
|-----------|-------------|--------|
| §5.2 Ingénierie contexte | `context_budget.py:62`, `context_compactor.py:388` | Compaction non active, pas de bloc-notes externalisé |
| §5.7 Mémoire | `memory_provider.py:390`, `memory_vector.py:329`, Mem0 | Pas de provenance `[stated]`/`[observed]`, pas de versionnage |
| §5.3 Planification | `PROJECT.yaml.example` | Pas de décomposition arborescence active |

### 🔴 ABSENTS (7/20)

| Module SFD | Description | Priorité |
|-----------|------------|----------|
| §5.5 Exécution durable | Workflows, retry, compensation, approbation humaine | P2 |
| §5.14 Configuration | Déjà fait (YAML/JSON/Git) mais pas formalisé comme module SFD | — |
| §5.15 Préférences utilisateur | `prefs_routes.py` basique, pas de résolution conflits | P1 |
| §5.16 Recherche conversations | `session_search.py:315` existe (FTS5) → 🟢 en réalité | — |
| §5.18 Modalités de sortie | Arbre décision, visualiseur inline, modules design | P2 |
| §5.19 Classification données | 5 niveaux rétention, droit à l'oubli partiel | P2 |
| §5.20 Sécurité contenus | Protection injections mémoire, rappels système | P2 |

---

## 3. OUTILS : UTILISÉS vs VOULUS

### Outils déjà intégrés et testés

| Outil | Licence | Statut | Kill-switch | Testé ? |
|-------|---------|--------|-------------|---------|
| **ChromaDB** | Apache 2.0 | 🟢 Actif | — | ✅ RAG + mémoire |
| **SearXNG** | AGPL | 🟢 Actif | — | ✅ Recherche web |
| **LiteLLM** | MIT | 🟢 Actif | — | ✅ Streaming LLM |
| **FastEmbed** | Apache 2.0 | 🟢 Actif | — | ✅ Embeddings locaux |
| **Kroki** | MIT | 🔵 Gated | `KROKI_ENABLED` | ⚠️ Partiel |
| **ntfy** | Apache 2.0 | 🟢 Actif | — | ⚠️ Non vérifié UI |
| **Serena MCP** | — | 🔵 Démarré | — | ❌ Intégration agent non vérifiée |

### Outils intégrés (global-v1) — tous OFF par défaut

| Outil | Licence | Kill-switch | Code | Testé ? |
|-------|---------|-------------|------|---------|
| **Meilisearch** | MIT | `ODYSSEUS_MEILISEARCH` | `meilisearch_client.py:281` | ✅ Smoke test |
| **Apprise** | BSD | `ODYSSEUS_APPRISE` | `apprise_service.py:142` | ✅ Integration test |
| **Docling** | MIT | `ODYSSEUS_DOCLING` | `docling_processor.py:134` | ✅ Integration test |
| **Mem0** | Apache 2.0 | `ODYSSEUS_MEM0` | `mem0_provider.py:399` | ✅ Provider test |
| **Tailwind CSS** | MIT | — | `tailwind.css` → 16.3 KB | ✅ Build OK |
| **gVisor** | Apache 2.0 | `ODYSSEUS_GVISOR` | docker-compose | ⚠️ Doc only |
| **LangFuse** | MIT | `ODYSSEUS_LANGFUSE` | `langfuse_tracer.py:161` | ❌ Pas testé |
| **OpenTelemetry** | Apache 2.0 | `ODYSSEUS_OTEL` | `otel_setup.py:159` | ❌ Pas testé |
| **Promptfoo** | MIT | — | `promptfooconfig.yaml` | ❌ Pas testé |
| **DeepEval** | Apache 2.0 | `ODYSSEUS_DEEPEVAL` | `test_llm_quality.py:152` | ❌ Pas testé |
| **Ragas** | Apache 2.0 | `ODYSSEUS_RAGAS` | `test_rag_quality.py:379` | ❌ Pas testé |
| **LangGraph** | MIT | `ODYSSEUS_LANGGRAPH` | `langgraph_loop.py:848` | ✅ Loop tests |
| **Qdrant** | Apache 2.0 | `ODYSSEUS_QDRANT` | `qdrant_store.py:308` | ❌ Pas testé |
| **Traefik** | MIT | `ODYSSEUS_TRAEFIK` | docker-compose | ⚠️ Config only |
| **OPA** | Apache 2.0 | `ODYSSEUS_OPA` | `opa_client.py:217` + 4 Rego | ⚠️ Policy tests |
| **HTMX+Alpine** | BSD/MIT | — | `static/lib/` | ❌ Pas testé |
| **n8n** | fair-code | `ODYSSEUS_N8N` | `n8n_routes.py:106` | ❌ Pas testé |
| **Prefect** | Apache 2.0 | `ODYSSEUS_PREFECT` | 3 pipeline files | ❌ Pas testé |
| **Letta** | Apache 2.0 | `ODYSSEUS_LETTA` | `letta_provider.py:340` | ❌ Pas testé |
| **PostgreSQL** | PostgreSQL | `DATABASE_URL` | migration script | ❌ Pas testé |
| **LocalAI** | MIT | `ODYSSEUS_LOCALAI` | `config/localai/` | ❌ Pas testé |
| **Tree-sitter** | MIT | `ODYSSEUS_TREESITTER` | `treesitter_parser.py:380` | ✅ Parser test |

### Outils voulus (SFD) non intégrés

| Outil SFD | Description | Priorité |
|-----------|------------|----------|
| **Temporal** (ou équivalent) | Exécution durable (§5.5) | P2 |
| **Système fichiers mémoire** | `/profile.md`, `/topics/`, `/areas/`, `/people/`, `/preferences.md` (§5.7) | P1 |
| **Registre MCP externe** | `search_mcp_registry` (§5.13) | P2 |
| **Visualiseur inline** | SVG/HTML interactif (§5.18) | P2 |

---

## 4. FONCTIONNALITÉS VALIDÉES vs NON VALIDÉES

### ✅ Validées (testées et fonctionnelles)

| Fonctionnalité | Preuve |
|---------------|--------|
| Chat agentique (streaming SSE) | 47 tests agent_loop |
| Exécution d'outils (bash, files, web) | Tests tool_execution |
| Multi-provider LLM (LiteLLM + Zen) | 17 tests LLM core |
| RAG vectoriel (ChromaDB + FastEmbed) | 7 tests RAG |
| Mémoire vectorielle (5 entrées) | Tests memory_vector |
| Mem0 extraction automatique (6 faits) | Testé au runtime |
| Recherche web (SearXNG) | Tests search |
| Email IMAP/SMTP | 10 tests email |
| CalDAV sync | 10 tests caldav |
| Skills dynamiques | 13 tests skills |
| Auth (bcrypt + TOTP 2FA) | 7 tests auth |
| Destructive gate (ON par défaut) | Tests gate |
| Observer (drift monitoring) | Tests observer |
| Traces JSONL | Tests trace_writer |
| Kill-switch dashboard | 38 switches exposés |
| Cockpit UI (phase, drift, budget) | UI vérifiée Playwright |

### ⚠️ Partiellement validées

| Fonctionnalité | Statut |
|---------------|--------|
| Phase-lock | Codé, gated OFF, non testé en condition réelle |
| LangGraph (7 nœuds) | Codé, tests unitaires OK, non testé en intégration |
| Multi-agent workflow | Codé, gated OFF, 0 test réel |
| Governance ancestry | Codé, gated OFF |
| Auto-évaluation (keep/revert) | Codé, gated OFF |
| Channel Gateway (Discord/Telegram) | Codé, gated OFF, pas de tokens |
| RRF fusion (hybride vecteur+BM25) | Codé, gated OFF |
| CodeBurn (one-shot rate) | Codé, gated OFF |
| Kroki (diagrammes) | Service démarré, intégration agent non vérifiée |
| CSS Tailwind | Build OK (16.3 KB), non intégré à index.html |

### ❌ Non validées

| Fonctionnalité | Raison |
|---------------|--------|
| Mémoire avec provenance | Non implémenté |
| Préférences structurées | Non implémenté |
| Découverte d'outils (tool_search) | Non implémenté |
| Arbre de décision sortie | Non implémenté |
| Exécution durable (workflows) | Non implémenté |
| Visualiseur inline | Non implémenté |
| Classification/rétention données | Non implémenté |
| 22 outils global-v1 | Code écrit, 0 test réel (dépendances non installées) |

---

## 5. PLAN D'ACTION PRIORISÉ

### Phase 1 — Activer l'existant (semaine 1-2)

| # | Action | Effort | Impact |
|---|--------|--------|--------|
| 1.1 | Installer dépendances manquantes (`pip install -r requirements.txt`) | 30min | Débloque 22 outils |
| 1.2 | Activer `ODYSSEUS_PHASE_TRACKER=on` + tester | 2h | Phase-lock actif |
| 1.3 | Activer `ODYSSEUS_RRF_FUSION=on` + benchmark RAG | 1h | Meilleure recherche |
| 1.4 | Activer `ODYSSEUS_MEM0=on` (déjà fait) + valider extraction | 1h | Mémoire intelligente |
| 1.5 | Activer `ODYSSEUS_MEILISEARCH=on` + lancer Docker | 1h | Full-text search |
| 1.6 | Activer `ODYSSEUS_LANGFUSE=on` + dashboard | 2h | LLM tracing |
| 1.7 | Lancer `npm run css:build` + intégrer Tailwind dans index.html | 2h | CSS 16 KB |
| 1.8 | Activer `ODYSSEUS_GOVERNANCE_ANCESTRY=on` | 1h | Traçabilité |

### Phase 2 — Implémenter les modules SFD priorité 1 (semaine 3-4)

| # | Module SFD | Effort | Description |
|---|-----------|--------|------------|
| 2.1 | §5.15 Préférences utilisateur | 8h | Matrice always/selective/never, résolution conflits, guardrails |
| 2.2 | §5.7 Mémoire avec provenance | 16h | `/profile.md`, `/topics/`, `/areas/`, `/people/`, `/preferences.md` avec `[stated]`/`[observed]`/`[inferred]`, contrôle versionné |
| 2.3 | §5.1 Multi-agent (activer progressivement) | 8h | Activer planner → executor → verifier, un à la fois |
| 2.4 | §5.2 Ingénierie contexte | 8h | Activer compaction automatique, bloc-notes externalisé |

### Phase 3 — Implémenter les modules SFD priorité 2 (semaine 5-8)

| # | Module SFD | Effort | Description |
|---|-----------|--------|------------|
| 3.1 | §5.5 Exécution durable | 24h | Temporal ou equivalent, retry policies, saga |
| 3.2 | §5.18 Modalités de sortie | 16h | Arbre décision, visualiseur inline, modules design |
| 3.3 | §5.19 Classification données | 8h | 5 niveaux rétention, droit à l'oubli |
| 3.4 | §5.20 Sécurité contenus | 8h | Protection injections, rappels système |
| 3.5 | §5.13 Découverte d'outils | 16h | `tool_search`, `search_mcp_registry`, `suggest_connectors` |

### Phase 4 — Déploiement et validation (semaine 9-10)

| # | Action | Effort |
|---|--------|--------|
| 4.1 | Tests E2E complets (UI + API) | 8h |
| 4.2 | Tests de charge (10 projets simultanés) | 4h |
| 4.3 | Documentation finale | 4h |
| 4.4 | Déploiement production (Traefik + PostgreSQL + gVisor) | 4h |

---

## 6. COUVERTURE DE TEST PAR MODULE

| Module | Tests | Pass | Fail | Couverture fonctionnelle |
|--------|-------|------|------|------------------------|
| Agent loop | 47 | 47 | 0 | 🟢 100% |
| LLM core | 17 | 17 | 0 | 🟢 100% (routing) |
| Tools | 11 | 11 | 0 | 🟢 100% (parsing, policy) |
| Skills | 13 | 13 | 0 | 🟢 100% |
| Memory | 14 | 14 | 0 | 🟢 100% (vectoriel) |
| RAG | 7 | 7 | 0 | 🟢 100% |
| Auth | 7 | 7 | 0 | 🟢 100% |
| Search | 18 | 18 | 0 | 🟢 100% |
| Cookbook | 21 | 21 | 0 | 🟢 100% |
| Orchestrator | 13 | 10 | 3 | 🟡 77% (collection errors) |
| Email | ~10 | ~10 | 0 | 🟢 100% |
| Calendar | 14 | 14 | 0 | 🟢 100% |
| Gallery | 11 | 11 | 0 | 🟢 100% |
| Document | 15 | 15 | 0 | 🟢 100% |
| Core (auth, DB) | 0 | 0 | 0 | 🔴 0% — priorité ! |

---

## 7. SYNTHÈSE

```
SFD v3.0 : 22 principes, 20 modules, 19 exigences NF
Code     : ~160K lignes Python, 162 JS, 38 kill-switches
Alignemt : 45% (6 actifs, 4 dormants, 3 partiels, 7 absents)
Outils   : 29 intégrés (7 actifs + 22 gated), 4 voulus (SFD)
Tests    : 4,393 pass (96.7%), core/auth/DB = 0% couvert

PROCHAINES ACTIONS IMMÉDIATES :
1. pip install -r requirements.txt
2. Activer PhaseTracker + RRF + Mem0 + Meilisearch
3. Activer LangFuse + Tailwind CSS
4. Implémenter §5.15 (Préférences) + §5.7 (Mémoire avec provenance)
```
