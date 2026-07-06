# MOVE — Active Work Items

Items currently in progress or ready to be picked up. Each item has an owner and current status.

| ID | Item | Owner | Status | Phase | Notes |
|----|------|-------|--------|-------|-------|
| M-001 | GSD subagents definition (gsd-planner, gsd-executor, gsd-verifier, gsd-researcher, gsd-debugger, gsd-roadmapper) | agentos | IN_PROGRESS | Phase 6 | Agent .md files being created |
| M-002 | Stage model assignment YAML for llm_router.py | agentos | IN_PROGRESS | Phase 6 | Maps GSD stages to LiteLLM tiers |
| M-003 | Pipeline Lists bootstrap (WAIT/MOVE/SKIP) | agentos | IN_PROGRESS | Phase 6 | This file |
| M-004 | llm_router.py stage parameter integration | agentos | TODO | Phase 6 | Consume stage-model-assignment.yaml in route() |
| M-005 | GSD skill wiring to subagent invocations | agentos | TODO | Phase 7 | Connect /gsd:plan-phase etc. to gsd-planner agent |

## Statuses

- `TODO` — ready to start, no blocker
- `IN_PROGRESS` — actively being worked on
- `REVIEW` — work done, awaiting review/merge
- `DONE` — completed and verified

Move completed items to SKIP.md with rationale "completed" or archive them.
