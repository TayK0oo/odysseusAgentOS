# Integrations — Autres outils

## Discord

```bash
# .env
ODYSSEUS_INPROCESS_DISCORD=on
DISCORD_BOT_TOKEN=votre-token
DISCORD_DEFAULT_CHANNEL_ID=id-canal
```

Adaptateur : `src/adapters/discord_adapter.py`

## Telegram

```bash
# .env
ODYSSEUS_INPROCESS_TELEGRAM=on
TELEGRAM_BOT_TOKEN=votre-token
TELEGRAM_DEFAULT_CHAT_ID=id-chat
```

Adaptateur : `src/adapters/telegram_adapter.py`

## CalDAV (Calendar)

```bash
# Configuration via UI Settings → Calendar
# Supports: Radicale, Nextcloud, Apple, Fastmail
```

Routes : `/api/calendar/*` (19 endpoints)
Sync engine : `src/caldav_sync.py` (620 lignes)
Writeback : `src/caldav_writeback.py` (253 lignes)

## CardDAV (Contacts)

Routes : `/api/contacts/*` (10 endpoints)
Sync via `caldav` library

## Email (IMAP/SMTP)

Routes : `/api/email/*` (47 endpoints)
- IMAP sync avec polling périodique
- SMTP send avec drafts
- Support OAuth Google
- Parsing de threads email
- Résumés et réponses IA

## Notifications (Apprise)

```bash
# .env
ODYSSEUS_APPRISE=on
APPRISE_CHANNELS=discord://...,tgram://...,ntfy://...
```

100+ canaux supportés via une API unique.

## YouTube

Extraction de transcripts via `youtube-transcript-api`.
Routes : `/api/test/youtube`

## Deep Research

Recherche web multi-étapes avec :
- Planification de recherche
- Extraction de sources
- Synthèse de rapports
- Génération HTML

Routes : `/api/research/*` (15 endpoints)

---

→ Voir aussi : [Email/Outlook](email-outlook.md) · [Workflows n8n](n8n-workflows.md) · [Services](../architecture/services.md)
