# Dissection — Void (Void Editor)

> **Cible** : `voideditor/void` — 28 750 etoiles — fork open source de Cursor.
> **Preuve machine REA** : `docs/omega/research/rea/evidence/intake-void.summary.json`
> — `status: unavailable`, `code: resource_constraint`, `cloned: true`.
> **Limite de la preuve, citee telle quelle** : *« The parent's result capacity was exhausted
> after retaining static facts for `src/vs/workbench/contrib/notebook/browser/contrib/find/notebookFindReplaceWidget.ts`;
> remaining sources were not analyzed »*. Consequence methodologique : **aucune affirmation de
> cette dissection ne s'appuie sur la sortie REA**. Toutes les `refs` sont des `fichier:ligne`
> lus directement dans le clone `/home/tayk0oo/omega-re/void` (lecture seule). La REA n'est citee
> que pour documenter ce qui n'a **pas** pu etre confirme par une source independante.

---

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | **Void Editor** |
| Licence | **Apache-2.0** pour les ajouts Void, **MIT** pour l'heritage VS Code (dual-licence, voir `LICENSE.txt` + `LICENSE-VS-Code.txt` : *« Void is a fork of VS Code, which is licensed under the MIT License. Void's additions and modifications are licensed under the Apache 2.0 License »*) |
| Langage | TypeScript |
| Repo | https://github.com/voideditor/void |
| Version | `voidVersion: 1.4.9` (`product.json:4`), `dataFolderName: .void-editor` (`product.json:7`) |
| Nature | **Fork VS Code en amont**, pas une extension |

### Architecture en une phrase

Tout le code IA tient dans `src/vs/workbench/contrib/void/` (64 fichiers `.ts`/`.tsx`),
reparti en trois couches dictifiees par `VOID_CODEBASE_GUIDE.md` : `common/` (types + prompts,
aucun import), `browser/` (services + composants React), `electron-main/` (reseau +
`node_modules`), communiquees par des canaux IPC `ProxyChannel`.

Deux consequences structurantes :

1. **Le navigateur n'a pas le droit d'importer `node_modules`.** Void a deux contournements :
   bundler React dans le navigateur, ou porter le code dans `electron-main/` et ouvrir un canal.
   C'est pourquoi les appels LLM (`sendLLMMessageChannel.ts`), les metriques
   (`metricsMainService.ts`) et le client MCP (`mcpChannel.ts`) vivent tous dans `electron-main/`.
2. **Le modele est choisi par *fonctionnalite*, pas globalement** :
   `featureNames = ['Chat', 'Ctrl+K', 'Autocomplete', 'Apply', 'SCM']`
   (`voidSettingsTypes.ts:365`). On peut donc faire tourner un modele local pour l'autocompletion
   et un modele distant pour l'agent — un decouplage que peu de harnais font.

---

## 2. Entrees

| Entree | Mecanisme | Ref |
|---|---|---|
| **Void Editor** | binaire Electron, `void.contribution.ts` monte une trentaine de modules | `browser/void.contribution.ts:1-85` |
| **Sidebar Chat** | `Ctrl+L`, action `void.addSelectionToChat`, pane React `Sidebar.tsx` + `SidebarChat.tsx` | `browser/sidebarActions.ts:106-133` |
| **Quick Edit** | `CtrlCmd+K`, `editorFocus && !terminalFocus`, zone de diff restreinte a la selection | `browser/quickEditActions.ts:36-69` |
| **Inline Autocomplete** | `ILanguageFeaturesService`, `_provideInlineCompletionItems` | `browser/autocompleteService.ts:633-670, 892-904` |
| **Apply** | bouton de la sidebar, transforme la reponse LLM en `DiffZone` | `VOID_CODEBASE_GUIDE.md` (section « Apply ») |
| **Commit generator** | Source Control, `FeatureName = 'SCM'` | `browser/voidSCMService.ts:71-152` |
| **Settings pane** | React `Settings.tsx` + `ModelDropdown.tsx` | `browser/react/src/void-settings-tsx/` |
| **Client MCP** | `.mcp.json`, 3 transports | `electron-main/mcpChannel.ts:13-204` |

