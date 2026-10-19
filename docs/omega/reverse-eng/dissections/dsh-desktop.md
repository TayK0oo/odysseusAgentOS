# Dissection — DSH Desktop (`anywhere-labs/dsh-desktop`)

> Cible : 31 119 etoiles · TypeScript · MIT · editeur Anywhere Labs (Communaute, sans
> lien de subordination avec DeepSeek) · clone `/home/tayk0oo/omega-re/dsh-desktop`
> · preuve REA `docs/omega/research/rea/evidence/intake-dsh-desktop.summary.json`
> (6 812 noeuds / 12 028 aretes, 1 079 limites).
> Donnees : `omega/crates/omega-harness/data/apps/dsh-desktop.json` (26 fonctionnalites).

---

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Anywhere Labs (projet communautaire, declare sans participation d'employes DeepSeek ni de l'equipe amont) |
| Licence | MIT |
| Langage | TypeScript (workspace Yarn 4.18 `nodeLinker: node-modules`, sous-module amont en pnpm) |
| Repo | <https://github.com/anywhere-labs/dsh-desktop> |
| Etoiles | 31 119 |

**Architecture.** Coquille Electron mince au-dessus d'un sous-module Git **epingle** de
DeepSeek Harness execute **tel quel** : « ce projet ne modifie pas le code amont et
n'est pas une coque figee » (README). Le Host (Cordis) demarre dans le processus
principal Electron, monte les entrees du Loader, et expose le Web UI amont via un
porteur **HTTP + WebSocket** sur la boucle locale. Le renderer est une page sandboxed
**same-origin** : il ne voit ni Electron, ni les services Host, ni un pont IPC
generique. Le誌 shell desktop lui-meme est un plugin comme les autres.

Diagramme amont :

```
User ──> Electron main / tray / window
       ──> Launcher ──> Host (generation Cordis)
                          ├──> Carrier HTTP+WS ──> Renderer sandboxed
                          ├──> Services amont (agent/modele/outils/session)
                          ├──> Plugins propriétaires Desktop
                          └──> Plugins tiers
```

**Ordre de demarrage** (`docs/architecture.en.md:21-31`) : verrou mono-instance →
lecture de l'etat profil/mode → preparation du profil actif → fourniture du runtime
natif, du bootstrap `desktopProfiles` et du pnpm empaquete → **montage des entrees
Loader avec les services Desktop enregistres AVANT toute entree tierce** → composition
`dsh-base` + `dsh-web-app` + bundles du profil → binding boucle (ou toutes interfaces
sur confirmation explicite) → creation du BrowserWindow sur la meme origine →
**creation du tray seulement apres chargement de la surface Web** → profil commite
comme `last-known-good`.

**Point le plus important pour Omega.** Le slogan du depot — « 万物皆插件 », tout est
plugin — n'est pas un argument marketing : il est **verifiable dans le code**. La
coquille desktop est un plugin Cordis sans privilege special, insere dans le bundle du
profil par un patch declaratif, avec `inject` sur des services amont ordinaires.

---

## 2. Entrees et points d'extension

### Binaires / surfaces

| Surface | Nature | Ref |
|---|---|---|
| `dsh-plugin-desktop` (npm, stable) | paquet Cordis + bootstrap Electron | `upstream.json:12`, `cordis.patch.yml:1-25` |
| `dsh-plugin-desktop-beta` (npm, beta) | paquet physique separe | `upstream.json:18`, `docs/architecture.en.md:64-71` |
| `dsh-desktop-next` | coquille experimentale (Web frontend amont publie) | `dsh-desktop-next/README.md` |
| processus principal Electron | fenetre, tray, terminal, updates, generations | `dsh-plugin-desktop/src/electron-runtime.ts` |
| tray | commandes `desktop-terminal`, `desktop-diagnostics`, `desktop-profiles`, `desktop-updates` | `terminal.ts:22-31`, `diagnostics.ts:14-23` |
| `/api/desktop/*` | 12 routes privees, filtrees par `ctx.connection.requestRejection` | `desktop-settings-contract.ts:5-43` |
| `/_dsh/desktop/renderer-boot` | rapport de boot client → Host | `renderer-boot-contract.ts:3` |
| `/api/community-market/*` | 10 routes du marche | `dsh-community-market/src/host/routes.ts:45-58` |

