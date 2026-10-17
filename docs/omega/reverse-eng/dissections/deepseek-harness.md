# Dissection — DeepSeek Harness (`deepseek-ai/deepseek-harness`)

> Clone : `/home/tayk0oo/omega-re/deepseek-harness`, commit `1517a4db` (2026-10-16),
> version `0.2.1-alpha.5` (release `dsh-0.2.1-alpha.5`, encore **developer preview** :
> changements cassants annoncés).
> Preuve machine REA : `docs/omega/research/rea/evidence/intake-deepseek-harness.summary.json`
> (`evidence_id` `ev_a1d6444c…`, provider `rea-javascript-application`, opération
> `analyze_javascript_application`, `confidence: "derived"`). **Elle corrobore uniquement
> les métadonnées** : `id`, `name`, `repo` `deepseek-ai/deepseek-harness`, `stars: 251234`,
> `language: TypeScript`, `license: MIT`, `cloned: true`, `status: analyzed` — plus un
> graphe de `23 593` nœuds / `26 647` arêtes avec un digest `sha256 5d087612…`.
> **Limites de cette preuve, explicitement** : (a) `limitations_count: 11358` pour
> 23 593 nœuds — l'analyseur JS n'a pas résolu près de la moitié du graphe (attendu sur
> un monorepo TypeScript + pnpm + Cordis proxy) ; (b) l'analyse est **statique et
> générique** (`nature: js`) : elle ne dit **rien** du système de plugins, des coutures,
> de la boucle d'agent ni des permissions ; (c) elle n'a produit **aucune trace
> d'exécution**. Toutes les affirmations d'architecture ci-dessous viennent donc d'un RE
> **manuel sur le clone**, chacune ancrée sur un `fichier:ligne` du checkout, et **non**
> de la preuve REA. Aucun `dsh` n'a été lancé : les comportements décrits sont lus dans
> le code et la documentation du dépôt, pas observés à l'exécution.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | DeepSeek AI |
| Licence | MIT (`LICENSE`) |
| Langage | TypeScript (monorepo pnpm : `packages/*`, `apps/*`, `vendor/*`, `native/*`) |
| Repo | https://github.com/deepseek-ai/deepseek-harness |
| Slogan | « DeepSeek Harness: Everything is a Plugin. » (`README.md`) |
| Architecture | Chaque composant est un **plugin Cordis** (framework vendorisé dans `vendor/cordis`) qui contribue services, événements typés et effets **réversibles** au même contexte ; l'application est un **arbre de plugins composé au boot** par couches (bundles d'un profil → patch du profil → patch home → overlays `--patch`) ; la boucle d'agent, le registre d'outils, le log de session et l'adaptateur modèle sont eux-mêmes des plugins, donc remplaçables depuis la configuration (`docs/architecture.md:7-11`). |

**Le point doctrinal qui nous concerne directement** : « There is no privileged core
to patch: you extend dsh by mounting a plugin beside the others, and registrations are
effects that unwind when their plugin unloads. » (`docs/architecture.md:11`). C'est
exactement notre « everything is a plugin », poussé à la racine : *l'agent loop est un
plugin* (`packages/core/agent-loop/src/index.ts:341`, `docs/architecture.md:9`).

## 2. Entrées

- **Binaires/CLI** : `apps/cli` → `@deepseek-ai/dsh`, bin `dsh` (`apps/cli/package.json`). Profils
  livrés : `web`, `headless`, `sdk`, `sdk-minimal`, `acp` (`docs/architecture.md:17`,
  `docs/cli-help.md:21-166`), plus un profil `desktop` réservé à Electron
  (`docs/architecture.md:49-53`). La wheel Python `python/sdk-runtime` embarque le même CLI
  (`docs/architecture.md:47`).
