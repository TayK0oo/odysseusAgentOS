# Domain 01 — Models & Providers (Odysseus native)

Maps the NATIVE model/provider system so the grafted "intelligence layer" never
duplicates it. All claims cite `file:line`. Root: `odysseusAgentOS/`.

---

## 1. Native capabilities

### 1.1 Provider detection (URL → provider)
- `src/llm_core.py:_detect_provider` (llm_core.py:652-682): hostname-match (not
  substring) → `ollama | anthropic | opencode-go | opencode-zen | openrouter |
  groq | nvidia | moonshot | chatgpt-subscription | copilot`; unknown hosts fall
  back to OpenAI-compatible default. Consumed everywhere for URL/header shaping.

### 1.2 Endpoint URL + header construction
- `src/endpoint_resolver.py:normalize_base` (152-161): strips API suffixes
  (`/models`, `/chat/completions`, `/v1/messages`, `/responses`, ollama `/api/*`).
- `build_chat_url` (198-210): provider-specific chat path (anthropic
  `/v1/messages`, ollama `/api/chat`, chatgpt-subscription `/responses`, else
  `/chat/completions`, auto-inserting `/v1` for bare `api.openai.com`).
- `build_models_url` (213-242): provider-specific model-list URL; inserts `/v1`
  for local servers (LM Studio, llama.cpp, vLLM) and deepseek/openai.
- `build_headers` (245-267): anthropic `x-api-key`+`anthropic-version`; copilot &
  chatgpt-subscription delegate to their own header builders; else `Bearer`;
  openrouter attribution headers; kimi-code `User-Agent`.

### 1.3 Endpoint registry (DB) & curated model lists
- `core.database.ModelEndpoint` row is the unit of registration: `base_url`,
  `api_key`, `is_enabled`, `cached_models`, `hidden_models`, `provider_auth_id`.
- Curated lists: `routes/model_routes.py:_PROVIDER_CURATED` (235-288) — real IDs
  for openai/anthropic/zai/zai-coding/kimi-code/deepseek/groq/mistral/together/
  fireworks/google/xai. `_HOST_TO_CURATED` (296-309) maps hostnames→curated key;
  `_match_provider_curated` (312-328) + `_curate_models` (331-356) partition a
  server's raw model list into (curated, extra).
- Enable/disable per model = `hidden_models`; helpers in endpoint_resolver:
  `_endpoint_cached_models` (38), `_endpoint_hidden_models` (50),
  `_endpoint_enabled_models` (62), `_first_chat_model` (30) — auto-pick skips
  embeddings/tts/etc via `_NON_CHAT_MODEL` (24).
- Docker rewrite: `routes/model_routes.py:_rewrite_loopback_for_docker` (197-229)
  + `_docker_host_gateway_reachable` (150-168) rewrite `localhost`→
  `host.docker.internal` when running in a container.

### 1.4 Role → model settings keys  (`src/settings.py` DEFAULT_SETTINGS)
| Role | endpoint key | model key | fallback-chain key |
|------|--------------|-----------|--------------------|
| Default chat | `default_endpoint_id` (137) | `default_model` (138) | `default_model_fallbacks` (143) |
| Utility | `utility_endpoint_id` (148) | `utility_model` (149) | `utility_model_fallbacks` (152) |
| Task | `task_endpoint_id` (135) | `task_model` (136) | — (borrows utility→default) |
| Research | `research_endpoint_id` (91) | `research_model` (92) | — |
| Vision | — | `vision_model` (43), `vision_enabled` (44) | `vision_model_fallbacks` (46) |
| Image | — | `image_model` (41) | — |
| TTS | — | `tts_model` (52), `tts_provider` (51), `tts_voice`/`tts_speed` | — |
| STT | — | `stt_model` (57), `stt_provider` (56), `stt_language` | — |
| Teacher | — | `teacher_model` (153) | — |

Per-user overridable keys whitelisted at settings.py:262-273.

### 1.5 Fallback resolution flow (native)
`resolve_endpoint(setting_prefix, ...)` (endpoint_resolver.py:270-361) is the core:
1. Read `{prefix}_endpoint_id` / `{prefix}_model` (per-user via `get_user_setting`).
2. If unset and prefix ∉ {utility, default} → borrow `utility_*`.
3. Still unset but caller passed `fallback_url`+`fallback_model` → use that
   (preserves mid-session model choice).
