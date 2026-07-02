# STATE — AgentOS Odysseus

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-27)

**Core value:** Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, apprend de chaque run, et ne dépasse jamais ses limites
**Current focus:** Milestone 2 — INTÉGRATION RÉELLE du harness dans Odysseus (câblage live)

> ⚠️ **CORRECTION 2026-07-01 (audit vérifié).** Le statut « 100% / 15 phases » ci-dessous
> était FAUX. L'audit runtime (4 agents + vérifs manuelles, voir `INTEGRATION-TRACKING.md`)
> montre que la plupart des modules du harness sont **codés & unit-testés mais NON branchés**
> au loop live. Les 48 tests passent car ils testent les modules en isolation, pas l'intégration.
> **Câblage réel estimé : ~30-40%.** Détail + preuves fichier:ligne dans `INTEGRATION-TRACKING.md`.

## Current Position

- **Milestone 1 (modules du harness) :** ✅ écrit & unit-testé — mais majoritairement dormant
- **Milestone 2 (intégration live dans Odysseus) :** 🔨 EN COURS — planification
- **Câblage réel :** ~30-40% (voir INTEGRATION-TRACKING.md)

## Progress — RÉEL (post-audit 2026-07-01)

```
Câblage live : ███░░░░░░░ ~35%

Vraiment vivant :   trace_writer 🟢 · budget itérations 🟢 · command_validator (1 chemin) 🟡
Codé, non appelé :  llm_router · intent_gate · RRF hybride · governance · observer · autoeval
Placeholder/cassé : acontext (dump JSON) · stage-model-yaml (modèles fantômes) · Graphify (0 svc)
                    · Obsidian MCP (jamais démarré) · adapters Discord/Telegram (non enregistrés)
Inactif dans l'UI : les 10 agents .opencode/agents/*.md (CLI OpenCode seulement)

Milestone 1 (modules écrits, Wave 1-4) — voir historique ci-dessous : livré mais non intégré.
```

## Milestone 1 — modules écrits (Wave 1-4, non intégrés)

```
Phase  1 ~ Constitution        risk_classifier(loggue, ne gate PAS) · trace_writer 🟢
Phase  2 ~ LLM Router          codé, 0 call-site live · stage-yaml = modèles fantômes
Phase  3 ~ Tool Registry       phase-lock codé mais inerte (phase jamais set → BUILD)
Phase  4 ~ MCP Tools           scrapling/kroki REST-only, pas exposés comme tools agent
Phase  5 ~ Budget + Observer   itération hard-stop 🟢 · token/coût ignoré · observer loggue seul
Phase  6 ~ Pipeline Lists      ABSENT · subagents .md inactifs dans l'UI web
Phase  7 ~ Autoeval            keep/revert réel mais API-only (loop ne le pilote pas)
Phase  8 ~ Acontext/Obsidian   acontext = stub JSON · Obsidian MCP jamais démarré
Phase  9 ~ Governance          tables DB réelles mais API-only (loop ne crée pas d'ancestry)
Phase 10 ~ Channel Gateway     adapters existent mais jamais enregistrés → bus vide
Phase 11 ~ Knowledge Trinité   CBM 🟢 · Graphify 0 svc · checkpoint jamais appelé
Phase 12 ~ Command Validator   1 chemin protégé · run_script/run_local contournent
Phase 13 ~ Design Agents       .md seulement, non chargés par l'app
Phase 14 ~ Debate/Audit        .md seulement, inactifs dans l'UI web
Phase 15 ~ RAG/BYOX/Tests      RRF codé mais jamais appelé · chat = blend naïf 0.7/0.3
```

## Recent Decisions

