# Dissection — Crush

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Charm (`charmbracelet/crush`) |
| Licence | **FSL-1.1-MIT** — Functional Source License 1.1, MIT Future License (`LICENSE.md`) ; GitHub renvoie `NOASSERTION` |
| Langage | Go (module `github.com/charmbracelet/crush`) |
| Repo | https://github.com/charmbracelet/crush |
| Architecture | Monolithe Go en couches (`internal/*`) : noyau d'agent sur `charm.land/fantasy`, `coordinator` d'orchestration, services persistés SQLite/sqlc, config en couches interprétant un `crushrc` Bash, et deux surfaces — TUI Bubble Tea et serveur HTTP (`crush serve`) exposant une API v1 générée + SSE avec `workspaces` partagés par cwd. Catalogue providers/modèles déporté dans `charm.land/catwalk`. |

## 2. Entrées

- **Binaire TUI** : `crush` (`main.go:10` → `cmd.Execute()`, `internal/cmd/root.go:187`).
- **Binaire non interactif** : `crush run` (`internal/cmd/run.go:180`), streaming des événements.
- **Serveur** : `crush serve` — API v1 + SSE (`internal/server/endpoints.go:11`).
- **CLI d'administration** : `crush logs`, `crush update-providers`, `crush projects`, `crush models`, `crush auth/login/logout`.
- **Points d'extension** : MCP (`stdio`/`http`/`sse` + OAuth 2.1), LSP, hooks shell, Agent Skills, fournisseurs custom OpenAI/Anthropic-compat, `crushrc` Bash (exécution de config).
- **Fichiers de config** : `./.crushrc`, `./crushrc`, `~/.config/crush/crushrc`, `~/.local/share/crush/crush.json` (état) et `.crush/crush.json` (workspace) ; `$schema` = `https://charm.land/crush.json`.

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| TUI d'agent de code (client/serve) | ui | Interface terminal complète ; exécution détachée (202) | `internal/cmd/root.go:187`, `internal/server/endpoints.go:304-314` | service | planned | M |
| Sessions persistées (SQLite/sqlc) | memoire | CRUD sessions, tokens/coût, todos, canal, pubsub | `internal/session/session.go:51,67,417` | event-store → `EventStore` → `omega-app::{File,InMemory}EventStore` | adopted | M |
| Outils fichiers (view/write/edit/multiedit) | outils | Lecture/édition atomique, CRLF, filetracker | `internal/agent/tools/view.go:90`, `edit.go:58`, `multiedit.go:59`, `filetracker/service.go:38` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | adopted | M |
| Exécution shell + jobs d'arrière-plan | execution | Allowlist sûre, permission, background, spill | `internal/agent/tools/bash.go:198,242`, `internal/shell/background.go:91` | exec-policy → `ExecPolicy` → `omega-sandbox::{RuleBasedExecPolicy,LocalSandboxBackend}` | planned | L |
| File de permissions + grants persistants | securite | allow/deny, grant par session, auto-approve, skip, hook | `internal/permission/permission.go:188,135,165,309` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | adopted | M |
| Pont LSP (8 outils) | outils | Auto-start par filetype/racine, diagnostics/références/renommage | `internal/lsp/manager.go:38,253`, `tools/lsp_definition.go:34` | tool-surface → `ToolSurface` → `plugins/lsp-bridge` | planned | L |
| Intégration MCP (stdio/http/sse + OAuth) | outils | Multi-transport, OAuth 2.1, sessionless, filtrage | `internal/config/config.go:217`, `tools/mcp/init.go:338`, `tools/mcp/tools.go:113` | tool-surface → `ToolSurface` → `omega-mcp::McpToolSurface` | adopted | M |
| Catalogue multi-fournisseurs + Catwalk | routage | Providers + auto-update + découverte locale | `internal/config/provider.go:42`, `catwalk.go:31`, `discover/discover.go:107`, `coordinator.go:1319` | llm-provider → `Provider` → `omega-providers::{OllamaProvider,OllamaCatalogSource}` | adopted (partiel) | L |
| Configuration en couches | configuration | crushrc/JSON global/workspace, atomic write, scope | `internal/config/load.go:946,981`, `scope.go:12`, `store.go:301` | config-source → `ConfigSource` → `omega-config` | adopted | M |
| Compaction automatique | contexte | Seuil, résumé structuré, queue seule, reprise | `internal/agent/agent.go:1119,1370,1716` | context-engine → `ContextBudget` → `omega-context::{DefaultContextEngine,ContextBudget}` | adopted | M |
| Agents déclaratifs (coder/task/plan) | agents | Masques d'outils lecture seule/plan, MCP par agent | `internal/config/config.go:1116,1080,1086` | agent-store → `AgentStore` → `plugins/agent-declarative` | planned | M |
| Sous-agents isolés (outil `agent`) | agents | Sous-session, coût remonté, sans hooks | `internal/agent/agent_tool.go:1`, `coordinator.go:1665,1745` | subagent-runner → `SubagentRunner` → `plugins/subagent-isolated` | planned | L |
| Hooks shell à décision | automatisation | allow/deny/ask, agrégation, payload JSON/env | `internal/hooks/runner.go:90`, `hooks.go:46`, `input.go:32` | exec-policy → `ExecPolicy` → `plugins/hooks-runner` | planned | M |
| Agent Skills (SKILL.md) | extension | Multi-chemins, frontmatter invocation, injection XML | `internal/skills/skills.go:213,299`, `manager.go:173` | skill-store → `SkillStore` → `omega-context::SkillIndex` | adopted | M |
| Canaux MCP entrants | automatisation | Routage messages, réponse auto via outil | `internal/backend/channels.go:35`, `channelreply.go:78`, `config.go:295` | event-store → `EventStore` → `plugins/channel-router` | planned | L |
| Télémétrie pseudonyme (opt-out) | observabilite | PostHog, hash appareil, jamais de contenu | `internal/event/event.go:84`, `all.go`, `identifier.go:20` | telemetry-sink → `TelemetrySink` → `omega-otel::{Stdout,File}Sink` | adopted | S |
| Serveur HTTP + API v1/OpenAPI + SSE | execution | API générée, docs, SSE, contrôle d'arrêt | `internal/server/endpoints.go:11`, `server.go`, `events.go:103` | service | planned | L |
| Workspaces partagés par cwd | execution | Sessions/permissions/LSP/MCP partagés, first-wins | `README.md:499-531`, `backend/channels.go:120`, `events.go:296` | event-store → `EventStore` → `plugins/workspace-router` | planned | L |
| Suivi des fichiers lus | memoire | Par session + dernière lecture, API | `internal/filetracker/service.go:38,49`, `endpoints.go:206` | event-store → `EventStore` → `omega-app::FileEventStore` | adopted | S |
| Contexte git + init AGENTS.md + attribution | contexte | Statut/branche/commits injectés, init projet | `internal/agent/prompt/prompt.go:241,281`, `prompts.go:47`, `config.go:443` | context-engine → `ContextEngine` → `omega-context::ContextContributor` | planned | S |
| Todos par session | agents | Persistance + continuation | `internal/session/session.go:35`, `tools/todos.go:36` | context-engine → `ContextEngine` → `plugins/todos` | planned | S |
| Question interactive (batch) | agents | Question à l'utilisateur, interactif seul | `internal/agent/tools/question.go:83,58`, `coordinator.go:913` | tool-surface → `ToolSurface` → `plugins/question` | planned | S |
| Auto-vérification de version | configuration | Comparaison GitHub Releases | `internal/update/update.go:15,58` | service | planned | S |
| Journaux locaux + CLI | observabilite | `.crush/logs/crush.log`, `crush logs --tail/--follow` | `internal/cmd/logs.go:58`, `README.md:907-932` | telemetry-sink → `TelemetrySink` → `omega-otel::FileSink` | adopted | S |
| Détection de boucles | contexte | Stop si appels d'outils répétés | `internal/agent/loop_detection.go:11,19`, `agent.go:1121` | context-engine → `ContextEngine` → `plugins/loop-guard` | planned | S |
| Exécution du crushrc comme Bash | securite | **REFUSÉ** : config = code exécuté (supply-chain) | `internal/config/load.go:981`, `README.md:256-259,322-327` | rejected | rejected | M |

