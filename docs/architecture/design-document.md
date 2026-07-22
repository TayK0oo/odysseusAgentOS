# AGENT OS — DOCUMENT DE CONCEPTION OFFICIEL

**Version :** 1.0 — Juillet 2026
**Statut :** Approuvé pour implémentation
**Base :** [SFD v3.0](../../SFD.md) — 22 principes invariants, 20 modules fonctionnels
**Alignement :** 45% → cible 100%

---

## 1. Introduction et vision

### 1.1 Mission

Agent OS est un assistant autonome de conduite de projet. Il comprend une demande en langage naturel, élabore un plan, l'exécute de façon fiable, et apprend de chaque expérience. Il n'est pas un simple wrapper LLM : c'est une plateforme modulaire où le raisonnement (IA), la fiabilité d'exécution (durabilité), la mémoire (provenance) et l'observabilité (traces) sont des couches distinctes et indépendantes.

### 1.2 Parties prenantes et cas d'usage

| Acteur | Besoin principal | Cas d'usage SFD |
|--------|-----------------|-----------------|
| Utilisateur final | Déléguer un projet et suivre son avancement | UC-01 à UC-19 |
| Administrateur | Configurer modèles, outils, sécurité, budgets | UC-04, UC-05 |
| Agents internes | Exécuter des tâches dans un périmètre défini | UC-10, UC-11 |
| Auditeur externe | Reconstituer la chaîne de décision | UC-12 |

### 1.3 Objectifs mesurables

| Objectif | Métrique | Cible | Source |
|----------|----------|-------|--------|
| Survie aux pannes | Taux de reprise sans perte d'état | > 99% | NF-04 |
| Latence perçue | Temps de réponse interaction simple | < 2s | NF-01 |
| Projets simultanés | Workspace unique | ≥ 10 | NF-02 |
| Sobriété architecturale | Taux de décision mono-agent | > 80% des tâches | NF-10 |
| Intégrité mémoire | Dégradation après N mises à jour | 0% (deltas purs) | NF-11 |
| Provenance | Faits tagués | 100% | NF-12 |
| Vie privée | Données protégées persistées | 0 | NF-13 |
| Couverture de traces | Composants sans traces structurées | 0 | NF-07 |

---

## 2. Architecture générale

### 2.1 Diagramme de contexte

```
┌──────────────┐     ┌─────────────────────────────────────────────────────────┐
│  Utilisateur │────▶│                  AGENT OS WORKSPACE                       │
│  (web/CLI/   │     │                                                          │
│   messagerie)│     │  ┌─────────────────────────────────────────────────────┐ │
└──────────────┘     │  │              Gateway multi-canal                     │ │
                     │  │  Web (FastAPI) · CLI · Discord · Email · Telegram    │ │
┌──────────────┐     │  └──────────────────────────┬──────────────────────────┘ │
│  Admin       │────▶│                             │                             │
└──────────────┘     │  ┌──────────────────────────▼──────────────────────────┐ │
                     │  │                 Noyau d'orchestration                │ │
                     │  └──────────────────────────┬──────────────────────────┘ │
┌──────────────┐     │                             │                             │
│  Services    │◀────│  ┌──────────────────────────▼──────────────────────────┐ │
│  externes    │     │  │   Exécution durable · Agents · Outils · Mémoire     │ │
│  (APIs, DB)  │     │  └─────────────────────────────────────────────────────┘ │
└──────────────┘     │                                                          │
                     │  Observabilité & Gouvernance (transversal)                │
                     └──────────────────────────────────────────────────────────┘
```

### 2.2 Vue macroscopique des couches

La SFD §7 définit 6 couches. La conception les affine en 7 couches avec séparation claire des responsabilités :

| Couche | Responsabilité | Composants clés |
|--------|---------------|-----------------|
| **Gateway** | Entrées/sorties multi-canal, multimodalité | Convertisseur de messages, Routeur de modalité, Visualiseur inline |
| **Noyau** | Orchestration, contexte, planification, routage | Context Engine, Planificateur, Routeur de modèles, Décideur de décomposition, Gestionnaire de préférences |
| **Exécution durable** | Fiabilité, retry, reprise après panne | Moteur de workflows, Activités avec politiques de retry, Compensation (saga) |
| **Agents** | Raisonnement spécialisé | Chef d'orchestre, Agents spécialisés, Sous-agents de vérification |
| **Outils** | Capacités d'action, découverte | Serveurs MCP, Registre, tool_search, Skills, Outils internes |
| **Persistance** | Mémoire, projets, conversations, préférences | Système de fichiers mémoire, Base transversale, Historique, Artefacts |
| **Observabilité** | Traces, évaluation, gouvernance | Traces structurées, Évaluation continue, Tableaux de bord, Classification |

### 2.3 Principes architecturaux

Chaque principe de la SFD se traduit en une règle de conception concrète :

| Principe SFD | Règle de conception |
|-------------|-------------------|
| P1 — Risque modifie la boucle | Le Superviseur de sécurité intercepte toute action avant exécution ; les actions destructives exigent approbation humaine |
| P3 — Contexte construit | Le Context Engine ne transmet jamais le contexte brut ; il filtre par pertinence, phase et rôle |
| P11 — Commencer simple | L'architecture multi-agent est câblée mais dormant (kill-switch OFF). Activation conditionnée à preuve chiffrée |
| P12 — Découpage par contexte | Les frontières entre agents suivent l'isolation du contexte, jamais les rôles métier |
| P13 — Contexte = budget | La compaction est déclenchée par seuil de tokens, pas par nombre de tours |
| P14 — Actions longues durables | Toute action > 5s ou coûteuse à refaire passe par la couche d'exécution durable |
| P15 — Observabilité dès conception | Chaque composant émet des spans structurées ; pas de `print()` ni de log ad-hoc |
| P16 — Provenance explicite | Toute écriture mémoire inclut un tag de provenance obligatoire |
| P18 — Lire avant d'écrire | Contrôle de concurrence versionné : `memory_read` → jeton → `memory_write(if_version=jeton)` |

### 2.4 Alignement avec la stack technique existante

Le design s'appuie sur la stack déjà en production (cf. [overview.md](overview.md)) :

| Composant SFD | Implémentation actuelle | Écart à combler |
|--------------|------------------------|-----------------|
| Gateway | FastAPI (app.py, 55 modules de routes) | Ajouter le routeur de modalité et le visualiseur inline |
| Noyau | `stream_agent_loop` (3637 lignes) | Remplacer progressivement par `CanonicalLoop` (gated OFF) |
| Exécution durable | Aucun | **Module entier à créer** |
| Mémoire | `services/memory/` (9 fichiers) + ChromaDB | **Refonte complète** : taxonomie, provenance, versions |
| Outils | 6 serveurs MCP built-in + 5 Docker | Ajouter registre MCP, tool_search, suggest_connectors |
| Observabilité | LangFuse + OpenTelemetry | Standardiser les spans `gen_ai.*` |
| Préférences | Aucun | **Module entier à créer** |
| Recherche conversations | Meilisearch (gated) | Activer, ajouter signaux linguistiques |

