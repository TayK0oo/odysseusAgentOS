# Domain 04 — Memory / RAG / Knowledge / Context (NATIVE Odysseus)

> Map of NATIVE capabilities so the grafted "intelligence layer" (RRF, acontext, Trinité)
> extends rather than duplicates. All citations are `fichier:ligne`. CBM project:
> `C-Users-ttmdu-Documents-GitHub-odysseusAgentOS`.
> **Key verdict up front:** a naive `0.7/0.3` vector/keyword blend AND an RRF `hybrid_search`
> already exist in `src/rag_vector.py`. The RRF one is ORPHAN (tests only, no prod caller).

---

## 1. Native memory subsystem (store / retrieve)

Two-tier: JSON store of truth + optional Chroma vector index, unified behind a provider.

- **`src/memory.py` — `MemoryManager`** (flat-file store of record `memory.json`)
  - `load_all` / `load(owner)` `memory.py:113/129`, `save` (atomic tmp+replace) `memory.py:196`,
    `add_entry` `memory.py:215`, `increment_uses` `memory.py:232`, `find_duplicates` `memory.py:247`.
  - **Two native keyword retrievers already exist here:**
    - `get_relevant_memories` `memory.py:291` — Jaccard token similarity + query-type
      classification (identity/contact/preference/task/fact) with multiplicative boosts
      (1.4/1.3/1.3) `memory.py:355-379`, threshold default `0.05`.
    - `categorize_memory_by_relevance` `memory.py:255`.
  - `get_text_similarity` Jaccard helper `memory.py:17`; `extract_memory_from_chat` bullet-scrape
    fallback `memory.py:40`; inline `remember:` command parser `memory.py:87`.
- **`src/memory_vector.py` — `MemoryVectorStore`** (Chroma collection `odysseus_memories` `memory_vector.py:28`)
  - `add` `:104`, `remove` (O(1) delete) `:122`, `search(query,k)` cosine → `1-distance`
    `:132`, `find_similar(threshold=0.92)` dedup `:165`, `rebuild` (batched 100) `:188`, `get_stats` `:246`.
  - Multi-lane aware (custom + fastembed), dedup + lane-priority sort `:162-163`.
- **`src/memory_provider.py` — provider abstraction** (already generic!)
  - `MemoryProvider` ABC `:33` with `remember/recall/list_memories/delete` + `get_tool_schemas`.
  - `NativeMemoryProvider` `:96` wraps `MemoryManager` + `MemoryVectorStore`.
    `recall` `:169` = **vector-first, keyword fallback** (`memory_vector.search` then
    `memory_manager.get_relevant_memories` `:200`). Owner filtering `:188`.
  - `MemoryProviderRegistry` `:251` — supports EXTERNAL providers side-by-side, with
    tool-name conflict detection `:281`. This is the native extension seam for grafted memory.
- **`src/chat_processor.py` — `ChatProcessor._hybrid_retrieve`** `chat_processor.py:54` —
  the memory retriever actually used in chat context building. Full BM25 (k1=1.5, b=0.75
  `:90`) + IDF `:95` + optional vector `:106` + category boosts `:126-136` + recency tiebreaker
  (≤5% `:141`). Blend when vector present: `0.55*vs + 0.40*kw + 0.05*recency` `chat_processor.py:147`;
  keyword-only: `0.95*kw + 0.05*recency` `:151`. Gates on real relevance `:145`.
- **Routes `routes/memory_routes.py`**: `/add` `:84`, `GET ""` list `:132`, `POST /search` `:138`,
  `/timeline` `:154`, `/by-session/{id}` `:192`, `/extract` `:222`, `/audit` `:272`, `/import` `:323`,
  `/{id}/pin` `:486`, GET/PUT/DELETE `/{id}` `:500-533`.
- Also present: `mcp_servers/memory_server.py`, `services/memory/{memory,memory_extractor,memory_vector}.py`
  (parallel service-layer copies).

**Takeaway:** memory already has store, vector recall, keyword+BM25 fallback, hybrid blend,
category boosting, dedup, owner scoping, AND a pluggable provider registry.

---

## 2. Native RAG + retrieval flow — WHERE the 0.7/0.3 blend lives

Prod path: `app.py:511-512 get_rag_manager()` → `src/rag_singleton.py:19` (lazy, throttled retry
30s, returns None if Chroma down `:39`) → `VectorRAG` (`src/rag_vector.py`). `RAGManager`
(`src/rag_manager.py`) is a thin backward-compat wrapper delegating to `VectorRAG` `rag_manager.py:35-37`.

