# Boucle Canonique — AgentOS Odysseus

> Extrait de `src/orchestrator/phases.py` et `src/orchestrator/loop.py`. Document de référence.

## Les 7 Phases

```
CLASSIFY → KNOW → PLAN → BUILD → QUALITY → AUTOEVAL → MEMORY_OBSERVE
```

| # | Phase | Description | Outils forcés | Catégories bloquées |
|---|---|---|---|---|
| 1 | **CLASSIFY** | Classification du risque — lecture seule | Aucun | write, exec, destructive |
| 2 | **KNOW** | Récupération mémoire/connaissance | Aucun | write, exec, destructive |
| 3 | **PLAN** | Planification — écriture limitée à process/ + .planning/ | Aucun | exec, destructive |
| 4 | **BUILD** | Implémentation — accès complet | Aucun | Aucun |
| 5 | **QUALITY** | Tests et lint uniquement | Aucun | write, destructive |
| 6 | **AUTOEVAL** | Auto-évaluation — pas de destruction | Aucun | destructive |
| 7 | **MEMORY_OBSERVE** | Distillation mémoire et observabilité | Aucun | destructive |

## Kill-switches associés

| Kill-switch | Défaut | Phase | Effet |
|---|---|---|---|
| `ODYSSEUS_PHASE_TRACKER` | `off` | Toutes | Active le tracking de phase par round dans le loop live |
| `ODYSSEUS_LIVE_ORCHESTRATION` | `off` | Toutes | Active `CanonicalLoop` complet (7 phases) au lieu de `PhaseTracker` simplifié |
| `ODYSSEUS_AUTOEVAL` | `off` | AUTOEVAL | Active keep/revert via `git reset --hard` |
| `ODYSSEUS_CHECKPOINT` | `off` | MEMORY_OBSERVE | Persiste checkpoint Obsidian post-round |
| `ODYSSEUS_GOVERNANCE_ANCESTRY` | `off` | MEMORY_OBSERVE | Écrit GoalTask avec ancestry |

## Extension

Pour ajouter une phase :
1. Ajouter le membre dans `Phase` enum (`src/orchestrator/phases.py`)
2. Ajouter l'entrée YAML dans `config/phase-lock.yaml`
3. Ajouter dans `CANONICAL_SEQUENCE` (`src/orchestrator/loop.py`)

## Références

- `src/orchestrator/phases.py:14-33` — Enum + séquence canonique
- `src/orchestrator/loop.py:20-48` — CanonicalLoop
- `config/phase-lock.yaml` — Configuration des outils par phase
- `src/orchestrator/phase_tracker.py:29-64` — PhaseTracker (pont loop live)
