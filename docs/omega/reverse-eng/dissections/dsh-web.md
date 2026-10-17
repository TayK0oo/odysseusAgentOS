# DSH Web — Dissection de harnais

> **Fiche** : éditeur = communauté DSH (zhu10900993659) · licence = Apache-2.0 · langage = TypeScript · repo = https://github.com/zhu10900993659/dsh-web · 8 835 étoiles.
> **Nature** : écosystème web de plugins pour DeepSeek Harness (DSH) — « 万物皆插件, 通过创意工坊分发 » (tout est plugin, distribué via un atelier créatif).
> **Preuve REA** : `docs/omega/research/rea/evidence/intake-dsh-web.summary.json` (provider rea-javascript-application, 7 515 nodes / 12 085 edges, 3 240 limitations, confidence=derived). Limites : le graphe REA ne capture pas la sémantique Cordis (loader, patch rows, services) ; les refs ci-dessous sont des lectures directes du clone.

---

## 1. Architecture

Monorepo **pnpm workspace** (`packages/*`, `satellites/*`, `shared/*`) construit sur le framework **Cordis** (`@deepseek-ai/cordis`, framework de plugins typé, équivalent de Koishi pour DSH). Chaque plugin est un package npm avec :

- `cordis.patch.yml` (insert rows : lignes entrées dans la composition du host).
- `src/index.ts` (apply host) + `src/client/` (panneau browser React).
- `dsh.bundle.patch.yml` (patch agrégat pour les enfants).

Le **profile** (`$DSH_HOME/profiles/<name>/`) porte :

- `package.json` (dependencies npm, `dsh.profile.bundles`).
- `cordis.patch.yml` (user layer : bare rows `{ id, name, disabled }` qui surchargent les insert rows).

Le **loader DSH** assemble la composition : patch rows (bundle layer) + user layer = lignes effectives. Chaque ligne a un `id` (loader entry id), un `nom`, et un état `enabled`/`disabled`. Les **agrégats** (dsh-web-all) replient toute la famille dans un seul package npm.

Le **host** (Node.js) expose des routes HTTP **loopback-only** et le **browser** (React) injecte des panneaux dans les slots DSH (`dsh.client.ui.slots`).

---

## 2. Entrées

| Entrée | Rôle |
|---|---|
| `@linxin666/dsh-web-all` | npm aggregate (host + browser) |
| `dsh-plugin-manager` | registre de plugins (install/update/uninstall/enable/disable) |
| `dsh-market` | atelier créatif (skins, pets, presets, community plugins) |
| `dsh-remote-bridge` | pont téléphone ↔ host (WebSocket `/ios/v1`) |
| `dsh-skill-explorer` | explorateur de skills (frontmatter SKILL.md) |
| `dsh-task-board`, `dsh-ssh`, `dsh-usage`, etc. | plugins spécialisés |
| `dsh plugin --profile <name> add/remove` | CLI officiel (single writer) |