### Fichiers de configuration

| Fichier | Contenu |
|---|---|
| `common/storageKeys.ts:1-23` | Cles de stockage versionnees (`void.settingsServiceStorageII`, `void.chatThreadStorageII`, `void.app.optOutAll`) avec l'historique des valeurs remplacees commente |
| `.mcp.json` | `{ mcpServers: { nom: {command,args,env} | {url,headers} } }` (`common/mcpServiceTypes.ts:119-132`) |
| `product.json` | Identite de l'application, version Void |

### Points d'extension (au sens Omega)

Void n'a **aucun** point d'extension tierce : les 16 outils builtin sont un objet TypeScript
litteral (`common/prompt/prompts.ts:186-345`) et la seule source d'extension est le **protocole
MCP**, lu depuis un fichier de config et rechargé a chaud (`common/mcpService.ts:157-171`).
Pour Omega, MCP est donc le **seul** point d'integrabilite reelle de ce harnais.

---

## 3. Inventaire fonctionnel

27 fonctionnalites. Table complete (source : `omega/crates/omega-harness/data/apps/void.json`).

| # | Fonctionnalite | Domaine | Ce que ca fait | Ou (file:line) | Cible Omega | Statut | Cout |
|---|---|---|---|---|---|---|---|
| 1 | Fork amont VS Code | `extension` | Code IA dans `contrib/void`, pas une extension ; build React+Tailwind | `README.md:14-24`, `void.contribution.ts:1-85` | — | **rejected** | L |
| 2 | Diagramme de services 3 couches | `extension` | `createDecorator`+`registerSingleton`, canaux `ProxyChannel` | `metricsService.ts:26-54`, `sendLLMMessageTypes.ts:142-151` | `registry`/`Registre` | planned | M |
| 3 | Boucle agent outillee | `agents` | sendLLM → validate → exec → feedback ; retry 3x/2500ms ; **pas de plafond d'iterations** | `chatThreadService.ts:732-911`, `45-46` | `agent-runtime`/`AgentRuntime` | planned | L |
| 4 | 3 modes de chat filtrant les outils | `agents` | `normal`=0 outil, `gather`=lecture seule, `agent`=tout+MCP | `prompts.ts:361-378`, `428-530` | `tool-surface`/`ToolSurface` | planned | S |
| 5 | Checkpoints + voyage temporel | `memoire` | Snapshot par URI en Message ; saut multi-fichiers avant/arriere | `chatThreadService.ts:914-1185` | `event-store`/`EventStore` | planned | L |
| 6 | 16 outils en contrat modal | `outils` | `validateParams`/`callTool`/`stringOfResult` types, `interruptTool` | `toolsService.ts:24-27`, `159-567` | `tool-surface` | planned | M |
| 7 | Surface d'outils searchable | `outils` | nom+description+params, filtres par mode, injection prompt | `prompts.ts:144-398` | `tool-surface`/`ToolSurface` | **adopted** | S |
| 8 | Client MCP multi-transport | `outils` | stdio/streamable-http/SSE, `tools/list`→outils, watcher config | `mcpChannel.ts:13-204`, `mcpService.ts:157-332` | `tool-surface`/`ToolExecutor` | **adopted** | M |
| 9 | Grammaire XML de tool calling | `outils` | `<outil><param>` pour modeles sans tools natifs, parse **en streaming** | `extractGrammar.ts:146-364`, `prompts.ts:402-423` | `tool-surface`/`ToolSurface` | planned | M |
| 10 | Fast Apply Search/Replace | `execution` | Blocs `ORIGINAL/DIVIDER/FINAL`, parseur tolerant au streaming | `extractCodeFromResult.ts:175-265` | `apply-engine`/`ApplyEngine` | planned | L |
| 11 | Moteur de zones de diff | `ui` | `DiffZone`/`CtrlKZone`/`TrackingZone`, realign a chaque edition | `editCodeService.ts:876-945`, `findDiffs.ts:1-251` | `apply-engine`/`ApplyEngine` | planned | L |
| 12 | Approbation par classe d'outil | `securite` | `edits`/`terminal`/`MCP tools`, auto-approbation, etat `awaiting_user` | `toolsServiceTypes.ts:21-30`, `chatThreadService.ts:605-727` | `permission-policy`/`PermissionPolicy` | **adopted** | S |
| 13 | Terminaux temp. vs persistants | `execution` | `run_command` (8s) vs `open/run/kill_persistent_terminal` (5s) | `terminalToolService.ts:1-388`, `prompts.ts:29-31` | `sandbox-backend`/`SandboxBackend` | planned | M |
| 14 | Boucle de retour lint | `execution` | +2s → marqueurs → erreurs renvoyees dans le `tool_result` | `toolsService.ts:413-445`, `574-588` | `exec-policy`/`ExecPolicy` | planned | M |
| 15 | Autocompletion FIM | `ui` | 4 strategies de prediction, LRU 20, debounce 500ms, post-traitement parenthese | `autocompleteService.ts:169-172`, `271-363`, `536-610` | `context-engine`/`ContextEngine` | planned | L |
| 16 | Quick Edit Ctrl+K en FIM tags | `ui` | `preTag/midTag/sufTag` configurables, DiffZone restreinte | `quickEditActions.ts:36-69`, `prompts.ts:698-817` | `context-engine`/`ContextEngine` | planned | M |
| 17 | Arborescence compressee | `contexte` | 2 passes : `Infinity/3` puis `depth 3 / 3 items`, budget 20 000 chars | `directoryStrService.ts:19-23`, `389-485` | `context-engine`/`ContextEngine` | planned | M |
| 18 | Pagination systematique | `contexte` | `page_number` partout : 500 000 chars/fichier, 500 URI/liste | `prompts.ts:25-29`, `toolsService.ts:297-371` | `context-engine`/`ContextEngine` | planned | S |
| 19 | Contexte systeme par tour | `contexte` | OS, workspace, fichier actif, fichiers ouverts, terminaux, arborescence | `prompts.ts:428-530` | `context-engine`/`ContextEngine` | planned | M |
| 20 | 16 fournisseurs en dur | `routage` | `if/else` sur le nom + table litterale de dispatch | `sendLLMMessage.impl.ts:72-172`, `857-941` | `llm-provider`/`Provider` | planned | L |
| 21 | Descripteur de capacites | `routage` | 1 type declaratif : fenetre, FIM, tools, raisonnement, couts | `modelCapabilities.ts:162-252` | `llm-provider`/`Provider` | **adopted** | M |
| 22 | Fournisseurs locaux de premier plan | `routage` | shim OpenAI `{endpoint}/v1` pour Ollama/vLLM/LiteLLM/LMStudio + FIM natif | `sendLLMMessage.impl.ts:81-96`, `621-681` | `llm-provider`/`Provider` | **adopted** | S |
| 23 | Overrides de capacites | `configuration` | 7 cles patchables par modele (fenetre, FIM, tools…) | `modelCapabilities.ts:208-221`, `254-256` | `config-source`/`ConfigSource` | planned | S |
| 24 | Rafraichissement du catalogue | `configuration` | Polling des fournisseurs locaux, resync sur changement de reglage | `refreshModelService.ts:96-160` | `config-source`/`ConfigSource` | planned | S |
| 25 | Instructions IA + settings | `configuration` | `aiInstructions` libre, `modelSelectionOfFeature` par feature | `voidSettingsTypes.ts:441-474`, `storageKeys.ts:1-23` | `config-source`/`ConfigSource` | planned | S |
| 26 | Telemetrie PostHog + opt-out | `observabilite` | `capture(event,params)` + `OPT_OUT_KEY` + debug | `metricsMainService.ts:100-160`, `storageKeys.ts:22` | `telemetry-sink`/`TelemetrySink` | planned | S |
| 27 | Blacklist copilotes + transfert | `extension` | Supprime Cline/Roo/Continue/Cody au demarrage | `extensionTransferService.ts:28-50` | — | **rejected** | S |

