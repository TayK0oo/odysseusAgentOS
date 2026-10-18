# Dissection — Dyad (Dyad, MIT, TypeScript)

> Source : clone `--depth 1` de https://github.com/dyad-sh/dyad dans `/home/tayk0oo/omega-re/dyad`.
> Donnees : `omega/crates/omega-harness/data/apps/dyad.json` (14 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Dyad |
| Langage | TypeScript |
| Repo | https://github.com/dyad-sh/dyad |
| Architecture | Application Electron pour construire des apps web par chat : monorepo vite (main/preload/renderer + workers), React renderer MobX, 121 handlers IPC partitionnes par domaine, machine d'etat app_run pure (transition), parser incremental de tags streamants (dyad-write...), compaction de contexte sur seuil, 121 handlers avec deploiements (vercel/cloudflare/coolify/supabase/neon), provider par credentials (envVarName), client MCP, evals e2e. |

## 2. Inventaire fonctionnel

### `dyd-app-run-machine` — Machine d'etat app_run : start/restart/rebuild avec commandes et URL LATENTE

- **domaine** : automatisation · **capacite** : event-store · **port** : EventStore · **cout** : S · **statut** : adopted
- **what** : Une machine d'etat pure (transition, pas de side-effect) declenche un run d'application : START/RESTART/REBUILD emettent une commande 'start' avec options (removeNodeModules, recreateSandbox), et l'URL arrive plus tard (pendingUrl).
- **how** : export function transition(state: RunEvent)->TransitionResult (src/app_run/transition.ts:48) : chaque event retourne kind=applied + nouveau state + commands[] (transition.ts:59-84) ; resolveAppRunInvocationRef (definition.ts:279) nomme l'invocation ; AppRunRemoteManager (remote_manager.ts:67) gere la partie distante. Omega : omega-core (matrice evenements + effets reversibles) : la transition pure testable est exactement notre garde.
- **adaptateur Omega** : omega-core::AgentLoop (machine d'etat du run pure)
- **preuves** : `src/app_run/transition.ts:48-84` · `src/app_run/definition.ts:279`

### `dyd-stream-parser` — Parser de tags streamant incrementaux (dyad-write/rename/delete/...) avec blocs stables

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel (rendu streamant event-sourced) · **cout** : M · **statut** : planned
- **what** : Le messages de l'agent est parse au fil de l'eau : des tags custom (dyad-write, dyad-rename, dyad-delete, dyad-add-dependency...) produisent des blocs stables referentiellement, les octets en attente sont rendus comme texte markdown temporaire.
- **how** : src/lib/streamingMessageParser.ts:90-125 types Block/Mode/ParserState ; DYAD_CUSTOM_TAG_NAMES (streamingMessageParser.ts:26-31) ; le header committe les Block avec identite preservee pour React.memo (streamingMessageParser.ts:5-9). Omega : l'event stream parse par token : le pattern incremental-idempotent d'un parser streamant est utile a omega-server (reponse event-sourced).
- **adaptateur Omega** : omega-server (parser de tags streamant)
- **preuves** : `src/lib/streamingMessageParser.ts:5-31`

### `dyd-streaming-patch` — Patch de stream : tail-only splice avec hash de prefixe (stale dectection)

- **domaine** : execution · **capacite** : event-store · **port** : EventStore · **cout** : S · **statut** : planned
- **what** : Le contenu streamant reconverge avec un retour d'offset + hash : un 'tail-only patch' est rejete si le prefixe local differe (detremination de l'obsolescence DB).
- **how** : applyStreamingPatch (src/lib/applyStreamingPatch.ts:7-18) reconstruit current.slice(0, offset)+content ; renvoie false si hash djb2 du prefixe (prefixHash.ts) differe ; le caller resynchronise (applyStreamingPatch.ts:19-25). Omega : le EventStore peut rejouer identiquement ; le pattern hash-prefixe est utile pour les r-sens de la console.
- **adaptateur Omega** : omega-server (splice de stream verifiale par hash)
- **preuves** : `src/lib/applyStreamingPatch.ts:7-25`

### `dyd-compaction-handler` — Compaction de contexte declenchee sur seuil depense, par modele auxiliaire

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : M · **statut** : adopted
- **what** : Compaction orchestrée : quand la convergation depasse le seuil du contexte, une passe de compaction est lancee sur un mode auxiliaire, une transcript pre-compaction est stockee, la passé post-compaction replaçe les messages.
- **how** : src/ipc/handlers/compaction/compaction_handler.ts orchestre le streamText (compaction_handler.ts:1-50) ; shouldTriggerCompaction (src/ipc/utils/token_utils.ts:112-117) : totalTokens >= getCompactionThreshold(contextWindow, provider) (token_utils.ts:100-117) ; storePreCompactionMessages + formatAsTranscript + buildCompactionBlock (compaction_storage.ts / compaction_utils.ts). Omega : LlmCompactionEngine double couche couvert ; pattern respecter : seuil par provider et model auxiliaire distincte.
- **adaptateur Omega** : omega-context::LlmCompactionEngine (seuil par provider, model auxiliaire)
- **preuves** : `src/ipc/handlers/compaction/compaction_handler.ts:1-50` · `src/ipc/utils/token_utils.ts:100-117`

### `dyd-providers-env` — Providers LLM par credentials : env vars declared par provider, vertex/azure speciaux

- **domaine** : routage · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Les providers sont verifies par credentials : presets avec envVarName, avec cas speciaux vertex (project) et azure (url) ; les customs * ne s'appliquent que si la cle dispo.
- **how** : src/lib/providerUtils.ts:22-48 : isProviderSetUp(provider) lit settings.providerSettings + envVars + providerData (providerUtils.ts:18-25) ; vertex gestion par projectId (providerUtils.ts:34-46) ; azure par baseUrl (providerUtils.ts:47-48). Omega : omega-config (refs de secrets) : le pattern envVarName par provider est le imaginaire du CredentialProvider de la couture deepseek.
- **adaptateur Omega** : omega-config (refs de secrets par provider)
- **preuves** : `src/lib/providerUtils.ts:18-48`

### `dyd-mcp-handlers` — Client MCP integre (ipc/handlers/mcp) + classifiers d'erroe par serveur

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : M · **statut** : planned
- **what** : Interfacing unifiee MCP : configuration serveurs, tools, erreurs classifies pour un message utilisateur lisible.
- **how** : src/ipc/handlers/mcp_handlers.ts + mcp_error_classifiers.ts (l'erreur MCP est classifiee par type pour une UX claire). Omega : mcp:rea est plugin ; la classifier d'erreur est du bruit pour le plugin surface.
- **adaptateur Omega** : omega-context::ToolSurfaceIndex (catalogue MCP + erreur classifiee)
- **preuves** : `src/ipc/handlers/mcp_handlers.ts` · `src/ipc/handlers/mcp_error_classifiers.ts`

### `dyd-cloud-deploys` — Support de deploiements : cloudflare, coolify, vercel, supabase, neon

- **domaine** : automatisation · **capacite** : scheduler-job · **port** : — · **cout** : M · **statut** : planned
- **what** : L'app construite peut se deployer vers plusieurs fournisseurs (cloudflare pages, coolify autoge, vercel, neon db, supabase), avec approche unifiee par handlers dedies.
- **how** : src/ipc/handlers/{cloudflare,coolify,vercel,supabase,neon}_handlers.ts ; le systeme de deployement est unifiee par uses des adapters. Omega : omega-app::App job de deploiement + coordinator de la surface.
- **adaptateur Omega** : omega-app (jobs de deploiement par cible)
- **preuves** : `src/ipc/handlers/cloudflare_handlers.ts` · `src/ipc/handlers/coolify_handlers.ts`

### `dyd-problems` — Problems/fallback : l'agent reçoit les diagnostics IDE (TS, lint) en retour

- **domaine** : observabilite · **capacite** : observabilite · **port** : TelemetrySink · **cout** : S · **statut** : planned
- **what** : Le loop de chat recupere les erreurs IDE a l'execution et les renvoie comme problemes a l'agent.
- **how** : src/ipc/handlers/problems_handlers.ts + src/main/preview_cdp_broker.ts (les console logs et problemes via CDP). Omega : l'EventStore recoit deja les problemes ; pattern utile : renvoyer vers l'agent au tour suivant.
- **adaptateur Omega** : omega-app (diagnostics recircules a l'agent)
- **preuves** : `src/ipc/handlers/problems_handlers.ts` · `src/main/preview_cdp_broker.ts`

### `dyd-conversation-summary` — Resumé de conversation persiste (compaction_storage).

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : S · **statut** : planned
- **what** : Le historique de la conversation est applate en transcript lisible avant de servir a la compaction/explication.
- **how** : src/ipc/handlers/compaction/compaction_storage.ts : storePreCompactionMessages + formatAsTranscript. Omega : re-decoupage omega-memory (summarisation transcript persiste) ; utile pour la mémoire d'inférieur de durée.
- **adaptateur Omega** : omega-app (transcript de resume persiste)
- **preuves** : `src/ipc/handlers/compaction/compaction_storage.ts`

### `dyd-free-quota` — Quota d'usage gratuit : budget de requetes gated par modele

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : rejected
- **what** : Un budget de l'utilisation du model gratuit est enforce par requete (freeProModel, free_agent_quota).
- **how** : src/lib/free_pro_model.ts + src/ipc/handlers/free_agent_quota_handlers.ts + chat_stream_free_subscription. Omega : pas un kernel feature ; la note du budget peut se locker via ExecPolicy si un usage est stocké.
- **adaptateur Omega** : omega-ports::ExecPolicy (budget de usage par modele)
- **preuves** : `src/lib/freeProModel.ts` · `src/ipc/handlers/free_agent_quota_handlers.ts`

### `dyd-chat-modes` — Modes de chat : build / ask / plan avec precondition avant raisonnement

- **domaine** : agents · **capacite** : agent-store · **port** : SubagentRunner · **cout** : S · **statut** : planned
- **what** : Le chat mode distingue build (edite), ask (repond), plan (produit un plan), chaque reset du mode impose une precondition a la chaîne.
- **how** : src/lib/chatMode.ts + src/ipc/handlers/chat_mode_resolution.ts + chat_mode_precondition.test.ts (resolution à la chaîne + precondition de mode avant propagation). Omega : SessionType d'omega-app a session type de mode ask/edit/create ; ajouter la precondition chainable.
- **adaptateur Omega** : omega-app (session par mode + precondition chain)
- **preuves** : `src/lib/chatMode.ts` · `src/ipc/handlers/chat_mode_resolution.ts`

### `dyd-pro-subscription` — Routage par souscription : modeles Pro vs relay Dyad, effort par modele

- **domaine** : routage · **capacite** : router-policy · **port** : Selector · **cout** : S · **statut** : planned
- **what** : Un petit plan de souscription relaye les demandes vers les cibles (pro) ; l'effort de raisonnement (reasoning effort) est parametrisable par modele.
- **how** : src/ipc/utils/model_effort.ts + resolveModelSelection/normalizeModelSelection + src/lib/modelEffort.ts (clé de préférence). Omega : omega-router Selector ; l'eclat ici : la selection effort de raisonnement par default depuis le mode.
- **adaptateur Omega** : omega-router (effort de raisonnement par tache)
- **preuves** : `src/ipc/utils/model_effort.ts` · `src/lib/modelEffort.ts`

### `dyd-slug-encoders` — Chiffrement des clées API via safe-storage electron (legacy safe)

- **domaine** : securite · **capacite** : config-source · **port** : — · **cout** : S · **statut** : rejected
- **what** : Les clée API stockee en base sont chiffre par la tombe systeme, backup/restore inclus.
- **how** : src/main/safe_storage_legacy.ts (lecture des credential legacy chiffrees) + providerApiKey.ts. Omega : omega-config (refs de secrets) : chiffrement par l'OS du front (HEAD).
- **adaptateur Omega** : omega-config (refs de secrets chiffrees)
- **preuves** : `src/main/safe_storage_legacy.ts`

### `dyd-eval-harness` — Benchmark d'agent : une suite d'eval de programme end-to-end (evals)

- **domaine** : observabilite · **capacite** : tool-surface · **port** : ToolSurface · **cout** : M · **statut** : planned
- **what** : Une infrastructure de tests de qualité au-dessus des evals de fukcts end-to-end de génération de code avec de tests de conformance transverses.
- **how** : src/__tests__/evals/ + get_eval_model.ts ; vitest.eval.config.ts. Omega : une üste d'evals corrée ; utile a l'omega-harness crate pour des tests de regression des capacités.
- **adaptateur Omega** : omega-harness (buat de regression de capacite)
- **preuves** : `src/__tests__/evals` · `vitest.eval.config.ts`

