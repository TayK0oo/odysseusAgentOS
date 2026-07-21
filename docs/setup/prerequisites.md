# Setup — Prérequis

## Système

- **Linux** (recommandé), **macOS** (Apple Silicon supporté), **Windows** (Docker Desktop ou WSL2)
- **Docker** 24+ et Docker Compose v2+
- **Python 3.14** (pour le développement natif)
- **Node.js 18+** (pour PurgeCSS, tests JS, Promptfoo)
- **Git**

## Docker

```bash
# Vérifier l'installation
docker --version      # ≥ 24.0.0
docker compose version # ≥ 2.0.0
```

### GPU (optionnel)

**NVIDIA :**
- NVIDIA drivers + NVIDIA Container Toolkit
- Vérifier : `docker run --rm --gpus all nvidia/cuda:12.0-base nvidia-smi`

**AMD :**
- ROCm drivers
- Vérifier : `docker run --rm --device /dev/kfd --device /dev/dri rocm/dev-ubuntu-22.04 rocminfo`

## Python (développement natif)

```bash
python --version  # 3.14+
pip install -r requirements.txt
pip install -r requirements-optional.txt  # optionnel
```

## Node.js (frontend)

```bash
node --version  # ≥ 18
npm install
npm run css:build  # Génère static/css/style.tailwind.min.css
```

## Services externes requis

| Service | Obligatoire | Rôle |
|---------|------------|------|
| ChromaDB | ✅ Oui | Vector store |
| SearXNG | ✅ Oui | Recherche web |
| LLM (OpenAI/Ollama/autre) | ✅ Oui | Génération de réponses |

## Vérification

```bash
# Docker
docker compose config  # Valide la configuration

# Python
python -c "import app; print('OK')"

# Frontend
node -e "console.log('OK')"
```

---

→ Voir aussi : [Installation](installation.md) · [Configuration](configuration.md) · `docs/setup.md`
