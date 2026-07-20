# Agent A21: LocalAI — Unified Local LLM

## TASK
Add LocalAI (MIT) as an all-in-one local AI backend — replacing 4 separate services (Ollama, fastembed, TTS, STT, diffusion) with a single Docker container.

## CONTEXT
- Current: 4+ separate services for local AI: Ollama (LLM), fastembed (embeddings), TTS service, STT service, diffusion server.
- LocalAI: One container that does everything: LLM (llama.cpp), embeddings, TTS (whisper/vall-e-x), STT, image generation (stable diffusion). OpenAI-compatible API.
- Benefit: Dramatically simpler docker-compose for local deployments. One API surface.

## REQUIREMENTS

### 1. Docker Compose
Add LocalAI (profile `localai`):
```yaml
localai:
  image: localai/localai:latest
  ports: ["127.0.0.1:8081:8080"]
  environment:
    MODELS_PATH: /models
    THREADS: ${LOCALAI_THREADS:-4}
  volumes:
    - localai-models:/models
  profiles: ["localai"]
```

### 2. Model Configuration
Create `config/localai/models/` with YAML configs:
```yaml
# config/localai/models/llama.yaml
name: llama
backend: llama-cpp
parameters:
  model: llama-3.2-3b-instruct.Q4_K_M.gguf

# config/localai/models/embeddings.yaml
name: text-embedding
backend: sentencetransformers

# config/localai/models/tts.yaml
name: tts-1
backend: piper
```

### 3. LiteLLM Integration
Add LocalAI as a provider in `model-routing.json`:
```json
{
  "providers": {
    "localai": {
      "base_url": "http://localai:8080/v1",
      "api_key": "not-needed"
    }
  }
}
```

### 4. Kill-Switch
`ODYSSEUS_LOCALAI=off` → use individual services (Ollama, fastembed, etc.)

## VERIFICATION
- LocalAI container serves OpenAI-compatible API
- `curl http://localhost:8081/v1/models` returns model list
- Chat completion works via LiteLLM → LocalAI
- Embeddings endpoint returns vectors
- Existing tests pass (Ollama still default)

## OUTPUT
- LocalAI docker-compose service
- Model config YAML files
- LiteLLM provider config
- Simple deployment: 1 container vs 4+
