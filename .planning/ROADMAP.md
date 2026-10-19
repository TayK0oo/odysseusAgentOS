# Roadmap: odysseusAgentOS-omega (Omega)

**Created:** 2026-10-17 · **Updated:** 2026-10-18 (v0.3)
**Status:** In Progress

## Milestone: v0.2 — NOYAU Phase A→D

**Status:** ✅ Shipped (2026-10-18) — archive : `.planning/milestones/v0.2-phases/`, détails `.planning/MILESTONES.md`
Phases 1–5 : squelette effets réversibles · coutures MemoryStore/Hook/SubagentRunner · sandbox namespace + compaction double couche · provider sous-agents externes · fermeture des 3 trous.

## Milestone: v0.3 — CORPUS 100%

**Status:** In Progress
**Objectif :** diqueter les 16 harnais OSS restants (intake clone + REA machine + dissection validée), puis raffiner le combine déterministe pour un regroupement final de toutes les fonctionnalités.

### Phase 6: Intake vague 1 — top étoiles

**Goal:** Les 6 harnais les plus populaires restants sont disséqués et validés : Onlook, Roo Code, Dyad, SWE-agent, Devika, Agent Zero.
**Depends on:** None
**Requirements:** DIS-01, DIS-02, DIS-03, DIS-04, DIS-05, DIS-06, REA-01, REA-02
**Success criteria:**
1. 6 fiches `dissections/<id>.md` validées par `validate_dissections.py` (0 erreur).
2. Taux REA machine chiffré par harnais (succès/échec + fichiers analysés).
3. Écarts langages hors surface REA (Python/Rust/Go) documentés par fiche avec raison datée `[observed]`.
4. Fiches enregistrées via `register_dissections.py` (intake + provenance).

### Phase 7: Intake vague 2 — mi-peuplée

**Goal:** SuperAGI, bolt.new, Codebuff, Claude Engineer, Bloop, Sweep disséqués et validés.
**Depends on:** Phase 6
**Requirements:** DIS-07, DIS-08, DIS-09, DIS-10, DIS-11, DIS-12
**Success criteria:**
1. 6 fiches validées ( script + structure conforme).
2. Chaque fiche propose un port/adaptateur + exemplaire de référence avec preuves `fichier:ligne`.
3. Preuves REA attachées quand le langage est dissectable.

### Phase 8: Intake vague 3 — queue OSS

**Goal:** Refact.ai, Nanocoder, Sourcegraph Cody, Mentat disséqués ; les 2 dépôts « 0 étoiles API 404 » résolus (renommage/déplacement).
**Depends on:** Phase 7
**Requirements:** DIS-13, DIS-14, DIS-15, DIS-16
**Success criteria:**
1. 4 fiches validées (Cody/Mentat avec dépôt réel résolu et note de résolution).
2. 0 harnais OSS silencieux : chaque entrée du census est dissequée ou documentée.

### Phase 9: Consolidation — OMEGA-COMPLET v2

**Goal:** Le corpus final (51 harnais) est regroupé intelligemment et entièrement déterministe.
**Depends on:** Phase 8
**Requirements:** CONS-01, CONS-02, CONS-03
**Success criteria:**
1. `omega_complet.py` + `register_dissections.py` consomment 51 dissections, zéro édition manuelle.
2. `OMEGA-COMPLET.md` v2 regroupé par famille avec consensus pondéré popularité (>=4 demandeurs), adéquation et adaptateurs proposés.
3. `HARNESS-CENSUS.md` recalculé : 66 recensés · 51 disséqués · 3 écarts documentés (copilot-chat, kimi-cli, pearai) · 0 silencieux.

### Phase 10: Organisation — docs & planning

**Goal:** Toute la mémoire projet reflète le corpus 100%.
**Depends on:** Phase 9
**Requirements:** ORG-01, ORG-02, ORG-03
**Success criteria:**
1. `generate_memory.py` sort les stats corpus à jour (harnais, fonctionnalités, preuves REA).
2. `docs/omega/INDEX.md` régénéré avec les nouvelles fiches + census.
3. MEMORY.md « Prochaines actions » sans résidus v0.2 (trou 1 marqué résolu, entrée INDEX ✅).

## Milestone: v0.4 — PLUGIN CONVERGENCE

**Status:** ✅ Shipped (2026-10-19) — archive : `.planning/milestones/v0.4-phases/`, détails `.planning/MILESTONES.md`

**Status:** In Progress
**Objectif :** Adapter l'architecture complète au modèle "everything is a plugin" en intégrant les capacités extraites du corpus (51 harnais + 148 vidéos TikTok) + travail existant odysseusAgentOS.

### Phase 11: Convergence des outils TikTok

