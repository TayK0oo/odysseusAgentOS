# INVENTAIRE-MAÎTRE — Odysseus AgentOS

**Date** : 2026-07-07
**Commit** : `2821daa`
**Cartographe** : Agent open-coder (read-only, zero-trust)
**Durée d'exploration** : ~1 session

---

## SYNTHÈSE EXÉCUTIVE

**Odysseus AgentOS** est une application FastAPI (~130K lignes Python) qui expose :
- Un **chat agentique** avec LLM via LiteLLM (streaming SSE)
- **60+ modules API** (chat, mémoire, email, calendrier, galerie, notes, tâches, modèles, shell, cookbook…)
- Un **orchestrateur** modulaire (7 phases canoniques, phase-lock, gates, agents .opencode/)
- Une **UI SPA** vanilla JS (~474 fichiers statiques)

**État réel** (post-audit M3) : le harness d'orchestration est **codé et testé** (modules écrits, 4196 tests passent) mais **~70% des modules d'orchestration sont gated OFF par défaut** derrière des kill-switches. La boucle live (`stream_agent_loop`) est l'orchestrateur de fait ; les orchestrateurs formels (PhaseTracker, CanonicalLoop, AgentDispatcher) sont tous inactifs dans la configuration par défaut. Ce design ("câblé mais dormant") est intentionnel : chaque module peut être activé individuellement sans casser le base product.

**Statistiques clés** :
- 35+ kill-switches (tous les modules M3 = OFF par défaut, `ODYSSEUS_DESTRUCTIVE_GATE` = ON par défaut)
- 12 agents `.opencode/` (tous dormant, CLI-only)
- ~38% des capacités backend (~17) sont des **angles morts UI** (aucune représentation visuelle)
- 113 fichiers `src/`, 60+ routeurs, 16 modules orchestrator, 7 services Docker Compose (+ 4 profils)

---

## TABLEAU DE BORD DES STATUTS

| Statut | Nb | Exemples notables |
|---|---|---|
| 🟢 **CÂBLÉ-ACTIF** | ~30 | `stream_agent_loop`, `llm_core`, `tool_execution`, `Observer`, `destructive gate`, `MemoryVectorStore`, `SkillsManager`, `VectorRAG`, auth, CORS, middlewares, UI SPA, tous les routeurs API, MCP builtins |
| 🔵 **CÂBLÉ-DORMANT** (kill-switch OFF) | ~25 | `PhaseTracker`, `CanonicalLoop`, `router_advice`, `AgentDispatcher`, `apply_autoeval`, `record_run_ancestry`, `record_checkpoint`, `run_codeburn`, `maybe_autoevolve`, `Acontext`, `bootstrap_channels`, 12 agents, RRF, `Obsidian MCP`, `Graphify`, `Serena`, `Scrapling`, `Kroki` |
| 🟡 **CONFIGURÉ-NON-CÂBLÉ** | ~4 | `model-routing.json providers.*` (redondant avec ModelEndpoint natif), `Decision Engine` (repo externe, profil Docker), `ntfy` (service démarré, intégration UI non vérifiée), `budget tokens` (PROJECT.yaml.example, intégration partielle) |
| 🟠 **STUB / SQUELETTE** | 1 | `data/skills/` (répertoire vide — skills extraits dynamiquement, pas pré-peuplés) |
| 🔴 **ORPHELIN** | 0 | Aucun orphelin confirmé (tous les modules importés ont au moins un call-site) |
| 👻 **FANTÔME** | 1 | `ModelRouter`/`src/llm_router.py` (supprimé M3.1, commit c4cda4b). *(Correction : `trace_writer` figurait ici par erreur d'indexation CBM — c'est en réalité un module CÂBLÉ-ACTIF, voir ci-dessous.)* |
| ⚪ **DOC-ONLY** | 2 | `Supabase MCP` (mentionné ROADMAP Phase 4, non implémenté) ; `s6-overlay` (mentionné ROADMAP Phase 12, patterns HolyClaude) |

