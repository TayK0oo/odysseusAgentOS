# Dissection — Bloop (BloopAI, Apache-2.0, Rust + TypeScript)

> Source : clone `--depth 1` de https://github.com/BloopAI/bloop dans `/home/tayk0oo/omega-re/bloop`.
> Donnees : `omega/crates/omega-harness/data/apps/bloop.json` (8 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | BloopAI |
| Langage | Rust + TypeScript |
| Repo | https://github.com/BloopAI/bloop |
| Architecture | Recherche de code + agent de QDM en Rust (server/bleep) : index de code tree-sitter multi-langage (sqlite), query DSL en pest (parse/planner/execute/ranking), agent d'échange LLM tools.json limité, registre multi-provider openai-compatible + local, client React/TS en alt. |

## 2. Inventaire fonctionnel

### `blp-code-index` — Index de code multi-langage tree-sitter (14 langages gérés)

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : L · **statut** : planned
- **what** : Un index de recherche de code sémantique (grep-like et QDDM). Le Bloop indexe toutes langues de type grammars tree-sitter (14 langages déclarés) selon l'index file/repo, avec un base de données sqlite local.
- **how** : server/bleep/src/indexes.rs + indexes/{doc,file,repo,schema,reader}.rs ; tree-sitter grammars (server/bleep/Cargo.toml:62-66) ; languages.yml declare les langages analysés. Omega : omega-context (index de code local : index de code诗文, CLAY multi-complice).
- **adaptateur Omega** : omega-context (index de code tree-sitter)
- **preuves** : `server/bleep/Cargo.toml:62-66` · `server/bleep/src/indexes.rs`

### `blp-query-pest` — Query DSL Compiler : un langage de requete pest parse

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : M · **statut** : planned
- **what** : Un meta query langage pete (grammar.pest, planner/compiler/execute): recherche de symbole simplifiée au user.
- **how** : server/bleep/src/query/{parser.rs,grammar.pest,compiler.rs,execute.rs,planner.rs},ranking.rs (server/bleep/src/query/ranking.rs). Omega : omega-context (query DSL, parse, planner, execute).
- **adaptateur Omega** : omega-context (query DSL + planner)
- **preuves** : `server/bleep/src/query/grammar.pest` · `server/bleep/src/query/planner.rs`

### `blp-agent-tools` — Outils de l'agent : answer/code/path/proc

- **domaine** : agents · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : planned
- **what** : Les outils de l'assistant LLM sont limités : réponse, interrogation code, path tool, process shell.
- **how** : server/bleep/src/agent/tools/{answer.rs,code.rs,path.rs,proc.rs}. Omega : ToolExecutor.
- **adaptateur Omega** : omega-app (bundle tools limités)
- **preuves** : `server/bleep/src/agent/tools/proc.rs`

### `blp-agent-model` — exchange/agent : modèle de conversation avec tool requests structurés

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : S · **statut** : planned
- **what** : L'agent conversation bloop : exchange structuré (questions/réponses), transcoder.
- **how** : server/bleep/src/agent/{exchange.rs,model.rs,transcoder.rs,prompts.rs}. Omega : l'échange maps le ContextEngine.
- **adaptateur Omega** : omega-context (smart exchange)
- **preuves** : `server/bleep/src/agent/exchange.rs`

### `blp-gpt-provider` — Registre LLM : openai-compatible + azure / local (llama)

- **domaine** : routage · **capacite** : llm-provider · **port** : Provider · **cout** : S · **statut** : planned
- **what** : Le provider LLM multi-fournisseurs : openai, azure, llama local, HF hub etc.
- **how** : server/bleep/src/llm{client.rs,call.rs}. Omega : omega-providers.
- **adaptateur Omega** : omega-providers (llm multi-providers en Rust)
- **preuves** : `server/bleep/src/llm/client.rs` · `server/bleep/src/llm/call.rs`

### `blp-git-index` — Index de dépét git + commits/remotes : le dépôt entier en map

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : S · **statut** : planned
- **what** : Un index github de l'arbre de fichiers + remotes + ongoing commits (bleep.db sqlite local)
- **how** : server/bleep/src/{commits.rs,remotes,db.rs,cache.rs}. Omega : corn-structure à test indexing tree (omega-context).
- **adaptateur Omega** : omega-context (index git,re discoveries par commit)
- **preuves** : `server/bleep/src/commits.rs` · `server/bleep/src/remotes/`

### `blp-periodic-jobs` — Jobs de fond periodiques : re-index des remotes + logrotate

- **domaine** : automatisation · **capacite** : scheduler-job · **port** : — · **cout** : S · **statut** : planned
- **what** : Un executor de jobs periodiques : re-index des remotes (mirrors) et rotation des logs, en tache de fond single-threaded.
- **how** : server/bleep/src/periodic.rs:23 start_background_jobs(app) + periodic/remotes.rs + periodic/logrotate.rs. Omega : omega-app (jobs periodiques ; deja couvert par le planificateur).
- **adaptateur Omega** : omega-app (re-index periodique des remotes)
- **preuves** : `server/bleep/src/periodic.rs:23` · `server/bleep/src/periodic/remotes.rs`

### `blp-background-executor` — SyncQueue/BackgroundExecutor : pool rayon + wait des taches longues

- **domaine** : execution · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : planned
- **what** : Un executor de taches de fond (SyncQueue, Progress) : le rayon pool absorbe le re-index lourd sans bloquer le serveur.
- **how** : server/bleep/src/background.rs:28 rayon_pool, :55 SyncQueue, :66 BackgroundExecutor, :117 spawn<T>. Omega : omega-app::App (execution de fond non bloquante).
- **adaptateur Omega** : omega-sandbox (pool de taches de fond)
- **preuves** : `server/bleep/src/background.rs:28-137`