- **Points d'extension** : les **coutures** (seams) — `ctx.llm`, `ctx.tools`, `ctx.sessions`,
  `ctx.agents`, `ctx.subprocess`, `ctx.fs`, `ctx.sandbox`, `ctx.subagents`, `ctx.skills`,
  `ctx.otel`, `ctx.settings`, `ctx.credentials`… liste complète et graphe dans
  `docs/capability-seams.md:610-674` (généré par `scripts/gen-doc-graphs.ts`).
  Les **événements** sont *le* point d'extension : trois domaines, `session/*` (durable),
  `agent/*` (live), capacités `fs/*`, `tools/*`, `telemetry/*` (`docs/architecture.md:67-77`).
- **Config** : `$DSH_HOME/profiles/<name>/` = `package.json` (`dsh.profile.bundles`) +
  `cordis.patch.yml` + `compatibility.json` ; `.env` en couches ; entrées YAML avec
  expressions `!!js` (`docs/architecture.md:15-27`, `packages/boot/app-boot/README.md:45-69`,
  `vendor/loader/src/config/entry.ts:10-22`).
- **Surfaces** : Web UI (`apps/web` + `packages/host/webserver`, `http://127.0.0.1:3080`),
  TUI/terminal, desktop Electron, SDK JSON-RPC, ACP.
