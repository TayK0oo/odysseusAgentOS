# Dissection — SuperAGI (TransformerOptimus, MIT, Python)

> Source : clone `--depth 1` de https://github.com/TransformerOptimus/SuperAGI dans `/home/tayk0oo/omega-re/superagi`.
> Donnees : `omega/crates/omega-harness/data/apps/superagi.json` (9 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | TransformerOptimus |
| Langage | Python |
| Repo | https://github.com/TransformerOptimus/SuperAGI |
| Architecture | Framework d'agents autonomes multi-tenant (Python/FastAPI/Celery) : workflow persiste par etapes (iteration/tool/wait-permission/queue), toolbox de 25+ tools, stockage vectoriel par adapters (pinecone/qdrant/weaviate/workspace), jobs Celery recurrents, orchestration Redis, isolation par organisation. |

## 2. Inventaire fonctionnel

### `sup-workflow-steps` — Boucle par etapes de workflow (iteration/tool/wait/queue) persistee

- **domaine** : agents · **capacite** : agent-loop · **port** : — · **cout** : M · **statut** : planned
- **what** : L'agent marche par etapes de workflow persistantes en base (iteration, tool, wait, queue) : chaque etape a un handler execute_step(), et la prochaine etape est pointee en base.
- **how** : AgentIterationStepHandler (superagi/agent/agent_iteration_step_handler.py:35) execute_step (:46) construit le prompt (_build_agent_prompt :135) et propose la suite via _update_agent_execution_next_step (:123) ; QueueStepHandler (queue_step_handler.py:20) gère une queue de taches ; AgentWorkflowStepWaitHandler (:18) attend une permission humaine. Omega : omega-app::App::run durable + EventStore : le point d'etape persiste + resume est deja couvert, le pattern des handler-step par type est un shablon.
- **adaptateur Omega** : omega-core::AgentLoop + omega-app (etapes persiste/reprise)
- **preuves** : `superagi/agent/agent_iteration_step_handler.py:35-170` · `superagi/agent/queue_step_handler.py:20-55`

### `sup-wait-permission` — Wait-for-permission : l'agent s'arrete et attend l'humain

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : Une etape peut requérir l'approbation humaine : l'execution se met en attente (status WAITING) jusqu'au clic.
- **how** : _handle_wait_for_permission (agent/agent_iteration_step_handler.py:170) pose le statut WAITING sur l'execution ; AgentWorkflowStepWaitHandler gere la reprise (agent_workflow_step_wait_handler.py:18). Omega : pas de checkpoint d'explicit-approval sur la boucle ; utile pour un ExecPolicy avec confirmation tierce.
- **adaptateur Omega** : omega-ports::ExecPolicy (confirm-then-run)
- **preuves** : `superagi/agent/agent_iteration_step_handler.py:170`

### `sup-tool-bundles` — Un catalogue de 25+ tools immuable : base_tool, each tool self-contained

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : planned
- **what** : Un toolbox categorisé (code, file, github, google, slack, email, thinking, knowledge_search...) ou chaque tool s'auto-déclare avec sa schema ; le registre est fixé.
- **how** : superagi/tools/ : dossiers par outil + base_tool.py (la classe de base, independent) ; tool_builder.py assembles le set pour l'agent. Omega : ToolExecutor + ToolSurface : la couverture est deja couverte ; le pattern toolbox-par-domaine note pour le catalogue initial.
- **adaptateur Omega** : omega-app (toolkit catalogue initiale)
- **preuves** : `superagi/tools/base_tool.py` · `superagi/agent/tool_builder.py`

### `sup-vector-stores` — Vector store by adapter : pinecone/qdrant/weaviate/workspace

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : M · **statut** : planned
- **what** : Les memories vectorielles passent par une multiplex de fournisseurs, avec une factory d'adaptation par key.
- **how** : superagi/vector_embeddings/vector_embedding_factory.py + vector_store/ par dossiers (pinecone, qdrant, weaviate, workspace local). Omega : MemoryStore couvert ; l'ecart : les embeddings cloud multiples sont le pattern a ajouter : adapter par fournisseur.
- **adaptateur Omega** : omega-app (memory store par fournisseur vecteur)
- **preuves** : `superagi/vector_embeddings/vector_embedding_factory.py`

### `sup-celery-jobs` — Jobs via Celery : executions d'agents planifiees et recurrence

- **domaine** : automatisation · **capacite** : scheduler-job · **port** : — · **cout** : M · **statut** : planned
- **what** : Les executions d'agents (batch et recurrent) sont des jobs Celery queueux, avec scheduler de recurrence.
- **how** : superagi/worker.py (workers celery) + jobs/ + DockerfileCelery. Omega : omega-app::App jobs + planificateur : couvre le pattern cron+queue, en Rust.
- **adaptateur Omega** : omega-app (scheduler + job queue durable)
- **preuves** : `superagi/worker.py`

### `sup-tools-json` — tools.json + config par fichier yaml (orga/étanchéité clé)

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : La config est decouple en environnements de YAML (config_template.yaml, org keys par env), et les tools déclarés en tools.json.
- **how** : tools.json (la definition des schemas du registre d'outils) + config_template.yaml (les clés par environnement, personnelles sans créer). Omega : omega-config avec le manifeste (declarator par les fichiers d'assets).
- **adaptateur Omega** : omega-config + manifestes declares anque
- **preuves** : `tools.json` · `config_template.yaml`

### `sup-llm-factories` — Registre LLM par familles (openai/anthropic/replicate/local)

- **domaine** : routage · **capacite** : llm-provider · **port** : Provider · **cout** : S · **statut** : rejected
- **what** : Un registre de fournisseurs LLM construits par factory, avec mappage de la tache au modele.
- **how** : superagi/llms/ : pour chaque famille de provider, la classe de base. Omega : omega-router + omega-providers couvert.
- **adaptateur Omega** : omega-providers (factory par famille)
- **preuves** : `superagi/llms`

### `sup-apm` — Agent Performance Monitoring : run metrics exposee en API

- **domaine** : observabilite · **capacite** : telemetry-sink · **port** : TelemetrySink · **cout** : S · **statut** : planned
- **what** : Un module APM : les metrics par execution d'agent (time take, number of calls, comodity) sont exposes API/webview.
- **how** : superagi/apm/ : data collectors des execution (run, tokens, etc.) par agents/task. Omega : otel + EventStore.
- **adaptateur Omega** : omega-otel (peformance per run)
- **preuves** : `superagi/apm`

### `sup-organisations` — Organisations / isolation multi-tenant en base

- **domaine** : securite · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Chaque agent/tool/resource est scope par organisation_id ; l'isolation est guidée par la base de donees (SQLAlchemy).
- **how** : superagi/models/*.py nous font chaque entité avec organization_id (SQLAlchemy, alembic migrations). Omega : le worktree isolation de l'agent-harness est au niveau nos runners ; le pattern de scope en base est le meme que ExecutionScope.
- **adaptateur Omega** : omega-harness (scope par organisation)
- **preuves** : `superagi/models`

