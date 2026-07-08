# 01 — RECENSEMENT STRUCTUREL BRUT

**Date** : 2026-07-07 | **Commit** : `2821daa`

---

## 1. AGENTS OPENCODE (`.opencode/agents/`) — 12 agents

| # | Agent | Fichier | Mode | Description (extrait frontmatter) | Kill-switch env (si applicable) |
|---|---|---|---|---|---|
| 1 | `constitution` | `constitution.md` | primary | 10 invariants de la Constitution AgentOS | `ODYSSEUS_AGENT_CONSTITUTION` |
| 2 | `debate-5-personas` | `debate-5-personas.md` | primary | Débat 5 personas → verdict GO/CAUTION/STOP | `ODYSSEUS_AGENT_DEBATE` |
| 3 | `design-extract` | `design-extract.md` | primary | Extraction design system avec designlang | `ODYSSEUS_AGENT_DESIGN_EXTRACT` |
| 4 | `edge-case-gen` | `edge-case-gen.md` | primary | Génération specs test 12 dimensions | `ODYSSEUS_AGENT_EDGECASE` |
| 5 | `gsd-debugger` | `gsd-debugger.md` | primary | Debug systématique méthode scientifique | `ODYSSEUS_AGENT_DEBUGGER` |
| 6 | `gsd-executor` | `gsd-executor.md` | primary | Exécute plans phase par tâche, commits atomiques | `ODYSSEUS_AGENT_EXECUTOR` |
| 7 | `gsd-planner` | `gsd-planner.md` | primary | Crée plans de phase (goal-backward analysis) | `ODYSSEUS_AGENT_PLANNER` |
| 8 | `gsd-researcher` | `gsd-researcher.md` | primary | Recherche approche d'implémentation | `ODYSSEUS_AGENT_RESEARCHER` |
| 9 | `gsd-roadmapper` | `gsd-roadmapper.md` | primary | Maintient ROADMAP.md + checks cohérence | `ODYSSEUS_AGENT_ROADMAPPER` |
| 10 | `gsd-verifier` | `gsd-verifier.md` | primary | Vérifie atteinte objectif phase (goal-backward) | `ODYSSEUS_AGENT_VERIFIER` |
| 11 | `open-design` | `open-design.md` | primary | Génération artefacts design (142 design systems) | `ODYSSEUS_AGENT_OPEN_DESIGN` |
| 12 | `security-audit` | `security-audit.md` | primary | STRIDE + OWASP Top 10, auto-fix par sévérité | `ODYSSEUS_AGENT_SECURITY` |

**Preuve** : `.opencode/agents/` (12 fichiers), `src/orchestrator/agent_dispatcher.py:40-65` (mapping phase→agent)

**Statut global** : Tous les kill-switches sont **OFF par défaut** (`docker-compose.yml:107-120`). Le catalogue est chargé au démarrage seulement si `ODYSSEUS_AGENT_CATALOG=on` (`app.py:1254-1266`). Les agents existent en `.md` mais sont inactifs dans la boucle live (CLI only). Le dispatch est possible via `/api/agents/dispatch` (explicite) et via `AgentDispatcher.dispatch_for_phase()` (auto, si kill-switch ON).

---

## 2. SKILLS (`.opencode/skills/`)

**NON TROUVÉ** : Aucun répertoire `.opencode/skills/` n'existe. Le `data/skills/` contient un répertoire vide. Les skills sont gérés via `src/settings.py` et `routes/skills_routes.py` — ils sont extraits des sessions de chat par le pipeline de distillation mémoire (acontext → SKILL.md). 

**Preuve** : `data/skills/` (répertoire vide), `app.py:610-611` (mount `skills_routes`), `STATE.md:139,166-170` (pipeline distillation planifié Phase 8)

---

## 3. HOOKS & LIFECYCLE

