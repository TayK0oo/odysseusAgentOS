# Agent A12: LangGraph — Decompose stream_agent_loop

## TASK
Refactor the monolithic `stream_agent_loop` (3,568 lines) into a LangGraph StateGraph with 7 testable, checkpointable nodes — one per canonical phase.

## CONTEXT
- Current: `stream_agent_loop` in `src/agent_loop.py` is ONE function (3,568 lines) that handles everything.
- Problem: Impossible to test individual phases. Hard to modify one phase without breaking others.
- LangGraph: StateGraph with nodes and edges. Each node = one phase. Checkpointing built-in. Human-in-the-loop natively.

## ARCHITECTURE TARGET

```python
# src/orchestrator/langgraph_loop.py (NEW)

from langgraph.graph import StateGraph, END
from langgraph.checkpoint.sqlite import SqliteSaver

class AgentState(TypedDict):
    messages: list
    phase: str
    risk_level: str
    context: dict
    plan: dict
    tool_calls: list
    budgets: dict

def build_canonical_graph() -> StateGraph:
    graph = StateGraph(AgentState)
    
    graph.add_node("classify", classify_node)       # Risk assessment
    graph.add_node("know", know_node)               # Knowledge retrieval
    graph.add_node("plan", plan_node)               # Planning
    graph.add_node("build", build_node)             # Implementation
    graph.add_node("quality", quality_node)         # Testing/linting
    graph.add_node("autoeval", autoeval_node)       # Self-evaluation
    graph.add_node("memory_observe", memory_node)   # Memory distillation
    
    graph.set_entry_point("classify")
    graph.add_edge("classify", "know")
    graph.add_edge("know", "plan")
    graph.add_edge("plan", "build")
    graph.add_edge("build", "quality")
    graph.add_edge("quality", "autoeval")
    graph.add_conditional_edges("autoeval", 
        lambda s: "memory_observe" if s["passed"] else "build")
    graph.add_edge("memory_observe", END)
    
    return graph.compile(checkpointer=SqliteSaver.from_conn_string("checkpoints.db"))
```

## REQUIREMENTS

### 1. Extract Nodes
Create `src/orchestrator/langgraph_loop.py` with 7 node functions.
Each node = extract logic from `stream_agent_loop` for that phase.

### 2. State Management
Define `AgentState` TypedDict. Each node reads/writes state.

### 3. Checkpointing
Use SQLite checkpoint saver. After each node, state is saved.
If crash, resume from last checkpoint.

### 4. Backward Compatibility
- `ODYSSEUS_LANGGRAPH=off` → use existing `stream_agent_loop` (100% backward compat)
- `ODYSSEUS_LANGGRAPH=on` → use new LangGraph loop

### 5. Testing
Test each node independently. Test full graph with mock LLM.

### 6. Human-in-the-Loop
Before `build` node: if `risk_level == DESTRUCTIVE`, `interrupt()` waits for human approval.

## VERIFICATION
- Byte-identical output for same input (kill-switch OFF vs LangGraph ON)
- Each node testable independently
- Checkpoint: crash mid-loop → resume from last checkpoint
- All 4,393 existing tests pass (no regression)

## OUTPUT
- `src/orchestrator/langgraph_loop.py` (new)
- Modified `src/agent_loop.py` (add kill-switch routing)
- Node-level tests
