# Agent OS — Objectifs & Couverture (score mesuré)

> **Fusion de :** objectifs du projet (`PROJECT-2026-06-27.md`) + matrice de couverture.
> **MAJ :** 2026-09-25 — **source unique de couverture** : le score est **recalculé depuis la traçabilité vérifiée** (`../traceability/`), en remplacement des scores contradictoires antérieurs (59 % / 83 % / 90 %).

---

## Core Value

**Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, apprend de chaque run, et ne dépasse jamais ses limites — sans intervention humaine constante.**

Workspace AI self-hosté complet bâti sur OpenCode (fork Odysseus), transformé en système d'agents autonomes de niveau production. Architecture déterministe multi-couches (Connaissance → Exécution → Automatisation → Gouvernance) capable de faire tourner des agents longue durée, bornés par des budgets et des objectifs, avec une mémoire d'apprentissage continue.

---

## Architecture — 5 Couches

```
CONNAISSANCE (Trinité)    → CBM + Graphify + Obsidian
EXÉCUTION                 → Agents, MCP, Sandbox, Routing
DESIGN                    → Extract + Open Design
OBSERVABILITÉ             → Traces, CodeBurn, AutoEval
GOUVERNANCE               → Budgets, Goal-Ancestry, Heartbeat
```

---

## Objectifs — 8 Axes

| Axe | Objectif |
|---|---|
| **1 — Orchestration** | Lancer des plans entiers avec subagents, déléguer et spécialiser les tâches au bon agent avec les bons outils. |
| **2 — Modularité** | Chaque brique interchangeable sauf le cœur ; ajouter/retirer un service, un agent, un outil sans redémarrage. |
| **3 — Routage** | Sélectionner automatiquement le bon modèle par tâche/phase/agent, avec repli. |
| **4 — Connaissance (Trinité)** | Interroger CBM (code) + Graphify (sémantique) + Obsidian (mémoire) avant de répondre. |
| **5 — Sandbox** | Toute exécution de code isolée, validée, tracée. |
| **6 — Gouvernance** | Budgets granulaires, goal-ancestry, approval gates, heartbeat. |
| **7 — Apprentissage** | Distiller les runs en compétences, mémoire persistante à provenance. |
| **8 — UI Cockpit** | Tableau de bord live : phase, santé, dérive, budget, agents. |

---

## Score de couverture MESURÉ (source unique)

**Méthode** (reprise de la traçabilité, pondération « codé ») :

- `Couverture (codé)` = (ACTIF × 1 + PARTIEL × 0.5 + DORMANT × 0.5) ÷ total — mesure *ce qui est écrit dans le code*, dormant inclus.
- `Activation (live)` = ACTIF ÷ total — mesure *ce qui tourne par défaut*.

Statuts issus de `../traceability/02-PRINCIPES-UC.md` (vérifiés sur le code).

| Ensemble | ACTIF | PARTIEL | DORMANT | ABSENT | Total | Couverture (codé) | Activation (live) |
|---|---:|---:|---:|---:|---:|---:|---:|
| **Principes P1-P22** | 6 | 12 | 4 | 0 | 22 | **64 %** | **27 %** |
| **Cas d'usage UC-01→19** | 6 | 10 | 1 | 2 | 19 | **61 %** | **32 %** |
| **TOTAL (41 exigences)** | 12 | 22 | 5 | 2 | 41 | **≈ 62 %** | **≈ 29 %** |

> **SFD v3.1** ajoute P23-P25 et UC-20-21 (proactivité, grounding, data flywheel, brief planifié, vérifier la source). Non encore vérifiés côté code (spec only) → à intégrer au score lors de leur implémentation.

**Lecture :** le système est **codé à ~62 %** de ses exigences (la majorité du reste étant *câblé mais dormant* via kill-switch OFF), mais **~29 % seulement sont actives par défaut**. Détail par module/axe : `../traceability/TRACEABILITY.md` et `01-SFD-TRACEABILITY.md`.

---

## Exigences fondatrices (historique)

- **Validées** (12) : FastAPI, LLM routing + fallback, MCP servers, Deep Research, Task scheduler, Context compactor, CalDAV/Email, PWA UI, Cookbook, Auth, Claude/Codex integrations, ChromaDB RAG.
- **Actives** (28) : loop canonique + invariants, phase-lock, Serena/Scrapling MCP, observations JSONL, CodeBurn, GSD, PROJECT.yaml + autoeval, Acontext, Governance, Channel Gateway, Trinité UI, Decision Engine, Sandbox durci, Design Extract/Open Design, Debate 5 personas, RRF hybrid search…
- **Planifiées** (7) : Trinité branchée UI, Graphify pipeline, Obsidian second-brain, Heartbeat, budgets granulaires live, goal-ancestry live, multi-agent live.

> ⚠️ Ces listes proviennent de l'ère v3.0 et **recoupent** les statuts ci-dessus ; en cas de conflit, **le score mesuré et `../traceability/` font foi**.

---

## Séquence de Build (Constitution)

```
loop manuel → tools → permissions → observations → budgets → tracing
→ planning → context/memory → compaction → skills/connectors
→ goal loop → subagents
```

Ne jamais sauter d'étape.
