# 00 — VISION-OMEGA

> **La branche de rêve.** Document de référence de `feat/agentos-omega`.
> Compagnons : `docs/omega/rules/RULES-OMEGA.md` (règles), `docs/omega/MEMORY.md` (état),
> `docs/omega/adr/` (décisions), `docs/omega/reverse-eng/` (extraction des harnais).
> **Date :** 2026-10-15.

---

## 0. North Star

> **Un harnais d'agent universel où chaque brique est un plugin interchangeable, où le
> coût de calcul est payé par le SYSTÈME et non par le modèle, et où chaque décision est
> traçable et rejouable — pour faire plus, mieux, avec moins.**

**Core value :** *Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver,
apprend de chaque run, ne dépasse jamais ses limites — sans intervention humaine constante.*

**Slogan :** *Everything is a plugin.*

**Devise :** *Interchangeable · Traçable · Économique.*

---

## 1. Le problème que l'on résout

Les harnais d'aujourd'hui (Claude Code, Codex, Cline, Goose…) sont excellents mais :
1. **monolithiques** — remplacer une brique (contexte, mémoire, routage, sandbox) est
   difficile ou impossible ; on subit les choix de l'éditeur ;
2. **coûteux** — chaque opération déterministe (parcourir, indexer, formater) finit en
   appel de modèle ;
3. **opaques** — on ne sait pas pourquoi le système a décidé X, ni comment le rejouer ;
4. **enfermés** — la politique de l'éditeur (limites, refus, coûts) s'impose.

Omega prend le meilleur de chacun (voir `reverse-eng/`) et le rend **modulaire, frugal,
transparent et souverain**.

---

## 2. Les 6 piliers

| # | Pilier | Traduction concrète |
|---|---|---|
| 1 | **Routing intelligent** | Registre multi-fournisseurs ; chaque modèle a un *purpose*, un coût, une vitesse, des forces mesurées par des benchmarks communautaires ; choix par type de tâche (vitesse ↔ réflexion) ; fallback ; self-hosted au besoin. |
| 2 | **Reverse-engineering** | Capacité reproductible d'extraire features et contrats des harnais du marché, puis de les re-spécifier proprement. |
| 3 | **Everything is a plugin** | Ports + adaptateurs + registre de capacités ; rien en dur ; configuration > code. |
| 4 | **Rust core / React-TS UI** | Noyau performant et sûr ; interface claire, réactive, observable. |
| 5 | **Économie & traçabilité** | Le déterminisme va au système (scripts/outils) ; tout est mesuré, prouvé, daté, rejouable. |
| 6 | **Adaptatif & évolutif** | Le système apprend de chaque run (scoring, feedback, distillation) sans dépendre d'un fournisseur unique. |

---

## 3. L'axe unique : *Everything is a plugin*

### 3.1 Définition d'un plugin

Un **plugin** est une unité qui **implémente une capacité** derrière un **port** :

```
Plugin = (Capacité, Manifeste, Implémentation, Tests d'acceptation)
```

- **Capacité** — un besoin du système (`llm-provider`, `context-engine`, `memory-store`,
  `tool`, `sandbox-backend`, `router-policy`, `catalog-source`, `ui-panel`…).
- **Manifeste** — déclaratif (schema-validé) : id, capacité, version, config schema,
  capacité requise (« capabilities »), points d'entrée, permissions.
- **Implémentation** — un adaptateur (Rust natif, WASM, ou service distant).
- **Tests d'acceptation** — la preuve que l'adaptateur respecte le contrat du port.

### 3.2 Deux régimes de capacité (issu du RE des harnais)

- **Slot exclusif** — une seule implémentation active à la fois (ex. `context-engine`).
  On peut en changer, mais pas en empiler.
- **Ensemble concurrent** — N implémentations actives (ex. `llm-provider`, `tool`).

### 3.3 Règles d'interchangeabilité

1. Le noyau ne connaît que des **traits/ports**, jamais des implémentations.
2. Une brique se remplace par **manifeste + config**, sans recompiler le noyau.
3. Découverte **dynamique** (scan de répertoires + registre), pas d'import en dur.
4. Toute frontière a un **schéma typé et versionné**.
5. Un plugin **déclare** ses permissions ; le noyau les **applique** (policy par code).

