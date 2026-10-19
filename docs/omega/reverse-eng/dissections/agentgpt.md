# Dissection AgentGPT

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Reworkd |
| Licence | GPL-3.0 |
| Langage | TypeScript (frontend + CLI), Python (backend) |
| Repo | https://github.com/reworkd/AgentGPT |
| Étoiles | 36 279 |

**Architecture :** Monorepo hybride avec trois composantes principales :
- `next/` — Application web Next.js (React + tRPC + Prisma/MySQL) pour l'UI et la persistance
- `platform/` — Serveur Python FastAPI (reworkd_platform) pour l'exécution des agents autonomes via LangChain + OpenAI
- `cli/` — CLI TypeScript pour le setup interactif

Le frontend communique avec le backend via API REST/SSE. Le backend orchestre la boucle complète : objectif → décomposition en tâches → analyse (function calling) → exécution (outils) → synthèse.

## 2. Entrées

- **next/** : App web Next.js (port 3000), servit via `next start` ou `next dev`
- **platform/** : Serveur FastAPI (port 8000), lancé via `python -m reworkd_platform` ou uvicorn
- **cli/** : CLI de setup interactif, lancé via `npx` ou `node`

**Points d'extension :**
- Ajout d'outils via la classe `Tool` (ABC Python)
- Ajout de providers LLM via `model_factory.py`
- Ajout de providers OAuth via `oauth_installers.py`
- Ajout de routes tRPC via `agentRouter.ts`
- Ajout de prompts via `prompts.py`

**Fichiers de config :**
- `.env` (variables d'environnement, voir `settings.py`)
- `next.config.mjs` (config Next.js)
- `prisma/schema.prisma` (schéma de base de données)

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Agent autonome objectif → tâches → exécution | agents | Boucle complète de décomposition et d'exécution | `autonomous-agent.ts:17-169`, `open_ai_agent_service.py:43-227` | plugins/agent-autonome | planned | L |
| Registre d'outils (Tool ABC) | outils | Système d'outils extensible avec disponibilité dynamique | `tool.py:11-41`, `tools.py:12-78` | omega-context::ToolSurfaceIndex | adopted | M |
| Recherche Google (Serper) | outils | Recherche web avec fallback raisonnement | `search.py:42-109` | omega-context::ToolSurfaceIndex | planned | S |
| Génération d'images (Replicate/DALL-E) | outils | Génération d'images avec fallback | `image.py:47-66` | omega-context::ToolSurfaceIndex | planned | S |
| Recherche privée SID | outils | Recherche Notion/GDrive avec OAuth | `sidsearch.py:72-143` | omega-context::ToolSurfaceIndex | planned | M |
| Exécution de code LLM | outils | Génération/revue de code via LLM | `code.py:10-25` | omega-context::ToolSurfaceIndex | planned | S |
| Raisonnement LLM | outils | Décision/sélection via LLM | `reason.py:10-27` | omega-context::ToolSurfaceIndex | planned | S |
| Mémoire vectorielle avec fallback | memoire | Pinecone avec fallback automatique | `memory_with_fallback.py:10-53` | omega-context::DefaultContextEngine | planned | M |
| Persistance Prisma (MySQL) | execution | CRUD agents/tâches via Prisma | `schema.prisma:1-150` | omega-app::FileEventStore | planned | M |
| API tRPC typée | execution | CRUD agents avec validation Zod | `agentRouter.ts:56-124` | omega-app::FileEventStore | planned | M |
| Auth NextAuth | securite | OAuth Google/GitHub/Discord | `auth.ts:9-30` | omega-config (auth) | planned | S |
| OAuth installers (SID) | securite | Installation OAuth avec chiffrement | `oauth_installers.py:18-147` | omega-config (oauth) | planned | M |
| Service de tokens (tiktoken) | contexte | Comptage et budget de tokens | `token_service.py:7-33` | omega-context::DefaultContextEngine | planned | S |
| Streaming SSE | execution | Streaming réponses LLM | `stream_mock.py:11-26` | omega-app::FileEventStore | adopted | S |
| Factory de modèles (OpenAI/Azure/Helicone) | routage | Multi-provider LLM avec proxy | `model_factory.py:29-97` | omega-providers::OllamaProvider | adopted | M |
| Parseur de sortie tâches | execution | Extraction JSON/multiline robuste | `task_output_parser.py:8-88` | omega-otel::TraceReplayer | adopted | S |
| Mock agent service | configuration | Développement sans appel API | `mock_agent_service.py:13-74` | omega-config (feature flags) | planned | S |
| Settings Pydantic | configuration | Config centralisée validée | `settings.py:34-178` | omega-config (couches) | adopted | S |
| S3 presigned URLs | outils | Upload/download via URLs presignées | `s3.py:19-85` | omega-context::ToolSurfaceIndex | planned | M |
| Prompts Plan-and-Solve | contexte | Décomposition d'objectifs | `prompts.py:5-92` | omega-context::DefaultContextEngine | planned | S |
| Analyse de tâche (function calling) | agents | Sélection d'outil via function calling | `analysis.py:6-45` | plugins/agent-autonome | planned | M |
| CLI de setup | configuration | Configuration initiale interactive | `cli/src/index.js` | omega-config (init) | planned | S |
| UI Next.js | ui | Interface React + Tailwind | `ChatWindow.tsx`, `agentStore.ts` | native | planned | L |

## 4. Traduction en plugins

### 4.1 Agent autonome (plugins/agent-autonome)
- **Capacité :** agent-store
- **Port :** AgentStore (discover/load)
- **Adaptateur :** plugins/agent-autonome
- **Mode :** plugin
- **Coût :** L
- **Notes :** La boucle complète (objectif → tâches → exécution → synthèse) est à réimplémenter en Rust. Le workLog TypeScript et l'OpenAIAgentService Python sont les références.

### 4.2 Registre d'outils (omega-context::ToolSurfaceIndex)
- **Capacité :** tool-surface
- **Port :** ToolSurface (index/hydrate)
- **Adaptateur :** omega-context::ToolSurfaceIndex
- **Mode :** plugin
- **Coût :** M
- **Statut :** adopted
- **Notes :** La classe Tool (ABC) avec available() et dynamic_available() est déjà couverte par ToolSurfaceIndex.

### 4.3 Streaming SSE (omega-app::FileEventStore)
- **Capacité :** event-store
- **Port :** EventStore (stream)
- **Adaptateur :** omega-app::FileEventStore
- **Mode :** service
- **Coût :** S
- **Statut :** adopted
- **Notes :** Le streaming via lanarky et stream_string() est déjà couvert par FileEventStore.

### 4.4 Factory de modèles (omega-providers::OllamaProvider)
- **Capacité :** llm-provider
- **Port :** Provider (providers/models)
- **Adaptateur :** omega-providers::OllamaProvider
- **Mode :** plugin
- **Coût :** M
- **Statut :** adopted
- **Notes :** Le support multi-provider (OpenAI/Azure/Helicone) est déjà couvert par OllamaProvider.

### 4.5 Parseur de sortie tâches (omega-otel::TraceReplayer)
- **Capacité :** event-store (replay)
- **Port :** EventStore (replay)
- **Adaptateur :** omega-otel::TraceReplayer
- **Mode :** plugin
- **Coût :** S
- **Statut :** adopted
- **Notes :** Le parsing robuste (JSON/multiline) avec filtrage est déjà couvert par TraceReplayer.

### 4.6 Settings Pydantic (omega-config)
- **Capacité :** config-source
- **Port :** ConfigSource (layers)
- **Adaptateur :** omega-config (couches + precedence)
- **Mode :** plugin
- **Coût :** S
- **Statut :** adopted
- **Notes :** La configuration centralisée avec validation est déjà couvert par omega-config.

## 5. Écarté

| Fonctionnalité | Raison |
|---|---|
| UI Next.js (React + Tailwind) | Le frontend est spécifique à Next.js ; Omega a son propre cockpit React/TS. Les composants ne sont pas réutilisables directement. |
| Persistance Prisma (MySQL) | Omega utilise son propre EventStore ; Prisma est spécifique à l'écosystème Node.js. |
| API tRPC | Omega utilise son propre serveur HTTP/OpenAPI ; tRPC est spécifique à l'écosystème Node.js. |
| Auth NextAuth | Omega a son propre système d'auth ; NextAuth est spécifique à Next.js. |
| S3 presigned URLs | Spécifique à AWS S3 ; Omega peut utiliser d'autres backends de stockage. |
| CLI de setup | Spécifique à l'écosystème Node.js ; Omega a son propre système de configuration. |

## 6. Sources

- **Repo :** https://github.com/reworkd/AgentGPT
- **Clone local :** `/home/tayk0oo/omega-re/agentgpt`
- **Preuve REA :** `docs/omega/research/rea/evidence/intake-agentgpt.summary.json`
- **Limites REA :** L'analyse REA (1854 nœuds, 2055 arêtes) confirme la structure générale mais ne détaille pas les interactions runtime entre les composantes. Les références `file:line` sont basées sur la lecture directe du clone.
