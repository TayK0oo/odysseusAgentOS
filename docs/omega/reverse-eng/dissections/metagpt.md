# Dissection MetaGPT (geekan/MetaGPT)

> Dissection fonctionnelle du harnais MetaGPT pour Omega.
> Refs `metagpt/...` = clone local en lecture seule `/home/tayk0oo/omega-re/metagpt`.
> Données machine : `omega/crates/omega-harness/data/apps/metagpt.json` (24 fonctionnalités).
> Preuve REA : `docs/omega/research/rea/evidence/intake-metagpt.summary.json`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Nom | MetaGPT |
| Éditeur | geekan (Chenglin Wu) + communauté (projet ouvert) |
| Licence | MIT (vérifiée dans le clone, `LICENSE` : Copyright (c) 2024 Chenglin Wu) |
| Langage | Python (pydantic + asyncio + llama-index) |
| Stars | 70 887 |
| Repo | https://github.com/geekan/MetaGPT (clone : `/home/tayk0oo/omega-re/metagpt`) |
| Nature | Framework multi-agents « société logicielle » : des rôles autonomes coordonnés par une SOP, produisant documents structurés puis code |

**Architecture en une phrase :** des `Role` (PM, architecte, ingénieur, QA, researcher…) publient
des `Message` (clé de routage `cause_by` = type d'Action producteur, `send_to` = destinataires)
dans un `Environment` ; chaque rôle observe, choisit une stratégie de réaction (ReAct / ordre /
plan-and-act) et agit via des `Action` à sorties structurées (`ActionNode`) ; les artefacts sont
publiés dans un workspace Git versionné.

## 2. Entrées

- **CLI `metagpt`** : `console_scripts` → `metagpt.software_company:app` (`metagpt/setup.py:116-121`).
- **API Python** : `Team(use_mgx=True).run(idea, n_round=...)` (`metagpt/team.py:122-137`),
  `startup()` / `generate_repo()` (`metagpt/software_company.py`).
- **Config** : `~/.metagpt/config2.yaml` (`metagpt/config2.py:101-107`) + paramètres CLI.
- **Points d'extension** : décorateur `@register_tool` pour les outils, décorateurs
  `mark_as_readable`/`mark_as_writeable` pour les environnements, rôles personnalisés
  via `roles: [RoleCustomConfig]`.

## 3. Inventaire fonctionnel (24 fonctionnalités)

| Domaine | Fonctionnalité | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|
| agents | Pipeline de rôles SOP | product_manager.py:32, architect.py:28, engineer.py:73, qa_engineer.py:39 | AgentStore + Registre | planned | M |
| agents | Abstraction Role (profil/but/contraintes) | roles/role.py:125-153 | AgentStore | planned | S |
| agents | Bus pub/sub labellisé par cause_by | schema.py:232, base_env.py:175-211 | Registre + EventStore | planned | M |
| execution | Environnements hot-swap (gym-like + registres d'API) | base_env.py:29-104 | Registre (port EnvBackend) | planned | L |
| agents | Trois stratégies de réaction | roles/role.py:82-89,261-276,340-379 | AgentStore (run-modes) | planned | S |
| agents | Planner explicite + revue humaine + re-planning | strategy/planner.py:58-167 | TaskClassifier + ContextEngine | planned | M |
| contexte | ActionNode : sorties structurées + révision partielle | actions/action_node.py:135-841 | ContextEngine (sortie typée) | planned | M |
| outils | ToolRegistry, schémas auto, tags, Editor:read,write | tools/tool_registry.py:27-163 | ToolSurface | adopted | M |
| outils | Recommandation d'outils (recall BM25 + rank LLM) | tools/tool_recommend.py:54-228 | ToolSurface | planned | M |
| agents | RoleZero : agent à commandes JSON | roles/di/role_zero.py:55-447 | ToolSurface + ContextEngine | planned | L |
| ui | Human-in-the-loop via TeamLeader + chat direct | environment/mgx/mgx_env.py:24-71 | PermissionPolicy (ApprovalQueue) | planned | S |
| observabilite | Budget monétaire + garde-fou par round | utils/cost_manager.py:25-111 | TelemetrySink | planned | S |
| routage | Catalogue 18 fournisseurs LLM + pricing | configs/llm_config.py:19, provider/* | Provider | adopted | M |
| configuration | Config en couches (YAML home + CLI + role custom) | config2.py:30-107 | ConfigSource | adopted | S |
| memoire | Mémoire par rôle indexée par cause | memory/memory.py:20-107 | EventStore | planned | S |
| memoire | Long-term memoire vectorielle FAISS par rôle | memory/memory_storage.py:19, role_zero.py:173-193 | ContextEngine | planned | M |
| memoire | Pool d'expériences auto-évaluées | exp_pool/decorator.py:29-175 | SkillStore (ExperienceStore) | planned | L |
| memoire | SkillStore retrouvée par similarité | management/skill_manager.py:17-66 | SkillStore | adopted | S |
| contexte | Moteur RAG hybride (FAISS/BM25/ES) + FLARE | rag/engines/simple.py:64-241 | ContextEngine | planned | L |
| execution | Publication d'artefacts Git + archive auto | utils/git_repository.py:61-247 | EventStore + Provenance | planned | M |
| execution | Interruption / reprise par sérialisation Team/Role | team.py:59-81, role.py:155-351 | EventStore + TraceReplayer | planned | M |
| routage | Tree-of-Thought BFS/DFS | strategy/tot.py:33-172 | — | rejected | L |
| outils | Gamme de moteurs web et multimédia | tools/search_engine*.py, tools/web_browser_engine.py | ToolSurface | planned | M |
| routage | Quick-think : classifieur d'intention pré-boucle | roles/di/role_zero.py:342-383 | TaskClassifier | planned | S |

## 4. Traduction en plugins

### 4.1 Les 4 fortes (déjà adoptées chez Omega)

- **`mgt-tool-registry` → `ToolSurface` (`omega-context::ToolSurfaceIndex`)**. Le registre
  décoré avec schéma auto-extrait et index par tags correspond directement à notre
  ToolSurfaceIndex ; la syntaxe `Editor:read,write` (sous-ensemble des méthodes d'un outil)
  se traduit en filtrage de surface par plugin. La version Omega : un `derive`/proc-macro
  Rust au lieu du décorateur Python.
- **`mgt-providers` → `Provider` (`omega-providers::OllamaProvider`)**. L'enum `LLMType`
  + registre est la même idée que notre catalogue par transforms : chaque fournisseur est
  un plugin avec `pricing_plan`, proxy et hooks de post-traitement. `OllamaProvider`
  couvre déjà le socle ; l'élargissement du catalogue est un ajout incrémental.
- **`mgt-config-layer` → `ConfigSource` (`omega-config`)**. Couches `~/.metagpt/config2.yaml`
  + CLI + config par rôle : correspond aux couches et à la précedence de `omega-config`.
  La sous-section par plugin (`role_zero:`, `browser:`, `mermaid:`) est un pattern propre
  à adopter : un bloc yaml par plugin enregistré.
- **`mgt-skill-manager` → `SkillStore` (`omega-context::SkillIndex`)**. Réserve de
  compétences (nom + code + description) retrouvée par similarité ; notre SkillIndex
  couvre le stockage et l'indexation.

### 4.2 Les intégrations les plus fortes à venir (planned)

1. **`mgt-tool-recommend`** (M) : recall BM25 no-commentaire + ranking LLM + auto-réparation
   JSON est le meilleur pattern observé pour hydrater un sous-ensemble d'outils par tour
   (vs tout exposer toujours). Gain direct de contexte. Porte : `ToolSurface` + `omega-router`.
2. **`mgt-sop-roles` + `mgt-msg-bus`** (M+M) : le duo SOP + pub/sub par `cause_by` est
   l'élément signature de MetaGPT. En Omega : chaîne de plugins abonnés aux événements
   `ActionProduced` de l'EventStore — tout est tracé et rejouable, aucun orchestrateur caché.
3. **`mgt-actionnode`** (M) : « champs déclarés + LLM remplit un document typé + révision
   des seuls champs modifiés » se raccroche à `context-engine` et donne une sortie
   structurée révisable document par document.
4. **`mgt-exp-pool`** (L) : boucle d'expérience (sauvegarde req/resp, score, rappel few-shot)
   est la réalisation concrète du pilier « Adaptatif & évolutif », à ancrer avec
   `omega-otel::Provenance` pour prouver chaque expérience.
5. **`mgt-quick-think`** (S) : classifieur d'intention pré-boucle (réponse directe vs
   recherche vs tâche) recouvre `HeuristicTaskClassifier` déjà en place.

### 4.3 Integration service

Rien de prioritaire : MetaGPT est embarque (pas de serveur HTTP à dissecter), tout est
in-process. Le rendu de documents se limite à Mermaid (`utils/mermaid.py`).

## 5. Écarté (ce qu'on ne reprend pas)

- **Tree-of-Thought** (`mgt-tot`, `strategy/tot.py`) : solveur arborescent BFS/DFS récurrent
  avec évaluation LLM de chaque nœud. Coût Rust L, dépense de tokens démesurée, et la valeur
  attendue (pondération de plans) est déjà couverte par `omega-router` (Pareto + TierFallback).
  **Rejected** avec justification dans le JSON.
- **Environnements de jeu maison** (werewolf, minecraft, stanford_town, android) : domaines
  spécifiques sans transfert direct pour Omega ; on retient le mécanisme (`mgt-env-multi`,
  registres d'API read/writable), pas les implémentations.

## 6. Sources

- Clone lecture seule : `/home/tayk0oo/omega-re/metagpt` — toutes les refs machine
  (`metagpt/<fichier>:<lignes>`) viennent de ce clone.
- Licence MIT : `LICENSE` du clone.
- Preuve REA : `docs/omega/research/rea/evidence/intake-metagpt.summary.json` — corrobore
  (`stars=70887`, `language=Python`, `license=MIT`, `cloned=true`) mais reste générique
  (249 nœuds / 325 arêtes, 7 limitations déclarées, provider `rea-javascript-application`,
  confidence `derived`, analyse effectuee comme une `javascript application` alors que le
  projet est Python) : elle ne prouve pas finement chaque caractéristique. Chaque ref du
  JSON a donc ete verifiee manuellement dans le clone ; la preuve servit d'enrôlement du
  repo uniquement (digest sha256 pour vérification future).
- Repo GitHub : https://github.com/geekan/MetaGPT
