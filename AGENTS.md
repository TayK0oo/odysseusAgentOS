# AGENTS.md — Orientation pour les agents (branche `feat/agentos-omega`)

> Ce fichier est la **première lecture** de tout agent qui ouvre ce dépôt.
> Il décrit ce qu'est le projet, où sont les choses, et les règles à respecter.
> **Lis ensuite, dans l'ordre :**
> 1. `docs/omega/rules/RULES-OMEGA.md` — règles directrices (autorité).
> 2. `docs/omega/MEMORY.md` — mémoire persistante (état, décisions, credentials).
> 3. `docs/omega/00-VISION-OMEGA.md` — la vision cible.
> 4. `docs/VISION-INTEGRATION.md` — la vision antérieure du dépôt (contexte).

---

## 1. Qu'est-ce que ce projet

Un **harnais d'agent universel** où **chaque composant est un plugin interchangeable**.
Le socle de départ est le dépôt `odysseusAgentOS` (fork d'`arcahyadi/odysseus`) : un
assistant/agent self-hosted en Python. La branche `feat/agentos-omega` prolonge ce
travail pour en faire un harnais **modulaire, traçable, économique**, avec un **moteur
en Rust** et une **UI React/TS**.

**Slogan :** *Everything is a plugin.*

## 2. Les 6 piliers

1. **Routing intelligent** — registre multi-fournisseurs, modèles notés par la
   communauté, choix par type de tâche (vitesse ↔ réflexion).
2. **Reverse-engineering de harnais** — extraire les meilleures fonctionnalités des
   harnais existants et les reconstruire proprement.
3. **Everything is a plugin** — ports + adaptateurs, SOLID, « pieuvre », microservices
   ciblés. Zéro couplage caché.
4. **Rust core / React-TS UI** — performance et robustesse au cœur, DX au bord.
5. **Économie & traçabilité** — le coût de calcul va au *système* (scripts/outils), pas
   à l'agent ; tout est mesuré, prouvé, daté.
6. **Adaptatif & évolutif** — le système apprend de chaque run (feedback, scoring),
   sans jamais dépendre d'un fournisseur unique (self-hosted au besoin).

## 3. Arborescence clé

| Chemin | Rôle |
|---|---|
| `docs/omega/` | Tout l'espace « Omega » (vision, règles, ADR, recherche) |
| `docs/omega/rules/` | Règles directrices de session |
| `docs/omega/adr/` | Décisions d'architecture (ADR) |
| `docs/omega/research/` | Recherche (TikTok @0xloucash, harnais, modèles) |
| `docs/omega/MEMORY.md` | Mémoire persistante (source de vérité) |
| `.opencode/rules/` | Règles chargées par l'agent OpenCode |
| `obsidian-vault/loucash/` | Vault Obsidian de la recherche TikTok |
| `src/ services/ core/ routes/` | Code existant du socle (Python) |
| `docs/master-ref/01-SFD-v3.1.md` | Spécification fonctionnelle de référence |

## 4. Commandes utiles

```bash
# État & branche
git branch --show-current          # doit afficher feat/agentos-omega
git log --oneline -10

# Qualité (socle) — après activation du venv
ruff check . && pytest -q

# Recherche TikTok (enrichissement)
python3 docs/omega/research/loucash/enrich.py
```

## 5. Conventions (résumé — détail dans RULES-OMEGA.md)

- **Commits :** Conventional Commits (`feat|fix|docs|refactor|chore|test(scope): …`).
- **Docs :** français, sans accents dans les identifiants/chemins.
- **Une décision = un ADR.** Provenance taguée `[stated]/[observed]/[inferred]`.
- **Docs d'abord, code ensuite.** Un port, un contrat et un test d'acceptation avant tout code.
- **Zéro exécution directe par l'orchestrateur.** Décomposer, déléguer, vérifier.
- **Le déterministe n'est jamais une tâche d'agent** (parser/calculer/formater → script).

## 6. Interdits

- ❌ Modèle/fournisseur en dur hors registre.
- ❌ Couplage `UI → logique` ou `service → service` sans port.
- ❌ Documenter une feature non mesurée comme « faite ».
- ❌ Réécrire le noyau avant d'avoir inventorié l'existant.
