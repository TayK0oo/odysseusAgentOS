# VAGUE 2 — Observabilité & Qualité (5 agents, ~14h effort)

**Objectif:** Ajouter tracing LLM structuré, évaluation prompts, et métriques RAG.
**Dépendance:** Vague 1 terminée.

---

## A7: LangFuse — LLM Tracing

**Outil:** LangFuse (MIT, self-hosted, traces+coûts+prompt versioning)
**Prompt:** `prompts/agent-07-langfuse.md`
**Durée estimée:** 3h

### Tâches:
1. Ajouter `langfuse` à `docker-compose.yml` (profile `observability`)
2. Ajouter `langfuse` Python SDK à `requirements.txt`
3. Décorer `stream_llm`, `stream_llm_with_fallback`, tool_execution
4. Remplacer `trace_writer.py` (JSONL) par LangFuse
5. Dashboard traces dans l'UI (iframe ou lien)
6. Kill-switch: `ODYSSEUS_LANGFUSE=off`

---

## A8: OpenTelemetry — Distributed Tracing

**Outil:** OpenTelemetry (CNCF, Apache 2.0)
**Prompt:** `prompts/agent-08-opentelemetry.md`
**Durée estimée:** 4h

### Tâches:
1. Ajouter `opentelemetry-*` packages à `requirements.txt`
2. Auto-instrumentation FastAPI, SQLAlchemy, httpx
3. Spans manuels: agent_loop rounds, tool calls, MCP calls
4. OTel Collector en Docker Compose (profile `observability`)
5. Exporter vers LangFuse + Jaeger
6. Kill-switch: `ODYSSEUS_OTEL=off`

---

## A9: Promptfoo — Prompt Regression Testing

**Outil:** Promptfoo (MIT, YAML-based, CI-friendly)
**Prompt:** `prompts/agent-09-promptfoo.md`
**Durée estimée:** 2h

### Tâches:
1. Ajouter `promptfoo` à `package.json` devDependencies
2. Créer `tests/prompts/promptfooconfig.yaml`
3. Test cases: system prompt, tool selection, refusal patterns
4. Ajouter GitHub Actions job `prompt-eval`
5. Documenter dans `docs/testing.md`

---

## A10: DeepEval — LLM Evaluation

**Outil:** DeepEval (Apache 2.0, pytest-native, 14+ métriques)
**Prompt:** `prompts/agent-10-deepeval.md`
**Durée estimée:** 3h

### Tâches:
1. Ajouter `deepeval` à `requirements-optional.txt`
2. Créer `tests/quality/test_llm_quality.py`
3. Métriques: faithfulness, relevancy, hallucination, bias, toxicity
4. CI integration: `deepeval test run` dans GitHub Actions
5. Kill-switch: `ODYSSEUS_DEEPEVAL=off`

---

## A11: Ragas — RAG Evaluation

**Outil:** Ragas (Apache 2.0, RAG-specific metrics)
**Prompt:** `prompts/agent-11-ragas.md`
**Durée estimée:** 2h

### Tâches:
1. Ajouter `ragas` à `requirements-optional.txt`
2. Créer `tests/quality/test_rag_quality.py`
3. Métriques: context precision/recall, faithfulness, relevancy
4. Benchmark RRF vs naive blend
5. Génération synthetic test cases

---

## CHECKLIST VAGUE 2

- [ ] LangFuse dashboard accessible sur `:3000`
- [ ] Traces LLM visibles dans LangFuse UI
- [ ] OpenTelemetry spans exportés
- [ ] Promptfoo passe en CI
- [ ] DeepEval metrics > seuils définis
- [ ] Ragas confirme RRF > naive blend
