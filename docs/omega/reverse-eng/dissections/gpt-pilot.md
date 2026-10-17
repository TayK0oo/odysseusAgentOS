# Dissection de harnais — GPT Pilot (Pythagora)

> Cloné (lecture seule) : `/home/tayk0oo/omega-re/gpt-pilot`
> Repo : `https://github.com/Pythagora-io/gpt-pilot` — 33 654 étoiles — Python
> Données structurées : `omega/crates/omega-harness/data/apps/gpt-pilot.json`

## 1. Fiche

| Attribut | Valeur |
|---|---|
| Éditeur | Pythagora (Pythagora Technologies, Inc.) |
| Licence (fichier LICENSE du clone) | Functional Source License 1.1, MIT Future License → SPDX `FSL-1.1-MIT` (GitHub renvoie `NOASSERTION`, le fichier LICENSE lit localement fait foi) |
| Langage | Python (paquet `pythagora-core`, v2.0.10, poetry) |
| OSS | Oui (source ouverte, licence FSL : usage interne/éducation/recherche autorisé ; usage commercial concurrentiel restreint) |
| Repo | https://github.com/Pythagora-io/gpt-pilot |
| Architecture | Noyau Python (`pythagora-core`) orchestrant, via une machine à états SQLAlchemy/SQLite, une équipe d'agents spécialisés (Wizard, Spec Writer, Architecte, Tech Lead, Développeur, Code Monkey, Frontend, Chasseur de bogues, Dépanneur) chacun piloté par un LLM. Une CLI lance un serveur IPC (`localhost:8222`) consommé par une extension VS Code ou une UI console. Des templates (Vite React, Node/Express) génèrent le squelette de l'application cible. La télémétrie remonte vers l'API cloud de Pythagora (`https://api.pythagora.io/telemetry`). |

## 2. Entrées

