# n8n Workflow Templates for Odysseus

Pre-built workflow templates designed for import into the n8n visual editor.
Navigate to **http://localhost:5678** (profile `automation`) and use
**Import from File** to load any JSON below.

> **Kill-switch**: Set `ODYSSEUS_N8N=on` in your `.env` or `docker-compose.yml`
> to activate n8n routing. When off (default), Odysseus uses its hardcoded
> channel adapters.

---

## Workflow 1: Email → Summary → Notification

**Use case**: Automatically summarize incoming emails and push a notification
via Apprise (ntfy, Discord, Telegram, Slack, etc.).

### Nodes

| # | Type | Name | Config |
|---|------|------|--------|
| 1 | **Email Trigger (IMAP)** | Listen Inbox | Host: `imap.gmail.com` · User: `you@gmail.com` · Password: App password · Mailbox: `INBOX` |
| 2 | **AI Agent** (or **HTTP Request**) | Summarize | POST to Odysseus `/api/chat` with prompt: `Summarize this email concisely: {{ $json.text }}` |
| 3 | **HTTP Request** | Send Notification | POST to Apprise webhook with body: `{"title": "📧 {{ $json.subject }}", "body": "{{ $json.summary }}"}` |

### Flow

```
[Email Trigger] → [Summarize via Odysseus] → [Apprise / ntfy / Discord]
```

### JSON Template (n8n import)

```json
{
  "name": "Email → Summary → Notification",
  "nodes": [
    {
      "name": "Email Trigger",
      "type": "n8n-nodes-base.emailReadImap",
      "position": [250, 300],
      "parameters": {
        "mailbox": "INBOX",
        "options": {}
      },
      "typeVersion": 2
    },
    {
      "name": "Summarize",
      "type": "n8n-nodes-base.httpRequest",
      "position": [500, 300],
      "parameters": {
        "method": "POST",
        "url": "http://host.docker.internal:7000/api/chat",
        "sendBody": true,
        "bodyParameters": {
          "parameters": [
            { "name": "message", "value": "Summarize this email concisely: {{ $json.text }}" }
          ]
        }
      },
      "typeVersion": 4
    },
    {
      "name": "Notify",
      "type": "n8n-nodes-base.httpRequest",
      "position": [750, 300],
      "parameters": {
        "method": "POST",
        "url": "http://host.docker.internal:8091/odysseus-summary",
        "sendBody": true,
        "bodyParameters": {
          "parameters": [
            { "name": "title", "value": "📧 Email Summary" },
            { "name": "message", "value": "{{ $json.response }}" }
          ]
        }
      },
      "typeVersion": 4
    }
  ],
  "connections": {
    "Email Trigger": { "main": [[ { "node": "Summarize", "type": "main", "index": 0 } ]] },
    "Summarize": { "main": [[ { "node": "Notify", "type": "main", "index": 0 } ]] }
  }
}
```

---

## Workflow 2: Webhook → Agent → Response

**Use case**: Expose an n8n webhook that triggers an Odysseus agent task and
returns the result synchronously. Ideal for external integrations (Slack
commands, CI/CD, Zapier, Make.com).

### Nodes

| # | Type | Name | Config |
|---|------|------|--------|
| 1 | **Webhook** | Inbound | Method: `POST` · Path: `agent-trigger` · Response: `Last Node` |
| 2 | **HTTP Request** | Call Agent | POST to `http://host.docker.internal:7000/api/chat` with `{{ $json.body }}` |
| 3 | **Respond to Webhook** | Return | Body: `{{ $json }}` (agent response) |

### Flow

```
[Webhook POST] → [Call Odysseus Agent API] → [Respond to Webhook]
```

### JSON Template (n8n import)

```json
{
  "name": "Webhook → Agent → Response",
  "nodes": [
    {
      "name": "Inbound",
      "type": "n8n-nodes-base.webhook",
      "position": [250, 300],
      "parameters": {
        "httpMethod": "POST",
        "path": "agent-trigger",
        "responseMode": "responseNode"
      },
      "webhookId": "agent-trigger",
      "typeVersion": 2
    },
    {
      "name": "Call Agent",
      "type": "n8n-nodes-base.httpRequest",
      "position": [500, 300],
      "parameters": {
        "method": "POST",
        "url": "http://host.docker.internal:7000/api/chat",
        "sendBody": true,
        "bodyParameters": {
          "parameters": [
            { "name": "message", "value": "={{ $json.body.message || $json.body.text }}" }
          ]
        }
      },
      "typeVersion": 4
    },
    {
      "name": "Return",
      "type": "n8n-nodes-base.respondToWebhook",
      "position": [750, 300],
      "parameters": {
        "respondWith": "json",
        "responseBody": "={{ $json }}"
      },
      "typeVersion": 1
    }
  ],
  "connections": {
    "Inbound": { "main": [[ { "node": "Call Agent", "type": "main", "index": 0 } ]] },
    "Call Agent": { "main": [[ { "node": "Return", "type": "main", "index": 0 } ]] }
  }
}
```

