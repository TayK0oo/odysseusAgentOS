# Agent A8: OpenTelemetry — Distributed Tracing

## TASK
Add OpenTelemetry (CNCF, Apache 2.0) distributed tracing across the entire Odysseus stack — FastAPI routes, SQLAlchemy queries, LLM calls, tool executions, and MCP server calls.

## CONTEXT
- LangFuse covers LLM-level tracing (Agent A7). OpenTelemetry covers infrastructure-level.
- OTel auto-instruments FastAPI, SQLAlchemy, httpx with zero code changes.
- Manual spans for agent_loop rounds, tool execution, phase transitions.

## REQUIREMENTS

### 1. Dependencies
Add to `requirements.txt`:
```
opentelemetry-api
opentelemetry-sdk
opentelemetry-instrumentation-fastapi
opentelemetry-instrumentation-sqlalchemy
opentelemetry-instrumentation-httpx
opentelemetry-exporter-otlp
```

### 2. Instrumentation
Create `services/observability/otel_setup.py`:
```python
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.instrumentation.sqlalchemy import SQLAlchemyInstrumentor

def setup_otel(app, engine):
    FastAPIInstrumentor.instrument_app(app)
    SQLAlchemyInstrumentor().instrument(engine=engine)
```

### 3. Manual Spans
Add spans in `src/agent_loop.py`:
```python
from opentelemetry import trace
tracer = trace.get_tracer(__name__)

with tracer.start_as_current_span("agent_round") as span:
    span.set_attribute("phase", phase)
    span.set_attribute("round", round_num)
```

### 4. Docker Compose
Add OTel Collector (profile `observability`):
```yaml
otel-collector:
  image: otel/opentelemetry-collector-contrib
  ports: ["4317:4317", "4318:4318"]
  profiles: ["observability"]
```

### 5. Kill-Switch
`ODYSSEUS_OTEL=off` → no tracing overhead

## VERIFICATION
- Traces visible in OTel collector
- Exported to LangFuse (A7) and/or Jaeger
- No performance regression when OFF

## OUTPUT
Files modified, span waterfall example