---

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| `dsh-plugin-registry-gateway` | extension | Registre de plugins via gateway HTTP loopback (list/install/update/remove/enable) proxyfiant le CLI officiel | `packages/dsh-plugin-manager/src/core/service.ts:1-62`, `packages/dsh-plugin-manager/src/host/routes.ts:207-651` | `omega-registry::Registry` | adopted | M |
| `dsh-plugin-manager-loopback-fence` | securite | Barrière de confiance loopback (socket + Host header + same-origin markers) | `packages/dsh-plugin-manager/src/host/loopback.ts:1-63`, `packages/dsh-remote-bridge/src/core/loopback.ts:1-60` | `omega-sandbox::RuleBasedExecPolicy` | adopted | S |
| `dsh-plugin-manager-spec-validation` | securite | Validation stricte des specs (rejet métachar shell, paths relatifs, taille caps) | `packages/dsh-plugin-manager/src/host/gateway.ts:39-56`, `packages/dsh-market/src/core/installer.ts:117-152` | `omega-sandbox::RuleBasedExecPolicy` | adopted | S |
| `dsh-market-asset-installer` | extension | Installation d'assets (skins/pets/presets) depuis dsh-market.com vers DSH_HOME avec provenance sha256 | `packages/dsh-market/src/core/installer.ts:1-351`, `packages/dsh-market/src/routes.ts:1-135` | `omega-context::SkillIndex` | planned | M |
| `dsh-market-provenance` | observabilite | Manifest de provenance (sha256 de chaque fichier) pour distinguer officiel vs manuel | `packages/dsh-market/src/core/installer.ts:39-56` | `omega-otel::Provenance` | planned | S |
| `dsh-plugin-manager-compatibility-gate` | configuration | Gate de compatibilité : chaque plugin déclare un dsh.engines.dsh minimum ; host vérifie >=X.Y.Z | `packages/dsh-plugin-manager/src/core/version.ts:1-142`, `packages/dsh-plugin-manager/src/host/routes.ts:130-179` | `omega-config` | adopted | S |
| `dsh-plugin-manager-bundle-guard` | extension | Garde anti-double-mount : détecte et retire les bundles déjà montés via patch row | `packages/dsh-plugin-manager/src/host/bundle-guard.ts:1-113` | `omega-registry::Registry` | planned | M |
| `dsh-plugin-manager-conflict-ledger` | observabilite | Ledger de conflits : capture avant/après de chaque mutation (rows disabled, bundles added) | `packages/dsh-plugin-manager/src/core/patch-diff.ts:1-77` | `omega-app::FileEventStore` | planned | S |
| `dsh-remote-bridge-pairing` | securite | Pairing de terminaux : bootstrap token (30s) + nonce (60s) signé ES256 (P-256) | `packages/dsh-remote-bridge/src/core/pairing.ts:1-113`, `packages/dsh-remote-bridge/src/core/crypto.ts:1-113` | `omega-sandbox::ApprovalQueue` | planned | M |
| `dsh-remote-bridge-approval-claim` | securite | Claim des approval/request tant qu'un device paired est connecté, delegue au host sinon (fail-closed) | `packages/dsh-remote-bridge/src/core/approvals.ts:1-142`, `packages/dsh-remote-bridge/src/core/hub.ts:97-117` | `omega-sandbox::ApprovalQueue` | adopted | M |
| `dsh-remote-bridge-protocol-v1` | agents | Protocole v1 WebSocket (/ios/v1) : list/titles/snapshot/history/prompt/interrupt/approval | `packages/dsh-remote-bridge/src/core/connection.ts:1-797`, `packages/dsh-remote-bridge/src/core/protocol.ts:1-89` | `plugins/subagent-isolated` | planned | L |
| `dsh-skill-explorer-frontmatter` | contexte | Parsing frontmatter SKILL.md (zero dependance) : name/description/whenToUse/input.hint/recordInput | `packages/dsh-skill-explorer/src/frontmatter.ts:1-153` | `omega-context::SkillIndex` | adopted | S |
| `dsh-web-all-aggregate` | extension | Paquet agrégatif repliant 16+ plugins dans un seul npm avec lignes web-ui-* namespaces | `packages/dsh-web-all/aggregate.yml:1-100` | `omega-registry::Registry` | planned | M |
| `dsh-plugin-manager-mount-once` | extension | Guard d'instance unique : un plugin npm ne monte qu'une fois ; mount refusé = queued | `packages/dsh-plugin-manager/src/mount-once.ts:1-142` | `omega-registry::Registry` | planned | S |
| `dsh-plugin-manager-restart` | execution | Redémarrage du host DSH : shell (Electron) / relaunch (terminal) / manual (supervisé) | `packages/dsh-plugin-manager/src/host/restart.ts:1-238` | — | planned | M |
| `dsh-plugin-manager-detached-work` | execution | Escape hatch AsyncLocalStorage : écriture dans cordis.patch.yml hors contexte HMR | `packages/dsh-plugin-manager/src/host/detached-work.ts:1-58` | — | rejected | S |
| `dsh-plugin-manager-console-output` | observabilite | Capture de output console avec budget byte et decode UTF-8 strict + fallback GBK | `packages/dsh-plugin-manager/src/host/console-output.ts:1-97` | — | rejected | S |
| `dsh-market-client-categories` | ui | Navigation du market client : catégories, filtres, tri, cards avec boutons install/update | `packages/dsh-market/src/client/categories.ts:1-50`, `packages/dsh-market/src/client/MarketCard.tsx:1-50` | — | rejected | M |

