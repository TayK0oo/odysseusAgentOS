# RULES-OMEGA — Règles directrices de la session

> **Portée :** s'applique à *toute* la branche `feat/agentos-omega`.
> **Autorité :** primauté sur toute autre convention locale du dépôt en cas de conflit.
> **Dernière révision :** 2026-10-15.

---

## 0. Objectif unique (ne jamais le perdre)

> **Construire, brique par brique et de façon traçable, un HARNAIS D'AGENT universel
> où *chaque* composant est un plugin interchangeable, configurable et remplaçable
> intelligemment — le moteur en Rust, l'interface en React/TS — piloté par un
> routeur de modèles multi-fournisseurs notés par la communauté.**

Trois mots d'ordre : **interchangeable · traçable · économique.**

---

## 1. Règle zéro — Everything is a plugin

1. **Toute brique est un plugin** derrière un port (interface) et chargée par un registre.
   Rien n'est « en dur ». Si ce n'est pas remplaçable, ce n'est pas terminé.
2. **Un contrat, N implémentations.** Chaque port expose un *trait Rust* (cœur) et une
   *interface TS* (UI). Les implémentations sont découvertes, pas codées en dur.
3. **Zéro dépendance circulaire** entre briques. Le sens des dépendances va de
   l'orchestration → les ports → les adaptateurs. Jamais l'inverse.
4. **Configuration > code.** Un comportement change par configuration/manifeste,
   jamais par réécriture. (Déclaratif d'abord : TOML/YAML/JSON schema-validé.)
5. **Capability-first.** Une brique déclare une *capacité* (`routing`, `memory`,
   `search`, `sandbox`…), pas une marque. Le routeur choisit l'implémentation.

## 2. Architecture

1. **Hexagonal / ports & adaptateurs (« pieuvre »).** Le noyau métier ne connaît
   que des ports. Tout le reste (LLM, DB, FS, réseau) est un adaptateur enfichable.
2. **SOLID** appliqué littéralement :
   - **S**RP : une brique = une responsabilité = un plugin = un test.
   - **O**CP : étendre = ajouter un plugin, pas modifier le noyau.
   - **L**SP : toute implémentation d'un port est substituable sans surprise.
   - **I**SP : ports fins et spécialisés (pas de « port-dieu »).
   - **D**IP : le noyau dépend d'abstractions, jamais d'implémentations.
3. **Microservices là où c'est justifié** (isolation, mise à l'échelle, crash-boundary :
   sandbox, reverse-eng, indexation) ; **monolithe modulaire** sinon. On ne
   distributionne pas par dogme.
4. **Performance = contrainte d'architecture.** Cœur critique en **Rust**. Pas d'`await`
   bloquant qui traverse 5 couches. Budget de latence et de tokens mesurés, pas supposés.
5. **Zéro-friction frontière.** Toute frontière inter-brique (Rust↔TS, service↔service)
   a un schéma typé et versionné (JSON Schema / serde / protobuf).

## 3. Économie & ressources

1. **Mentalité économique d'abord.** Préférer le gratuit, l'open-source et le
   self-hosted *compris en profondeur*. Payer = dernier recours justifié.
2. **Le coût de calcul va au SYSTÈME, pas à l'agent.** Un besoin déterministe
   (parsing, scoring, indexation, diff, grep) est un *script/outil*, jamais un appel LLM.
   « Si ça s'écrit en 20 lignes de Rust/Python → ce n'est pas une tâche d'agent. »
3. **Budget avant action.** Chaque appel LLM est estimé, tracé, plafonné. Dépassement = arrêt.
4. **Réutilisation avant réinvention.** Chercher la meilleure méthode existante
   (recherche web/veille) avant d'écrire. Documenter la source.
5. **Aucune limite morale imposée pour usage strictement personnel.** Si un sujet est
   bloqué par un fournisseur, router vers un modèle self-hosted. Ne jamais dépendre
   d'un seul fournisseur.

## 4. Traçabilité & vérité

1. **Provenance obligatoire.** Tout fait est tagué `[stated]` / `[observed]` / `[inferred]`.
2. **Pas de doc qui ment.** Un document décrit l'état *réel*, vérifiable. Un écart
   constaté est un signal dans le document, pas un silence.
3. **Chaque décision → un ADR** (`docs/omega/adr/NNNN-*.md`), daté, avec alternatives.
4. **Tout artefact a une source.** Fichier, URL, commande, ou run-id. Sinon → `[inferred]`.

## 5. Exécution & parallélisme

1. **Parallélisation par défaut.** Toute suite d'actions indépendantes part en un seul bloc.
2. **L'orchestrateur ne fait pas.** Il décompose, délègue, vérifie. (Zéro exécution directe
   pour les gros chantiers ; l'orchestrateur peut produire de la doc/plan.)
3. **Vérification indépendante.** Deux agents distincts (Reviewer + TestEngineer) pour
   tout livrable critique. On ne s'auto-certifie pas.
4. **Docs d'abord, code ensuite.** On ne code pas une brique dont le port, le contrat
   et le test d'acceptation ne sont pas écrits.
5. **Petits pas vérifiables.** Un commit = une intention + sa preuve (test/mesure).
6. **Parler peu, agir.** Pas de remplissage. Réponses denses, orientées livrable.

## 6. Conventions

- **Docs/ADR/plans :** français, sans accents dans les *identifiants* (chemins, noms de
  fichiers, slugs) — accents OK dans la prose.
- **Code :** identifiants en anglais ; commentaires en français seulement si utile.
- **Commits :** Conventional Commits (`feat|fix|docs|refactor|chore|test(scope): …`).
- **Branches :** `feat/omega-*`, `adr/omega-*`, `research/omega-*`.
- **Chemins :** docs vision `docs/omega/` · recherche `docs/omega/research/` ·
  ADR `docs/omega/adr/` · règles `.opencode/rules/` · mémoire `docs/omega/MEMORY.md`.

## 7. Ce qu'on ne fait PAS (garde-fous de la session)

- ❌ Réécrire le noyau existant avant d'avoir inventorié ce qui marche.
- ❌ Ajouter un fournisseur/modèle en dur sans passer par le registre.
- ❌ Confier à un agent une tâche déterministe (parser, calculer, formater).
- ❌ Documenter une fonctionnalité non mesurée comme « faite ».
- ❌ Introduire un couplage caché `UI → logique` ou `service → service` sans port.
