# VAGUE 4 — Automatisation & Workflow (3 agents, ~14h effort)

**Objectif:** Remplacer les adapteurs codés en dur par des solutions d'automatisation visuelles et des pipelines de données.
**Dépendance:** Vague 3 terminée.

---

## A17: n8n — Workflow Automation

**Outil:** n8n (fair-code, self-hosted gratuit, 400+ connecteurs, UI visuelle)
**Prompt:** `prompts/agent-17-n8n.md`
**Durée estimée:** 4h

### Tâches:
1. Ajouter `n8n` au `docker-compose.yml` (profile `automation`)
2. Créer workflows n8n prêts à l'emploi:
   - Email → Résumé → Notification
   - Webhook → Agent → Réponse
   - RSS → Deep Research → Rapport
3. Endpoint webhook: `POST /api/n8n/trigger/{workflow_id}`
4. Documenter templates dans `docs/n8n-workflows.md`
5. Kill-switch: `ODYSSEUS_N8N=off`

---

## A18: Prefect — Data Pipelines

**Outil:** Prefect (Apache 2.0, Python-native, scheduling, caching)
**Prompt:** `prompts/agent-18-prefect.md`
**Durée estimée:** 4h

### Tâches:
1. Ajouter `prefect` à `requirements-optional.txt`
2. Créer `services/pipelines/` avec flows:
   - `rag_reindex_flow` — reindexe tous les documents
   - `memory_cleanup_flow` — nettoie vieilles mémoires
   - `model_update_flow` — vérifie nouveaux modèles Cookbook
   - `backup_flow` — backup data/ vers S3/local
3. Prefect server en Docker Compose (profile `automation`)
4. Kill-switch: `ODYSSEUS_PREFECT=off`

---

## A19: Letta/MemGPT — Mémoire Contextuelle

**Outil:** Letta (Apache 2.0, virtual context management, self-editing memory)
**Prompt:** `prompts/agent-19-letta.md`
**Durée estimée:** 6h

### Tâches:
1. Ajouter `letta` au `docker-compose.yml` (profile `memory`)
2. Créer `services/memory/letta_provider.py`
3. Intégrer comme MemoryProvider dans `src/memory_provider.py`
4. Pagination automatique du contexte (résout bloat 4k/8k/16k)
5. Archival memory pour stockage long-terme
6. Kill-switch: `ODYSSEUS_LETTA=off`

### Bénéfice clé:
Les agents avec petits contextes (4k/8k/16k tokens) pourront utiliser outils+docs+mémoire sans dépasser leur limite de contexte. Letta pagine automatiquement.

---

## CHECKLIST VAGUE 4

- [ ] n8n: 3 workflows templates fonctionnels
- [ ] Prefect: 4 flows testés
- [ ] Letta: pagination contexte fonctionnelle avec modèles 4k
