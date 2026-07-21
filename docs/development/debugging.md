# Development — Debug

## Logs

### Application
```bash
# Docker
docker compose logs odysseus --tail=100 -f

# Natif
tail -f data/logs/app.log
```

### Traces LLM (JSONL)
```bash
cat data/traces/2026-07-*.jsonl | jq .
```

## Observabilité

### Observer (drift)
```bash
curl http://localhost:7000/api/observer/drift
# {"ok":true,"drift_level":"low","runs_tracked":0}
```

### Kill-switches dashboard
```bash
curl http://localhost:7000/api/killswitches
# Liste les 38 switches avec leur statut
```

### LangFuse (si activé)
```bash
docker compose --profile observability up -d
# Dashboard sur http://localhost:3000
```

## Commandes de diagnostic

```bash
# Santé
curl http://localhost:7000/api/health

# Version
curl http://localhost:7000/api/version

# Readiness (DB, data dir)
curl http://localhost:7000/api/ready

# Runtime (Docker, Ollama)
curl http://localhost:7000/api/runtime

# Services Docker
curl http://localhost:7000/api/diagnostics/services

# DB stats
curl http://localhost:7000/api/db/stats

# RAG stats
curl http://localhost:7000/api/rag/stats
```

## Mode debug kill-switches

Pour activer tous les systèmes d'observation :
```bash
export ODYSSEUS_PHASE_TRACKER=on
export ODYSSEUS_LANGGRAPH=on
export ODYSSEUS_AUTOEVAL=on
export ODYSSEUS_CODEBURN=on
export ODYSSEUS_UNIFIED_TOKENS=on
export ODYSSEUS_GOVERNANCE_ANCESTRY=on
export ODYSSEUS_CHECKPOINT=on
export ODYSSEUS_MEM0=on
export ODYSSEUS_RRF_FUSION=on
```

## Points d'arrêt courants

| Symptôme | Cause probable | Solution |
|----------|---------------|----------|
| `stream_agent_loop` ne répond pas | Pas de modèle configuré | Vérifier `OPENAI_API_KEY` ou `OLLAMA_BASE_URL` |
| RAG retourne 503 | ChromaDB down | `docker compose up -d chromadb` |
| Recherche web 502 | SearXNG down | `docker compose up -d searxng` |
| CSS non appliqué | Build Tailwind manquant | `npm run css:build` |
| Erreur d'import module | Dépendance manquante | `pip install -r requirements.txt` |

---

→ Voir aussi : [Tests](testing.md) · [Observabilité](../operations/observability.md) · [Conventions](guidelines.md)