### Points d'extension (les 4 verbes)

1. **Entree Loader** (`cordis.patch.yml`) — insérer/desactiver une ligne de bundle.
2. **`export const inject`** + **`ctx.provide`** — publier/consommer un service Cordis.
3. **`ctx.slots.register`** — occuper une region d'UI (cote client, dans la fibre navigateur).
4. **`ctx.effect(label, disposer)`** — toute ressource est un effet etiquete et jetable.

### Fichiers de configuration cles

| Fichier | Role |
|---|---|
| `cordis.patch.yml` (paquet + `home/`) | composition declarative des lignes Loader |
| `package.json` du profil (`dsh.profile.bundles`, `dsh.desktopDeselectedBundles`) | source de verite de la composition |
| `upstream.json` | pin du sous-module, versions, manifestes runtime vendorises par canal |
| `home/settings.yaml` | config Host ; l'id de ligne Loader **est** la cle de config |
| `home/desktop-preferences.json` (Next) | preferences Desktop hors Host |
| `docs/schemas/*.schema.json` | contrats publics du marche |

---

## 3. Inventaire fonctionnel

| Fonctionnalite | Domaine | Ce que ca fait | Ou (`file:line`) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|
| `dsh-shell-as-plugin` | extension | La coquille desktop est un plugin Cordis ordinaire, sans privilege | `src/index.ts:80-84`, `:149-158`, `cordis.patch.yml:1-25` | `registry` → `Registre` | **adopted** | M |
| `dsh-ui-plugged-in-slots` | ui | L'UI se plugue dans les slots `root` ; ancres DOM stables pour la compatibilite | `client/extended-shell.ts:19-49`, `client/advanced-shell.ts:43-44`, `plugin-services.md:98-109` | `ui-surface` → `UiSurface` | planned | M |
| `dsh-window-geometry-service` | ui | `desktopWindow` : geometrie native immuable, sans mutation de fenetre | `client/window-service.ts:20-73`, `plugin-services.md:66-92` | `ui-surface` → `WindowGeometry` | planned | S |
| `dsh-public-private-boundary` | extension | 3 services publics, tout le reste declare prive | `plugin-services.md:196-240`, `:260-311` | `registry` → contrat vs prive | planned | S |
| `dsh-profile-generation-scope` | configuration | Profil = generation jetable ; `select()` = redemarrage | `plugin-services.md:131-148`, `architecture.en.md:21-48` | `config-source` → `ConfigSource` | planned | M |
| `dsh-pnpm-one-op-gate` | execution | Une operation pnpm par generation, arbre possede, **aucun** rollback | `src/pnpm.ts:41-60`, `plugin-services.md:150-194` | `exec-policy` → `ExecPolicy` | planned | M |
| `dsh-plugin-inventory-opaque-ids` | extension | Inventaire a ids opaques + desactivation par couches | `src/desktop-plugins.ts:59-135`, `:225-252` | `registry` → `describe/active_ids` | planned | M |
| `dsh-quarantine-previews` | securite | Confirmation montee a usage unique (TTL 5 min) avant quarantaine | `src/desktop-plugins.ts:29-41`, `:127-146` | `registry` → `quarantine` | **adopted** | S |
| `dsh-market-public-schemas` | extension | 4 JSON Schema publics compiles en types ; commandes fournisseur jamais executees | `contracts/schemas.ts:21-38` | `registry` → `Manifest` | planned | M |
| `dsh-market-renderer-no-package-name` | securite | Le renderer ne nomme jamais un paquet ; l'Host resout l'identite vue | `install/service.ts:27-45`, `:110-140` | `permission-policy` → `ApprovalQueue` | planned | M |
| `dsh-restricted-egress-client` | securite | HTTPS seul, DNS epingle, blocklist SSRF, 3 redirections max, 2 Mo | `network/blocked-subnets.ts:1-58`, `network/restricted-http.ts:11-15`, `:73-84` | `sandbox-backend` → `LocalSandboxBackend` | planned | M |
| `dsh-market-delegates-to-manager` | execution | Toute mutation delegatee au `pluginManager` amont ; zero etat propre | `install/service.ts:59-65`, `README.md` | `registry` → `replace/unregister` | planned | M |
| `dsh-native-permission-service` | securite | `query/request/openSettings` ; le Host ne peut pas imiter un geste | `next/src/permissions.ts:1-35` | `permission-policy` → `ApprovalQueue` | planned | S |
| `dsh-no-generic-ipc` | securite | Aucun pont IPC generique ; canaux nommes ou routes filtrees | `next/src/preload-app.ts:1-47`, `src/index.ts:88-96` | `permission-policy` → `ApprovalQueue` | planned | S |
| `dsh-window-hardening` | securite | Fabrique unique : sandbox, partition `dsh-*`, pas de popup/webview | `src/local-window-policy.ts:19-46` | `sandbox-backend` → `LocalSandboxBackend` | planned | M |
| `dsh-lan-https-edge` | securite | Le Host reste sur `127.0.0.1` ; l'LAN passe par une arete HTTPS scellee | `src/desktop-network.ts:6-45`, `src/index.ts:196-231` | `sandbox-backend` → `LocalSandboxBackend` | planned | M |
| `dsh-renderer-boot-report` | observabilite | Issue de boot du Loader, meme origine, 16 Ko / 64 plugins max | `src/renderer-boot.ts:1-80`, `client/ui-diagnostics.ts:1-8` | `event-store` → `EventStore` | planned | S |
| `dsh-run-scoped-logging` | observabilite | Effets etiquetes, fichier borne 128 KiB, tracing opt-in | `src/index.ts:300-345`, `settings-bridge.ts:95-113` | `telemetry-sink` → `TelemetrySink` | **adopted** | S |
| `dsh-secret-masking-and-archive` | securite | Redaction des logs + export dans un Worker (50 Mo / 60 s) | `src/mask-secrets.ts:1-50`, `src/diagnostic-export.ts:1-50` | `telemetry-sink` → sinks + redaction | planned | M |
| `dsh-profile-checkpoints` | memoire | 3 slots rotatifs de fichiers declaratifs, restaurables sans pnpm | `src/profile-checkpoint.ts:1-10`, `:41-70` | `event-store` → `FileEventStore` | planned | M |
| `dsh-release-channel-protocol` | configuration | Stable/Beta physiques, en-tetes d'echo, SemVer stricte, SHA-256 avant execution | `upstream.json:1-14`, `src/update-checker.ts:11-30` | `config-source` → `ConfigSource` (canal) | planned | M |
| `dsh-declared-config-subset` | configuration | Sous-ensemble editable declare ; l'id de ligne Loader = namespace | `src/settings-bridge.ts:1-19`, `:63-77` | `config-source` → `omega-config` | planned | S |
| `dsh-theme-token-projection` | ui | Projection du snapshot de theme, `dispose` sans effet de bord | `client/theme-presenter.ts:6-37` | `ui-surface` | planned | S |
| `dsh-optional-inject-lifecycle` | automatisation | `ctx.inject` imbrique pour les deps live ; effet detruit avec la fibre | `src/notifications.ts:113-150`, `:72-110` | `registry` → dep optionnelle | planned | S |
| `dsh-variant-and-direction-gates` | execution | Portes CI : editions alignees octet par octet ; marche ne depend pas du Desktop | `scripts/verify-desktop-variants.mjs:1-33`, `scripts/market-dependency-direction.mjs:1-40` | `registry` (gate CI) | planned | S |
| `dsh-pinned-upstream-yarn-patches` | extension | **ECARTE** — 80+ patches Yarn version-scopes sur l'amont | `patches/`, `AGENTS.md:20-27` | — | **rejected** | L |

