# AgentOS — TRACEABILITY (document maître)

> **Objet :** combler la lacune de traçabilité. Répondre à : *qu'est-ce qui est fait, fonctionnel, prévu, et comment c'est fait ?*
> **Méthode :** analyse statique du code réel (branche `feat/inventaire-global-v1`), vérification d'existence, croisement avec les specs `docs/master-ref/`.
> **Date :** 2026-09-25 · **Dernier commit de travail :** 2026-08-04 (`e077b3f`) · **Backup :** 2026-09-22.
> **Sous-documents :** `00-CODE-INVENTORY.md`, `01-SFD-TRACEABILITY.md`, `02-PRINCIPES-UC.md`, `03-MODULARITE.md`, `04-TESTS-REALITE.md`, `../veille/VEILLE-FEATURES.md`.

---

## TL;DR — le verdict

1. **Le projet est bien plus large que ce que la doc prétend.** `03/07` annoncent « Odysseus élagué → ~15 routes / ~50 fichiers » ; la réalité est **63 fichiers de routes, 503 endpoints, 1 071 fichiers Python actifs**. La « réduction » documentée n'a pas eu lieu.
2. **Tout est codé, presque rien n'est actif.** Les 20 modules de la SFD v3.0 ont **tous** une implémentation présente. Mais **2 kill-switches seulement sont ON par défaut** sur 35 → la majorité est **câblée mais dormante**.
3. **Les scores des docs sont non fiables.** `04`=59 %, `09`=83 %, `08`=90 % — tous non traçables, et démentis par la vérification : **6/22 principes ACTIFS**, **6/19 UC ACTIFS**.
4. **Des bugs de nommage neutralisent des features « terminées »** : 6 variables d'env écrites dans `.env` ne sont **jamais lues** par le code (noms différents) → durable, provenance, visuel sont OFF malgré `on` dans `.env`.
5. **La modularité est réelle mais inégale** : bons points (packages npm, MCP, skills, `MemoryProvider`), mauvais points (`core.database` importé par 64 modules, `route_loader` non dynamique, shims vers `archive/legacy/`).
6. **Le projet ne tourne pas localement** : aucun venv/dépendances installés (post-réinstallation). C'est environnemental, pas un défaut du code (0 erreur de syntaxe).

---

## 1. Inventaire du code (réel)

| Élément | Volume |
|---|---|
| Fichiers Python actifs (hors `archive/`) | **1 071** |
| Fichiers legacy (`archive/legacy/`) | 2 (`agent_loop.py` 4 418 l., `llm_core.py`) |
| Fichiers de routes | **63** |
| Endpoints HTTP | **503** |
| Modules `src/` | **136** |
| Services (`services/`) | 17 |
| Serveurs MCP (`mcp_servers/`) | 7 |
| Packages npm (`@agentos/sfd-*`) | 10 |
| Agents `.opencode/agents/` | 20 |
| Skills (`skills/` + `.opencode/skills/`) | 12 + 3 |
| Fichiers de test / fonctions `test_*` | 693 / ~4 151 |
| Kill-switches curatés | 35 (**2 ON**, 33 OFF) |

Détail complet : `00-CODE-INVENTORY.md`.

---

## 2. Traçabilité SFD v3.0 ↔ code

### 2.1 Modules fonctionnels (§5.1 → §5.20)
**Tous les 20 modules ont du code présent** (vérifié par existence). Le statut réel dépend du kill-switch : la plupart sont **dormants** (ex. orchestration live, model router, mémoire/Obsidian, canaux, gouvernance, autoeval, LangFuse, CBM/Graphify/Serena, préférences, visuel, classification). Détail : `01-SFD-TRACEABILITY.md`.

### 2.2 Principes P1→P22 — vérifiés
| Statut | Nb | IDs |
|---|---:|---|
| ACTIF | **6** | P3, P7, P9, P10, P11, P15 |
| PARTIEL | **12** | P1, P2, P4, P6, P8, P12, P13, P17, P18, P20, P21, P22 |
| DORMANT | **4** | P5, P14, P16, P19 |
| ABSENT | 0 | — |

### 2.3 Use cases UC-01→UC-19 — vérifiés
| Statut | Nb | IDs |
|---|---:|---|
| ACTIF | **6** | UC-01, UC-03, UC-04, UC-05, UC-08, UC-13 |
| PARTIEL | **10** | UC-02, UC-09, UC-10, UC-12, UC-14, UC-15, UC-16, UC-17, UC-18, UC-19 |
| DORMANT | **1** | UC-11 |
| **ABSENT** | **2** | **UC-06 (fork worktree), UC-07 (merge worktree)** |

> **Total exigences SFD : 41 → 12 ACTIVES, 22 PARTIELLES, 5 DORMANTES, 2 ABSENTES.**

Détail (fichiers + preuves + divergences avec `08`) : `02-PRINCIPES-UC.md`.

---

## 3. Kill-switches — le levier « activable »

Registre vérifiable : `src/killswitch_registry.py` (name/env/default/source). **2 ON par défaut :**

- `ODYSSEUS_DESTRUCTIVE_GATE` (sécurité) — `src/orchestrator/gate.py:26`
- `ODYSSEUS_LANGGRAPH_INTERRUPT` — `src/orchestrator/langgraph_loop.py:441`

**33 OFF par défaut** = features câblées mais inactives (orchestration live, phase tracker, model router, autoeval/autoevolve/codeburn, checkpoint, langfuse, CBM/Graphify/Obsidian/Serena/Playwright/Supabase, canaux Discord/Telegram, RRF, progressive disclosure, memory impact, unified tokens…).

> Décision structurante à prendre : **rester « off par défaut » (prudent)** ou **passer en « on par défaut » par paliers** pour atteindre la vision « système holistique complet ». Voir §7.

