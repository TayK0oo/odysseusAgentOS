# AUDIT — 02 PLAN NETTOYAGE (Table rase contrôlée)

> **Statut :** En attente de validation utilisateur. **Aucune action destructive exécutée.**
> **Condition :** GO explicite requis avant toute suppression.
> **Worktree :** Tout travail se fera dans `audit/table-rase`, jamais sur `dev`.
> **Règle :** Smoke test de boot après CHAQUE commit destructif.
> **Référence :** Basé sur le mapping complet `01-MAPPING-COMPLET.md` (10 zones, ~25K lignes d'audit).

---

## 0. Préambule — Ce qui N'EST PAS à nettoyer

Avant de lister les cibles, clarifions ce qui est **intentionnel et conservé** :

| Élément | Justification | Source |
|---|---|---|
| `model-routing.json` | Politique de routing UNIQUE (modèles/stages/heuristic/fallback). Seul `providers.*` partiellement redondant. | STATE.md:30,84 |
| `PhaseTracker` + kill-switch OFF | Pont M3.0, design "Option C" convergence incrémentale | STATE.md:95 |
| `CanonicalLoop` non branché au live | Testé (38 tests verts), prêt pour orchestration future | dispatcher.py |
| 2 listes de patterns (`gate.py` + `risk_classifier.py`) | Politiques distinctes (bash shell vs tool classification), duplication intentionnelle | STATE.md:91 |
| `memory.py` / `memory_vector.py` wrappers (5 lignes) | Compatibilité d'import, 129+ call-sites | services/memory/ |
| 3 phases YAML orphelines (RESEARCH, INNOVATE, VERIFY) | Features gaps documentés, pas de code mort | phase-lock.yaml:5-68 |
| Adaptateurs Discord/Telegram dormants | Kill-switchés, prêts à l'activation | channel_bootstrap.py |

---

## 1. Classification — Toutes les cibles

### 🔴 SUPPRIMER — Code mort confirmé, aucun appelant

| # | Cible | Zone | Justification | Preuve |
|---|---|---|---|---|
| S1 | `services/faces/__init__.py` | A | Stub vide (1 docstring), aucun appelant, jamais importé | `grep faces` → 0 hit hors du fichier |
| S2 | `src/adapters/__init__.py` | D | Placeholder vide (1 commentaire), les vrais adapters sont en lazy-import | `grep "from src.adapters"` → 0 hit |
| S3 | Entrées fantômes dans `TOOL_RISK_MAP` : `create_file`, `list_dir`, `search_files`, `get_symbol`, `pipeline`, `edit_image` | E | 6 outils inexistants ou renommés. `create_file` jamais dispatché, `list_dir` → `ls`, `search_files` → `grep`/`glob` | `risk_classifier.py:30,32,35,50,68,71` |
| S4 | Outils fantômes dans `phase-lock.yaml` : `run_command`, `create_file`, `delete_file` | F | Jamais dispatchés. `run_command` = alias fantôme supprimé en M3.4. `create_file`/`delete_file` = jamais définis. | `phase-lock.yaml:14,15,40,67,80,93` |
| S5 | `_AGENT_PREAMBLE` 1ère définition | A | Lignes 62-66 écrasées par lignes 175-177. ~50 lignes de règles détaillées perdues (conservées dans `_DOMAIN_RULES` l.215-274) | `agent_loop.py:62-66` vs `175-177` |
| S6 | `_AGENT_RULES` 1ère définition | A | Lignes 68-111 écrasées par 179-200. Règles longues mortes. | `agent_loop.py:68-111` vs `179-200` |
| S7 | Supabase MCP — code client + config serveur | F, H | Service docker absent. `supabase_mcp.py` (64 lignes) + `mcp_manager.py:40-46` → `localhost:8900` ne répondra jamais. Kill-switch `SUPABASE_MCP_ENABLED=false` masque le problème. | `docker-compose.yml` grep supabase = 0 |

### 🟡 ARCHIVER — Docs obsolètes ou aspirational

| # | Cible | Zone | Destination | Justification |
|---|---|---|---|---|
| A1 | `.planning/INTEGRATION-TRACKING.md` | J | `.planning/archive/INTEGRATION-TRACKING-2026-07-01.md` | Dernière MAJ 2026-07-01. Contredit STATE.md sur command_validator (1 chemin vs 3). Contenu migré dans STATE+ROADMAP-M3. |
| A2 | `.planning/REQUIREMENTS.md` | J | `.planning/archive/REQUIREMENTS-v1-2026-06-27.md` | 76 requirements tous "Pending" depuis 06-27. Jamais mis à jour. Tracking réel dans STATE+ROADMAP-M3. |
| A3 | `.planning/pipeline-lists/MOVE.md` | J | `.planning/archive/pipeline-lists/MOVE-2026-06.md` | 2/5 items réfèrent à composants supprimés (stage-model-assignment.yaml, llm_router.py). |
| A4 | `.planning/pipeline-lists/WAIT.md` | J | `.planning/archive/pipeline-lists/WAIT-2026-06.md` | W-002 mentionne LiteLLM/OpenRouter — ModelRouter supprimé en M3.1. |
| A5 | `docs/memory-pipeline.md` | J | `docs/archive/memory-pipeline-aspirational.md` | Décrit pipeline complet Experience→Skill Agent. Réalité : `services/acontext/app.py` = 38 lignes stub. |
| A6 | `docs/ANALYSE-OUTILS-UNIFIE.md` | J | `docs/archive/ANALYSE-OUTILS-UNIFIE-ConfigOpenCodeNew.md` | Doublon de `.planning/AGENT-OS-ANALYSE-PROFONDE.md`. Conclusions divergentes (Graphify score 9/10). Provient de ConfigOpenCodeNew, pas de ce repo. |
| A7 | `.planning/PROJECT.md` | J | `.planning/archive/PROJECT-2026-06-27.md` | 22 items unchecked, toutes décisions "Pending". Obsolète depuis verdicts redondance M3. |

### 🟢 GARDER — Vivant, intentionnel ou référence

| # | Cible | Justification |
|---|---|---|
| G1 | `services/memory/memory.py` (5 lignes) | Compat import. 129+ call-sites. |
| G2 | `services/memory/memory_vector.py` (5 lignes) | Compat import. |
| G3 | `model-routing.json` | Politique routing UNIQUE (hors `providers.*`). |
| G4 | Tous les kill-switches M3 (19) | Architecture défaut-OFF vérifiée. |
| G5 | `.planning/intel/INDEX.md` + 7 `domains/*.md` | Cartographie native fiable, post-M3. |
| G6 | `ROADMAP-M3-ORCHESTRATION.md` | Meilleure source unique état M3. |
| G7 | `STATE.md` (après correction ligne 140) | Document canonique état réel. |
| G8 | `docs/audit/00-BASELINE.md` | Document fondateur audit. |
| G9 | 3 phases YAML orphelines (RESEARCH, INNOVATE, VERIFY) | Features gaps documentés. |
| G10 | Adaptateurs Discord/Telegram dormants | Prêts à l'activation. |

### 🔵 FUSIONNER — Doublons à consolider

| # | Cibles | Zone | Action | Justification |
|---|---|---|---|---|
| F1 | `.planning/ROADMAP.md` (357L) + `ROADMAP.md` racine (80L) | J | Renommer `.planning/ROADMAP.md` → `ROADMAP-15-PHASES.md` | Même nom, audiences différentes (interne vs communautaire). |
| F2 | `docs/ANALYSE-OUTILS-UNIFIE.md` + `.planning/AGENT-OS-ANALYSE-PROFONDE.md` | J | Garder UN seul document canonique d'analyse outils | Doublon sévère — 2 analyses des 60 mêmes outils, conclusions divergentes. Résolution : archiver UNIFIE (ConfigOpenCodeNew), garder PROFONDE comme base, enrichir avec décisions M3. |
| F3 | `BudgetEnforcer.consume_tool_call()` + `auto_pause_if_exceeded()` | E | Supprimer les méthodes mortes OU les brancher | Jamais appelées. Si feature gap → créer; sinon → nettoyer. |

### 🟣 CRÉER — Documents manquants

| # | Cible | Zone | Contenu attendu | Priorité |
|---|---|---|---|---|
| C1 | `loop-canonique.md` (racine) | B | Document standalone décrivant la boucle canonique 7 phases, extrait de `phases.py` + `loop.py` docstrings | **P1** |
| C2 | `permission-matrix.md` (racine) | B | Matrice de permissions formelle : outils par phase, niveaux de risque, gates | **P1** |
| C3 | `PROJECT.yaml` (racine) | J | Configuration budget + manifeste projet. Référencé par `BudgetEnforcer` mais jamais créé. | **P2** |
| C4 | `docs/archive/README.md` | J | Index des documents archivés avec date d'archivage et raison | **P2** |

---

## 2. Corrections prioritaires (bugs, pas nettoyage)

Ces items sont des **bugs confirmés**, pas du nettoyage. Traitement prioritaire avant toute suppression.

| # | Bug | Zone | Sévérité | Correctif |
|---|---|---|---|---|
| B1 | `STATE.md:140` — "101 commits ahead" (faux, 0 ahead) | J | **CRITIQUE** | Remplacer par "synchronisé avec origin/dev" |
| B2 | `obsidian_mcp.py` pas MCP stdio (Flask HTTP) | C, F | **CRITIQUE** | Réécrire avec `mcp.server.stdio` ou documenter l'incompatibilité et supprimer de `builtin_mcp.py` |
| B3 | Routes Obsidian `/api/knowledge/memory/*` → `localhost:9751` fantôme | C | **HAUTE** | Supprimer ces routes ou les rediriger vers appels directs `obsidian_mcp.py` |
| B4 | Traces `cost_tokens` toujours 0 | E | **MOYENNE** | Passer `cost_tokens` dans `tool_execution.py:630-638` |
| B5 | `Observer` instancié neuf par run (pas mémoire inter-run) | E | **HAUTE** | Rendre `_codeburn_reports` persistant ou documenter comme limitation intentionnelle |
| B6 | Standalone GPU files désynchronisés | H | **HAUTE** | Regénérer depuis `docker-compose.yml` + overlays, ou supprimer et documenter workflow `COMPOSE_FILE` |

---

## 3. Plan d'exécution (après GO)

### Phase 2a — Corrections de bugs (B1-B6)
- Worktree `audit/table-rase`
- 1 commit par bug
- Smoke test `python -c "import app"` après chaque commit
- Push sur branche `audit/table-rase`

### Phase 2b — Suppressions (S1-S7)
- 1 commit par suppression
- Vérification `grep` avant/après pour confirmer 0 appelant résiduel
- Smoke test après chaque commit

### Phase 2c — Archivage docs (A1-A7)
- `git mv` vers `docs/archive/` (préserve l'historique)
- Créer `docs/archive/README.md` (C4)
- 1 commit groupé pour tous les archivages

### Phase 2d — Création documents (C1-C3)
- Écrire `loop-canonique.md`, `permission-matrix.md`, `PROJECT.yaml.example`
- 1 commit par document

### Phase 2e — Fusions/renommages (F1-F3)
- Renommer `.planning/ROADMAP.md`
- Nettoyer méthodes mortes `BudgetEnforcer`
- 1 commit par action

---

## 4. Smoke test de boot — Protocole

Après CHAQUE commit destructif, exécuter :

```bash
# Test 1 : import app (subprocess isolé)
python -c "import sys; sys.path.insert(0, '.'); import app"

# Test 2 : smoke test pytest
pytest tests/test_app.py::TestAppBoots -xvs

# Test 3 : vérification routes (pas de mount mort)
python -c "
from app import app
routes = [r.path for r in app.routes]
print(f'{len(routes)} routes enregistrées')
"
```

Si échec → `git revert` immédiat, diagnostic, correctif, re-test.

---

## 5. Récapitulatif — Impact estimé

| Catégorie | Nombre | Impact |
|---|---|---|
| **SUPPRIMER** | 7 | ~150 lignes de code mort, ~10 entrées config |
| **ARCHIVER** | 7 | ~1200 lignes de docs obsolètes → `archive/` |
| **GARDER** | 12 | Intentionnel, documenté |
| **FUSIONNER** | 3 | 2 doublons docs + nettoyage méthodes |
| **CRÉER** | 4 | 3 docs + 1 index archive |
| **BUGS** | 6 | 2 critiques, 3 hautes, 1 moyenne |

**TOTAL : 39 actions**, dont 6 bugs à fixer avant tout nettoyage.

---

## ⚠️ ATTENTION — GO REQUIS

**Ce plan n'exécute rien.** Toute action destructive est bloquée jusqu'à validation explicite de l'utilisateur.

Pour donner le GO, répondre avec :
- `GO` pour tout exécuter
- `GO sauf X` pour exclure des items
- `GO bugs seulement` pour ne traiter que B1-B6
- `STOP` pour abandonner

— Fin Phase 2 / 02-PLAN-NETTOYAGE.