---

## 4. Traduction en plugins

### 4.1 Registre de plugins (dsh-plugin-registry-gateway)

**Capacité → Port → Adaptateur :** `registry` → `Registre` → `omega-registry::Registry (Slot/Ensemble + quarantaine)`

- **Inventaire** : le snapshot se lit depuis `profile/package.json` (dependencies + bundles) + `node_modules/*/cordis.patch.yml` (insert rows) + `cordis.patch.yml` (user layer). Le gateway HTTP (`/api/plugin-manager/*`) expose ce snapshot en JSON.
- **Installation** : le CLI officiel `dsh plugin --profile <name> add|remove` est le **single writer**. Le gateway proxyfie ce CLI avec une job table, un mutation lock, et des captures avant/après.
- **Mise à jour** : résolution npm latest (`registry.npmjs.org/<name>/latest`), gate de compatibilité (`dsh.engines.dsh`), puis CLI.
- **Activation/désactivation** : écriture de bare rows `{ id, name, disabled }` dans `cordis.patch.yml` via round-trip YAML (préserve commentaires et rows non liés).
- **Redémarrage** : `planRestart` → shell (Electron) / relaunch (terminal interactif, detached helper) / manual (supervisé).
- **Mode d'intégration** : plugin (le gateway est un plugin Cordis comme les autres, avec `inject = ['webServer']`).

### 4.2 Barrière de confiance loopback (dsh-plugin-manager-loopback-fence)

**Capacité → Port → Adaptateur :** `exec-policy` → `ExecPolicy` → `omega-sandbox::RuleBasedExecPolicy`

- **Règle** : socket address (127/8, ::1, ::ffff:127/8) + Host header (localhost, [::1], 127/8) + same-origin markers (sec-fetch-site, Origin).
- **Variante stricte** (remote-bridge) : 127.0.0.1 exact + Host exact (car un tunnel cloudflared arrive aussi en loopback).
- **Mode d'intégration** : plugin (toutes les routes host passent par ce fence).

### 4.3 Gate de compatibilité (dsh-plugin-manager-compatibility-gate)

**Capacité → Port → Adaptateur :** `config-source` → `ConfigSource` → `omega-config`

- **Règle** : `dsh.engines.dsh` (ou `engines.dsh`, ou `peerDependencies['@deepseek-ai/dsh']`) doit être satisfait par la version du host (>=X.Y.Z[-prerelease]).
- **Lecture version** : `profileContext.installAnchor` (package.json local, pas de subprocess) → fallback `dsh --version`.
- **Fail-closed** : si inconnu = incompatible (ne jamais installer un plugin incompatible).
- **Mode d'intégration** : plugin.

### 4.4 Atelier créatif — installation d'assets (dsh-market-asset-installer)

**Capacité → Port → Adaptateur :** `skill-store` → `SkillStore` → `omega-context::SkillIndex`