| Hook | Emplacement | Quand | Description | Preuve |
|---|---|---|---|---|
| `_startup_event()` | `app.py:994-1268` | Démarrage app | Init managers, MCP, keepalive, agents, canaux | app.py:994 |
| `_shutdown_event()` | `app.py:1270-1293` | Arrêt app | Cancel upload cleanup, stop scheduler, MCP disconnect | app.py:1270 |
| `_lifespan` | `app.py:982-991` | FastAPI lifespan | Wrapper moderne remplaçant `@app.on_event` | app.py:982 |
| `_startup_mcp_connections` | `app.py:1030-1048` | Post-startup | Connecte MCP builtins + externes (fire-and-forget) | app.py:1030 |
| `_warmup_tool_index` | `app.py:1057-1067` | Post-startup | Pré-charge l'index d'outils RAG | app.py:1057 |
| `_warmup_endpoints` | `app.py:1069-1090` | Post-startup | Ping tous les endpoints LLM connus | app.py:1069 |
| `_keepalive_loop` | `app.py:1093-1102` | Toutes les 60s | Ping endpoints pour éviter cold starts | app.py:1093 |
| `_null_owner_sweep_loop` | `app.py:1187-1197` | Toutes les heures | Ré-assigne les données sans propriétaire | app.py:1187 |
| `_skill_audit_nightly_loop` | `app.py:1204-1227` | ~02:00 chaque nuit | Test + juge un batch de skills | app.py:1204 |
| `_startup_channel_adapters` | `app.py:1243-1252` | Post-startup | Réveille adapters Discord/Telegram (gated) | app.py:1243 |
| `_startup_agent_catalog` | `app.py:1256-1266` | Post-startup | Charge catalogue agents .opencode/ (gated) | app.py:1256 |

---

## 4. MCP SERVERS

### 4.1 Built-in (Python, in-process)

| Serveur | Fichier | Transport | Tools exposés | Preuve |
|---|---|---|---|---|
| `email_server` | `mcp_servers/email_server.py` | stdio (embedded) | email search/send/draft | mcp_servers/email_server.py |
| `graphify_mcp` | `mcp_servers/graphify_mcp.py` | stdio | knowledge graph queries | mcp_servers/graphify_mcp.py |
| `image_gen_server` | `mcp_servers/image_gen_server.py` | stdio | image generation | mcp_servers/image_gen_server.py |
| `memory_server` | `mcp_servers/memory_server.py` | stdio | memory CRUD | mcp_servers/memory_server.py |
| `obsidian_mcp` | `mcp_servers/obsidian_mcp.py` | stdio | `list_notes`/`get_note`/`search_notes` | mcp_servers/obsidian_mcp.py |
| `rag_server` | `mcp_servers/rag_server.py` | stdio | RAG document queries | mcp_servers/rag_server.py |

**Preuve** : `mcp_servers/` (6 fichiers), `src/builtin_mcp.py` (enregistrement), `app.py:1030-1035` (startup)

### 4.2 Docker externes

| Serveur | Compose service | Port | Statut | Kill-switch/Profile |
|---|---|---|---|---|
| `serena-mcp` | `serena-mcp` | 8765 | Configuré | Toujours démarré (pas de profile) |
| `scrapling-mcp` | `scrapling-mcp` | 8800 | Profil `scrapling` | `SCRAPLING_ENABLED` |
| `codebase-memory` | `codebase-memory` | 9749 | Profil `knowledge` | Aucun kill-switch |
| `graphify` | `graphify` | 9750 | Profil `knowledge` | `ODYSSEUS_GRAPHIFY` (off) |
| `decision-engine` | `decision-engine` | 8001 | Profil `decision-engine` | `DECISION_ENGINE_PATH` |
| `acontext` | `acontext` | 8029 | Profil `acontext` | `ACONTEXT_ENABLED` (off) |

**Preuve** : `docker-compose.yml:160-441`, `src/mcp_manager.py`

### 4.3 Obsidian MCP (Obsidian vault)

- **Gate** : `ODYSSEUS_OBSIDIAN_MCP` (défaut OFF, `docker-compose.yml:85,106`)
- **Script** : `mcp_servers/obsidian_mcp.py` — scaffold stdio ajouté (2026-07-05)
- **Volume** : `${OBSIDIAN_VAULT_PATH_HOST}` → `/obsidian-vault` (ro)
- **Statut** : 🔵 Codé, câblé (bridge checkpoint écrit la leg write), mais serveur MCP lui-même non démarré par défaut