- **Binaire / CLI** : `main.py` → `core.cli.main.run_pythagora` ; sous-commandes `list`, `list_json`, `show_config`, `import_v0` (import base v0), `delete`. Port API server par défaut `8222` (`core/cli/main.py:375`).
- **Points d'extension** : agents spécialisés instanciés par l'orchestrateur (`core/agents/orchestrator.py:463 create_agent`), UI (`PlainConsoleUI`, `VirtualUI`, `IPCServer`), templates de projet (`core/templates/registry.py` `PROJECT_TEMPLATES`), prompts Jinja2 (`core/prompts/*`).
- **Fichiers de config** : `example-config.json` (LLM providers OpenAI/Anthropic/Groq/Azure, agents par modèle, DB `sqlite+aiosqlite:///data/database/pythagora.db`, FS workspace, ignore paths, seuil 50 Ko) et `~/.gpt-pilot/config.json` (télémétrie, `core/config/user_settings.py`).

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Clients LLM multi-fournisseurs + config par agent | routage | Appels LLM OpenAI, Anthropic, Groq, Azure, Relace ; modèle/température par agent | `core/llm/{openai,anthropic,groq,azure,relace}_client.py`, `core/llm/base.py:36`, `core/agents/base.py:175`, `example-config.json` | `llm-provider → Provider → omega-providers::OllamaProvider` | adopted | S |
| Édition de fichiers par blocs `<pythagoracode>` + revue diff | outils | Création/modification de fichiers, diff, acceptation/rework/ignorage par hunk | `core/agents/code_monkey.py:68/81/213/160`, `core/agents/developer.py:204` | `tool-surface → ToolSurface → omega-context::ToolSurfaceIndex` | adopted | M |
| Journal d'états append-only + reprise à une étape | observabilité | Chaîne `prev_state_id`/`step_index`, rechargement à une étape, relecture de conversation, rollback des états futurs | `core/db/models/project_state.py:59/269/492`, `core/state/state_manager.py:272/433/480`, `core/ui/api_server.py:678` | `event-store → EventStore → omega-otel::TraceReplayer` | adopted | M |
| Wizard d'onboarding (product owner) | agents | Type de pile, authentification, envoi docs OpenAPI vers RAG, base de connaissances, épic initial | `core/agents/wizard.py:20/30/133` | `agent-store → AgentStore` | planned | M |
| Rédaction de spécification (Spec Writer) | agents | Spécification complète, nom de projet, détection auth, complexité, template | `core/agents/spec_writer.py:19/63/163/305/345/34`, `core/db/models/specification.py:22` | `agent-store → AgentStore` | planned | M |
| Architecte : template, technologies, dépendances | agents | Sélection template, technologies (dép. système + paquets), compatibilités, vérification dépendances système | `core/agents/architect.py:95/124/173/237/227`, `core/templates/registry.py` | `agent-store → AgentStore` | planned | M |
| Tech Lead : épics, sous-épics, tâches, endpoints API | agents | Décomposition épic → tâches avec endpoints API, tâche auth hardcodée (register/login), plan de développement | `core/agents/tech_lead.py:55/92/296/381` | `agent-store → AgentStore` | planned | L |
| Développeur : décomposition de tâche en étapes | execution | Étapes save_file/command/human_intervention/utility_function, exécution/édition/saut, parse JSON | `core/agents/developer.py:91/204/349/313` | `task-execution → TaskRunner` | planned | L |
| Agent Frontend : construction + itération UI | ui | Template Vite React, itération interface, suppression mocks, auto-débogage frontend | `core/agents/frontend.py:40/61/151/451` | `ui-rendering → UIRunner` | planned | L |
| Chasseur de bogues + pair programming | execution | Analyse logs, ajout journaux de débogage, test utilisateur, pair programming | `core/agents/bug_hunter.py:57/103/144/241` | `debug-session → Debugger` | planned | L |
| Dépanneur : boucle de revue, détection de boucle, solution alternative | agents | Itération de dépannage, retour utilisateur, détection boucle (seuil 3), ProblemSolver, trace boucle | `core/agents/troubleshooter.py:35/63/244/426` | `review-loop → Reviewer` | planned | M |
| Finalisation de tâche + commit Git | execution | Validation tâche, commit Git (message LLM à partir du diff), marque application/fonctionnalité finie | `core/agents/task_completer.py:11`, `core/agents/git.py:92/35` | `vcs → VCS` | planned | S |
| Gestionnaire d'état + reprise de session | contexte | Chargement projet à étape/branche/état, current/next_state, commit/rollback, restauration fichiers, import hors-ligne | `core/state/state_manager.py:47/272/433/740`, `core/agents/orchestrator.py:370` | `context-engine → ContextEngine` | planned | L |
| Modèles de base de données du projet | mémoire | SQLAlchemy async : projet, branche, état, spécification, fichier, journal exécution, requête LLM, chat, saisie utilisateur, KB | `core/db/models/*.py`, `core/db/migrations/versions/*`, `example-config.json:52` | `persistence → Store` | planned | L |
| Exécution commandes shell + gestion de processus | execution | Commandes shell (npm install, git), processus arrière-plan avec watcher, journalisation exécution | `core/proc/process_manager.py:140/218/202/171`, `core/proc/exec_log.py:7`, `core/agents/orchestrator.py:169` | `exec-policy → ExecPolicy` | planned | M |
| CLI, serveur IPC et interfaces utilisateur | ui | Point d'entrée CLI, création/chargement projet, serveur IPC localhost:8222 (extension VS Code), UI console/virtuelle | `core/cli/main.py:1/429/194/121`, `core/ui/api_server.py:43`, `core/ui/console.py:11` | — (service) | planned | M |
| Télémétrie de session + rapports de crash | observabilité | Compteurs (LLM, tokens, étapes, commandes, fichiers, lignes), crash diagnostics, trace événements, envoi HTTP opt-out | `core/telemetry/__init__.py:22/322/275/363/391`, `core/config/user_settings.py:17`, `docs/TELEMETRY.md` | `telemetry-sink → TelemetrySink` | planned | M |
| Système de fichiers virtuel + règles d'ignorance | contexte | Abstraction disque/mémoire, hachage SHA-1, ignore chemins/binaires au-dessus du seuil | `core/disk/vfs.py:12/115`, `core/disk/ignore.py`, `core/state/state_manager.py:628` | `file-system → FileSystem` | planned | S |

Total : 18 fonctionnalités (3 `adopted`, 15 `planned`, 0 `rejected`).

## 4. Traduction en plugins

Pour chaque fonctionnalité retenue, le triplet capacité → port → adaptateur, et le mode d'intégration.

### Déjà adoptés dans Omega (adapter existant)

1. `llm-provider → Provider → omega-providers::OllamaProvider` (plugin, `S`).
   GPT Pilot expose plusieurs fournisseurs LLM avec configuration par agent ; Omega possède déjà le port `Provider` et l'adaptateur `OllamaProvider`. L'ajout des fournisseurs OpenAI/Anthropic/Groq/Relace serait une extension de catalogue (config + scoring) sur le même port.
2. `tool-surface → ToolSurface → omega-context::ToolSurfaceIndex` (plugin, `M`).
   GPT Pilot génère des fichiers via des blocs `<pythagoracode file>`, produit un diff et accepte/rework/ignore par hunk (CodeMonkey). C'est l'équivalent direct d'une surface d'outils d'édition de code.
3. `event-store → EventStore → omega-otel::TraceReplayer` (plugin, `M`).
   GPT Pilot stocke une chaîne d'états de projet append-only (`prev_state_id`, `step_index`), autorise le rechargement à une étape (`load_project(step_index=…)`), la relecture de conversation (`load_convo`, `get_project_states_in_between`) et le rollback des états futurs (`delete_after`). C'est un magasin d'événements à la portée d'un projet.