---

## 4. Modularité — forces et couplages

### 4.1 Les 5 couplages les plus critiques (à casser en priorité)
1. **`core.database` — 64 importateurs.** Aucune abstraction de persistance.
2. **`core/route_loader.py` — 58 imports codés en dur.** Malgré son nom, **pas dynamique** : ajouter une route exige d'éditer un fichier central.
3. **Inversion de dépendance `src/* → routes.*` (~30 arêtes).** Le métier dépend de la couche HTTP (imports paresseux).
4. **Shims `src/llm_core.py` / `src/agent_loop.py` → `archive/legacy/`.** 71 fichiers dépendent d'un moteur hors arborescence masqué par un `re-export *` → principal obstacle à un cœur LLM remplaçable.
5. **Routage modèle à 3 sources de vérité** (`model-routing.json`, `ModelEndpoint` DB, `opencode.json`) + mappings en dur.

### 4.2 Les 5 briques les plus remplaçables (à généraliser)
1. **`src/memory_provider.py`** (ABC + registre) — **le modèle du projet**.
2. **Skills** (`SKILL.md` découverts par glob).
3. **Serveurs MCP** (frontière processus + protocole standard).
4. **Packages npm `@agentos/sfd-*`** (isolation TS ; 9/10 non branchés = dette d'intégration, pas de couplage).
5. **Kill-switch registry** (module pur).

11 recommandations détaillées : `03-MODULARITE.md` §6.

---

## 5. Réalité fonctionnelle (exécution)

- **Environnement local : non opérationnel.** Pas de `.venv` ; `python3` = 3.14.7 ; `fastapi/sqlalchemy/redis/pydantic` absents ; `ruff`/`pytest` absents → **0 test exécutable** en l'état.
- **Le code, lui, est sain** : **0 erreur de syntaxe** sur l'ensemble (analyse AST) ; 4 151 fonctions de test détectées.
- **CI** : `ci.yml` = Python 3.11 (pytest `continue-on-error`), `e2e.yml` = 3.14 + ruff. Local ≠ CI → échec **environnemental**.
- **Pour rendre exécutable** (à valider, non exécuté) : `python3.12 -m venv venv && pip install -r requirements.txt && pip install ruff pytest`.

Détail : `04-TESTS-REALITE.md`.

---

## 6. Divergences doc ↔ code (à corriger dans les docs)

| Sujet | Doc | Réalité |
|---|---|---|
| Taille projet | `03/07` : ~15 routes, ~50 fichiers | 63 routes, 503 endpoints, 1 071 `.py` |
| Couverture | `04`=59 %, `09`=83 %, `08`=90 % | 6/22 principes + 6/19 UC ACTIFS |
| UC-06/07 | « codés » | **absents** (aucun `git worktree`) |
| P18 | 08 : « if_version dans memory_writer » | faux ; le vrai `if_version` est dans `provenance_memory.py` (dormant) |
| Env | `.env` met `on` | 6 variables **jamais lues** (noms ≠ code) |

### 6.1 Bugs de nommage des kill-switches (à corriger, gains immédiats)
| `.env` écrit | Code lit | Effet |
|---|---|---|
| `ODYSSEUS_DURABLE_EXEC` | `ODYSSEUS_DURABLE_EXECUTION` | durable OFF |
| `ODYSSEUS_MEMORY_PROVENANCE` | `ODYSSEUS_PROVENANCE_MEMORY` | provenance OFF |
| `ODYSSEUS_VISUAL_OUTPUT` | `ODYSSEUS_OUTPUT_ROUTER` | routage visuel OFF |
| `ODYSSEUS_THOUGHT_BUS` | *(aucun)* | inerte |
| `ODYSSEUS_PROJECT_MANIFEST` | *(aucun)* | inerte |
| `ODYSSEUS_TOOL_DISCOVERY` | lu, mais `ToolDiscovery` sans appelant | sans effet |

---

## 7. Veille — synthèse exploitable

**108 ressources** inventoriées (29 outils, 34 systèmes IA, 12 repos, 33 patterns), **33 features candidates** (10 quick wins, 14 chantiers, 9 long terme). Détail : `../veille/VEILLE-EXTRACTION.md` et `../veille/VEILLE-FEATURES.md`.

**Top 5 idées fortes :**
1. **Boucle d'auto-amélioration fermée** (Génération→Réflexion→Curation + skills auto) — Axe 7, le plus grand écart.
2. **Découverte dynamique d'outils** (`tool_search` + registre MCP + suggestion connecteurs) — §5.13.4.
3. **Catalogue de skills à chargement obligatoire** — quick win fort.
4. **Exécution durable intégrée** (workflows + saga + approbations longues) — codée, non branchée.
5. **Observabilité du budget tokens** (CodeBurn → UI + multiplicateur multi-agent) — Axe 6/8.

---

## 8. Comment rendre ce document vivant (traçabilité continue)

1. **CI de modularité** : transformer le script de `03-MODULARITE.md` en job qui échoue si un god node dépasse un seuil (ex. > 40 importateurs) ou si une nouvelle arête `src→routes` apparaît.
2. **Traçabilité par switch** : générer `docs/traceability/killswitches.md` depuis `src/killswitch_registry.py` à chaque PR (source déjà citée).
3. **Statut SFD** : rejouer les checks de `01`/`02` (existences + switches) en CI et publier un verdict à chaque release.
4. **Tests** : installer les deps + activer pytest en local pour transformer la couverture « documentaire » en couverture « mesurée ».

---

*Fin du document maître. Sous-documents : 00→04 (traceability), VEILLE-EXTRACTION/VEILLE-FEATURES (veille).*