---

## 5. GSD / PLANNING (`.planning/`)

| Fichier | Rôle | Preuve |
|---|---|---|
| `STATE.md` (140 lignes) | État du projet, historique, décisions, blocages | `.planning/STATE.md` |
| `ROADMAP-15-PHASES.md` (357 lignes) | Roadmap initiale 15 phases, 76 requirements | `.planning/ROADMAP-15-PHASES.md` |
| `ROADMAP-M3-ORCHESTRATION.md` (206 lignes) | Re-plan post-cartographie, verdicts redondance, vagues M3.0→M3.X | `.planning/ROADMAP-M3-ORCHESTRATION.md` |
| `config.json` | Configuration GSD | `.planning/config.json` |
| `intel/INDEX.md` | Routing table L3, redundancy map | `.planning/intel/INDEX.md` |
| `intel/domains/01→07.md` | Cartographie native par domaine (7 fichiers) | `.planning/intel/domains/` |
| `pipeline-lists/SKIP.md` | Pipeline list active SKIP | `.planning/pipeline-lists/SKIP.md` |
| `archive/` | Archive (PROJECT, REQUIREMENTS, INTEGRATION-TRACKING) | `.planning/archive/` |
| `AGENT-OS-ANALYSE-PROFONDE.md` | Analyse 60 outils | `.planning/AGENT-OS-ANALYSE-PROFONDE.md` |

---

## 6. DECISION ENGINE

| Composant | Emplacement | Statut | Preuve |
|---|---|---|---|
| Service Docker | `docker-compose.yml:416-440` | 🟡 Configuré (profil `decision-engine`) | docker-compose.yml:416 |
| Build context | `${DECISION_ENGINE_PATH:-../ConfigOpenCodeNew/decision-engine}` | Externe (repo ConfigOpenCodeNew) | docker-compose.yml:418 |
| Port | 8001 | Configuré | docker-compose.yml:421 |
| Dépendance | `odysseus` (service principal) | Configuré | docker-compose.yml:425 |

**Note** : Le Decision Engine vit dans un repo séparé (`ConfigOpenCodeNew/decision-engine`). Il n'est PAS intégré dans le code de ce repo. Le service Docker est défini mais derrière un profil.

---

## 7. CHANNEL GATEWAY

### 7.1 Composants

| Module | Emplacement | Rôle | Preuve |
|---|---|---|---|
| `ChannelGateway` | `src/channel_gateway.py` | Abstraction inbound/outbound bus | channel_bootstrap.py:24-30 |
| `ChannelType` | `src/channel_gateway.py` | Enum DISCORD, TELEGRAM | channel_bootstrap.py:27 |
| `DiscordAdapter` | `src/adapters/discord_adapter.py` | Adapter Discord (discord.py) | channel_bootstrap.py:185 |
| `TelegramAdapter` | `src/adapters/telegram_adapter.py` | Adapter Telegram (python-telegram-bot) | channel_bootstrap.py:192 |
| `channel_bootstrap` | `src/channel_bootstrap.py` | Point d'éveil unique (best-effort) | app.py:1243-1252 |
| `channel_routes` | `routes/channel_routes.py` | Routes API `/api/channels/*` | app.py:807-808 |

### 7.2 Kill-switches

| Switch | Défaut | Effet | Preuve |
|---|---|---|---|
| `ODYSSEUS_INPROCESS_DISCORD` | `off` | Active l'adapter Discord in-process | docker-compose.yml:97 |
| `ODYSSEUS_INPROCESS_TELEGRAM` | `off` | Active l'adapter Telegram in-process | docker-compose.yml:98 |
| `ODYSSEUS_CHANNEL_AGENT_REPLY` | `off` | Active le round-trip inbound→agent→reply | docker-compose.yml:99 |

---

## 8. SANDBOX / DOCKER

