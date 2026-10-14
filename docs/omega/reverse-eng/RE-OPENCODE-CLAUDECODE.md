# RE — OpenCode & Claude Code (T5 + T7)

> **Workstream :** W5 — Reverse-engineering des harnais.
> **Cibles :** T5 `sst/opencode` (MIT, TypeScript/Bun) · T7 Claude Code (propriétaire, RE par docs).
> **Date :** 2026-10-15 · **Branche :** `feat/agentos-omega`.
> **Provenance :** chaque affirmation est taguée `[observed]` (code/doc lue), `[stated]` (doc/vidéo déclarée), `[inferred]` (synthèse).
> **Règle appliquée :** patterns et idées uniquement. Aucun code propriétaire copié ; aucun contenu du system prompt fuité n'est reproduit (cf. §7).

---

## 1. Résumé exécutif

**OpenCode** est, parmi les harnais inventoriés, **le plus proche de la cible Omega « everything is a plugin » côté TS** `[inferred]`. Sa V2 (`opencode` 2.0.29, commit `3943b84`, clonée le 2026-10-15) `[observed]` pousse le modèle à un point que peu de harnais atteignent :

1. **Le noyau est lui-même composé de plugins internes** (`opencode.agent`, `opencode.command`, `opencode.skill`, `opencode.plan`, `opencode.prompt.*`, `opencode.config.*`, `opencode.tool.subagent`…). Zéro logique « en dur » : tout passe par les mêmes `transform`/`hook` que les plugins tiers `[observed]`.
2. **Deux verbes d'extension seulement** : `hook` (intercepter une opération vivante) et `transform` (contribuer à un état *rejouable*) + `reload`. C'est une surface d'API minuscule pour une extensibilité maximale `[observed]`.
3. **Les contrats sont des schémas Effect** (`@effect/schema`) avec interop **Standard Schema V1 + JSON Schema** — pas de zod ni de typebox `[observed]`.
4. **Le cycle de vie est scopé** : une inscription vit dans un `Scope` de plugin ; la fermer retire exactement son effet et révèle la couche précédente (overlay) `[observed]`.
5. **Le catalogue de modèles est construit par des transforms de plugins** (config, `models.dev`, auth) puis rejoué — décision d'archi documentée `[observed]`.

**Claude Code** apporte les mécanismes *contextuels* les plus matures du marché, documentés publiquement `[stated]` : **compaction structurée avec rechargement ciblé**, **prompt caching par préfixe** (breakpoints + TTL), **tool search** (schémas MCP différés), **skills en progressive disclosure** (corps rechargé ≤ 5 000 tokens/skill après compaction), **sous-agents** (frontmatter riche : `permissionMode`, `isolation: worktree`, `skills`, `mcpServers`, `hooks`), **plugins** (skills+agents+hooks+MCP) et **export OpenTelemetry** complet (metrics/logs/traces).

**Conclusion Omega** `[inferred]` : OpenCode fournit le **modèle de plugin** (registre + transforms rejouables + hooks + scopes + capacités). Claude Code fournit les **politiques de contexte** (compaction, caching, tool search, disclosure). Omega doit combiner les deux : un **registre de capacités** à la OpenCode, des **politiques de contexte** à la Claude Code, le tout derrière des traits Rust.

---

## 2. OpenCode — architecture, registre de plugins, contrats

### 2.1 Vue d'ensemble

- **Monorepo Bun + Turbo**, `packages/*` (~4 263 fichiers `.ts`), V2 = `"version": "2.0.29"` `[observed]` (`package.json`, `packages/cli/package.json`).
- **Licence MIT**, `Copyright (c) 2025 opencode` `[observed]` (`LICENSE`).
- Packages pivots `[observed]` (`packages/`) : `schema` (formes publiques + événements durables), `core` (runtime), `protocol` (HTTP/OpenAPI), `plugin` (SDK tiers + API interne), `plugins` (ex. `figma`), `server`, `cli`, `tui`, `desktop`, `app` (web), `sdk`, `client`, `codemode`, `ai` (couche modèles/protocoles).
- **Autorité par préoccupation** `[observed]` (`specs/v2/README.md`) :
  | Concern | Owner |
  |---|---|
  | HTTP operations + erreurs de transport | `packages/protocol` (HttpApi assemblé par Server) |
  | Formes de domaine publiques + événements durables | `packages/schema` |
  | Comportement runtime + persistance | `packages/core` |
  | Spécifications cross-module | `specs/v2/*.md` |

C'est **exactement la séparation contrats / runtime / transport** que RULES-OMEGA impose (ports → adaptateurs, schéma typé et versionné à chaque frontière) `[inferred]`.

### 2.2 Le registre de plugins

**Contrat public** `[observed]` (`packages/schema/src/plugin.ts`) :

- `Source` (l.10) = union taggée :
  - `{type:"builtin"}` — plugin interne au binaire ;
  - `{type:"package", target, version?, outdated?, updating?}` — paquet npm versionné et *updatable* ;
  - `{type:"local", path}` — chemin local ;
  - `{type:"sdk"}` — fourni par un hôte SDK.
- `Features` (l.24) = `{ server?, tui?, rpc? }` → **une capacité par surface d'exécution**. Un plugin déclare *où* il s'exécute.
- `Info` (l.38) = `{ id?, source, features, state }`.
- `State` (l.31) = `{status:"active"}` | `{status:"failed", error, ref?}` — un plugin peut échouer sans tuer le process.
- Événement `plugin.updated` (éphémère) → invalidation côté UI.