### Trigger from Odysseus

```bash
curl -X POST http://localhost:7000/api/n8n/trigger/agent-trigger \
  -H "Content-Type: application/json" \
  -d '{"data": {"message": "Research the latest Rust async runtimes"}}'
```

---

## Workflow 3: RSS → Deep Research → Report

**Use case**: Monitor RSS feeds, trigger deep research on new items, and
generate a Markdown report saved to the vault/Documents.

### Nodes

| # | Type | Name | Config |
|---|------|------|--------|
| 1 | **RSS Feed Read** | Watch Feed | URL: `https://example.com/rss.xml` · Every: `1 hour` |
| 2 | **Filter** | New Only | Only items not seen before (use `dedup` or Compare node) |
| 3 | **HTTP Request** | Deep Research | POST to Odysseus research endpoint with `{{ $json.title }} - {{ $json.link }}` |
| 4 | **Write Binary File** | Save Report | Path: `/app/data/reports/{{ $json.item.guid }}.md` · Content: research result |

### Flow

```
[RSS Feed] → [Dedup/Filter] → [Deep Research via Odysseus] → [Save Markdown Report]
```

### JSON Template (n8n import)

```json
{
  "name": "RSS → Deep Research → Report",
  "nodes": [
    {
      "name": "Watch Feed",
      "type": "n8n-nodes-base.rssFeedRead",
      "position": [250, 300],
      "parameters": {
        "url": "https://example.com/rss.xml"
      },
      "typeVersion": 1
    },
    {
      "name": "Research",
      "type": "n8n-nodes-base.httpRequest",
      "position": [500, 300],
      "parameters": {
        "method": "POST",
        "url": "http://host.docker.internal:7000/api/chat",
        "sendBody": true,
        "bodyParameters": {
          "parameters": [
            {
              "name": "message",
              "value": "Deep research on: {{ $json.title }} — {{ $json.link }}. Provide a detailed report with sources."
            }
          ]
        }
      },
      "typeVersion": 4
    },
    {
      "name": "Save Report",
      "type": "n8n-nodes-base.writeBinaryFile",
      "position": [750, 300],
      "parameters": {
        "fileName": "=/app/data/reports/rss-{{ $runIndex }}-{{ $now.toISO() }}.md",
        "dataPropertyName": "={{ $json.response }}"
      },
      "typeVersion": 1
    }
  ],
  "connections": {
    "Watch Feed": { "main": [[ { "node": "Research", "type": "main", "index": 0 } ]] },
    "Research": { "main": [[ { "node": "Save Report", "type": "main", "index": 0 } ]] }
  }
}
```

---

## Setup Guide

### 1. Start n8n

```bash
# With automation profile
docker compose --profile automation up -d n8n

# n8n UI: http://localhost:5678
```

### 2. Enable the kill-switch

Add to `.env`:

```
ODYSSEUS_N8N=on
```

### 3. Import a workflow

1. Open `http://localhost:5678`
2. Go to **Workflows → Import from File**
3. Paste one of the JSON templates above
4. Configure credentials (IMAP, API keys, etc.)
5. **Activate** the workflow

### 4. Verify

```bash
# Health check
curl http://localhost:7000/api/n8n/health

# Trigger workflow 2 (webhook → agent)
curl -X POST http://localhost:7000/api/n8n/trigger/agent-trigger \
  -H "Content-Type: application/json" \
  -d '{"data": {"message": "Hello from Odysseus!"}}'
```

### Network Topology

```
┌─────────────────────────────────────────┐
│  Docker network (profile: automation)   │
│                                         │
│  ┌──────────┐      ┌──────────────┐     │
│  │ Odysseus │◄────►│     n8n      │     │
│  │ :7000    │      │   :5678      │     │
│  └──────────┘      └──────────────┘     │
│       │                   │             │
│  ┌────┴────┐        ┌────┴─────┐       │
│  │ Agents  │        │ 400+     │       │
│  │ LLM API │        │ Connectors│       │
│  └─────────┘        └──────────┘       │
└─────────────────────────────────────────┘
```

When `ODYSSEUS_N8N=off` (default), the trigger endpoint returns `503` and
Odysseus continues to use its built-in channel adapters and task scheduler.
