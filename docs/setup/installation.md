# Setup — Installation

## Prérequis

- **Docker** et Docker Compose v2+
- **Python 3.14** (pour le développement natif)
- **Node.js** (pour PurgeCSS, tests JS)
- **Git**

## Installation rapide (Docker)

```bash
git clone https://github.com/pewdiepie-archdaemon/odysseus.git
cd odysseus
cp .env.example .env
# Éditer .env : ajouter les clés API (OPENAI_API_KEY, etc.)
docker compose up -d
```

L'application est accessible sur `http://localhost:7000`.

## Installation native (développement)

```bash
python -m venv venv
source venv/bin/activate  # ou venv\Scripts\activate sur Windows
pip install -r requirements.txt
npm install
npm run css:build
python app.py
```

## Profils Docker Compose

| Profil | Commande | Services ajoutés |
|--------|----------|-----------------|
| `knowledge` | `--profile knowledge` | codebase-memory, graphify |
| `acontext` | `--profile acontext` | acontext (distillation mémoire) |
| `scrapling` | `--profile scrapling` | scrapling-mcp |
| `observability` | `--profile observability` | langfuse-server (+ postgres) |
| `automation` | `--profile automation` | n8n |
| `security` | `--profile security` | opa |
| `vectordb` | `--profile vectordb` | qdrant |
| `production` | `--profile production` | postgres (+ pgvector) |
| `localai` | `--profile localai` | localai |
| `gateway` | `--profile gateway` | traefik |
| `decision-engine` | `--profile decision-engine` | decision-engine |

Exemple avec plusieurs profils :
```bash
docker compose --profile knowledge --profile observability up -d
```

## GPU (NVIDIA/AMD)

```bash
# NVIDIA
docker compose -f docker-compose.yml -f docker-compose.gpu-nvidia.yml up -d

# AMD
docker compose -f docker-compose.yml -f docker-compose.gpu-amd.yml up -d
```

---

→ Voir aussi : [Configuration](configuration.md) · [Prérequis](prerequisites.md) · [Déploiement](../operations/deployment.md)