**Goal:** Extraire et classifier tous les outils des 148 vidéos TikTok (3 comptes) et créer des adaptateurs plugin pour chaque outil identifié.
**Depends on:** Phase 10
**Requirements:** CONV-01, CONV-02, CONV-03
**Success criteria:**
1. Tous les outils TikTok sont extraits et classifiés par catégorie (IA, SaaS, skills, GitHub, etc.)
2. Chaque outil a un port/adaptateur proposé dans l'architecture plugin
3. Documentation complète dans `docs/omega/reverse-eng/tiktok/`

### Phase 12: Adaptation architecture "everything is a plugin"

**Goal:** Créer les 8 nouveaux ports et implémenter les adaptateurs pour chaque port. Mettre à jour le registre de plugins.
**Depends on:** Phase 11
**Requirements:** ARCH-01, ARCH-02, ARCH-03, ARCH-04
**Success criteria:**
1. 8 nouveaux ports créés (Registry, UiSurface, ConfigSource, SkillStore, AgentLoop, AgentStore, WorkflowEngine, SandboxBackend)
2. Chaque port a au moins un adaptateur implémenté
3. Le registre de plugins gère l'installation/activation/désactivation
4. Tests d'intégration pour chaque port

### Phase 13: Intégration du travail existant (odysseusAgentOS)

**Goal:** Importer les adaptateurs pertinents du repo principal. Fusionner les architectures (Python + Rust). Créer les ponts nécessaires.
**Depends on:** Phase 12
**Requirements:** INTEG-01, INTEG-02, INTEG-03
**Success criteria:**
1. Adaptateurs Python importés et wrappés en plugins Rust
2. Ponts (bridge) créés pour l'interopérabilité
3. Documentation de l'intégration

### Phase 14: Validation end-to-end

**Goal:** Tests d'intégration pour chaque port. Tests de plugin (install/uninstall/reload). Validation sandbox. Documentation complète.
**Depends on:** Phase 13
**Requirements:** VALID-01, VALID-02, VALID-03
**Success criteria:**
1. Tests d'intégration pour chaque port (16 existants + 8 nouveaux)
2. Tests de plugin (install/uninstall/reload)
3. Validation sandbox complète
4. Documentation complète mise à jour

## Milestone: v0.5 — IMPLEMENTATION

**Status:** ✅ Shipped (2026-10-19) — archive : `.planning/milestones/v0.5-phases/`, détails `.planning/MILESTONES.md`

**Status:** In Progress
**Objectif :** Implémenter les adaptateurs pour les 8 nouveaux ports, le PythonBridge (JSON-RPC), les tests d'intégration, et intégrer le travail existant odysseusAgentOS.

### Phase 15: Adaptateurs pour les 8 nouveaux ports

**Goal:** Implémenter au moins un adaptateur pour chaque nouveau port (Registry, UiSurface, ConfigSource, SkillStore, AgentLoop, AgentStore, WorkflowEngine, SandboxBackendV2).
**Depends on:** Phase 14
**Requirements:** IMPL-01, IMPL-02, IMPL-03
**Success criteria:**
1. Chaque port a au moins un adaptateur implémenté et testé
2. Le registre de plugins gère l'installation/activation/désactivation
3. Tests unitaires object-safety pour chaque port

### Phase 16: PythonBridge (JSON-RPC)

**Goal:** Implémenter le pont Python ↔ Rust via JSON-RPC over stdin/stdout avec timeout + retry + circuit breaker.
**Depends on:** Phase 15
**Requirements:** IMPL-04, IMPL-05, IMPL-06
**Success criteria:**
1. PythonBridge fonctionnel (subprocess Python + JSON-RPC)
2. Ponts haute priorité implémentés (PythonAgentBridge, PythonToolBridge, PythonMiddlewareBridge)
3. Tests d'interopérabilité Rust ↔ Python

### Phase 17: Intégration odysseusAgentOS

**Goal:** Importer les adaptateurs pertinents du repo principal. Fusionner les architectures (Python + Rust).
**Depends on:** Phase 16
**Requirements:** IMPL-07, IMPL-08, IMPL-09
**Success criteria:**
1. Adaptateurs Python importés et wrappés en plugins Rust
2. Ponts créés pour l'interopérabilité
3. Documentation de l'intégration mise à jour

### Phase 18: Validation end-to-end

**Goal:** Tests d'intégration complets. Validation sandbox. Documentation finale.
**Depends on:** Phase 17
**Requirements:** IMPL-10, IMPL-11, IMPL-12
**Success criteria:**
1. Tests d'intégration pour chaque port (16 existants + 8 nouveaux)
2. Tests de plugin (install/uninstall/reload)
3. Validation sandbox complète
4. Documentation complète mise à jour