| Composant | Fichier | Configuration | Preuve |
|---|---|---|---|
| Docker principal | `Dockerfile` | Python 3.14-slim, multi-stage (wheels Real-ESRGAN) | Dockerfile:1-97 |
| Docker Compose | `docker-compose.yml` (444 lignes) | 7 services principaux + 4 profils | docker-compose.yml |
| GPU AMD | `docker-compose.gpu-amd.yml` | GPU ROCm config | docker-compose.gpu-amd.yml |
| GPU NVIDIA | `docker-compose.gpu-nvidia.yml` | GPU CUDA config | docker-compose.gpu-nvidia.yml |
| Security hardening | `docker-compose.yml` | `cap_drop: ALL`, `no-new-privileges: true` sur tous les services | docker-compose.yml:134-137 |
| Entrypoint | `docker/entrypoint.sh` | Drop privileges (gosu, PUID/PGID) | Dockerfile:91-92 |
| GPU build | `docker/build-realesrgan-wheels.sh` | Patch Python 3.14 wheels | docker/build-realesrgan-wheels.sh |
| `.dockerignore` | Racine | Exclusion fichiers de build | .dockerignore |

---

## 9. UI (Frontend)

### 9.1 Pages

| Page | Fichier | Route | Preuve |
|---|---|---|---|
| Index (SPA shell) | `static/index.html` | `/`, `/notes`, `/calendar`, `/cookbook`, `/email`, `/memory`, `/gallery`, `/tasks`, `/library` | app.py:881-927 |
| Login | `static/login.html` | `/login` | app.py:934-938 |
| Backgrounds sandbox | `static/backgrounds.html` | `/backgrounds` | app.py:930-932 |

### 9.2 Assets

| Dossier | Contenu | Preuve |
|---|---|---|
| `static/css/` | `style.min.css` (1.2MB, refactor planifié → <200KB) | STATE.md:115 |
| `static/js/` | Modules ES natifs (474+ fichiers) | static/js/ tree |
| `static/js/calendar/` | Module calendrier | static/js/calendar/ |
| `static/js/compare/` | Module comparaison modèles | static/js/compare/ |
| `static/js/editor/` | Éditeur d'images (build, filters, fx, tools) | static/js/editor/ |
| `static/js/emailLibrary/` | Client email | static/js/emailLibrary/ |
| `static/js/markdown/` | Rendu Markdown | static/js/markdown/ |
| `static/js/model/` | Gestion modèles | static/js/model/ |
| `static/js/research/` | Interface recherche | static/js/research/ |
| `static/js/util/` | Utilitaires | static/js/util/ |
| `static/fonts/` | Polices (OpenDyslexic, custom) | static/fonts/ |
| `static/icons/` | Icônes | static/icons/ |
| `static/lib/` | Bibliothèques tierces | static/lib/ |

### 9.3 PWA / Desktop

| Composant | Fichier | Preuve |
|---|---|---|
| Build Windows portable | `build-windows-portable.ps1` | build-windows-portable.ps1 |
| Build macOS app | `build-macos-app.sh` | build-macos-app.sh |
| PyInstaller spec | `Odysseus.spec` | Odysseus.spec |
| Launcher GUI | `launcher.py` | launcher.py |
| Launch script | `launch-windows.ps1` | launch-windows.ps1 |
| Service systemd | `odysseus-ui.service` | odysseus-ui.service |
| Companion mobile | `companion/` (pairing.py, routes.py) | companion/ |

---

## 10. CONFIG TRANSVERSE

| Fichier | Rôle | Preuve |
|---|---|---|
| `model-routing.json` (136 lignes) | Politique routing modèle (OmO) : 6 providers, 6 modèles, 8 stages, heuristic, fallback | model-routing.json |
| `config/phase-lock.yaml` (114 lignes) | 10 phases canoniques + restrictions outils par phase | config/phase-lock.yaml |
| `permission-matrix.md` (52 lignes) | Matrice outils × phases, gates, niveaux de risque | permission-matrix.md |
| `loop-canonique.md` (43 lignes) | Spécification 7 phases + kill-switches associés | loop-canonique.md |
| `PROJECT.yaml.example` | Template budget/objectif par projet (Phase 5) | PROJECT.yaml.example |
| `.env.example` | Template variables d'environnement | .env.example |
| `.opencode/` | Dossier config OpenCode (agents, .gitignore) | .opencode/ |