### 2.5 Anti-patterns architecturaux

- **NE PAS** faire communiquer les agents par contexte brut — utiliser le bus de messages avec condensation (§5.1.6)
- **NE PAS** activer le multi-agent sans preuve chiffrée d'un des trois critères (§5.1.2)
- **NE PAS** traiter la mémoire comme un blob de contexte — c'est un système de fichiers structuré avec provenance
- **NE PAS** mélanger raisonnement et durabilité — ce sont deux couches distinctes avec des cycles de vie indépendants

---

## 3. Conception détaillée des modules

### 3.1 Noyau d'orchestration

#### 3.1.1 Context Engine

Le Context Engine est le chef d'orchestre de l'information. Il applique le principe P3 (contexte construit, pas déversé).

**Architecture interne :**

```
Context Engine
├── Aggregator        — collecte les sources (mémoire, historique, plan, préférences)
├── Filter            — filtre par pertinence (phase, rôle, domaine)
├── Compactor         — déclenche la compaction quand le seuil de tokens est atteint
├── Notepad           — externalise le plan et les décisions dans le stockage persistant
├── Thread Tracker    — maintient une trace d'identité par sous-projet
└── Preference Injector — injecte les préférences applicables au contexte courant
```

**Règles de compaction :**
- Seuil configurable (défaut : 80% de la fenêtre de contexte du modèle)
- Les échanges anciens sont résumés en un état condensé (décisions, faits, décisions en attente)
- La compaction préserve les décisions et faits essentiels, pas le verbatim
- Déclenchée automatiquement, pas manuellement

**Règles du bloc-notes externalisé :**
- Le plan est écrit dans le stockage persistant dès sa création
- Les décisions clés sont écrites immédiatement (pas en fin de session)
- Format : `project/.planning/notepad.yaml` avec sections horodatées

**Isolation par sous-agent :**
- Chaque sous-agent reçoit un contexte filtré : uniquement les informations pertinentes à son rôle
- Le filtrage est fait par le Context Engine, pas par le sous-agent lui-même
- Un sous-agent de vérification reçoit les critères de succès, pas l'historique d'implémentation

**Anti-patterns :**
- **NE PAS** donner le contexte brut complet à un sous-agent
- **NE PAS** compter sur le contexte de conversation pour retenir le plan (utiliser le bloc-notes)
- **NE PAS** déclencher la compaction manuellement — c'est automatique

#### 3.1.2 Planificateur

Transforme une demande en langage naturel en une arborescence exécutable.

**Modèle de données :**

```yaml
mission:
  id: "m-001"
  statement: "Créer un système de backup automatisé"
  objectives:
    - id: "o-001"
      description: "Sauvegarde incrémentale quotidienne"
      criteria: ["Backup < 5 min", "Restore < 10 min", "Intégrité vérifiable"]
      subprojects:
        - id: "sp-001"
          name: "Module de snapshot"
          tasks:
            - id: "t-001"
              description: "Implémenter le snapshot filesystem"
              agent: "conductor"
              tools: ["filesystem", "shell"]
              budget: {tokens: 50000, cost: 0.50}
              done_when: ["Tests passent", "Benchmark < 5 min"]
```

**Règles de planification :**
- Le plan est stocké en YAML dans `project/plan.yaml` dès sa création
- Chaque élément a un identifiant unique, une description, des critères de finition, un budget
- Le plan est révisable : le chef d'orchestre peut proposer des ajustements soumis aux mêmes validations que les actions (P8)
- En cas de blocage, un mécanisme de débat interne (plusieurs personas) est disponible mais pas actif par défaut (P11)

**Anti-patterns :**
- **NE PAS** planifier en mode multi-agent par défaut
- **NE PAS** créer des tâches sans critères de finition mesurables

#### 3.1.3 Routeur de modèles

Associe un type de tâche à un modèle d'IA selon une matrice multi-dimensionnelle.

**Dimensions de routage :**
- Rapidité (latence acceptable)
- Coût (budget tokens/$)
- Qualité (benchmarks, évaluations continues)
- Disponibilité (health check)

**Chaînes de repli :**
1. Modèle primaire échoue (timeout, erreur, incohérence)
2. Bascule automatique vers le secondaire
3. La bascule est elle-même une activité durable avec retry (§3.3)
4. Si tous les modèles échouent → escalade au Superviseur

**Auto-adaptation :**
- Les métriques de performance sont collectées via les traces d'observabilité
- Si les métriques se dégradent (latence, taux d'échec, coût), le système suggère un changement de routage

**Anti-patterns :**
- **NE PAS** router uniquement sur le coût le plus bas
- **NE PAS** ignorer les échecs de modèle — chaque échec est tracé et analysé

#### 3.1.4 Décideur de décomposition

Implémente la logique du §5.1.2 : rester en agent unique sauf preuve contraire.

**Algorithme de décision :**

```
fonction doit_decomposer(tache):
    si mesure_protection_contexte(tache) > 1000 tokens non pertinents:
        retourner DECOMPOSE (protection de contexte)
    si est_parallele_independant(tache):
        retourner DECOMPOSE (parallélisation)
    si nombre_outils_pertinents(tache) > 15:
        retourner DECOMPOSE (spécialisation)
    retourner AGENT_UNIQUE
```

**Mesures obligatoires :**
- Comptage des tokens de contenu non pertinent produits par sous-tâche
- Vérification de l'indépendance réelle (pas de dépendance cachée, pas d'état partagé)
- Décompte exact des outils pertinents pour la tâche (pas le total des outils disponibles)
- Traçage du multiplicateur de coût réel (tokens multi-agent / tokens agent unique)

**Anti-patterns :**
- **NE PAS** décomposer par métier (planifieur/développeur/testeur/relecteur)
- **NE PAS** activer le multi-agent par anticipation
- **NE PAS** ignorer le surcoût : 3-10x tokens en multi-agent normal, jusqu'à 15x en pattern orchestrateur-ouvriers

---

### 3.2 Système de mémoire et de fichiers

C'est le composant le plus transformateur de la v3.0. La mémoire n'est pas un blob de contexte vectoriel — c'est un système de fichiers structuré avec provenance, versionné et curé.

#### 3.2.1 Taxonomie des fichiers

```
/memory/
├── profile.md              # Identité stable (vérité dans 3 mois ?)
├── preferences.md          # Comportementales + contextuelles
├── topics/                 # Faits par domaine
│   ├── food.md
│   ├── schedule.md
│   └── technology.md
├── areas/                  # Projets, responsabilités, incidents en cours
│   ├── auth-redesign.md
│   └── oncall.md
└── people/                 # Contexte relationnel (pas un dossier)
    └── collaborateur.md
```

**Règle cardinale :** un fait sur le sujet X va dans le fichier de X, pas dans le fichier qu'on a déjà ouvert.