---

## 4. Traduction en plugins

### 4.1 Le desktop comme plugin (le coeur de la these)

Le point d'entree du paquet stable (`dsh-plugin-desktop/src/index.ts:80-84`) :

```ts
export const name = 'desktop-shell'
export const inject = ['webServer', 'appExit', 'settings', 'connection']
```

Puis `apply(ctx, config)` monte la generation native. Trois consequences
verifiables :

1. **Composition declarative, pas d'edition de code.** `cordis.patch.yml:1-25`
   insere sept lignes Loader (`desktop-shell`, `desktop-terminal`,
   `desktop-diagnostics`, `desktop-notifications`, `desktop-pnpm`,
   `desktop-profiles`, `desktop-updates`) et patche `web-runtime` pour
   desactiver `openBrowser` (« un lancement desktop n'a pas d'operateur terminal
   devant »). Pour *desactiver* une capacite, on ecrit `disabled: true` — jamais
   on ne modifie le plugin.
2. **Degradation propre hors Desktop.** `apply()` ecrit sur `stderr` et **retourne**
   si `ctx.desktopRuntime` est absent (`index.ts:150-158`) : le meme paquet reste
   chargeable dans un `dsh web` ordinaire, ou il ne fait rien.
3. **Une capacite = une ligne.** `desktop-diagnostics` fait 14 lignes
   (`diagnostics.ts:9-24`), `desktop-terminal` refuse de s'activer sur Linux,
   `desktop-updates` expose sa politique de sondage par un schema zod
   (`updates.ts:16-43`). C'est le decoupage exact qu'Omega cherche : registre
   granulaire, pas gros module.

