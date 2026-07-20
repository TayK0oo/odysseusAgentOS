# VAGUE 1 — Quick Wins (6 agents, ~13h effort, impact maximal)

**Objectif:** Intégrer 6 outils à faible complexité et fort impact immédiat.
**Stratégie:** Parallèle, 6 agents simultanés. Aucune dépendance entre agents.

---

## A1: Meilisearch — Full-Text Search

**Outil:** Meilisearch (MIT, Rust, <50ms, typo-tolerant)
**Prompt:** `prompts/agent-01-meilisearch.md`
**Durée estimée:** 2h

### Tâches:
1. Ajouter `meilisearch` au `docker-compose.yml` (profile default)
2. Créer `services/search/meilisearch_client.py` — wrapper Python
3. Ajouter endpoint `POST /api/search/fulltext` dans `routes/search_routes.py`
4. Indexer automatiquement: chat messages, notes, documents
5. Ajouter barre de recherche omnibox dans `static/js/search-chat.js`
6. Kill-switch: `ODYSSEUS_MEILISEARCH=off`

### Fichiers modifiés:
- `docker-compose.yml` (+15 lignes)
- `services/search/meilisearch_client.py` (nouveau, ~80 lignes)
- `routes/search_routes.py` (+30 lignes)
- `static/js/search-chat.js` (+40 lignes)
- `src/config.py` (+5 lignes)

---

## A2: Apprise — Notifications Unifiées

**Outil:** Apprise (BSD, 100+ canaux, 3 lignes Python)
**Prompt:** `prompts/agent-02-apprise.md`
**Durée estimée:** 1h

### Tâches:
1. Ajouter `apprise` à `requirements.txt`
2. Créer `services/notifications/apprise_service.py`
3. Remplacer les appels directs Discord/Telegram/ntfy par Apprise
4. Ajouter config UI dans Settings
5. Kill-switch: `ODYSSEUS_APPRISE=off`

### Fichiers modifiés:
- `requirements.txt` (+1 ligne)
- `services/notifications/apprise_service.py` (nouveau, ~60 lignes)
- `src/channel_gateway.py` (refactor, -50 lignes)
- `src/channel_bootstrap.py` (-30 lignes)
- `docker-compose.yml` (optionnel, apprise-api en profile)

---

## A3: Docling — Traitement Documents

**Outil:** Docling (MIT, IBM, extraction tableaux/layout PDF)
**Prompt:** `prompts/agent-03-docling.md`
**Durée estimée:** 3h

### Tâches:
1. Ajouter `docling` à `requirements-optional.txt`
2. Créer `services/documents/docling_processor.py`
3. Remplacer `pypdf` par Docling pour l'extraction PDF
4. Ajouter extraction tableaux → Markdown
5. Préserver layout multi-colonnes
6. Kill-switch: `ODYSSEUS_DOCLING=off`

### Fichiers modifiés:
- `requirements-optional.txt` (+1 ligne)
- `services/documents/docling_processor.py` (nouveau, ~120 lignes)
- `src/document_processor.py` (+20 lignes)
- `routes/document_routes.py` (+15 lignes)

---

## A4: Mem0 — Mémoire Agent

**Outil:** Mem0 (Apache 2.0, auto-extraction faits, wrap existant)
**Prompt:** `prompts/agent-04-mem0.md`
**Durée estimée:** 2h

### Tâches:
1. Ajouter `mem0ai` à `requirements.txt`
2. Créer `services/memory/mem0_provider.py`
3. Wrapper Mem0 autour de ChromaDB existant
4. Auto-extraction faits depuis conversations
5. Endpoint `GET /api/memory/facts`
6. Kill-switch: `ODYSSEUS_MEM0=off`

### Fichiers modifiés:
- `requirements.txt` (+1 ligne)
- `services/memory/mem0_provider.py` (nouveau, ~100 lignes)
- `routes/memory_routes.py` (+25 lignes)
- `src/memory_provider.py` (+20 lignes)

---

## A5: Tailwind CSS — Design System

**Outil:** Tailwind CSS (MIT, JIT compiler, utility-first)
**Prompt:** `prompts/agent-05-tailwind.md`
**Durée estimée:** 4h

### Tâches:
1. Ajouter Tailwind standalone CLI au build
2. Configurer `tailwind.config.js` avec design tokens Odysseus
3. Scanner `static/**/*.{html,js}` pour génération JIT
4. Remplacer `style.css` par `style.min.css` généré
5. Cible: <200KB (actuel: 1.22MB)
6. Ajouter script `npm run css:build`

### Fichiers modifiés:
- `package.json` (+3 lignes)
- `tailwind.config.js` (nouveau, ~50 lignes)
- `static/style.css` → remplacé par import Tailwind
- `static/index.html` (classes Tailwind sur composants clés)
- `.dockerignore` (+1 ligne)

---

## A6: gVisor — Sandbox Kernel

**Outil:** gVisor (Apache 2.0, Google, user-space kernel)
**Prompt:** `prompts/agent-06-gvisor.md`
**Durée estimée:** 1h

### Tâches:
1. Documenter installation `runsc` dans `docs/setup.md`
2. Ajouter `runtime: runsc` aux services MCP dans docker-compose
3. Optionnel: profile `sandbox` pour activation gVisor
4. Mettre à jour `SECURITY.md`
5. Kill-switch: `ODYSSEUS_GVISOR=off`

### Fichiers modifiés:
- `docker-compose.yml` (+6 lignes par service MCP)
- `docs/setup.md` (+30 lignes)
- `SECURITY.md` (+20 lignes)

---

## CHECKLIST DE VÉRIFICATION VAGUE 1

- [ ] Tous les 6 agents ont commité
- [ ] `docker compose up` démarre sans erreur
- [ ] Tests: même nombre de pass qu'avant (pas de régression)
- [ ] Chaque outil a son kill-switch dans `.env.example`
- [ ] Chaque outil est documenté dans `docs/services.md`