## 4. Traduction en plugins

Le mode d'intégration dominant est **plugin** (capacité → port → adaptateur), complété de **service** pour les surfaces de transport (TUI, serveur HTTP, auto-update) et d'un **rejet** de sécurité (exécution de config).

### 4.1 Adoptés — le bloc existe déjà chez nous

- `crush-mcp-integration` → `tool-surface` / `ToolSurface` → **`omega-mcp::McpToolSurface`** : le pont MCP existe déjà ; on y branche les transports stdio/http/sse, l'OAuth et le filtrage d'outils par serveur.
- `crush-permission-queue` → `permission-policy` / `PermissionPolicy` → **`omega-sandbox::ApprovalQueue`** : la file d'approbation existe ; on y ajoute les grants persistants par session et la pré-approbation par hook.
- `crush-provider-catalog` → `llm-provider` / `Provider` → **`omega-providers::{OllamaProvider,OllamaCatalogSource}`** : la feuille de catalogue et le provider Ollama existent ; il reste le scoring multi-fournisseurs (voir blocages).
- `crush-file-tools`, `crush-question-tool` → `tool-surface` / `ToolSurface` → **`omega-context::ToolSurfaceIndex`** : l'index d'outils existe ; on y enregistre view/write/edit/multiedit/glob/grep/ls et les outils MCP.
- `crush-skills` → `skill-store` / `SkillStore` → **`omega-context::SkillIndex`** : l'index de skills existe ; on y ajoute la découverte multi-chemins et les frontmatter d'invocation.
- `crush-compaction` → `context-engine` / `ContextBudget` → **`omega-context::{DefaultContextEngine,ContextBudget}`** : le moteur et le budget existent ; on y branche le seuil de résumé et le rechargement de la queue.
- `crush-config-layers` → `config-source` / `ConfigSource` → **`omega-config`** : les couches et la précédence existent (`LayeredConfig`) ; on y ajoute la portée globale/workspace et l'écriture atomique.
- `crush-sessions-persistence`, `crush-filetracker` → `event-store` / `EventStore` → **`omega-app::{File,InMemory}EventStore`** : le store d'événements durable existe ; sessions et fichiers lus sont modélisés comme événements.
- `crush-telemetry`, `crush-logging` → `telemetry-sink` / `TelemetrySink` → **`omega-otel::{Stdout,File}Sink`** : les sinks existent ; on y branche le catalogue d'événements pseudonymes et le fichier de log local.

