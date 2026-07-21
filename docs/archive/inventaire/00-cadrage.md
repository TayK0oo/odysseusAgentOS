# 00 — CADRAGE DU TERRITOIRE

**Date** : 2026-07-07
**Commit** : `2821daa26227683476b8dea4fd5007d8ede73a21`
**Racine** : `C:\Users\ttmdu\Documents\GitHub\odysseusAgentOS`
**Branche** : `dev`

---

## 1. STACKS DÉTECTÉS

| Catégorie | Technologie | Fichier manifeste | Preuve |
|---|---|---|---|
| Backend | Python 3.14 (FastAPI + Uvicorn) | `pyproject.toml` + `requirements.txt:5-6` | app.py:120-125 |
| LLM Routing | LiteLLM (abstraction multi-provider) | `requirements.txt:1-2` | model-routing.json:1-136 |
| Frontend | HTML/CSS/JS natif (SPA, pas de framework) | `static/index.html`, `static/js/` (474+ fichiers) | app.py:881-907 (routes SPA) |
| Dev Node | Node.js (purgecss, faker) | `package.json:10-14` | purgecss, @faker-js/faker |
| DB | SQLite (via SQLAlchemy) | `requirements.txt:11` | docker-compose.yml:45 (DATABASE_URL) |
| Vecteur | ChromaDB (client HTTP) | `requirements.txt:21` | docker-compose.yml:160-178 |
| Cache embeddings | Fastembed (ONNX local) | `requirements.txt:22` | docker-compose.yml:55 |
| RAG | BM25 (rank-bm25) | `requirements.txt:23` | utilisé via rag_vector.py |
| MCP | mcp (Python SDK) | `requirements.txt:44` | mcp_servers/ (6 fichiers) |
| Auth | bcrypt, pyotp (TOTP) | `requirements.txt:42,45` | app.py:76,308-427 |
| CalDAV | caldav, icalendar, python-dateutil | `requirements.txt:32,37,41` | routes/calendar_routes.py |
| Channels | discord.py 2.4+, python-telegram-bot 21+ | `requirements.txt:56-57` | docker-compose.yml:97-103 |
| Sécurité | cryptography, nh3 (HTML sanitizer) | `requirements.txt:31,42` | docs/security.md |
| Conteneurisation | Docker (python:3.14-slim) | `Dockerfile:12` | docker-compose.yml:1-152 |
| Desktop | PyInstaller (Odysseus.spec) | `Odysseus.spec` | build-windows-portable.ps1 |
| CI/CD | GitHub Actions | `.github/workflows/` (9 fichiers) | ci.yml, docker-publish.yml, etc. |

---

## 2. POINTS D'ENTRÉE

| Point d'entrée | Type | Mécanisme | Preuve |
|---|---|---|---|
| `app.py` | Serveur FastAPI | `uvicorn.run(app)` | app.py:1302 |
| `Dockerfile` | Conteneur | `CMD uvicorn app:app --port 7000` | Dockerfile:97 |
| `launcher.py` | GUI Desktop | Tkinter + bootstrap | launcher.py |
| `launch-windows.ps1` | Script Windows | PowerShell | launch-windows.ps1 |
| `build-windows-portable.ps1` | Build Desktop | PyInstaller | build-windows-portable.ps1 |
| `build-macos-app.sh` | Build macOS | shell script | build-macos-app.sh |
| `install-service.sh` | Service Linux | systemd | install-service.sh |
| `scripts/odysseus` | CLI wrapper | shell | scripts/odysseus |
| `scripts/odysseus-mail`, etc. | CLI tools | shell (+20 scripts) | scripts/odysseus-* |

---

## 3. ARBORESCENCE RÉSUMÉE

```
odysseusAgentOS/
├── app.py                      # Point d'entrée FastAPI (1302 lignes)
├── Dockerfile / docker-compose.yml
├── requirements.txt / pyproject.toml / package.json
├── model-routing.json           # Politique de routing modèle (OmO)
├── loop-canonique.md            # Spéc 7 phases
├── permission-matrix.md         # Matrice outils × phases
│
├── core/                        # 11 modules : auth, DB, sessions, middleware, exceptions...
├── routes/                      # 60+ routeurs FastAPI
├── src/                         # Logique métier
│   ├── orchestrator/            # 16 modules : loop, phases, gate, dispatcher, registry...
│   ├── adapters/                # Discord, Telegram
│   ├── agent_tools/             # Outils exposés aux agents
│   ├── tools/                   # Outils internes
│   └── search/                  # Search engines
├── services/                    # YouTube, TTS, STT
├── mcp_servers/                 # 6 serveurs MCP (email, graphify, memory, obsidian, rag, image_gen)
├── companion/                   # pairing, routes (mobile companion)
├── config/
│   └── phase-lock.yaml          # Phases canoniques + restrictions outils
├── static/                      # Frontend SPA (~474 fichiers)
│   ├── index.html, login.html, backgrounds.html
│   ├── css/                     # style.min.css
│   ├── js/                      # Modules ES : calendar, compare, editor, emailLibrary...
│   ├── fonts/, icons/, lib/
├── .opencode/agents/            # 12 agents Markdown
├── .planning/                   # GSD : STATE, ROADMAP, intel/
├── .github/workflows/           # 9 CI/CD workflows
├── data/                        # Runtime : DB, cache, uploads, traces...
├── docs/                        # Documentation + audits + superpowers
├── integrations/                # Claude, Codex
├── scripts/                     # 30+ utilitaires
├── sandbox/                     # Fixtures TestEngineer
└── docker/                      # entrypoint + build GPU
```

---

## 4. INDICATEURS GLOBAUX

| Métrique | Valeur | Source |
|---|---|---|
| Lignes de code Python (est.) | ~130K+ | `app.py:1302` + `routes/` (60 fichiers) + `src/` (20+ modules) |
| Fichiers Python | ~180+ | comptage `src/` + `routes/` + `core/` + `services/` + `mcp_servers/` |
| Fichiers statiques frontend | ~474 | `static/` tree |
| Agents OpenCode (.md) | 12 | `.opencode/agents/` |
| Routes API enregistrées | 60+ routeurs | `app.py` boucle `include_router` |
| Services Docker Compose | 7 (+ profiles) | odysseus, chromadb, searxng, ntfy, kroki (+mermaid), serena-mcp, acontext, scrapling-mcp, codebase-memory, graphify, decision-engine |
| Kill-switches env | 35+ | `docker-compose.yml:85-133` |
| Phases canoniques | 10 (config) / 7 (loop) | `config/phase-lock.yaml` / `loop-canonique.md` |
| Tests passés (HEAD) | 4196 passed, 123 failed (env.) | STATE.md:132 |
| Nœuds CBM | 11749 | `list_projects()` |
| Arêtes CBM | 38126 | `list_projects()` |

---

## 5. OUTILS DE CARTOGRAPHIE DISPONIBLES

| Outil | État | Note |
|---|---|---|
| **CBM** (codebase-memory-mcp) | 🟢 Disponible | 11749 nœuds, 38126 arêtes, mode full |
| **Serena** (MCP) | 🟡 Configuré (docker-compose) | Service `serena-mcp` défini, port 8765 — NON VÉRIFIÉ si actif |
| **Graphify** (MCP) | 🔵 Configuré-dormant | Service défini mais `0 svc` selon STATE.md:46 |
| **Obsidian** (MCP) | 🔵 Configuré-dormant | `mcp_servers/obsidian_mcp.py` codé, gate OFF par défaut |
| **grep/glob/bash** | 🟢 Toujours disponible | Fallback |