- **Outils** : catalogue généré en *bootant* chaque plugin d'outil
  (`docs/tool-catalog.md:7-20`) — preuve que les schémas ne sont pas connaissables statiquement.

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Contexte = dépôt de services (clé + `inject`) | extension | `ctx.<key>` stable, ordre de chargement par dépendances de services | `vendor/cordis/src/context.ts:29-48`, `vendor/cordis/src/service.ts:11-68`, `vendor/cordis/src/registry.ts:195-330` | registry → `Registre` → `omega-registry::Registry` | adopted | M |
| Effets réversibles + cycle de vie Fiber | extension | `ctx.effect()`/`ctx.on()` → disposer, démontage en ordre inverse | `vendor/cordis/src/fiber.ts:79-116`, `:144`, `:184-331` | registry → `Registre` (Scope) | planned | M |
| 5 modes de dispatch typés | extension | emit/waterfall/parallel/serial/bail, contrat public vérifié | `vendor/cordis/src/events.ts:27-32`, `:53-88`, `:183-235` | registry → `Registre` (Hook) | planned | M |
| Loader : `id/name/config/group/disabled/inject` + `!!js` | extension | Entrées adressables par `id`, `disabled` évaluable, include/group | `vendor/loader/src/config/entry.ts:10-22`, `:67-93`, `:105-149`, `vendor/include/src/index.ts:18-20` | registry → `Registre` | planned | S |
| Profils + bundles + couches de patch | extension | Composition au boot, patch par `id` remplaçant tout le config | `docs/architecture.md:13-27`, `packages/boot/app-boot/README.md:45-61`, `packages/bundle/base/cordis.patch.yml:1-30` | registry → `Registre` | planned | M |
| Gestionnaire de plugins | extension | Install/enable/remove, compat de version + exemptions, rollback | `packages/boot/plugin-manager/README.md:31-104`, `:148-162`, `src/operations.ts` | registry → `Registre` | planned | L |
| HMR : config-only vs module | extension | Watch, unité atomique par plugin, file d'attente sérialisée | `vendor/hmr/src/index.ts:97-166`, `:255-398`, `docs/architecture.md:27` | registry → `Registre` | planned | M |
| Enregistrements scopés (`agent.ctx`, realm isolé) | extension | Listeners par agent, `isolate` par label, `group` = realm | `packages/core/scope/src/index.ts:105-201`, `docs/subsystems/scope.md:1-60` | registry → `Registre` (Scope) | planned | M |
| Agent qui monte des plugins | extension | Montage/inspection à chaud (`tool-cordis`, runner dynamique) | `packages/extensions/tool-cordis/src/index.ts:21`, `packages/extensions/cordis-host-runner/src/index.ts:103-141` | registry → `Registre` | planned | M |
| Couches de configuration (env/patches/settings/editor) | configuration | Précédence explicite, `!!js` au boot, éditeur sous verrou | `packages/boot/app-boot/README.md:58-63`, `packages/settings/settings/src/index.ts:223`, `packages/boot/config-editor/src/index.ts:26` | config-source → `ConfigSource` → `omega-config` | adopted | M |
| Couture credentials | configuration | Refs de secrets en config, résolution par opération, vues rédigées | `packages/credentials/credentials/src/index.ts:118-171`, `docs/capability-seams.md:652-654` | credentials → `CredentialProvider` | planned | S |
| La boucle d'agent est un plugin | agents | Driver par défaut = plugin enregistré, entrée requise | `packages/core/agent-loop/src/index.ts:341-410`, `packages/boot/app-boot/README.md:43` | agent-store → `AgentFactory` | planned | L |
| Cycle turn/step (pre-step, request, freeze) | agents | Waterfalls, annulation, gel de requête, « model-visible = logged » | `docs/architecture.md:79-116`, `packages/core/agent/src/runtime-types.ts:320-393` | agent-store → `AgentStore` | planned | M |
| Couture subagent : un service, N providers | agents | Providers nommés, capacité, profondeur, catalog durable | `packages/subagent/subagent/src/index.ts:142-269`, `src/depth.ts:1-23`, `README.md:51-73` | subagent-runner → `SubagentRunner` | planned | L |
| Claude Code / Codex comme providers d'enfants | agents | SDK Claude Code et app-server Codex derrière la couture | `packages/subagent/subagent-claude-code/src/index.ts:30-138`, `src/process.ts:41-46`, `packages/subagent/subagent-codex/src/index.ts:30-122` | subagent-runner → `SubagentRunner` | planned | M |
| Ponts de hooks Claude Code / Codex | agents | `hooks.json` réutilisé tel quel, blocage visible par le modèle | `packages/experimental/hooks-claude-code/README.md:21-40`, `packages/experimental/hooks-codex/README.md:1-30` | hook → `Hook` | planned | M |
| Presets d'agent par session | agents | Composition YAML montée à la demande, révisions retenues | `packages/preset/agent-preset-registry/src/index.ts:52`, `docs/architecture.md:146` | agent-store → `AgentStore` | planned | M |
| Plan mode + `exit_plan_mode` stable | agents | État replié du log, approbation humaine, schéma stable | `packages/plan/plan-mode/src/index.ts:69-175`, `docs/tool-catalog.md:25` | agent-store → `AgentStore` | planned | S |
| Contrat d'outil schéma-first + DSL | outils | `ToolDefinition`, `projectContent`/`finalizeContent`, présentation UI séparée | `packages/core/tools/src/index.ts:224-260`, `:1071-1094`, `src/schema.ts:12-103` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | adopted | M |
| Pipeline d'exécution gardé | outils | pre → guards → execute → project → post → finalize → result | `packages/core/tools/src/index.ts:154-205`, `:1135-1152`, `docs/tool-execution-pipeline.md:6-63` | exec-policy → `ExecPolicy` | planned | M |
| Guards livrés (répétition, timeout) | outils | Guard monotone, scope-aware, qui ne peut que réduire | `packages/guard/repeat-tool-reminder/src/index.ts:35-169`, `packages/guard/timeout-policy/src/index.ts:55`, `docs/subsystems/tools.md:325-333` | exec-policy → `ExecPolicy` | planned | S |
| PTC (`run_code`) — écarté | outils | Sous-appels ré-entrants, runtime JS/Python in-process | `packages/core/tools/src/ptc.ts`, `packages/ptc-runtime/ptc-runtime/src/index.ts:104`, `docs/tool-catalog.md:23` | — | rejected | L |
| Client MCP (stdio + HTTP) | outils | Outils/ressources MCP importés, egress, reconnexion | `packages/mcp/mcp-client/src/index.ts:52-80`, `src/tools.ts`, `docs/capability-seams.md:622` | tool-surface → `ToolSurface` | planned | M |
| Couture sandbox (argv confiné, fail-closed) | securite | `SandboxExecutionPolicy`, `ConfinedArgv`, `SandboxUnavailableError` | `packages/sandbox/sandbox/src/index.ts:40-132`, `docs/subsystems/sandbox.md:9-98` | sandbox-backend → `SandboxBackend` → `omega-sandbox::LocalSandboxBackend` | adopted | M |
| Couture d'approbation (one-shot, deny par défaut) | securite | `approval/request` waterfall, absence = refus, audit | `packages/interaction/user-approval/src/index.ts:100-150`, `docs/subsystems/approval.md:31-160` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | adopted | S |
| Presets de permission | securite | read-only/auto/custom/danger-full-access, `custom` dérivé | `docs/subsystems/permission-presets.md:9-81`, `docs/capability-seams.md:230` | permission-policy → `PermissionPolicy` | planned | S |
| Porte d'observation FS | securite | Gate `fs/write-intent`/`edit-intent`, lecture-avant-édition | `packages/fs/fs-observation-policy/src/index.ts:106`, `docs/tool-execution-pipeline.md:19-63` | exec-policy → `ExecPolicy` | planned | S |
| Coutures shell / subprocess / PTY / SSH | execution | bash, pwsh, PTY persistant, transport distant, jobs | `docs/capability-seams.md:204-227`, `:561-567`, `docs/tool-catalog.md:27` | tool → `ToolExecutor` | planned | M |
| Serveur ACP + enfants ACP | execution | Serveur d'automatisation et consommation d'agents distants | `packages/acp/acp/src/index.ts:75-97`, `src/session.ts`, `docs/architecture.md:43` | service → — | planned | M |
| Log de session append-only + migrations | contexte | `deriveMessages`, générations `vN`, JSONL+zstd, publication exclusive | `docs/architecture.md:118-126`, `packages/session/session-persistence/src/index.ts:137`, `docs/subsystems/persistence.md:327-343` | event-store → `EventStore` → `FileEventStore` | adopted | M |
| Projections de session | contexte | Fold incrémental, `stateOf()`/`snapshot()`, couture obligatoire | `packages/session/session-projection/src/index.ts:48-199`, `docs/subsystems/session-projection.md` | context-engine → `ContextEngine` | planned | M |
| Compaction (couture + pruner + token meter) | contexte | Pruner sans modèle, puis résumé ; mesure révisionnée | `packages/compaction/compaction/src/index.ts:119`, `compaction-basic/src/index.ts:113`, `compaction-tool-result-pruner/src/index.ts:44` | context-engine → `ContextEngine` | planned | M |
| Assemblage du prompt système | contexte | Sections + schémas d'outils, prompt = historique `system/message` | `packages/core/system-prompt/src/index.ts:42-93`, `docs/architecture.md:110` | context-engine → `ContextEngine` | planned | M |
| Providers de skills + disclosure | memoire | Registre de providers de skills, invocation modèle/utilisateur, contenu à la demande | `packages/skill/skill/src/index.ts:126-278`, `docs/capability-seams.md:183` | skill-store → `SkillStore` → `omega-context::SkillIndex` | adopted | M |
| Registre d'adaptateurs LLM | routage | Adapters sur `ctx.llm`, preparation vs exécution, extensions de fil | `packages/llm/llm/src/index.ts:96-354`, `docs/capability-seams.md:628-629` | llm-provider → `Provider` → `omega-providers::OllamaProvider` | adopted | M |
| OTel : canaux indépendants, montage passif | observabilite | `createEventReporter`/`createSessionLogReporter`, rien sans consommateur | `packages/telemetry/otel/src/index.ts:1-30`, `packages/session/session-telemetry/src/index.ts:65-148` | telemetry-sink → `TelemetrySink` → `omega-otel::{Stdout,File}Sink` | adopted | M |
| Taxonomie d'événements + matrice vérifiée | observabilite | 3 domaines, mode déclaré vérifié contre le site de dispatch | `docs/architecture.md:67-77`, `docs/event-producer-consumer.md:10-16` | event-store → `EventStore` | planned | S |
| Serveur web + projection typée des services | ui | Routes plugins, transport authentifié, services → méthodes Remote | `packages/host/webserver/src/index.ts:47-181`, `packages/api/gateway/src/index.ts:209`, `packages/client/modules/src/index.ts:508-603` | service → `omega-server` | planned | L |
| CLI : profils, dump de config, audit de démarrage | ui | Surfaces par profil, `--dump-config`, fail-loud, matrice de pannes | `docs/cli-help.md:21-166`, `packages/boot/app-boot/README.md:36-43`, `:98-111` | service → — | planned | M |
| Jobs, planification, webhooks, workflow | automatisation | Travail hors du tour, chacun un service composé | `packages/jobs/jobs/src/index.ts:88`, `packages/schedule/schedule/src/index.ts:99`, `packages/experimental/webhook/src/index.ts:58` | service → — | planned | M |
| Agent Teams (expérimental) | agents | Roster durable + inbox directe sur sous-agents recontinuables | `docs/subsystems/agent-team.md`, `docs/architecture.md:136`, `docs/capability-seams.md:251-253` | agent-store → `AgentStore` | planned | L |
| Spill store + attachements durables | contexte | Résultats volumineux et binaires sortis du contexte, committés avant le log | `packages/spill/spill/src/index.ts:45`, `packages/attachment/attachment/src/index.ts:53`, `docs/capability-seams.md:625-626` | service → `omega-context` | planned | M |

