# Agent A18: Prefect — Data Pipelines

## TASK
Integrate Prefect (Apache 2.0, Python-native) for scheduled data pipelines — RAG reindexing, memory cleanup, model updates, backups.

## CONTEXT
- Current: `src/task_scheduler.py` (2,285 lines) handles cron-like scheduling. No pipeline observability, no retry logic, no caching.
- Prefect: Python decorators (`@flow`, `@task`). Automatic retry, caching, scheduling. Web dashboard.

## REQUIREMENTS

### 1. Dependency
Add `prefect` to `requirements-optional.txt`

### 2. Pipeline Flows
Create `services/pipelines/`:

```python
# services/pipelines/rag_pipeline.py
from prefect import flow, task

@task(retries=3, cache_key_fn=lambda: "rag-reindex")
def reindex_documents():
    """Reindex all documents in ChromaDB/Qdrant"""
    ...

@flow
def rag_reindex_flow():
    reindex_documents()

# services/pipelines/maintenance_pipeline.py
@flow
def memory_cleanup_flow():
    """Remove old memories, consolidate duplicates"""
    ...

@flow  
def backup_flow():
    """Backup data/ to local/S3"""
    ...
```

### 3. Scheduling
```python
if __name__ == "__main__":
    rag_reindex_flow.serve(
        name="rag-reindex",
        cron="0 2 * * *",  # 2 AM daily
    )
```

### 4. Docker Compose
Prefect server (profile `automation`):
```yaml
prefect-server:
  image: prefecthq/prefect:latest
  ports: ["127.0.0.1:4200:4200"]
  profiles: ["automation"]
```

### 5. Kill-Switch
`ODYSSEUS_PREFECT=off` → use built-in task_scheduler

## VERIFICATION
- Prefect dashboard at `:4200`
- `python -m services.pipelines.rag_pipeline` runs successfully
- Scheduled flows execute on cron

## OUTPUT
- `services/pipelines/*.py` (4 flow files)
- RAG reindex tested
- Flow execution logs