**Domaines couverts** : `extension`, `agents`, `memoire`, `outils`, `execution`, `ui`,
`securite`, `contexte`, `routage`, `configuration`, `observabilite` (11 domaines).
**Statuts** : 5 `adopted`, 20 `planned`, 2 `rejected`.

---

## 4. Traduction en plugins

### 4.1 Les cinq integrations les plus fortes

#### A. `vd-checkpoint-timeline` — le voyage temporel multi-fichiers → `EventStore`

**Le probleme que Void resout et que personne ne resout bien** : un agent edite 5 fichiers,
l'utilisateur veut revenir en arriere de 3 messages. Un undo lineaire ne suffit pas, parce que
la derniere version d'un fichier peut etre **posterieure** au point de retour vise.

L'algorithme de Void (`chatThreadService.ts:1123-1148`) :

1. `_getCheckpointsBetween(lo=to+1, hi=from)` donne le dernier index **par URI** sur l'intervalle.
2. Pour chaque URI, un generateur `idxes()` remonte de `toIdx` vers `0`, **puis redescend**
   vers la fin du thread, et s'arrete au premier checkpoint portant un snapshot.
3. Le redo fait l'inverse : applique le snapshot le plus bas entre `from` et `to`.
4. Les modifications manuelles de l'utilisateur sont stockees dans un champ separe
   (`userModifications`) et ne sont **pas** melangees lors d'un saut (ligne 1038).

