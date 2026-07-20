# AGENT MANIFEST — 22 Agents Déployables

**Fichier maître listant tous les agents disponibles pour l'orchestration globale v1.**

---

## AGENTS DE LA VAGUE 1 — Quick Wins

| ID | Agent | Outil | Licence | Effort | Impact |
|----|-------|-------|---------|--------|--------|
| A1 | meilisearch-agent | Meilisearch | MIT | 2h | ⭐⭐⭐⭐⭐ |
| A2 | apprise-agent | Apprise | BSD | 1h | ⭐⭐⭐⭐⭐ |
| A3 | docling-agent | Docling (IBM) | MIT | 3h | ⭐⭐⭐⭐ |
| A4 | mem0-agent | Mem0 | Apache 2.0 | 2h | ⭐⭐⭐⭐ |
| A5 | tailwind-agent | Tailwind CSS | MIT | 4h | ⭐⭐⭐⭐ |
| A6 | gvisor-agent | gVisor (Google) | Apache 2.0 | 1h | ⭐⭐⭐⭐ |

## AGENTS DE LA VAGUE 2 — Observabilité & Qualité

| ID | Agent | Outil | Licence | Effort | Impact |
|----|-------|-------|---------|--------|--------|
| A7 | langfuse-agent | LangFuse | MIT | 3h | ⭐⭐⭐⭐⭐ |
| A8 | otel-agent | OpenTelemetry | Apache 2.0 | 4h | ⭐⭐⭐ |
| A9 | promptfoo-agent | Promptfoo | MIT | 2h | ⭐⭐⭐ |
| A10 | deepeval-agent | DeepEval | Apache 2.0 | 3h | ⭐⭐⭐ |
| A11 | ragas-agent | Ragas | Apache 2.0 | 2h | ⭐⭐⭐ |

## AGENTS DE LA VAGUE 3 — Refactor Structurel

| ID | Agent | Outil | Licence | Effort | Impact |
|----|-------|-------|---------|--------|--------|
| A12 | langgraph-agent | LangGraph | MIT | 16h | ⭐⭐⭐⭐⭐ |
| A13 | qdrant-agent | Qdrant | Apache 2.0 | 4h | ⭐⭐⭐ |
| A14 | traefik-agent | Traefik | MIT | 3h | ⭐⭐⭐ |
| A15 | opa-agent | OPA (CNCF) | Apache 2.0 | 6h | ⭐⭐⭐ |
| A16 | htmx-alpine-agent | HTMX + Alpine.js | BSD/MIT | 8h | ⭐⭐⭐⭐ |

## AGENTS DE LA VAGUE 4 — Automatisation & Workflow

| ID | Agent | Outil | Licence | Effort | Impact |
|----|-------|-------|---------|--------|--------|
| A17 | n8n-agent | n8n | fair-code | 4h | ⭐⭐⭐⭐ |
| A18 | prefect-agent | Prefect | Apache 2.0 | 4h | ⭐⭐⭐ |
| A19 | letta-agent | Letta/MemGPT | Apache 2.0 | 6h | ⭐⭐⭐⭐ |

## AGENTS DE LA VAGUE 5 — Productionisation

| ID | Agent | Outil | Licence | Effort | Impact |
|----|-------|-------|---------|--------|--------|
| A20 | postgres-agent | PostgreSQL+pgvector | PostgreSQL | 4h | ⭐⭐⭐ |
| A21 | localai-agent | LocalAI | MIT | 3h | ⭐⭐⭐ |
| A22 | treesitter-agent | Tree-sitter | MIT | 4h | ⭐⭐ |

---

## RÈGLES DE DÉPLOIEMENT

1. Chaque agent lit son prompt dans `prompts/agent-XX-*.md`
2. Chaque agent produit un plan d'intégration AVANT d'écrire du code
3. Chaque agent commit atomiquement (1 commit = 1 outil)
4. Chaque agent vérifie que les tests passent après son travail
5. Tout nouvel outil est activable via `ENABLE_*` dans `.env` et `docker-compose.yml`
6. Tout nouvel outil a un kill-switch OFF par défaut
7. Les outils lourds (>500MB RAM) vont en profile Docker Compose

## MODÈLES PAR AGENT

| Complexité | Modèle | Agents |
|-----------|--------|--------|
| TRIVIAL | deepseek-v4-flash | A2, A6 |
| STANDARD | deepseek-v4-pro | A1, A3, A4, A5, A9, A10, A11, A14, A20, A21, A22 |
| COMPLEX | kimi-k2.6 (reasoning) | A7, A8, A12, A13, A15, A16, A17, A18, A19 |
