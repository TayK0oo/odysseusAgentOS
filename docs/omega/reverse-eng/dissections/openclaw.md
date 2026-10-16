# Dissection — OpenClaw (`openclaw`)

> Provenance : `[observed]` = lu dans le clone `--depth 1` du 2026-10-16 (version `2026.10.4`) ;
> `[stated]` = déclaré par le projet ; `[inferred]` = synthèse pour Omega.
> Fiche antérieure : `docs/omega/reverse-eng/RE-OPENCLAW-HERMES.md` (ne pas dupliquer, compléter).
> Données : `omega/crates/omega-harness/data/apps/openclaw.json` (17 fonctionnalités).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Peter Steinberger / communauté OpenClaw `[stated]` (`package.json`) |
| Licence | MIT (`LICENSE`) `[observed]` |
| Langage | TypeScript (monorepo pnpm) + 2 crates Rust `[observed]` |
| Repo | https://github.com/openclaw/openclaw |
| Architecture | Gateway multi-canal adossée à un runner d'agent embarqué ; toute capacité exclusive (mémoire, moteur de contexte) est fournie par un plugin sélectionné via **slots** ; comportement décrit par **manifeste déclaré** (policy-par-code) et intercepté par **hooks typés**. `[inferred]` |

## 2. Entrées

| Entrée | Chemin `[observed]` |
|---|---|
| CLI | `openclaw.mjs`, `src/entry.ts`, `src/index.ts` |
| Gateway multi-canal | `src/gateway/` (RPC HTTP) + `src/channels/` (adaptateurs) |
| Node host (sidecar Rust) | `crates/openclaw-node-host/`, client `crates/openclaw-gateway-client/` |
| Control UI web + TUI | `ui/src/`, `src/tui/commands.ts`, `src/web/` |
| Points d'extension | `definePluginEntry` (`src/plugin-sdk/plugin-entry.ts:244`), ~73 `register*` (`src/plugins/plugin-api.types.ts`), `openclaw.plugin.json` par plugin/extension |
| Fichiers de config | `OpenClawConfig` typée (`src/config/`), `PluginSlotsConfig` (`src/config/types.plugins.ts:10`), `extensions/*/openclaw.plugin.json` |

Inventaire brut du clone : 180 extensions (`extensions/`), 24 packages internes (`packages/` : `plugin-sdk`, `memory-host-sdk`, `model-catalog-core`, `net-policy`, `plugin-package-contract`, `worker-runtime`, …), `src/` (~80 zones : `agents/`, `context-engine/`, `plugins/`, `cron/`, `sessions/`, `security/`, `skills/`, `trajectory/`, `pairing/`, `mcp/`, `routing/`, `model-catalog/`, `worker/`, `node-host/`).

## 3. Inventaire fonctionnel