---

## TOP ÉCARTS DOC↔CODE

| Écart | Doc | Code réel | Sévérité |
|---|---|---|---|
| **Loop canonique « 7 phases »** | `loop-canonique.md` décrit CLASSIFY→KNOW→PLAN→BUILD→QUALITY→AUTOEVAL→MEMORY_OBSERVE | La boucle live ne traverse JAMAIS ces phases. `PhaseTracker.infer_phase()` retourne toujours BUILD ou PLAN. Les 7 phases n'existent que dans `CanonicalLoop`, gated OFF. | 🔴 CRITIQUE |
| **« Câblage réel ~35% »** | STATE.md:14 (2026-07-01) | Post-M3 (2026-07-05), le câblage est ~70% (modules codés ET importés dans agent_loop), mais ~70% de ces modules sont **dormants** (gated OFF). Le « câblé réel actif » est ~30%. | 🟡 NUANCÉ |
| **« 100% / 15 phases »** | STATE.md:10 (corrigé 2026-07-01) | Corrigé par l'audit : le « 100% » était faux. | ✅ CORRIGÉ |
| **« trace_writer 🟢 »** | STATE.md:64 | **CONFIRMÉ CÂBLÉ-ACTIF.** `src/trace_writer.py` existe et est appelé à chaque tour : `agent_loop.py:1934,3584`, `tool_execution.py:542,630`, kill-switch `ODYSSEUS_UNIFIED_TOKENS` (`killswitch_registry.py:64`). Le « non trouvé » initial était une lacune d'indexation CBM. | ✅ CORRIGÉ |
| **« TOUS les providers passent par LiteLLM »** | ROADMAP Phase 2 success criteria | `llm_core.py` utilise LiteLLM pour le streaming, mais `zen_router` a son propre chemin de dispatch. | 🟡 PARTIEL |
| **« ModelEndpoint natif remplace model-routing.json »** | STATE.md:85-86, ROADMAP-M3 | `model-routing.json providers.*` est PARTIELLEMENT redondant. Les sections `models`/`stages`/`heuristic` sont UNIQUES (routing par complexité). | 🟡 PARTIEL |

---

## TOP ANGLES MORTS UI

Voir détail complet dans `C-COUVERTURE-UI.md`. Top 5 par criticité :

1. **Drift score & qualité** — L'utilisateur ne sait jamais si l'agent dérive (LOW/MED/HIGH). Le score est calculé mais jamais affiché.
2. **Budgets & consommation** — Pas de jauge de tokens/coûts dans l'UI. Le hard-stop est invisible jusqu'à ce qu'il bloque.
3. **Phase-lock & orchestration** — Aucun indicateur de phase courante, de forced_tools, ou d'état du phase-lock.
4. **Kill-switches dashboard** — 35+ switches de configuration, aucun tableau de bord centralisé. L'utilisateur doit lire les variables d'env.
5. **Trinité Connaissance** — CBM est actif (externe) mais inaccessible depuis l'UI. Graphify et Obsidian sont inactifs.

---

## TOP ZONES À INVESTIGUER (priorisées, sans correctifs)

