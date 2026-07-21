# Services Docker

Liste complète des services du stack Odysseus, leurs ports, et comment les activer.

## Services actifs par défaut

Ces services démarrent avec `docker compose up -d`.

| Service      | Port hôte | Port conteneur | Description                          |
|--------------|-----------|----------------|--------------------------------------|
| odysseus     | 7000      | 7000           | Application principale FastAPI       |
| chromadb     | 8100      | 8000           | Vector store ChromaDB                |
| searxng      | 8080      | 8080           | Moteur de recherche SearXNG          |
| ntfy         | 8091      | 80             | Notifications push                   |
| kroki        | 8700      | 8000           | Rendu de diagrammes (Mermaid, PlantUML, etc.) |
| serena-mcp   | 8765      | 8765           | Édition symbolique LSP via MCP (SSE) |

## Services avec profils (désactivés par défaut)

Ces services nécessitent un profil explicite pour démarrer.

| Service          | Port hôte | Profil           | Description                                      |
|------------------|-----------|------------------|--------------------------------------------------|
| acontext         | 8029      | `acontext`       | Mémoire Acontext self-hosted (placeholder)       |
| scrapling-mcp    | 8800      | `scrapling`      | Web scraping via MCP                             |
| codebase-memory  | 9749      | `knowledge`      | Codebase Memory MCP (CBM / Trinité)              |
| decision-engine  | 8001      | `decision-engine`| Decision Engine depuis ConfigOpenCodeNew         |

## Commandes d'activation

```bash
# Stack minimal (défaut)
docker compose up -d

# Avec scraping web
docker compose --profile scrapling up -d

# Avec knowledge graph (CBM)
docker compose --profile knowledge up -d

# Avec decision engine
docker compose --profile decision-engine up -d

# Avec mémoire Acontext
docker compose --profile acontext up -d

# Tout activer
docker compose --profile scrapling --profile knowledge --profile decision-engine --profile acontext up -d
```

## Notes par service

### kroki
Image officielle `yuzutech/kroki`. Supporte Mermaid, PlantUML, GraphViz, D2, etc. Healthcheck sur `/health`.

### serena-mcp
Démarre Serena via `uvx` depuis le repo GitHub `oraios/serena`. Monte le workspace en lecture seule. Transport SSE sur le port 8765.

### acontext
Placeholder — l'image officielle `acontextio/acontext` n'est pas encore publiée sur Docker Hub. Activer manuellement une fois l'image disponible et remplacer le placeholder dans `docker-compose.yml`.

### scrapling-mcp
Installe `scrapling[all]` au démarrage. Premier boot lent (installation des navigateurs). Désactivé par défaut via profil `scrapling`.

### codebase-memory
Image `ghcr.io/superpowers-sh/codebase-memory-mcp`. Monte le workspace en lecture seule. Données persistées dans `./data/cbm`.

### decision-engine
Requiert que le dossier `decision-engine` de `ConfigOpenCodeNew` soit accessible. Configurer `DECISION_ENGINE_PATH` dans `.env` si le repo est ailleurs.
