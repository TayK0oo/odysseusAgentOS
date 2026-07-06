# Permission Matrix — AgentOS Odysseus

> Matrice outils × phases. Niveaux de risque : READ < DRAFT < WRITE < EXEC < DESTRUCTIVE.

## Niveaux de risque

| Niveau | Description | Exemples d'outils |
|---|---|---|
| **READ** | Lecture seule, pas d'effet de bord | `read_file`, `grep`, `glob`, `ls`, `list_models`, `list_sessions` |
| **DRAFT** | Brouillon, modifiable | `suggest_document` |
| **WRITE** | Écriture sur disque | `write_file`, `edit_file`, `create_document`, `update_document`, `manage_memory` |
| **EXEC** | Exécution de code | `bash`, `python`, `api_call`, `web_search`, `trigger_research` |
| **DESTRUCTIVE** | Suppression irréversible | `remove_dir`, `stop_served_model`, `cancel_download`, `manage_documents` |

## Matrice par phase

| Outil | Risque | CLASSIFY | KNOW | PLAN | BUILD | QUALITY | AUTOEVAL | MEM_OBS |
|---|---|---|---|---|---|---|---|---|
| `read_file` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `grep` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `glob` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `ls` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `get_workspace` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `list_models` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `list_sessions` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `web_search` | EXEC | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `write_file` | WRITE | ❌ | ❌ | ⚠️ | ✅ | ❌ | ✅ | ✅ |
| `edit_file` | WRITE | ❌ | ❌ | ⚠️ | ✅ | ❌ | ✅ | ✅ |
| `create_document` | WRITE | ❌ | ❌ | ⚠️ | ✅ | ❌ | ✅ | ✅ |
| `bash` | EXEC | ❌ | ❌ | ❌ | ✅ | ⚠️ | ✅ | ✅ |
| `python` | EXEC | ❌ | ❌ | ❌ | ✅ | ⚠️ | ✅ | ✅ |
| `api_call` | EXEC | ❌ | ❌ | ❌ | ✅ | ⚠️ | ✅ | ✅ |
| `remove_dir` | DESTRUCTIVE | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ |

**Légende** : ✅ Autorisé | ❌ Bloqué | ⚠️ Restreint (voir `phase-lock.yaml`)

## Gates

| Gate | Portée | Kill-switch | Défaut |
|---|---|---|---|
| Destructive gate | Bloque `rm -rf`, `mkfs`, `dd`, fork bomb sur bash/python | `ODYSSEUS_DESTRUCTIVE_GATE` | ON |
| Phase-lock | Bloque outils hors phase courante | `ODYSSEUS_PHASE_TRACKER` | OFF |
| Admin gate | Bloque outils admin pour non-admins | N/A | ON |
| Tool policy gate | Bloque outils selon politique guide-only | N/A | ON |

## Références

- `src/risk_classifier.py:27-78` — TOOL_RISK_MAP
- `config/phase-lock.yaml` — Configuration YAML
- `src/orchestrator/gate.py:38-52` — Destructive gate
- `src/tool_execution.py:587-608` — Phase-lock enforcement
- `src/tool_execution.py:553-573` — Gate enforcement