---

## 11. SCRIPTS

| Script | Rôle | Preuve |
|---|---|---|
| `scripts/odysseus` | CLI wrapper principal | scripts/odysseus |
| `scripts/odysseus-mail`, `-memory`, `-notes`, `-calendar`, `-docs`, `-gallery`, `-cookbook`, `-sessions`, `-research`, `-preset`, `-mcp`, `-personal`, `-backup`, `-contacts`, `-logs`, `-signature` | CLI tools (20+ scripts) | scripts/odysseus-* |
| `scripts/diffusion_server.py` | Serveur diffusion images | scripts/diffusion_server.py |
| `scripts/agentseal-ci.sh` | AgentSeal probe CI | scripts/agentseal-ci.sh |
| `scripts/demo_email/` | Démo email (seed, manage) | scripts/demo_email/ |
| `scripts/hf_download.py` | Téléchargement modèles HuggingFace | scripts/hf_download.py |
| `scripts/index_documents.py`, `index_byox.py` | Indexation documents | scripts/index_*.py |
| `scripts/migrate_faiss_to_chroma.py` | Migration FAISS → ChromaDB | scripts/migrate_faiss_to_chroma.py |
| `scripts/agent_migration_manifest.py` | Manifest migration agents | scripts/agent_migration_manifest.py |
| `scripts/check-docker-gpu.sh`, `check-docker-amd-gpu.sh` | Vérification GPU Docker | scripts/check-docker-*.sh |

---

## 12. ROUTES API (60+ routeurs)

Routes enregistrées dans `app.py` (lignes 582-876) :

| Routeur | Fichier | Domaine |
|---|---|---|
| `auth_routes` | `routes/auth_routes.py` | Authentification (login, signup, status) |
| `upload_routes` | `routes/upload_routes.py` | Upload fichiers |
| `emoji_routes` | `routes/emoji_routes.py` | Proxy SVG emoji Twemoji |
| `session_routes` | `routes/session_routes.py` | Sessions chat |
| `admin_wipe_routes` | `routes/admin_wipe_routes.py` | Danger Zone admin |
| `memory_routes` | `routes/memory_routes.py` | Mémoire agent |
| `skills_routes` | `routes/skills_routes.py` | Gestion skills |
| `chat_routes` | `routes/chat_routes.py` | Chat principal (SSE stream) |
| `research_routes` | `routes/research_routes.py` | Deep research |
| `history_routes` | `routes/history_routes.py` | Historique sessions |
| `search_routes` | `routes/search_routes.py` | Recherche web |
| `preset_routes` | `routes/preset_routes.py` | Presets de configuration |
| `diagnostics_routes` | `routes/diagnostics_routes.py` | Diagnostics système |
| `cleanup_routes` | `routes/cleanup_routes.py` | Nettoyage sessions |
| `personal_routes` | `routes/personal_routes.py` | Documents personnels RAG |
| `embedding_routes` | `routes/embedding_routes.py` | Gestion modèles embedding |
| `model_routes` | `routes/model_routes.py` | Découverte modèles |
| `copilot_routes` | `routes/copilot_routes.py` | GitHub Copilot device flow |
| `chatgpt_subscription_routes` | `routes/chatgpt_subscription_routes.py` | ChatGPT device flow |
| `tts_routes` | `routes/tts_routes.py` | Text-to-Speech |
| `stt_routes` | `routes/stt_routes.py` | Speech-to-Text |
| `document_routes` | `routes/document_routes.py` | Documents/artifacts/canvas |
| `signature_routes` | `routes/signature_routes.py` | Signatures image |
| `gallery_routes` | `routes/gallery_routes.py` | Galerie d'images |
| `editor_draft_routes` | `routes/editor_draft_routes.py` | Brouillons éditeur image |
| `task_routes` | `routes/task_routes.py` | Tâches planifiées + webhooks |
| `assistant_routes` | `routes/assistant_routes.py` | Assistant personnel |
| `calendar_routes` | `routes/calendar_routes.py` | CalDAV calendrier |
| `shell_routes` | `routes/shell_routes.py` | Exécution commandes shell |
| `cookbook_routes` | `routes/cookbook_routes.py` | Cookbook modèles (download/serve) |
| `workspace_routes` | `routes/workspace_routes.py` | Workspace fichiers |
| `hwfit_routes` | `routes/hwfit_routes.py` | Hardware model fitting |
| `compare_routes` | `routes/compare_routes.py` | Comparaison A/B modèles |
| `prefs_routes` | `routes/prefs_routes.py` | Préférences utilisateur |
| `backup_routes` | `routes/backup_routes.py` | Backup/restore données |
| `font_routes` | `routes/font_routes.py` | Polices custom |
| `mcp_routes` | `routes/mcp_routes.py` | Gestion MCP servers |
| `mcp_tools_routes` | `routes/mcp_tools_routes.py` | Outils MCP externes (Phase 4) |
| `ai_interaction` | `src/ai_interaction.py` | Outils interaction IA (debates, pipelines) |
| `webhook_routes` | `routes/webhook_routes.py` | Webhooks sortants |
| `api_token_routes` | `routes/api_token_routes.py` | Tokens API |
| `note_routes` | `routes/note_routes.py` | Notes style Google Keep |
| `email_routes` | `routes/email_routes.py` | Email (IMAP/SMTP) |
| `codex_routes` | `routes/codex_routes.py` | Plugin Codex HTTP bridge |
| `vault_routes` | `routes/vault_routes.py` | Vault chiffré |
| `contacts_routes` | `routes/contacts_routes.py` | CardDAV contacts |
| `companion` | `companion/routes.py` | Mobile companion |
| `phase_routes` | `routes/phase_routes.py` | Phase-lock API (Constitution Phase 3) |
| `channel_routes` | `routes/channel_routes.py` | Channel Gateway API (Phase 10) |
| `governance_routes` | `routes/governance_routes.py` | Governance API (Phase 9) |
| `autoeval_routes` | `routes/autoeval_routes.py` | Autoeval loop API (Phase 7) |
| `knowledge_routes` | `routes/knowledge_routes.py` | Trinité Connaissance (Phase 11) |
| `_agents` (inline) | `app.py:828-877` | Catalogue agents + dispatch |

