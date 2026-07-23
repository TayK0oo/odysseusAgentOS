# AgentOS — Mapping Complet: Projet → Phases → Agents → Outils → Sorties

## Vue d'ensemble

```
NOUVEAU PROJET (langage naturel)
    │
    ▼
┌──────────────────────────────────────────────────────────────────┐
│ 1. CLASSIFY          Évalue le risque, décide mono/multi-agent   │
│    Agents: constitution, mode_detector                            │
│    Outils: risk_classifier, multi_agent_decision                  │
│    Sortie: risk_level (low/medium/high/critical), agent_mode      │
├──────────────────────────────────────────────────────────────────┤
│ 2. KNOW              Recherche mémoire, conversations, skills     │
│    Agents: — (pas d'agent, lookup direct)                         │
│    Outils: memory_provenance, conversation_search, skills         │
│    Sortie: [memories], [past_conversations], skills disponibles   │
├──────────────────────────────────────────────────────────────────┤
│ 3. PLAN              Décompose en objectifs → tâches             │
│    Agents: gsd-planner, gsd-researcher                            │
│    Outils: planning_engine, context_manager                       │
│    Sortie: plan YAML {objectifs: [{id, description, tasks}]}     │
├──────────────────────────────────────────────────────────────────┤
│ 4. BUILD             Exécute avec LLM + outils + sous-agents     │
│    Agents: gsd-executor, open-coder, open-design                  │
│    Outils: BASH, WRITE_FILE, GET_WORKSPACE, MANAGE_MCP           │
│          + tous les outils MCP connectés                          │
│    Sortie: code, fichiers, artefacts                              │
├──────────────────────────────────────────────────────────────────┤
│ 5. QUALITY           Vérifie, teste, audite                      │
│    Agents: debate-5-personas, security-audit, reviewer            │
│    Outils: test_runner, linter, py_compile, OWASP check          │
│    Sortie: {tests: pass/fail, lint: pass/fail, security: ok/warn} │
├──────────────────────────────────────────────────────────────────┤
│ 6. AUTOEVAL          Auto-évalue vs critères de succès           │
│    Agents: gsd-verifier, edge-case-gen, test-engineer             │
│    Outils: autoeval (keep/revert), codeburn_runner                │
│    Sortie: score (0-100%), verdict (PASS/NEEDS WORK)             │
├──────────────────────────────────────────────────────────────────┤
│ 7. MEMORY_OBSERVE    Apprend, écrit en mémoire avec provenance   │
│    Agents: gsd-roadmapper, context-agent                          │
│    Outils: memory_provenance ([stated]/[observed]), skills        │
│    Sortie: leçons apprises, faits tagués, skills mis à jour       │
└──────────────────────────────────────────────────────────────────┘
    │
    ▼
PROJET TERMINÉ — résultat + mémoire + cockpit mis à jour
```

## Mapping Agents → Phases

| Agent | Phases | Rôle |
|-------|--------|------|
| constitution | CLASSIFY | Vérifie les invariants SFD |
| gsd-planner | PLAN | Décompose en objectifs→tâches |
| gsd-researcher | PLAN | Recherche contexte/solutions |
| gsd-executor | BUILD | Exécute les tâches |
| open-coder | BUILD | Génère du code |
| open-design | BUILD | Génère UI/design |
| debate-5-personas | QUALITY | Débat de la qualité |
| security-audit | QUALITY | Audit sécurité (STRIDE+OWASP) |
| reviewer | QUALITY | Code review |
| gsd-verifier | AUTOEVAL | Vérifie les critères |
| edge-case-gen | AUTOEVAL | Génère cas limites |
| test-engineer | AUTOEVAL | Exécute tests |
| gsd-roadmapper | MEMORY_OBSERVE | Met à jour la roadmap |
| context-agent | MEMORY_OBSERVE | Consolide le contexte |

## Mapping Outils → Phases

| Outil | Phase | Rôle |
|-------|-------|------|
| mode_detector | CLASSIFY | Détecte agent vs chat |
| risk_classifier | CLASSIFY | Évalue risque (5 niveaux) |
| multi_agent_decision | CLASSIFY | Décide mono vs multi-agent |
| memory_provenance | KNOW, MEMORY | [stated]/[observed]/[inferred] |
| conversation_search | KNOW | Cherche conversations passées |
| linguistic_signals | KNOW | Détecte références au passé |
| skills (SKILL.md) | KNOW, BUILD | Chargement obligatoire |
| planning_engine | PLAN | Mission→Objectifs→Tâches |
| context_manager | PLAN | Compaction, bloc-notes |
| BASH | BUILD | Exécution shell |
| WRITE_FILE | BUILD | Création fichiers |
| GET_WORKSPACE | BUILD | Structure projet |
| MANAGE_SKILLS | BUILD | Gestion skills |
| Kroki | BUILD | Diagrammes (Mermaid) |
| MANAGE_MCP | BUILD | Connecteurs services |
| test_runner | QUALITY | Tests unitaires |
| linter | QUALITY | Qualité code |
| autoeval | AUTOEVAL | Keep/revert décision |
| codeburn_runner | AUTOEVAL | Analyse one-shot rate |
| trace_writer | ALL | Traces JSONL |
| budget_enforcer | ALL | Budget tokens/coût |
| InjectionGuard | ALL | Protection prompt injection |

## Kill-switches par phase

| Phase | Switch requis (ON) |
|-------|-------------------|
| CLASSIFY | ODYSSEUS_THOUGHT_BUS, ODYSSEUS_DESTRUCTIVE_GATE |
| KNOW | ODYSSEUS_MEMORY_PROVENANCE, ODYSSEUS_MEILISEARCH |
| PLAN | ODYSSEUS_PLANNING_ENGINE |
| BUILD | ODYSSEUS_DURABLE_EXEC, ODYSSEUS_TOOL_DISCOVERY |
| QUALITY | ODYSSEUS_AUTOEVAL, ODYSSEUS_CONTENT_SECURITY |
| AUTOEVAL | ODYSSEUS_AUTOEVAL, ODYSSEUS_CODEBURN |
| MEMORY_OBSERVE | ODYSSEUS_MEMORY_PROVENANCE, ODYSSEUS_GOVERNANCE_ANCESTRY |

## Workflow type: Nouveau projet

```
User: "build a backup system"
  → mode_detector: AGENT
  → CLASSIFY: risk=medium, mode=SINGLE
  → KNOW: found 3 memories, 7 skills
  → PLAN: 3 objectives, 9 tasks, ~2000-5000 tokens
  → BUILD: gsd-executor + BASH + WRITE_FILE → code produit
  → QUALITY: lint=pass, tests=pending
  → AUTOEVAL: score=90%, verdict=PASS
  → MEMORY: 3 lessons stored [stated]
  → Cockpit: C●K●P●B●Q●A●M● complete
```
