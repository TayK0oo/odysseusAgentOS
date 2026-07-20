# Agent A19: Letta/MemGPT — Virtual Context Management

## TASK
Integrate Letta (Apache 2.0, formerly MemGPT) as a memory provider with virtual context management — solving the prompt/context bloat problem for small models (4k/8k/16k tokens).

## CONTEXT
- Problem: Agents with small context windows (4k/8k/16k) can't use tools + memory + docs simultaneously — context overflows.
- ROADMAP.md: "Agent prompt/context bloat — slimmer prompts, better tool selection" is a top priority.
- Letta: Virtual context management. Agents have "working memory" (in context) and "archival memory" (database). Letta pages data in/out automatically. Self-editing memory.

## REQUIREMENTS

### 1. Docker Compose
Add Letta (profile `memory`):
```yaml
letta:
  image: letta/letta:latest
  ports: ["127.0.0.1:8283:8283"]
  volumes: [letta-data:/root/.letta]
  profiles: ["memory"]
```

### 2. Python Client
Add `letta-client` to `requirements.txt`.
Create `services/memory/letta_provider.py`:
```python
from letta import create_client

class LettaMemoryProvider:
    def __init__(self, base_url="http://localhost:8283"):
        self.client = create_client(base_url=base_url)
    
    async def create_agent(self, name, system_prompt, tools):
        """Create a Letta agent with virtual context management"""
    
    async def send_message(self, agent_id, message):
        """Letta handles context paging automatically"""
        # Returns response, while managing what's in context vs archival
```

### 3. Memory Provider Integration
Register in `src/memory_provider.py`:
```python
if os.getenv("ODYSSEUS_LETTA") == "on":
    providers.append(LettaMemoryProvider())
```

### 4. Agent Loop Integration
When Letta enabled:
- Each agent round → Letta manages what's in context
- Large documents → auto-archived, retrieved when needed
- Long conversations → older messages auto-archived

### 5. Kill-Switch
`ODYSSEUS_LETTA=off` → standard context management (current)

## VERIFICATION
- Letta agent created via API
- Send 100 messages → context stays within token limit
- Archival memory accessible via search
- 4k context model can use 10+ tools without overflow

## OUTPUT
- `services/memory/letta_provider.py`
- Modified `src/memory_provider.py`
- Demo: 4k model with 10 tools + 100 messages → no overflow