**Déclaration en configuration** `[observed]` (`packages/schema/src/config/plugin.ts:6-14`) : `plugins: (string | {package, options})[]` — un plugin = **une chaîne ou un manifeste + options typées** (`PluginOptions = Record<string, any>`).

**Découverte & chargement** `[observed]` :
- `packages/plugin/src/host.ts` : `resolve(target)` essaie les sous-chemins `server`, `tui`, `rpc`, puis `index` → **entrypoints multiples par surface** ; `load(entrypoint)` importe dynamiquement.
- `packages/plugin/src/source.ts:8` `createPluginSources(watch)` : empreinte **sha256** des fichiers, **cache de l'échec d'import** (un module en échec n'est pas réévalué à chaque event), **watch** des fichiers → reload sur changement. `localSource` (l.61) accepte `file://`, `./…`, chemins absolus.
- `packages/plugin/src/source.node.ts` / `.bun.ts` / `.package.ts` : trois stratégies de préparation selon l'hôte (Node, Bun, paquet).
- `packages/core/src/plugin/service.ts` + `host.ts` : cycle de vie (activation, `Scope`, supervision, `hold`/`ready`), `packages/core/src/plugin/supervisor.ts`.

**Supervision / mise à jour** `[observed]` : `packages/protocol/src/groups/plugin.ts:10-49` expose `plugin.list`, `plugin.check` (updates disponibles), `plugin.update` (« update concurrent + notify active locations to reload »).

### 2.3 Les deux verbes d'extension, et rien de plus

`[observed]` (`packages/plugin/src/README.md`, `src/effect/README.md`, `src/promise/registration.ts:1-21`) :

- `hook(name, callback)` → **intercepter une opération vivante** (runtime).
- `transform(callback)` → **contribuer à un état rejouable** (déclaratif) ; l'éditeur est **synchrone**, donc on charge ses données async *avant* d'enregistrer.
- `reload()` → rejoue tous les transforms du domaine.
- Les deux API `@opencode/plugin` (Promise) et `@opencode/plugin/effect` (Effect) offrent **les mêmes capacités**.

**Domaines de transform** `[observed]` : `agent`, `command`, `integration`, `mcp`, `model`, `provider`, `reference`, `skill`, `tool`, `vcs`, `websearch`.
**Domaines de hook** `[observed]` : `aisdk` (`sdk`, `language`), `session` (`context`, `retry`, `compaction`, `generate`), `tool` (`execute.before`, `execute.after`), `permission`, `event`, `storage`, `rpc`, `shell`, `worktree`, `plugin`.

Exemple clé `[observed]` (`packages/core/src/plugin/host.ts`) : le `PluginHost` n'expose à un plugin que la **capacité étroite** du domaine (ex. `tools.register`), jamais le registre interne. C'est le principe ISP (ports fins) appliqué littéralement.

### 2.4 Loi de superposition (« overlay »)

`[observed]` (`specs/v2/tools.md:75`, « Scoped registration ») :
> Dans un placement donné : la **dernière inscription active gagne** ; fermer une inscription ne retire que celle-là ; fermer la gagnante **révèle la précédente** ; muter la record source plus tard n'affecte pas l'inscription capturée.

C'est un **empilement par Scope** qui donne gratuitement : thème/policy par projet, override par plugin, désactivation propre, sans « undo » manuel.

### 2.5 Schémas de contrat

- **Format : `effect/Schema`** `[observed]` (`packages/schema/src/*`). `Tool.ValueSchema` accepte **Effect Schema | Standard Schema V1 | JSON Schema** `[observed]` (`packages/schema/src/tool.ts:34`). Chaque schéma porte un `.annotate({ identifier })` → **JSON Schema stable** pour l'OpenAPI dérivé.
- **Agents** `[observed]` (`packages/schema/src/agent.ts:23`) : `{ id, name, model?, request, system?, description?, mode:"subagent"|"primary"|"all", hidden, color?, steps?, permissions }`. Le constructeur `default(id)` (l.39) pose des permissions par défaut (`*.env` → `ask`).
- **Commands** `[observed]` (`packages/schema/src/command.ts`) : `{ name, description? }` ; config `{ template, description?, agent?, model?, subagent? }` (`config/command.ts`).
- **Skills** `[observed]` (`packages/schema/src/skill.ts:27`) : `{ id, name, description?, autoinvoke?, path, content }` ; `Source` = `directory | url | embedded`. Le pull réseau lit un `index.json` (`packages/core/src/skill/discovery.ts`) avec garde-fous anti-traversée (`isSafeSegment`, `isSafeRelativePath`).
- **Tool** `[observed]` (`packages/schema/src/tool.ts`) : `Tool.make({ description, input, output, execute })` ; `Context = { sessionID, agent, messageID, id, progress }` ; `Result = { output?, content?, metadata? }` — **trois valeurs distinctes** : `output` (valeur machine validée pour Code Mode), `content` (valeur modèle durable), `metadata` (JSON compact UI) `[observed]` (`specs/v2/tools.md:8-15`).
- **Provider** `[observed]` (`packages/schema/src/provider.ts`) : `{ id, canonical?, integrationID?, name, activation:"auto"|"enabled"|"disabled", package, compatibility?, settings, headers, body }`. `Provider.ID` liste en `statics` les fournisseurs connus (`anthropic`, `openai`, `google`, `github-copilot`, `amazon-bedrock`, `openrouter`, `mistral`…) — **mais la liste est un défaut, pas un verrou** : un provider custom arrive par `Config.Provider.Info` (`config/provider.ts`) avec `models`, `cost` (input/output/**cache.read**/**cache.write**), `limit`, `capabilities`, `variants`.
- **Policy** `[observed]` (`packages/schema/src/config/policy.ts`) : `{ action: "provider.use"|"tool.use"|"integration.use", resource, effect:"allow"|"deny" }`.
- **Permissions** `[observed]` : `Permission.Ruleset` (règles ordonnées `{action, resource, effect:"allow"|"ask"|"deny"}`), évaluées par `Permission.evaluate` (`packages/core/src/skill.ts:31`).

### 2.6 Extensibilité déclarative en pratique

- **Agents markdown** `[observed]` (`packages/core/src/config/plugin/agent.ts:24-30`) : chargés depuis `{agent,agents}/**/*.md` et `{mode,modes}/*.md` (frontmatter YAML via `gray-matter`, avec repli de *sanitize* pour les valeurs contenant `:` `[observed]` `config/markdown.ts`). **Compat** : OpenCode lit aussi `~/.claude/` et `~/.agents/` (`config/discovery.ts`, `config.ts:72-74`), et un plugin `opencode.config.compatibility` charge les **skills** de ces dossiers `[observed]`.
- **Commandes markdown** `[observed]` (`config/plugin/command.ts`) : depuis un répertoire ; un command peut exécuter un `session.prompt`, un shell, ou un subagent.
- **Skills** `[observed]` : injectées à la demande ; l'output modèle est un bloc `<skill_content>` + `<skill_files>` (liste échantillonnée ≤ 10 fichiers) `[observed]` (`packages/core/src/skill.ts:36-56`).
- **Compaction** `[observed]` (`packages/core/src/session/compaction.ts`) : `compile` produit un **résumé structuré** (template `Objective / Requirements / Decisions / Work State (Completed|Active|Blocked) / Next Move / Relevant Files / Important Context`, l.104-136), avec `SHRINK_STEPS=[0.7,0.5,0.35]` sur rejet « trop long », `RESERVE_MIN=16_000`, `UNKNOWN_WINDOW=200_000`, `keep` par défaut `15_000` tokens (l.197). Config `auto`/`buffer`/`keep.tokens` (`config/compaction.ts`).
- **Plan mode** `[observed]` (`packages/core/src/plugin/plan.ts`) : un plugin ajoute un agent `plan` en lecture seule (edit `deny` sauf `~/.opencode/plan`), et un hook `session.context` **ré-appose ou retire** le rappel de mode **près de la queue** pour préserver le préfixe de cache.
- **Sous-agents** `[observed]` (`packages/core/src/tool/plugin/subagent.ts`) : outil `subagent` qui crée une **session enfant**, avec `background`, `sessionID` (reprise), `model` (`provider/model#variant`) ; les sous-agents builtins `general` et `explore` ont des permissions restreintes (`packages/core/src/plugin/agent.ts:87-119`).

### 2.7 Prompt caching (côté OpenCode)

`[observed]` (`packages/ai/src/cache-policy.ts`) :
- Politique `cache: "auto" | "none" | {…}` ; `AUTO = {tools:true, system:true, messages:{tail:1}}` (l.17).
- **`BREAKPOINT_CAP = 4`** breakpoints (l.24) ; les `CacheHint` manuels comptent dans le budget.
- `RESPECTS_INLINE_HINTS` (l.40) : ensemble d'IDs de protocoles qui honorent `cache_control` (anthropic-messages, bedrock-converse, openrouter, vercel-ai-gateway-messages, alibaba, …). Les autres (OpenAI implicite, Gemini hors bande) sont **skippés**.
- Politiques spéciales : `QWEN` (tools exclus) et `gatewayPolicy(modelID)` (route par préfixe d'ID).
- **Affinité de session** `[observed]` (`packages/core/src/session/affinity.ts:7`) : la clé de routage cache = `parentID ?? fork.sessionID ?? session.id` (les enfants partagent le cache du parent).
- `PromptCacheKeyPlugin` `[observed]` (`packages/core/src/plugin/provider/prompt-cache-key.ts`) pose `supportsPromptCacheKey` sur une liste auditée de providers.
- `Model.Compatibility.cacheFormat: "anthropic"|"alibaba"` `[observed]` (`packages/schema/src/model-compatibility.ts:16`).

### 2.8 Divers utiles

- **Code Mode** `[observed]` (`packages/codemode/`) : le modèle écrit du code appelant les outils ; les schémas JSON sont **rendus en types TypeScript** (`tool-schema.ts` `MAX_RENDER_DEPTH=8`, gestion `$ref`/`$defs`, tags `@deprecated`/`@default`/`@format`). Option `tool.codemode` / `pinned` (`schema/tool.ts:35`).
- **Event stream** `[observed]` : événements `durable(...)` vs `ephemeral(...)` + `inventory(...)` (`schema/event.ts`), manifeste durable (`event-manifest.ts`), décision documentée `specs/v2/event-stream-architecture.md`.
- **Catalogue rejouable** `[observed]` : `specs/v2/catalog-config-plugin-lifecycle.md` — **Option B retenue** : « Catalog Transforms ». Config, `models.dev` et auth **enregistrent des transforms rejouables** ; la désactivation les retire et le catalogue se rematérialise sans « undo ».
- **LSP / formatter** `[observed]` : configurables (`schema/src/config/lsp.ts`, `formatter.ts`) — plugins-adaptateurs.
- **Agent `build` par défaut** `[observed]` (`packages/core/src/agent.ts:13` `defaultID = "build"`).

---

## 3. Claude Code — mécanismes documentés

> Aucun clone (propriétaire). RE par **documentation publique** `[stated]` (`code.claude.com/docs`, `platform.claude.com/docs`).

### 3.1 Compaction (`/compact` + auto)

`[stated]` (`how-claude-code-works#the-context-window`, `context-window`) :
- Auto : « clears older tool outputs first, then summarizes » ; si un fichier/output est trop gros, l'auto-compaction **s'arrête après quelques tentatives** (protection contre le « thrashing ») au lieu de boucler.
- Le résumé **structuré** conserve : requêtes/intention, concepts techniques clés, fichiers examinés/modifiés + extraits importants, erreurs et corrections, tâches pendantes, état du travail. C'est le **remplacement exact** de la conversation verbatim (outputs complets et raisonnements intermédiaires perdus).
- **Rechargement ciblé** : contenu de démarrage (system prompt, CLAUDE.md, auto-memory, MCP, *sauf la liste des descriptions de skills*) rechargé ; **re-lecture des ≤ 5 fichiers modifiés les plus récemment** ; **ré-injection du corps de chaque skill invoqué, plafonné à 5 000 tokens par skill**.
- Contrôle : section `Compact Instructions` dans CLAUDE.md, `/compact focus on the API changes`, `autoCompactWindow` (100 k–1 M) par sous-agent.
- Hooks `PreCompact` / `PostCompact` (matcher `manual|auto`) `[stated]` (`hooks`).

### 3.2 Prompt caching par préfixe

`[stated]` (`build-with-claude/prompt-caching`) :
- Deux modes : **automatic** (`cache_control` top-level, le breakpoint avance vers le dernier bloc cacheable) et **explicit** (`cache_control` sur des blocs, **max 4 breakpoints**).
- Ordre de hiérarchie : **tools → system → messages** ; toute modif à un niveau invalide ce niveau **et tous les suivants**.
- **Lookback window = 20 blocs** ; en cas de conversation longue, un 2ᵉ breakpoint plus proche s'impose.
- TTL : **5 min par défaut** (rafraîchi gratuitement à la lecture), **1 h à 2×** le prix input. Sous-agent : `experimental.cacheTtl: 5m|1h`.
- Métriques : `cache_creation_input_tokens`, `cache_read_input_tokens`, `input_tokens` (= tokens *après* le dernier breakpoint). Minimums de longueur par modèle.
- Piège documenté : placer le breakpoint sur un bloc qui change (timestamp) → **jamais de hit**.

### 3.3 Tool search (schémas MCP différés)

`[stated]` (`mcp#scale-with-mcp-tool-search`) :
- Par défaut, seuls **noms d'outils + instructions de serveur** chargent au démarrage ; les **définitions complètes sont différées** et chargées à la demande via l'outil `ToolSearch` (renvoie des blocs `tool_reference`).
- Contrôle : `ENABLE_TOOL_SEARCH` = `unset` (déféré) / `true` / `auto` / `auto:N` (charge en amont tant que < N % de la fenêtre) / `false`. Repli automatique si `ANTHROPIC_BASE_URL` non-first-party, modèles anciens, Foundry/Azure.
- Par serveur : `alwaysLoad: true|false` ; par outil : `_meta."anthropic/alwaysLoad"`.
- Limites de description : **4 096 car.** (chargé en amont) vs **16 384 car.** (chargé par search) ; `CLAUDE_CODE_MAX_MCP_DESCRIPTION_LENGTH`.
- `WaitForMcpServers` remplace la recherche quand tool search est off.

### 3.4 Skills (progressive disclosure)

`[stated]` (`skills`) :
- `SKILL.md` + frontmatter YAML ; suit le standard ouvert **Agent Skills** (agentskills.io) étendu.
- **Progressive disclosure** : au démarrage seules les `description` (+ `when_to_use`) sont chargées, **tronquées à 1 536 car.** ; le corps ne charge qu'à l'usage ; après `/compact` le corps est ré-injecté **≤ 5 000 tokens/skill**.
- Frontmatter clés : `description`, `when_to_use`, `disable-model-invocation`, `user-invocable`, `allowed-tools`, `disallowed-tools`, `model`, `effort`, `context: fork` (+ `agent`, `background`), `paths` (activation par glob), `hooks` (+ `once`), `shell`, `metadata`, `license`, `compatibility`.
- **Les commandes custom fusionnent dans les skills** (`.claude/commands/x.md` ≡ `.claude/skills/x/SKILL.md`).
- Détection live des changements ; `/reload-skills`.

### 3.5 Sous-agents

`[stated]` (`sub-agents`) :
- Fichiers markdown à frontmatter : `name`, `description` (requis) + `tools`, `disallowedTools`, `model` (alias/ID/`inherit`), `permissionMode` (`default|acceptEdits|auto|dontAsk|bypassPermissions|plan`), `maxTurns`, `autoCompactWindow`, `skills` (préchargées en entier), `mcpServers`, `hooks`, `memory`, `background`, `omitClaudeMd`, `effort`, **`isolation: worktree`**, `color`, `initialPrompt`, `experimental.cacheTtl`.
- **Built-ins** : `Explore` (read-only, file discovery), `Plan` (recherche pour plan mode), `general-purpose` (tout). Explore/Plan **démarrent sans CLAUDE.md ni git status** ; budget total des descriptions de sous-agents custom **plafonné à 15 000 tokens** (warning au-delà).
- Contexte **isolé** : chaque sous-agent a sa propre fenêtre ; seul un résumé revient (ciblé sur les économies de contexte).
- Restrictions plugin : les sous-agents de plugin **ignorent** `hooks`, `mcpServers`, `permissionMode` (sécurité).

### 3.6 Plugins

`[stated]` (`plugins/overview`) :
- **Un dossier de composants** chargé comme une unité : manifeste `.claude-plugin/plugin.json` + `skills/`, `agents/`, `hooks/hooks.json`, `.mcp.json`, `output-styles/` ; « mods » = hooks JS in-process.
- Distribués par **marketplace** (`.claude-plugin/marketplace.json`) ; scopes `user` / `project` / `local` ; namespacing `/plugin-name:skill-name` et serveurs `plugin:<plugin>:<server>` ; outils MCP `mcp__plugin_<plugin>_<server>__<tool>`.
- Coût : descriptions de skills/agents/commandes présents **à chaque tour** ; le texte complet ne charge qu'à l'usage.

### 3.7 Hooks (cycle de vie)

`[stated]` (`hooks`) :
- Événements : `SessionStart/End`, `Setup`, `UserPromptSubmit`, `UserPromptExpansion`, `PreToolUse`/`PostToolUse`/`PostToolUseFailure`/`PostToolBatch`, `PermissionRequest`/`PermissionDenied`, `Notification`, `MessageDisplay`, `SubagentStart`/`Stop`, `TaskCreated`/`Completed`, `TeammateIdle`, `Stop`/`StopFailure`, `InstructionsLoaded`, `ConfigChange`, `CwdChanged`, `DirectoryAdded`, `FileChanged`, `WorktreeCreate`/`Remove`, `PreCompact`/`PostCompact`, `PreModelSwitch`/`PostModelSwitch`, `Elicitation`/`ElicitationResult`.
- Types de handler : `command`, `http`, `mcp_tool`, `prompt`, `agent` ; `matcher` + `if` (syntaxe de règle de permission) ; décision via **exit code + JSON stdout** (`permissionDecision`, `hookSpecificOutput.additionalContext`) ; `async`, `asyncRewake`, `once`, `onFailure`.

### 3.8 OpenTelemetry

`[stated]` (`monitoring-usage`) :
- Métriques + logs/événements + (beta) **traces** via variables `OTEL_*` standard ; `CLAUDE_CODE_ENABLE_TELEMETRY=1`.
- **Hiérarchie de spans** : `claude_code.interaction` → `claude_code.llm_request` / `claude_code.tool` (→ `.blocked_on_user` + `.execution`, et sous-spans de sous-agent) / `claude_code.hook`.
- Attributs `gen_ai.*` (conventions OTel GenAI) ; `cache_read_tokens` / `cache_creation_tokens` sur `llm_request`.
- **Redaction par défaut** et *gates* opt-in : `OTEL_LOG_USER_PROMPTS`, `OTEL_LOG_TOOL_DETAILS`, `OTEL_LOG_TOOL_CONTENT`, `OTEL_LOG_RAW_API_BODIES` ; limite de contenu 60 KB (`CLAUDE_CODE_OTEL_CONTENT_MAX_LENGTH`).
- **Contrôle admin** : les managed settings *verrouillent* la destination (elles retirent les variables développeur conflictuelles) ; variables `OTEL_*` **ignorées** dans `.claude/settings.json` de repo.
- Propagation W3C `TRACEPARENT` aux sous-processus ; `traceparent` sur les requêtes modèle/MCP.

### 3.9 Corpus local (TikTok @0xloucash)

`[observed]` (`docs/omega/research/loucash/`) :

| id | Titre | Pertinence |
|---|---|---|
| `7659512794085166358` | « Le système prompt complet de Claude … a fuité sur GitHub, 120 000 caractères » (vidéo #30) | Prompt fuité — **non reproduit** (propriétaire) |
| `7623863018832596246` | « Le code de Claude a fuité … 4 fonctionnalités cachées » | Pistes features |
| `7624269671780617494` | « 10 fonctionnalités … de Claude Code » | Inventaire features |
| `7623453038639926550` | « … 10× plus puissant avec seulement 4 fichiers » | Claude Code déclaratif (CLAUDE.md/règles/skills/hooks) |
| `7620479472445295894` | Récap : Telegram, scheduled tasks, remote sessions, **subagents** | Sous-agents / surfaces |

Les descriptions de vidéos sont `[stated]` par le créateur et non vérifiées `[inferred]`. Le contenu du system prompt fuité n'est **pas** utilisé (règle : patterns seulement, aucun contenu propriétaire).

---

## 4. Ports & contrats extraits (re-spécification originale)

Ces ports sont des **spécifications Omega originales** inspirées des deux harnais — pas des copies.

```text
Port  PluginRegistry
  register(manifest: PluginManifest) -> Handle
  list() -> [PluginInfo]
  status(id) -> { active | failed(cause) }
  update(target) / check(target) -> [PluginInfo]
  # manifest: { id, source(builtin|package|local|sdk), capabilities: [server|tui|rpc|...], options }
  # invariant: un plugin peut echouer sans arreter le noyau

Port  ContextEngine                     # "everything is a plugin" applique au contexte
  register_transform(domain, fn) -> Registration   # rejouable (declaratif)
  register_hook(domain, event, fn) -> Registration # interception live
  reload(domain)
  # invariant: fermer une Registration retire exactement son effet (Scope)
  # invariant: la derniere inscription active gagne ; fermer revele la precedente

Port  ModelRegistry                     # multi-fournisseurs, catalogue rejouable
  providers() -> [ProviderInfo]         # { id, activation(auto|enabled|disabled), package, compat }
  models(filter) -> [ModelInfo]         # cost{in,out,cache_read,cache_write}, limit, capabilities
  # la config, le catalogue distant (models.dev-like) et l'auth sont des transforms rejouables

Port  PromptCachePolicy
  plan(request) -> [Breakpoint]         # tools -> system -> messages ; <= 4 breakpoints
  affinity(session) -> CacheKey         # parent ?? fork ?? self
  # ttl: 5m | 1h ; protocoles qui ignorent les marqueurs => no-op

Port  ContextBudget                     # compaction
  estimate(request) -> tokens
  compact(trigger{auto|overflow|manual}, keep, buffer) -> Outcome
  # sortie: summary struct + recent verbatim + (option) provider_context natif
  # invariant: le transcript complet reste durable ; ne jamais boucler (anti-thrash)

Port  ToolRegistry
  register(name, tool: {description, input, output, execute}) -> Registration
  snapshot(request) -> {definitions, captured_tools}
  # output (valeur machine) / content (valeur modele) / metadata (UI) distincts

Port  ToolSurface                       # progressive disclosure / tool search
  index() -> [ToolStub]                 # nom + description seulement
  hydrate(query) -> [ToolSchema]        # schemas charges a la demande
  always_load(server|tool) -> bool

Port  SkillStore
  discover(sources) -> [SkillInfo]
  load(id) -> {body, files, base_dir}
  # disclosure progressive ; plafond de reinjection (ex. 5000 tokens/skill) apres compaction

Port  SubagentRunner
  spawn(spec{agent, prompt, model?, isolation?, background?}) -> ChildHandle
  resume(session_id, message) -> Result
  # contexte isole ; seul le resume revient ; worktree pour isolation

Port  PermissionPolicy                  # allow / ask / deny + policy provider/tool/integration
  evaluate(action, resource, scope) -> Effect
  ruleset(agent|skill|subagent) -> [Rule]

Port  TelemetryExporter                 # OTel
  emit(metric|event|span) ; redaction par gates ; destination verrouillable par policy
```

**Décisions de contrat reprises**
| # | Décision | Source |
|---|---|---|
| C1 | Schémas avec interop **Standard Schema V1 + JSON Schema** à chaque frontière | OpenCode `tool.ts:34`, `specs/v2/README.md` `[observed]` |
| C2 | **Un plugin peut échouer** (`State.failed` + ref) sans arrêter le noyau | `schema/plugin.ts:31` `[observed]` |
| C3 | Registres **superposables par Scope**, fermeture ⇒ révélation de la couche précédente | `specs/v2/tools.md:75` `[observed]` |
| C4 | Entrypoints **par surface** (`server`/`tui`/`rpc`) | `plugin/src/host.ts` `[observed]` |
| C5 | **Rejouabilité** : transform déclaratif + `reload` ⇒ désactivation sans undo | `plugin/src/README.md`, `catalog-config-plugin-lifecycle.md` `[observed]` |
| C6 | **Idempotence** : empreinte des fichiers + cache d'échec d'import | `plugin/src/source.ts:8` `[observed]` |
| C7 | Clé d'affinité cache = parent/fork/self | `session/affinity.ts:7` `[observed]` |
| C8 | Breakpoints **≤ 4**, ordre tools→system→messages | `cache-policy.ts:24` + docs Claude Code `[observed]/[stated]` |

---

## 5. Features détaillées — valeur Omega × difficulté de portage Rust

> **Valeur Omega (0–3)** : contribution aux 6 piliers / RULES-OMEGA. **Difficulté Rust** : S (≤ 1 jour-agent), M (2–5 j), L (> 5 j, orchestration multi-crates).

| # | Feature | Origine | Valeur | Difficulté Rust | Dépendances |
|---|---|---|---|---|---|
| F1 | **Context Engine enfichable** (transforms rejouables + hooks + Scope) | OpenCode | **3** | M | registry, events |
| F2 | **Registre de plugins** (manifest + capacités server/tui/rpc + update/check) | OpenCode | **3** | M | fs, semver, source fingerprint |
| F3 | **Agents déclaratifs** (frontmatter + permissions + mode) | OpenCode + CC | **3** | S | yaml, markdown |
| F4 | **Skills / commands déclaratifs + progressive disclosure** | OpenCode + CC | **3** | M | skill store, budget tokens |
| F5 | **Compaction structurée + rechargement ciblé** | CC | **3** | M | compaction, budget, snapshots |
| F6 | **Prompt caching par préfixe** (breakpoints + policy + affinité) | OpenCode + CC | **3** | M | provider compat, session |
| F7 | **Tool search / schémas différés** | CC | 2 | M | tool registry, index |
| F8 | **Sous-agents isolés parallèles** (+ `isolation: worktree`) | OpenCode + CC | **3** | L | sessions, worktree, jobs |
| F9 | **Hooks d'événements de cycle de vie** (matcher + décision) | CC + OpenCode | **3** | M | bus, event manifest |
| F10 | **Policy/permissions** (allow/ask/deny + provider/tool/integration) | OpenCode | **3** | S | règles ordonnées |
| F11 | **Registre multi-fournisseurs + catalogue communautaire rejouable** | OpenCode | **3** | M | config transforms, compat |
| F12 | **Export OpenTelemetry** (metrics/logs/traces + gates) | CC | 2 | S–M | OTel, redaction |
| F13 | **Code Mode** (outils appelés via code sandboxé) | OpenCode | 2 | L | sandbox, TS→schémas |
| F14 | **Event stream durable vs éphémère** + manifeste | OpenCode | 2 | M | store, protocole |
| F15 | **Snapshots / revert / worktrees / rollback** | les deux | **3** | L | git, fs, storage |
| F16 | **Client MCP + notifications `list_changed`** | les deux | **3** | M | MCP, transport |
| F17 | **Hooks en JS in-process** (« mods ») | CC | 1 | L | runtime JS embarqué (hors périmètre Rust) |
| F18 | **Marketplace de plugins** (catalogue + scopes + namespacing) | CC | 2 | M | F2, package manager |
| F19 | **Mémoire auto** (MEMORY.md bornée, 200 lignes/25 KB) | CC | 2 | S | memory plugin |
| F20 | **Plan mode** (agent lecture seule + rappel ré-apposé) | OpenCode | 2 | S | F3, F9 |

**Lecture** `[inferred]` : les quick wins Rust = **F3, F10, F19, F20** (S). Le cœur de valeur = **F1, F2, F5, F6, F9, F11** (M). Les gros chantiers = **F8, F13, F15** (L), à traiter après les ports et contrats.

---

## 6. Ce qu'Omega reprend / améliore / écarte

### 6.1 Reprend (tel quel, ré-implémenté proprement)
- **« Everything is a plugin » littéral** : le noyau Rust expose ses agents/commands/skills/tools comme des plugins internes enregistrés par le même port que les plugins tiers `[inferred]` (démontré faisable par OpenCode `[observed]`).
- **Deux verbes d'extension** `transform` (rejouable) + `hook` (live) + `reload` : surface d'API minimale, testable, sérialisable.
- **Superposition par Scope** : désactivation propre, override par projet, zéro « undo ».
- **Contrats schéma-first** avec interop JSON Schema, et **séparation contrats / runtime / transport**.
- **Compaction structurée** + **rechargement ciblé** (system prompt, instructions, ≤ N fichiers récents, skills réinjectés ≤ plafond).
- **Prompt caching par préfixe** : breakpoints ≤ 4, ordre tools→system→messages, TTL 5m/1h, clé d'affinité parent/fork/self.
- **Tool search** : index (noms+descriptions) vs schémas différés, `always_load` par serveur/outil.
- **Sous-agents** avec contexte isolé, `isolation: worktree`, reprise par session id, modèle par invocation.
- **Hooks** avec matcher + décision structurée (exit code + JSON).
- **Policy** allow/ask/deny + policy provider/tool/integration.
- **Manifeste de plugin** avec capacités par surface + état `failed` non fatal.

### 6.2 Améliore
- **Provenance native** : chaque événement/artefact porte `[stated]/[observed]/[inferred]` + source (RULES-OMEGA §4) — absente des deux harnais.
- **Budget économique d'abord** : chaque `estimate/compact`/appel modèle est **tracé, plafonné, daté** ; le coût de calcul va au système (RULES-OMEGA §3), là où Claude Code se contente de « clear older tool outputs first ».
- **Zéro fournisseur en dur** : la liste `Provider.ID` d'OpenCode est un *défaut* ; Omega impose un **manifeste signé** + scoring communautaire (pilier 1).
- **Traits Rust + versionnage** : les ports (§4) sont des traits Rust avec schéma `serde`/JSON Schema **versionné**, là où OpenCode mélange types Effect et JSON Schema sans version explicite.
- **Observabilité sans dépendance fournisseur** : OTel repris, mais destination **locale/self-hosted par défaut** et redaction *fail-closed*.

### 6.3 Écarte
- **Runtime JS in-process** (mods Claude Code, `code: true` OpenCode) : hors périmètre Rust ; remplacer par WASM sandboxé si besoin.
- **Marketplace propriétaire / namespacing imposé** : Omega garde un registre de capacités, le « marketplace » n'est qu'un *résolveur*.
- **Télémétrie opérationnelle Anthropic séparée** : non reprise.
- **Poids d'un monorepo TS** : Omega garde le cœur en Rust, la couche TS limitée à l'UI.
- **Tout contenu de system prompt fuité** : interdit (propriétaire) — seuls les *mécanismes* sont analysés.

---

## 7. Sources

**OpenCode** (clone local `/tmp/opencode/re/opencode`, commit `3943b84`, `opencode` 2.0.29, MIT) `[observed]` :
- Contrats : `packages/schema/src/plugin.ts` (l.10 Source, l.24 Features, l.31 State, l.38 Info) · `agent.ts` (l.23, l.30, l.34) · `command.ts` · `skill.ts` (l.27) · `tool.ts` (l.34 ValueSchema, Context, Result) · `provider.ts` · `config/plugin.ts` (l.6-14) · `config/agent.ts` · `config/command.ts` · `config/compaction.ts` · `config/policy.ts` · `model-compatibility.ts` (l.16 CacheFormat).
- Runtime : `packages/core/src/plugin.ts` · `core/src/plugin/host.ts` (capacités étroites) · `core/src/plugin/service.ts`, `supervisor.ts` · `core/src/plugin/agent.ts` (l.77-134, builtins) · `core/src/plugin/plan.ts` · `core/src/plugin/optimize.ts` · `core/src/plugin/skill.ts` · `core/src/tool/plugin/subagent.ts` · `core/src/session/compaction.ts` (l.104-136 template, l.197 keep=15k) · `core/src/session/affinity.ts:7` · `core/src/config/markdown.ts` · `core/src/config/discovery.ts` · `core/src/config/plugin/{agent,command,compatibility,compaction}.ts` · `core/src/agent.ts:13`.
- Prompt caching / Code Mode : `packages/ai/src/cache-policy.ts` (l.17 AUTO, l.24 CAP=4, l.40, l.173, l.188) · `core/src/plugin/provider/prompt-cache-key.ts` · `packages/codemode/src/tool-schema.ts`.
- Specs & API : `specs/v2/README.md` · `specs/v2/tools.md` · `specs/v2/session.md` · `specs/v2/catalog-config-plugin-lifecycle.md` · `specs/v2/event-stream-architecture.md` · `packages/protocol/src/groups/plugin.ts:10-49` · `packages/plugin/src/README.md`, `effect/README.md`, `host.ts`, `source.ts`, `promise/registration.ts` · `package.json` · `LICENSE` · `README.md`.

**Claude Code** (propriétaire — docs publiques) `[stated]` :
- `https://code.claude.com/docs/en/how-claude-code-works` (context window, compaction)
- `https://code.claude.com/docs/en/context-window` (timeline : compaction, tool search, 5 000 tokens/skill, re-lecture fichiers)
- `https://code.claude.com/docs/en/sub-agents` (frontmatter, built-ins Explore/Plan, isolation:worktree, plafond 15 000 tokens)
- `https://code.claude.com/docs/en/skills` (progressive disclosure, 1 536 car., fusion commands)
- `https://code.claude.com/docs/en/plugins/overview` (plugin = dossier, marketplace, scopes)
- `https://code.claude.com/docs/en/hooks` (événements, handlers, décisions)
- `https://code.claude.com/docs/en/mcp#scale-with-mcp-tool-search` (ToolSearch, ENABLE_TOOL_SEARCH, alwaysLoad)
- `https://code.claude.com/docs/en/monitoring-usage` (OTel metrics/logs/traces, gates, spans)
- `https://platform.claude.com/docs/en/build-with-claude/prompt-caching` (cache_control, 4 breakpoints, 20-block lookback, TTL)

**Corpus local** `[observed]` : `docs/omega/research/loucash/classified.json`, `videos_enriched.json`, `videos.json` (ids §3.9).
**Règles** : `docs/omega/rules/RULES-OMEGA.md`, `docs/omega/workstreams/W5-reverse-engineering.md`.

---

## Synthèse finale — le modèle de plugin idéal pour Omega

**Un plugin Omega = un manifeste + des capacités + deux verbes, dans un Scope, avec provenance.**

1. **Manifeste** `{ id, source: builtin|package|local|sdk, capabilities: [context-engine, routing, memory, search, sandbox, ui, telemetry, …], options }` — versionné, schéma validé (JSON Schema), signé. Rien de « en dur » : le noyau lui-même s'enregistre ainsi.
2. **Capabilities, pas marques** (RULES-OMEGA §1.5) : un plugin déclare `routing`/`memory`/… ; le routeur choisit l'implémentation. Les *surfaces* d'exécution (`server`/`tui`/`rpc`) sont des capacités comme les autres.
3. **Deux verbes seulement** : `transform` (contribue à un état **rejouable** → désactivation propre) et `hook` (intercepte une opération **live**), plus `reload`. Éditeurs synchrones ; données async chargées avant.
4. **Scope obligatoire** : toute inscription vit dans un `Scope` ; fermer retire exactement son effet et **révèle la couche précédente**. Superposition native (projet > user > plugin).
5. **Contrat chaud / froid** : un plugin expose (a) un **schéma** (JSON Schema, interop Standard Schema) pour ses entrées/sorties et (b) un **trait Rust** pour son comportement. Frontière Rust↔TS toujours schémiée.
6. **Fail-safe** : un plugin peut passer `failed(cause, ref)` sans arrêter le noyau ; ses avertissements sont tracés, pas silencieux.
7. **Idempotence & coût système** : découverte par empreinte de fichiers, cache des échecs, calculs déterministes délégués à des scripts/outils (jamais à l'agent).
8. **Contexte sous politique** : compaction structurée, rechargement ciblé, prompt caching (≤ 4 breakpoints, affinité parent/fork/self), tool search et progressive disclosure des skills sont des **plugins de contexte** branchés sur le `ContextEngine` — pas du code au fond du noyau.
9. **Observabilité de première classe** : chaque transform/hook émet un événement durable ou éphémère, exportable en OTel, avec **provenance** et redaction fail-closed.

En une phrase : **Omega = le registre scopé et rejouable d'OpenCode + les politiques de contexte de Claude Code, exprimés en traits Rust, schémas versionnés et provenance `[stated]/[observed]/[inferred]`.**
