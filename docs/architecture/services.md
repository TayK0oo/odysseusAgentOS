# Architecture — Services

## Services Docker (21 au total)

### Core (profil default)

| Service | Port | Rôle | Kill-switch |
|---------|------|------|-------------|
| **odysseus** | 7000 | Application principale FastAPI | — |
| **chromadb** | 8100 | Base vectorielle (RAG, mémoire) | — |
| **searxng** | 8080 | Meta-search engine (pinned 2026.5.31) | — |
| **ntfy** | 8091 | Notifications push | — |
| **kroki** | 8700 | Rendu de diagrammes (Mermaid, PlantUML) | `KROKI_ENABLED` |
| **kroki-mermaid** | 8002 | Compagnon Mermaid pour Kroki | — |
| **serena-mcp** | 8765 | Code intelligence LSP (MCP) | — |
| **meilisearch** | 7700 | Full-text search | `ODYSSEUS_MEILISEARCH` |

### Profil `knowledge`

| Service | Port | Rôle |
|---------|------|------|
| **codebase-memory** | 9749 | CBM — graphe de code (11,749 nœuds) |
| **graphify** | 9750 | Graphe sémantique | `ODYSSEUS_GRAPHIFY` |

### Profil `observability`

| Service | Port | Rôle |
|---------|------|------|
| **langfuse-postgres** | — | PostgreSQL pour LangFuse |
| **langfuse-server** | 3000 | LLM tracing dashboard | `ODYSSEUS_LANGFUSE` |

### Profil `automation`

| Service | Port | Rôle |
|---------|------|------|
| **n8n** | 5678 | Workflow automation (400+ connecteurs) | `ODYSSEUS_N8N` |

### Profil `security`

| Service | Port | Rôle |
|---------|------|------|
| **opa** | 8181 | Policy engine (Rego) | `ODYSSEUS_OPA` |

### Profil `vectordb`

| Service | Port | Rôle |
|---------|------|------|
| **qdrant** | 6333 | Base vectorielle alternative (Rust) | `ODYSSEUS_QDRANT` |

### Profil `production`

| Service | Port | Rôle |
|---------|------|------|
| **postgres** | 5432 | PostgreSQL 16 + pgvector |

### Autres profils

| Service | Profil | Port | Rôle |
|---------|--------|------|------|
| **acontext** | `acontext` | 8029 | Distillation mémoire → SKILL.md |
| **scrapling-mcp** | `scrapling` | 8800 | Web scraping MCP |
| **localai** | `localai` | 8081 | LLM+embeddings+TTS+STT unifié |
| **traefik** | `gateway` | 80/443 | Reverse proxy + Let's Encrypt |
| **decision-engine** | `decision-engine` | 8001 | Orchestrateur externe |

## Services Python (58 fichiers)

| Service | Fichiers | Rôle |
|---------|----------|------|
| `search/` | 10 | Multi-provider web search + Meilisearch |
| `memory/` | 9 | Mémoire, extraction, skills, Mem0, Letta |
| `hwfit/` | 6 | Détection hardware, fit modèles |
| `research/` | 3 | Deep research handler |
| `pipelines/` | 3 | Prefect (RAG, maintenance, modèles) |
| `observability/` | 2 | LangFuse + OpenTelemetry |
| `tts/`, `stt/` | 4 | Text-to-speech, speech-to-text |
| `youtube/` | 2 | Transcripts YouTube |
| `shell/` | 2 | Exécution shell sécurisée |
| `code/` | 2 | Tree-sitter parsing |
| `documents/` | 2 | Docling PDF processing |
| `vector/` | 2 | Qdrant store |
| `security/` | 2 | OPA client |
| `notifications/` | 2 | Apprise (100+ canaux) |

## MCP Servers (6 built-in)

| Serveur | Protocole | Rôle |
|---------|-----------|------|
| `email_server.py` | SSE | Email MCP |
| `graphify_mcp.py` | SSE | Graphe sémantique |
| `memory_server.py` | SSE | Mémoire MCP |
| `obsidian_mcp.py` | stdio | Obsidian vault |
| `image_gen_server.py` | SSE | Génération images |
| `rag_server.py` | SSE | RAG MCP |

---

→ Voir aussi : [Vue d'ensemble](overview.md) · [Boucle agent](agent-loop.md) · [INDEX-MAITRE](../../.planning/INDEX-MAITRE.md)