4. Still unset → borrow `default_*`.
5. Load `ModelEndpoint` (enabled, owner-scoped), `resolve_endpoint_runtime`
   (73-89) resolves runtime base+key (static `api_key` OR refreshable
   `ProviderAuthSession` via `provider_auth_id`).
6. `build_chat_url` + `build_headers`; discard a model now in `hidden_models`;
   auto-pick `_first_chat_model(_endpoint_enabled_models(ep))`.
- `resolve_endpoint_by_id` (364-407): resolve a specific `{endpoint_id, model}`.
- Fallback chains: `resolve_chat_fallback_candidates` (410),
  `resolve_utility_fallback_candidates` (420),
  `resolve_vision_fallback_candidates` (436) → `_resolve_fallback_candidates`
  (441-455) turns each configured `{endpoint_id, model}` entry into a resolvable
  `(chat_url, model, headers)` tuple, skipping dead ones.
- Runtime dispatch: `stream_llm_with_fallback` (llm_core.py ~2464) dedupes the
  candidate list and streams each `(url, model, headers)` in order, swallowing
  pre-content failures and advancing to the next candidate. Dead-host cooldown
  makes retries near-instant.

### 1.6 Model discovery (running local models)
`src/model_discovery.py:ModelDiscovery`:
- `discover_tailscale_hosts` (36-87): parses `tailscale status --json`, returns
  online-peer IPv4s (skips funnel/android).
- `_get_hosts` (98-148): `LLM_HOSTS` override → tailscale → default; always adds
  `host.docker.internal`; merges ports from `OLLAMA_BASE_URL`/`OLLAMA_URL`/
  `LM_STUDIO_URL`.
- `_fingerprint_provider` (150-181): `/api/v1/models`→lmstudio, `/props`→llamacpp.
- `_check_port` (183-203) + `discover_models` (205-240): parallel-scans ports
  `8000-8020, 8080, 1234, 11434, 11435` for OpenAI-compatible `/v1/models`,
  dedupes by (port, model-ids).
- `warmup_ping_urls` (242), `get_providers` (261-288: vllm items + optional
  static OpenAI list).

---

## 2. API surface  (`routes/model_routes.py`)
- `GET /models` (1331) — visible models (curated).
- `GET /model-endpoints/probe-local` (1385), `GET /discover` (1619) — discovery.
- `GET /ping` (1451), `POST /probe-selected` (1491), `GET /probe` (1536),
  `GET /model-endpoints/{ep_id}/probe` (1973) — reachability/model probing.
- `GET /providers` (1607) — provider list.
- `GET /model-endpoints` (1627) list · `POST /model-endpoints` (1727) create ·
  `POST /model-endpoints/test` (1942) · `PATCH /model-endpoints/{ep_id}` (2223)
  toggle enable/disable · `DELETE /model-endpoints/{ep_id}` (2380) ·
  `GET /model-endpoints/{ep_id}/dependents` (2374).
- `GET /model-endpoints/{ep_id}/models` (2023) · `PATCH .../models` (2074)
  set `hidden_models` (per-model enable/disable).
- `GET /default-chat` (2109-…) — resolves per-user default endpoint+model+
  fallbacks (owner-scoped, no admin-default leak).
- `GET /tools` (2412) · `POST /tools` (2426).

This is a full CRUD + discovery + curation + role-resolution surface.

---

## 3. Extension seams (add value WITHOUT duplicating)
- **Intent → existing ROLE mapping.** Native system already resolves
  role→endpoint→model with fallbacks. An intelligence layer should classify a
  turn's *intent* and pick a native **setting_prefix** (`default`/`utility`/
  `research`/`task`) or a native fallback-chain, then call the existing
  `resolve_endpoint(...)` / `stream_llm_with_fallback(...)`. It must NOT invent
  its own provider/model catalog.
- **Advisory logging** of a suggested role (as `router_advice.advise` already
  does) is safe; overriding the user's `sess.model` is the risky part and is
  deliberately deferred.
- **Curated-list enrichment**: contribute new real model IDs into
  `_PROVIDER_CURATED`, not a parallel JSON.
- **Discovery hooks**: consume `ModelDiscovery.discover_models()` output rather
  than re-scanning ports.

---

## 4. Grafted / redundant modules for this domain

### 4.1 `src/llm_router.py::ModelRouter`  — **REDUNDANT**
- Reads `model-routing.json` + `.planning/stage-model-assignment.yaml`, maps
  intent/stage→tier→LiteLLM id, has its own `complete()` fallback loop
  (429/500/502/503, cooldown). Duplicates native role-resolution + fallback.