```
capabilite  → event-store
port        → EventStore
adaptateur  → omega-app::FileEventStore + omega-otel::TraceReplayer
integration → plugin        cout → L       statut → planned
```

Omega a deja le magasin et le rejeu ; il manque **l'algorithme de saut**, qui est la vraie
valeur. Le store de Void est un tableau de messages dans le `.vscdb` ; celui d'Omega est un
journal durable versionne — meme principe, support de rejeu natif. C'est le portage le plus
rentable du lot : peu de code, forte valeur percue utilisateur.

#### B. `vd-xml-tool-grammar` — la seule grammaire qui marche en local → `ToolSurface`

Un `qwen2.5-coder:7b` tourne sur la machine de l'utilisateur. Il ne sait pas appeler d'outils
nativement. Void le lui dit en prompt et **parse la sortie pendant le streaming** :

- `findPartiallyWrittenToolTagAtEnd` (`extractGrammar.ts:146`) detecte une balise **ouverte
  mais incomplete** — indispensable parce qu'on ne peut pas attendre la fin du flux pour savoir
  si c'est un tool call.
- `parseXMLPrefixToToolCall` (168-262) ne retient que les parametres **declares** dans
  l'outillage (ligne 206) : un `<param>inconnu</param>` est ignore, pas execute.
- `prepareMessages_XML_tools` (`convertToLLMMessageService.ts:199-240`) **re-injecte le XML**
  dans le message precedent, sinon le modele oublie qu'il a appele un outil.
- Le format est choisi par capacite : `specialToolFormat: 'openai-style' | 'anthropic-style' |
  'gemini-style' | undefined` ou `undefined` signifie « XML ».

```
capabilite  → tool-surface
port        → ToolSurface (index/hydrate)
adaptateur  → plugins/xml-tool-grammar (parseur streaming tolerant + prompt)
integration → plugin        cout → M       statut → planned
```

C'est le **deblocage** du pilier « self-hosted » d'Omega : sans cette grammaire, un modele
local n'a pas d'agent outille. Cout modere, portabilite elevee.

