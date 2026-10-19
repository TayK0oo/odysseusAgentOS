# Dissection — Roo Code (Roo Code, Apache-2.0, TypeScript)

> Source : clone `--depth 1` de https://github.com/RooCodeInc/Roo-Code dans `/home/tayk0oo/omega-re/roo-code`.
> Donnees : `omega/crates/omega-harness/data/apps/roo-code.json` (14 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Roo Code |
| Langage | TypeScript |
| Repo | https://github.com/RooCodeInc/Roo-Code |
| Architecture | Extension VS Code (monorepo pnpm : src + packages/core|ipc|types|vscode-shim), successeur de Cline : boucle Task recursive avec 20+ outils BaseTool, modes orthogonaux definissables (.roomodes, hot reload), checkpoints shadow-git par task, auto-approbation allowlist/denylist, index de code semantique configurable (embedders injectes), client MCP integre, surface IPC programmatique, 30+ providers factorises sur une base compat OpenAI. |

## 2. Inventaire fonctionnel

### `roo-task-loop` — Task : boucle agent recursive avec messages streames et timeline

- **domaine** : agents · **capacite** : agent-loop · **port** : — · **cout** : S · **statut** : rejected
- **what** : Boucle principale d'un task (extends EventEmitter) : requetes API recursives vers le modele, acquisition/desact de la reponse assistant, outils en streaming, reprise apres interruption.
- **how** : export class Task extends EventEmitter<TaskEvents> (src/core/task/Task.ts:158) ; recursivelyMakeClineRequests (Task.ts:2461) enchaine requete -> parse assistant-message -> execution outils -> recurs ; persistence via task-persistence. Omega : omega-core::AgentLoop couvre ; pattern a ajouter : EventEmitter de TaskEvents internes.
- **adaptateur Omega** : omega-core::AgentLoop (events internes taches)
- **preuves** : `src/core/task/Task.ts:158` · `src/core/task/Task.ts:2461`

### `roo-repetition-guard` — ToolRepetitionDetector : garde de repetition d'appel d'outil

- **domaine** : execution · **capacite** : agent-loop · **port** : — · **cout** : S · **statut** : planned
- **what** : Detecte N appels identiques consecutifs (defaut 3) au meme outil avec le meme input et coupe/escalade a l'utilisateur.
- **how** : export class ToolRepetitionDetector (src/core/tools/ToolRepetitionDetector.ts:9) ; previousToolCallJson + compteur consecutiveIdenticalToolCallCount ; check() serialize le ToolUse en JSON canonique et compare (ToolRepetitionDetector.ts:31-40) ; limite injectable au constructeur (defaut 3, ToolRepetitionDetector.ts:15). Omega : omega-core::AgentLoop n'a pas de garde de repetition ; meme famille que gemini-cli loop-guard.
- **adaptateur Omega** : omega-core::AgentLoop (garde de repetition en serie)
- **preuves** : `src/core/tools/ToolRepetitionDetector.ts:9-40`

### `roo-allowlist-commands` — Allowlist/denylist de commandes shell avec wildcards et match le plus long

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : L'auto-approbation des commandes shell suit une garde positive : allowlist explicite, denylist peut toujours bloquer, wildcard '*' permitif, et match le plus long gagne.
- **how** : src/core/auto-approval/commands.ts:118-139 : une commande n'est auto-approuvee que si elle matche l'allowlist explicite ET ne matche pas la denylist (ou matche une entree allowlist plus longue) ; le wildcard '*' dans l'allowlist permet tout, la denylist peut quand meme bloquer (commands.ts:139) ; charge par AutoApprovalHandler (AutoApprovalHandler.ts:13). Omega : ExecPolicy actuel n'a que le set statique ; le pattern allow/deny avec match le plus long est utile.
- **adaptateur Omega** : omega-core (exec-policy allow/deny, le plus long gagne)
- **preuves** : `src/core/auto-approval/commands.ts:118-139`

### `roo-validate-tool-use` — validateToolUse : nom connu + permis par mode, avant execution

- **domaine** : securite · **capacite** : permission-policy · **port** : PermissionPolicy · **cout** : S · **statut** : rejected
- **what** : Un garde unique d'un appel d'outil avant execution : nom valide, outil permis pour le mode courant, exigences par outil.
- **how** : src/core/tools/validateToolUse.ts:32-60 : isValidToolName rejette les outils inconnus (validateToolUse.ts:38-41) ; isToolAllowedForMode (validateToolUse.ts:120) croise mode + customModes + toolRequirements + params. Omega : ExecPolicy + PermissionPolicy couvrent ; le mode est ici la cle d'organisation (croise avec custom-modes).
- **adaptateur Omega** : omega-core (validate tool-use avant execution)
- **preuves** : `src/core/tools/validateToolUse.ts:32-60`

### `roo-custom-modes` — Modes composable utilisateur avec prompts et outils par delta

- **domaine** : agents · **capacite** : agent-store · **port** : SubagentRunner · **cout** : M · **statut** : planned
- **what** : Un systeme orthogonal de modes (architect, code, debug...) dont chacun possede son role, ses prompts et ses outils requis ; defini par l'utilisateur (.roomodes a hot reload).
- **how** : src/core/config/CustomModesManager.ts:19 ROOMODES_FILENAME='.roomodes' chargee depuis le workspace root (CustomModesManager.ts:101) ; le schema modeConfigSchema (@roo-code/types) tient roleDefinition, groupes d'outils, promptComponent (CustomModesManager.ts:9) ; config import/exportable (src/core/config/importExport.ts). Omega : omega-app::App agents ; l'ecart : definition de mode par dossier workspace avec hot reload
- **adaptateur Omega** : omega-app (modes definis par dossier workspace)
- **preuves** : `src/core/config/CustomModesManager.ts:19-101`

### `roo-checkpoints` — Checkpoints shadow-git : snapshot du workspace par etape taske

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : L · **statut** : planned
- **what** : Avant chaque modification, un snapshot de l'etat de l'espace de travail (shadow git) permet de restituer/remonte a n'importe quelle etape.
- **how** : src/services/checkpoints/RepoPerTaskCheckpointService (RepoPerTaskCheckpointService.ts:6) extends ShadowCheckpointService (ShadowCheckpointService.ts:79) = git cache par task ; restore passe par le webview handler (src/core/webview/checkpointRestoreHandler.ts). Omega : omega-core a des effets reversibles dans le registre mais pas de snapshot FS par tache.
- **adaptateur Omega** : omega-sandbox (checkpoints shadow-git)
- **preuves** : `src/services/checkpoints/ShadowCheckpointService.ts:79` · `src/services/checkpoints/RepoPerTaskCheckpointService.ts:6`

### `roo-condense-folded` — Condense fichier : contexte de fichier en signatures only

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : S · **statut** : planned
- **what** : Economie de contexte : le contenu d'un fichier est replie pour ne garder que les signatures (functions/classes) avec des renvois numeriques de ligne.
- **how** : src/core/condense/foldedFileContext.ts:48-76 : generateFoldedFileContext decrit chaque bloc de signature par `1--15 | export function formatDate(...)` (foldedFileContext.ts:67-68). Omega : le port ContextEngine couvre la compaction chez provider ; le pattern FoldedFileContext est un format disjoint (fold-map numerique) transposable.
- **adaptateur Omega** : omega-context (folded file context)
- **preuves** : `src/core/condense/foldedFileContext.ts:1-76`

### `roo-providers` — Registre de 30+ providers composes sur une base compat OpenAI

- **domaine** : routage · **capacite** : llm-provider · **port** : Provider · **cout** : S · **statut** : rejected
- **what** : Registre de providers d'entree : anthropic, bedrock, vertex, openrouter, openai-native, deepseek, fireworks, gemini, poe, unbound, ollama local, VSCode LM, vercel AI gateway, etc.
- **how** : src/api/providers/*.ts ; base-openai-compatible-provider.ts factorise toutes les familles de providers compatibles OpenAI (base unique+) ; router-provider.ts en couche de composition. Omega : omega-router + omega-providers couvrent deja un registre multiple ; le pattern a refactoriser est la base communitaire compatible OpenAI par derive.
- **adaptateur Omega** : omega-providers (base commune pour compat OpenAI)
- **preuves** : `src/api/providers/base-openai-compatible-provider.ts` · `src/api/providers/index.ts`

### `roo-mentions` — Mentions unifiees : @fichier, @terminal, @image, @url resolues dans le message

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : S · **statut** : planned
- **what** : Un message utilisateur peut citer @fichier, dossier, @screen terminal, image, url ou probleme IDE ; chaque mention est resolue en contenu prompt avant envoi.
- **how** : src/core/mentions/processUserContentMentions.ts + resolveImageMentions.ts decouvrent et remplacent chaque mention en un fragment exploitable. Omega : utile pour un resolver de data-sources (mention=data source) sur ContextEngine.
- **adaptateur Omega** : omega-context (resolver de mentions)
- **preuves** : `src/core/mentions/processUserContentMentions.ts`

### `roo-code-index` — Index de code semantique : embedders + orchestrator + search + cache

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : M · **statut** : planned
- **what** : Index semantique du code : embedders faciles a changer (openai/ollama/gemini/voyage), parseur tree-sitter, cache persistant, service de recherche hybride.
- **how** : src/services/code-index : CodeIndexManager (manager.ts:17), CodeIndexOrchestrator (orchestrator.ts:13), CodeIndexSearchService (search-service.ts:11), CodeIndexServiceFactory (service-factory.ts:31) injecte les dependances ; CacheManager (cache-manager.ts:10) ; embedders dans interfaces/. Omega : omega-context a ToolSurface BM25 ; l'ecart : index semantique par embeddings configurables.
- **adaptateur Omega** : omega-context (index semantique par embedder)
- **preuves** : `src/services/code-index/manager.ts:17-31` · `src/services/code-index/service-factory.ts:29-31`

### `roo-skills` — Skills invocables par mode : systeme de contenu ad hoc

- **domaine** : contexte · **capacite** : skill-store · **port** : CatalogSource · **cout** : S · **statut** : planned
- **what** : Le tool skill charge et applique un skill du workspace selon le mode courant.
- **how** : src/core/tools/SkillTool.ts:16 class SkillTool extends BaseTool ; resolveSkillContentForMode(skillsManager, skillName, currentMode) (SkillTool.ts:51) ; SkillsManager (src/services/skills/SkillsManager.ts:21). Omega : CatalogSource + ToolSurface hydratent ; l'ecart : le detabpacage de contenu estDONE au niveau ToolSurface.
- **adaptateur Omega** : omega-context (ToolSurface skills par mode)
- **preuves** : `src/core/tools/SkillTool.ts:16-51` · `src/services/skills/SkillsManager.ts:21`

### `roo-mcp` — MCP client integre : tools + resources geres depuis des serveurs distants

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : M · **statut** : planned
- **what** : Client MCP unifie : tools ammenees, resources chargees, auto-approbation par serveur.
- **how** : src/services/mcp + core/tools/UseMcpToolTool.ts / accessMcpResourceTool.ts : chaque serveur MCP declare ses tools qui rejoignent le toolset du modele. Omega : catalogue deja gere par plugin/api ; l'interfacing MCP se couvre au-dessus du ToolSurface.
- **adaptateur Omega** : omega-context::ToolSurfaceIndex (catalogue MCP)
- **preuves** : `src/core/tools/UseMcpToolTool.ts`

### `roo-ipc` — Serveur IPC workspace : surface programmatique de l'extension

- **domaine** : extension · **capacite** : tool · **port** : — · **cout** : M · **statut** : planned
- **what** : Une surface programmatique (node-ipc) expose les capacites de l'extension (demarrage de task, config, etat) a des processus externes.
- **how** : packages/ipc/src/ipc-server.ts + ipc-client.ts echangent via node-ipc (ipc-client.ts:13) ; nom d'instance unique `roo-code-ipc-${hex}` (ipc-client.ts:27). Omega : comparable a un plugin v2 omega : la surface programmatique locale du core (exporte par omega-server).
- **adaptateur Omega** : omega-server (surface programmatique locale)
- **preuves** : `packages/ipc/src/ipc-server.ts` · `packages/ipc/src/ipc-client.ts:13-30`

### `roo-costs` — Tracing de cout : tokens et prix par requete, agreges par task

- **domaine** : observabilite · **capacite** : telemetry-sink · **port** : TelemetrySink · **cout** : S · **statut** : planned
- **what** : Cout mesure pour chaque requete (tokens entree/sortie, prix source), agreges par task et exposes au webview.
- **how** : src/core/webview/aggregateTaskCosts.ts agrege les couts par task ; l'aggregate alimente le compte de run. Omega : omega-otel + EventStore capture deja ; pattern a ajouter : prix de la demande reconstruit par provider.
- **adaptateur Omega** : omega-app (aggregate de couts par run)
- **preuves** : `src/core/webview/aggregateTaskCosts.ts`