#### 3.2.2 Format de fichier avec frontmatter

```yaml
---
name: <slug>
description: <une ligne — ce que ça couvre et quand le lire>
sources: [chat, claude-code, api]
aliases: [autre nom, raccourci]
version: <jeton 12 caractères>
---

- [stated] fait énoncé directement par l'utilisateur
- [observed] fait constaté par le système
- [inferred] fait déduit avec niveau de confiance
```

#### 3.2.3 Mécanisme de provenance

| Tag | Signification | Qui l'écrit | Jamais en mode conversation |
|-----|--------------|------------|---------------------------|
| `[stated]` | L'utilisateur l'a dit explicitement | L'agent en mode conversation | — |
| `[observed]` | Le système l'a constaté | Sous-systèmes, autres surfaces | **Jamais** |
| `[inferred]` | Déduit avec confiance | Agent d'analyse | **Jamais** |

**Règles strictes :**
- En mode conversation, seul `[stated]` est autorisé
- On ne convertit jamais un `[stated]` en généralisation ("aime X" → "aime toute la catégorie")
- On ne stocke pas les suggestions du système, ses conclusions, ses résultats de recherche

#### 3.2.4 Opérations CRUD versionnées

| Opération | Signature | Condition |
|-----------|-----------|-----------|
| `memory_read` | `(path) → (content, version_token)` | Retourne contenu + jeton de 12 caractères |
| `memory_write` | `(path, content, if_version)` | `if_version="new"` pour création, ou jeton pour mise à jour |
| `memory_append` | `(path, content, if_version)` | Fichier doit exister ; pas de doublon |
| `memory_str_replace` | `(path, old, new, if_version)` | `old` doit matcher exactement à un seul endroit |
| `memory_delete` | `(path, if_version)` | Uniquement sur demande explicite de l'utilisateur |
| `memory_list` | `(prefix) → [paths]` | Découverte avant lecture |

#### 3.2.5 Contrôle de concurrence

```
1. Agent A: memory_read("/topics/food.md") → (content, "a1b2c3d4e5f6")
2. Agent B: memory_read("/topics/food.md") → (content, "a1b2c3d4e5f6")
3. Agent A: memory_append("/topics/food.md", "- [stated] aime les sushis", "a1b2c3d4e5f6") → OK, nouveau jeton "x7y8z9..."
4. Agent B: memory_append("/topics/food.md", "- [stated] préfère le thé vert", "a1b2c3d4e5f6") → REJETÉ
   → Retourne contenu actuel avec nouveau jeton "x7y8z9..."
   → Agent B doit fusionner ses changements et réessayer avec "x7y8z9..."
```

#### 3.2.6 Règles temporelles d'écriture

- **Écrire immédiatement** : chaque fait durable est écrit dans l'échange même où il est énoncé
- **Écrire avant de poser une question** : si on s'apprête à demander une clarification, on écrit d'abord ce qu'on a déjà appris
- **Ne pas attendre un "OK"** : l'absence de confirmation explicite ne bloque pas l'écriture
- **Ne jamais annoncer l'écriture** : le système écrit silencieusement ; l'interface notifie

#### 3.2.7 La boucle Génération → Réflexion → Curation

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ GENERATION  │────▶│  REFLEXION  │────▶│  CURATION   │
│ (exécute)   │     │ (extrait     │     │ (intègre    │
│             │     │  la leçon)   │     │  dans la    │
│             │     │              │     │  base)      │
└─────────────┘     └─────────────┘     └─────────────┘
  Produit une         Extrait une         Ajoute/modifie
  trajectoire         leçon réutilisable  une fiche (delta)
  (succès/échec)      pas un résumé       pas de réécriture
```

**Format de fiche de compétence :**

```yaml
id: skill-<uuid>
nom: "<titre descriptif>"
description: "<ce que ça couvre>"
domaine: ["catégorie"]
prerequis: ["outil", "connaissance"]
conditions_succes: ["critère mesurable"]
compteur:
  aidant: <int>
  nuisant: <int>
  derniere_maj: "<date>"
exemples:
  - contexte: "<situation>"
    resultat: "<issue>"
```

**Anti-patterns mémoire :**
- **NE PAS** réécrire toute la base de connaissance en une seule opération
- **NE PAS** stocker des attributs protégés (origine, religion, orientation, santé...)
- **NE PAS** stocker des informations sensibles (croyances politiques, données financières...)
- **NE PAS** mentionner l'infrastructure mémoire dans les réponses ("basé sur ce que je sais de toi")
- **NE PAS** appliquer un fait mémoire qui ne change rien à la réponse (principe "earn its place")

---

### 3.3 Exécution durable

La couche qui garantit qu'une action engagée va jusqu'au bout, même à travers une panne. Indépendante du modèle d'IA.

#### 3.3.1 Architecture

```
Couche d'exécution durable
├── Workflow Engine    — état persistant, reprise après crash
├── Activity Executor  — politiques de retry, idempotence
├── Saga Coordinator   — compensation en ordre inverse
└── Signal Waiter      — attente d'approbation humaine longue durée
```

#### 3.3.2 Modèle de workflow

```yaml
workflow:
  id: "wf-001"
  name: "Déploiement avec approbation"
  activities:
    - id: "build"
      action: "docker build"
      retry_policy:
        max_attempts: 3
        backoff: exponential
        initial_delay_ms: 1000
      compensation: "docker rmi <image>"
    - id: "approve"
      action: "wait_for_human_approval"
      timeout_days: 7
    - id: "deploy"
      action: "docker compose up"
      retry_policy:
        max_attempts: 2
        backoff: constant
      compensation: "docker compose down"
  saga:
    on_failure: "compensate_reverse"