**Total : 42 fonctionnalités** — 9 `adopted`, 32 `planned`, 1 `rejected` ; 36 plugins,
5 services, 0 `native`.

## 4. Traduction en plugins

### 4.1 Le socle d'extension (ce qui nous manque le plus)

| Cordis | Capacité Omega | Port | Adaptateur |
|---|---|---|---|
| Contexte = dépôt de services + `inject` | `registry` | `Registre` | `omega-registry::Registry` (existant → **adopted**) |
| Effets réversibles (`ctx.effect` / `Fiber`) | `registry` | `Registre` (Scope/effets) | `omega-plugin-api` (à écrire) |
| 5 modes de dispatch typés | `registry` | `Registre` (Hook) | `omega-plugin-api` (à écrire) |
| Loader `id/name/config/group/disabled/inject` | `registry` | `Registre` (manifeste) | `omega-registry::Manifest` + `omega-config` |
| Profils + bundles + couches de patch | `registry` | `Registre` (composition) | `omega-config` (couches) + `omega-registry` |
| Gestionnaire de plugins + HMR + presets + scope | `registry` | `Registre` | `omega-registry` (scope/versionnement/quarantaine) |

**Lecture** : notre `omega-registry` fournit déjà les deux régimes (`Slot` / `Ensemble`),
la priorité et la **quarantaine** — c'est-à-dire l'équivalent du « failed non fatal ».
Ce qui manque est la couche au-dessus : les **effets réversibles** et la **composition
par couches patchables par `id`**. C'est `omega-plugin-api`, aujourd'hui quasi vide.

