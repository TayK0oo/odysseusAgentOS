# Agent A7: LangFuse — LLM Tracing & Observability

## TASK
Integrate LangFuse (MIT, self-hosted) for structured LLM tracing — replacing the current JSONL-based `trace_writer.py` with a full observability dashboard showing traces, token usage, costs, and prompt versions.

## CONTEXT
- Current: `trace_writer.py` writes JSONL files. No dashboard, no search, no cost tracking.
- LangFuse: Open source (MIT), self-hosted Docker, Python SDK with `@observe()` decorator.
- Solves: "What did the LLM actually say?" "How much did this session cost?" "Which prompt version performed best?"

## REQUIREMENTS

### 1. Docker Compose
Add LangFuse to `docker-compose.yml` (profile `observability`):
```yaml
langfuse-server:
  image: ghcr.io/langfuse/langfuse:latest
  ports: ["127.0.0.1:3000:3000"]
  environment:
    DATABASE_URL: postgresql://...
    NEXTAUTH_SECRET: ...
  profiles: ["observability"]
```

### 2. Python SDK
Add `langfuse` to `requirements.txt`. Create `services/observability/langfuse_tracer.py`:
```python
from langfuse import Langfuse
from langfuse.decorators import observe, langfuse_context

@observe()
async def trace_llm_call(model, messages, **kwargs):
    # Automatically captures: input, output, tokens, latency, cost
    ...
```

### 3. Integration Points
Decorate with `@observe()`:
- `src/llm_core.py::stream_llm` — capture all LLM calls
- `src/llm_core.py::stream_llm_with_fallback` — capture fallback chains
- `src/tool_execution.py::execute_tool` — capture tool usage
- `src/agent_loop.py::stream_agent_loop` — capture full agent rounds

### 4. Trace Context
Pass `session_id` and `run_id` as trace metadata:
```python
langfuse_context.update_current_trace(
    session_id=session_id,
    user_id=user_id,
    tags=["agent", phase]
)
```

### 5. Kill-Switch
`ODYSSEUS_LANGFUSE=off` → no tracing (preserves current behavior)
When ON → traces go to LangFuse instead of JSONL

### 6. Documentation
Add `docs/observability.md` with LangFuse setup and dashboard access instructions.

## VERIFICATION
- `docker compose --profile observability up` starts LangFuse
- Run a chat → trace visible in LangFuse dashboard
- Token counts and costs are accurate
- Existing tests pass (LangFuse OFF by default in tests)

## OUTPUT
Files modified, dashboard screenshot URL, example trace
