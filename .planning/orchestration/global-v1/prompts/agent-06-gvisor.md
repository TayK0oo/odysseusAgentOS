# Agent A6: gVisor — Kernel-Level Sandbox

## TASK
Add gVisor (Google, Apache 2.0) as a kernel-level sandbox runtime for Odysseus's MCP containers — 1 line per service in docker-compose.yml.

## CONTEXT
- Current sandbox: Docker `cap_drop: ALL` + `no-new-privileges:true`
- Limitation: Still shares host kernel. Malicious code in MCP containers could exploit kernel vulnerabilities.
- gVisor: User-space kernel (Go) that intercepts ALL syscalls. Used by Google Cloud Run, Docker, Kubernetes.

## REQUIREMENTS

### 1. Documentation
Add to `docs/setup.md`:
```markdown
## gVisor Sandbox (Optional)
gVisor provides kernel-level isolation for MCP containers.
Install: https://gvisor.dev/docs/user_guide/install/
Then enable in docker-compose: set `ODYSSEUS_GVISOR=on`
```

### 2. Docker Compose
Add `runtime: ${ODYSSEUS_GVISOR_RUNTIME:-runc}` to MCP services:
- serena-mcp
- scrapling-mcp
- codebase-memory
- graphify
- decision-engine

Example:
```yaml
serena-mcp:
  runtime: ${ODYSSEUS_GVISOR_RUNTIME:-runc}
```

### 3. Env Config
In `.env.example`:
```bash
# gVisor kernel sandbox (requires: https://gvisor.dev/docs/user_guide/install/)
ODYSSEUS_GVISOR=off
ODYSSEUS_GVISOR_RUNTIME=runsc  # Set to 'runc' to disable
```

### 4. Security Docs
Update `SECURITY.md`:
- Document gVisor as additional sandbox layer
- Performance overhead: ~5-15% for I/O-heavy workloads
- Not required for local development

### 5. Kill-Switch
`ODYSSEUS_GVISOR=off` → use default runc (no sandbox)
`ODYSSEUS_GVISOR=on` → use runsc (gVisor sandbox)

## VERIFICATION
- `docker info | grep runsc` confirms gVisor runtime installed
- `ODYSSEUS_GVISOR=on docker compose up serena-mcp` starts with runsc
- `docker inspect serena-mcp | grep Runtime` shows "runsc"

## OUTPUT
Files modified, setup instructions, security doc update
