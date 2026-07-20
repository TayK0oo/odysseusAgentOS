# Agent A20: PostgreSQL + pgvector — Production Database

## TASK
Add PostgreSQL 16 + pgvector as an alternative database backend alongside SQLite, with a migration script and hybrid vector+relational search capability.

## CONTEXT
- Current: SQLite via SQLAlchemy. Works for single-user but has concurrency limits (single writer).
- PostgreSQL: Row-level locking, concurrent access. pgvector extension for embeddings in the same DB.
- Goal: Add PostgreSQL as an option. Not replace SQLite. User chooses via `DATABASE_URL`.

## REQUIREMENTS

### 1. Docker Compose
Add PostgreSQL (profile `production`):
```yaml
postgres:
  image: pgvector/pgvector:pg16
  environment:
    POSTGRES_DB: odysseus
    POSTGRES_USER: odysseus
    POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-odysseus}
  ports: ["127.0.0.1:5432:5432"]
  volumes: [postgres-data:/var/lib/postgresql/data]
  profiles: ["production"]
```

### 2. Python Dependencies
Add to `requirements.txt`:
```
asyncpg
pgvector
```

### 3. Database URL
In `.env.example`:
```bash
# SQLite (default)
DATABASE_URL=sqlite:///./data/app.db

# PostgreSQL (production profile)
# DATABASE_URL=postgresql+asyncpg://odysseus:odysseus@postgres:5432/odysseus
```

### 4. Migration Script
Create `scripts/migrate_sqlite_to_pg.py`:
- Read all tables from SQLite
- Create equivalent schema in PostgreSQL (with pgvector columns)
- Copy all data
- Verify row counts match

### 5. Vector Integration
Replace `chromadb-client` calls with pgvector when PostgreSQL is active:
```python
# With pgvector:
SELECT * FROM documents 
ORDER BY embedding <=> query_vector 
LIMIT 10;
```

### 6. Full-Text Search
PostgreSQL `tsvector` as Meilisearch fallback:
```sql
SELECT * FROM chat_messages 
WHERE to_tsvector('english', content) @@ plainto_tsquery('english', :query)
ORDER BY ts_rank(...) DESC;
```

### 7. Kill-Switch
No kill-switch needed. Database backend is determined by `DATABASE_URL`.
If SQLite URL → SQLite. If PostgreSQL URL → PostgreSQL.

## VERIFICATION
- Migration script runs without errors
- Row counts match between SQLite and PostgreSQL
- Vector search returns same results (±epsilon)
- Existing tests pass with SQLite (no regression)

## OUTPUT
- Migration script
- Modified `core/database.py` (asyncpg support)
- Benchmark: SQLite vs PostgreSQL (concurrent reads/writes)