---

## 13. ORCHESTRATEUR (`src/orchestrator/`) — 16 modules

| Module | Lignes (est.) | Rôle | Kill-switch |
|---|---|---|---|
| `__init__.py` | - | Package init | - |
| `phases.py` | 66 | Enum Phase, séquence canonique, phase-lock names, forced_tools | - |
| `loop.py` | 48 | `CanonicalLoop` — itère les 7 phases et push vers le registry | - |
| `dispatcher.py` | 84 | `resolve_model()` + `dispatch()` — orchestre un run 7 phases | - |
| `phase_tracker.py` | 64 | `PhaseTracker` — pont loop live ↔ phase-lock | `ODYSSEUS_PHASE_TRACKER` (off) |
| `gate.py` | 52 | `should_block_destructive()` — bloque commandes catastrophiques shell | `ODYSSEUS_DESTRUCTIVE_GATE` (on) |
| `registry.py` | 54 | `AgentRegistry` — découvre et indexe les agents .opencode/ | - |
| `spec.py` | 82 | `AgentSpec` + `parse_agent_spec()` — parsing YAML frontmatter + Markdown | - |
| `agent_dispatcher.py` | 228 | `AgentDispatcher` — dispatch agents par phase (auto) ou explicite | 10+ kill-switches |
| `router_advice.py` | ~50 | Advisory routing (classify_intent → rôle natif) | `ODYSSEUS_MODEL_ROUTER` (off) |
| `autoeval.py` | ~100 | `decide_keep_or_revert()` + `apply_autoeval()` — keep/revert post-build | `ODYSSEUS_AUTOEVAL` (off) |
| `autoevolve.py` | - | Auto-évolution mémoire (post-session) | `ODYSSEUS_AUTOEVOLVE` (off) |
| `checkpoint_tracker.py` | ~120 | Bridge vers Obsidian/Trinité (checkpoint post-round) | `ODYSSEUS_CHECKPOINT` (off) |
| `ancestry_tracker.py` | ~80 | Goal ancestry → DB en MEMORY_OBSERVE | `ODYSSEUS_GOVERNANCE_ANCESTRY` (off) |
| `codeburn_runner.py` | - | CodeBurn one-shot rate + waste patterns | `ODYSSEUS_CODEBURN` (off) |
| `spec.py` | 82 | Spécification agent (immutable dataclass) | - |