### 4.2 Planned — bloc à construire

- **`plugins/lsp-bridge`** (`tool-surface` / `ToolSurface`) : gestionnaire LSP (auto-start par filetype/racine) et les 8 outils LSP.
- **`plugins/agent-declarative`** (`agent-store` / `AgentStore`) : agents coder/task/plan avec masques d'outils lecture seule/plan et MCP par agent.
- **`plugins/subagent-runner`** (`subagent-runner` / `SubagentRunner`) : sous-sessions isolées, coût remonté, pas d'interception de hooks.
- **`plugins/hooks-runner`** (`exec-policy` / `ExecPolicy`) : hooks shell à décision, agrégation, payload JSON/env.
- **`plugins/channel-router`** (`event-store` / `EventStore`) : canaux MCP entrants et réponse automatique via outil.
- **`plugins/workspace-router`** (`event-store` / `EventStore`) : workspaces partagés par cwd, first-wins, teardown.
- **`plugins/todos`**, **`plugins/loop-guard`**, **`plugins/question`** : helpers d'agents sur les ports `ContextEngine` / `ToolSurface`.
- **Services** : TUI client/serve, serveur HTTP + API v1/OpenAPI + SSE, auto-vérification de version.

### 4.3 Écarté

- **Exécution du `crushrc` comme Bash** (`crush-crusherc-exec`) : refusé. Le `crushrc` est du code exécuté à chaque démarrage et tout `$(...)` de `crush.json` s'exécute au chargement — un risque supply-chain que nous n'acceptons pas. Nous gardons une configuration **déclarative en couches (JSON)** et n'exécutons jamais la config comme du code.
- **Runtime JS in-process** : sans objet chez Crush (pas de code mode), donc non repris par construction.

