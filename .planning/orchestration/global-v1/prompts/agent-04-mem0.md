# Agent A4: Mem0 — Agent Memory Layer

## TASK
Integrate Mem0 (Apache 2.0) as an intelligent memory layer that auto-extracts structured facts from conversations, wrapping the existing ChromaDB vector store.

## CONTEXT
- Current: `MemoryVectorStore` stores raw chunks. No structured fact extraction.
- Mem0: Auto-extracts entities, preferences, decisions from conversation text. Consolidates/updates memories.
- Wraps existing vector infrastructure (ChromaDB/Qdrant) — minimal new infrastructure.

## REQUIREMENTS

### 1. Dependency
Add `mem0ai` to `requirements.txt`

### 2. Provider
Create `services/memory/mem0_provider.py`:
```python
from mem0 import Memory

class Mem0Provider:
    def __init__(self, vector_store_config):
        self.memory = Memory.from_config({...})
    
    async def add_conversation(self, session_id, messages):
        """Auto-extract facts from conversation"""
    
    async def search_facts(self, query, user_id):
        """Semantic search across extracted facts"""
    
    async def get_user_profile(self, user_id):
        """Consolidated user preferences and facts"""
```

### 3. Integration
Register Mem0Provider in `src/memory_provider.py` as an additional memory backend.
Add to agent loop: after each round, extract facts via Mem0.

### 4. API
Add to `routes/memory_routes.py`:
- `GET /api/memory/facts?user_id=...` → structured facts
- `GET /api/memory/profile` → consolidated user profile

### 5. Kill-Switch
`ODYSSEUS_MEM0=off` → use raw MemoryVectorStore only

## VERIFICATION
- Converse about preferences → facts extracted
- "What do I prefer?" → Mem0 returns structured preferences
- Existing memory tests pass

## OUTPUT
Files modified, example extracted facts, test results