## Milestone: v0.6 — ADAPTATEURS & INTEGRATION

**Status:** ✅ Shipped (2026-10-19) — archive : `.planning/milestones/v0.6-phases/`, détails `.planning/MILESTONES.md`

**Status:** In Progress
**Objectif :** Implémenter les adaptateurs spécifiques pour chaque port, le PythonBridge handler, les tests d'intégration end-to-end, et intégrer le travail existant odysseusAgentOS.

### Phase 19: Adaptateurs spécifiques pour les ports

**Goal:** Implémenter au moins un adaptateur spécifique (pas juste les defaults) pour chaque port existant (16) et nouveau (8). Remplacer les defaults par des implémentations fonctionnelles.
**Depends on:** Phase 18
**Requirements:** ADAPT-01, ADAPT-02, ADAPT-03
**Success criteria:**
1. Chaque port a au moins un adaptateur spécifique implémenté
2. Les adaptateurs sont testés (unit + integration)
3. Le registre de plugins gère l'installation/activation/désactivation

### Phase 20: PythonBridge handler + tests E2E

**Goal:** Implémenter le script Python handler pour le bridge, tester l'interopérabilité Rust ↔ Python end-to-end.
**Depends on:** Phase 19
**Requirements:** BRIDGE-01, BRIDGE-02, BRIDGE-03
**Success criteria:**
1. Script Python handler fonctionnel (JSON-RPC dispatcher)
2. Tests E2E Rust ↔ Python passent
3. Timeout + retry + circuit breaker validés

### Phase 21: Intégration odysseusAgentOS effective

**Goal:** Importer les adaptateurs pertinents du repo principal. Fusionner les architectures (Python + Rust). Créer les ponts nécessaires.
**Depends on:** Phase 20
**Requirements:** INTEG-04, INTEG-05, INTEG-06
**Success criteria:**
1. Adaptateurs Python importés et wrappés en plugins Rust
2. Ponts créés et testés
3. Documentation mise à jour

### Phase 22: Validation sandbox + déploiement

**Goal:** Validation sandbox complète. Tests de plugin (install/uninstall/reload). Documentation finale. Déploiement.
**Depends on:** Phase 21
**Requirements:** VALID-04, VALID-05, VALID-06
**Success criteria:**
1. Tests de plugin (install/uninstall/reload)
2. Validation sandbox complète
3. Documentation finale mise à jour
4. Déploiement via validate.sh → run.sh → deploy.sh

## Milestone: v0.7 — PORT RUST COMPLET

**Status:** ✅ Shipped (2026-10-19) — archive : `.planning/milestones/v0.7-phases/`, détails `.planning/MILESTONES.md`

**Status:** In Progress
**Objectif :** Portage complet de odysseusAgentOS (Python) vers Rust (omega), intégrant les capacités pertinentes dans l'architecture "everything is a plugin".

### Phase 23: Audit et cartographie odysseusAgentOS

**Goal:** Auditer le codebase odysseusAgentOS (13k+ fichiers Python) et identifier les composants pertinents à porter en Rust. Créer une cartographie complète.
**Depends on:** Phase 22
**Requirements:** PORT-01, PORT-02, PORT-03
**Success criteria:**
1. Audit complet du codebase odysseusAgentOS (services, routes, core, tools)
2. Cartographie des composants pertinents → ports Omega
3. Priorisation par impact et complexité

### Phase 24: Portage des services core

**Goal:** Porter les services core (memory, search, security, observability) en Rust avec adaptateurs plugin.
**Depends on:** Phase 23
**Requirements:** PORT-04, PORT-05, PORT-06
**Success criteria:**
1. Services core portés en Rust
2. Adaptateurs plugin créés
3. Tests passent

### Phase 25: Portage des outils d'agent

**Goal:** Porter les 100+ outils d'agent (agent_tools/) en Rust avec adaptateurs plugin.
**Depends on:** Phase 24
**Requirements:** PORT-07, PORT-08, PORT-09
**Success criteria:**
1. Outils d'agent portés en Rust
2. Adaptateurs plugin créés
3. Tests passent

### Phase 26: Portage des routes et API

**Goal:** Porter les routes API en Rust (omega-server) avec intégration des services portés.
**Depends on:** Phase 25
**Requirements:** PORT-10, PORT-11, PORT-12
**Success criteria:**
1. Routes API portées en Rust
2. Intégration avec les services portés
3. Tests passent

### Phase 27: Validation et déploiement

**Goal:** Validation complète du portage. Tests end-to-end. Documentation finale. Déploiement.
**Depends on:** Phase 26
**Requirements:** PORT-13, PORT-14, PORT-15
**Success criteria:**
1. Tests d'intégration complets
2. Validation sandbox
3. Documentation finale
4. Déploiement