---

## 14. MODULES `src/` — 113 fichiers

Liste complète via CBM query_graph (file_path starts with `src/`) — 113 fichiers Python dans `src/`. Principaux :

| Catégorie | Modules clés |
|---|---|
| **Agent Loop** | `agent_loop.py` (~3500 lignes, cœur du système) |
| **LLM** | `llm_core.py`, `zen_router.py`, `endpoint_resolver.py` |
| **Memory** | `memory.py`, `memory_provider.py`, `memory_vector.py` |
| **RAG** | `rag_vector.py`, `rag_manager.py`, `rag_singleton.py`, `chroma_client.py` |
| **Tools** | `tool_execution.py`, `tool_schemas.py`, `tool_parsing.py`, `tool_policy.py`, `tool_security.py`, `tool_index.py`, `tool_implementations.py`, `tool_utils.py` |
| **Agent Tools** | `agent_tools/admin_tools.py`, `bg_job_tools.py`, `document_tools.py`, `filesystem_tools.py`, `model_interaction_tools.py`, `session_tools.py`, `subprocess_tools.py`, `web_tools.py` |
| **Security** | `risk_classifier.py`, `prompt_security.py`, `url_safety.py`, `url_security.py`, `command_validator.py` (NON TROUVÉ comme fichier séparé — intégré dans `builtin_actions.py`) |
| **Search** | `search/core.py`, `search/providers.py`, `search/query.py`, `search/ranking.py`, `search/cache.py`, `search/content.py`, `search/analytics.py` |
| **Channels** | `channel_gateway.py`, `channel_bootstrap.py`, `adapters/discord_adapter.py`, `adapters/telegram_adapter.py` |
| **Tasks** | `task_scheduler.py`, `task_endpoint.py`, `event_bus.py`, `webhook_manager.py` |
| **Config** | `config.py`, `constants.py`, `settings.py`, `runtime_paths.py` |
| **Observability** | `observer.py` (NON TROUVÉ comme module séparé — intégré dans orchestrator) |
| **MCP** | `mcp_manager.py`, `mcp_oauth.py`, `builtin_mcp.py` |
| **Éditeur** | `document_processor.py`, `document_actions.py` |
| **Divers** | `chat_handler.py`, `chat_processor.py`, `research_handler.py`, `deep_research.py`, `upload_handler.py`, `model_discovery.py`, `context_budget.py`, `context_compactor.py`, `settings_scrub.py`, `cleanup_service.py`, `cookbook_serve_lifecycle.py`, `bg_monitor.py`, `bg_jobs.py`, `copilot.py`, `caldav_sync.py`, `caldav_writeback.py`, `visual_report.py`, `office_doc.py`, `pdf_runtime.py`, `pdf_forms.py`, `secret_storage.py`, `service_health.py`, `rate_limiter.py`, `app_initializer.py`, `app_helpers.py`, `auth_helpers.py`, `assistant_log.py`, `generated_images.py`, `user_time.py`, `integrations.py`, `topic_analyzer.py`, `teacher_escalation.py`, `reminder_personas.py`, `goal_based_extractor.py`, `markitdown_runtime.py`, `request_models.py`, `model_context.py`, `api_key_manager.py`, `preset_manager.py`, `session_actions.py`, `session_search.py`, `email_thread_parser.py`, `text_helpers.py`, `tls_overrides.py`, `optional_deps.py`, `readiness.py`, `embedding_lanes.py`, `embeddings.py` |

---

## 15. KILL-SWITCHES (35+ variables d'environnement)

