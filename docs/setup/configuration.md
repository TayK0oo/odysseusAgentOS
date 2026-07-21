# Setup — Configuration

## Variables d'environnement essentielles

### Application

| Variable | Défaut | Description |
|----------|--------|-------------|
| `APP_PORT` | 7000 | Port de l'application |
| `APP_DATA_DIR` | ./data | Répertoire des données |
| `DATABASE_URL` | sqlite:///./data/app.db | Base de données (SQLite ou PostgreSQL) |
| `AUTH_ENABLED` | true | Activer l'authentification |
| `LOCALHOST_BYPASS` | false | Contourner auth sur localhost |
| `SECURE_COOKIES` | false | Cookies sécurisés (HTTPS) |

### LLM

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | Clé API OpenAI |
| `OLLAMA_BASE_URL` | URL Ollama (défaut: http://127.0.0.1:11434/v1) |
| `LLM_HOST` | Hôte LLM |
| `HF_TOKEN` | Token HuggingFace |

### Recherche

| Variable | Description |
|----------|-------------|
| `SEARXNG_INSTANCE` | URL SearXNG (interne: http://searxng:8080) |
| `BRAVE_API_KEY` | Clé Brave Search |
| `GOOGLE_API_KEY` | Clé Google |
| `TAVILY_API_KEY` | Clé Tavily |
| `SERPER_API_KEY` | Clé Serper |

### Vectorielle

| Variable | Défaut | Description |
|----------|--------|-------------|
| `CHROMADB_HOST` | chromadb | Hôte ChromaDB |
| `CHROMADB_PORT` | 8000 | Port ChromaDB |
| `FASTEMBED_MODEL` | all-MiniLM-L6-v2 | Modèle d'embedding |
| `VECTOR_BACKEND` | chromadb | Backend vectoriel (chromadb/qdrant) |

## Fichiers de configuration

### `model-routing.json` (143 lignes)

Définit le routage des modèles :
- **6 providers** : opencode_zen, ollama, anthropic_claude, openrouter, localai
- **6 modèles** : strong, reasoning, standard, code, creative, fast
- **8 stages** : classify, plan, research, build, quality, autoeval, memory, chat
- **Heuristic** : strong_signals (40 mots, 0.6), weak_signals (8 mots, 0.35)
- **Fallback** : retry sur [400,429,500,502,503,529], cooldown 30s, max 3 tentatives

### `config/phase-lock.yaml` (114 lignes)

Définit 10 phases avec leurs permissions :
```yaml
phases:
  RESEARCH:    {allowed: [READ, SEARCH], blocked: [WRITE, EXEC, DESTRUCTIVE]}
  INNOVATE:    {allowed: [READ, SEARCH, DRAFT], blocked: [EXEC, DESTRUCTIVE]}
  PLAN:        {allowed: [READ, DRAFT], blocked: [EXEC, DESTRUCTIVE], scoped_write: true}
  BUILD:       {allowed: [ALL], blocked: []}
  VERIFY:      {allowed: [READ, TEST], blocked: [WRITE, DESTRUCTIVE]}
  CLASSIFY:    {allowed: [READ], blocked: [WRITE, EXEC, DESTRUCTIVE]}
  KNOW:        {allowed: [READ, SEARCH], blocked: [WRITE, EXEC, DESTRUCTIVE]}
  QUALITY:     {allowed: [READ, TEST], blocked: [WRITE, DESTRUCTIVE]}
  AUTOEVAL:    {allowed: [READ, EVAL], blocked: [DESTRUCTIVE]}
  MEMORY_OBSERVE: {allowed: [READ, WRITE], blocked: [DESTRUCTIVE]}
```

### `config/policies/*.rego` (OPA)

- `tool_access.rego` — autorisation des outils par phase
- `phase_lock.rego` — transitions de phase valides

## Kill-switches (38 au total)

Activation dans `.env` :
```bash
# Orchestration
ODYSSEUS_PHASE_TRACKER=on
ODYSSEUS_LANGGRAPH=on
ODYSSEUS_AUTOEVAL=on

# Services
ODYSSEUS_MEILISEARCH=on
ODYSSEUS_QDRANT=on
ODYSSEUS_OPA=on

# Connaissance
ODYSSEUS_OBSIDIAN_MCP=on

# Qualité
ODYSSEUS_DEEPEVAL=on
ODYSSEUS_CODEBURN=on

# Agents
ODYSSEUS_AGENT_CATALOG=on
ODYSSEUS_AGENT_PLANNER=on
```

---

→ Voir aussi : [Installation](installation.md) · [Prérequis](prerequisites.md) · `model-routing.json` · `config/phase-lock.yaml`
