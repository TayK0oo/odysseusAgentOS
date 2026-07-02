# Domain 02 — Chat & Agent Loop (NATIVE capability map)

Maps how Odysseus natively runs one chat turn, so the grafted intelligence layer
(orchestrator/, intent_gate, phase-lock, budget, observer) never duplicates it.
Core file: `src/agent_loop.py`. Live entry: `routes/chat_routes.py`.

---

## 1. Native turn lifecycle (request → prep → round loop → tool exec → metrics → save)

**Entry (live chat):** `routes/chat_routes.py:1271` calls `stream_agent_loop(...)`.
The caller resolves everything and passes it in — the loop is stateless re: routing:
- `sess.endpoint_url`, `sess.model`, `sess.headers` from `session_manager.get_session()` (`chat_routes.py:606/364`).
- `fallbacks=_fallback_candidates` = `resolve_chat_fallback_candidates(owner)` (`chat_routes.py:1065`) — list of `(url, model, headers)` tuples.
- `max_rounds=_max_rounds` from setting `agent_max_rounds` clamped `[1,200]` (`chat_routes.py:1262-1265`).
- `max_tool_calls=_tool_budget` from `agent_max_tool_calls` (`chat_routes.py:1258`).
- `tool_policy` = `build_effective_tool_policy(last_user_message)` (`chat_routes.py:385/874`).
- `forced_tools` (web toggle → `{web_search, web_fetch}`), `plan_mode`, `approved_plan`, `active_document`, `active_email`, `workspace`, `owner`.
- History/context assembled upstream by `routes/chat_helpers.build_chat_context` (`chat_helpers.py:564`) + `src/chat_processor.ChatProcessor.build_context_preface` (`chat_processor.py:159`, RAG memory preface via `_hybrid_retrieve`).

**`stream_agent_loop` signature:** `src/agent_loop.py:1934`. Async SSE generator yielding
`delta`, `tool_start`, `tool_output`, `tool_progress`, `agent_step`, `metrics`, `[DONE]`.

**PREP (once, before the round loop):**
1. `request_setup` — merge disabled tools from `tool_policy`, owner blocklist (`blocked_tools_for_owner`), plan-mode disabled set (`agent_loop.py:1972-2104`).
2. Intent/retrieval classify: `_detect_admin_intent` (`:1993`), `_extract_last_user_message` (`:1994`), **`_classify_agent_request` (`:1995`, def `:860`)**.
3. **Low-signal direct path** (`:1998-2095`): if `_direct_low_signal` (casual "hey", no domains, no doc/email/workspace/forced tools), it bypasses the whole agent machinery — one `stream_llm_with_fallback` call, `max_tokens≤128`, NO tools, emits metrics with `direct_low_signal:True` and returns. Cheap greeting path.
4. `tool_selection` (`:2106-2253`): RAG tool retrieval via `src.tool_index.get_tool_index().get_tools_for_query` (8 tools, timeout-guarded), keyword fallback (`ToolIndex._KEYWORD_HINTS`), then deterministic domain seeding from `_intent["domains"]` via `_DOMAIN_TOOL_MAP`, plus forced/document/skill tool unions.
5. `prompt_build` (`:2255-2363`): decide `_is_api_model` (native function-calling vs fenced) from endpoint `supports_tools` flag → model-name keyword list → `_API_HOSTS` (`:2265-2326`). Then **`_build_system_prompt` (`:2327`, def `:975`)** which calls **`_assemble_prompt` (def `:568`)** = `_AGENT_PREAMBLE` + tool sections (compact or full) + `_AGENT_RULES` + `_domain_rules_for_tools`. Plan-mode / approved-plan / guide-only directives prepended to system msg (`:2337-2362`).
6. `context_trim` (`:2365-2415`): soft budget trim via `src.context_budget.compute_input_token_budget` + `src.context_compactor.trim_for_context` (see domain 01/context).
7. Emits `agent_prep` SSE with per-stage `prep_timings`.

**ROUND LOOP** (`for round_num in range(1, max_rounds+1)`, `:2533`):
- `_phase_tracker.on_round_start(...)` (grafted seam, `:2535`).
- `_budget_enforcer.consume_iteration()` per-round if a PROJECT.yaml manifest exists (`:2539`).
- Build `all_tool_schemas`: `[]` if `_force_answer`; RAG-filtered `FUNCTION_TOOL_SCHEMAS`+`mcp_schemas` if API model; MCP-only-on-keyword if local (`:2561-2602`).
- Stream from `stream_llm_with_fallback(_candidates, ...)` where `_candidates = [(endpoint_url,model,headers)] + fallbacks` (`:2611/2630`). Wall-clock `_round_deadline = agent_stream_timeout*4` (`:2616`), plus per-read inactivity timeout inside stream_llm.
- Accumulate `round_response`, `round_reasoning`, `native_tool_calls`, real usage tokens, backend TPS.
- Detect tool calls: native `tool_calls` OR fenced blocks via `strip_tool_blocks`/`ToolBlock` scanner; auto-create documents from big fenced code blocks (`:2900-2914`).
- If **no tool_blocks** → completion verifier check, intent-nudge check, else `break` (done).
- Else: loop-breaker check, pre-stream doc content, then **execute each block** via `execute_tool_block(...)` (`:3159`) with live `tool_progress` drainer (`:3153-3182`); skill `requires_toolsets` unlocked for next round; web_search sources extracted.

