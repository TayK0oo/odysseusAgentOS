# PROJECT — odysseusAgentOS-omega (Omega)

## Project

**Omega — harnais d'agent universel, tout est plugin**

Branche `feat/agentos-omega` du dépôt `odysseusAgentOS` (fork de `arcahyadi/odysseus`).
Moteur en Rust, UI React/TS séparée, architecture hexagonale à ports/adaptateurs, où
chaque brique (LLM provider, contexte, mémoire, outils, sandbox, routing, telemetry) est
un plugin interchangeable. Le coût déterministe est payé par le système, jamais par le
modèle. Chaque décision est traçable et rejouable.

**Core Value:** Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver,
apprend de chaque run, ne dépasse jamais ses limites — sans intervention humaine
constante. Tout est un plugin : aucun cœur privilégié.

## Branch / Workspace

- Branche de travail : `feat/agentos-omega` (worktree isolé)
- Répertoire : `/home/tayk0oo/Projets/odysseusAgentOS-omega`
- Dépôt principal : `/home/tayk0oo/Projets/odysseusAgentOS` (reste sur `feat/inventaire-global-v1`)
- Distant : `github.com/TayK0oo/odysseusAgentOS`

## Constraints

- **Container-first** : tout développement se fait dans le conteneur `opencode-sandbox`
  (`http://127.0.0.1:49376`). L'hôte OpenCode Desktop n'est qu'une cible post-validation.
- **Modèles gratuits par défaut** pour les sous-agents (`opencode/*-free`) ; les payants
  `opencode/*` (Zen) sont interdits ; `opencode-go/*` seulement en exception justifiée.
- **Un écrivain par arbre** : fichiers disjoints entre agents (incident vécu).
- **Vérification indépendante** : Reviewer + TestEngineer distincts pour tout code critique.
- **Déterminisme au système** : indexation, recherche, parsing = scripts/outils, jamais LLM.
- **Provenance** : `[stated]/[observed]/[inferred]` partout (mémoire, docs, réponses).

## Decisions (locked)

Voir `docs/omega/adr/` et `docs/omega/MEMORY.md` §6 (D1→D15).

- D5 — Moteur : réécriture complète en Rust (ADR-0001).
- D6 — UI : nouvelle app React/TS à côté (ADR-0002).
- D9 — Architecture plugin/capacités (ADR-0003).
- D10 — Réconciliation dette W4 : source unique `omega-types`, port `TaskClassifier` dans `omega-ports`.
- D14/D15 — W6 incrément 1 : crate `omega-context` (29→40 tests).

## Stack

- Rust (workspace `omega/`, 16 crates) + React/TS (Vite) UI.
- GSD Core pour le workflow (`.planning/`).
- REA (reverse-engineering), MCP, Ghidra toolchain.

## Non-goals (court terme)

- Pas de réutilisation du socle Python comme moteur (sert de catalogue de besoins).
- Pas de microservices par dogme.
- Pas de génération média/pub ni navigateur autonome visible.

## Source of truth

`docs/omega/MEMORY.md` (état, décisions, credentials, backlog) + `docs/omega/rules/RULES-OMEGA.md`.
Ce `.planning/` est bootstrappé depuis ces docs via `/gsd-ingest-docs` (2026-10-17).