### À planifier (pas encore dans Omega)

4. `agent-store → AgentStore` — Wizard (product owner), Spec Writer, Architecte, Tech Lead : agents spécialisés à découvrir/charger (frontmatter YAML, règles, permissions) — plugin, `M`/`M`/`M`/`L`.
5. `task-execution → TaskRunner` — Développeur : décomposition de tâche en étapes typées — plugin, `L`.
6. `ui-rendering → UIRunner` — Agent Frontend : construction/itération de l'interface — plugin, `L`.
7. `debug-session → Debugger` — Chasseur de bogues + pair programming — plugin, `L`.
8. `review-loop → Reviewer` — Dépanneur : boucle de revue, détection de boucle (seuil 3), solution alternative — plugin, `M`.
9. `vcs → VCS` — Finalisation de tâche + commit Git — plugin, `S`.
10. `context-engine → ContextEngine` — Gestionnaire d'état + reprise de session — plugin, `L`.
11. `persistence → Store` — Modèles SQLAlchemy du projet — plugin, `L`.
12. `exec-policy → ExecPolicy` — Exécution shell et gestion de processus — plugin, `M`.
13. `telemetry-sink → TelemetrySink` — Télémétrie + crash reports — plugin, `M`.
14. `file-system → FileSystem` — VFS + règles d'ignorance — plugin, `S`.
15. `—` — CLI / serveur IPC / UI : service transport (pas un plugin Omega) — `service`, `M`, mappé sur `omega-server` (cockpit + `/api/health\|plugins\|run\|trace/:id`) côté service, pas sur un port plugin.

### Modes d'intégration

- `plugin` (15 fonctionnalités) : chaque capacité s'expose via un port Omega et serait implémentée comme adaptateur de plugin (registre, contexte, router, sandbox, otel…).
- `service` (1 fonctionnalité, `gp-cli-ipc-ui`) : le CLI/serveur IPC est un transport de bout en bout, intégré comme service `omega-server` plutôt que comme plugin.

## 5. Écarté

Aucune fonctionnalité n'est marquée `rejected` dans le JSON. Deux points de vigilance qualitatifs (non intégrés tels quels) :

- **Couplage cloud / phone-home** : la télémétrie envoie par défaut vers `https://api.pythagora.io/telemetry` et le Wizard envoie les docs OpenAPI vers `PYTHAGORA_API/rag/upload` (service RAG cloud de Pythagora). Cela entre en tension avec le pilier Omega "self-hosted au besoin / zéro fournisseur en dur". À intégrer en mode opt-out, endpoint configurable, et jamais en dur hors registre.
- **Couplage UI ↔ logique** : des appels UI sont embarqués directement dans les agents (ex. `send_file_status`, `generate_diff`, `send_bug_hunter_status`) et des références à l'extension VS Code sont hardcodées (`pythagora_source`, textes de boutons partagés avec l'extension). En Omega, l'UI ne doit pas être appelée par la logique métier sans port (interdiction "UI → logique sans port").

## 6. Sources

- Clone local (lecture seule) : `/home/tayk0oo/omega-re/gpt-pilot`
- Repo officiel : https://github.com/Pythagora-io/gpt-pilot
- Fichier LICENSE (SPDX `FSL-1.1-MIT`) : `/home/tayk0oo/omega-re/gpt-pilot/LICENSE`
- Preuve machine REA : `docs/omega/research/rea/evidence/intake-gpt-pilot.summary.json` (33 654 étoiles, Python, `cloned: true`, 614 nœuds / 784 arêtes, 7 limitations, confiance `derived`)
- Schéma de dissection : `docs/omega/reverse-eng/DISSECTION-SCHEMA.md`
- Exemple de forme : `omega/crates/omega-harness/data/apps/opencode.json`

### Limites de la preuve REA

- `provider`: `rea-javascript-application` alors que GPT Pilot est Python → nature probablement mal classée par l'outil REA.
- `confidence`: `derived` (inférée, non mesurée directement sur le binaire).
- `license`: `NOASSERTION` dans la preuve REA (GitHub), alors que le fichier LICENSE du clone déclare explicitement `FSL-1.1-MIT` — le fichier local fait foi.
- L'analyse est statique (lecture du clone) : aucun run d'exécution, aucun appel LLM réel, pas de mesure de coût d'inférence.

## Note d'honnêteté (statut `adopted`)

Conformément à la table d'honnêteté du schéma, `adopted` n'est utilisé que pour les capacités dont le port et l'adaptateur existent réellement dans Omega (`llm-provider/Provider/OllamaProvider`, `tool-surface/ToolSurface/ToolSurfaceIndex`, `event-store/EventStore/TraceReplayer`). Toutes les autres fonctionnalités GPT Pilot sont marquées `planned` (adaptateur `null`) car elles n'existent pas encore dans notre code.
