# VAGUE 5 — Productionisation (3 agents, ~11h effort)

**Objectif:** Préparer Odysseus pour le multi-utilisateur et les déploiements production.
**Dépendance:** Vague 4 terminée.

---

## A20: PostgreSQL + pgvector — Base de Données

**Outil:** PostgreSQL 16 + pgvector (PostgreSQL License, MIT-like)
**Prompt:** `prompts/agent-20-postgres.md`
**Durée estimée:** 4h

### Tâches:
1. Ajouter `postgres:16-alpine` au `docker-compose.yml` (profile `production`)
2. Ajouter `asyncpg` et `pgvector` à `requirements.txt`
3. Config `DATABASE_URL=postgresql+asyncpg://...`
4. Script migration SQLite → PostgreSQL: `scripts/migrate_sqlite_to_pg.py`
5. pgvector pour embeddings (alternative à ChromaDB pour petite échelle)
6. Full-text search PostgreSQL (`tsvector`) comme fallback Meilisearch
7. Kill-switch: `ODYSSEUS_POSTGRES=off`

---

## A21: LocalAI — LLM Unifié

**Outil:** LocalAI (MIT, all-in-one: LLM + embeddings + TTS + STT + images)
**Prompt:** `prompts/agent-21-localai.md`
**Durée estimée:** 3h

### Tâches:
1. Ajouter `localai` au `docker-compose.yml` (profile `localai`)
2. Config models YAML pour LLM, embeddings, TTS, STT
3. OpenAI-compatible API → intégration transparente via LiteLLM
4. Remplacer fastembed par LocalAI embeddings (optionnel)
5. Kill-switch: `ODYSSEUS_LOCALAI=off`

### Bénéfice:
Un seul conteneur remplace: Ollama + fastembed + TTS service + STT service + diffusion server.
Simplifie drastiquement le docker-compose pour les déploiements locaux.

---

## A22: Tree-sitter — Parsing Code

**Outil:** Tree-sitter (MIT, incremental parsing, error-tolerant)
**Prompt:** `prompts/agent-22-treesitter.md`
**Durée estimée:** 4h

### Tâches:
1. Ajouter `tree-sitter` + grammaires à `requirements-optional.txt`
2. Créer `services/code/treesitter_parser.py`
3. Fonctions: parse_file, find_function, find_references, syntax_diff
4. Intégrer dans `src/agent_tools/filesystem_tools.py` (read_file avec AST)
5. Complément à CBM: syntax-level vs graph-level
6. Kill-switch: `ODYSSEUS_TREESITTER=off`

---

## CHECKLIST VAGUE 5

- [ ] PostgreSQL: migration script testé, fallback SQLite intact
- [ ] LocalAI: remplace 4 services par 1
- [ ] Tree-sitter: intégré dans filesystem_tools