### 4.2 Le noyau d'exécution

| DeepSeek Harness | Capacité | Port | Adaptateur |
|---|---|---|---|
| Contrat d'outil + DSL de schémas | `tool-surface` | `ToolSurface` | `omega-context::ToolSurfaceIndex` (**adopted**) |
| Pipeline pre → guards → execute → post → result | `exec-policy` | `ExecPolicy` + `ToolExecutor` | `omega-sandbox::RuleBasedExecPolicy` (partiel) |
| Guards monotones scope-aware | `exec-policy` | `ExecPolicy` | `omega-sandbox::RuleBasedExecPolicy` |
| Couture sandbox + argv confiné | `sandbox-backend` | `SandboxBackend` | `omega-sandbox::LocalSandboxBackend` (**adopted**) |
| Couture d'approbation (deny par défaut) | `permission-policy` | `PermissionPolicy` | `omega-sandbox::ApprovalQueue` (**adopted**) |
| Log de session + migrations | `event-store` | `EventStore` | `omega-app::FileEventStore` + `omega-otel::TraceReplayer` (**adopted**) |
| OTel canaux indépendants | `telemetry-sink` | `TelemetrySink` | `omega-otel::{Stdout,File}Sink` (**adopted**) |
| Registre d'adaptateurs LLM | `llm-provider` | `Provider` | `omega-providers::OllamaProvider` (**adopted**) |
| Providers de skills | `skill-store` | `SkillStore` | `omega-context::SkillIndex` (**adopted**) |
| Couches de configuration | `config-source` | `ConfigSource` | `omega-config` (**adopted**) |
| Sous-agents multi-providers | `subagent-runner` | `SubagentRunner` | `omega-ports::SubagentRunner` |
| La boucle comme plugin | `agent-store` | `AgentFactory` / `AgentStore` | `omega-core` |

