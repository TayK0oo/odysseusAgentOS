# Design — Milestone 2 : Intégration réelle du harness dans Odysseus

**Date :** 2026-07-01
**Statut :** Approuvé (design), en attente de revue spec puis roadmap GSD
**Auteur :** Session brainstorming (superpowers:brainstorming)

---

## Sources de référence (à lire avant toute implémentation)

Ce design est ancré sur trois corpus. Toute décision ci-dessous doit rester cohérente avec eux.

| Source | Chemin | Rôle |
|---|---|---|
| **Analyse d'outils profonde** | `.planning/AGENT-OS-ANALYSE-PROFONDE.md` (copie durable de `~/Downloads/AGENT-OS-ANALYSE-PROFONDE(2).md`, 722 lignes) | La **vision cible** : 60 outils analysés (5 batches + addendum GSD), architecture §A.6, stack §5.5, loop canonique §5.3, ordre de build §5.6, décisions finales §A.7 |
| **Audit vérifié vision vs réalité** | `.planning/INTEGRATION-TRACKING.md` | La **base de référence** : mapping 1:1 vision → câblage réel, preuves `fichier:ligne`, tracker d'avancement (blocs A–G) |
| **Analyse projet GSD** | `.planning/PROJECT.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md` | Le **cadre projet** : valeur cœur, 76 requirements, séquence de phases, état réel corrigé (~35 % câblé) |

**Choix d'architecture retenu (validé par l'utilisateur) :** Orchestration **native dans Odysseus**
(Option 1/3). Les fichiers `.opencode/agents/*.md` deviennent une **source de vérité parsée** par un
module business-logic in-process ; **aucune dépendance au CLI OpenCode**. UI en dernier, optionnelle.

---

## Problème

L'audit runtime (voir `INTEGRATION-TRACKING.md`) a établi que la couche d'intelligence greffée sur
Odysseus est majoritairement **dormante** : ~35 % réellement câblé. Les modules sont écrits et
unit-testés (48 tests verts) mais testés en isolation, pas intégrés au loop live. Concrètement :

- Le **routing de modèles** (`llm_router.py`) existe mais a **zéro call-site live**.
- Les **agents** (`.opencode/agents/*.md`, 10 agents) sont **inactifs dans l'UI web** (CLI-only).
- La **mémoire auto-apprenante** (`acontext`) est un **stub** de 38 lignes qui dump du JSON.
- **Governance / autoeval / RRF hybride / channel gateway / observer** sont codés mais non pilotés
  par le loop.

**Objectif du Milestone 2 :** brancher réellement ces couches, en construisant une **couche métier
d'orchestration native** qui fait de la vision (§5.3, §A.6) un comportement runtime observable.

---

## Section 1 — Couche métier : `src/orchestrator/`

Nouveau package in-process qui devient le **cœur logique** de l'agent. Il ne remplace pas
l'infra de sessions existante d'Odysseus — il l'orchestre.

| Module | Responsabilité | Dépend de |
|---|---|---|
| `registry.py` | Découvre et charge les `.opencode/agents/*.md` au démarrage ; expose `get_agent(name)` | filesystem, `spec.py` |
| `spec.py` | Parse un `.md` (frontmatter YAML + corps) → `AgentSpec` (name, description, model, tools, prompt, phase) | pyyaml |
| `loop.py` | Implémente le loop canonique 7 phases (Section 2) ; pilote forced/blocked tools par phase | `phases.py`, agent_loop existant |
| `dispatcher.py` | Spawn un sous-agent via l'infra de sessions Odysseus existante, avec le modèle de sa spec | session infra, `llm_router.py` |
| `phases.py` | Définit les 7 phases, leurs outils autorisés/forcés/bloqués, et les transitions | — |

**Interface claire :** `registry` répond « quels agents existent », `spec` répond « que fait cet
agent et avec quel modèle », `loop` répond « dans quelle phase suis-je et qu'ai-je le droit de
faire », `dispatcher` répond « lance ce sous-agent ». Chaque module est testable seul.

