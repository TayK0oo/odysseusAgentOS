# Dissection — Agent Zero (frdel, Apache-2.0, Python)

> Source : clone `--depth 1` de https://github.com/frdel/agent-zero dans `/home/tayk0oo/omega-re/agent-zero`.
> Donnees : `omega/crates/omega-harness/data/apps/agent-zero.json` (13 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | frdel |
| Langage | Python |
| Repo | https://github.com/frdel/agent-zero |
| Architecture | Framework d'agents generaliste (Python) : Agent.monologue boucle extensible par call_extensions, 43 plugins auto-discover (plugin.yaml, config, webui) couvrant memoire, compaction, orchestrateur de CLI externes (skills a la demande), sandbox Docker persistante, tool-access, time-travel, tools call_subordinate et parallel. |

## 2. Inventaire fonctionnel

### `az-agent-monologue` — Boucle monologue : tour de boucle LLM->tools jusqu'au tool response

- **domaine** : agents · **capacite** : agent-loop · **port** : — · **cout** : M · **statut** : planned
- **what** : Le noyau est un agent loop : prepare prompt, stream, extraction de tool request, execution, jusqu'au tool 'response' qui arrete la boucle.
- **how** : class Agent (agent.py:364) monologue() (agent.py:396) : while True -> loop_data -> extensions monologue_start/message_loop_start (agent.py:420-423) -> prepare_prompt -> stream_callback extrait extract_tools.extract_tool_request(full) (agent.py:468-470) ; handle_intervention (agent.py:1097) verifie user interrupt sur chaque chunk. Omega : omega-core::AgentLoop couvert ; l'ecart : intervention utilisateur durante stream.
- **adaptateur Omega** : omega-core::AgentLoop (intervention sur le stream)
- **preuves** : `agent.py:364-396` · `agent.py:468-470`

### `az-extensions` — Extensions extensibles sur tout le cycle de vie (call_extensions)

- **domaine** : extension · **capacite** : hook · **port** : Hook · **cout** : S · **statut** : adopted
- **what** : Toutes les etapes cles (agent_init, monologue_start, message_loop_start, before_main_llm_call, stream chunks...) sont extensibles avec des extensions declarees par plugins.
- **how** : @extension.extensible sur __init__ et monologue (agent.py:372,395) ; call_extensions_sync/async (agent.py:388-423) ; extensions/ + plugins/*/extensions/. Omega : omega-ports::Hook couvert ; le count de points d'extension est plus fin que nos events internes.
- **adaptateur Omega** : omega-ports::Hook (points d'extension du cycle de vie)
- **preuves** : `agent.py:364-396`

### `az-plugins` — 43 plugins auto-discover avec plugin.yaml + config + webui

- **domaine** : extension · **capacite** : registry · **port** : — · **cout** : M · **statut** : adopted
- **what** : Un catalogue de plugins decouverts au runtime (plugin.yaml) sans import en dur, avec config + webui de reglage par plugin.
- **how** : plugins/ : 43 dossiers avec plugin.yaml + default_config.yaml + extensions/ + webui/ ; plugins/_plugin_installer + _plugin_scan + _plugin_validator = le cycle du plugin au runtime. Omega : registre de capacites + manifestes valides (omega-app/omega-registry) : notre architecture, retenue : validation de manifeste par _plugin_validator.
- **adaptateur Omega** : omega-app::omega-registry (manifeste valide par plugin)
- **preuves** : `plugins/README.md` · `plugins/_plugin_validator`

### `az-orchestrator` — Orchestrator de CLI externes : skill a la demande, pas de tool verbeux

- **domaine** : agents · **capacite** : subagent-runner · **port** : SubagentRunner · **cout** : M · **statut** : planned
- **what** : Un plugin orchestrateur de sous-agents externes (Claude Code, Codex CLI, Cursor, OpenCode...) charge son guide a la demande (skill) au lieu d'un tool verbeux dans chaque prompt.
- **how** : plugins/_orchestrator/README.md : 'no terminal_agent tool and no settings-screen install button' ; le skill decide de la route (host CLI vs pal in container) ; adapter metadata pour 8+ harnais. Omega : ExternalSubagentRunner couvre l'esprit ; retenu : guide a la demande plutot qu'un tool lourd permanent.
- **adaptateur Omega** : omega-app::ExternalSubagentRunner (guides a la demande)
- **preuves** : `plugins/_orchestrator/README.md`

### `az-history` — History : records d'interaction serialisables

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : S · **statut** : planned
- **what** : Une structure History (Record + serialize) agnostic du moteur.
- **how** : helpers/history.py:374 class History(Record) ; serialize history.py:539. Omega : omega-context couvert.
- **adaptateur Omega** : omega-context (History serialisable)
- **preuves** : `helpers/history.py:374-539`

### `az-chat-compaction` — Chat compaction plugin : compaction de conversation par plugin

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : S · **statut** : adopted
- **what** : La compaction de conversation est un plugin atome (pas dans le noyau).
- **how** : plugins/_chat_compaction/ (extensions + helpers + prompts + webui) ; plugin.yaml declare le plugin, config par default_config.yaml. Omega : LlmCompactionEngine double couche deja couvert.
- **adaptateur Omega** : omega-context::LlmCompactionEngine
- **preuves** : `plugins/_chat_compaction`

### `az-sandbox-persistent` — Sandbox Docker persistante par agent, via plugin

- **domaine** : securite · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : M · **statut** : planned
- **what** : L'execution se fait dans une sandbox Docker persistante par agent (conteneur vivant entre les calls).
- **how** : plugins/_code_execution/ + helpers/docker.py ; le conteneur persiste entre les runs. Omega : NamespaceSandboxBackend ; a ajouter : sandbox persistante par run si demande.
- **adaptateur Omega** : omega-sandbox (execution persistante par agent)
- **preuves** : `plugins/_code_execution` · `helpers/docker.py`

### `az-memory-plugin` — Memory plugin : embeddings + persistence, tout dans le plugin

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : S · **statut** : planned
- **what** : La memoire long terme par embeddings est un plugin atome avec sa base.
- **how** : plugins/_memory/ = le cycle entier de memoire dans le plugin (embedding, storage, retrieval). Omega : MemoryStore couvert ; pattern de plugin self-contained : la memoire est branchable, pas noyau.
- **adaptateur Omega** : omega-app::FileMemoryStore (memoire branchee)
- **preuves** : `plugins/_memory`

### `az-tool-access` — Tool access : gate d'outils par contexte, par plugin

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : Un gate d'outils selon le contexte et le role de l'agent, implem. par plugin.
- **how** : plugins/_tool_access/ : le gate s'applique au runtime des outils. Omega : ExecPolicy ; le gate declaratif par manifeste est plus flexible ; a integrer.
- **adaptateur Omega** : omega-ports::ExecPolicy (gate par plugin/manifeste)
- **preuves** : `plugins/_tool_access`

### `az-time-travel` — Time-travel : rewind de conversation a un etat passe

- **domaine** : contexte · **capacite** : event-store · **port** : EventStore · **cout** : S · **statut** : planned
- **what** : Un plugin de time-travel : revenir a un etat anterieur de la conversation.
- **how** : plugins/_time_travel/ : rewind par scene historique dans l'UI. Omega : EventStore rejoue deja un point timestampe ; l'UI rewind est cliente de l'event.
- **adaptateur Omega** : omega-app (rewind de conversation)
- **preuves** : `plugins/_time_travel`

### `az-model-config` — Config modele par agent : selection + effort de raisonnement

- **domaine** : routage · **capacite** : router-policy · **port** : Selector · **cout** : S · **statut** : planned
- **what** : Le config modele (selection + reasoning effort) s'applique par agent, par plugin.
- **how** : plugins/_model_config/ + models.py specifie les modeles/champs. Omega : omega-router::Selector couvre.
- **adaptateur Omega** : omega-router (selection de modele par agent)
- **preuves** : `plugins/_model_config` · `models.py`

### `az-call-subordinate` — call_subordinate : fork un sous-agent avec role configure

- **domaine** : agents · **capacite** : subagent-runner · **port** : SubagentRunner · **cout** : S · **statut** : rejected
- **what** : L'outil call_subordinate declenche un sous-agent (fork) qui possede sa propre monologue de recherche, avec le role choisi.
- **how** : tools/call_subordinate.py ; fork par agent + subordinate data. Omega : LocalSubagentRunner couvert.
- **adaptateur Omega** : omega-app::LocalSubagentRunner
- **preuves** : `tools/call_subordinate.py`

### `az-parallel-tools` — parallel : execution de plusieurs outils en parallel, avec defer

- **domaine** : execution · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : planned
- **what** : L'outil parallel execute plusieurs tool calls en parallel ; defer retarde.
- **how** : tools/parallel.py + helpers/defer.py. Omega : execution concurrente de tools branchable sur ToolSurface.
- **adaptateur Omega** : omega-app (parallelisme des tool calls)
- **preuves** : `tools/parallel.py` · `helpers/defer.py`