### 4.3 Mode d'intégration

- **36 plugins** : tout ce qui est une couture ou un domaine composable (extension,
  agents, outils, sécurité, contexte, mémoire, routage, observabilité) devient une
  entrée de registre avec son `id`, son `Registre`/`Slot` et sa priorité.
- **5 services** : ce qui est un *hôte* et non une capacité — CLI/profils, serveur web +
  projection typée, ACP, jobs/schedule/webhook/workflow, spill/attachements.
  Ils restent derrière une frontière (process, HTTP) et ne rentrent pas dans le noyau.
- **0 `native`** : rien ici n'est un invariant du noyau. C'est précisément la leçon :
  DeepSeek Harness n'a *rien* mis en dur, y compris la boucle d'agent.

## 5. Écarté

| Écarté | Pourquoi |
|---|---|
| **PTC / `run_code`** (`packages/core/tools/src/ptc.ts`, `packages/tool-catalog.md:23`) | Le mode PTC n'émet plus qu'un transport réservé `run_code` et exige un runtime JS/Python **in-process** (`ptc-runtime-node`, `experimental-ptc-runtime-python`) plus un pont de types. Le noyau Rust n'a pas de runtime JS embarqué ; un équivalent WASM sandboxé serait requis. On garde les leçons de structure (un seul transport au schéma stable, sous-appels qui **ré-entrent** le pipeline gardé, budget `maxParallelSubCalls`, schéma figé à la construction), pas le runtime. |
| **Surface Cordis côté client navigateur** (`packages/client/modules`, `ui-cordis`) | Le graphe de modules client + boot injections suppose un document HTML et un bundler ESM. Notre cockpit React n'a pas besoin de rejouer le modèle d'effets Cordis dans le navigateur : il consomme `/api/*`. Seule la *projection typée* des services (typert → Remote) est reprise, comme contrat serveur. |
| **Stack pnpm / installation de bundles par package manager** | Le gestionnaire de plugins écrit dans `package.json`, `pnpm-lock.yaml` et `pnpm-workspace.yaml`, avec `pnpm view`/`pnpm add`. Le **protocole** (inspecter avant d'installer, cascade de registres, snapshot/restore, exemptions de version) est repris ; l'implémentation pnpm ne l'est pas (nous n'avons pas d'écosystème npm à piloter). |
| **Runtime d'inspection cross-realm** (`packages/experimental/inspector`) | Suppose plusieurs `realm`/`vm` et le proxy Cordis vivant. Napkin : nous préférons un `--dump-config` équivalent statique (déjà le cas avec notre catalogue embarqué). |

## 6. Sources

- Clone : `/home/tayk0oo/omega-re/deepseek-harness` `@1517a4db` (`0.2.1-alpha.5`, 2026-10-16).
- Docs dépôt : `README.md`, `docs/architecture.md`, `docs/cordis-primer.md`,
  `docs/capability-seams.md` (généré), `docs/tool-execution-pipeline.md` (généré),
  `docs/event-producer-consumer.md` (généré), `docs/tool-catalog.md` (généré),
  `docs/cli-help.md`, `docs/subsystems/*.md`.
- Code : `vendor/cordis/src/{context,service,fiber,events,registry}.ts`,
  `vendor/{loader,include,hmr}/src/`, `packages/boot/{app-boot,plugin-manager,config-editor}/`,
  `packages/core/{tools,agent,agent-loop,scope,system-prompt}/`,
  `packages/subagent/subagent{,-claude-code,-codex}/`, `packages/sandbox/sandbox/`,
  `packages/guard/*/`, `packages/session/{session-persistence,session-projection,session-telemetry}/`,
  `packages/compaction/*/`, `packages/mcp/mcp-client/`, `packages/acp/acp/`,
  `packages/telemetry/otel/`, `packages/extensions/{tool-cordis,cordis-host-runner}/`.
- Preuve machine REA : `docs/omega/research/rea/evidence/intake-deepseek-harness.summary.json`
  (`ev_a1d6444c…`, `confidence: derived`, `23 593` nœuds / `26 647` arêtes,
  `limitations_count: 11358`). **Corrobore les métadonnées seulement** (repo, étoiles,
  langage, licence, clone) ; ne corrobore **aucune** des 42 fonctionnalités listées en §3,
  faute de résolution du graphe (≈48 % de limitations) et de toute trace d'exécution.
- Preuves machines voisines (même lot, autres cibles, non utilisées ici) :
  `intake-dsh-desktop.summary.json` (`anywhere-labs/dsh-desktop`, 31 119 ★, MIT) et
  `intake-deep-agents.summary.json` (`langchain-ai/deepagents`, 30 347 ★, MIT).

---

## 7. Ce que notre noyau doit copier

Par ordre de priorité décroissante. Chaque point est une **décision de noyau**, pas une
feature cosmétique : ce sont les endroits où notre doctrine « everything is a plugin »
est aujourd'hui plus faible que la leur.

### 1. Les enregistrements doivent être des **effets réversibles**, pas des insertions

Chez eux, *toute* contribution (section de prompt, schéma d'outil, adaptateur, provider,
listener) passe par `ctx.effect()` ou `ctx.on()` et retourne un **disposer** ; l'unload
défait exactement ce que le plugin a installé, en ordre inverse, sans `undo` manuel
(`vendor/cordis/src/fiber.ts:79-116`, `:405-449`, `docs/cordis-primer.md:13`). Notre
`omega-registry` sait `register`/`unregister`/`quarantine`, mais l'inscription *composée*
n'est pas encore un effet révocable : c'est ce qui empêche un vrai hot-swap.
**À copier : `Effect::dispose()` retourné par chaque inscription, et démontage en ordre
inverse.**

### 2. Les couches de composition doivent être **patchables par `id`**

`EntryOptions = {id, name, config, group, disabled, inject}`
(`vendor/loader/src/config/entry.ts:10-22`) ; un patch cible une ligne par `id` et
**remplace tout son `config`**, les couches s'appliquant dans un ordre explicite
(bundles → patch profil → patch home → overlays, `docs/architecture.md:25`,
`packages/bundle/base/cordis.patch.yml:1-30`). Notre `omega-config` a des couches à
précédence, mais pas d'**identité de ligne stable et patchable**. Sans `id`, on ne peut
pas dire « cette couche-ci a changé ce champ-là » — et donc pas tracer ni repropager.
**À copier : `id` stable par entrée + patch par `id` remplaçant le config entier.**
Corollaire indispensable : `dsh --profile web --dump-config` imprime la liste composée
groupée **par fichier source et par couche ayant posé chaque ligne**
(`packages/boot/app-boot/README.md:71-73`) — c'est cet artefact qui rend la composition
auditable, et sans lui on ne peut pas prouver la précédence.

### 3. Le rechargement doit être une **unité atomique par plugin**, sérialisée avec les écritures

Le watcher classe les fichiers (externes → reload complet ; sinon dispose + réimport),
chaque plugin résolu en file URL est une unité atomique, avec debounce, et le même file
d'attente partagé avec les écritures du gestionnaire de plugins
(`vendor/hmr/src/index.ts:255-398`, `packages/boot/plugin-manager/README.md:98`).
C'est ce qui rend le hot-reload sûr. **À copier : verrou/queue unique partagée entre
rechargement et mutation de configuration, unité = plugin.**

### 4. Un **guard monotone** qui ne peut QUE réduire la permission

`ToolGuard` n'a délibérément **aucun** résultat « allow » : `undefined` préserve la
décision du waterfall et une raison retournée ne peut que réduire, donc un listener
tardif ne peut pas annuler un refus (`docs/subsystems/tools.md:325-333`, `:542-551`).
Notre `RuleBasedExecPolicy` refuse, mais rien n'interdit structurellement un plugin
d'annuler un refus d'un autre. **À copier : type de retour sans variante `Allow`, et
évaluation de la couche la plus lointaine à la plus proche.**

### 5. L'approbation doit résoudre les `ask` **avant** les guards, et l'absence de canal = refus

`ctx.approval` est un waterfall résolu avant les guards monotones, et un canal absent,
injoignable, refusé, annulé ou en timeout donne **deny** (`docs/tool-execution-pipeline.md:16-42`,
`docs/subsystems/approval.md:31-160`). Une approbation autorise **une fois** et ne change
pas le mode de permission de la session. Notre `ApprovalQueue` va dans ce sens ; il faut
figer l'**ordre** (ask avant guards) et le **défaut fermé** comme invariants testés.

### 6. Un registre de types runtime pour projeter les services en API typée

`ctx.typert` reçoit des contributions zod, et `api-gateway` associe des descripteurs
Remote **générés** à des services Cordis vivants, exposés par un carrier RPC partagé avec
contrôle d'accès par rôle (`packages/api/gateway/src/index.ts:209`,
`docs/capability-seams.md:645-647`). Chaque nouveau plugin devient une méthode d'API sans
écrire de handler. **À copier : un registre de schémas runtime qui dérive `/api/*` depuis
les capacités, plutôt qu'un routeur écrit à la main.**