```

#### 3.3.3 Politiques de retry

| Paramètre | Description | Défaut |
|-----------|-------------|--------|
| `max_attempts` | Nombre maximal de tentatives | 3 |
| `backoff` | Stratégie : constant, linear, exponential | exponential |
| `initial_delay_ms` | Délai initial entre tentatives | 1000 |
| `max_delay_ms` | Délai maximal | 60000 |
| `retry_on` | Erreurs déclenchant un retry | timeout, 5xx, connection_refused |

**Règle :** un échec transitoire (timeout réseau, erreur 5xx) est absorbé par la couche d'exécution sans remonter au chef d'orchestre.

#### 3.3.4 Compensation (pattern saga)

Chaque activité déclare son action de compensation :

```
Activité A: créer_facture()    → compensation: annuler_facture()
Activité B: envoyer_email()    → compensation: envoyer_email_retractation()
Activité C: mettre_a_jour_crm() → compensation: restaurer_etat_crm()
```

Si C échoue après A et B : compenser B, puis A (ordre inverse).

#### 3.3.5 Points d'approbation longue durée

- Un workflow se met en attente d'un signal externe (approbation humaine)
- Durée d'attente arbitraire (heures, jours) sans consommation de ressources
- Reprise automatique à réception du signal
- Le Superviseur de sécurité revalide les permissions au moment de la reprise (pas seulement au moment de la proposition initiale)

#### 3.3.6 Implémentation technique

- **Stockage d'état** : SQLite (existant) pour les workflows actifs ; chaque étape validée est journalisée
- **Reprise** : au démarrage, le Workflow Engine scanne les workflows interrompus et les reprend
- **Idempotence** : chaque activité a un identifiant unique ; le moteur vérifie avant exécution

**Anti-patterns :**
- **NE PAS** implémenter l'exécution durable comme une bibliothèque liée au code agent
- **NE PAS** faire confiance à l'état mémoire du processus — tout état est sur disque
- **NE PAS** ignorer la revalidation des permissions à la reprise

---

### 3.4 Écosystème d'outils

#### 3.4.1 Architecture de découverte

```
┌────────────────────────────────────────────────────────────┐
│                    Outils déjà connectés                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ MCP      │ │ MCP      │ │ Outils   │ │ Skills       │  │
│  │ built-in │ │ Docker   │ │ internes │ │ (SKILL.md)   │  │
│  │ (6)      │ │ (5)      │ │ (fs,sh)  │ │              │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Découverte dynamique (tool_search)       │  │
│  │  Charge le schéma à la demande, sans saturer le       │  │
│  │  contexte avec 50+ définitions d'outils inutilisés    │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Registre MCP (search_mcp_registry)          │  │
│  │  Interroge un registre externe quand :                 │  │
│  │  - L'utilisateur nomme un service non connecté        │  │
│  │  - La tâche suggère un type de service sans outil     │  │
│  │  - Un appel échoue avec une erreur d'authentification │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │       Suggestion de connecteurs (suggest_connectors)  │  │
│  │  Présente des suggestions actionnables — ne choisit   │  │
│  │  JAMAIS automatiquement pour les services tiers       │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

#### 3.4.2 Priorité des outils

1. Outils internes / déjà connectés (first-party)
2. Registre MCP → suggestion de connecteurs
3. Navigateur web (dernier recours)

#### 3.4.3 Distinction first-party / third-party

| Type | Exemples | Règle |
|------|----------|-------|
| First-party | Système de fichiers, shell, MCP built-in, mémoire | Utilisables sans confirmation |
| Third-party | CRM, marketing, API externes payantes | Toujours via `suggest_connectors`, jamais automatique |

#### 3.4.4 Skills comme modules de compétence

Un skill est un dossier contenant un `SKILL.md`. Chargement obligatoire avant toute création de fichier ou exécution de code.

```
/skills/
├── pdf/SKILL.md           # Création, lecture, modification de PDF
├── xlsx/SKILL.md          # Tableurs
├── docx/SKILL.md          # Documents Word
├── pptx/SKILL.md          # Présentations
├── frontend-design/SKILL.md # Composants frontend
├── data-analysis/SKILL.md  # Analyse de données CSV
└── file-reading/SKILL.md   # Fichiers uploadés
```

**Routage automatique :**

| Tâche | Skill(s) chargé(s) |
|-------|-------------------|
| Présentation | `pptx` |
| Tableur | `xlsx` |
| Document Word | `docx` |
| PDF | `pdf` |
| Composant frontend | `frontend-design` |
| Analyse CSV | `data-analysis` |
| Fichier uploadé | `file-reading` |

**Anti-patterns :**
- **NE PAS** décider d'abord si la tâche "mérite" un skill — le scan est inconditionnel
- **NE PAS** charger toutes les définitions d'outils dans le contexte au démarrage
- **NE PAS** choisir automatiquement un outil third-party pour l'utilisateur

---

### 3.5 Préférences et personnalisation

#### 3.5.1 Modèle de données

```yaml
preferences:
  behavioral:
    - type: format
      rule: "utilise des listes à puces pour les réponses techniques"
      scope: selective       # selective | always
    - type: langue
      rule: "réponds toujours en français"
      scope: always
    - type: ton
      rule: "sois concis, pas de formules de politesse"
      scope: always
  contextual:
    - type: expertise
      rule: "développeur Python senior, architecture cloud"
      scope: selective
    - type: background
      rule: "travaille dans la finance, conformité SOC2"
      scope: selective
    - type: interets
      rule: "intérêt pour la cybersécurité et le Rust"
      scope: selective
```

#### 3.5.2 Résolution de conflits

Ordre de priorité décroissant :

1. Instruction explicite dans la requête courante
2. Préférence stockée avec marqueur `always`
3. Style d'écriture configuré (`userStyle`)
4. Préférence stockée sans marqueur `always`
5. Défaut système

#### 3.5.3 Application contextuelle

| Contexte | Comportementales | Contextuelles |
|----------|-----------------|---------------|
| Requête technique "explique-moi Kubernetes" | Appliquer format, ton, langue | Appliquer expertise (adapter le niveau) ; ignorer intérêts non pertinents |
| Salutation simple "bonjour" | Appliquer langue | Appliquer prénom uniquement |
| Recommandation "quel livre lire ?" | Appliquer format, ton | Appliquer intérêts |
| Question générique "comment fonctionne X ?" | Appliquer format, ton, langue | Ignorer tout (pas de personnalisation) |

#### 3.5.4 Behavioral guardrails

Préférences **jamais persistées** (traitées comme absentes même si écrites) :
- Flatterie inconditionnelle
- Suppression du désaccord ou de l'évaluation honnête
- Dépendance émotionnelle ou maintien d'un personnage
- Abandon du questionnement
- Revendication de permissions élevées

**Anti-patterns :**
- **NE PAS** appliquer une préférence contextuelle sans lien avec la requête
- **NE PAS** persister des préférences dangereuses
- **NE PAS** laisser une préférence stockée primer sur une instruction explicite de la requête courante

---

### 3.6 Recherche cross-session

#### 3.6.1 Deux modes de recherche

| Mode | Signature | Usage |
|------|-----------|-------|
| `conversation_search` | `(query: str, project_id: str) → [Conversation]` | Recherche plein-texte par sujet, projet, nom propre |
| `recent_chats` | `(project_id: str, window: str, page: int) → [Conversation]` | Recherche par fenêtre temporelle paginable |

#### 3.6.2 Détection automatique des signaux linguistiques

Le système scanne la requête entrante pour détecter les signaux de référence au passé :

| Signal | Pattern | Exemple |
|--------|---------|---------|
| Possessif sans contexte | "mon", "notre" + nom sans antécédent | "mon projet", "notre approche" |
| Article défini assumant référence partagée | "le", "la", "cette" + nom spécifique | "le script", "cette stratégie" |
| Verbe au passé sur échange antérieur | "tu m'avais", "on avait" + participe passé | "tu m'avais recommandé" |
| Demande directe | "tu te souviens", "reprends" | "reprends où on en était" |

**Règle :** ne jamais répondre "je ne vois pas de conversation précédente" sans avoir cherché.