---

## 4. Architecture — « pieuvre » hexagonale

```
                    ┌──────────────────────────── UI React/TS ───────────────────────────┐
                    │  Cockpit · Plugin Explorer · Router Dashboard · Trace Viewer        │
                    └───────────────────────────▲────────────────────────────────────────┘
                                                │ API typée (HTTP/WS) — schéma versionné
        ┌───────────────────────────────────────┴────────────────────────────────────────┐
        │                          omega-server  (frontière adaptateur)                    │
        └───────────────────────────────────────▲────────────────────────────────────────┘
                                                │
   ┌───────────────────────────────  NOYAU (ports)  ───────────────────────────────────────┐
   │  omega-core (types, capacités, registre, erreurs)                                      │
   │  omega-ports (traits) : LlmProvider · ContextEngine · MemoryStore · ToolExecutor ·      │
   │           SandboxBackend · RouterPolicy · CatalogSource · TelemetrySink · UiPanel       │
   │  omega-runtime (boucle d'agent, orchestration, event-bus, goal-ancestry)                │
   └───────▲───────────▲───────────▲───────────▲───────────▲───────────▲───────────▲────────┘
           │           │           │           │           │           │           │
      ┌────┴───┐  ┌────┴───┐  ┌────┴───┐  ┌────┴───┐  ┌────┴───┐  ┌────┴───┐  ┌────┴───┐
      │ router │  │context │  │ memory │  │ tools  │  │sandbox │  │telemetry│ │plugins │
      └────────┘  └────────┘  └────────┘  └────────┘  └────────┘  └────────┘  └────────┘
        (adaptateurs enfichables : providers, moteurs de contexte, stores, backends…)
```

Le noyau est aveugle aux marques. Tout le reste est **adaptateur**.

### 4.1 Workspace Rust proposé (à confirmer par `RE-GOOSE-CODEX.md`)

```
omega/
  Cargo.toml                 # workspace
  crates/
    omega-core/              # domaine : types, capacités, registre, erreurs, config typée
    omega-ports/             # traits (les ports) — aucune dépendance d'implémentation
    omega-runtime/           # boucle d'agent, orchestration, event-bus, durable execution
    omega-router/            # registre modèles, scoring, politique, fallback, feedback
    omega-context/           # Context Engine : ingest → assemble → compact → after-turn
    omega-memory/            # mémoire à provenance + session search (FTS)
    omega-tools/             # registre d'outils, tool search (chargement différé des schémas)
    omega-sandbox/           # exécution isolée multi-backend (local/docker/ssh/serverless)
    omega-telemetry/         # traces, provenance, OTel, replay
    omega-plugins/           # chargeur de plugins : manifeste, découverte, permissions
    omega-cli/               # binaire CLI
    omega-server/            # API pour l'UI (adaptateur de bord)
  ui/                        # app React/TS (Vite + TS) — voir §6
  plugins/                   # plugins livrés (natif/WASM) + exemples
  docs/
```

**Sens des dépendances :** `cli/server → runtime → ports → core`. Jamais l'inverse.
Un crate d'adaptateur dépend de `ports`, jamais de `runtime`.

---

## 5. Le routeur intelligent (pilier 1)

Source détaillée : `docs/omega/research/MODEL-ROUTING-BENCHMARKS.md`.

- **Registre de capacités** : chaque modèle = (id, provider, contexte, coût, vitesse,
  TTFT, forces par axe : raisonnement/code/agentique/knowledge/chat/vision, limites de
  débit, support outils).
- **Scoring composite** alimenté par des **sources communautaires** (Artificial Analysis,
  Epoch AI/ECI, SWE-bench Verified, Terminal-Bench, Aider Polyglot, LiveBench, HF
  Leaderboard, LMArena) — qualité ≠ score unique, on garde des axes.
- **Politique par type de tâche** : `T1 trivial → modèle rapide & bon marché` ;
  `raisonnement long/code difficile → modèle élite`. Un **classifieur déterministe**
  (zéro LLM) route ; escalade si échec.
- **Fallback** : même tier → tier supérieur → self-hosted → file/backoff ; circuit breaker.
- **Feedback loop** : bandit (UCB1/Thompson) qui ajuste les poids selon les succès réels.
- **Souveraineté** : si un fournisseur bloque un sujet, bascule vers un modèle self-hosted.