### 7. Une **matrice producteur/consommateur d'événements** générée, avec le mode vérifié

Trois domaines (durable `session/*`, live `agent/*`, capacité `fs/*`, `tools/*`, `telemetry/*`),
chaque événement déclarant son mode de dispatch, et le générateur **vérifie les
déclarations `@mode` contre les sites de dispatch réels** (`docs/architecture.md:67-77`,
`docs/event-producer-consumer.md:10-16`, `docs/cordis-primer.md:27`). C'est de la
traçabilité obtenue par outil, pas par discipline. **À copier : notre carnet d'événements
comme artefact *généré* et vérifié, comme on le fait déjà pour le catalogue de modèles.**

### 8. Un **fail-closed explicite** à chaque couture

`SandboxUnavailableError` est *fail-closed* : pas de backend applicable = erreur, jamais
une exécution non confinée (`packages/sandbox/sandbox/src/index.ts:132`,
`docs/subsystems/sandbox.md:154-163`). Idem pour une couture de projection obligatoire :
un lecteur doit soit exiger le service à l'activation, soit **échouer explicitement**,
sans défaut silencieux (`docs/architecture.md:126`). **À copier : règle « une couture
absente est une erreur typée, jamais une valeur par défaut », appliquée à
`omega-ports` et vérifiée par test.**
