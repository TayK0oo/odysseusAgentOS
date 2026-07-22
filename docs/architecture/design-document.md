# AGENT OS — Document de Conception

**Version :** 1.0 | **Date :** 2026-07-22 | **Base :** [SFD v3.0](../../SFD.md)
**Issu de :** [Phase 5.1.5 — Discuss](../../.planning/phases/05-activation/5.1.5-design-conception/5.1.5-CONTEXT.md)

---

## Table des matières

1. [Objectifs et vision](#1-objectifs-et-vision)
2. [Cheminement de pensée](#2-cheminement-de-pensée)
3. [Architecture des modules](#3-architecture-des-modules)
4. [Orchestration et autonomie](#4-orchestration-et-autonomie)
5. [Plan d'implémentation](#5-plan-dimplémentation)
6. [Modèles de données](#6-modèles-de-données)
7. [Gouvernance et kill-switches](#7-gouvernance-et-kill-switches)
8. [Annexes](#8-annexes)

---

## 1. Objectifs et vision

### 1.1 Ce qu'est Agent OS

Agent OS est un assistant autonome de conduite de projet. Il reçoit un objectif en langage naturel, élabore un plan, l'exécute de façon fiable, et apprend de chaque expérience. Ce n'est pas un wrapper LLM — c'est une plateforme modulaire où le raisonnement (IA) et la fiabilité d'exécution (durabilité) sont deux couches distinctes et indépendantes.

### 1.2 Objectifs mesurables

| Objectif | Métrique | Cible | Source SFD |
|----------|----------|-------|------------|
| Autonomie contrôlée | Taux d'actions exécutées sans intervention humaine | > 80% (configurable) | P1, P10 |
| Survie aux pannes | Reprise sans perte d'état après crash | 100% des workflows durables | P14, NF-04 |
| Intégrité mémoire | Faits sans provenance | 0% | P16, NF-12 |
| Sobriété contextuelle | Tokens non pertinents dans le contexte | < 10% du total | P3, P13 |
| Latence perçue | Temps de réponse interaction simple | < 2 secondes | NF-01 |
| Observabilité | Composants sans traces structurées | 0 | P15, NF-07 |
| Vie privée | Données protégées persistées | 0 | P17, NF-13 |
| Évolutivité | Ajout d'un module sans modifier la boucle | < 1 fichier touché | NF-05 |

### 1.3 Positionnement

Agent OS n'est pas un chatbot amélioré. Il se distingue par cinq piliers :

| Pilier | Ce que ça change |
|--------|-----------------|
| **Mémoire à provenance** | Chaque fait est tagué `[stated]`/`[observed]`/`[inferred]`. Pas de contamination par des inférences non vérifiées. |
| **Exécution durable** | Une action interrompue reprend exactement là où elle s'est arrêtée. Indépendant du modèle d'IA. |
| **Bus de pensée** | Le cheminement du système est explicite, traçable, et extensible par événements. |
| **Autonomie configurable** | L'humain est sur la boucle, pas dans la boucle. Le niveau d'autonomie s'ajuste par projet. |
| **Sortie multimodale** | Les visuels sont des réponses à part entière, pas des pièces jointes. |

---

## 2. Cheminement de pensée

### 2.1 Le bus de pensée (Thought Bus)

Le système nerveux d'Agent OS. Une boucle en 7 phases, chaque transition émet un événement auquel les modules s'abonnent.

```
┌─────────────────────────────────────────────────────────────────┐
│                       BUS DE PENSÉE                              │
│                                                                  │
│  [Préférences] ──→ CLASSIFY ──→ KNOW ──→ PLAN ──→ BUILD         │
│       │               │           │         │        │           │
│       ▼               ▼           ▼         ▼        ▼           │
│  ┌─────────┐    ┌──────────┐ ┌────────┐ ┌────────┐ ┌────────┐  │
│  │Pref.    │    │Recherche │ │Mémoire │ │Exéc.   │ │Outils  │  │
│  │Engine   │    │convers.  │ │Context │ │durable │ │MCP     │  │
│  └─────────┘    └──────────┘ └────────┘ └────────┘ └────────┘  │
│                                                                  │
│  BUILD ──→ QUALITY ──→ AUTOEVAL ──→ MEMORY_OBSERVE ──→ [Sortie] │
│    │          │           │              │                │      │
│    ▼          ▼           ▼              ▼                ▼      │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐ ┌─────────┐ │
│  │Skills  │ │Vérif.  │ │Eval    │ │Mémoire       │ │Visuel   │ │
│  │        │ │agent   │ │continue│ │+ Rétention   │ │inline   │ │
│  └────────┘ └────────┘ └────────┘ └──────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Les 7 phases

| Phase | Rôle | Permissions | Modules abonnés |
|-------|------|-------------|-----------------|
| **CLASSIFY** | Évaluer le risque, classifier la demande | Lecture seule | PreferencesEngine |
| **KNOW** | Récupérer le contexte, la mémoire, l'historique | Lecture seule | ConversationSearch, ContextEngine |
| **PLAN** | Décomposer en objectifs et tâches | Écriture `.planning/` uniquement | ContextEngine (notepad) |
| **BUILD** | Exécuter, coder, créer | Accès complet | DurableExecution, Tools, Skills |
| **QUALITY** | Tester, vérifier, valider | Lecture + exécution tests | VerificationAgent |
| **AUTOEVAL** | Auto-évaluer la qualité | Évaluation uniquement | ContinuousEval |
| **MEMORY_OBSERVE** | Apprendre, mémoriser, nettoyer | Écriture mémoire | MemoryEngine, RetentionEngine |

### 2.3 Mécanique du bus

```python
# Un module s'abonne à une phase
@on_phase("BUILD")
class DurableExecutionSubscriber:
    def on_enter(self, context):
        # Enveloppe les actions dans des workflows durables
        ...
    
    def on_exit(self, context):
        # Vérifie que tous les workflows sont terminés
        ...

# Le bus émet à chaque transition
thought_bus.emit(PhaseEvent.ENTER, phase="BUILD", context={...})
# Les subscribers sont appelés dans l'ordre de priorité
# Le résultat de chaque subscriber enrichit le contexte
```

### 2.4 Cheminement complet d'une requête

```
1. L'utilisateur envoie "Crée un système de backup automatisé"
2. PreferencesEngine injecte les préférences (format, ton, langue)
3. CLASSIFY: le Superviseur évalue le risque → niveau MODERATE
4. KNOW: ContextEngine interroge la mémoire → projets similaires, compétences
5. KNOW: ConversationSearch vérifie si le sujet a déjà été abordé
6. PLAN: Planificateur décompose en objectifs → stocké dans notepad
7. BUILD: DurableExecution enveloppe chaque action → workflows SQLite
8. BUILD: Tools MCP exécutent, Skills chargés automatiquement
9. QUALITY: VerificationAgent exécute les tests
10. AUTOEVAL: ContinuousEval compare aux critères de succès
11. MEMORY_OBSERVE: MemoryEngine extrait les leçons, écrit en mémoire
12. Sortie: VisualRouter décide texte seul ou texte + visuel
```

---

## 3. Architecture des modules

### 3.1 Context Engine

Module dédié: `src/context_engine/`. Six sous-modules.

| Sous-module | Rôle | Implémentation |
|-------------|------|---------------|
| **Aggregator** | Collecte les sources (mémoire, historique, plan) | ChromaDB + Graphify + fichiers MD |
| **Filter** | Filtre par similarité sémantique × phase × quota | Embeddings ChromaDB, quota configurable (défaut 10) |
| **Compactor** | Résume l'historique ancien quand le seuil est atteint | Seuil = 80% fenêtre de contexte. Résumé par LLM léger (Haiku) |
| **Notepad** | Externalise le plan et les décisions | ChromaDB — chaque décision = document vectoriel requêtable |
| **ThreadTracker** | Trace d'identité par sous-projet | Associe décisions et contexte à un fil |
| **PreferenceInjector** | Injecte les préférences applicables | Lit `/preferences.md`, applique selon la matrice always/selective/never |

**Stratégie de compaction :**

```
1. tokens_utilisés > 80% fenêtre_contexte → déclencher compaction
2. Sélectionner les N plus anciens tours de conversation
3. Appeler LLM léger: "Résume ces échanges en préservant: décisions prises, faits appris, questions en suspens"
4. Remplacer les tours originaux par le résumé condensé
5. Mettre à jour le compteur de tokens
```

**Stratégie de filtrage :**

```
1. Embedding de la requête courante
2. Recherche similarité cosinus dans ChromaDB (mémoire + historique)
3. Filtrer par phase (BUILD n'a pas besoin des préférences de format)
4. Filtrer par rôle (un sous-agent de vérification n'a pas besoin de l'historique)
5. Limiter au quota (défaut 10 entrées)
6. Retourner les entrées triées par pertinence
```

### 3.2 Système de mémoire avec provenance

#### 3.2.1 Architecture de stockage

```
┌──────────────────────────────────────────────────────┐
│                 TRIPLE STOCKAGE                        │
│                                                       │
│  Fichiers MD (source de vérité)                       │
│  ├── Git versionné                                    │
│  ├── Lisible/éditable par un humain                   │
│  └── Portabilité totale                               │
│                                                       │
│  ChromaDB (index sémantique)                          │
│  ├── Recherche par similarité cosinus                  │
│  ├── Reconstruit depuis les MD                        │
│  └── Index secondaire, pas source de vérité           │
│                                                       │
│  Graphify (relations entre concepts)                  │
│  ├── Graphe sémantique                                │
│  ├── Requêtes multi-hop                               │
│  └── Optionnel, reconstruit depuis les MD             │
└──────────────────────────────────────────────────────┘
```

#### 3.2.2 Taxonomie

```
/memory/
├── profile.md              # Identité stable (> 3 mois)
├── preferences.md           # Comportementales + contextuelles
├── topics/                  # Faits par domaine
│   ├── food.md
│   ├── technology.md
│   └── schedule.md
├── areas/                   # Projets et responsabilités en cours
│   ├── auth-redesign.md
│   └── oncall.md
└── people/                  # Contexte relationnel
    └── collaborateur.md
```

**Règle cardinale:** un fait sur X va dans le fichier de X, pas dans le fichier déjà ouvert.

#### 3.2.3 Format de fichier avec provenance

```yaml
---
name: food
description: Préférences alimentaires — quand recommander un restaurant
sources: [chat]
aliases: [nourriture, cuisine, repas]
version: "a1b2c3d4e5f6"  # hash SHA-256 tronqué du contenu
retention: personal
updated: 2026-07-22T14:30:00Z
---

- [stated] préfère la cuisine japonaise
- [stated] allergique aux arachides
- [observed] commande souvent des sushis le vendredi
- [inferred] apprécie probablement la cuisine coréenne (confiance: 0.6)
```

#### 3.2.4 Provenance — règles strictes

| Tag | Signification | Qui écrit | Jamais en mode conversation |
|-----|--------------|-----------|---------------------------|
| `[stated]` | L'utilisateur l'a dit | L'agent en conversation | — |
| `[observed]` | Le système l'a constaté | Sous-systèmes, autres surfaces | **Jamais** |
| `[inferred]` | Déduit avec confiance | Agent d'analyse dédié | **Jamais** |

- On ne convertit jamais un `[stated]` en généralisation
- On ne stocke pas les suggestions, conclusions ou résultats de recherche du système
- Les catégories protégées (origine, religion, santé, etc.) ne sont **jamais** persistées

#### 3.2.5 Opérations CRUD versionnées

| Opération | Signature | `if_version` |
|-----------|-----------|-------------|
| `memory_read` | `(path) → (content, version)` | Retourne le hash actuel |
| `memory_write` | `(path, content, if_version)` | `"new"` pour création, ou hash |
| `memory_append` | `(path, entry, if_version)` | Hash actuel requis |
| `memory_str_replace` | `(path, old, new, if_version)` | Hash actuel requis |
| `memory_delete` | `(path, if_version)` | Hash actuel requis + confirmation explicite |
| `memory_list` | `(prefix) → [paths]` | N/A |

**Contrôle de concurrence :**

```
Agent A: read("/topics/food.md") → (content, "a1b2")
Agent B: read("/topics/food.md") → (content, "a1b2")
Agent A: append("aime sushis", "a1b2") → OK, nouveau hash "x7y8"
Agent B: append("aime thé vert", "a1b2") → REJETÉ
         → retourne contenu actuel + "x7y8"
         → Agent B fusionne et réessaie avec "x7y8"
```

#### 3.2.6 Boucle Génération → Réflexion → Curation

```
GENERATION (exécute) → produit une trajectoire (succès/échec)
    ↓
RÉFLEXION (extrait) → extrait une leçon concrète et réutilisable
    ↓
CURATION (intègre) → ajoute/modifie une fiche dans la base (delta, pas réécriture)
```

### 3.3 Exécution durable

#### 3.3.1 Architecture

Moteur custom Python avec état en SQLite. Zéro dépendance externe.

```
src/durable_execution/
├── engine.py           # Workflow Engine: création, exécution, reprise
├── activity.py         # Activity: unité d'action avec retry policy
├── saga.py             # Saga Coordinator: compensation en ordre inverse
├── signals.py          # Signal Waiter: approbation humaine longue durée
└── models.py           # Modèles SQLAlchemy pour l'état persistant
```

#### 3.3.2 Modèle de workflow

```python
@workflow(name="deploiement")
class DeployWorkflow:
    def execute(self, ctx):
        # Activité 1: Build — retry automatique
        ctx.activity(
            action="docker_build",
            retry=RetryPolicy(max_attempts=3, backoff="exponential"),
            compensation="docker_rmi"
        )
        
        # Activité 2: Approbation humaine — attente illimitée
        ctx.await_signal(
            signal_type="human_approval",
            timeout_days=7
        )
        
        # Activité 3: Déploiement
        ctx.activity(
            action="docker_compose_up",
            retry=RetryPolicy(max_attempts=2, backoff="constant"),
            compensation="docker_compose_down"
        )
```

#### 3.3.3 Périmètre

Seuil configurable. Par défaut :
- Actions > 5 secondes
- Appels outils externes
- Toute écriture (fichier, mémoire, base de données)
- Actions non-idempotentes

Un appel LLM simple ne passe PAS par le workflow engine.

#### 3.3.4 Reprise après panne

```
1. Au démarrage, le Workflow Engine scanne SQLite: workflows.status IN ('running', 'waiting_approval')
2. Pour chaque workflow interrompu:
   a. Vérifier l'idempotence de la dernière activité (activity_id déjà exécuté?)
   b. Si non exécutée → exécuter
   c. Si exécutée → passer à la suivante
   d. Le Superviseur revalide les permissions (le contexte a pu changer)
3. Continuer l'exécution normalement
```

#### 3.3.5 Compensation (Saga)

```python
class SagaCoordinator:
    def execute(self, activities):
        completed = []
        try:
            for activity in activities:
                activity.execute()
                completed.append(activity)
        except Exception:
            # Compenser en ordre inverse
            for activity in reversed(completed):
                activity.compensate()
            raise
```

### 3.4 Écosystème d'outils

#### 3.4.1 Découverte dynamique

```
Priorité de résolution d'outil:
1. Outils first-party déjà connectés (utilisation directe)
2. Registre MCP → suggestion de connecteurs
3. Navigateur web (dernier recours)

Distinction critique:
- First-party: utilisable sans confirmation
- Third-party: toujours via suggest_connectors, jamais automatique
```

#### 3.4.2 Skills

Chaque skill = un dossier avec `SKILL.md`. Chargement obligatoire avant création.

| Tâche | Skill chargé |
|-------|-------------|
| Présentation | `pptx` |
| Tableur | `xlsx` |
| PDF | `pdf` |
| Composant frontend | `frontend-design` |
| Analyse CSV | `data-analysis` |
| Diagramme | `diagram` |

### 3.5 Préférences

Deux familles, résolution ordonnée :

```
1. Instruction explicite dans la requête courante (priorité maximale)
2. Préférence stockée avec marqueur "always"
3. Style configuré (userStyle)
4. Préférence stockée sans marqueur "always"
5. Défaut système
```

**Behavioral guardrails:** ne jamais persister : flatterie inconditionnelle, suppression du désaccord, dépendance émotionnelle, abandon de l'évaluation honnête.

### 3.6 Sortie visuelle

#### Arbre de décision

```
1. La réponse est-elle purement textuelle? → texte seul
2. Un outil MCP connecté gère-t-il cette catégorie? → utiliser l'outil
3. L'utilisateur demande-t-il explicitement un fichier? → créer le fichier
4. Le contenu mérite-t-il une visualisation? → visualiseur inline
   (patterns: "montre-moi", "diagramme", "visualise", structure spatiale)
5. Sinon → texte seul
```

**Déclencheur à deux étages:** patterns + mots-clés (rapide) → décision LLM (confirmation).

**Technologie:** HTML/SVG inline rendu dans iframes sandboxées. Kroki (déjà dans la stack Docker) pour les diagrammes Mermaid/PlantUML.

**Modules de design:** chaque module (diagram, mockup, interactive, chart, art, data_viz) = un skill chargé avant génération.

---

## 4. Orchestration et autonomie

### 4.1 Niveaux d'autonomie

Paramètre configurable par projet :

| Niveau | Comportement | Usage |
|--------|-------------|-------|
| **Supervisé** (défaut) | Le système exécute, escalade aux points critiques | Projets standards |
| **Contrôlé** | Chaque phase majeure nécessite validation humaine | Projets sensibles |
| **Autonome** | Aucune interruption sauf budget épuisé ou action destructive | Projets de confiance |

### 4.2 Matrice de risque

Le Superviseur évalue dynamiquement :

| Action | Risque | Comportement supervisé | Comportement autonome |
|--------|--------|----------------------|---------------------|
| Lecture mémoire | FAIBLE | Auto | Auto |
| Écriture fichier projet | MODÉRÉ | Auto | Auto |
| Appel API externe | MODÉRÉ | Auto | Auto |
| Nouveau service tiers | ÉLEVÉ | Escalade | Escalade |
| Action destructive (`rm -rf`) | CRITIQUE | Escalade | Escalade |
| Dépassement budget | CRITIQUE | Stop + notif | Stop + notif |

### 4.3 Multi-agent

Le Chef d'orchestre peut instancier des sous-agents. Les 3 critères de décomposition (§5.1.2) sont des garde-fous, pas des verrous :

- **Protection de contexte:** une sous-tâche produit > 1000 tokens non pertinents
- **Parallélisation:** sous-problèmes indépendants sans état partagé
- **Spécialisation:** > 15 outils ou domaines non liés

Types de sous-agents :
- **Spécialisés:** par contexte (pas par métier), héritent d'un contexte filtré
- **Vérification:** boîte noire, exécute les tests, ne reçoit pas l'historique d'implémentation
- **Dynamiques:** instanciés à la volée pour une tâche spécifique

---

## 5. Plan d'implémentation

### 5.1 Ordre

```
Phase 5.1.5 — Document de Conception (cette phase)
    ↓
Phase 6.0 — Bus de pensée event-driven
    ↓
Phase 6.1 — Exécution durable (Custom SQLite)
    ↓
Phase 6.2 — Mémoire avec provenance (MD + ChromaDB + Graphify)
    ↓
Phase 6.3 — Préférences utilisateur
    ↓
Phase 6.4 — Sortie visuelle et multimodalité
```

### 5.2 Dépendances

```
Bus de pensée
    ├── Exécution durable ──→ Mémoire avec provenance ──→ Préférences
    │                              │
    │                              └──→ Sortie visuelle (indépendant)
    │
    └── Multi-agent (après Exécution durable)
```

### 5.3 Alignement SFD

À la fin de la Vague 2 (Mémoire), alignement estimé : 45% → **85%**.

| Module SFD | Phase | Statut après |
|-----------|-------|-------------|
| Exécution durable (§5.5) | 6.1 | 🟢 Actif |
| Mémoire + provenance (§5.7) | 6.2 | 🟢 Actif |
| Préférences (§5.15) | 6.3 | 🟢 Actif |
| Sortie visuelle (§5.18) | 6.4 | 🟢 Actif |
| Planification (§5.3) | 6.0 | 🟢 Intégré au bus |
| Multi-agent (§5.1) | Post-6.1 | 🟡 Progressif |

---

## 6. Modèles de données

### 6.1 Fichier mémoire

```yaml
---
name: <slug>
description: <une ligne>
sources: [chat, claude-code, api]
aliases: []
version: <hash SHA-256 tronqué 12 car.>
retention: public | internal | personal
---

- [stated] <fait>
- [observed] <fait>
- [inferred] <fait> (confiance: 0.X)
```

### 6.2 Workflow durable

```yaml
workflow:
  id: "wf-uuid"
  name: "deploiement"
  status: running | waiting_approval | completed | failed | compensating
  project_id: "proj-uuid"
  current_activity: "build"
  created_at: "2026-07-22T14:30:00Z"
  activities:
    - id: "act-uuid"
      action: "docker_build"
      status: completed
      attempt: 1
      started_at: "..."
      completed_at: "..."
      compensation: "docker_rmi"
```

### 6.3 Événement du bus de pensée

```yaml
event:
  type: phase_enter | phase_exit | module_subscribe | signal
  phase: BUILD
  context_snapshot:
    project_id: "proj-uuid"
    tokens_used: 1840
    active_agents: [conductor]
  timestamp: "2026-07-22T14:30:00Z"
```

### 6.4 Préférence

```yaml
preference:
  type: behavioral | contextual
  subtype: format | ton | langue | expertise | background | interets
  rule: "utilise des listes à puces"
  scope: always | selective
  created_at: "2026-07-22T14:30:00Z"
```

---

## 7. Gouvernance et kill-switches

### 7.1 Deux classes de modules

| Classe | Défaut | Exemples | Condition d'activation |
|--------|--------|----------|----------------------|
| **Cœur stable** | ON | Bus de pensée, Context Engine, Exécution durable, Mémoire | Testé et approuvé |
| **Expérimental** | OFF | Multi-agent avancé, Découverte outils, Mode autonome | Activation explicite |

### 7.2 Kill-switches par module

Chaque module SFD est livré avec son kill-switch :

```
ODYSSEUS_THOUGHT_BUS=ON        # Bus de pensée
ODYSSEUS_DURABLE_EXEC=ON       # Exécution durable
ODYSSEUS_MEMORY_PROVENANCE=ON  # Mémoire avec provenance
ODYSSEUS_PREFERENCES=ON        # Préférences
ODYSSEUS_VISUAL_OUTPUT=ON      # Sortie visuelle
ODYSSEUS_MULTI_AGENT=OFF       # Multi-agent avancé
ODYSSEUS_TOOL_DISCOVERY=OFF    # Découverte outils
```

### 7.3 Observabilité

Chaque composant émet des traces structurées OpenTelemetry `gen_ai.*`. Pas de `print()`, pas de logs ad-hoc.

- **Span d'invocation agent** : modèle, tokens, latence, motif de fin
- **Span d'appel outil** : nom, arguments, résultat, erreur
- **Span de décision d'orchestration** : pourquoi déléguer, à qui, critère
- **Événement d'évaluation** : score de succès, fidélité, signal sécurité

---

## 8. Annexes

### A. Références croisées

| Document | Chemin |
|----------|--------|
| SFD v3.0 | `../../SFD.md` |
| Vue d'ensemble architecture | `overview.md` |
| Boucle agent | `agent-loop.md` |
| Services | `services.md` |
| Flux de données | `data-flow.md` |
| ROADMAP GSD | `../../.planning/ROADMAP.md` |
| STATE | `../../.planning/STATE.md` |
| Contexte phase 5.1.5 | `../../.planning/phases/05-activation/5.1.5-design-conception/5.1.5-CONTEXT.md` |

### B. Mapping principes SFD → décisions

| Principe | Décision de conception |
|----------|----------------------|
| P1 — Risque modifie la boucle | Matrice de risque dynamique, escalade configurable |
| P3 — Contexte construit | Context Engine avec filtrage sémantique + phase + quota |
| P10 — Humain ON the loop | Autonomie supervisée par défaut, configurable par projet |
| P11 — Commencer simple | Kill-switches deux classes, modules expérimentaux OFF |
| P12 — Découpage par contexte | Sous-agents par contexte, pas par métier |
| P13 — Contexte = budget | Compaction automatique à 80% fenêtre, résumé LLM |
| P14 — Actions longues durables | Custom SQLite, seuil durée/type configurable |
| P15 — Observabilité dès conception | Traces structurées gen_ai.* obligatoires |
| P16 — Provenance explicite | Tags [stated]/[observed]/[inferred] sur chaque fait |
| P17 — Ne pas stocker sensible | Règles d'omission strictes, 3 catégories protégées |
| P18 — Lire avant d'écrire | if_version = hash contenu, rejet si divergence |
| P20 — Préférences par priorité | Résolution ordonnée: requête > always > style > selective > défaut |
| P22 — Sortie visuelle premier rang | Arbre décision patterns+LLM, HTML/SVG inline + Kroki |

### C. Config YAML de référence

```yaml
# workspace/config.yaml
agent_os:
  autonomy:
    level: supervised       # supervised | controlled | autonomous
    escalation_matrix:
      destructive: critical  # critical: always escalate
      third_party: high      # high: escalate in supervised/controlled
      budget_exceeded: critical
      
  thought_bus:
    enabled: true
    phases: [classify, know, plan, build, quality, autoeval, memory_observe]
    
  context_engine:
    compaction_threshold: 0.8       # % de la fenêtre de contexte
    compaction_model: "haiku"       # modèle léger pour les résumés
    filter_quota: 10                # entrées max par injection
    notepad_backend: "chromadb"     # vectoriel
    
  durable_execution:
    engine: "sqlite"
    threshold_ms: 5000              # actions > 5s = durables
    retry_defaults:
      max_attempts: 3
      backoff: exponential
      
  memory:
    storage: [markdown, chromadb, graphify]
    versioning: "content_hash"
    taxonomy: [profile, topics, areas, people, preferences]
    
  visual_output:
    renderer: "html_svg_inline"
    diagram_backend: "kroki"
    trigger: "pattern_llm"          # patterns + confirmation LLM
    
  kill_switches:
    thought_bus: on
    durable_exec: on
    memory_provenance: on
    preferences: on
    visual_output: on
    multi_agent: off
    tool_discovery: off
```

---

*Document de Conception v1.0 — Issu de la phase GSD 5.1.5*