- Uses **FAKE/placeholder model IDs** (model-routing.json: `deepseek-v4-pro`,
  `kimi-k2.6`, `deepseek-v4-flash`, `kimi-k2.7-code`, `qwen3.7-plus`, `glm-5.2`)
  — none registered as native `ModelEndpoint`s or in `_PROVIDER_CURATED`.
- **Live call-sites:** only `src/orchestrator/router_advice.py:43` (advisory,
  gated OFF by env `ODYSSEUS_MODEL_ROUTER`, router_advice.py:26-29) and the
  eager re-export `src/__init__.py:3`. `complete()` has **zero** live callers.
  Its docstring/router_advice.py:5-13 admit the catalog is placeholder and it
  never overrides the user's model. Depends on optional `litellm`.
- **Verdict: REDUNDANT** — advisory only, fake catalog, no dispatch path.

### 4.2 `src/zen_router.py`  — **PARTIAL-OVERLAP** (one UNIQUE bit)
- Reads the **same** `model-routing.json`; `classify_complexity` (51-91)
  heuristic tiering, `build_zen_candidates` (233-274), `call_zen`/`route_and_call`,
  plus a blacklist/fail-count layer (`_zen_blacklist`, `_record_zen_failure`).
- **Live call-site:** `src/llm_core.py:2457` — inside `stream_llm_with_fallback`,
  if `OPENCODE_API_KEY` is set it **prepends** zen candidates ahead of the native
  chain. So it DOES reach real dispatch (unlike ModelRouter). It also uses the
  fake tier IDs but pointed at the real OpenCode-Zen `/zen/go/v1` endpoint.
- Overlap: candidate-building + fallback duplicate native `_resolve_fallback_
  candidates` + `stream_llm_with_fallback`. UNIQUE: the OpenCode-Zen Go-plan
  integration + per-model blacklist/cooldown are not natively modeled.
- **Verdict: PARTIAL-OVERLAP** — safe-to-remove requires migrating the Zen Go
  provider into a native `ModelEndpoint` (opencode-go provider already exists in
  `_detect_provider`) and dropping the fake-tier catalog; the blacklist feature
  would be lost unless ported.

### 4.3 `routes/routing_routes.py` (`/api/route*`)  — **REDUNDANT**
- Thin REST wrapper over `zen_router` (`route_and_call`, `classify_complexity`,
  `_load_routing_config`, blacklist stats). Registered in `app.py:803`.
- Endpoints `POST /api/route`, `/api/route/dry-run`, `GET /api/route/config`,
  `/api/route/stats`. All served by native `/api/model-routes` + resolver flow
  except the heuristic classifier + blacklist dump.
- **Verdict: REDUNDANT** as an alternative chat path; only value is exposing the
  intent classifier / blacklist, which belong (if kept) in an intelligence layer,
  not a parallel model catalog.

### 4.4 `src/intent_gate.py` (`classify_intent`) — **UNIQUE (keep)**
- Pure intent classification; no model catalog. Re-exported `src/__init__.py:4`,
  used by `router_advice`. This is exactly the "intent" seam of §3 and does not
  duplicate native provider/model logic. Keep; rewire to native roles.

### 4.5 Grafted config files
- `model-routing.json` (repo root) + `.planning/stage-model-assignment.yaml` —
  the fake-catalog config feeding 4.1/4.2/4.3. **REDUNDANT** once zen provider is
  natively registered.

---

## 5. Risks / notes
- `src/__init__.py:3-4` imports ModelRouter+intent_gate **eagerly** at package
  import; removing `llm_router.py` without fixing `__init__.py` breaks `import
  src.*` everywhere. Remove the re-export first.
- `zen_router` is the ONLY grafted piece on a real dispatch path
  (llm_core.py:2457, gated by `OPENCODE_API_KEY`). Deleting it silently changes
  chat behavior for users who set that env var — migrate the Zen Go endpoint to a
  native `ModelEndpoint` before removal, or the Go-plan routing is lost.
- `ModelRouter.complete()` depends on optional `litellm`; the codebase's real
  dispatch is `stream_llm`/`stream_llm_with_fallback` (httpx), so LiteLLM is dead
  weight.
- Fake model IDs in `model-routing.json` will silently 404/400 against any real
  provider except the OpenCode-Zen Go endpoint they were tuned for.
- `tests/test_harness_core.py` (156-196) and `tests/test_orchestrator_router_
  advice.py` still import the grafted routers — update/remove on cleanup.