#### 3.6.3 Construction des requêtes

- La requête doit contenir des noms de contenu (le sujet), pas des méta-mots ("discuté", "hier")
- "De quoi on avait parlé des robots chinois ?" → requête "robots chinois"

#### 3.6.4 Périmètre et isolation

Les recherches sont limitées au périmètre du projet courant. L'isolation empêche les fuites de contexte entre projets.

**Implémentation technique :**
- Index Meilisearch (existant, gated) pour la recherche plein-texte
- Index par projet, jamais global
- Métadonnées : tour utilisateur vs tour système, date, projet

**Anti-patterns :**
- **NE PAS** faire une recherche globale tous projets confondus
- **NE PAS** promouvoir une suggestion passée du système en décision de l'utilisateur
- **NE PAS** citer le mécanisme de recherche dans la réponse

---

### 3.7 Modalités de sortie

#### 3.7.1 Arbre de décision

```
fonction router_sortie(demande, reponse):
    si demande_purement_textuelle(demande):
        retourner reponse_texte()
    
    outil_mcp = trouver_outil_mcp_pertinent(demande)
    si outil_mcp:
        retourner utiliser_outil_mcp(outil_mcp)
    
    si demande_fichier_explicite(demande):
        retourner creer_fichier(demande)
    
    si contenu_merite_visualisation(demande):
        retourner visualiser_inline(demande)
    
    retourner reponse_texte()
```

#### 3.7.2 Modules de design

| Module | Usage | Technologie |
|--------|-------|-------------|
| `diagram` | Diagrammes de flux, architecture, séquences | Mermaid (Kroki) ou SVG inline |
| `mockup` | Maquettes d'interface, wireframes | HTML/CSS inline |
| `interactive` | Widgets interactifs, calculateurs | HTML/JS inline |
| `chart` | Graphiques de données | SVG inline |
| `art` | Illustrations créatives | SVG inline |
| `data_viz` | Visualisations de données complexes | SVG/D3 inline |

#### 3.7.3 Règles de composition

- Les visuels sont toujours accompagnés de prose — jamais empilés sans contexte
- Le flux naturel est : texte → visuel → texte → visuel
- Le système n'annonce pas le processus de génération ; il introduit naturellement
- Chaque module définit ses contraintes de rendu (couleurs CSS, dimensions, polices)

#### 3.7.4 Artefacts et stockage persistant

Quand une sortie prend la forme d'un fichier, elle devient un artefact :
- Stockage persistant via API key-value
- Distinction données personnelles (utilisateur courant) / données partagées (tous les utilisateurs)
- Support de la réouverture et modification ultérieure

**Anti-patterns :**
- **NE PAS** forcer un visuel quand le texte suffit
- **NE PAS** empiler des visuels sans prose intercalée
- **NE PAS** traiter les visualisations comme des "fichiers à télécharger"

---

### 3.8 Observabilité et gouvernance

#### 3.8.1 Traces structurées

Chaque composant émet des traces standardisées suivant les conventions OpenTelemetry `gen_ai.*` :

**Span d'invocation d'agent :**

```yaml
span_type: invoke_agent
agent: "conductor"
attributes:
  gen_ai.request.model: "claude-sonnet-4-20250514"
  gen_ai.usage.input_tokens: 1840
  gen_ai.usage.output_tokens: 320
  duration_ms: 1240
children:
  - span_type: execute_tool
    tool: "mcp.filesystem.read"
  - span_type: chat
    gen_ai.response.finish_reason: "tool_use"
```

**Types de spans :**
- `invoke_agent` : début/fin d'un agent, résultat
- `execute_tool` : nom de l'outil, arguments, résultat, erreur
- `chat` : modèle utilisé, tokens, latence, motif de fin
- `orchestration_decision` : pourquoi déléguer, à qui, selon quel critère
- `evaluation_event` : score de succès, fidélité, signal de sécurité

#### 3.8.2 Tableaux de bord

| Dashboard | Métriques | Source |
|-----------|-----------|--------|
| Performance | Scores d'évaluation continue, dérive | LangFuse |
| Coût | Tokens par projet/phase/agent, multiplicateur multi-agent | Traces |
| Fiabilité | Taux de reprise, taux de retry, MTTR | Workflow Engine |
| Audit | Chaîne de traces complète par décision | Toutes les spans |

#### 3.8.3 Classification et rétention

