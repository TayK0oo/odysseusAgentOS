# Agent A1: Meilisearch — Full-Text Search Integration

## TASK
Integrate Meilisearch (MIT, Rust-based, typo-tolerant search engine) into Odysseus AgentOS as a Docker Compose service with full-text search across chat messages, notes, and documents.

## CONTEXT
- Project: odysseusAgentOS (Python 3.14, FastAPI, SQLAlchemy, Docker Compose)
- Current search: SQLite LIKE queries in `routes/search_routes.py` — no fuzzy matching, no typo tolerance
- Branch: `feat/inventaire-global-v1`
- Working directory: `C:\Users\ttmdu\Documents\GitHub\odysseusAgentOS`

## REQUIREMENTS

### 1. Docker Compose
Add Meilisearch service to `docker-compose.yml`:
```yaml
meilisearch:
  image: getmeili/meilisearch:v1.14
  restart: unless-stopped
  environment:
    MEILI_MASTER_KEY: ${MEILI_MASTER_KEY:-odysseus-meili-key}
    MEILI_ENV: development
  ports:
    - "127.0.0.1:7700:7700"
  volumes:
    - meilisearch-data:/meili_data
  profiles: ["default"]
```

### 2. Python Client
Create `services/search/meilisearch_client.py`:
- Async client using `meilisearch` Python SDK
- Methods: `index_message()`, `index_note()`, `index_document()`, `search_all()`
- Auto-index on message save, note create, document upload
- Typo-tolerant search with facets (type, session, date)

### 3. API Endpoint
Add to `routes/search_routes.py`:
- `GET /api/search/fulltext?q=...&type=...` → returns ranked results with highlights

### 4. Frontend
Enhance `static/js/search-chat.js`:
- Omnibox search bar in top nav
- Results dropdown with typo-corrected suggestions

### 5. Kill-Switch
- `ODYSSEUS_MEILISEARCH=off` → fallback to SQLite LIKE
- Document in `.env.example`

## VERIFICATION
- `docker compose up` starts meilisearch without error
- `curl http://localhost:7700/health` returns 200
- `pytest tests/ -k "search"` passes
- Typo "restaurent" finds "restaurant" in chat messages

## OUTPUT
- List of files created/modified
- Git diff summary
- Any issues encountered