1. **`trace_writer`** — ✅ RÉSOLU : `src/trace_writer.py` existe et est CÂBLÉ-ACTIF (call-sites `agent_loop.py:1934,3584`, `tool_execution.py:542,630`, `killswitch_registry.py:64`). Le « non trouvé » était une lacune d'indexation CBM, pas un fantôme. Le seul vrai stub aspirationnel voisin : la chaîne `"memory_distill"` (`src/orchestrator/phases.py:57`, `_FORCED_TOOLS[MEMORY_OBSERVE]`), sans implémentation — `forced_tools(MEMORY_OBSERVE)` n'est jamais appelé au runtime.
2. **Redondance `providers.*` vs `ModelEndpoint`** — Le bloc `providers` de `model-routing.json` est déclaré PARTIEL : la DB native (`ModelEndpoint`) est la source autoritaire pour la config provider, le JSON est le fallback. La suppression définitive est bloquée sur une précondition opérationnelle (enregistrement endpoint Zen manuel).
3. **Orchestrateurs triples** — PhaseTracker, CanonicalLoop, et AgentDispatcher sont trois orchestrateurs qui peuvent théoriquement s'exécuter simultanément (chacun avec son kill-switch). Aucune validation de cohérence entre eux. Risque de comportement imprévisible si plusieurs sont activés.
4. **SPOF `stream_agent_loop`** — Une seule fonction de ~3500 lignes contient toute la logique d'orchestration, le streaming LLM, les hooks post-round, et la gestion d'erreurs. Aucune décomposition modulaire dans le flux principal.
5. **Couverture de test du boot** — Le bug M3.1 (suppression de `routing_routes.py` → `import app` cassé) n'a été détecté par aucun test avant le fix `37a511c` (test smoke boot subprocess ajouté après). Vérifier la couverture actuelle du chemin de boot complet.
6. **35 kill-switches sans validation** — Aucune validation de cohérence globale. Par exemple, activer `ODYSSEUS_LIVE_ORCHESTRATION=on` ET `ODYSSEUS_PHASE_TRACKER=on` simultanément crée 2 orchestrateurs concurrents dans la même boucle.

---

## QUESTIONS OUVERTES

1. Le `Decision Engine` (repo `ConfigOpenCodeNew/decision-engine`) est-il fonctionnel et intégré ? Ou est-ce un stub Docker ?
2. Le `Serena MCP` (docker-compose, port 8765) est-il réellement utilisé par un outil agent ? La recherche CBM n'a pas trouvé de call-site dans `tool_execution` ou `agent_tools`.
3. `ntfy` (notifications push, port 8091) est-il intégré à l'UI ? Ou est-ce un service headless-only ?
4. Les `scripts/odysseus-*` (20+ CLI wrappers) sont-ils maintenus et testés ? Ou sont-ils des relics ?
5. Le `companion/` mobile — y a-t-il une app mobile réelle ou est-ce uniquement le backend d'appairage ?
6. `data/skills/` est vide. D'où viennent les skills affichés dans l'UI Memory ? Sont-ils tous extraits dynamiquement des sessions ?

---

## FICHIERS DE L'INVENTAIRE

| Fichier | Contenu | Lignes (est.) |
|---|---|---|
| `00-cadrage.md` | Stacks, points d'entrée, arborescence, CBM | ~130 |
| `01-recensement.md` | 12 agents, hooks, MCP, GSD, routes, orchestrator, kill-switches, UI | ~320 |
| `A-INVENTAIRE-FONCTIONNEL.md` | 12 couches fonctionnelles, ~80 composants avec statut + preuve | ~320 |
| `B-ORCHESTRATION.md` | Edge-list (30+ arêtes), trace loop réelle, confrontation canonique, SPOF | ~200 |
| `C-COUVERTURE-UI.md` | 2 surfaces UI, matrice 45×6, 17 angles morts, gap analysis | ~140 |
| `INVENTAIRE-MAITRE.md` | Ce fichier — synthèse, dashboard statuts, écarts, angles morts, questions | ~100 |
| `diagramme-orchestration.mmd` | Diagramme Mermaid du graphe réel | ~80 |

---

## MÉTHODOLOGIE

- **Sources primaires** : CBM (search_graph, trace_path, query_graph), grep, lecture directe de fichiers
- **Sources secondaires** : `.planning/STATE.md`, `.planning/ROADMAP-*.md`, `loop-canonique.md`, `docker-compose.yml`
- **Preuves** : Chaque affirmation cite `fichier:ligne` ou `fichier:ligne_début-ligne_fin`
- **Limites** : `src/trace_writer.py` et `src/observer.py` existent mais n'étaient pas indexés par CBM (trouvés ensuite via grep — d'où le faux « fantôme » initial de trace_writer) ; Serena MCP non testé live ; les 474 fichiers JS frontend n'ont pas été audités individuellement
