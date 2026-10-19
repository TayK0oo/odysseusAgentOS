# Règles de session — AgentOS Omega

> Fichier chargé automatiquement par OpenCode. Référence complète :
> `docs/omega/rules/RULES-OMEGA.md`. Mémoire : `docs/omega/MEMORY.md`.

## Rappels essentiels

1. **Everything is a plugin** — toute brique derrière un port + un registre. Rien en dur.
2. **Moteur en Rust** (workspace de crates), **UI en React/TS** (`omega/ui/`).
3. **Le déterminisme va au système** (scripts/outils), jamais à un agent.
4. **Docs d'abord, code ensuite** — un port, un contrat et un test d'acceptation avant tout code.
5. **Traçabilité** — provenance `[stated]/[observed]/[inferred]` ; une décision = un ADR.
6. **Économie** — gratuit/self-hosted d'abord ; budget estimé, tracé, plafonné.
7. **Interdits** — modèle/fournisseur en dur ; couplage `UI→logique` ou `service→service`
   sans port ; documenter une feature non mesurée comme faite.

## Répertoire de travail

`/home/tayk0oo/Projets/odysseusAgentOS-omega` (worktree isolé).
Le dépôt principal `/home/tayk0oo/Projets/odysseusAgentOS` peut être utilisé par une
**autre session** : ne pas y opérer Omega.