### 2a. The naive 0.7/0.3 blend — PRODUCTION path
- Constants: **`VECTOR_WEIGHT = 0.7`** `src/rag_vector.py:37`, **`KEYWORD_WEIGHT = 0.3`** `rag_vector.py:38`.
- Applied in **`VectorRAG.search`** `rag_vector.py:343`, exact line:
  **`hybrid_score = (VECTOR_WEIGHT * vector_sim) + (KEYWORD_WEIGHT * keyword_score)` — `rag_vector.py:378`**.
  - `vector_sim = 1.0 - distance` `:374`; `keyword_score` = query∩doc word overlap / |query| `:375-377`.
  - Multi-lane via `query_lanes` `:356`, sort by blended similarity `:391`, `dedupe_results` `:392`.
  - Fallback `_keyword_search_fallback` `:400` on exception `:398`.
- This IS the naive linear blend the grafted RRF is meant to replace. It is the ACTIVE code path
  (RAGManager.search / personal_docs / chat all call `VectorRAG.search`).

### 2b. RRF hybrid — ALREADY EXISTS but ORPHAN (Phase-15 graft-in-progress)
- `src/rag_vector.py:696-784` "RRF HYBRID SEARCH — Phase 15" (French comments):
  - `_bm25_search` (uses `rank_bm25.BM25Okapi`, returns `[]` if not installed) `:701`.
  - `_rrf_score(rank,k=60)` = `1/(k+rank+1)` `:726`.
  - `hybrid_search(query, top_k=10, alpha=0.5)` `:731` — vector(alpha) + BM25(1-alpha) fused by RRF `:761-772`.
- **Callers: NONE in production.** `hybrid_search` referenced ONLY by `tests/test_e2e_smoke.py:15,52`.
  Its internal vector step uses reflection to find a module function accepting `top_k` `:746-749`,
  but `VectorRAG.search` uses kwarg `k` (not `top_k`) and is a METHOD not a module function — so
  the reflection likely finds nothing and `vector_results` stays `[]`. **Effectively dead/non-wired.**
- Other keyword/BM25 retrievers: `session_search.py:274` (SQLite `bm25()` FTS over chat messages),
  `personal_docs.retrieve_personal_keyword` `personal_docs.py:119`.
- **Retrieval flow overall:** `ChatProcessor.build_context_preface` `chat_processor.py:159`
  assembles system-preface (memory via `_hybrid_retrieve`, RAG via manager, personal docs) with
  `RAG_SIMILARITY_THRESHOLD=0.35` `:52`.
- **Routes:** `routes/document_routes.py`, `routes/personal_routes.py`, `routes/research_routes.py`,
  `routes/diagnostics_routes.py:67` (`get_rag_stats`). `mcp_servers/rag_server.py:33` also uses
  `get_rag_manager`.

---

## 3. Native embeddings + vector store

- **`src/embeddings.py`** — priority: (1) HTTP API `EmbeddingClient` (Ollama/vLLM/llama.cpp) `:42`,
  (2) local `FastEmbedClient` (ONNX all-MiniLM-L6-v2, zero-config fallback) `embeddings.py:38-39`.
  `get_embedding_client` `:241`; Windows symlink guard `:25`; endpoint persisted + secret-encrypted.
- **`src/embedding_lanes.py`** — dual-lane isolation so different embed dims never share a Chroma
  collection: `LANE_CUSTOM`/`LANE_FASTEMBED` `:19-20`, `EmbeddingLane` dataclass `:23`,
  `build_embedding_lanes` (order: custom, fastembed) `:252`, `_get_or_reset_collection` re-embeds on
  fingerprint change `:136`, `migrate_legacy_collection` backfill `:275`, `query_lanes` `:357`,
  `dedupe_results` `:343`, `lane_count` `:339`. Cosine space enforced `:81`.
- **`src/chroma_client.py`** — `get_chroma_client` singleton (persistent client) used by lanes/stores.
- Collections: RAG `odysseus_rag` `rag_vector.py:40`, memory `odysseus_memories` `memory_vector.py:28`,
  each split `_custom` / `_fastembed`. Embeddings normalized `embedding_lanes.py:39`.
- Routes: `routes/embedding_routes.py` (endpoint config/probe).

---

## 4. Native context budget / compaction

- **`src/context_budget.py`** — `compute_input_token_budget` `:21`: auto-scales soft input budget to
  `0.85 * context_window` (headroom `:18`) capped at `200_000` `:16`; explicit user cap honoured/clamped
  `:54`; conservative `default=6000` when window unknown `:61`. `budget_is_explicit` `:64`.
