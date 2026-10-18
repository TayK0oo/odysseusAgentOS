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