| # | Fonctionnalité | Domaine | Ce que ça fait | Où (`file:line` `[observed]`) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|---|
| 1 | Moteur de contexte enfichable + `ownsCompaction` | contexte | Port unique ingest/assemble/compact/afterTurn ; l'implémentation peut posséder la compaction | `src/context-engine/types.ts:191,355-542`, `registry-contract.ts:27-62`, `context-engine-lifecycle.ts:111-442` | context-engine → `ContextEngine` → `omega-context::DefaultContextEngine` | planned | L |
| 2 | Slots d'exclusivité | extension | Une seule implémentation active par capacité, résolue par config | `src/plugins/slots.ts:12-107`, `plugin-kind.types.ts:1-2` | registry → `Registre` → `omega-registry` | **adopted** | S |
| 3 | Manifeste déclaratif + policy-par-code | extension | Comportement décrit par schéma, lu sans exécuter le code | `src/plugins/manifest-types.ts:429,503-535`, `plugin-entry.ts:244-260` | plugin-manifest → `PluginManifest` → `plugins/plugin-manifest` | planned | M |
| 4 | Hooks typés Gate/Modify/Observe/Claim | extension | Cycle de vie agent/outils/session intercepte, avec budgets 15/30 s | `docs/plugins/hooks/reference.md:93-188`, `plugin-api.types.ts:239-257` | hook-bus → `HookBus` → `plugins/hook-bus` | planned | M |
| 5 | Télémétrie prompt-cache | contexte | Diagnostic typé des ruptures de cache donné à l'engine | `src/context-engine/types.ts:242-286` | context-engine → `ContextEngine` → `omega-context::PromptCacheTelemetry` | planned | M |
| 6 | Contrat mémoire + memory-core | memoire | Mémoire = plugin (recherche autorisée, origines classifiées, rappel déterministe) | `registry-contribution-types.ts:271-376`, `extensions/memory-core/search-api.ts` | memory-store → `MemoryStore` → `plugins/memory-fts` | planned | M |
| 7 | Registre ~73 `register*` + exécution d'outils | outils | Toutes les capacités s'enregistrent sur une API unique ; le runner dispatche | `plugin-api.types.ts:239-500`, `attempt-client-tools.ts`, `agent-tools.ts` | tool-surface → `ToolSurface` → `omega-context::ToolSurfaceIndex` | planned | M |
| 8 | Gateway multi-canal | extension | 180 extensions ; canaux + garde-fous DM + accès gateway enregistrables | `plugin-api.types.ts:257-283`, `channels/direct-dm-guard-policy.ts`, `channels/allow-from.ts` | channel → `Channel` → `plugins/channel-*` | planned | M |
| 9 | Appairage + approbations | securite | Join-code 16 o + allowlist ; file d'approbation avant outil sensible | `pairing/join-code.ts:1-9`, `agent-runtime-approval-authority.ts:12`, hooks ref `:147` | permission-policy → `PermissionPolicy` → `omega-sandbox::ApprovalQueue` | **adopted** | M |
| 10 | Routage + catalogue tarifé | routage | Routes de session + catalogue versionné + pricing + fallback | `routing/resolve-route.ts:85-204`, `model-catalog/pricing.ts:201-323` | router-policy → `ScoringPolicy/Selector/Fallback` → `omega-router::*` | **adopted** | M |
| 11 | Cron lié aux sessions | automatisation | Jobs déclaratifs anti-doublons, liés au cycle de vie des sessions | `cron/active-jobs.ts:1-192`, `cron/public-job.ts:3`, `job-session-bindings.ts:20-73` | scheduler-job → `Scheduler` → `plugins/scheduler-cron` | planned | M |
| 12 | Config typée à couches | configuration | Config validée, migrée, sauvegardée (canaux, slots, limites) | `config/types.plugins.ts:10`, `plugins/slots.ts:12-22` | config-source → `ConfigSource` → `omega-config` | **adopted** | S |
| 13 | Trajectoires + export | observabilite | Événements persistés (sqlite/mémoire), bundles rejouables, diagnostics | `trajectory/export.ts:951-968`, `runtime-store.contract.ts:3-33` | event-store → `EventStore` → `omega-app::FileEventStore` | **adopted** | M |
| 14 | Skills à invocation contrôlée | contexte | Découverte + filtres + chargement paresseux + politique + origine | `skills/types.ts:21-93`, `skills/discovery/command-specs.ts` | skill-store → `SkillStore` → `omega-context::SkillIndex` | **adopted** | S |
| 15 | Control UI + TUI + RPC | ui | Widgets/actions/routes contribués par plugins, rendus sans couplage | `plugin-api.types.ts:116-167`, `manifest-types.ts:520-535`, `tui/commands.ts:1` | control-ui → `ControlUi` → `plugins/control-ui` | planned | M |
| 16 | Sous-agents + node host Rust | agents | Spawn isolated/fork + rollback ; worker embarqué ou sidecar Rust | `context-engine/types.ts:206-238,522-536`, `crates/openclaw-node-host/src/{sidecar_protocol,lifecycle}.rs` | subagent-runner → `SubagentRunner` → `plugins/subagent-isolated` | planned | M |
| 17 | Audit sécu extensible | securite | Collecteurs d'audit enregistrables ; scans exec/fs/config/secrets | `plugin-entry.ts:244-260`, `security/audit-exec-surface.*`, `packages/net-policy/` | exec-policy → `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy` | planned | M |

