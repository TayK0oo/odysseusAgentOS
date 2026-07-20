# Agent A17: n8n — Workflow Automation

## TASK
Integrate n8n (fair-code, self-hosted free, 400+ connectors, visual workflow editor) to replace hardcoded channel adapters and scheduled tasks with visual drag-and-drop workflows.

## CONTEXT
- Current: `src/channel_gateway.py`, `src/channel_bootstrap.py`, `src/task_scheduler.py` — all code-based integration.
- n8n: Visual workflow editor. 400+ pre-built nodes. Webhook triggers. AI nodes (LLM chain, vector store, agent).

## REQUIREMENTS

### 1. Docker Compose
Add n8n (profile `automation`):
```yaml
n8n:
  image: n8nio/n8n:latest
  ports: ["127.0.0.1:5678:5678"]
  environment:
    N8N_SECURE_COOKIE: "false"
    N8N_HOST: localhost
  volumes:
    - n8n-data:/home/node/.n8n
  profiles: ["automation"]
```

### 2. Pre-built Workflow Templates
Create `docs/n8n-workflows.md` with templates:

**Workflow 1: Email → Summary → Notification**
- Trigger: IMAP email received
- Action: Send to LLM for summarization
- Action: Notify via Apprise (ntfy/Discord/Telegram)

**Workflow 2: Webhook → Agent → Response**
- Trigger: Incoming webhook
- Action: Call Odysseus agent via API
- Action: Return response to webhook caller

**Workflow 3: RSS → Deep Research → Report**
- Trigger: RSS feed update
- Action: Deep research on new items
- Action: Generate report → save to Documents

### 3. API Integration
Add endpoint for triggering n8n workflows from Odysseus:
`POST /api/n8n/trigger/{workflow_id}` → proxy to n8n webhook

### 4. Kill-Switch
`ODYSSEUS_N8N=off` → use hardcoded adapters (current)

## VERIFICATION
- n8n UI accessible at `:5678`
- Import workflow templates → execute → receive notification
- Webhook → Agent → Response round-trip works

## OUTPUT
- Docker Compose changes
- `docs/n8n-workflows.md`
- API endpoint
