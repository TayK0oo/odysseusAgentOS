# Dissection — Devika (stitionai, MIT, Python)

> Source : clone `--depth 1` de https://github.com/stitionai/devika dans `/home/tayk0oo/omega-re/devika`.
> Donnees : `omega/crates/omega-harness/data/apps/devika.json` (9 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | stitionai |
| Langage | Python |
| Repo | https://github.com/stitionai/devika |
| Architecture | Agent ingénieur logiciel autonome en Python (19k etoiles, depuis archivé/très bogué) : Agent core execute un pipeline fixe de sous-agents Dans Vue (planner -> researcher -> coder -> action -> runner -> patcher -> reporter), AgentState SQLModel persiste par projet, knowledge base RAG BERT, sandbox firejail, services github/netlify, socket.io pour la UI temps réel. |

## 2. Inventaire fonctionnel

### `dvk-agent-core` — Agent core : chef d'orchestre de sous-agents especialises

- **domaine** : agents · **capacite** : subagent-runner · **port** : SubagentRunner · **cout** : M · **statut** : planned
- **what** : Un Agent singleton orchestre un pipeline fixe de sous-agents : planner -> researcher -> coder -> action -> runner -> patcher -> reporter, avec monologue interne chauffant le serveur de chat.
- **how** : class Agent (src/agents/agent.py:36) def execute(prompt, project_name) (agent.py:270) ; les sous-agents sont des classes par dossier (src/agents/{planner,researcher,coder,action,runner,patcher,reporter,decision}) ; internal_monologue communique l'etape a l'UI. Omega : SubagentRunner + agent-store couvent ; l'ecart : la digestion sequentiale dure de la chaine (fixe) fait c'est un spin deterministe.
- **adaptateur Omega** : omega-app (pipeline fixe de sous-agents)
- **preuves** : `src/agents/agent.py:36-270`

### `dvk-agentstate` — AgentState : etat persiste par projet avec SQLModel + loi d'activite

- **domaine** : contexte · **capacite** : memory-store · **port** : MemoryStore · **cout** : S · **statut** : planned
- **what** : L'etat de chaque projet (historique d'interaction, token usage, agent actif) est persiste en base par project name.
- **how** : class AgentState (src/state.py:18) : create_state/delete_state/add_to_current_state/get_latest_state (state.py:47-102) ; set_agent_active/is_agent_active (state.py:109-125) ; AgentStateModel SQLModel table (state.py:10). Omega : MemoryStore couvert par FileMemoryStore ; la surfacization par project-id du state couvre.
- **adaptateur Omega** : omega-app::FileMemoryStore (state par project)
- **preuves** : `src/state.py:10-125`

### `dvk-knowledge-base` — Knowledge base RAG : embeddings bert locaux, retrieval par tag

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : M · **statut** : planned
- **what** : La memoire long-shot par base de conocimiento : erreurs de connaissance ajputent/taggable, retrieval par embeddings BERT sante locale + RAG.
- **how** : class KnowledgeBase (src/memory/knowledge_base.py:15) : add_knowledge/get_knowledge par tag (knowledge_base.py:22-28) ; Knowledge SQLModel (knowledge_base.py:10) ; src/memory/rag.py + src/bert/ pour les embeddings locaux. Omega : MemoryStore couve au stockage ; l'ecart : retrieval semantique.
- **adaptateur Omega** : omega-context (knowledge base par tag + RAG)
- **preuves** : `src/memory/knowledge_base.py:10-28`

### `dvk-firejail` — Sandbox firejail : approximation bash contrainte en require

- **domaine** : securite · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : S · **statut** : planned
- **what** : La sandbox de code local utilise FireJail pour contraindre l'éx Braun avec permission restrictives.
- **how** : src/sandbox/firejail.py + code_runner.py : code execute as firejail process + la gauche est a la policy permissive/secure. Omega : omega-sandbox::NamespaceSandboxBackend (namespace Linux) ; firejail est une alt de l'approche ; a retenir : process bash contrôlable sandboxé.
- **adaptateur Omega** : omega-sandbox (firejail sandbox adapter)
- **preuves** : `src/sandbox/firejail.py` · `src/sandbox/code_runner.py`

### `dvk-browser-agent` — Browser agent: navigation aléatoire, click, gather d'information web

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : M · **statut** : rejected
- **what** : Un agent navigateur autonome, gobble net + interaction et collecte d'information de la page (search, interaction).
- **how** : src/browser/browser.py + interaction.py + search.py. Omega : passe ; la recherche web sera le plugin ToolsSurface dans la version Universe web.
- **adaptateur Omega** : plugin/browser (outil web du ToolSurface)
- **preuves** : `src/browser/browser.py` · `src/browser/interaction.py`

### `dvk-git-netlify` — Services GitHub + Netlify : push et deploys externals

- **domaine** : automatisation · **capacite** : scheduler-job · **port** : — · **cout** : S · **statut** : planned
- **what** : The recording from a Completion: push to github and deploy to netlify.
- **how** : src/services/github.py + netlify.py + git.py. Omega : omega-app job de deploy.
- **adaptateur Omega** : omega-app (job de push + deploy)
- **preuves** : `src/services/github.py` · `src/services/netlify.py`

### `dvk-llm-providers` — Registre LLM multi-modes : claude/gpt/ollama local + selection par tache

- **domaine** : routage · **capacite** : llm-provider · **port** : Provider · **cout** : S · **statut** : rejected
- **what** : Un catalogue de providers sélectionnés par tache (plan/coder/execute), sélectionnés en adaptation des modèles.
- **how** : src/llm/ (les models par familles, avec requete par type de tache) ; sample.config.toml donne les selections. Omega : omega-router::Selector + FallbackPolicy, deja riche.
- **adaptateur Omega** : omega-providers (selection par tache)
- **preuves** : `src/llm` · `sample.config.toml`

### `dvk-socket-ui` — UI temps réel : socket.io pour les tables/times du run

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : M · **statut** : planned
- **what** : L'UI reçoit les专著還 des sous-agents par socket.io en temps réel (agent list, etape en cours, log terminal).
- **how** : src/socket_instance.py + devika.py (serveur Flask + socket). Omega : omega-server (API + UI une origine) ; la couche socket temps réel est a appareiller avec le journal d'events.
- **adaptateur Omega** : omega-server (socket temps reel du run)
- **preuves** : `src/socket_instance.py` · `devika.py`

### `dvk-ui-autonomous` — Interface de pilotage : webui pour envoyer des objectif et suivre les agents

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : S · **statut** : rejected
- **what** : Un frontend React/WebUI pour lancer un objectif, suivre les etapes internes (monologue), approuver.
- **how** : ui/ devika : Web UI (React). Omega : omega-ui couvre. Rien de nouveau.
- **adaptateur Omega** : omega-ui
- **preuves** : `devika.py`