- **Source** : `dsh-market.com` manifest (1 MiB cap, 30s timeout).
- **Validation** : chaque path relatif est validé (pas de .., pas absolu, pas //), URL reconstruite depuis la base.
- **Caps** : 2000 fichiers/asset, 200 MiB/fichier, timeout 30s.
- **Atomic write** : tmp dir + rename, replace only with force.
- **Provenance** : `dsh-market.provenance.json` (sha256 de chaque fichier).
- **Mode d'intégration** : plugin.

### 4.5 Pont téléphone ↔ host (dsh-remote-bridge)

**Capacité → Port → Adaptateur :** `permission-policy` → `PermissionPolicy` → `omega-sandbox::ApprovalQueue`

- **Pairing** : bootstrap token (30s, 32 bytes, loopback-only) → nonce (60s, 16 bytes) → signature ES256 (P-256 SPKI strict).
- **Roster** : `$DSH_HOME/remote-bridge/devices.json` (0600, dir 0700), stocke tokenHash (SHA-256) pas le token, fingerprint du SPKI.
- **Authentification** : constant-time (toutes les rows, pas de early-exit).
- **Approval claim** : claim tant qu'un device paired est connected, delegue au host sinon (fail-closed 'unavailable').
- **Protocole v1** : WebSocket `/ios/v1`, frames req/res/ev/ping/pong (10 MiB cap), session.follow (assistantStream), session.snapshot (baseline + page), session.history (backward page), session.prompt (idempotent).
- **Mode d'intégration** : service (le bridge est un service qui expose le protocole v1).

### 4.6 Agrégat dsh-web-all (dsh-web-all-aggregate)

**Capacité → Port → Adaptateur :** `registry` → `Registre` → `omega-registry::Registry`

- **Fonctionnement** : `aggregate.yml` liste les enfants (patchFrom), les deps npm (deps), les lignes externes (rows), les overrides (familyIds), les lignes par défaut désactivées (inactive), et les tombstones (retire les anciennes lignes).
- **Shell** : les lignes agrégats ont un shell compat pour les anciens profiles (ERR_PACKAGE_PATH_NOT_EXPORTED).
- **Mode d'intégration** : plugin.

---

## 5. Écarté

| Fonctionnalité | Pourquoi écartée |
|---|---|
| `dsh-plugin-manager-detached-work` | Escape hatch AsyncLocalStorage spécifique à Node.js + Cordis HMR. En Rust, on n'a pas ce problème de nested transactions. |
| `dsh-plugin-manager-console-output` | Capture de output console avec fallback GBK — spécifique à Windows OEM code page. En Rust, on std::process::Command directement. |
| `dsh-market-client-categories` | UI browser (React) — hors du périmètre du host Rust. |

---

## 6. Sources

- Clone : `/home/tayk0oo/omega-re/dsh-web` (lecture seule).
- Preuve REA : `docs/omega/research/rea/evidence/intake-dsh-web.summary.json` (provider rea-javascript-application, 7 515 nodes / 12 085 edges, 3 240 limitations, confidence=derived).
- Schéma : `docs/omega/reverse-eng/DISSECTION-SCHEMA.md`.
- Exemple : `omega/crates/omega-harness/data/apps/opencode.json`.

---

## 7. Résumé final

**18 fonctionnalités** (6 adopted, 8 planned, 4 rejected).

**Isolation d'un plugin tiers** : le plugin est un package npm avec `cordis.patch.yml` (insert rows). Le host charge la composition (patch rows + user layer) et les lignes sont montées dans le host. Le browser injecte des panneaux dans les slots DSH. Les plugins tiers ne peuvent pas écrire directement dans le profile : ils passent par le CLI officiel (single writer) ou par le gateway HTTP loopback-only.

**Distribution** : npm (packages individuels ou agrégat). Le market (dsh-market.com) distribue des assets (skins, pets, presets) avec provenance sha256. Les plugins community sont installés via npm ou git URL.

**Intégrations les plus fortes pour Omega** :
1. **Registre de plugins** (install/update/uninstall/enable via gateway HTTP loopback) — déjà adopted.
2. **Barrière de confiance loopback** (socket + Host header + same-origin) — déjà adopted.
3. **Gate de compatibilité** (dsh.engines.dsh minimum) — déjà adopted.
4. **Atelier créatif** (installation d'assets avec provenance) — planned.
5. **Pont téléphone ↔ host** (pairing ES256, approval claim) — planned.

**Blocages** :
- Le loader Cordis (insert rows, patch rows, bundle layer) est spécifique à DSH — Omega doit définir son propre modèle de composition (Slots/Ensembles).
- Le CLI officiel `dsh plugin` est le single writer — Omega doit définir son propre writer (ou réutiliser pnpm directement).
- Les agrégats (dsh-web-all) sont spécifiques à la famille DSH — Omega doit définir son propre modèle d'agrégation.