## 5. Corroboration avec la preuve REA

Preuve machine : `docs/omega/research/rea/evidence/intake-crush.summary.json`.

- **Concordances** : la preuve confirme `id=crush`, `name=Crush`, `repo=charmbracelet/crush`, `stars=28674`, `language=Go`, `cloned=true`, `status=analyzed`, `nature=go`. Les 26 fonctionnalités ci-dessus sont toutes sourcées par `fichier:ligne` du clone, au-delà des 15 nœuds / 15 arêtes du graphe REA (`counts`), qui ne couvre qu'une ossature du code.
- **Limites de la preuve** :
  - **Licence** : la preuve porte `license: NOASSERTION` (GitHub n'a pas reconnu le FSL). Le SPDX exact, lu directement dans `LICENSE.md`, est **`FSL-1.1-MIT`** (Functional Source License 1.1, MIT Future License). C'est le champ retenu dans le JSON.
  - **Confiance `derived`** : la preuve est dérivée (`confidence: derived`), pas observée ; elle ne remplace pas la lecture du clone, qui reste la source de vérité pour les `how`.
  - **Graphe partiel** : 15 nœuds/15 arêtes ne cartographie ni les outils, ni les providers, ni MCP/LSP, ni la config. Notre inventaire s'appuie donc sur le code, pas sur le graphe.
  - **`provider: rea-javascript-application`** : nom de l'opérateur REA ; la nature Go est bien confirmée (`nature: go`), le libellé du provider étant un artefact de nommage.
  - **`limitations_count: 8`** : la preuve déclare 8 limites, non détaillées dans le résumé ; à ne pas citer comme « vérifié » sans relecture du rapport complet.

## 6. Sources

**Code (clone en lecture seule `/home/tayk0oo/omega-re/crush`)**

- `main.go:10-19`, `internal/cmd/root.go:187`, `internal/cmd/run.go:180`, `internal/cmd/logs.go:58`
- `internal/server/endpoints.go:11`, `internal/server/server.go`, `internal/server/events.go:103`, `internal/client/client.go`
- `internal/session/session.go:35,51,67,116,417,428`
- `internal/config/config.go:217,330,847,1038,1080,1086,1116`, `internal/config/load.go:64-80,946,981`, `internal/config/scope.go:12`, `internal/config/store.go:92,301`, `internal/config/provider.go:42`, `internal/config/catwalk.go:31`
- `internal/agent/agent.go:1119,1370,1716`, `internal/agent/agent_tool.go:1`, `internal/agent/coordinator.go:936,1319,1665,1745`, `internal/agent/loop_detection.go:11,19`, `internal/agent/prompt/prompt.go:241,281`, `internal/agent/prompts.go:47`
- `internal/agent/tools/` : `bash.go:198,242`, `view.go:90`, `edit.go:58`, `multiedit.go:59`, `write.go:46`, `job_output.go:33`, `job_kill.go:29`, `question.go:83`, `todos.go:36`, `lsp_*.go`, `mcp/init.go:338`, `mcp/tools.go:113`
- `internal/lsp/manager.go:38,253`, `internal/permission/permission.go:135,165,188,309`, `internal/skills/skills.go:213,299`, `internal/skills/manager.go:173`, `internal/filetracker/service.go:38,49`, `internal/backend/channels.go:35`, `internal/update/update.go:15,58`, `internal/shell/background.go:91`, `internal/discover/discover.go:107`

**Docs officielles**

- https://github.com/charmbracelet/crush — `README.md` (features, config, MCP OAuth, workspaces, providers, metrics)
- https://github.com/charmbracelet/crush/blob/main/LICENSE.md — FSL-1.1-MIT
- https://charm.land/crush.json — schéma JSON de config
- https://github.com/charmbracelet/catwalk — catalogue communautaire de providers/modèles
- https://agentskills.io — standard Agent Skills

**Preuve REA**

- `docs/omega/research/rea/evidence/intake-crush.summary.json` (voir §5 pour ses limites)
