# Dissection — bolt.new (StackBlitz, MIT (Apache-2.0 parts), TypeScript)

> Source : clone `--depth 1` de https://github.com/stackblitz/bolt.new dans `/home/tayk0oo/omega-re/bolt-new`.
> Donnees : `omega/crates/omega-harness/data/apps/bolt-new.json` (9 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | StackBlitz |
| Langage | TypeScript |
| Repo | https://github.com/stackblitz/bolt.new |
| Architecture | Agent de creation de web apps dans le navigateur (Remix-vite + Cloudflare Worker) : WebContainer in-browser-isolated runtime, ActionRunner serialise (file/shell/start) avec abort par signal, message homme fin initialement dédié a un' tags boltArtifact/boltAction actionnables, stores nanostores par domaine, historique de chat persistence IndexedDB + drizzle. |

## 2. Inventaire fonctionnel

### `blt-webcontainer` — WebContainer in-browser : node runtime fully en sandbox browser

- **domaine** : securite · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : M · **statut** : planned
- **what** : Le code genere se construit et tourne dans un conteneur Node/WebContainer (StackBlitz) embarque dans l'onglet du navigateur (pas de serveur).
- **how** : app/lib/webcontainer/index.ts:21-38 boot une instance unique WebContainer.workdirName=WORK_DIR_NAME avec HMR-safe (import.meta.hot.data). Omega : NamespaceSandboxBackend couvre rest菲律宾입장 ; retenu : le sandbox client-side demand& par besoin; pas transéposable direct, mais le pattern sandbox isolée du runtime est couvert.
- **adaptateur Omega** : omega-sandbox (sandbox containerized)
- **preuves** : `app/lib/webcontainer/index.ts:21-38`

### `blt-action-runner` — ActionRunner : file/shell/start actions sequentielles avec abort

- **domaine** : execution · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : M · **statut** : planned
- **what** : Un runner d'actions en serie dans le WebContainer avec abort par signal, file écriture直 seriée: shell via terminal + start pour le serveur de dev.
- **how** : class ActionRunner (app/lib/runtime/action-runner.ts:36) ; ActionStatus 'pending|running|complete|aborted|failed' + abortSignal (action-runner.ts:9-16) ; #runFileAction ecrit sur le FS WebContainer (action-runner.ts:150) ; #runShellAction (action-runner.ts:124) passe par le terminal ; les actions sont en serie (#currentExecutionPromise :38). Omega : ToolExecutor + effets reversibles : l'sanitisation d'action (pend/running/failed) et l'execution serialise sont deja couverts ; l'abort signal par action est le model.
- **adaptateur Omega** : omega-app (ActionRunner serialise + abort par action)
- **preuves** : `app/lib/runtime/action-runner.ts:36-50` · `app/lib/runtime/action-runner.ts:150-168`

### `blt-message-parser` — Message parser : tags boltArtifact/boltAction en actions indexables

- **domaine** : routage · **capacite** : event-store · **port** : EventStore · **cout** : S · **statut** : planned
- **what** : Le message de l'assistant parse des tags specialises (boltArtifact, boltAction avec type = file|shell|start) pour rendre chaque bloc actionnable par l'utilisateur.
- **how** : app/lib/runtime/message-parser.ts:6-9 ARTIFACT_TAG_OPEN=<boltArtifact>, ARTIFACT_ACTION_TAG_OPEN=<boltAction> ; les actions extraites alimentent ActionRunner.addAction (message-parser reference). Omega : 'omega-server' event-sourced ; un parser de tags determines sert la reponse actionnable.
- **adaptateur Omega** : omega-server (parser de tags d'actions)
- **preuves** : `app/lib/runtime/message-parser.ts:6-9`

### `blt-workbench-stores` — Workbench stores : chat/files/terminal/previews separés par nanostores

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : S · **statut** : planned
- **what** : Octupler des domaines (chat, files, editor, terminal, previews, settings, theme) en stores atomiques nanostores décollés complets.
- **how** : app/lib/stores/{chat,files,editor,terminal,previews,theme,settings}.ts ; workbench.ts assembles. Omega : omega-server + omega-ui couverts ; la note separée des stores par domaine astuce.
- **adaptateur Omega** : omega-ui (stores par domaine)
- **preuves** : `app/lib/stores/workbench.ts`

### `blt-worker-remix` — Remix-vite app serveur avec Cloudflare deploy (wrangler.toml).

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : L'app isolée est un Remix app en worker Cloudflare (wrangler.toml) : le noyau se marie a un runtime par cloud.
- **how** : wrangler.toml + vite.config.ts de Remix vite plugin. Omega : beta-app unit server : omega-server expose mono runtime, le pattern de provider de runtime (Cloudflare) est note.
- **adaptateur Omega** : omega-app (runtime provider wrapper)
- **preuves** : `wrangler.toml` · `vite.config.ts`

### `blt-llm-stream` — streamText unifier (ai sdk) + judge de model via env

- **domaine** : routage · **capacite** : router-policy · **port** : Selector · **cout** : S · **statut** : planned
- **what** : Une streamText isole dans stream-text.ts ; les fournisseurs sont configure de envs (api key).
- **how** : app/lib/.server/llm/stream-text.ts:24-30 ; constants.ts pour les modes; prompt.ts pour les templates. Omega : omega-router::Selector + flags ; le pattern unifier de streamText est note.
- **adaptateur Omega** : omega-router (stream-text utilitaire)
- **preuves** : `app/lib/.server/llm/stream-text.ts:24-30`

### `blt-chat-history` — Chat history durable par idProject (IndexedDB client + db drizzle server)

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : S · **statut** : planned
- **what** : L'historique de chat/reponse est persisté en écran-local (db.ts avec IndexedDB + drizzle) par projet.
- **how** : app/lib/persistence/db.ts + useChatHistory.ts. Omega : MemoryStore couvert.
- **adaptateur Omega** : omega-app::FileMemoryStore (chat idProject)
- **preuves** : `app/lib/persistence/db.ts` · `app/lib/persistence/useChatHistory.ts`

### `blt-switchable-stream` — SwitchableStream : multiplexer les streams LLM en un seul flux

- **domaine** : routage · **capacite** : router-policy · **port** : Selector · **cout** : S · **statut** : planned
- **what** : Un TransformStream qui laisse switcher de reader en cours de route : quand l'assistant passe d'un provider/etape a l'autre, le flux theo reste ininterrompu pour le client.
- **how** : app/lib/.server/llm/switchable-stream.ts:1-20 class SwitchableStream extends TransformStream avec _currentReader + switchSource. Omega : omega-router (multiplexage de streams de providers).
- **adaptateur Omega** : omega-router (stream switchable multi-provider)
- **preuves** : `app/lib/.server/llm/switchable-stream.ts:1-20`

### `blt-editor-codemirror` — Editor CodeMirror v6 dans le workbench : voir les fichiers generations en direct

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : S · **statut** : planned
- **what** : Un editor CodeMirror v6 integre au workbench, ou l'utilisateur voit/modifie les fichiers generes pendant l'agent tourne.
- **how** : app/components/editor/codemirror/BinaryContent.tsx + cm-theme.ts (Compartment/EditorView). Omega : omega-ui couvert.
- **adaptateur Omega** : omega-ui (editor integre au workbench)
- **preuves** : `app/components/editor/codemirror/cm-theme.ts:1-2` · `app/components/editor/codemirror/BinaryContent.tsx:1`