**METRICS + SAVE** (after loop): **`_compute_final_metrics` (def `:1701`)** — real-vs-estimated tokens, `tokens_per_second` (prefers backend `timings.predicted_per_second`, `tps_source` = backend/computed), `context_percent` off peak last-round input, `agent_prep_time` breakdown, `tool_events`, `round_texts`. Yielded as `metrics` SSE. `chat_routes.py` consumes the stream, strips thinking deltas, persists reply + `tool_events` to session history.

---

## 2. Native intent classification vs grafted `intent_gate` — OVERLAP VERDICT

**Native `_classify_agent_request(messages, last_user)` (`agent_loop.py:860-943`)** returns:
```
{ low_signal: bool, continuation: bool, domains: set[str], retrieval_query: str }
```
- `domains` ⊂ {cookbook, email, notes_calendar_tasks, documents, web, ui, sessions, files, settings, contacts, integrations} — regex keyword match, English + some Polish. Drives **tool retrieval / prompt domain-rule packs**, NOT model routing.
- `continuation` = explicit-continuation / assistant-followup / retry detection → inherit recent context for retrieval.
- `low_signal` gates the direct greeting path.
- Also native: `_detect_admin_intent` (`:735`), `src/action_intents.classify_tool_intent` (`:120`), `src/zen_router.classify_complexity` (`:51`) for model-complexity tiering.

**Grafted `src/intent_gate.classify_intent(prompt)`** returns a **single string** ∈
{quick, deep, utility, vision, code, creative} from keyword dicts + word-count heuristic.
Its ONLY live call-site is `src/orchestrator/router_advice.advise` (`:38`), which feeds `ModelRouter.route()` — **advisory only, log-only, kill-switched `ODYSSEUS_MODEL_ROUTER` (default OFF), never overrides `sess.model`** (`router_advice.py:8-13,32-57`).

**Overlap verdict: PARTIAL / DISJOINT PURPOSE.**
- Both classify the user turn from the same `last_user` text, both keyword-based. Surface duplication of *effort*.
- But **different output spaces and different consumers**: native `domains` → tool selection + prompt packs (in-loop, always on); grafted `intent_gate` categories → model-tier routing suggestion (advisory, off). They do NOT currently collide — grafted never touches tool selection or the prompt, native never picks a model tier.
- Redundancy risk is future: if auto-routing is enabled, `intent_gate` becomes a second, coarser classifier running per turn alongside `_classify_agent_request`. Consolidation candidate, not a live conflict today.

---

## 3. Native guardrails already in the loop

All in `stream_agent_loop`, no graft needed:
- **Iteration cap:** `max_rounds` (`:2533`), caller-clamped `[1,200]`.
- **Tool-call budget:** `max_tool_calls` → `budget_exceeded` SSE + break (`:3121-3124`).
- **Loop-breaker / stall detector (Terminus-style):** `_recent_call_sigs` deque + `_stuck_rounds`; a round is "useless" only if it re-issues a recent call AND writes no real text. `_stuck_rounds>=4` → trip (`:2470-2471,3033-3078`).
- **Runaway backstop:** `_call_freq` Counter + `_detect_runaway_call` (def `:1922`) — same exact call sig repeated an absurd number of times (distinct batch calls allowed) (`:3051`).
- **Force-answer:** on loop-breaker/runaway trip, `_force_answer=True` → next round sends `all_tool_schemas=[]` so the model must write its answer or state it's blocked (`:2477,2561-2565,3065-3078`).
- **Intent-without-action supervisor:** `_INTENT_RE` catches "Let me tail the logs" with no tool call; injects one sharp nudge, capped `_MAX_INTENT_NUDGES=2` (`:2491-2500,2969-3019`).
- **Completion verifier subagent (mechanism 3a):** `_run_verifier_subagent` (def `:1801`) fires only on effectful turns, capped `_VERIFIER_MAX_ROUNDS`, **default OFF** via `agent_verifier_subagent` setting (`:2926-2968`).
- **Empty-response fallback:** `_empty_response_fallback` (def `:1851`).
- **Round wall-clock deadline** `agent_stream_timeout*4` (`:2616`) + inactivity timeout in stream_llm.
- **Exhausted-rounds signal:** `_exhausted_rounds` → "Continue" affordance instead of silent stall (`:2511`).
- **Native fallback chain:** `stream_llm_with_fallback` switches endpoint only on pre-content failure (no dup output), dead-host cooldown (`:2608-2611`).
- **Plan-mode / guide-only tool gating** (read-only enforcement) baked into disabled_tools + prompt directives.