**Traduction** : `registry` → `Registre` → `omega-registry`
(`register/resolve/quarantine/active_ids`) — **deja la nôtre**, c'est pourquoi le
statut est `adopted`. Ce qu'il faut reprendre, c'est la **discipline** :

- une entree de manifeste porte son `inject` et son mode de desactivation ;
- un adaptateur degrade proprement quand la capacite hote est absente ;
- les services **hotes** (inchange entre generations) sont distingues des services
  **de generation** (jetables a chaque bascule de profil/mode) ;
- les references de service ne traversent **jamais** une frontiere de generation.

### 4.2 L'UI comme plugin : slots + ancres, pas remplacement du DOM

C'est la fonctionnalite la plus transposable a notre cockpit React.

```ts
ctx.effect(() => ctx.slots.register({
  name: 'root',
  children: {
    'sidebar':       { kind: 'single', scope: 'root' },
    'main':          { kind: 'keyed',  scope: 'root' },
    'rightbar':      { kind: 'single', scope: 'root' },
    'shell.overlay': { kind: 'list',   scope: 'root' },
  },
  inject: () => ({ layout, platform }),
}, ExtendedFrame))
```

`applyExtendedOwnedShell` (`client/extended-shell.ts:19-49`) **enregistre sa propre
racine** et continue d'heberger les occupants amont (sidebar, conversation,
details) dedans. `applyAdvancedShell` fait pareil (`client/advanced-shell.ts:43-44`).
`installDesktopLayout` fournit le service `layout` et le hook root `panelInfo` via
`ctx.slots.provideRoot` (`client/layout-service.ts:7-20`).

La probleme que cela resout — et que toute refonte d'UI encountering — est nomme
dans `plugin-services.md:98-109` :

> « A plugin that locates a shell region by querying for the upstream column classes
> would otherwise find nothing and mount nothing, **with no error to observe**. »

La reponse : des **ancres DOM stables** declarees comme contrat, pas des classes CSS.
`[data-pane="sidebar"]`, `[data-rightbar-col"]`, `[data-shell-overlay"]`, et
l'alias `.dshDesktop_sidebarCol` qui n'a **aucun style** et n'existe que pour que
`[class*="sidebarCol"]` resoudre identiquement sous les deux coquilles. La regle
dependant-plugin : « Query for the anchor, then search within it. Do not depend on
the presentation classes, on element tag names, or on nesting depth. »

Deux autres decisions из UI reutilisables :

- le viewport racine est un **bloc fixe separe** sous la barre de commande, donc
  un `position:fixed` ne peut pas fuir dans le chrome natif ; et la reservation
  exacte (36 px) est publiee par le parametre d'URL `dsh-desktop-titlebar-inset`
  pour les portails body-level, « Plugins must not compensate for a boundary they
  already consume » ;
- `DesktopThemePresenter` (`client/theme-presenter.ts:17-37`) ne fait qu'appliquer
  et **desappliquer** ses propres variables CSS, jamais celles d'autrui.