---

## 6. L'UI React/TS (pilier 4) — « un cockpit clair et intuitif »

Nouvelle app isolée (`omega/ui/`), branchée par API typée. Principes :
1. **Clarté d'abord** — chaque écran répond à une question : que fait le système ? pourquoi ?
   combien ça coûte ? où est le contexte ?
2. **Panneaux clés** :
   - **Cockpit** — état live du run (phases, budget tokens, modèle courant).
   - **Plugin Explorer** — capacités, plugins installés, slot actif, config éditable.
   - **Router Dashboard** — modèles, scores par axe, coût/vitesse, décisions récentes.
   - **Trace Viewer / Replay** — arbre des décisions, provenance, rejeu.
   - **Context Inspector** — ce qui est dans le contexte et pourquoi (budget par source).
3. **Design** : composants accessibles (cf. skill `magic-ui`), thème sombre, densité
   maîtrisée, états vides et erreurs soignés.
4. **Contrats** : types générés depuis les schémas du serveur (source unique de vérité).

---

## 7. Économie de contexte & de calcul (pilier 5)

- **Déterminisme au système** : indexation, recherche, parsing, diff, scoring = scripts/outils,
  **jamais** un tour de LLM. « Si ça s'écrit en 20 lignes de Rust → ce n'est pas une tâche d'agent. »
- **Progressive disclosure** : ne charger que les *descriptions*, charger le contenu à l'usage.
- **Tool search** : schémas d'outils différés, récupérés à la demande (BM25, cf. Hermes).
- **Compaction** : résumé structuré + stabilité du préfixe (prompt caching).
- **Sous-agents isolés** : seul un résumé remonte ; le contexte parent reste léger.
- **Budget** : chaque appel estimé, tracé, plafonné ; dépassement = arrêt.

## 8. Traçabilité & adaptativité (piliers 5-6)

- **Tout est un événement** (event-bus) : décision, appel modèle, outil, coût, erreur.
- **Provenance** : `[stated]/[observed]/[inferred]` partout (mémoire, réponses, docs).
- **Replay** : rejouer une session à partir des traces, sans ré-exécuter les effets.
- **Apprentissage** : distillation des runs → ajustement des scores de routing, création de
  skills, mémoire consolidée (boucle auto-améliorante, cf. Hermes).

## 9. Économie & souveraineté (pilier 5)

- **Gratuit / self-hosted d'abord** : modèles locaux (Ollama/llama.cpp), gateways, cache ;
  on ne paie que ce qui le justifie.
- **Aucune dépendance à un fournisseur unique.**
- **Aucune limite morale imposée pour usage strictement personnel** : sujet bloqué → self-hosted.

## 10. Non-objectifs (pour l'instant)

- ❌ Réutiliser le socle Python *tel quel* comme moteur (réécriture Rust assumée ; le socle
  sert de **catalogue de besoins et de tests de comportement**, pas de base de code).
- ❌ Distribution microservices par dogme (microservice seulement si isolation/échelle/crash-boundary).
- ❌ Génération média/pub, navigateur autonome visible (hors scope court terme).

## 11. Métriques de succès (à mesurer, pas à espérer)

| Axe | Métrique |
|---|---|
| Interchangeabilité | Temps pour remplacer une brique (context-engine) **sans toucher au noyau** |
| Économie | Part des opérations résolues **sans** appel LLM ; tokens/run ; coût/run |
| Qualité | Taux de tâches terminées sans intervention ; taux de fallback ; succès par tier |
| Traçabilité | % d'étapes tracées et rejouables |
| Vitesse | TTFT p50/p95 ; latence de routing p95 |
| Robustesse | Taux de crash par brique (isolation) ; récupération après erreur |

## 12. Phasage (voir `docs/omega/workstreams/`)

`W0 Fondation → W1 Recherche → W2 Plugin → W3 Rust → W4 Routing → W5 Reverse-eng →
W6 Contexte → W7 UI → W8 Traçabilité → W9 Sandbox → W10 Économie`.

Chemin critique : **W5 (en cours) → W2 → W3+W4 → W6/W7/W8.**

---

*Fin de la vision. Toute évolution passe par un ADR.*
