# Agent A15: OPA — Policy-as-Code for Agent Authorization

## TASK
Replace hardcoded `phase-lock.yaml` and `destructive_gate` with OpenPolicyAgent (CNCF, Apache 2.0) Rego policies — making agent authorization auditable, testable, and hot-reloadable.

## CONTEXT
- Current: `config/phase-lock.yaml` (114 lines, hardcoded restrictions) + `src/orchestrator/gate.py` (hardcoded pattern checks).
- Problem: Policies are code, not config. Change = redeploy. No audit trail of decisions.
- OPA: Declarative Rego policies. Decisions are logged with reasons. Hot-reload without restart.

## REQUIREMENTS

### 1. Docker Compose
Add OPA to `docker-compose.yml` (profile `security`):
```yaml
opa:
  image: openpolicyagent/opa:latest
  command: ["run", "--server", "/policies"]
  ports: ["127.0.0.1:8181:8181"]
  volumes:
    - ./config/policies:/policies
  profiles: ["security"]
```

### 2. Rego Policies
Create `config/policies/tool_access.rego`:
```rego
package odysseus.tool

default allow = false

allow {
    input.phase == "BUILD"
}

allow {
    input.phase == "PLAN"
    input.tool_category == "READ"
}

allow {
    input.phase == "KNOW"
    input.tool_category in ["READ", "SEARCH"]
}

# Destructive tools require approval
allow {
    input.tool_category == "DESTRUCTIVE"
    input.approved == true
}

deny_reason[msg] {
    not allow
    msg := sprintf("Tool %s blocked in phase %s (category: %s)", 
        [input.tool, input.phase, input.tool_category])
}
```

Create `config/policies/phase_lock.rego`:
```rego
package odysseus.phase

# Phase transition rules
allow_transition {
    input.from == "CLASSIFY"
    input.to == "KNOW"
}
# ... all valid transitions
```

### 3. Python Client
Create `services/security/opa_client.py`:
```python
class OPAClient:
    async def check_tool_access(self, phase, tool, category):
        result = await self._query("odysseus/tool/allow", {
            "input": {"phase": phase, "tool": tool, "tool_category": category}
        })
        return result["result"]
```

### 4. Integration
Replace calls to `phase-lock.yaml` and `destructive_gate` with `OPAClient.check_tool_access()`.

### 5. Testing
Create `config/policies/tool_access_test.rego`:
```rego
test_build_phase_allows_write {
    allow with input as {"phase": "BUILD", "tool_category": "WRITE"}
}
```

### 6. Kill-Switch
`ODYSSEUS_OPA=off` → use hardcoded policies (current behavior)

## VERIFICATION
- `opa test config/policies/` passes all policy tests
- Tool blocked in KNOW phase → OPA returns deny with reason
- Policy hot-reloaded without restart

## OUTPUT
- `config/policies/tool_access.rego`
- `config/policies/phase_lock.rego`
- `services/security/opa_client.py`
- Policy test files
