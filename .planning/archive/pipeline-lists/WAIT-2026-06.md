# WAIT — Blocked Items

Items in this list are blocked on external dependencies and cannot proceed until the blocker is resolved.

| ID | Item | Blocker | Owner | Unblocked When |
|----|------|---------|-------|----------------|
| W-001 | Publish odysseusAgentOS base Docker image to GHCR | Docker image build pipeline not yet finalized | — | Phase 5 executor image is stable and tagged |
| W-002 | LiteLLM OpenRouter quota increase | Waiting for OpenRouter team approval | — | Quota approval email received |
| W-003 | Ollama local fallback integration tests | Local Ollama instance not yet provisioned in CI | — | CI runner has Ollama sidecar available |

## Format

```
| W-XXX | Short description | What is blocking | Owner | Condition to unblock |
```

Move items to MOVE.md once blocker is resolved.
