# Dissection — Onlook (Onlook, Apache-2.0, TypeScript)

> Source : clone `--depth 1` de https://github.com/onlook-dev/onlook dans `/home/tayk0oo/omega-re/onlook`.
> Donnees : `omega/crates/omega-harness/data/apps/onlook.json` (14 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Onlook |
| Langage | TypeScript |
| Repo | https://github.com/onlook-dev/onlook |
| Architecture | Stack UI agentic (monorepo Bun, packages/* + apps/*) : canvas multi-frames injectant un preload dans l'app cible (API DOM style/insert/move/drag/texte/theme/screenshot), un EditorEngine MobX a 25 managers par branche, re-ancrage AST (babylon) des edits DOM dans les sources JSX, fournisseurs de sandbox pluggables (CodeSandbox local, E2B, Daytona, Vercel, Modal, NodeFs) synchronises bidirectionnellement, agents chat (ask/edit/create) avec toolset filtre par mode, fast-apply morph/relace, OpenRouter seul provider LLM, tRPC+Postgres, hosting vercel preview/custom. |

## 2. Inventaire fonctionnel

### `onl-editor-engine` — EditorEngine : 25 managers coordonnes par projet (MobX)

- **domaine** : agents · **capacite** : (execution) · **port** : — · **cout** : S · **statut** : rejected
- **what** : Un point d'entree unique qui compose tous les sous-systemes de l'editeur visuel (canvas, elements, overlay, insert, move, style, ast, code, chat, git, sandbox, frames, pages, theme, font, screenshot...) autour d'un projectId, avec etat observable.
- **how** : class EditorEngine (apps/web/client/src/components/store/editor/engine.ts:30) declare 25 managers en champs readonly, chacun recevant editorEngine pour le wiring croise ; les getters deleguent les ressources actives a la branche courante via this.branches.activeSandbox / activeHistory / activeCodeEditor (engine.ts:36-45) ; makeAutoObservable(this) (engine.ts:75) rend le tout reactif. Omega : omega-app::App compose deja des managers, mais pas de sous-managers branches par contexte : pattern utile pour l'association par run/branche.
- **adaptateur Omega** : omega-app::App (composition fine des sous-managers)
- **preuves** : `apps/web/client/src/components/store/editor/engine.ts:36-45` · `apps/web/client/src/components/store/editor/engine.ts:75`

### `onl-preload-dom` — Script preload injecte : surface d'edition DOM bidirectionnelle

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel (surface DOM d'edition de la frame) · **cout** : L · **statut** : planned
- **what** : L'app editee recoit un script injecte qui expose depuis le canvas une API complete : traitement DOM, getElementAtLoc/getElementByDomId, insert/remove/move, drag (normal et absolu), updateStyle, edition de texte editable, theme, screenshot, etat lie a frameId/branchId.
- **how** : apps/web/preload/script/api/index.ts:1-35 exporte l'API ; l'etat est lie aux ids (apps/web/preload/script/api/state.ts). Omega : omega-ui n'a pas de couce d'outillage DOM vers une app cible : separer l'outillage de la frame editeee (comme plugin preload) est le pattern a retenir pour un inspecteur runtime.
- **adaptateur Omega** : omega-server (surface d'inspection vers la frame editee)
- **preuves** : `apps/web/preload/script/api/index.ts:1-35` · `apps/web/preload/script/api/state.ts`

### `onl-ast-reanchor` — Re-ancrage AST : DOM edite -> fichiers source JSX (ids poses dans le DOM)

- **domaine** : execution · **capacite** : tool-surface · **port** : ToolExecutor (participant lecture/écriture) · **cout** : L · **statut** : planned
- **what** : Chaque modification faite dans la preview est re-appliquee dans les fichiers sources via un AST JSX : les elements DOM portent des ids Onlook, la modification est localisee par l'AST, serialize puis reformate (prettier).
- **how** : packages/parser/src/parse.ts:5-22 parse le JSX (babylon, plugins typescript/jsx/decorators) ; removeIdsFromAst retire les ids Onlook a la serialisation ; packages/parser/src/prettier/index.ts:4-25 formate les fichiers Next.js ; le rattachement depuis l'EditorEngine passe par l'AstManager. Omega : aucun edit-source par motif DOM<->fichier : ecart principal pour un plugin editeur visuel.
- **adaptateur Omega** : plugin/source-reanchor (parse JSX, re-ancrer, re-formater)
- **preuves** : `packages/parser/src/parse.ts:5-22` · `packages/parser/src/prettier/index.ts:4-25`

### `onl-sandbox-providers` — Fournisseurs de sandbox pluggables (6 fournisseurs de preview)

- **domaine** : execution · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : M · **statut** : planned
- **what** : Le code utilisateur tourne dans un provider de sandbox interchangeable : CodeSandbox (local), E2B, Daytona, Vercel Sandbox, Modal, NodeFs.
- **how** : enum CodeProvider (packages/code-provider/src/providers.ts:3-10) ; dossiers implementations packages/code-provider/src/providers/codesandbox + providers/nodefs ; le SandboxManager branche le provider et declenche la synchro bidirectionnelle via CodeProviderSync (apps/web/client/src/components/store/editor/sandbox/index.ts:1-36). Omega : omega-ports::SandboxBackend existe ; pattern a remonter : cataloguer des providers cloud (Daytona/Modal/Vercel) pour preview web.
- **adaptateur Omega** : omega-sandbox (adaptateurs de providers sandbox cloud)
- **preuves** : `packages/code-provider/src/providers.ts:3-10` · `apps/web/client/src/components/store/editor/sandbox/index.ts:1-36`

### `onl-sync-engine` — Sync engine bidirectionnel FS local <-> sandbox fournisseur

- **domaine** : execution · **capacite** : sandbox-backend · **port** : SandboxBackend (watch/pull/push) · **cout** : M · **statut** : planned
- **what** : Service de synchronisation des fichiers dans les deux sens : watcher du cote provider et diff/pull du cote local, avec debouncing et tolerance aux cas limites (dossier parent cree par le watcher de la sandbox).
- **how** : class CodeProviderSync (apps/web/client/src/services/sync-engine/sync-engine.ts:32) avec ProviderFileWatcher ; arret propre du watcher (sync-engine.ts:171-173) ; provider.watchFiles (sync-engine.ts:363) + surveillance du FS local pour la sync bidirectionnelle (sync-engine.ts:581-583) ; cas limite de creation de dossier parent (sync-engine.ts:455). Omega : un SyncEngine par adaptateur epsilon outils de confinement avant commit.
- **adaptateur Omega** : omega-sandbox::SyncEngine (adaptateur par provider)
- **preuves** : `apps/web/client/src/services/sync-engine/sync-engine.ts:32-50` · `apps/web/client/src/services/sync-engine/sync-engine.ts:581-583`

### `onl-branch-sandbox` — Branches de preview : chaque branche = son sandbox + son git

- **domaine** : execution · **capacite** : sandbox-backend · **port** : SandboxBackend (par branche) · **cout** : M · **statut** : planned
- **what** : L'editeur gere des branches paralleles : chaque branche a son propre sandbox (session), son propre historique git et son propre editeur de fichiers ; la bascule active se fait par selection de frame.
- **how** : class BranchManager (apps/web/client/src/components/store/editor/branch/manager.ts:20) tient un BranchData par id ; le suivi selectedFrames (manager.ts:69-78) bascule currentBranchId sur activeFrame.frame.branchId ; activeSandbox/activeHistory re-deroutent vers la branche active via les getters de engine.ts:36-45. Omega : omega-sandbox rattache deja un worktree a chaque branche ; pattern de bascule par id de branch active transposable.
- **adaptateur Omega** : omega-sandbox (session par branche) + omega-app (selection active par run)
- **preuves** : `apps/web/client/src/components/store/editor/branch/manager.ts:69-78` · `apps/web/client/src/components/store/editor/engine.ts:36-45`

### `onl-chat-toolset` — Toolset chat filtre par mode (ask = lecture seule, edit = lecture+ecriture)

- **domaine** : agents · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : adopted
- **what** : Les outils exposés au modele changent selon le mode de chat : ask n'a que la lecture (12 outils), edit ajoute les edits/terminal/sandbox ; a l'execution, un appel d'outil edit en mode ask est refuse avec message guide.
- **how** : readOnlyToolClasses + editOnlyToolClasses (packages/ai/src/tools/toolset.ts:36-46) ; getToolClassesFromType(chatType) (toolset.ts:60-63) ; handleToolCall revalide a l'execution et toast 'Switch to build mode' (apps/web/client/src/components/tools/tools.ts:13-20). Omega : ExecPolicy interdit deja ; la garde positive (allowlist par mode de session) est le complement utile.
- **adaptateur Omega** : omega-ports::ExecPolicy (allowlist par mode de session)
- **preuves** : `packages/ai/src/tools/toolset.ts:36-63` · `apps/web/client/src/components/tools/tools.ts:13-20`

### `onl-root-agent` — Agent chat racine : streamText avec reparation d'outils + telemetrie traçable

- **domaine** : agents · **capacite** : agent-loop · **port** : — · **cout** : M · **statut** : planned
- **what** : Boucle d'agent AI SDK avec arret budgete (20 pas max), reparation des appels d'outils rates (NoSuchToolError), streaming lisse, et telemetrie Langfuse chargee d'identifiants traçables.
- **how** : createRootAgentStream (packages/ai/src/agents/root.ts:15) streamText({stopWhen: stepCountIs(20), experimental_repairToolCall: repairToolCall, experimental_transform: smoothStream()}) (root.ts:26-34) ; experimental_telemetry attend langfuseTraceId + sessionId=conversationId (root.ts:35-45) ; system prompt choise par ChatType (root.ts:49-58). Omega : omega-core::AgentLoop + omega-ports::EventStore couvrent la structure ; a ajouter : reparation d'outil rate et budget de pas.
- **adaptateur Omega** : omega-core::AgentLoop (repair fallback + budget de pas)
- **preuves** : `packages/ai/src/agents/root.ts:15-45`

### `onl-fast-apply` — Routage fast-apply : snippet reconcilie par un modele specialise (morph/relace)

- **domaine** : routage · **capacite** : router-policy · **port** : Selector · **cout** : S · **statut** : planned
- **what** : La modification de code proposee par le modele principal est reconciliee au fichier cible par un modele de fast-apply dedie (morph ou relace) avant application.
- **how** : packages/ai/src/apply/client.ts:8-12 enum FastApplyProvider {MORPH, RELACE} ; applyCodeChangeWithMorph (client.ts:25-40) poste originalCode+updateSnippet+instruction sur api.morphllm.com. Omega : omega-router::Selector+FallbackPolicy : un tour dedie specialise par tache d'application est le pattern a ajouter.
- **adaptateur Omega** : omega-router (etape dediee fast-apply, modele specialise)
- **preuves** : `packages/ai/src/apply/client.ts:8-12` · `packages/ai/src/apply/client.ts:25-40`

### `onl-llm-openrouter` — Providers LLM : un seul fournisseur (OpenRouter) + cache anthropique de session

- **domaine** : routage · **capacite** : llm-provider · **port** : Provider · **cout** : S · **statut** : rejected
- **what** : Toutes les inferences passent par OpenRouter ; les modeles Claude recoivent un cache de prompt ephemere ; les autres cas sont refuses a la compilation.
- **how** : initModel (packages/ai/src/chat/providers.ts:12-48) switch sur LLMProvider (un seul cas OPENROUTER, assertNever sinon) ; getOpenRouterProvider (providers.ts:51-53) exige OPENROUTER_API_KEY ; providerOptions.anthropic.cacheControl={type:'ephemeral'} applique sur les modeles Claude (providers.ts:33-35). Omega : opencore::Selector+FallbackPolicy est deja plus riche ; l'ajout utile : cache de prompt par segment (breakpoints).
- **adaptateur Omega** : omega-router (cache de prompt anthropique optionnel)
- **preuves** : `packages/ai/src/chat/providers.ts:12-48`

### `onl-frames-canvas` — Canvas multi-frames : poses antipode, redimensionnement, navigation

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : M · **statut** : planned
- **what** : Plusieurs vues iframe (par branche/resolution) posees sur un canvas libre, repositionnees sans chevauchement, avec navigation et multi-selection.
- **how** : FramesManager (apps/web/client/src/components/store/editor/frames/manager.ts:22) tient frameId -> FrameData (frame + view + selected) ; calculateNonOverlappingPosition (manager.ts:6) repousse les frames superposees ; roundDimensions (dimension.ts) garde des dims propres ; FrameNavigationManager (navigation.ts) gere l'historique d'URL. Omega : passe a l'observatoire multi-runs.
- **adaptateur Omega** : omega-server (panneau canvas multi-run)
- **preuves** : `apps/web/client/src/components/store/editor/frames/manager.ts:16-30`

### `onl-hosting-deploy` — Hosting : deploiement preview/custom (vercel) + depublication par projet

- **domaine** : automatisation · **capacite** : scheduler-job · **port** : — · **cout** : M · **statut** : planned
- **what** : Provider d'hosting unifie preview et domaine custom par projet : deploiement/annulation via API dediee, etat garde par type de deploiement.
- **how** : HostingProvider (apps/web/client/src/components/store/hosting/provider.tsx:20-26) tient deployments: Record<DeploymentType, Deployment|null> ; requetes trpc api.publish.deployment.{getByType}.useQuery + create.useMutation (provider.tsx:50-80) ; unpublish retourne deploymentId (provider.tsx:26). Omega : omega-app::App (job de deploiement) + omega-server (API de revue) : planificateur de deploiements.
- **adaptateur Omega** : omega-app (job de deploiement) + omega-server (API de revue)
- **preuves** : `apps/web/client/src/components/store/hosting/provider.tsx:20-26` · `apps/web/client/src/components/store/hosting/provider.tsx:79-80`

### `onl-i18n` — i18n du cockpit (messages par locale, provider react)

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Interface de l'app utilisateur (et messages d'outils) traduite par locale via un provider central.
- **how** : apps/web/client/messages/ (fichiers par locale) + apps/web/client/src/i18n/ (provider). Omega : l'i18n utilisateur se traite au bord UI si on l'ouvre au public.
- **adaptateur Omega** : omega-app (dictionnaire de messages charge)
- **preuves** : `apps/web/client/messages` · `apps/web/client/src/i18n`

### `onl-feature-flags` — Feature flags d'evaluation runtime (hooks react)

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Fonctionnalite drapeaux capable d'activer/masquer des surfaces sans release.
- **how** : apps/web/client/src/hooks/use-feature-flags.tsx consomme PostHog. Omega : ConfigSource avec flag runtime ; côté Omega la logique de flag se branche au niveau du manifeste (enabled_if) : les plugins se desactivent sans redeployer.
- **adaptateur Omega** : omega-config (enabled_if manifeste)
- **preuves** : `apps/web/client/src/hooks/use-feature-flags.tsx`

