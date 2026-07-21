# Reference — Glossaire

> Terminologie partagée pour tout le projet. Basé sur le SFD v3.0 (§9) et l'INDEX-MAITRE.

## A
- **Agent** — composant logiciel autonome, ayant un rôle, des outils et un modèle d'IA
- **AgentDispatcher** — dispatch les agents `.opencode/` par phase canonique (gated OFF)
- **Artefact** — fichier créé par le système avec support de stockage persistant

## B
- **Budget** — limite explicite par projet (itérations, tokens, coût, temps)
- **BUILD** — phase 4 de la boucle canonique : implémentation avec accès complet

## C
- **CBM** — Codebase Memory MCP : graphe de code (11,749 nœuds)
- **CanonicalLoop** — boucle 7 phases (CLASSIFY→KNOW→PLAN→BUILD→QUALITY→AUTOEVAL→MEMORY_OBSERVE), gated OFF
- **Channel Gateway** — bus inbound/outbound pour Discord, Telegram, Email
- **ChromaDB** — base vectorielle utilisée pour RAG et mémoire
- **CLASSIFY** — phase 1 : classification du risque
- **Compaction** — résumé automatique de l'historique pour libérer du contexte
- **Contexte** — ensemble des informations pertinentes pour une étape donnée

## D
- **DESTRUCTIVE** — niveau de risque maximal (suppression irréversible)
- **Destructive gate** — bloque les commandes shell dangereuses (ON par défaut)
- **Drift** — dérive de performance mesurée par l'Observer

## E
- **EXEC** — niveau de risque : exécution de code
- **Exécution durable** — garantie qu'une action survit à une panne

## G
- **Gate** — porte de sécurité (destructive, phase-lock, admin, tool policy)

## K
- **Kill-switch** — variable d'env `ODYSSEUS_*` activant/désactivant une fonctionnalité (38 au total)
- **KNOW** — phase 2 : récupération mémoire/connaissance
- **Kroki** — service de rendu de diagrammes (Mermaid, PlantUML)

## L
- **LangGraph** — implémentation alternative de la boucle en graphe 7 nœuds (gated OFF)
- **LiteLLM** — abstraction multi-provider pour les appels LLM

## M
- **MCP** (Model Context Protocol) — standard ouvert d'intégration outils/agents
- **Mem0** — couche mémoire avec extraction automatique de faits
- **MEMORY_OBSERVE** — phase 7 : distillation mémoire et observabilité

## O
- **Observer** — analyse continue du drift (toujours actif)
- **OPA** (OpenPolicyAgent) — moteur de politiques Rego (gated OFF)
- **Orchestrateur** — coordonne les phases et les agents

## P
- **Phase** — étape du cycle de vie d'un projet (7 phases canoniques)
- **Phase-lock** — restriction des permissions selon la phase en cours
- **PhaseTracker** — pont entre la boucle live et le phase-lock (gated OFF)
- **PLAN** — phase 3 : planification

## Q
- **Qdrant** — base vectorielle alternative (Rust, filtrage payload)
- **QUALITY** — phase 5 : tests et lint

## R
- **RAG** (Retrieval-Augmented Generation) — recherche vectorielle + génération
- **RRF** (Reciprocal Rank Fusion) — fusion hybride vecteur + BM25 (gated OFF)
- **READ** — niveau de risque minimal (lecture seule)

## S
- **SFD** — Spécification Fonctionnelle Détaillée (`SFD.md`, v3.0, 1065 lignes)
- **Skill** — module de compétence (extrait de l'expérience ou pré-encodé)
- **SPOF** — Single Point of Failure
- **stream_agent_loop** — fonction cœur de 3637 lignes, orchestrateur de fait

## T
- **Tailwind CSS** — framework CSS utility-first (v4, 16.3 KB compilé)
- **Trace** — unité structurée d'observabilité (JSONL)
- **Trinity** — CBM + Graphify + Obsidian (knowledge graph)

## W
- **Worktree** — branche de travail d'un projet (Git worktree)
- **Workspace** — conteneur racine (configuration globale, projets)
- **WRITE** — niveau de risque : écriture sur disque

## Z
- **Zen Router** — routage intelligent vers l'API OpenCode Zen

---

→ Voir aussi : [Résumé SFD](sfd-summary.md) · [Index maître](index-master.md) · `SFD.md` §9