---

## 4. Extension seams already added by us (present, mostly inert live)

- **`PhaseTracker`** `src/orchestrator/phase_tracker.py`, instantiated `agent_loop.py:2516-2517`, called `_phase_tracker.on_round_start(round_num, intent, plan_mode)` at top of every round (`:2535`, try/except-wrapped, never breaks loop). Kill-switched `ODYSSEUS_PHASE_TRACKER` (default OFF). When OFF, `set_phase` never called → live behavior byte-identical. `infer_phase` conservative: `PLAN` if plan_mode else `BUILD`.
- **`router_advice.advise`** `src/orchestrator/router_advice.py`, called `agent_loop.py:2519-2531`. First live call-site of `intent_gate.classify_intent` + `llm_router.ModelRouter`. Log-only, kill-switched `ODYSSEUS_MODEL_ROUTER` (default OFF), NEVER overrides `model`.
- **`BudgetEnforcer`** `src/budget_enforcer.py`, instantiated `agent_loop.py:2453-2464` only if `src/project_manifest.load_manifest(".")` finds a `PROJECT.yaml`. Per-round `consume_iteration()` gate (`:2539-2543`). Inert unless a manifest exists at cwd.
- **`gate.should_block_destructive`** `src/orchestrator/gate.py` — catastrophic shell-command block via `risk_classifier`; enforcement lives in tool_execution, not the loop head (kill-switch `ODYSSEUS_DESTRUCTIVE_GATE`, default ON).

---

## 5. Redundancy verdict — grafted modules touching chat

| Grafted module | Native equivalent | Verdict |
|---|---|---|
| `src/intent_gate.py` (classify_intent → quick/deep/…) | `_classify_agent_request` (domains) + `zen_router.classify_complexity` | **PARTIAL**. Same keyword-classify effort on same text, but disjoint output space + advisory-only consumer. Not live-competing; consolidation candidate if auto-routing ships. |
| `router_advice` / `ModelRouter` model routing | caller picks `sess.model`; `zen_router` complexity tiering exists | **PARTIAL / DEFERRED**. No live override; native model choice + fallback chain already complete. |
| `PhaseTracker` phase-per-round | none (loop had no phase concept) | **UNIQUE** (but inert until phase-lock is wired). |
| `BudgetEnforcer` (manifest budgets) | native `max_rounds` + `max_tool_calls` | **PARTIAL**. Native already caps rounds & tool calls; enforcer adds manifest-driven multi-dimensional budgets. Overlap on iteration cap only. |
| `gate.should_block_destructive` | risk logging in tool_execution (log-only historically) | **UNIQUE** (adds the actual block decision). |
| Verifier/autoeval graft | native `_run_verifier_subagent` (mechanism 3a) | **REDUNDANT** for chat completion-verification — native already has a capped, effectful-only verifier subagent (off by default). A grafted autoeval must reuse/extend this, not add a parallel one. |
| Observer/metrics graft | native `_compute_final_metrics` + `tool_events`/`round_texts` + SSE | **PARTIAL**. Rich per-turn metrics + tool event log already emitted & persisted; graft should consume the `metrics` SSE, not recompute. |

---

## 6. Risks / notes

- **Two classifiers on the hot path.** With `ODYSSEUS_MODEL_ROUTER=on`, both `_classify_agent_request` and `intent_gate.classify_intent` run per turn on the same text — double keyword scan, two taxonomies. Fine while advisory; unify before any auto-route override.
- **intent_gate is English/French keyword-only**, no embeddings; native tool-RAG (`tool_index`) already handles non-English retrieval. Don't route model tier off intent_gate for non-EN/FR without a fallback.
- **Verifier duplication trap.** Native verifier is off by default because weak local models false-reject from the action-snapshot. A grafted autoeval must respect the same effectful-only + capped + opt-in design, or it will cost an extra round every turn.
- **Budget double-counting.** If BudgetEnforcer and native `max_rounds`/`max_tool_calls` both count, ensure they don't independently abort mid-turn with conflicting reasons.
- **PhaseTracker mapping is stub** (`BUILD`/`PLAN` only). Phase-lock is effectively inert live until `infer_phase` is enriched — grafted phase-gated tool permissions will do nothing until then.
- **Direct low-signal path skips the whole harness** (no tools, no phase tracker, no budget). Any grafted observer that must see every turn needs a hook there too (`agent_loop.py:2022-2095`) or it will miss greetings.
- **The loop is a caller-driven pure executor**: model, headers, fallbacks, budgets, policy, plan all arrive as args from `chat_routes.py`. Grafting routing/budget logic *inside* the loop fights this design — prefer wiring at the `chat_routes` call-site or via the existing kill-switched seams.