Regroupés par domaine (source : `docker-compose.yml:86-133`) :

### 15.1 Orchestration (M3)
| Switch | Défaut | Description |
|---|---|---|
| `ODYSSEUS_DESTRUCTIVE_GATE` | `on` | Bloque commandes shell catastrophiques |
| `ODYSSEUS_PHASE_TRACKER` | `off` | Active PhaseTracker dans le loop live |
| `ODYSSEUS_GOVERNANCE_ANCESTRY` | `off` | Écrit GoalTask avec ancestry |
| `ODYSSEUS_CHECKPOINT` | `off` | Persiste checkpoint Obsidian post-round |
| `ODYSSEUS_MODEL_ROUTER` | `off` | Active router_advice (advisory routing) |
| `ODYSSEUS_RRF_FUSION` | `off` | Active RRF hybride (vs blend 0.7/0.3) |
| `ODYSSEUS_ZEN_FROM_ENDPOINT` | `off` | Active Zen depuis ModelEndpoint natif |
| `ODYSSEUS_AUTOEVAL` | `off` | Active keep/revert via git reset --hard |
| `ODYSSEUS_UNIFIED_TOKENS` | `off` | Token accounting unifié |
| `ODYSSEUS_AUTOEVOLVE` | `off` | Auto-évolution mémoire post-session |
| `ODYSSEUS_LIVE_ORCHESTRATION` | `off` | Active CanonicalLoop complet (7 phases) |
| `ODYSSEUS_PROJECT_MANIFEST` | `off` | Active PROJECT.yaml manifest |

### 15.2 Channel Gateway
| `ODYSSEUS_INPROCESS_DISCORD` | `off` | Active adapter Discord in-process |
| `ODYSSEUS_INPROCESS_TELEGRAM` | `off` | Active adapter Telegram in-process |
| `ODYSSEUS_CHANNEL_AGENT_REPLY` | `off` | Active round-trip inbound→agent→reply |

### 15.3 Agents (.opencode)
| `ODYSSEUS_AGENT_CATALOG` | `off` | Charge le catalogue agents |
| `ODYSSEUS_AGENT_CONSTITUTION` | `off` | Agent constitution (CLASSIFY) |
| `ODYSSEUS_AGENT_PLANNER` | `off` | Agent planner (PLAN) |
| `ODYSSEUS_AGENT_RESEARCHER` | `off` | Agent researcher (PLAN) |
| `ODYSSEUS_AGENT_EXECUTOR` | `off` | Agent executor (BUILD) |
| `ODYSSEUS_AGENT_DEBUGGER` | `off` | Agent debugger (BUILD, on fail) |
| `ODYSSEUS_AGENT_DEBATE` | `off` | Agent debate (QUALITY) |
| `ODYSSEUS_AGENT_SECURITY` | `off` | Agent security audit (QUALITY) |
| `ODYSSEUS_AGENT_VERIFIER` | `off` | Agent verifier (AUTOEVAL) |
| `ODYSSEUS_AGENT_EDGECASE` | `off` | Agent edge-case gen (AUTOEVAL) |
| `ODYSSEUS_AGENT_ROADMAPPER` | `off` | Agent roadmapper (MEMORY_OBSERVE) |
| `ODYSSEUS_AGENT_DESIGN_EXTRACT` | `off` | Agent design-extract (explicit) |
| `ODYSSEUS_AGENT_OPEN_DESIGN` | `off` | Agent open-design (explicit) |

### 15.4 Connaissance
| `ODYSSEUS_GRAPHIFY` | `off` | Active service Graphify |
| `ODYSSEUS_OBSIDIAN_MCP` | `off` | Active MCP Obsidian |

### 15.5 Exécution & Observabilité
| `SCRAPLING_ENABLED` | `false` | Active Scrapling MCP |
| `KROKI_ENABLED` | `off` | Active Kroki |
| `ACONTEXT_ENABLED` | `false` | Active distillation mémoire Acontext |
| `ODYSSEUS_CODEBURN` | `off` | Active CodeBurn observability |
| `ODYSSEUS_AGENTSEAL` | `off` | Active AgentSeal security probe |
