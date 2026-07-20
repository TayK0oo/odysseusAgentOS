# Agent A2: Apprise — Unified Notifications

## TASK
Integrate Apprise (BSD, 100+ notification channels) to replace the custom Discord, Telegram, and ntfy adapters with a single unified notification layer.

## CONTEXT
- Current: `src/channel_gateway.py`, `src/channel_bootstrap.py`, individual Discord/Telegram adapters
- Problem: Each channel requires custom code. Adding a new channel = new Python adapter.
- Apprise solves this: `apobj.add('discord://...'); apobj.notify("msg")` — 3 lines.

## REQUIREMENTS

### 1. Dependency
Add `apprise` to `requirements.txt`

### 2. Service Layer
Create `services/notifications/apprise_service.py`:
```python
class AppriseService:
    def __init__(self):
        self.apobj = apprise.Apprise()
    
    async def notify(self, message, channels=None, tags=None):
        ...
```

### 3. Configuration
In `src/config.py`:
```python
APPRISE_CHANNELS: list[str] = []  # e.g. ["discord://token/webhook_id", "tgram://token/chat_id"]
```

### 4. Refactor Channel Gateway
Replace direct adapter calls in `src/channel_gateway.py` with `AppriseService.notify()`. Keep `ChannelGateway` interface but delegate to Apprise internally.

### 5. Kill-Switch
`ODYSSEUS_APPRISE=off` → fallback to individual adapters

## VERIFICATION
- `pip install apprise` succeeds
- Test notification to ntfy/Discord/Telegram via Apprise
- Existing channel tests pass

## OUTPUT
Files modified, git diff, test results