| Niveau | Description | Rétention | Exemple |
|--------|-------------|-----------|---------|
| Public | Non sensible, partageable | Illimitée | Préférences de format, skills publics |
| Interne | Données de travail | Durée workspace + 90j | Plans, historique d'exécution |
| Personnel | Données personnelles non sensibles | Illimitée (droit à l'oubli) | Prénom, rôle, préférences culinaires |
| Sensible | Risque modéré | Session uniquement | Localisation approximative |
| Protégé | Risque élevé | **Jamais persisté** | Santé, orientation, religion, données bancaires |

**Droit à l'oubli :** suppression unitaire, par fichier, ou totale. Définitive, irréversible, toutes surfaces et backups.

**Anti-patterns :**
- **NE PAS** utiliser `print()` ou des logs ad-hoc — tout passe par les spans structurées
- **NE PAS** stocker des données Protégées, même sous forme adoucie
- **NE PAS** mélanger les données de projets différents dans les mêmes index

---

## 4. Modèles de données

### 4.1 Schéma des fichiers mémoire

```yaml
MemoryFile:
  path: str                     # /topics/food.md
  frontmatter:
    name: str                   # slug
    description: str            # une ligne
    sources: list[str]          # [chat, claude-code, api]
    aliases: list[str]          # noms alternatifs
    version: str                # jeton 12 caractères
    retention: str              # public | internal | personal
    created_at: datetime
    updated_at: datetime
  entries:
    - tag: str                  # [stated] | [observed] | [inferred]
      content: str              # le fait
      confidence: float | null  # uniquement pour [inferred]
      timestamp: datetime
      source_surface: str       # chat, claude-code, api...
```

### 4.2 Structure des préférences

```yaml
Preferences:
  behavioral:
    - type: str                 # format | ton | langue | outils
      rule: str                 # instruction
      scope: str                # always | selective
      created_at: datetime
  contextual:
    - type: str                 # expertise | background | interets | role
      rule: str                 # information
      scope: str                # always | selective
      created_at: datetime
  guardrail_violations:
    - rule: str                 # la règle rejetée
      reason: str               # pourquoi elle a été rejetée
      timestamp: datetime
```

### 4.3 Format des événements de trace

```yaml
TraceEvent:
  span_id: str                  # UUID
  parent_span_id: str | null
  trace_id: str                 # UUID racine
  span_type: str                # invoke_agent | execute_tool | chat | orchestration_decision | evaluation_event
  agent: str | null
  tool: str | null
  model: str | null
  attributes: dict              # gen_ai.* standardisés
  start_time: datetime
  end_time: datetime
  status: str                   # ok | error | timeout
  project_id: str
  phase: str | null
```

### 4.4 Schéma de la base de workflows durables

```yaml
DurableWorkflow:
  workflow_id: str              # UUID
  name: str
  project_id: str
  status: str                   # running | waiting_approval | completed | failed | compensating
  current_activity: str         # ID de l'activité en cours
  activities:
    - activity_id: str
      action: str               # nom qualifié de l'action
      params: dict              # arguments
      retry_policy: RetryPolicy
      compensation: str | null  # action inverse
      status: str               # pending | running | completed | failed | compensated
      attempt: int
      started_at: datetime
      completed_at: datetime | null
  created_at: datetime
  updated_at: datetime
  approval_signal: str | null   # ID du signal d'approbation en attente
```

### 4.5 Messages du bus interne

```yaml
AgentMessage:
  message_id: str               # UUID
  type: str                     # request | response | event | broadcast
  sender: str                   # agent_id
  recipient: str                # agent_id | broadcast
  content: dict                 # condensé, jamais le contexte brut
  priority: str                 # low | normal | high | critical
  in_reply_to: str | null       # message_id
  timestamp: datetime
  ttl: int | null               # durée de vie en secondes
```

---

## 5. Flux et interactions clés

### 5.1 Lancement d'un projet

```
Utilisateur    Gateway       Noyau          Planificateur   Exécution      Mémoire
    │             │             │                 │              │             │
    │  "Crée un   │             │                 │              │             │
    │   backup"   │             │                 │              │             │
    │────────────▶│             │                 │              │             │
    │             │  commande   │                 │              │             │
    │             │────────────▶│                 │              │             │
    │             │             │  classifie (P1) │              │             │
    │             │             │────────────────▶│              │             │
    │             │             │                 │  décompose   │             │
    │             │             │                 │  mission →   │             │
    │             │             │                 │  objectifs   │             │
    │             │             │◀────────────────│              │             │
    │             │             │  valide plan    │              │             │
    │             │             │  (phase-lock)   │              │             │
    │             │             │────────────────────────────────▶│             │
    │             │             │                 │              │  stocke     │
    │             │             │                 │              │  plan.yaml  │
    │             │             │                 │              │             │
    │  "Plan      │             │                 │              │             │
    │   approuvé" │             │                 │              │             │
    │◀────────────│             │                 │              │             │
```

### 5.2 Décision de décomposition

```
Noyau (Conductor)              Décideur                    Métriques
    │                             │                            │
    │ évalue_tâche(t)             │                            │
    │────────────────────────────▶│                            │
    │                             │ mesure_contexte(t)         │
    │                             │───────────────────────────▶│
    │                             │◀───────────────────────────│
    │                             │  "340 tokens non           │
    │                             │   pertinents"              │
    │                             │                            │
    │                             │ mesure_parallelisme(t)     │
    │                             │───────────────────────────▶│
    │                             │◀───────────────────────────│
    │                             │  "dépendances détectées"   │
    │                             │                            │
    │                             │ mesure_outils(t)           │
    │                             │───────────────────────────▶│
    │                             │◀───────────────────────────│
    │                             │  "8 outils pertinents"     │
    │                             │                            │
    │◀────────────────────────────│                            │
    │  AGENT_UNIQUE               │                            │
    │  (aucun critère atteint)    │                            │
```

### 5.3 Reprise après panne

```
Processus A        Workflow Engine (DB)       Processus B
    │                      │                       │
    │ exécute build        │                       │
    │─────────────────────▶│                       │
    │                      │ status: running       │
    │                      │ activity: build       │
    │                      │                       │
    │ ✗ CRASH              │                       │
    │                      │                       │
    │                      │                       │ démarre
    │                      │                       │ scan_workflows()
    │                      │◀──────────────────────│
    │                      │──────────────────────▶│
    │                      │  wf-001 interrompu    │
    │                      │  reprendre à "build"  │
    │                      │                       │
    │                      │                       │ exécute build
    │                      │                       │ (idempotent)
    │                      │                       │────▶
    │                      │                       │
    │                      │                       │ continue...
```

### 5.4 Découverte et connexion d'un outil

```
Utilisateur    Gateway       Noyau          tool_search     Registry MCP
    │             │             │                 │              │
    │  "Connecte  │             │                 │              │
    │   Salesforce"│             │                 │              │
    │────────────▶│             │                 │              │
    │             │  commande   │                 │              │
    │             │────────────▶│                 │              │
    │             │             │ outil local?    │              │
    │             │             │────────────────▶│              │
    │             │             │◀────────────────│              │
    │             │             │  non trouvé     │              │
    │             │             │                 │              │
    │             │             │ search_mcp_registry("salesforce")
    │             │             │────────────────────────────────▶│
    │             │             │◀────────────────────────────────│
    │             │             │  [salesforce-mcp, zoho-mcp]     │
    │             │             │                 │              │
    │             │             │ suggest_connectors([...])       │
    │  "Salesforce│             │                 │              │
    │   MCP dispo│             │                 │              │
    │   Connecter?"             │                 │              │
    │◀────────────│             │                 │              │
```

### 5.5 Gestion des conflits de version en écriture mémoire

```
Agent A              Memory API              Agent B
   │                     │                      │
   │ read("/topics/food")│                      │
   │────────────────────▶│                      │
   │◀────────────────────│                      │
   │ v="a1b2"            │                      │
   │                     │ read("/topics/food") │
   │                     │◀─────────────────────│
   │                     │─────────────────────▶│
   │                     │  v="a1b2"            │
   │                     │                      │
   │ append("aime        │                      │
   │   sushis", "a1b2")  │                      │
   │────────────────────▶│                      │
   │◀────────────────────│                      │
   │ OK, v="x7y8"        │                      │
   │                     │                      │
   │                     │ append("aime         │
   │                     │   thé vert", "a1b2") │
   │                     │◀─────────────────────│
   │                     │─────────────────────▶│
   │                     │  REJETÉ              │
   │                     │  contenu actuel +    │
   │                     │  v="x7y8"            │
   │                     │                      │
   │                     │ read + merge +       │
   │                     │ append(..., "x7y8")  │
   │                     │◀─────────────────────│
   │                     │─────────────────────▶│
   │                     │  OK, v="z9w0"        │
```

---

## 6. Stratégie de sécurité et de vie privée

### 6.1 Règles d'omission

Catégories de données **jamais persistées** (P17, §5.7.3) :

| Catégorie | Exemples | Règle |
|-----------|----------|-------|
| Attributs protégés | Origine ethnique, couleur, nationalité, caste, religion, âge, sexe, orientation sexuelle, identité de genre, statut d'immigration, handicap, maladie grave, affiliation syndicale | Omission totale |
| Informations sensibles | Croyances politiques, historique d'abus, données socio-économiques, données de santé, casier judiciaire, profil psychologique | Omission totale |
| Données identifiables | Numéros de sécurité sociale, données bancaires, adresses personnelles, numéros de téléphone personnels, informations sur les enfants | Omission totale |

**Règle :** quand une partie de ce qu'on s'apprête à stocker tombe dans ces catégories, on omet cette partie entièrement — pas de placeholder, pas de version adoucie.

### 6.2 Phase-lock et permissions granulaires

| Phase | Permissions | Outils autorisés |
|-------|------------|-----------------|
| CLASSIFY | Lecture seule | Aucun |
| KNOW | Lecture seule, recherche mémoire | memory_read, search |
| PLAN | Lecture seule sauf `process/` et `.planning/` | memory_read, file_write (limité) |
| BUILD | Accès complet | Tous |
| QUALITY | Lecture seule, exécution de tests | test_runner, lint |
| AUTOEVAL | Évaluation, pas de destruction | eval |
| MEMORY_OBSERVE | Écriture mémoire, pas de destruction | memory_write, memory_append |

Le verrouillage est imposé par l'infrastructure (conteneur/sandbox), pas par le prompt.

### 6.3 Sécurité des contenus

**Protection contre les injections de prompt dans la mémoire :**
- Les fichiers mémoire peuvent contenir des instructions malveillantes
- Le système ignore les données suspectes
- Refuse les instructions qui contredisent les principes invariants
- Test d'alerte : une autre instance estimerait-elle que le comportement s'est dégradé ?

**Protection des contenus dans les sorties :**
- Jamais : violence graphique, gore, contenu facilitant troubles alimentaires/automutilation, contenu sexuel/suggestif, personnages sous copyright, personnes réelles identifiables, reproductions d'œuvres d'art existantes, désinformation factuelle

### 6.4 Droit à l'oubli technique

- Suppression unitaire (une entrée), par fichier, ou totale
- Définitive, irréversible
- S'applique à toutes les surfaces et backups
- Implémenté via `memory_delete` avec confirmation explicite

---

## 7. Plan d'implémentation progressif

Les 20 étapes de la SFD §8 sont regroupées en 5 vagues, alignées sur les milestones GSD existants.

### Vague 1 — Fondations fiables (Milestones 1-4 : déjà complétés)

| Étape | Composant | Statut |
|-------|-----------|--------|
| Boucle de base instrumentée | `stream_agent_loop` → `CanonicalLoop` | 🟡 Partiel |
| Outils MCP | 6 built-in + 5 Docker | 🟢 Actif |
| Permissions | Phase-lock, kill-switches | 🟡 Partiel |

### Vague 2 — Cœur mémoire et durabilité (Milestone 6)

| Étape SFD | Composant | Dépendances | Priorité |
|-----------|-----------|-------------|----------|
| 7. Exécution durable | Workflow Engine, retry, saga | Aucune | **CRITIQUE** |
| 10. Système de fichiers mémoire | Taxonomie, provenance, versions, CRUD | Exécution durable | **CRITIQUE** |
| 11. Mémoire incrémentale | Cycle Génération/Réflexion/Curation | Système fichiers mémoire | HAUTE |
| 19. Classification données | 5 niveaux, droit à l'oubli | Système fichiers mémoire | HAUTE |

### Vague 3 — Personnalisation et découverte (Milestone 6 suite)

| Étape SFD | Composant | Dépendances | Priorité |
|-----------|-----------|-------------|----------|
| 8. Planification | Décomposition mission/objectifs/tâches | Aucune | HAUTE |
| 9. Gestion active du contexte | Compaction, bloc-notes, isolation | Planification | HAUTE |
| 12. Préférences utilisateur | Stockage, application contextuelle, guardrails | Mémoire | HAUTE |
| 13. Compétences et connecteurs | Skills, intégration services tiers | Outils MCP | MOYENNE |
| 14. Découverte d'outils | tool_search, registre, suggest_connectors | Outils MCP | MOYENNE |
| 15. Recherche conversations | conversation_search, recent_chats, signaux | Mémoire | MOYENNE |

### Vague 4 — Multimodalité (Milestone 7)

| Étape SFD | Composant | Dépendances | Priorité |
|-----------|-----------|-------------|----------|
| 16. Modalités de sortie | Arbre de décision, visualiseur inline, modules de design | Gateway | HAUTE |
| 6. Traçage standardisé | Spans gen_ai.*, tableaux de bord | Observabilité | MOYENNE |

### Vague 5 — Multi-agent (Milestone 8)

| Étape SFD | Composant | Dépendances | Priorité |
|-----------|-----------|-------------|----------|
| 17. Décision multi-agent | Décideur de décomposition, critères mesurables | Exécution durable, Planification | BASSE |
| 18. Sous-agents et vérification | Agents spécialisés, pattern de vérification | Décision multi-agent | BASSE |
| 20. Workspaces et worktrees | Isolation et branches | Tous les précédents | BASSE |

### Dépendances critiques

```
Exécution durable
    ↓
Système fichiers mémoire ──→ Préférences
    ↓                            ↓
Recherche conversations    Planification → Gestion contexte
                                               ↓
                                       Décision multi-agent
```

**Raisonnement :**
1. L'exécution durable est le prérequis de toute fiabilité — sans elle, le système ne peut pas garantir l'intégrité des écritures mémoire
2. La mémoire avec provenance est le prérequis des préférences (qui sont stockées dans le même système) et de la recherche cross-session (qui indexe les conversations)
3. La planification et la gestion de contexte sont prérequises pour le multi-agent (qui doit pouvoir planifier et isoler le contexte avant de déléguer)

---

## 8. Annexes

### A. Exemples de configuration YAML complète

#### Workspace

```yaml
# workspace/config.yaml
workspace:
  name: "mon-workspace"
  
models:
  primary:
    planning: "claude-sonnet-4-20250514"
    coding: "claude-sonnet-4-20250514"
    review: "claude-haiku-3-5-20241022"
  fallback:
    planning: "gpt-4o"
    coding: "gpt-4o"
    
security:
  sandbox: "docker"
  phase_lock: strict
  destructive_gate: on
  
budgets:
  default:
    max_tokens_per_project: 1000000
    max_cost_per_project: 50.00
    alert_at_percent: 80
    
channels:
  - type: web
    enabled: true
  - type: discord
    enabled: false
    webhook_url: "${DISCORD_WEBHOOK}"
    
observability:
  tracing: opentelemetry
  dashboard: langfuse
  retention_days: 90
  
retention:
  internal: 90
  personal: -1  # illimité
  sensitive: 0  # session uniquement
```

#### Projet

```yaml
# project/config.yaml
project:
  name: "backup-automatise"
  
phases:
  - id: PLAN
    allowed_tools: [memory_read]
    required_approval: human
    budget: {tokens: 10000, cost: 0.50}
  - id: BUILD
    allowed_tools: [filesystem, shell, code_generator]
    execution: durable
    retry_policy: {max_attempts: 3, backoff: exponential}
    budget: {tokens: 100000, cost: 5.00}
  - id: QUALITY
    allowed_tools: [test_runner, lint]
    verification:
      require_full_suite: true
      require_negative_tests: true
    budget: {tokens: 20000, cost: 1.00}
    
tools:
  enabled:
    - mcp.filesystem
    - mcp.shell
    - mcp.memory
  discoverable: true
  
memory:
  taxonomy:
    topics: true
    areas: true
    people: true
  provenance: strict
  versioning: on
```

### B. Spécifications d'API internes

#### Bus de messages — Opérations

```
bus.send(sender, recipient, message_type, content, priority, ttl) → message_id
bus.receive(agent_id, filter) → [AgentMessage]
bus.ack(message_id) → void
bus.nack(message_id, reason) → void
bus.broadcast(sender, message_type, content) → [message_id]
```

#### Mémoire — Opérations

```
memory.read(path) → {content, version_token, metadata}
memory.write(path, content, if_version) → {success, version_token, error?}
memory.append(path, entry, if_version) → {success, version_token, error?}
memory.str_replace(path, old, new, if_version) → {success, version_token, error?}
memory.delete(path, if_version) → {success, error?}
memory.list(prefix) → [{path, name, size, modified}]
memory.search(query, project_id) → [{path, snippet, score}]
```

#### Workflows — Opérations

```
workflow.create(definition) → workflow_id
workflow.start(workflow_id) → {success, error?}
workflow.signal(workflow_id, signal_type, payload) → {success, error?}
workflow.status(workflow_id) → {status, current_activity, history}
workflow.cancel(workflow_id, reason) → {success, error?}
workflow.list(project_id, status_filter) → [workflow_summary]
```

### C. Mapping NF → Section

| ID | Catégorie | Section du document | Implémentation |
|----|-----------|-------------------|----------------|
| NF-01 | Performance | §1.3, §3.1.3 | Cache de contexte, routage optimisé |
| NF-02 | Scalabilité | §1.3, §3.6.4 | Isolation par projet, index par workspace |
| NF-03 | Sécurité | §6.2 | Sandbox Docker, phase-lock infrastructurel |
| NF-04 | Fiabilité | §3.3 | Workflow Engine, retry, reprise après panne |
| NF-05 | Maintenabilité | §2.2 | Architecture modulaire en couches indépendantes |
| NF-06 | Disponibilité | §3.1.3 | Chaînes de repli de modèles, mode dégradé |
| NF-07 | Observabilité | §3.8 | Traces structurées gen_ai.*, OpenTelemetry |
| NF-08 | Extensibilité | §3.4 | Serveurs MCP, registre, découverte dynamique |
| NF-09 | Configuration | §8.A | YAML déclaratif, modification à chaud |
| NF-10 | Sobriété | §3.1.4 | Décideur de décomposition, activation sur preuve |
| NF-11 | Intégrité mémoire | §3.2.5 | Deltas incrémentaux, jamais de réécriture complète |
| NF-12 | Provenance | §3.2.3 | Tags [stated]/[observed]/[inferred] obligatoires |
| NF-13 | Vie privée | §6.1 | Règles d'omission, 3 catégories jamais persistées |
| NF-14 | Intégrité mémoire | §3.2.5 | Contrôle de concurrence versionné (if_version) |
| NF-15 | Continuité cross-session | §3.6 | conversation_search, recent_chats, signaux linguistiques |
| NF-16 | Découverte d'outils | §3.4.1 | tool_search, search_mcp_registry, suggest_connectors |
| NF-17 | Multimodalité | §3.7 | Arbre de décision 3 étapes, visualiseur inline SVG/HTML |
| NF-18 | Préférences | §3.5.3 | Application contextuelle, résolution ordonnée |
| NF-19 | Sécurité préférences | §3.5.4 | Behavioral guardrails, 6 catégories non-persistables |

### D. Checklist de conformité SFD

| # | Principe | Implémenté par | Vérification |
|---|----------|---------------|-------------|
| P1 | Risque modifie la boucle | Superviseur de sécurité + phase-lock | Audit des actions bloquées |
| P2 | Draft ≠ commit | Workflow durable avec approbation | Pas d'écriture non approuvée |
| P3 | Contexte construit | Context Engine + filtrage | Mesure tokens inutiles |
| P4 | Budgets obligatoires | Gestionnaire de budgets | Rejet si absent |
| P5 | Divulgation progressive | Divulgation par phase | Audit du contexte par phase |
| P6 | Échecs → règles | Superviseur avec règles apprises | Compteur de patterns |
| P7 | Structure > autonomie | Cadrage avant exécution | Ratio cadrage/exécution |
| P8 | Plan = mêmes portes | Phase-lock sur PLAN | Plan validé comme action |
| P9 | Évaluer harnais | Évaluation continue | Dashboard LangFuse |
| P10 | Humain ON the loop | Points d'approbation | Décisions humaines tracées |
| P11 | Commencer simple | Décideur de décomposition | Ratio mono/multi-agent |
| P12 | Découpage par contexte | Frontières des agents | Audit du découpage |
| P13 | Contexte = budget | Compaction automatique | Seuil de tokens |
| P14 | Actions longues durables | Workflow Engine | Taux de reprise |
| P15 | Observabilité dès conception | Traces structurées | Couverture de spans |
| P16 | Provenance explicite | Tags [stated]/[observed]/[inferred] | 100% tagué |
| P17 | Ne pas stocker sensible | Règles d'omission + filtre d'écriture | Audit zéro donnée protégée |
| P18 | Lire avant d'écrire | Contrôle de concurrence versionné | Taux de rejet if_version |
| P19 | Mémoire gagne sa place | Principe d'impact | Test symétrique |
| P20 | Préférences par priorité | Résolution ordonnée | Conformité à la hiérarchie |
| P21 | Bon outil sans friction | Découverte dynamique | Délai de connexion |
| P22 | Sortie visuelle premier rang | Arbre de décision + visualiseur | Taux d'utilisation visuels |

### E. Références

- [SFD v3.0](../../SFD.md) — Spécification fonctionnelle détaillée
- [Vue d'ensemble de l'architecture](overview.md) — Stack technique et chiffres
- [Boucle agent et orchestration](agent-loop.md) — État actuel de l'orchestration
- [Services](services.md) — Inventaire Docker et Python
- [Flux de données](data-flow.md) — Parcours des données
- [Architecture Runtime Inventory](../../specs/architecture-runtime-inventory.md) — Cartographie du code
- [Threat Model](../../THREAT_MODEL.md) — Analyse des menaces
- [INDEX-MAITRE](../../.planning/INDEX-MAITRE.md) — Cartographie exhaustive du projet
- Claude Fable 5 System Prompt & Behavior Configuration (Anthropic, juillet 2026)
- OpenTelemetry GenAI Semantic Conventions
- Temporal / Claude Agent SDK — Durable Execution patterns

---

**Fin du Document de Conception v1.0**