### 4.3 La surface publique, en trois services

| Service | Face | Contenu | Ref |
|---|---|---|---|
| `desktopProfiles` | Host | `current` immuable, `list()` lecture seule, `select()` = redemarrage | `plugin-services.md:131-148` |
| `desktopPnpm` | Host | `run/runPlugin/runExternalMarketPluginInstall` → `{stdout, stderr, done, cancel}` | `plugin-services.md:150-194` |
| `desktopWindow` | Client | geometrie figee ; **aucune** mutation de fenetre | `plugin-services.md:66-92` |

Le reste est nommement prive : `desktopRuntime` (interne), `desktopPnpmBootstrap`
(chemins packing, ABI Electron), `DesktopProfileServiceBootstrap` (entree de
constructeur, « pas un service Cordis »). Regle explicite et importante :

> « The fact that a private type is present in emitted declarations does not make
> its runtime service a supported third-party capability. »

Et le pattern d'injection qui permet a **un seul paquet** de tourner dans les deux
mondes : dependances dures en tete (`inject = ['desktopProfiles','desktopPnpm']`)
pour un plugin Desktop-only ; `ctx.get('desktopProfiles')` comme discriminant puis
`ctx.inject(['desktopPnpm'], ...)` imbrique pour un plugin cross-environnement,
avec le fallback DSH ordinaire qui reste l'implementation **de reference** — jamais
l'inverse. Regle de securite : « Never fall back to a guessed `web` profile after
`desktopProfiles` is present. »

**Traduction** : `registry` → `Registre` (contrat public porte par l'`EntryInfo`,
surface privee hors registre). Cout S. Omega n'a pas encore de notion de
« surface d'API publique documentee vs surface interne » — c'est a ajouter.

### 4.4 Execution de sous-processus : la porte a usage unique

`src/pnpm.ts:41-60` est la meilleure description de frontiere d'execution que j'aie
lue dans un harnais JS :

```ts
/** ... Desktop deliberately performs no install snapshot, rollback, retry,
 *  protection, receipt reconciliation, or recovery bookkeeping. */
```

Les regles qui rendent la chose sure :

- **une seule operation de packages par generation** ; la seconde leve
  **synchrone**, avant meme de retourner un handle ;
- argv **non vide et sans NUL** ; `run()` prend le repertoire du profil actif comme
  cwd, les adaptateurs exigent un repertoire absolu appartenant a l'appelant ;
- `done` ne se resout pas au retour du wrapper direct mais **quand l'arbre complet
  est mort** ; `cancel()` puis teardown de generation visent l'arbre entier
  (`taskkill /t /f` sous Windows, `process.kill(-pid)` ailleurs) ;
- erreurs de spawn → `done` **rejette** ; echec de commande → `done` **resout**
  avec `exitCode != 0` ; le consommateur doit traiter `exitCode` **et** `signal` ;
- pas de timeout interne : **la deadline appartient a l'appelant** ;
- un exit code 0 « ne remplace pas la validation metier post-installation ».

**Traduction** : `exec-policy` → `ExecPolicy` →
`omega-sandbox::RuleBasedExecPolicy + LocalSandboxBackend`. Cout M. Omega a la
politique ; il lui manque le **handle a signaux** et la garantie d'arbre.

### 4.5 Securite : quatre patrons tres reutilisables

1. **Le renderer ne nomme jamais un paquet.** Le client soumet `sourceRecordId` +
   `itemId` ; le Host resout l'identite npm normalisee *qu'il a lui-meme observee*.
   Plus : intention montee a usage unique de 5 min, 256 intentions max, 10 000
   candidats max, et eligibilite automatique volontairementetroite (un seul nom
   npm valide + npm `latest` stable exact + manifeste declarant un
   `dsh.bundle.patch`). Les badges du fournisseur **ne decide de rien**.
2. **Client HTTP restreint.** HTTPS seul, port 443, sans credentials ; DNS resolu
   puis **epingle** (on se connecte a l'adresse resolue, chaque redirection
   revalidee) ; `BlockList` couvrant RFC1918, loopback, CGNAT, lien-local, ULA,
   NAT64, 6to4, Teredo, multicast et la plage fake-IP des proxies locaux
   (`blocked-subnets.ts`) ; corps 2 MiB, 3 redirections, 8 s/12 s/30 s. Le module
   est **partage** entre le client catalogue et le fetcher d'images pour qu'ils ne
   puissent pas diverger.
