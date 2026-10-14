# HARNESS-LANDSCAPE — État de l'art des harnais d'agent

> **Objet :** inventaire des harnais d'agent (CLI, IDE, frameworks, plateformes) et des
> fonctionnalités à reverse-engineer pour *AgentOS Omega*.
> **Méthode :** lecture directe des README officiels (GitHub) et des documentations
> officielles via HTTP (`webfetch`). La `websearch` étant hors quota (HTTP 402) sur cette
> session, chaque fait est adossé à une URL de repo/doc, listée en §6.
> **Date :** 2026-10-15 · **Branche :** `feat/agentos-omega`
> **Provenance :** `[observed]` = lu dans le README/la doc officielle (URL citée) ·
> `[inferred]` = déduction à partir de plusieurs sources · `[todo]` = à vérifier.
> Sauf mention contraire, tout le contenu ci-dessous est `[observed]`.
> **Avertissement :** ce document décrit l'état *tel que documenté* par les éditeurs. Les
> licences marquées « OSS » sans précision sont à revérifier avant réutilisation de code.

---

## 1. Tableau récapitulatif des harnais majeurs

| Outil | Éditeur / OSS | Langage cœur | Licence | Architecture (plugins ?) | Gestion du contexte | Points forts uniques |
|---|---|---|---|---|---|---|
| **Claude Code** | Anthropic (propriétaire) | TypeScript | Propriétaire | Plugins + Skills + Hooks + MCP + Agent SDK ; extension par composants déclaratifs | Context window 200k, `CLAUDE.md` + auto-memory, `.claude/rules/` path-scoped, skills en *progressive disclosure*, **tool search** (schémas MCP différés), `/compact` + **prompt caching** par préfixe | Sous-agents (Explore/Plan/general), agent teams, background agents, worktrees, Routines cloud/scheduled ; profondeur de la compaction et du caching documentée |
| **OpenAI Codex (CLI/cloud)** | OpenAI | **Rust** (`codex-rs`) | Apache-2.0 | Config TOML à couches, profils, **hooks**, skills, MCP, sous-agents déclarés (`~/.codex/agents/*.toml`) | `AGENTS.md` découvert en remontant au `project_root`, `memories` locales générées, compaction (`task.compact`), prefix caching via Responses API | Cœur Rust ; sandbox (workspace-write / danger-full-access) + politiques d'approbation granulaires ; **OTel** métriques/logs exhaustifs |
| **Cursor** | Anysphere (propriétaire) | TypeScript (fork VS Code) | Propriétaire | Règles `.cursor/rules`, MCP, pas d'API plugin publique (`[inferred]`) | Index du codebase + **Instant Grep** (moteur de recherche maison > ripgrep), **Explore subagent** isolé | Instant Grep local (pas d'upload des chemins), sous-agent d'exploration à contexte séparé |
| **Cline** | Cline Bot Inc. | TypeScript | Apache-2.0 | **SDK plugins** (`createTool`, hooks de cycle de vie), MCP, skills, `.clinerules` | Modes **Plan/Act**, checkpoints, règles projet, skills chargées à la demande | Multi-agents (coordinator + spécialistes), agents scheduled (cron), connecteurs Slack/Telegram/Discord/WhatsApp/Linear, CLI headless CI/CD |
| **Roo Code / Kilo Code** | Roo (arrêté 15/05) / Kilo-Org | TypeScript | Apache-2.0 (Roo) · MIT (Kilo) | Kilo : **Marketplace** (agents, skills, MCP, plugins) ; Kilo CLI = fork d'OpenCode | Modes (Code/Architect/Ask/Debug/Custom) ; Kilo : 500+ modèles avec **switch en cours de tâche** | Kilo : marketplace ouvert + changement de modèle mid-task sans markup ; Roo repris par la communauté (ZooCode) |
| **Aider** | Aider-AI (OSS) | Python | Apache-2.0 | Pas de système de plugins ; extensible par **formats d'édition** et config YAML | **Repo map** (tree-sitter + classement par graphe/PageRank dans un budget de tokens), prompt caching | Repo map respectueuse du budget ; git auto-commit ; lint/test après édition ; leaderboards d'édition |
| **OpenHands** | OpenHands (OSS) | Python + TypeScript | MIT | **Software Agent SDK** (agents/tools/conversations/workspaces), Agent Server REST, Canvas UI, **ACP** | Conversations + workspaces isolés par conteneur, événements, buffers de contexte par backend | Multi-backend local/VM/cloud/Docker, agents ACP tiers (Claude Code, Codex, Gemini…), **Automations** (schedule + webhooks) |
| **SWE-agent** | Princeton/Stanford (recherche) | Python | MIT | Config par **un seul YAML** ; pas de plugins | Agent-Computer Interface (ACI) ; config YAML unique | SoTA SWE-bench historique ; design « simple & hackable » ; remplacé par **mini-SWE-agent** (~100 lignes) |
| **Devin** | Cognition (propriétaire) | (fermé) | Propriétaire | Outils intégrés (Shell, IDE, Browser) + Slack/Teams + API ; pas de plugin public | Sessions longues, workspace persistant, reprise/prise en main (takeover) | Exécution autonome cloud parallèle, tâches Linear/Jira, PR reviews, CLI + `/handoff` vers le cloud |
| **Windsurf / Cascade** | Cognition (propriétaire) | (fermé, Rust/TS `[inferred]`) | Propriétaire | MCP, **Workflows**, memories & rules, ignore files | Cascade Code/Chat, **agent de planification dédié** en tâche de fond, @-mention des conversations passées (résumé) | Plan + Todo list continus, real-time awareness, checkpoints nommés/reverts, worktrees pour cascades simultanées |
| **Continue** | Continue Dev (archivé) | TypeScript | Apache-2.0 | Config-as-code hub (modèles, context providers, assistants) | Context providers + `@`-mentions | Pionnier OSS ; repo et releases **2.0.0 finales** (lecture seule) ; CLI/VS Code/JetBrains |
| **Goose** | Block → **AAIF / Linux Foundation** | **Rust** | Apache-2.0 | **Extensions = serveurs MCP** (70+), distributions personnalisées, ACP | **Context revision** : résumé par petit modèle, suppression algorithmique du contenu obsolète, find-and-replace, ripgrep, résumé des sorties verbeuses | Moteur Rust desktop/CLI/API ; serveur ACP *et* agents ACP comme providers ; « custom distros » |
| **gptme** | Erik Bjäreholt (OSS) | Python | MIT | **Plugins + Skills + Lessons + Hooks** ; MCP (double sens) ; ACP | Lessons contextuelles (mots-clés/outils/patterns), compression de contexte, token awareness | **Cross-harness memory** (mémoire partagée avec Claude Code & Codex), agents autonomes (systemd/cron), sandbox Docker/Wasmtime, références durables (Bob) |
| **Plandex** | Plandex (OSS) | Go | OSS (voir LICENSE) | Packs de modèles, sandbox de diff cumulatif, arborescence de version du plan | **2M tokens effectifs**, chargement à la demande par étape, **project map tree-sitter** (20M+ tokens indexables), context caching | Diff review sandbox *hors* de l'arbre de travail, version control du plan (branches par modèle), debug auto de commandes/navigateur |
| **OpenCode** | anomalyco (OSS) | TypeScript (Bun) | MIT | **Plugins TS** (`@agentos/*`-style), agents, commandes, MCP, providers via registry | Agents `build`/`plan`, subagent `general`, LSP, contexte par agent | TUI + desktop ; notions `agent`/`command`/`skill`/`plugin` déclaratives (base du fork Kilo) ; multi-providers universel |
| **Zed** | Zed Industries | **Rust** | GPL-3.0 (Apache-2.0 par parties) | Agent Panel, **profils d'agent**, ACP (external agents), MCP, skills | Compaction auto, usage de tokens visible, **New From Summary**, @-mentions (fichiers/symboles/threads/URL) | Éditeur Rust GPU ; isolation par **worktree** par thread ; checkpoints ; ACP de premier ordre |
| **Hermes Agent** | Nous Research | Python (+ Rust assis.) | MIT | **Plugins + Skills (agentskills.io) + Tool Search** ; MCP ; profiles | Mémoire bornée (MEMORY.md/USER.md en *snapshot gelé*), **FTS5 session search**, **tool search BM25**, compaction | Boucle d'auto-amélioration (création/auto-patch de skills), 7 backends d'exécution (local/Docker/SSH/Singularity/Modal/Daytona/Vercel), cron + gateway messagerie |
| **OpenClaw** | OpenClaw Foundation (501c3) | TypeScript (Node) | MIT | **Gateway + plugins** (manifest + SDK), **slot context engine**, slot memory, skills, canaux | Contexte assemblé par un **Context Engine enfichable** (ingest/assemble/compact/after-turn), `ownsCompaction` par plugin | Modèle « gateway de confiance / exécution non fiable, policy par code » ; adaptateurs de harnais (Codex, Claude) ; ClawHub |
| **MetaGPT** | DeepWisdom (OSS) | Python | MIT | Rôles/composants, SOP ; pas de registre de plugins | Rôles + artefacts partagés (docs, code) ; pas de compaction LLM native | `Code = SOP(Team)` : « entreprise logicielle » multi-agents (PM/architecte/PM/ingénieurs), Data Interpreter, MGX |
| **AutoGen / AG2** | Microsoft (maintenance) | Python / .NET | MIT | Couches Core / AgentChat / Extensions ; MCP | Message passing événementiel, conversation state | Multi-agents event-driven, `AgentTool` (agents as tools), runtime distribué ; **en mode maintenance** → Microsoft Agent Framework |
| **CrewAI** | CrewAI Inc. | Python | MIT | Crews + **Flows** ; memory/knowledge ; MCP/A2A ; skills | Crews (autonomie) / Flows (événementiel, état typé) ; memory, knowledge, checkpointing | Séparation nette autonomie (Crew) vs contrôle (Flow) ; agents/tasks en JSONC ; AMP control plane (tracing) |
| **LangGraph** | LangChain Inc | Python / TS | MIT | Graphe stateful bas niveau ; Deep Agents (planning+subagents+FS) | **Durable execution**, memory court/long terme, checkpoints, interrupts | Orchestration durable, human-in-the-loop, reprise exacte après crash, LangSmith (observabilité) |
| **smolagents** | Hugging Face | Python | Apache-2.0 | Tools MCP/LangChain/Hub ; pas de registre plugin | `agent.memory` (messages), ReAct | **CodeAgent** : actions écrites en code Python (≈30 % d'étapes en moins), sandbox E2B/Modal/Blaxel/Docker, ~1000 lignes |
| **OpenAI Swarm** | OpenAI | Python | MIT | Agents + **handoffs** ; stateless | Historique `messages` côté client (aucun stockage) | Primitive de coordination minimale (Agent/handoff) ; **éducationnel**, remplacé par OpenAI Agents SDK |
| **Dify** | LangGenius | Python + TypeScript | Dify OSS License (Apache-2.0 + conditions) | **Marketplace** de plugins/outils/MCP, workflow visuel | Workflow/RAG pipeline, mémoire de conversation, LLMOps | Canvas visuel workflow + RAG + agent + observabilité (Opik/Langfuse/Phoenix) ; backend-as-a-service |
| **n8n** | n8n GmbH | TypeScript | Sustainable Use License (fair-code) | **1500+ intégrations**, nœuds custom, LangChain nodes | Glue par nœuds ; mémoire d'agent (fenêtre) | Plateforme d'automatisation mature (audit, RBAC, self-host/cloud), agents IA + workflows classiques |
| **Flowise** | Flowise AI | TypeScript | Apache-2.0 | UI React/server/components ; nodes tiers | Chaînes/agents visuels, memory nodes | Builder visuel d'agents ; **repo archivé** |
| **Open Interpreter** | Open Interpreter | **Rust** (fork de Codex) | Apache-2.0 | Skills/`.agents/skills`, MCP, ACP, hooks, permissions | **Harness emulation** (`/harness` : `claude-code`, `native`, `swe-agent`, `minimal`…) | Réimplémente le *harnais* qui maximise un modèle donné (interface Codex-like) ; compat Codex SDK & ACP ; computer use |

---

## 2. Fiches détaillées (les 10 plus pertinents pour Omega)

### 2.1 Claude Code (Anthropic)
- **Modèle d'extension :** composants déclaratifs — `CLAUDE.md`/`AGENTS.md`, `.claude/rules/` (path-scoped par glob), **skills**, **hooks** (PreToolUse/PostToolUse/SessionStart…), **plugins** (skills+commands+agents+hooks+MCP+themes), **MCP**, et l'**Agent SDK** pour des agents sur mesure. Sécurité des plugins : les sous-agents de plugin ne peuvent pas porter `hooks`/`mcpServers`/`permissionMode`.
- **Contexte :** au démarrage ~system prompt + auto-memory (200 lignes / 25 Ko de `MEMORY.md`) + env + **descriptions** de skills + `CLAUDE.md` (user puis projet) ; le contenu complet ne charge qu'à l'usage (*progressive disclosure*). Les **schémas MCP sont différés** par défaut et chargés à la demande via **tool search**. `CLAUDE.md` ciblé < 200 lignes.
- **Compaction :** `/compact` remplace l'historique par un **résumé structuré** ; build cache, CLAUDE.md projet, mémoire et MCP sont rechargés ; quelques fichiers récemment modifiés sont relus ; les skills invoquées sont réinjectées (plafond 5 000 tokens/skill) ; l'index des skills n'est *pas* réinjecté. `/rewind` (checkpoints) tronque vers un préfixe déjà caché.
- **Sous-agents :** définition Markdown + frontmatter (`tools`, `model`, `permissionMode`, `maxTurns`, `skills`, `mcpServers`, `memory`, `effort`, `isolation: worktree`). Build-ins : **Explore** (read-only), **Plan**, general-purpose ; profondeur d'imbrication limitée ; modes premier plan/arrière-plan. Agent teams (teammates), background agents, cross-session messaging.
- **Routing multi-modèles :** alias `sonnet`/`opus`/`haiku`/`fable` par sous-agent et par invocation ; `CLAUDE_CODE_SUBAGENT_MODEL`(+`_FORCE`) ; fallback automatique ; providers tiers (Bedrock, Vertex, Foundry, LLM gateway).
- **Exécution :** terminal, VS Code/JetBrains, desktop, web/cloud, CI (GitHub Actions/GitLab), **Routines** (cloud, déclencheurs API/GitHub) et tâches planifiées desktop ; sandbox + permissions.
- **Observabilité :** exporteur **OpenTelemetry** (tokens cache read/create par user/session), `/usage` (ratio de cache, cause probable du dernier miss), statusline (`current_usage`, `prompt_cache`).

### 2.2 OpenAI Codex (CLI / cloud)
- **Modèle d'extension :** `~/.codex/config.toml` + **profils** (`--profile`, `~/.codex/<name>.config.toml`), config projet `.codex/config.toml` (walk-up, uniquement si projet de confiance), **hooks** (`hooks.json`/`[hooks]`, événement PreToolUse…), **skills**, MCP, et **sous-agents** via `[agents]` + fichiers `~/.codex/agents/*.toml`.
- **Contexte :** `AGENTS.md` découvert en remontant depuis le cwd jusqu'à un **project root** (marqueurs `.git`/`.hg`/`.sl` configurables) ; `project_doc_max_bytes`/fallback filenames. **Memories** locales (`~/.codex/memories/`, générées en tâche de fond, avec seuil de rate-limit) activables par flag.
- **Compaction :** métrique `task.compact` (remote/local) ; mécanismes de compactage local/remote.
- **Sous-agents :** built-ins `default`/`worker`/`explorer` ; agents custom (name, description, developer_instructions, model, reasoning effort, feature flags). Réglages globaux : `max_concurrent_threads_per_session`, `default_subagent_model`, `default_subagent_reasoning_effort`. Chaque sous-agent hérite du sandbox/policy du parent.
- **Routing multi-modèles :** providers custom (base_url/env_key/wire_api/headers/auth par commande), provider Bedrock built-in, OSS mode (Ollama/LM Studio), `openai_base_url`.
- **Exécution :** sandbox (workspace-write par défaut, `.git`/`.agents`/`.codex`/`.aws` en lecture seule, `network_access` opt-in, `danger-full-access`) + politiques d'approbation (dont `granular`, `auto_review`) ; `approvals_reviewer`, `allow_login_shell`.
- **Observabilité :** **OTel** logs + métriques riches (`api_request`, `turn.token_usage` par type, `tool.call`, `mcp.call`, `multi_agent.spawn`, `memory.phase1/2`, `hooks.run`, `task.compact`…). `notify` (webhook externe sur `agent-turn-complete`).

### 2.3 Cursor
- **Modèle d'extension :** règles `.cursor/rules`, MCP ; pas d'API plugin documentée publiquement `[inferred]`.
- **Contexte :** **Instant Grep** — moteur de recherche maison (regex + word-boundary) présenté comme plus rapide que `ripgrep` sur gros codebases, **index construit et requêté localement** (pas d'upload des chemins ni d'embeddings du code). Agent déclenche grep sur des symboles.
- **Sous-agents :** **Explore subagent** — fenêtre de contexte séparée, modèle plus rapide, exécute de nombreuses recherches parallèles et ne renvoie que les **résultats pertinents** (garde la conversation principale légère).
- **Compaction :** non détaillée dans la doc consultée `[todo]`.
- **Multi-modèles :** large choix de modèles ; **Cloud Agents** (ne supportent pas les workspaces multi-root).
- **Exécution :** éditeur (fork VS Code) + cloud ; worktrees.
- **Observabilité :** non documentée publiquement dans la page consultée `[todo]`.

### 2.4 Cline
- **Modèle d'extension :** **SDK** (`@cline/sdk`) — `createTool`, **hooks de cycle de vie** (logging, audit, policy, capacités métier) ; **MCP** ; **skills** ; `.clinerules` (partagées CLI/VS Code/JetBrains).
- **Contexte :** modes **Plan** (explore, questions, stratégie) / **Act** (exécute) ; checkpoints pour tout annuler ; règles projet + skills chargées à la demande.
- **Sous-agents / équipes :** **Multi-agent teams** — agent coordinateur qui décompose et délègue à des spécialistes (chacun avec ses tools et contexte) ; l'état d'équipe **persiste entre sessions** (`cline --team-name`).
- **Multi-modèles :** Anthropic/OpenAI/Google/OpenRouter/Bedrock/Azure/Vertex/Cerebras/Groq/Ollama/LM Studio/any OpenAI-compatible.
- **Exécution :** VS Code, JetBrains, CLI (interactif/headless), desktop (Tauri+Bun+Next), **agents scheduled** (cron persistant, indépendant d'un terminal), exécution de commandes bash avec suivi temps réel.
- **Observabilité / intégrations :** connecteurs messagerie (Telegram/Slack/Discord/Google Chat/WhatsApp/Linear) mappés à des sessions ; sortie JSON pour CI/CD.

### 2.5 Aider
- **Modèle d'extension :** pas de registre de plugins ; extensibilité par **formats d'édition** (edit formats) et config YAML/`.env`.
- **Contexte :** **repo map** — carte concise du dépôt (classes/fonctions/signatures) construite via **tree-sitter**, puis **classée par un algorithme de graphe** (type PageRank ; fichiers = nœuds, dépendances = arêtes) et **tronquée au budget de tokens** (`--map-tokens`, défaut 1k, adaptatif). Le modèle peut demander des fichiers précis, ajoutés au chat.
- **Compaction :** non documentée comme `/compact` ; le repo map joue le rôle de compression structurelle ; supporte le **prompt caching** (doc dédiée).
- **Sous-agents :** absents (mono-agent).
- **Multi-modèles :** large catalogue (Anthropic/OpenAI/DeepSeek/Gemini/Groq/xAI/Ollama/OpenRouter/Bedrock/Vertex…) ; leaderboards d'édition/refactor et par date.
- **Exécution :** terminal, IDE (mode `--watch` via commentaires), lint/tests automatiques après édition, **git auto-commit** avec messages.
- **Observabilité :** analytics, historique git (diff/undo).

### 2.6 OpenHands
- **Modèle d'extension :** **Software Agent SDK** (agents, tools, conversations, workspaces, events) exposé par un **Agent Server REST** ; l'UI Agent Canvas consomme une API via un client TS. Agents tiers via **ACP** (Claude Code, Codex, Gemini CLI, Pi, OpenCode).
- **Contexte :** conversations + workspaces isolés ; **chaque conversation peut tourner dans son propre conteneur Docker** avec workspace et état persistés ; plusieurs Agent Server interchangeables.
- **Sous-agents :** via SDK/agents ; pas de « team » intégrée comme Cline `[inferred]`.
- **Multi-modèles :** « any LLM » (LLM profiles).
- **Exécution :** local, Docker, VM, cloud, entreprise ; **Automations** (schedule + webhooks) déclenchées par Slack/GitHub/Linear/Notion/Datadog.
- **Observabilité :** historique de conversations, run history des automations ; multi-repo (Canvas/SDK/typescript-client/automation) — frontières explicites.

### 2.7 Goose (Block → Agentic AI Foundation)
- **Modèle d'extension :** **Extensions = serveurs MCP** (70+) ; **custom distributions** (rebranding + providers/extensions préconfigurés).
- **Contexte :** **Context Revision** — résumé par LLM plus petit/rapide, suppression algorithmique du contenu ancien/irrelevant, préférence find-and-replace (pas de réécriture de gros fichiers), ripgrep (ignore les fichiers système), résumé des sorties de commandes verbeuses ; inclus « tout » plutôt que recherche sémantique selon la doc.
- **Sous-agents :** l'interface peut créer **plusieurs agents** pour tâches concurrentes ; délégation à des agents ACP externes (Claude Code/Codex) comme *providers*.
- **Multi-modèles :** 15+ providers ; abonnements via **ACP** ; local Ollama.
- **Exécution :** desktop macOS/Linux/Windows, CLI, API ; `goose acp` (serveur ACP) ; built in **Rust**.
- **Observabilité :** extension MCP dédiée ; gouvernance Linux Foundation.

### 2.8 Hermes Agent (Nous Research)
- **Modèle d'extension :** **Plugins** + **Skills** (standard ouvert `agentskills.io`) + **Tool Search** ; MCP ; profiles (isolation par profil).
- **Contexte :** mémoire **bornée** — `MEMORY.md` (~800 tokens) + `USER.md` (~500 tokens), injectées en **snapshot gelé** au démarrage (préserve le prefix cache) ; débordement = erreur → l'agent consolide dans le même tour. **`session_search`** = FTS5 SQLite sur tous les messages (recherche/scroll, sans LLM). **Progressive disclosure des skills** en 3 niveaux (liste → contenu → fichier de référence). **Tool Search** : remplace les schémas MCP/plugin par 3 outils passerelles (`tool_search`/`tool_describe`/`tool_call`), **BM25 + stemming + rarest-token + relevance floor**, budget `min(threshold_pct% du contexte, listing_max_tokens)`.
- **Compaction :** `/compress` ; revue d'arrière-plan (`background_review`) qui alimente mémoire/skills, avec digest si modèle différent (préserve le cache si modèle identique), budgets `max_input_tokens`, `defer` sur GPU local.
- **Sous-agents :** `delegate_task` → sous-agents isolés parallèles ; scripts Python qui appellent les outils via RPC (« tours à coût de contexte nul »).
- **Multi-modèles :** Nous Portal, OpenRouter, OpenAI, endpoint custom, etc. ; `hermes model` (pas de lock-in).
- **Exécution :** **7 backends** — local, Docker, SSH, Singularity, Modal, Daytona, Vercel Sandbox (persistance serverless) ; **cron scheduler** avec livraison multi-plateforme ; gateway Telegram/Discord/Slack/WhatsApp/Signal/email.
- **Observabilité :** `/usage`, `/insights`, `/journey` (timeline des apprentissages, prune/edit des nœuds), logs de fork (`session_model_usage`, `agent.log`).

### 2.9 OpenClaw
- **Modèle d'extension :** **Gateway** (control plane : sessions, tools, events, canaux) + **plugins** (manifest `openclaw.plugin.json` + SDK, ClawHub) + **skills** ; **slots** exclusifs (`contextEngine`, `memory`). Policy d'installation (`security.installPolicy`, provenance/trust, quarantaine).
- **Contexte :** **Context Engine enfichable** avec 4 points de cycle de vie — `ingest`, `assemble` (retourne messages + `systemPromptAddition` + `estimatedTokens`), `compact`, `after-turn` (+ `maintain`, `bootstrap`, `ingestBatch`) ; hooks de cycle de vie **sous-agents** (`prepareSubagentSpawn`, `onSubagentEnded`) ; `ownsCompaction` par plugin ; isolation d'échec → repli sur moteur `legacy`.
- **Sous-agents :** spawn natif avec `contextMode` `isolated`/`fork`, TTL, rollback.
- **Multi-modèles / harnais :** providers hôtes et locaux ; **adaptateurs de harnais** (Codex app-server, OpenClaw embedded) — « un Gateway » pour assistant perso ou équipe.
- **Exécution :** Gateway local ; canaux Discord/iMessage/Slack/Teams/Telegram/WhatsApp + 20 ; companion apps (macOS/iOS/Android/Windows/Linux) ; **sandbox** configurable.
- **Observabilité :** `openclaw doctor`, `gateway status --deep`, `plugins inspect --runtime`, métriques de démarrage des tools plugins.

### 2.10 OpenCode (+ descendances Kilo)
- **Modèle d'extension :** **agents** (`build`/`plan` + `general` subagent), **commands**, **skills**, **plugins TS**, **MCP**, providers via registre (`models.dev`). Config déclarative (JSON).
- **Contexte :** contexte par agent, LSP intégré ; l'agent `plan` est read-only par défaut (denies edit, demande permission bash). Sub-agent `general` invocable par `@general` ; sous-agents natifs configurables.
- **Compaction :** mécanismes de session ; `[todo]` sur les détails.
- **Multi-modèles :** universel (registre de providers).
- **Exécution :** TUI, desktop (macOS/Windows/Linux), ACP possible `[inferred]`.
- **Observabilité :** serveur/client SDK ; requêtes typées.
- **Note :** Kilo Code est un **fork d'OpenCode** ; cette architecture (agents/commands/skills/plugins) est le meilleur modèle « tout est plugin » côté CLI, directement pertinent pour Omega.

---

## 3. Patterns d'architecture plugin observés

Synthèse des mécanismes qui rendent réellement les briques interchangeables, avec la bonne pratique réutilisable.

1. **Manifeste déclaratif + registre, pas d'import en dur.** OpenClaw (`openclaw.plugin.json`, `plugins.entries.<id>`, `plugins.allow/deny`), Claude Code (`README` de plugin + composants), Codex (config TOML à couches), Hermes (profils + plugins). → *Bonne pratique :* un **registre** découvre les plugins ; l'activation est un flag de config, jamais une modification du noyau.

2. **Port + slot exclusif pour les capacités singulières.** OpenClaw définit des **slots** (`contextEngine`, `memory`) où *un seul* plugin est résolu par run, avec repli `legacy`. → *Bonne pratique :* distinguer « N implémentations concurrentes » (tools, providers) de « 1 seule active » (moteur de contexte, mémoire). C'est exactement le pattern *capability-first* d'Omega.

3. **SDK d'extension runtime.** Cline (`createTool` + hooks de cycle de vie), OpenClaw (`api.on`/`api.registerHook`), Codex/Claude (hooks), Hermes (`skill_manage`, plugin provider). → *Bonne pratique :* exposer un **SDK typé** avec des crochets de cycle de vie, pas seulement de la config.

4. **Interopérabilité par protocoles ouverts.** **MCP** (tools/data) et **ACP** (agent ↔ éditeur) sont adoptés par la quasi-totalité : Goose, Zed, OpenHands, Open Interpreter, gptme, AutoGen, CrewAI, OpenClaw, Kilo… → *Bonne pratique :* ne pas inventer de protocole « tool » ou « agent-client » ; **implémenter MCP (client+serveur) et ACP**, et garder les outils « neutres » (`.agents/skills`, `AGENTS.md`).

5. **Config à couches avec précédence explicite.** Codex (user → puis `~/.codex/<profile>` → puis projet `.codex/` → CLI), Claude Code (managed → CLI → projet → user → plugin), Hermes (project → local → create_dir → external). → *Bonne pratique :* documenter **l'ordre de résolution**, et refuser qu'une couche non fiable court-circuite les credentials (Codex ignore `openai_base_url` en config projet).

6. **Frontières de dépôts / de processus explicites.** OpenHands sépare Canvas / SDK / client TS / automation (4 repos, responsabilités écrites noir sur blanc). → *Bonne pratique :* un service = une frontière de crash ; schéma typé et versionné à chaque frontière.

7. **Isolation d'échec.** OpenClaw **quarantine** un moteur de contexte défaillant et repart sur `legacy` pour le process courant. → *Bonne pratique :* un plugin qui échoue ne doit jamais faire taire l'agent ; repli déterministe + log opérateur.

8. **Confiance / provenance / policy par code.** OpenClaw (install policy, provenance du plugin, allowlist de canaux, sandbox), Codex (projet de confiance), Claude Code (trust de dossier, managed settings). → *Bonne pratique :* traiter **l'entrée (canal, message, skill de repo) comme non fiable**, et appliquer la policy dans le code, pas seulement dans le prompt.

9. **Distributions personnalisées.** Goose (`CUSTOM_DISTROS`), Open Interpreter (branding de fork), Kilo (fork d'OpenCode). → *Bonne pratique :* permettre un **re-branding** et un *preset* providers/extensions sans fork du cœur.

10. **Séparation « cœur déterministe » / « LLM ».** Aider (repo map = algo de graphe, pas de LLM), Cursor (Instant Grep local), Hermes `session_search` (FTS5, sans LLM), OpenClaw (`assemble`/`compact` déterministes côté hôte). → *Bonne pratique :* le coût de calcul va au système (script/Rust), jamais au modèle. C'est la règle §3.2 d'Omega, déjà appliquée par les meilleurs.

---

## 4. Fonctionnalités à reverse-engineer en priorité

Liste **priorisée**. Format : (a) ce que ça fait · (b) pourquoi ça compte pour Omega · (c) où c'est implémenté · (d) difficulté Rust (**S**/M/L).

### P1 — Moteur de contexte enfichable (Context Engine)
- **(a)** Interface unique avec cycle de vie `ingest → assemble → compact → after-turn` (+ `maintain`, `bootstrap`, hooks sous-agents) ; le plugin contrôle ce que voit le modèle et peut posséder la compaction (`ownsCompaction`).
- **(b)** C'est le port le plus structurant d'Omega : « chaque brique est un plugin » appliqué au cœur (mémoire, retrieval, compaction, recall sont tous *derrière* ce port). Permet de tout remplacer sans toucher l'orchestrateur.
- **(c)** OpenClaw `docs/concepts/context-engine` ; inspiration `goose context revision`, `Hermes tool search/memory`.
- **(d)** **L** — nécessite un trait Rust (`trait ContextEngine`), un scheduler de maintenance, l'annulation coopérative, l'isolation d'échec et un chargeur de plugins (WASM/dylib/process). À faire tôt car il contraint tout le reste.

### P2 — Progressive disclosure des skills (agentskills.io)
- **(a)** Chargement en 3 niveaux : index `{name, description}` (~3k tokens) → `SKILL.md` complet à l'usage → fichiers `references/` à la demande.
- **(b)** Économie de contexte massive et standardisation (`~/.agents/skills`, `SKILL.md` partagé Claude Code/Codex/Hermes/gptme/OpenClaw).
- **(c)** Hermes `features/skills` + `features/tool-search` ; Claude Code `docs/skills` ; gptme ; standard `agentskills.io/specification`.
- **(d)** **S** — parsing frontmatter + index + loading paresseux + invocations `/skill`.

### P3 — Tool Search (schémas d'outils différés)
- **(a)** Remplace les schémas MCP/plugin par des ponts `tool_search`/`tool_describe`/`tool_call` ; recherche **BM25 + stemming + rarest-token + relevance floor** ; listing « skills-style » pour garder les capacités visibles ; tiers selon le budget.
- **(b)** Les schémas d'outils saturent le contexte (cf. Cloudflare ~3 300 tools ≈ 32k tokens rien qu'en noms). Indispensable pour une « armée » d'outils.
- **(c)** Hermes `features/tool-search` (impl. `tools/tool_search.py`) ; Claude Code « deferred MCP tools / tool search » ; Anthropic 49 %→74 % sur Opus 4.
- **(d)** **M** — index inversé + BM25/stemming en Rust + budget tokens + réécriture du tableau d'outils. Coût de schéma déterministe = système, pas LLM.

### P4 — Compaction + stabilité du prefix (prompt caching)
- **(a)** Résumé structuré de l'historique (`/compact`) et ordre canonique des couches (system → project context → conversation) pour maximiser les *cache hits* ; `/rewind` tronque vers un préfixe déjà caché ; TTL 5m/1h.
- **(b)** Le levier n°1 d'économie ($ et latence) ; prérequis à toute exécution longue.
- **(c)** Claude Code `docs/prompt-caching` + `context-window` ; Goose context revision ; Zed compaction auto ; Codex `task.compact`.
- **(d)** **M** — nécessite un ordonnancement stable des messages, un summarizer (modèle « petit » route), et l'invalidation consciente (model switch, changement d'outil).

### P5 — Sous-agents isolés + parallèles (armée d'agents)
- **(a)** Chaque sous-agent a son contexte/système/tools/perm sandboxé ; ne renvoie qu'un **résumé** au parent ; profondeur limitée ; exécution foreground/background.
- **(b)** Isole le travail bruyant, garde le fil principal propre, permet le fan-out parallèle — « armée de sous-agents ».
- **(c)** Claude Code `docs/sub-agents` (+ agent-teams, background agents) ; Codex `agent-configuration/subagents` ; Cline teams ; Hermes `delegate_task` ; Cursor Explore ; OpenClaw lifecycle.
- **(d)** **M** — scheduler + isolation de contexte + agrégation de résultats + limites ; l'isolation *worktree* est **M** en plus.

### P6 — Registre multi-modèles + routing par capacité
- **(a)** Registre de providers/modèles, notation communautaire, sélection par type de tâche (vitesse ↔ raisonnement), switch mid-task sans markup.
- **(b)** Objectif central d'Omega (6 piliers, pilier 1) ; anti-lock-in fournisseur.
- **(c)** Kilo (500+ modèles, switch mid-task) ; OpenCode `models.dev` ; Aider leaderboards ; gptme `docs/models` (fast/cheap triage vs coding) ; Codex providers custom ; Hermes `hermes model`.
- **(d)** **M** — trait `ModelProvider` + registry schema-validé + scoring + fallback. Le routing *déterministe* (règles) est **S**.

### P7 — Plan/Act + agent de planification séparé
- **(a)** Mode plan read-only qui explore, questionne et produit un plan ; un agent de planification dédié raffine la stratégie pendant que le modèle exécute ; Todo list.
- **(b)** « Pilotage organique » : alignement avant action, contrôle vs autonomie.
- **(c)** Cline Plan/Act ; Windsurf **planning agent** + Todo ; Zed plan mode ; OpenCode agent `plan` ; Claude Code plan mode.
- **(d)** **S** — surtout du prompt/port + permissions read-only ; le planificateur en tâche de fond est **M**.

### P8 — Checkpoints / worktrees / rollback
- **(a)** Snapshot de l'arbre + revert (y compris checkpoint nommé) ; isolation d'un thread dans un **git worktree** ; aires de fichiers protégées.
- **(b)** Autonomie sûre : l'agent peut tenter, échouer et revenir sans casser l'arbre de travail (cf. Plandex *sandbox de diff*).
- **(c)** Zed, Windsurf (named checkpoints), Claude Code (`isolation: worktree`), Plandex (diff review sandbox), Cline checkpoints.
- **(d)** **M** — travail sur git worktree en Rust (libgit2/gix) + intégration au cycle de vie du sous-agent.

### P9 — Exécution sandboxée multi-backend
- **(a)** Le même agent tourne sur local / Docker / SSH / serverless ; les commandes et le code s'exécutent dans un bac à sable.
- **(b)** Frontière de crash, sécurité, « run anywhere » — et self-host.
- **(c)** Hermes **7 backends** ; OpenHands (conteneur par conversation) ; smolagents (E2B/Modal/Blaxel/Docker) ; Codex/Claude sandbox ; gptme (Docker/Wasmtime).
- **(d)** **L** — interface `Executor` + adaptateurs (Docker/SSH/local) ; l'abstraction est **M**, le rooting sûr est **L**.

### P10 — Hooks / événements de cycle de vie
- **(a)** Exécuter du code à des points précis (PreToolUse, PostToolUse, SessionStart, after-turn, subagent-spawn) ; sortie structurée `additionalContext`.
- **(b)** Gouvernance déterministe (formatage, lint, policy, audit) sans « demander au modèle ».
- **(c)** Codex `hooks.json`/`[hooks]` ; Claude Code hooks ; gptme hooks ; OpenClaw `api.on`/`registerHook` ; Cline lifecycle hooks.
- **(d)** **S** — bus d'événements typé + exécution de sous-processus avec timeout et capture de sortie.

### P11 — Mémoire longue durée bornée + recherche de sessions (FTS5)
- **(a)** Fichiers mémoire bornés (`MEMORY.md`/`USER.md`) injectés en **snapshot gelé**, + index **FTS5** de toutes les sessions, recherche/scroll sans LLM.
- **(b)** Apprentissage inter-sessions sans exploser le contexte ; recall proportionnel à la question, pas au corpus.
- **(c)** Hermes `features/memory` ; Claude Code auto-memory + `MEMORY.md` ; Codex `memories` ; gptme cross-harness memory.
- **(d)** **S** — SQLite FTS5 en Rust (`rusqlite`), bornes en caractères, outil `memory` (add/patch/remove) + `session_search`.

### P12 — Orchestration durable (durable execution) + human-in-the-loop
- **(a)** Reprise exacte après crash, checkpoints d'état, `interrupts` pour validation humaine, mémoire court/long terme.
- **(b)** Longue durée + supervision organique ; « gouverner » plutôt qu'« exécuter ».
- **(c)** LangGraph (durable execution, interrupts, memory) ; CrewAI Flows (état typé) ; OpenHands automations.
- **(d)** **L** — journal d'événements append-only + replay déterministe + points d'interruption. Candidat **microservice** (crash-boundary).

### P13 — Planification/cron + canaux distants
- **(a)** Tâches cron persistantes livrées vers la messagerie ; sessions déclenchées par webhook/événement ; remote sessions.
- **(b)** « Pilotage organique » depuis n'importe où ; automatisations récurrentes.
- **(c)** Hermes cron + gateway ; Cline `schedule create` + connecteurs ; Claude Code Routines/Desktop scheduled ; OpenHands automations.
- **(d)** **M** — scheduler persistant + adaptateurs de canaux (déjà MCP/ACP pour le reste).

### P14 — Observabilité unifiée (OTel)
- **(a)** Traces/métriques par run : tokens (par type, cache read/create), outils, approbations, spawns d'agents, compaction.
- **(b)** « Économie & traçabilité » (pilier 5) : mesurer, prouver, dater.
- **(c)** Codex (`[otel]`, catalogue de métriques) ; Claude Code (OTel exporter) ; LangSmith ; CrewAI AMP.
- **(d)** **S** — instrumenter les chemins critiques avec `tracing`/`opentelemetry-rust` ; pas besoin du catalogue complet au départ.

---

## 5. Techniques d'économie de contexte / tokens recensées

1. **Progressive disclosure (skills & tools).** Index court chargé en permanence, contenu complet à l'usage ; skills en 3 niveaux (Hermes), `disable-model-invocation` pour sortir totalement du contexte (Claude Code).
2. **Tool search / schémas différés.** Les schémas MCP/plugin ne sont pas dans le contexte par défaut ; le modèle recherche et charge le schéma à la demande (Claude Code, Hermes, Anthropic). Coût : 1-2 aller-retours à froid, pas de cache du schéma différé.
3. **Compaction d'historique.** `/compact` remplace l'historique par un résumé structuré ; rechargement des couches stables ; réinjection des skills invoquées (plafond). (Claude Code, Zed, Codex, Goose, Hermes `/compress`).
4. **Prompt caching par préfixe + ordre canonique.** Couches `system → project context → conversation` ; un changement dans le système invalide tout ; les couches stables restent cachées. **Actions qui invalident** à connaître : switch de modèle, changement d'effort, activation/désactivation d'un outil ou d'un serveur MCP (si non différé), compaction, upgrade du binaire. TTL 5 min vs 1 h. (Claude Code ; Responses API pour Codex.)
5. **Isolation par sous-agent.** Le travail exploratoire (lectures de fichiers, logs, stack traces) se fait dans un contexte séparé ; seul le **résumé** revient au parent (Claude Code, Codex, Cursor Explore, Hermes `delegate_task`).
6. **Repo map / index structurel.** Aider : tree-sitter + classement de graphe (PageRank) tronqué au budget (`--map-tokens`, défaut 1k) ; Plandex : project map tree-sitter (20M+ tokens indexables) et 2M tokens effectifs ; Cursor : Instant Grep local.
7. **Recherche déterministe au lieu de LLM.** `session_search` FTS5 (Hermes), grep indexé (Cursor), repo map (Aider) : le coût de calcul va au système. (Règle Omega §3.2.)
8. **Sorties verbeuses compressées / spilées.** Goose résume les sorties de commandes ; Hermes `output_spill` (head/tail + chemin de fichier au-delà d'un seuil) ; OpenClaw trim des vieux résultats d'outils ; hooks > 10 000 caractères → fichier + aperçu (Claude Code).
9. **Find-and-replace / ciblage minimal des éditions.** Goose et Hermes préfèrent patch/find-and-replace à la réécriture de gros fichiers (moins de tokens d'édition). Repo map + edits ciblés.
10. **Mémoire bornée + snapshot gelé.** Limites en caractères (Hermes 2 200/1 375), injection en snapshot fixe pour préserver le prefix cache ; l'agent consolide sous contrainte plutôt que de déverser.
11. **Budget avant action.** Budgets explicites (`max_input_tokens` de la revue Hermes, seuil de rate-limit pour générer les memories Codex) : arrêter plutôt que dépasser.
12. **Chargement paresseux des fichiers de règle.** Règles `path-scoped` (Claude Code `.claude/rules/` avec `paths:`) qui ne chargent que quand un fichier correspondant est touché.
13. **Context pollution / context rot — cadre conceptuel.** Codex documente explicitement le *context rot* (paper Chroma) comme justification des sous-agents. À reprendre dans la doc d'Omega.
14. **Modes read-only.** Agents `plan`/`Explore` sans outils d'écriture : moins d'échecs, contexte plus petit (ils sautent parfois `CLAUDE.md`).

---

## 6. Sources

**Docs officielles (agent)**
- Anthropic Claude Code — Overview : https://docs.claude.com/en/docs/claude-code/overview
- Claude Code — Subagents : https://docs.claude.com/en/docs/claude-code/sub-agents
- Claude Code — Memory : https://docs.claude.com/en/docs/claude-code/memory
- Claude Code — Context window : https://docs.claude.com/en/docs/claude-code/context-window
- Claude Code — Prompt caching : https://docs.claude.com/en/docs/claude-code/prompt-caching
- OpenAI Codex — Config avancée : https://developers.openai.com/codex/config-advanced
- OpenAI Codex — Subagents : https://developers.openai.com/codex/agent-configuration/subagents
- OpenAI Codex — Memories : https://developers.openai.com/codex/customization/memories
- Cursor — Recherche & indexation : https://cursor.com/docs/context/codebase-indexing
- Devin — Introducing Devin : https://docs.devin.ai
- Devin Desktop / Windsurf — Cascade : https://docs.windsurf.com/windsurf/cascade/cascade
- Zed — Agent Panel : https://zed.dev/docs/ai/agent-panel
- Goose — Architecture : https://goose-docs.ai/docs/goose-architecture/
- Hermes Agent — Docs : https://hermes-agent.nousresearch.com/docs
- Hermes Agent — Skills : https://hermes-agent.nousresearch.com/docs/user-guide/features/skills
- Hermes Agent — Memory : https://hermes-agent.nousresearch.com/docs/user-guide/features/memory
- Hermes Agent — Tool Search : https://hermes-agent.nousresearch.com/docs/user-guide/features/tool-search
- OpenClaw — Plugins : https://docs.openclaw.ai/plugins
- OpenClaw — Context engine : https://docs.openclaw.ai/concepts/context-engine
- MCP — Introduction : https://modelcontextprotocol.io/docs/getting-started/intro
- Agent Client Protocol — Introduction : https://agentclientprotocol.com/overview/introduction
- Agentskills.io (standard skills) : https://agentskills.io/specification
- Aider — Repository map : https://aider.chat/docs/repomap.html

**Dépôts GitHub (README)**
- Codex : https://github.com/openai/codex
- Cline : https://github.com/cline/cline
- Roo Code : https://github.com/RooCodeInc/Roo-Code
- Kilo Code : https://github.com/Kilo-Org/kilocode
- Aider : https://github.com/Aider-AI/aider
- OpenHands : https://github.com/OpenHands/OpenHands (SDK : https://github.com/OpenHands/software-agent-sdk)
- SWE-agent : https://github.com/SWE-agent/SWE-agent (et mini-SWE-agent : https://github.com/SWE-agent/mini-swe-agent)
- Continue : https://github.com/continuedev/continue
- Goose : https://github.com/block/goose (→ https://github.com/aaif-goose/goose)
- gptme : https://github.com/gptme/gptme
- Plandex : https://github.com/plandex-ai/plandex
- OpenCode : https://github.com/sst/opencode (→ https://github.com/anomalyco/opencode)
- Zed : https://github.com/zed-industries/zed
- Hermes Agent : https://github.com/NousResearch/hermes-agent
- OpenClaw : https://github.com/openclaw/openclaw
- MetaGPT : https://github.com/geekan/MetaGPT
- AutoGen : https://github.com/microsoft/autogen (successeur : https://github.com/microsoft/agent-framework)
- CrewAI : https://github.com/crewAIInc/crewAI
- LangGraph : https://github.com/langchain-ai/langgraph
- smolagents : https://github.com/huggingface/smolagents
- OpenAI Swarm : https://github.com/openai/swarm (successeur : https://github.com/openai/openai-agents-python)
- Dify : https://github.com/langgenius/dify
- n8n : https://github.com/n8n-io/n8n
- Flowise : https://github.com/FlowiseAI/Flowise (archivé)
- Open Interpreter : https://github.com/OpenInterpreter/open-interpreter

**Notes de provenance**
- Les faits de §1/§2 proviennent des README/docs ci-dessus `[observed]`, consultés le 2026-10-15.
- Les licences marquées « OSS (voir LICENSE) » (Plandex) et les langages non documentés (Cursor, Devin, Windsurf) doivent être confirmés avant réutilisation `[todo]`.
- Les écosystèmes évoluent vite (Roo Code arrêté le 15/05, Continue archivé, AutoGen en maintenance, Plandex Cloud en extinction) : re-vérifier avant de construire dessus.
