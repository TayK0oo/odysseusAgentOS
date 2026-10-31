# AGENTS.md — Orientation pour les agents (branche `feat/agentos-omega`)

> Ce fichier est la **première lecture** de tout agent qui ouvre ce dépôt.
> Il décrit ce qu'est le projet, où sont les choses, et les règles à respecter.
> **Lis ensuite, dans l'ordre :**
> 1. `docs/omega/rules/RULES-OMEGA.md` — règles directrices (autorité).
> 2. `docs/omega/MEMORY.md` — mémoire persistante (état, décisions, credentials).
> 3. `docs/omega/00-VISION-OMEGA.md` — la vision cible.
> 4. `docs/VISION-INTEGRATION.md` — la vision antérieure du dépôt (contexte).

---

## 0. RÈGLE GLOBALE — modèles des sous-agents (impérative)

> **Tout sous-agent part sur un modèle GRATUIT.** Un modèle payant n'est utilisé
> qu'en **exception justifiée**, uniquement parmi `opencode-go/*` (inclus dans
> l'abonnement Go), et seulement après un **échec gratuit constaté** — cité dans
> le message de délégation. Les modèles `opencode/*` payants (Zen = solde) sont
> **interdits**.

- Référence : `docs/omega/rules/MODEL-POLICY.md` · machine : `docs/omega/rules/model-policy.json`
- Vérification : `python3 omega/scripts/model_policy.py --check` (liste live) ou `--for <usage>`
- Gratuits par défaut : `opencode/space-bunny-free` (code), `opencode/deepseek-v4.1-flash-free`
  (analyse), `opencode/mimo-v2.6-flash-free` / `opencode/nemotron-3.5-lightning-free`
  (léger), `opencode/longcat-2.5-preview-free` (multimodal/1M), `opencode/jev-1.13-free` (réponses typées).
- Quota Go : 20 %/5 h, 50 %/semaine, 100 %/mois **par modèle** — ne pas l'épuiser avec des sous-agents.

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

### Lancer le système complet (Rust + cockpit UI)

```bash
# Prerequis : Ollama en fond + le modele local
ollama pull qwen2.5-coder:7b
OLLAMA_NUM_CTX=32768 ollama serve     # contexte 32k (defaut Ollama = 4096)

# Une seule commande : build UI + serveur, puis sert tout sur une origine
omega/scripts/run-local.sh            # -> http://127.0.0.1:8787  (port 8787)
omega/scripts/run-local.sh 9000       # port explicite
```

Le cockpit (React) et l'API (`/api/health|plugins|run|trace/:id`) sont servis par
`omega-server` sur la même origine. Onglets : **Cockpit** (run local de bout en bout
+ réponse du modèle), **Plugins** (capacités du registre), **Router** (décision Pareto),
**Traces** (rejeu d'un journal sans ré-exécuter), **Contexte** (budget/estimation).

Repli sans Ollama : `cd omega && cargo run -p omega-app --example live_ollama`.

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
