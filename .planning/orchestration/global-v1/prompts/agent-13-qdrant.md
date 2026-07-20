# Agent A13: Qdrant — Vector Database Migration

## TASK
Add Qdrant (Apache 2.0, Rust) as a production-grade vector database alongside ChromaDB, with a migration script and configurable backend selection.

## CONTEXT
- ChromaDB is a single-node Python vector DB. Works well but is a SPOF and lacks advanced filtering.
- Qdrant: Rust (10-100x faster), payload filtering, quantization (4-16x RAM reduction), HA consensus.
- Goal: Add Qdrant as an option WITHOUT removing ChromaDB.

## REQUIREMENTS

### 1. Docker Compose
Add Qdrant (profile `vectordb`):
```yaml
qdrant:
  image: qdrant/qdrant:latest
  ports: ["127.0.0.1:6333:6333"]
  volumes: [qdrant-data:/qdrant/storage]
  profiles: ["vectordb"]
```

### 2. Python Client
Add `qdrant-client` to `requirements.txt`.
Create `services/vector/qdrant_store.py`:
```python
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, Filter, FieldCondition

class QdrantVectorStore:
    def __init__(self, url="http://localhost:6333"):
        self.client = QdrantClient(url=url)
    
    async def create_collection(self, name, vector_size):
        self.client.create_collection(
            collection_name=name,
            vectors_config=VectorParams(size=vector_size, distance=Distance.COSINE)
        )
    
    async def upsert(self, collection, points):
        self.client.upsert(collection_name=collection, points=points)
    
    async def search(self, collection, vector, limit=10, filter=None):
        return self.client.search(
            collection_name=collection,
            query_vector=vector,
            limit=limit,
            query_filter=filter
        )
```

### 3. Backend Config
In `src/config.py`:
```python
VECTOR_BACKEND: Literal["chromadb", "qdrant"] = "chromadb"
```

### 4. Migration Script
Create `scripts/migrate_chromadb_to_qdrant.py`:
- Read all collections from ChromaDB
- Write to Qdrant with same structure
- Verify: random sample search comparisons

### 5. Integration
Modify `src/rag_vector.py` and `src/memory_vector.py` to use `VECTOR_BACKEND` config.
Keep ChromaDB as default, Qdrant as opt-in.

### 6. Kill-Switch
`ODYSSEUS_QDRANT=off` → use ChromaDB

## VERIFICATION
- `docker compose --profile vectordb up` starts Qdrant
- Migration script copies all vectors
- Search results identical (±epsilon) between ChromaDB and Qdrant
- Existing RAG tests pass

## OUTPUT
Files modified, migration script, benchmark: Qdrant vs ChromaDB (latency, recall@10)