3. **Aucun pont IPC generique.** Huit modules de preload dedies, tous commentes
   « no generic IPC bridge » ; cote Host, chaque route Desktop passe par
   `ctx.connection.requestRejection` **avant** tout handler. Et la geometrie
   fenetre ne donne aucun pouvoir : ni mutation, ni focus, ni IPC.
4. **Permissions : l'hote ne peut pas imiter l'utilisateur.** `query` (lecture OS,
   « unknown » n'implique jamais une autorisation), `request` (renderer : geste
   utilisateur ; Host : **revele le dialogue** et renvoie l'etat, le plugin doit
   revérifier), `openSettings`.

### 4.6 Observabilite

- Chaque ressource est un `ctx.effect(label, disposer)` : le libelle est un
  identifiant de diagnostic gratuit.
- Le client remonte son issue de Loader a `/_dsh/desktop/renderer-boot` :
  POST seul, `origin` identique obligatoire, `application/json` obligatoire,
  16 KiB / 64 plugins / 512 caracteres / 12 KiB d'erreur, reponse 204.
- `installUiDiagnostics()` forwarde `window.error` et `unhandledrejection`
  (tronques a 8 KiB) vers la capture Desktop : sans ca, **une erreur UI est
  invisible**.
- L'export de diagnostic part dans un **Worker** borne (50 Mo, 60 s) pour ne pas
  bloquer le thread principal Electron, et `maskSecrets()` redige les logs a
  l'ecriture (cles `sk-`, tokens 32+ chars, Bearer/Basic, cookies, credentials
  d'URL, tous les noms de champs secrets).

### 4.7 Memoire / reprise : les checkpoints declaratifs

Trois slots rotatifs de fichiers **declaratifs** seulement (`package.json`,
`pnpm-lock.yaml`, `pnpm-workspace.yaml`, `cordis.patch.yml`, etat du marche,
`home/settings.yaml`, `home/cordis.patch.yml`). Restaurer ne lance **jamais** pnpm
et ne copie **jamais** `node_modules` ; checksum verifie avant ecriture, modes
0600/0700, version de manifeste evolutive (v2→v3→v4). Apres restauration, le
premier demarrage sain consomme un **skip marker** pour ne pas ecraser un point de
controle fraichement restaure. C'est le bon reflexe : la reprise d'un
*configuration cassee* ne se fait pas au niveau du code installe.

---

## 5. Ecarte (et pourquoi)

| Element | Decision | Raison |
|---|---|---|
| 80+ **patches Yarn version-scopes** sur les paquets amont (`patches/dsh@0.2.1-alpha.5.patch`, …) | **rejected** | Modification non typee d'une dependance tierce, invisible du type-checker amont, cassee silencieusement a chaque bump. Exemple : le patch Computer Use reroute `check_permissions`. Pour Omega la voie est **ports + adaptateurs** (`omega-registry`, `omega-sandbox`, `omega-otel`), pas reecriture de dependances. Cout si repris : **L**. |
| Runtime JS/TS in-process (le monde `dsh-web`) | hors perimetre | Coherent avec `oc-codemode-lsp` de la dissection OpenCode : hors perimetre Rust. |
| Three modes de presentation (`compatibility` / `extended` / `advanced`) | non repris tel quel | C'est une negociation de compatibilite avec un amont fige ; Omega n'a pas d'amont fige a ne pas casser. **Retenir uniquement le mecanisme** (ancres, slots, geometrie en service immuable). |
| Raccourcis clavier, menus natifs, Dock, icones, NSIS/Icon Composer | hors cible | Detaille Electron/macOS/Windows sans transposition architecturale. |
| **Epinglage du sous-module + manifeste runtime vendorise** (`upstream.json`) | **retenir** | Pratique de **provenance** : chaque canal enregistre commit, version source, version runtime et manifeste. Aligne sur `omega-otel::Provenance`. |

---

## 6. Sources