#### C. `vd-fast-apply` — ne jamais reecrire un fichier entier → `ApplyEngine`

Reecrire 1000 lignes pour en changer 3 coute 1000 lignes de tokens et casse tout le reste.
Void impose donc au modele d'emettre des blocs :

```
<<<<<<< ORIGINAL
// code d'origine
=======
// code de remplacement
>>>>>>> UPDATED
```

Le vrai ingenierie est dans le **parseur incremental** (`extractCodeFromResult.ts:175-265`) :
pour chaque delimiteur, `endsWithAnyPrefixOf` gere le cas « je suis en train d'ecrire
`=====` la, c'est un separateur ». Le coup de genie est sur `FINAL` (lignes 218-222) : Void
prefere `'\n'+FINAL` a `FINAL`, sinon un `>>>>>>> UPDATED` non termine serait confondu avec
une occurrence ulterieure dans le flux. Les ~25 jeux de tests du fichier documentent
exactement ces etats.

```
capabilite  → apply-engine
port        → ApplyEngine (apply/accept/reject)
adaptateur  → plugins/search-replace-apply (parseur streaming + FindDiffs)
integration → plugin        cout → L       statut → planned
```

Le cout est `L` parce qu'il faut aussi porter `FindDiffs` (le matching tolerant d'un bloc
SEARCH a une region du fichier) et le moteur de zones (#11). **Les deux sont indissociables** :
sans le realign des zones, une edition ulterieure en amont decale toutes les zones.

#### D. `vd-directory-str` — le contexte de fichiers qui densifie → `ContextEngine`

Deux passes qui meritent d'etre copiées telles quelles :

```ts
const START_MAX_DEPTH = Infinity;            // passe 1 : on essaie tout
const START_MAX_ITEMS_PER_DIR = Infinity;
const DEFAULT_MAX_DEPTH = 3;                 // passe 2 : si tronque, on resserre
const DEFAULT_MAX_ITEMS_PER_DIR = 3;
```

On calcule d'abord sans limite, et si le budget de caracteres est depasse on **recommence**
avec profondeur 3 / 3 items par repertoire (`directoryStrService.ts:389-435`). Le resultat :
sur un monorepo, un arbre court mais lisible vaut mieux qu'un arbre tronque au hasard.
Deux budgets distincts (20 000 pour le system prompt, 20 000 pour l'outil `get_dir_tree`),
et le budget restant est retranche du pre-alloc (`MAX_DIRSTR_CHARS_TOTAL_BEGINNING - str.length`,
lignes 462 et 473) : plusieurs dossiers racine **se partagent** le budget au lieu de deborder.

```
capabilite  → context-engine
port        → ContextEngine
adaptateur  → omega-context::ContextContributor (DirectoryStr, budget partage) + DefaultContextEngine
integration → plugin        cout → M       statut → planned
```

Omega a `ContextBudget::estimate/check/enforce` ; il n'a pas le **constructeur de contenu**
budgete. C'est un `Contributor` de plus, pas une reecriture.

#### E. `vd-lint-feedback-loop` — rendre les erreurs au modele → `ExecPolicy`

Apres chaque `edit_file` / `rewrite_file`, Void ne renvoie pas « OK » : il attend 2 s, relit
les marqueurs du fichier et **renvoie les erreurs dans le `tool_result`**
(`toolsService.ts:413-445`, `574-588`) :

```ts
edit_file: async ({ uri, searchReplaceBlocks }) => {
    await editCodeService.callBeforeApplyOrEdit(uri)          // checkpoint avant
    editCodeService.instantlyApplySearchReplaceBlocks({ uri, searchReplaceBlocks })
    const lintErrorsPromise = timeout(2000).then(() => this._getLintErrors(uri))
    return { result: lintErrorsPromise }                       // le LLM attend
}
```

et le texte remis au modele (lignes 515-531) :

> *« Lint errors found after change: … If this is related to a change made while calling this
> tool, you might want to fix the error. »*

```
capabilite  → exec-policy
port        → ExecPolicy
adaptateur  → plugins/lint-feedback (diagnostics -> tool_result)
integration → plugin        cout → M       statut → planned
```

C'est une **fermeture de boucle** : l'agent se corrige lui-meme sans intervention humaine.
Omega n'a pas de canal `diagnostics → tool_result`.

### 4.2 Les cinq `adopted` (deja dans notre code, honnetement verifie)

| Fonctionnalite | Port | Adaptateur existant | Ref cote Omega |
|---|---|---|---|
| `vd-tool-surface-index` | `ToolSurface` | `omega-context::ToolSurfaceIndex` | `omega-context/src/tool_surface.rs:7-232` |
| `vd-mcp-client` | `ToolSurface` + `ToolExecutor` | `omega-mcp::McpClient` + `McpToolExecutor` (`impl omega_ports::ToolExecutor`) | `omega-mcp/src/lib.rs:54-140`, `391-490` |
| `vd-tool-approvals` | `PermissionPolicy` | `omega-sandbox::ApprovalQueue` | `omega-sandbox/src/approvals.rs:21-165` |
| `vd-model-capabilities` | `Provider` | `omega-providers::OllamaCatalogSource` + `omega-types::Capabilities` | `omega-providers/src/ollama_catalog.rs:58-155` |
| `vd-local-providers-first-class` | `Provider` | `omega-providers::OllamaProvider` | `omega-providers/src/ollama.rs:1-251` |

Ces cinq sont marques `adopted` **uniquement** parce que le code existe reellement dans
`omega/crates/`. Aucun `adopted` n'est pose sur une intention.

### 4.3 Les deux `rejected`

| Fonctionnalite | Pourquoi on ecarte |
|---|---|
| `vd-fork-vscode` | Un fork d'editeur nous lierait a rebases VS Code et a un runtime Electron/TS : c'est hors du perimetre « Rust core / React-TS UI ». On prend les **algorithmes** (#9, #10, #11, #17), jamais l'hote. |
| `vd-copilot-ext-blacklist` | Propre au fait d'etre un IDE qui disputer des raccourcis clavier a Cline/Roo/Continue. Omega n'a pas de keybindings concurrents a neutraliser. |

---

## 5. Ecarté — et ce qu'on garde quand meme

| Element | Decision | Raison |
|---|---|---|
| Fork VS Code + build React/Tailwind (`README.md:14-24`) | ecarte | cout de maintenance (rebases) + runtime Electron hors perimetre |
| `newOpenAICompatibleSDK` en `if/else` + table litterale de 16 fournisseurs (`sendLLMMessage.impl.ts:72-172`, `857-941`) | forme ecartee, **donnees recyclees** | Modele en dur hors registre = interdit Omega. Mais la table de capacites (#21) est exactement ce qu'il nous faut. |
| Cle PostHog codee en dur (`metricsMainService.ts:96`) | ecarte | Envoyez a un tiers par defaut n'est pas acceptable ; Omega emet en local (`StdoutSink`/`FileSink`) + `Provenance`. |
| `deleteBlacklistExtensions` (`miscWokrbenchContrib.ts:31-36`) | ecarte | Propre a l'hote IDE, cf. ci-dessus. |
| Suppression silencieuse des extensions tierces | ecarte | Effet de bord irreversible sur l'environnement de l'utilisateur. |

### Points faibles releves (a ne pas recopier)

1. **Pas de plafond d'iterations ni de budget** dans la boucle agent
   (`chatThreadService.ts:771-901`). Le seul arret est l'approbation humaine ou l'interruption.
   Un modele local en boucle peut couler un budget indefini.
2. **Aucune validation du path** : commentaire explicite *« We are NOT checking to make sure in
   workspace »* (`toolsService.ts:41`). `validateURI` accepte n'importe quelle chaine et
   `delete_file_or_folder` n'a qu'un `is_recursive`.
3. **Approbation par classe, pas par argument** : `delete_file_or_folder` est dans `'edits'`,
   donc approuve par le meme interrupteur qu'un `edit_file` benin
   (`toolsServiceTypes.ts:21-30`).
4. **`read_file` lit le fichier entier en memoire avant de paginer** (`toolsService.ts:297-320`) :
   pas de fenetre glissante, un fichier de 2 Go (`DEFAULT_FILE_SIZE_LIMIT`) reste un pic RAM.
5. **Delai fixe de 2 s** pour le retour lint au lieu d'attendre la fin du debounce du linter.
6. **`sendFIM: null` chez Anthropic/OpenAI/Gemini** et `lmStudio` en `sendFIM` avec le
   commentaire *« lmStudio has no suffix parameter in /completions, so sendFIM might not work »*
   (`sendLLMMessage.impl.ts:914-919`) — l'auto-detection de capacite est au mieux approximative,
   d'ou le TODO `« double check all context sizes below »` (`modelCapabilities.ts:254`).
7. **`awsBedrock` n'est pas natif** : commentaire *« We treat Bedrock as OpenAI-compatible only
   through a proxy »* (`sendLLMMessage.impl.ts:125-146`), defaut `http://localhost:4000/v1`.

---

## 6. Sources

### Clone local (lecture seule)

`/home/tayk0oo/omega-re/void` — commit `b3166e7`, `product.json` `voidVersion: 1.4.9`.

| Chemin | Role |
|---|---|
| `src/vs/workbench/contrib/void/browser/` (30 fichiers) | Services workbench + React |
| `src/vs/workbench/contrib/void/common/` (21 fichiers) | Types, prompts, MCP, settings |
| `src/vs/workbench/contrib/void/electron-main/` (8 fichiers) | Reseau, node_modules, canaux IPC |
| `VOID_CODEBASE_GUIDE.md` | Guide d'architecture **officiel** (pipeline LLM, Apply, checkpoints) |
| `LICENSE.txt` / `LICENSE-VS-Code.txt` | Double licence Apache-2.0 + MIT |

### Preuve machine REA

`docs/omega/research/rea/evidence/intake-void.summary.json`

```json
{
  "id": "void", "repo": "voideditor/void", "stars": 28750,
  "language": "TypeScript", "license": "Apache-2.0",
  "nature": "js", "cloned": true,
  "status": "unavailable",
  "code": "resource_constraint",
  "message": "The parent's result capacity was exhausted after retaining static facts for src/vs/workbench/contrib/notebook/browser/contrib/find/notebookFindReplaceWidget.ts; remaining sources were not analyzed"
}
```

> **Limite assumee.** La REA a epuise sa capacite de resultat sur un fichier **hors sujet**
> (le widget *Find/Replace* des notebooks de VS Code, sans rapport avec Void) et n'a analyse
> aucune source de `contrib/void/`. Consequence : cette dissection repose **exclusivement** sur
> la lecture directe du clone. Aucune affirmation n'est attribuee a la REA, et les `refs` ne
> citent aucun artefact REA. A refaire plus tard si la REA redevient disponible.

### URLs officielles

- Codebase guide : https://github.com/voideditor/void/blob/main/VOID_CODEBASE_GUIDE.md
- Contribution / build : https://github.com/voideditor/void/blob/main/HOW_TO_CONTRIBUTE.md
- Build pipeline : https://github.com/voideditor/void-builder
- Site : https://voideditor.com
- Roadmap : https://github.com/orgs/voideditor/projects/2
- Spec MCP (2025-03-26) citee dans `common/mcpServiceTypes.ts:12`

### Fichiers produits par cette dissection

- `omega/crates/omega-harness/data/apps/void.json` (27 fonctionnalites)
- `docs/omega/reverse-eng/dissections/void.md` (ce fichier)