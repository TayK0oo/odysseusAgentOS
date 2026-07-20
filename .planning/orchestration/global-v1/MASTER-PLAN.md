# MASTER PLAN — Odysseus AgentOS Global Orchestration v1

**Branch:** `feat/inventaire-global-v1`
**Date:** 2026-07-21
**Status:** IN PROGRESS
**Orchestrator:** OpenAgent (deepseek-v4-pro)

---

## OBJECTIF

Déployer massivement des agents sur TOUS les axes d'amélioration identifiés dans l'inventaire complet du projet :
- Quick Wins (6 outils)
- Observabilité & Qualité (5 outils)
- Refactor Structurel (5 outils)
- Automatisation & Workflow (3 outils)
- Productionisation (3 outils)

## ARCHITECTURE D'EXÉCUTION

```
OpenAgent (Orchestrateur)
  │
  ├── VAGUE 1 - Quick Wins ─────────────────────────────────────
  │   ├── Agent 1: Meilisearch — full-text search
  │   ├── Agent 2: Apprise — notifications unifiées
  │   ├── Agent 3: Docling — traitement documents
  │   ├── Agent 4: Mem0 — mémoire agent
  │   ├── Agent 5: Tailwind CSS — design system
  │   └── Agent 6: gVisor — sandbox kernel
  │
  ├── VAGUE 2 - Observabilité & Qualité ────────────────────────
  │   ├── Agent 7: LangFuse — LLM tracing
  │   ├── Agent 8: OpenTelemetry — distributed tracing
  │   ├── Agent 9: Promptfoo — prompt regression testing
  │   ├── Agent 10: DeepEval — LLM evaluation
  │   └── Agent 11: Ragas — RAG evaluation
  │
  ├── VAGUE 3 - Refactor Structurel ────────────────────────────
  │   ├── Agent 12: LangGraph — décomposition stream_agent_loop
  │   ├── Agent 13: Qdrant — vector DB migration
  │   ├── Agent 14: Traefik — API gateway
  │   ├── Agent 15: OPA — policy-as-code
  │   └── Agent 16: HTMX+Alpine — frontend réactif
  │
  ├── VAGUE 4 - Automatisation & Workflow ─────────────────────
  │   ├── Agent 17: n8n — workflow automation
  │   ├── Agent 18: Prefect — data pipelines
  │   └── Agent 19: Letta/MemGPT — mémoire contextuelle
  │
  └── VAGUE 5 - Productionisation ──────────────────────────────
      ├── Agent 20: PostgreSQL+pgvector — base de données
      ├── Agent 21: LocalAI — LLM unifié
      └── Agent 22: Tree-sitter — parsing code
```

## RÈGLES D'EXÉCUTION

1. **Parallélisme intra-vague** — Tous les agents d'une même vague s'exécutent en parallèle
2. **Séquentialité inter-vagues** — Vague N+1 attend la fin de Vague N
3. **Kill-switch par défaut OFF** — Tout nouvel outil est activable via variable d'env
4. **Zero régression** — Tests passent avant ET après chaque vague
5. **Commit atomique par agent** — 1 commit = 1 outil intégré
6. **Docker Compose profile** — Outils lourds en profile, légers en default

## FICHIERS PRODUITS

| Fichier | Contenu |
|---------|---------|
| `MASTER-PLAN.md` | Ce fichier — orchestration globale |
| `waves/WAVE-1-QUICK-WINS.md` | Plan détaillé Vague 1 |
| `waves/WAVE-2-OBSERVABILITY.md` | Plan détaillé Vague 2 |
| `waves/WAVE-3-REFACTOR.md` | Plan détaillé Vague 3 |
| `waves/WAVE-4-AUTOMATION.md` | Plan détaillé Vague 4 |
| `waves/WAVE-5-PRODUCTION.md` | Plan détaillé Vague 5 |
| `prompts/agent-*.md` | 22 prompts d'agents spécialisés |
| `agents/AGENT-MANIFEST.md` | Catalogue des 22 agents |
| `INVENTAIRE-COMPLET.md` | Synthèse de l'inventaire réalisé |

## MÉTRIQUES CIBLES

| Métrique | Actuel | Cible |
|----------|--------|-------|
| Tests pass rate | 96.7% | ≥97% |
| CSS size | 1.22 MB | <200 KB |
| Observabilité LLM | JSONL traces | LangFuse dashboard |
| Full-text search | SQLite LIKE | Meilisearch <50ms |
| Vector DB SPOF | ChromaDB seul | Qdrant + fallback |
| Kill-switch cohérence | 0 validation | Dashboard santé |
| stream_agent_loop | 3,568 lignes | 7 nœuds LangGraph |