### Clone (lecture seule)
- `dsh-plugin-desktop/src/index.ts` — point d'entree du shell comme plugin
- `dsh-plugin-desktop/src/{diagnostics,terminal,updates,notifications,pnpm,profiles}.ts` — une capacite = une ligne
- `dsh-plugin-desktop/src/desktop-plugins.ts` — inventaire opaque + previews
- `dsh-plugin-desktop/src/{profile-checkpoint,mask-secrets,diagnostic-export,local-window-policy,desktop-network,renderer-boot}.ts`
- `dsh-plugin-desktop/src/client/{index,extended-shell,advanced-shell,layout-service,window-service,theme-presenter,ui-diagnostics}.ts(x)`
- `dsh-plugin-desktop/docs/plugin-services.md` — **le document de contrat** (344 lignes)
- `dsh-plugin-desktop/cordis.patch.yml`, `upstream.json`, `AGENTS.md`
- `dsh-community-market/src/{contracts/schemas,network/blocked-subnets,network/restricted-http,install/service,host/routes,adapters/*}.ts`
- `dsh-desktop-next/src/{permissions,extensions,ipc,preload-*}.ts`, `dsh-desktop-next/README.md`
- `scripts/{verify-desktop-variants,market-dependency-direction}.mjs`
- Tests : 154 fichiers (`dsh-plugin-desktop`), 152 (beta), 78 (next), 20 (market)

### Preuve machine
- `docs/omega/research/rea/evidence/intake-dsh-desktop.summary.json` — 6 812 noeuds, 12 028 aretes, 1 079 limites, confiance `derived`

### Limites de l'analyse (a assumer)
- Le sous-module `deepseek-harness/` est **vide** dans le clone : tout ce qui touche
  le noyau amont (Loader, Cordis, slots, `pluginManager`, `webServer`, `connection`)
  est decrit **via la documentation du projet et ses declarations de types**, pas
  par lecture du code amont. Les invariants d'injection (`pending` tant qu'une
  dependance manque, dechargement si elle disparait) sont donc cites, non verifies.
- `dsh-plugin-desktop-beta/` et `dsh-plugin-desktop/` sont des miroirs verifies
  **octet par octet** (`verify-desktop-variants.mjs`), donc les refs du paquet
  stable sont valables pour beta sauf `product-identity.ts`.
- `dsh-community-fabric/` (RFC d'interoperabilite unifiee) est explicitement un
  **echafaudage documentaire prive** : « must not declare loadable DSH or package
  entry points ». Aucun contrat unifie n'est donc encore utilisable comme
  dependance — c'est une reservation, pas une fonctionnalite.
- Les chiffres de tests (154/152/78/20 fichiers) mesurent la **quantite** de
  surface de test, pas sa qualite ; la qualite se lit dans les assertions
  citees (bornes, TTL, refus explicites).
- Les linedocs comme `docs/architecture.en.md` sont eux-memes des **decisions
  ecrites** ; la coherence code↔doc est forte (les noms de fichiers cites
  existent), mais une divergence resterait possible.

---

## 7. A retenir pour Omega (3 conclusions)

1. **Notre cockpit React peut devenir un plugin de slots, pas un remplacement de
   page.** Le patron `slots.register({name:'root', children:{sidebar, main,
   rightbar, shell.overlay}})` + ancres DOM stables (`[data-pane=...]`) est
   directement applicable a `omega/ui/src/` et resout un probleme que nous
  ]n'avons pas encore rencontre mais que nous rencontrerons.
2. **Trois services publics, tout le reste prive.** Omega a le registre
   (`omega-registry`) mais pas encore l'**API publique documentee** avec sa liste
   negative explicite. C'est une decision de conception, pas une ligne de code.
3. **La discipline du marche est la meilleur agrement du repo.** Deleguer toute
   mutation au gestionnaire officiel, ne jamais nommer un paquet depuis le client,
   reserver l'egress, et assumer « pas de snapshot / pas de rollback » par
   construction : ce sont des invariants de conception verifiables, pas des
   fonctionnalités. Ils doivent etre ports dans `omega-registry` et
   `omega-sandbox` **avant** d'ecrire la moindre ligne de marketplace.