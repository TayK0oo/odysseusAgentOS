# SKIP — Intentionally Deferred Items

Items in this list are deliberately skipped. Each entry must include a rationale and a revisit condition.

| ID | Item | Rationale | Revisit When |
|----|------|-----------|--------------|
| S-001 | Custom fine-tuned model for planning stage | Fine-tuning infra not justified at current scale; Claude Opus-4 sufficient | >10k planning calls/day, budget available |
| S-002 | Real-time streaming GSD progress to dashboard WebSocket | Dashboard WebSocket not yet implemented (Phase 8+); SSE polling sufficient | Dashboard WebSocket endpoint exists |
| S-003 | Multi-tenant GSD pipeline isolation | Single-user deployment for v1; multi-tenant adds complexity without current need | First external user onboarded |
| S-004 | GSD pipeline audit log to persistent DB | File-based STATE.md sufficient for v1; DB adds dependency | Audit query requirements emerge from usage |
| S-005 | Automated ROADMAP.md sync to GitHub Milestones via API | Manual sync acceptable at current cadence | >1 contributor actively using roadmap |

## Format

```
| S-XXX | Short description | Why skipped | Condition to revisit |
```

Move items back to MOVE.md if the revisit condition is met.