**Point de câblage :** `llm_router.py` (aujourd'hui 0 call-site) est appelé par `dispatcher.py`
pour résoudre le modèle → premier call-site live du router.

---

## Section 2 — Loop canonique 7 phases

Simplification des 10 phases de §5.3 en 7, chacune avec des outils **forcés** et **bloqués**
(pattern phase-lock : Serena ToolMarker + vibecode RIPER-5, déjà codé mais inerte).

| # | Phase | Forcé | Bloqué | Effet réel visé |
|---|---|---|---|---|
| 1 | **CLASSIFY** | risk_classifier | write/exec | Gate destructif réel (aujourd'hui il ne fait que logger) |
| 2 | **KNOW** | recherche mémoire (RRF hybride) | write/exec | Brancher RRF (`rag_vector.py:701`) à la place du blend 0.7/0.3 |
| 3 | **PLAN** | planner agent | shell destructif | Plan explicite avant action |
| 4 | **BUILD** | tools d'édition | — | Exécution outillée normale |
| 5 | **QUALITY** | tests/lint | — | Vérification avant validation |
| 6 | **AUTOEVAL** | autoeval keep/revert | — | Piloter l'autoeval depuis le loop (aujourd'hui API-only) |
| 7 | **MEMORY+OBSERVE** | distiller + observer + trace | — | Distillation session réelle + observabilité |

La phase courante est **set** par `loop.py` → le tool_registry phase-lock (déjà codé, `phase`
jamais renseigné aujourd'hui) devient enfin actif.

---

## Section 3 — Mémoire : ChromaDB + couche Markdown/skill

On **garde ChromaDB** (fonctionne, ports isolés). On ajoute une distillation légère **in-process**,
pas de stack lourde (pas de Postgres/Redis dédié).

- `src/memory/distiller.py` : à la fin d'une session (phase 7), distille le résultat en un
  `SKILL.md` / note Markdown (pattern Acontext, jugé « indispensable » par l'analyse).
- Remplace le stub `services/acontext/app.py` (dump JSON → distillation réelle appelée par le loop).
- Couche **Obsidian** (second-brain) branchée par-dessus quand le MCP Obsidian est démarré.

---

## Section 4 — Mapping roadmap (11 phases GSD)

| Phase | Livrable | Blocs tracker (INTEGRATION-TRACKING) |
|---|---|---|
| M2-P1 | `src/orchestrator/` (registry + spec + dispatcher) | Bloc A |
| M2-P2 | Loop canonique 7 phases + phase-lock actif | Bloc B |
| M2-P3 | Router live (call-site dans dispatcher) | Bloc B |
| M2-P4 | Gate destructif réel (CLASSIFY) | Bloc A |
| M2-P5 | RRF hybride branché (KNOW) | Bloc C |
| M2-P6 | Distiller mémoire réel (MEMORY) | Bloc C |
| M2-P7 | Autoeval piloté par loop | Bloc F |
| M2-P8 | Governance ancestry créée par loop | Bloc D |
| M2-P9 | Channel gateway : adapters enregistrés | Bloc E |
| M2-P10 | Observer + trace bout-en-bout | Bloc D |
| M2-P11 | UI (optionnel, dernier) | Bloc G |

---

## Section 5 — Hors périmètre (YAGNI)

- **Supabase / Faker / seed data** : hors périmètre M2.
- **UI** : dernière et optionnelle (M2-P11) ; l'orchestration doit être observable via traces/logs
  avant toute UI.
- **Stack mémoire lourde** (Postgres/Redis dédié) : refusée, on reste in-process + ChromaDB.
- **Dépendance CLI OpenCode** : refusée, orchestration 100 % native.

---

## Critères de succès

1. Un objectif utilisateur traverse les 7 phases avec des transitions **observables** (traces).
2. `llm_router` a au moins **un call-site live** prouvé (dispatcher).
3. Un agent `.opencode/*.md` est **spawné nativement** depuis l'UI web (plus CLI-only).
4. Une action destructive est **réellement bloquée** en phase CLASSIFY (plus juste loggée).
5. Une fin de session produit une **note distillée réelle** (plus le stub JSON).
6. Le tracker d'avancement de `INTEGRATION-TRACKING.md` passe de ~35 % à la cible par bloc.