- 2026-07-02 — **M3.1 cleanup routing (partiel) livré**. (1) `router_advice` retargeté : mappe l'intent → **rôle natif** (default/utility/research/vision) résolvable par `resolve_endpoint`, plus d'appel `ModelRouter` (commit a4086d5, 11 tests). (2) **Supprimé** `src/llm_router.py` + `.planning/stage-model-assignment.yaml` (REDONDANTS) ; re-export retiré de `src/__init__.py` (piège eager-import géré) ; `dispatcher.resolve_model` utilise un router injecté duck-typé (n'importait pas ModelRouter) ; `TestLlmRouter` retiré ; commentaire migration `llm_core.py` corrigé (commit c4cda4b, `import app` OK, 44 tests verts). **CONSERVÉ (live)** : `model-routing.json` + `routes/routing_routes.py` + `src/zen_router.py` — zen sur vrai chemin de dispatch (`llm_core.py:2457`, gated `OPENCODE_API_KEY`). **Retrait complet BLOQUÉ** sur migration endpoint Zen Go → `ModelEndpoint` natif.
- 2026-07-02 — **Cartographie native complète + re-plan M3 (zéro redondance)**. Directive user : « mapper Odysseus d'abord pour ne faire aucune redondance ». 7 agents parallèles → `.planning/intel/domains/01→07.md` + `INDEX.md` (routing table L3 + **redundancy map**). Verdict par greffon : **REDONDANT** (ModelRouter, model-routing.json, stage-model-assignment.yaml, routing_routes, channel enums EMAIL/WEBHOOK, gateway-as-bus) · **PARTIEL/réparer en place** (RRF déjà codé mais orphelin+bugué `rag_vector.py:731/:749`, command_validator, budget/trace/observer, zen_router live) · **UNIQUE/câbler** (PhaseTracker, destructive gate ✅, governance heartbeat/ancestry, Trinité) · **DORMANT/réveiller** (adapters Discord/Telegram via event_bus natif). Pièges retrait documentés (INDEX §3) : `src/__init__.py:3-4` eager import, zen live `llm_core.py:2457`. Risque #1 : token accounting éclaté en 5 lieux, aucun run_id de corrélation. `ROADMAP-M3-ORCHESTRATION.md` **réécrit** sur cette base : supprimer redondant · réparer partiel · câbler unique · réveiller dormant.
- 2026-07-02 — **M3.1 (advisory)** : premier call-site LIVE de `intent_gate.classify_intent` + `ModelRouter.route`, en tête de `stream_agent_loop` (`src/orchestrator/router_advice.py`). Log-only, kill-switch `ODYSSEUS_MODEL_ROUTER` (défaut OFF), **n'écrase jamais** le modèle choisi par l'user. Ferme l'audit « llm_router 0 call-site » + « intent_gate non appelé ». 8 tests. Commit f8fb55a. ⚠️ **Enforcement (auto-routing) BLOQUÉ** sur 2 décisions : (1) catalogue de modèles réels — les IDs de `model-routing.json` (deepseek-v4-pro, kimi-k2.6, glm-5.2…) sont fantômes + `stage-model-assignment.yaml` route vers `openrouter` (enabled:false) ; (2) l'auto-routing peut-il écraser le modèle explicite de l'user ?
- 2026-07-02 — **M3 roadmap** : `ROADMAP-M3-ORCHESTRATION.md`. Diagnostic racine du « 35% » = **deux boucles** (live `stream_agent_loop` vs orchestrée `CanonicalLoop`) ; la live n'appelait jamais `set_phase` → tout tombait sur `default_phase: BUILD`. Décision : **Option C** (convergence incrémentale via `PhaseTracker` partagé, kill-switch, mapping tuné vague par vague). 6 vagues M3.0→M3.5 séquencées par dépendance.
- 2026-07-02 — **M3.0 livré (le pont)** : `src/orchestrator/phase_tracker.py` `PhaseTracker` branché en tête du round loop live (`agent_loop.py:2513`). Pose une phase par round dans `ToolRegistry` = phase-lock enfin atteignable depuis le chat. Kill-switch `ODYSSEUS_PHASE_TRACKER` (défaut **OFF** → base product inchangé). 10 tests. 2 commits (7370610, 84e209f). Plan détaillé : `docs/superpowers/plans/2026-07-02-m3.0-phase-bridge.md`.
- 2026-07-01 — **M2-P1** : package `src/orchestrator/` (registry + spec + dispatcher) ; `resolve_model()` = premier call-site live de `ModelRouter`. 16 tests. (Bloc 0)
- 2026-07-01 — **M2-P2** : `CanonicalLoop` pilote `ToolRegistry.set_phase()` ; 7 phases canoniques dans `phase-lock.yaml` ; le phase-lock bloque/autorise réellement par phase. (Bloc 0b)
- 2026-07-01 — **M2-P3** : `dispatch()` exécute un objectif à travers le loop 7 phases (runs orchestrés isolés du chat). Suite orchestrateur = 38 verts. (Bloc 0c)
- 2026-07-02 — **M2-P4** : gate destructif RÉEL. `orchestrator/gate.should_block_destructive` bloque les commandes shell catastrophiques (`rm -rf`, fork bomb, `mkfs`, `dd`, `drop database`, …) au point central `tool_execution.py:553` (avant = log-only). Outils destructifs explicites préservés ; kill-switch `ODYSSEUS_DESTRUCTIVE_GATE`. 10 tests. Ferme l'écart audit « risk_classifier logue, ne gate PAS ». (Bloc A0)
- 2026-06-27 — Analyse profonde 60 outils complétée (5 batches + addendum). Toutes les décisions architecturales tranchées (A.7).
- 2026-06-27 — GSD initialisé sur le projet. 15 phases définies, 76 requirements mappés.
- 2026-06-27 — Séquence de build confirmée : phases 1→8 (harness) avant 9→15 (mémoire/governance/UI).
- 2026-06-29 — **Wave 1** (phases 1–5) : harness core implémenté (risk, trace, router, intent gate, hash validator, tool registry, phase-lock, MCP routes, budget, observer).
- 2026-06-29 — **Wave 2** (phases 7, 9–12) : autoeval loop, governance + DB schema, channel gateway, command validator + docker hardening.
- 2026-06-29 — **Wave 3** (phases 13–15) : agents design/debate/audit, RAG vector RRF hybrid, knowledge routes Trinité, BYOX indexer.
- 2026-07-01 — **Wave 4** (Phase 6+8) : GSD pipeline lists, 6 subagents, stage model assignment, Obsidian MCP server.
- 2026-07-01 — **Audit complet** : 0 import cassé, 7/7 routes enregistrées, 3 blockers infra + 3 gaps logiques détectés.
- 2026-07-01 — **Fixes** : acontext service réel, decision-engine build context, obsidian volume, stage yaml branché, channel_gateway branché, autoeval /summary, discord+telegram activés.
- 2026-07-01 — **Tests** : 29 unitaires (harness core) + 19 E2E (RAG, autoeval, gateway) — 48/48 ✅

## Pending Todos — Post-v1

### Optionnel (v2 ou selon besoins)
- [ ] CSS refactor static/style.css (1.2MB → <200KB) — BUG-08
- [ ] Accessibility pass (aria-labels, contraste, keyboard nav) — BUG-11
- [ ] Dead code pass (supprimer imports/helpers inutilisés) — BUG-10
- [ ] AgentSeal CI (probes injection/MCP empoisonnés) — OBS-06/SEC-04
- [ ] Provider probing audit live (Anthropic/Gemini/Groq/xAI) — BUG-05
- [ ] Smoke tests Linux/macOS/Windows/Docker/WSL — BUG-01

## Blockers / Concerns

- ✅ ~~Obsidian vault path~~ — résolu : volume mount ajouté, `OBSIDIAN_VAULT_PATH_HOST` dans .env.example
- ✅ ~~ChromaDB coexistence~~ — vérifié : ports isolés (8100 vs 8029), volumes séparés, pas de conflit
- ✅ ~~Decision Engine repo croisé~~ — résolu : `build: context:` + `DECISION_ENGINE_PATH` dans .env.example
- ✅ ~~Cookbook Docker socket~~ — préservé : cap_drop commenté, socket accessible uniquement pour Cookbook

## Session Continuity

Last session: 2026-07-02
Stopped at: **M3.1 cleanup routing (partiel) livré.** router_advice retargeté rôles natifs + `llm_router.py`/`stage-model-assignment.yaml` supprimés (commits a4086d5, c4cda4b, 44 tests verts, `import app` OK). Avant : cartographie native complète (7 docs `domains/01→07` + `INDEX.md`) + `ROADMAP-M3` réécrit zéro-redondance (654fa36). M3.0 + M3.1 advisory déjà livrés.
Resume file: _(aucun)_

**Prochaine action :** poursuivre le re-plan M3. Pistes non bloquées : (1) **M3.4** (sécurité pure) — fusionner `command_validator`+`risk_classifier` en 1 liste, corriger le bug SAFE_PREFIXES `startswith` (`ls; rm -rf /` passe), couvrir les bypass shell `run_script`/`run_local`/`/api/shell/*`. (2) **M3.2** — réparer le RRF orphelin (bug reflection `rag_vector.py:749` `top_k` vs `k`) et le câbler à la place du blend 0.7/0.3. **Bloqué** : retrait `routing_routes.py`/`model-routing.json`/`zen_router` → nécessite migration endpoint Zen Go en `ModelEndpoint` natif d'abord. Voir `.planning/intel/INDEX.md §5` + `ROADMAP-M3 §3`.
_(NB : branche `dev` = ~48 commits en avance sur `origin/dev`, non poussée.)_