## 4. Traduction en plugins

Capacité → port → adaptateur ; mode `plugin` sauf trajectoires/audit (`native`, cœur Rust) :

- `registry → Registre → omega-registry` (slots, **adopted**, S) — la pièce la plus rentable : ~150 lignes TS (`slots.ts:12-107`) deviennent une map `kind → slot` + résolution par config.
- `permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue` (**adopted**, M) — `before_tool_call` + autorité d'approbation mappent directement sur la file existante.
- `router-policy → ScoringPolicy/Selector/FallbackPolicy → omega-router::*` (**adopted**, M) — `resolve-route.ts` + `pricing.ts` + hook `before_model_resolve` alimentent classifieur/sélecteur/fallback existants.
- `config-source → ConfigSource → omega-config` (**adopted**, S) — couches + précédence déjà là ; ajouter les clés `slots.*`.
- `event-store → EventStore → omega-app::FileEventStore` (**adopted**, M) — contrat `runtime-store` (sqlite/mémoire + rétention + export bundle) ≈ store existant + `TraceReplayer`.
- `skill-store → SkillStore → omega-context::SkillIndex` (**adopted**, S) — métadonnées + découverte + politique d'invocation.
- `context-engine → ContextEngine → omega-context::DefaultContextEngine` (planned, L) — le gros morceau : cycle de vie complet + `ownsCompaction` + `acceptedHostParams` + projection `per_turn|thread_bootstrap`.
- `plugin-manifest / hook-bus / channel / scheduler-job / control-ui / subagent-runner / memory-store / tool-surface` (planned, M) — nouveaux plugins, contrats copiés des signatures citées (MIT, interop).

## 5. Écarté

- **Les 180 extensions prises une à une** : on reprend le *mécanisme* (registre + manifeste + slots), pas chaque adaptateur Telegram/Discord/provider — coût L dispersé, valeur locale. Seuls `memory-core` (contrat mémoire) et le catalogue de modèles sont repris comme gabarits.
- **Le worker embarqué TS et le node host TS** (`src/worker/`, `src/node-host/`) : Omega est Rust-first ; seul le protocole sidecar Rust (`duplex/reconnect/handshake`) est réutilisé comme gabarit.
- **Boucle d'auto-amélioration / dreaming** (`memory-core-dreaming-cron`) : coût LLM récurrent, conflit avec « le coût va au système » — même verdict que F-H-7 d'Hermes, désactivé par défaut si repris.
- **UI React telle quelle** : on reprend les *descripteurs* (controlUi/dashboard validés contre le runtime), pas les composants.

## 6. Sources

- Repo : https://github.com/openclaw/openclaw (MIT, tag `2026.10.4`), clone `--depth 1` dans `/home/tayk0oo/omega-re/openclaw`.
- Fiche antérieure : `docs/omega/reverse-eng/RE-OPENCLAW-HERMES.md` (§3.1 ports & contrats cités, §4 F-OC-1…F-OC-8).
- Chemins vérifiés le 2026-10-16 : `src/context-engine/types.ts`, `src/plugins/{slots,slot-selection,plugin-kind.types,manifest-types,plugin-api}.types.ts`, `src/plugin-sdk/plugin-entry.ts`, `src/{channels,cron,sessions,security,config,routing,model-catalog,skills,trajectory,pairing,tui,web}/`, `packages/{plugin-sdk,memory-host-sdk,model-catalog-core,net-policy,plugin-package-contract}/`, `crates/{openclaw-gateway-client,openclaw-node-host}/`, `extensions/`, `docs/plugins/{manifest.md,hooks/reference.md}`.