- **`src/context_compactor.py`** — trigger at `COMPACT_THRESHOLD=0.85` of window `:39`.
  - `maybe_compact` `:312`: summarize older half via utility LLM (Cursor-style structured prompt
    `:44`, temp 0.2, max 1024 tok `:381-388`), keep recent half, rewrite session history `:409`.
  - `trim_for_context` `:215`: progressive trim (drop extra system/memory/RAG msgs → truncate system
    prompt → drop old turns, PROTECT_RECENT=10 `:286`, never drop current user turn, truncate huge
    pastes with notice). Protects research primers `:252` and `_protected` msgs `:236`.
  - `_sanitize_tool_messages` `:73` repairs orphan tool/tool_calls pairs; `_truncate_tool_call_args` `:150`.
- **`src/model_context.py`** — `get_context_length`, `estimate_tokens` (chars*0.3 heuristic `:430`).

---

## 5. Extension seams — do grafted pieces add anything native lacks?

**Note:** the "grafted" code already exists in-repo as stubs on this branch — they are the graft in
progress, NOT upstream OpenCode. Verdicts compare the grafted DESIGN vs native prod behaviour.

| Grafted piece | Native equivalent | Verdict | Rationale |
|---|---|---|---|
| **RRF hybrid retrieval** | `VectorRAG.search` 0.7/0.3 blend (`rag_vector.py:378`, ACTIVE); `hybrid_search` RRF already coded `rag_vector.py:731` but ORPHAN/broken-reflection; `ChatProcessor._hybrid_retrieve` full BM25+vector for MEMORY | **PARTIAL** | RRF code already exists but is unwired & likely returns `[]` (kwarg `k` vs `top_k` mismatch `:749`). Real value = **wire it in + fix reflection** to replace the naive 0.7/0.3 blend in `VectorRAG.search`, not add a new module. Memory already has a genuine BM25 hybrid. |
| **acontext service** | `services/acontext/app.py` (session distillation stub, port 8029); native `context_compactor` already produces structured session summaries; `MemoryProviderRegistry` supports external providers; `memory_routes /extract` `:222`, `/audit` `:272` | **PARTIAL** | Session-end distillation endpoint is genuinely new (persists sessions to `/data/acontext`). But summarization/compaction and memory-extraction already exist natively; acontext should FEED the native memory provider (via `MemoryProviderRegistry`) rather than reimplement recall/store. |
| **Knowledge "Trinité" (CBM+Graphify+Obsidian)** | `routes/knowledge_routes.py` (already stubbed) proxies to 3 external services (ports 9749/9750/9751); native RAG/memory are Chroma-based, NOT graph/Obsidian | **UNIQUE** | Cross-source graph+notes orchestration (`/api/knowledge/checkpoint` `:116`) has no native analogue. Genuinely additive. Risk: overlaps native RAG for the "semantic search" leg — should defer doc semantic search to native `VectorRAG` instead of Graphify where possible. |

Native extension seams to prefer over new code:
- `MemoryProviderRegistry.register` (`memory_provider.py:259`) — plug acontext/external memory here.
- `VectorRAG.search` (`rag_vector.py:343`) — swap blend → RRF in place (constants at `:37-38`).
- `embedding_lanes` — already multi-model; grafted retrieval should reuse `query_lanes`, not new stores.

---

## 6. Risks / notes

- **Duplicate RRF risk (HIGH):** grafting a NEW RRF module would create a 3rd hybrid impl alongside the
  orphan `hybrid_search` `rag_vector.py:731` and the memory `_hybrid_retrieve` `chat_processor.py:54`.
  Fix/wire the existing one; delete or wire the orphan.
- **Blend location single source of truth:** the naive blend is ONLY at `rag_vector.py:378` (constants
  `:37-38`). Any RRF swap is localized — low blast radius.
- **Reflection bug in existing `hybrid_search`:** `top_k` vs `k` kwarg mismatch (`rag_vector.py:749` vs
  `search(..., k=...)` `:343`) means it silently returns `[]`. Verify before trusting Phase-15 tests
  (they only assert signatures/scores, `tests/test_e2e_smoke.py`, not end-to-end retrieval).
- **Two parallel memory copies:** `src/memory*.py` vs `services/memory/memory*.py` — confirm which is
  authoritative before grafting (prod app.py wires `src/`).
- **Chroma availability gating:** `rag_singleton` returns None when Chroma down (503 upstream) — grafted
  layer must handle None RAG manager gracefully.
- **acontext persistence path** `/data/acontext` (`services/acontext/app.py:9`) is container-absolute;
  ensure volume mount, else session distillation is lost.
- **Owner scoping:** native memory/RAG enforce `owner` filtering (`memory_provider.py:188`,
  `rag_vector.py:352`). Grafted retrieval must preserve owner filters or leak cross-user data.
