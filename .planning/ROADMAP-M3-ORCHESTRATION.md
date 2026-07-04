# ROADMAP M3 — Orchestration & Convergence des boucles

> **Statut :** re-planifié 2026-07-02 **après cartographie native complète** (`.planning/intel/`).
> **Objectif Milestone 3 :** activer dans le chat live UNIQUEMENT ce qu'Odysseus n'a PAS déjà,
> en réparant/supprimant/câblant selon le verdict de redondance — sans jamais casser le base product.
>
> ⚠️ **Révision majeure vs version initiale.** La v1 supposait câbler *tous* les modules greffés
> (ModelRouter par stage, RRF nouveau, gateway-as-bus, Graphify…). La cartographie native
> (`.planning/intel/INDEX.md`) prouve qu'une **grande partie est REDONDANTE** avec le natif.
> M3 devient : **supprimer le redondant · réparer le partiel en place · câbler l'unique · réveiller
> le dormant via les seams natifs.** Voir la redundancy map complète : `.planning/intel/INDEX.md §2`.

---

## 1. Diagnostic — pourquoi ~35 % câblé

Il existe **deux boucles distinctes** dans le code :

| | Boucle **live** | Boucle **orchestrée** |
|---|---|---|
| Fichier | `src/agent_loop.py:1934` `stream_agent_loop` (3321 lignes) | `src/orchestrator/loop.py` `CanonicalLoop` |
| Déclenchée par | **chaque message de chat** | `dispatch()` uniquement (runs isolés) |
| Round loop | `for round_num in range(1, max_rounds+1)` @2533 | `advance()` / `goto()` sur 7 phases |
| Appelle `set_phase()` ? | ❌ **jamais** (avant M3.0) | ✅ via `apply()` |

**Racine unique du "35 %" :** les modules greffés pendent à `CanonicalLoop`, une boucle que le
chat n'emprunte pas → chaque session live tombe sur `default_phase: BUILD` → phase-lock,
forced_tools, etc. jamais atteints. **PhaseTracker (M3.0) est le pont.**

**Deuxième cause, révélée par la cartographie :** une partie du "manque" n'était pas un
câblage absent mais une **redondance** — le natif faisait déjà le travail (routing modèle,
guardrails, gating, RRF orphelin, eventing). Câbler ces greffons = dupliquer, pas compléter.

---

## 2. Décision d'architecture — Option C + principe zéro-redondance

**Option C (convergence incrémentale via `PhaseTracker` partagé, kill-switch, mapping tuné
vague par vague)** reste retenue pour la mécanique de câblage.

**Nouveau principe directeur (post-cartographie) :** pour chaque module greffé, appliquer le
verdict de `.planning/intel/INDEX.md §2` :

| Verdict | Règle |
|---|---|
| **REDONDANT** | supprimer / ne pas câbler (le natif fait tout) |
| **PARTIEL** | réparer/étendre le natif en place, garder seulement le delta unique |
| **UNIQUE** / **UNIQUE-COMPLÉMENT** | câbler (vrai manque natif) |
| **DORMANT** | réveiller via les seams natifs (event_bus, delivery, provider registry) |

### Invariant de sécurité M3

> Tout câblage qui **restreint** le comportement arrive derrière un **kill-switch env défaut OFF**
> (patron `ODYSSEUS_DESTRUCTIVE_GATE`). On câble d'abord (observe-only), on teste, on ajuste,
> **puis** on flippe l'enforcement. Le base product ne change qu'une fois le mapping prouvé sûr.
> Tout **retrait** de module respecte les pièges de `.planning/intel/INDEX.md §3`.

---

## 3. Les vagues (re-séquencées par verdict + dépendance)

Chaque vague est livrable seule et testable seule. **M3.0 est le seul prérequis dur universel.**

### M3.0 — Le pont (`PhaseTracker`) ✅ LIVRÉ
- `stream_agent_loop` porte une phase par round via `PhaseTracker` partagé → `set_phase`.
- Kill-switch `ODYSSEUS_PHASE_TRACKER` (défaut OFF). 10 tests. Commits 7370610, 84e209f.
- **Débloque :** phase-lock live, forced_tools, toute la carte d'articulation §5.

### M3.1 — Nettoyage du routing redondant  🔧 REMPLACE l'ancien "routing par stage"
> **Verdict cartographie :** ModelRouter = REDONDANT, zen_router = PARTIEL (live), intent_gate = UNIQUE.
- **Retarget `router_advice`** : garder `intent_gate.classify_intent`, **retirer l'appel
  `ModelRouter.route`**. L'advisory suggère un **rôle natif** (`default`/`utility`/`research`/`task`)
  résolu par `resolve_endpoint`, PAS un catalogue fantôme. Kill-switch `ODYSSEUS_MODEL_ROUTER` OFF.
- **Supprimer** (après avoir retiré le re-export `src/__init__.py:3-4` — piège INDEX §3.1) :
  `src/llm_router.py`, `model-routing.json`, `.planning/stage-model-assignment.yaml`,
  `routes/routing_routes.py`. Mettre à jour `tests/test_harness_core.py:156-196` + `test_orchestrator_router_advice.py`.
- **zen_router :** NE PAS supprimer. Migrer d'abord l'endpoint Zen Go → `ModelEndpoint` natif
  (provider `opencode-go` déjà dans `_detect_provider`), puis retirer les tiers fantômes en
  gardant la blacklist par-modèle. **Tâche bloquante** — traitée séparément, hors chemin critique M3.
- **Débloque :** fin de la redondance routing ; intent advisory propre. **Dépend de :** M3.0.

### M3.2 — Connaissance : réparer l'existant, ne rien dupliquer
> **Verdict :** RRF = PARTIEL (existe déjà, orphelin+bugué), acontext = PARTIEL, Trinité = UNIQUE.
- **RRF :** ne PAS ajouter un 3ᵉ hybride. **Réparer `hybrid_search` existant** (`rag_vector.py:731`,
  bug reflection `top_k` vs `k` `:749`) et le **câbler à la place du blend naïf 0.7/0.3**
  (`rag_vector.py:378`, constantes `:37-38`). Blast radius faible (source unique). Phase KNOW.
- **acontext :** alimenter le provider natif via `MemoryProviderRegistry.register`
  (`memory_provider.py:259`), PAS ré-implémenter recall/store. Distillation fin-de-session =
  le seul bit neuf (phase MEMORY_OBSERVE). Vérifier volume `/data/acontext`.
- **Trinité (CBM/Graphify/Obsidian) :** UNIQUE — câbler `checkpoint` en MEMORY_OBSERVE ; déléguer
  la recherche sémantique doc au `VectorRAG` natif plutôt qu'à Graphify.
- **Débloque :** RRF réel dans le chat, mémoire persistante. **Dépend de :** M3.0.

### M3.3 — Qualité : réutiliser le verifier natif, consommer les metrics
> **Verdict :** autoeval greffé = REDONDANT (verifier natif existe), observer = PARTIEL.
- **Autoeval :** NE PAS ajouter un 2ᵉ verifier. **Étendre `_run_verifier_subagent`**
  (`agent_loop.py:1801`, effectful-only, capé, OFF par défaut `agent_verifier_subagent`) pour
  piloter keep/revert en phase AUTOEVAL. Respecter le design opt-in (petits modèles faux-rejettent).
- **Observer :** ✅ **FAIT** — `Observer.ingest_metrics(metrics, tool_events)` consomme les signaux
  existants (`_compute_final_metrics` + `tool_events`), ne recompute rien : one_shot_rate = ratio
  succès, waste_patterns = tools échoués, touched_files = writes seulement (reads harness non flaggés).
  Réutilise le drift machinery (writes harness → HIGH). Branché dans le bloc observer live
  (`agent_loop.py`, try-guardé, fresh-per-turn, pas de kill-switch). 8 tests.
- **Débloque :** boucle qualité fermée sans duplication de coût. **Dépend de :** M3.0.

### M3.4 — Sécurité résiduelle & gouvernance (parallélisable)
> **Verdict :** command_validator = PARTIEL, destructive gate = UNIQUE ✅, governance = mixte.
- ~~Corriger le bug SAFE_PREFIXES `startswith`~~ ✅ **FAIT** (commit 9a2c98a) : `_CHAIN_OPERATORS`
  empêche le blanchiment d'une commande chaînée.
- ✅ **Bypass shell agent-autonomes couverts** (commit e90538a) : helper partagé
  `_validate_shell_or_block` appliqué à `action_ssh_command`/`action_run_script`/`action_run_local`
  (`builtin_actions.py`). **`POST /api/shell/exec`+`/stream` : NON gatés** — admin-only +
  cross-site-protégés = intention humaine explicite ; `rm -rf [/~]` faux-positive sur le cleanup
  légitime à chemin absolu (frontend cookbook). Shell agent-autonome = destructive gate (M2-P4).
- ✅ **Nom fantôme `run_command` retiré** des 3 guards (`gate._SHELL_TOOLS`, `risk_classifier`
  TOOL_RISK_MAP+classify_tool, `tool_registry` exec_restriction) : aucun outil de ce nom n'est
  défini/dispatché (live = `bash`/`python`, `tool_schemas.py:27,41`). Test gate retargeté.
- ⛔ **Fusion des 2 listes de patterns REJETÉE (après analyse).** `command_validator.BLOCKED_PATTERNS`
  et `risk_classifier.DESTRUCTIVE_PATTERNS` ne sont pas des doublons : ce sont **2 politiques
  distinctes** — un *classifieur* avisé large (tout `rm -rf`, `DELETE FROM`, `git reset --hard`
  → DESTRUCTIVE, pour le gate + trace) vs un *enforceur* étroit (seul le catastrophique-irréversible
  `rm -rf /`/`*`, avec reason + tiers WARNING + whitelist SAFE_PREFIXES). Les regex diffèrent
  volontairement (`git push --force(?!-with-lease)` vs `git push --force`). Une fusion à plat ferait
  hard-bloquer `rm -rf /tmp/x` dans run_script/scheduled tasks = **régression**. Décision : garder 2
  politiques, se référencer mutuellement (même raisonnement que les routes HTTP admin).
- **governance :** heartbeat + goal ancestry = UNIQUE → la boucle crée l'ancestry en MEMORY_OBSERVE.
  Budgets/approval = PARTIEL → réutiliser la source tokens unique + le pattern email-confirm natif.
- **Sandbox (ops) :** activer `cap_drop:[ALL]` + minimal cap_add, proxy/retrait docker.sock.
- **Débloque :** couverture sécurité complète, traçabilité. **Dépend de :** M3.0. Parallèle à M3.1-3.3.

### M3.5 — Surface externe : réveiller le dormant, supprimer le bus redondant
> **Verdict :** gateway-as-bus + enums EMAIL/WEBHOOK = REDONDANT, adapters Discord/Telegram = DORMANT.
- **Supprimer** les arms `ChannelType.EMAIL`/`WEBHOOK` (email pollers + webhook_manager matures)
  et l'ambition event-bus du gateway (natif `event_bus` + `ScheduledTask(trigger_type=event)`).
- **Réveiller** les adapters Discord/Telegram (seul vrai neuf, 0 caller hors tests) : câbler
  `register_adapter`+`set_inbound_handler`+`start_all` dans `_startup_event` (gated `ODYSSEUS_INPROCESS_*`),
  **inbound → `event_bus.fire_event`**, **outbound → delivery natif** (`_deliver_task_result` /
  `execute_api_call`). Opt-in par session (éviter le spam broadcast `agent_loop.py:3521`).
- **Débloque :** canaux 2-way réels sans bus parallèle. **Dépend de :** M3.0..M3.3.

### M3.X — Token accounting unifié (transversal, prérequis fiabilité budget)
> **Risque #1 cartographie :** tokens comptés en 5 endroits, aucun `run_id` de corrélation.
- Choisir UN writer de vérité (metrics message `chat_messages.meta_data`), governance/trace **lisent**.
- Définir un `run_id` de corrélation partagé (traces↔budgets↔task_runs). Prérequis de M3.3/M3.4.

---

## 4. Graphe de dépendances

```
   M3.0 (pont ✅) ──┬──> M3.1 (cleanup routing) ──┐
                    ├──> M3.2 (connaissance) ──────┼──> M3.3 (qualité)
                    ├──> M3.4 (sécu/gouvernance, ∥)
                    └──> M3.5 (surface externe, après M3.1-3.3)
   M3.X (tokens unifiés) ── prérequis de M3.3 & M3.4
   [bloquant hors chemin] zen Go → ModelEndpoint natif ── prérequis retrait routing complet
```

---

## 5. Carte d'articulation — module → phase (verdict entre crochets)

```
CLASSIFY        → intent_gate [UNIQUE] · risk_classifier+gate [✅ UNIQUE-COMPL] · rôle natif (pas ModelRouter)
KNOW            → RRF réparé en place [PARTIEL] · VectorRAG natif · Trinité read [UNIQUE]
PLAN            → phase-lock write-scoped [PARTIEL] · GSD pipeline lists
BUILD           → set_phase/phase-lock [✅] · forced_tools · gate destructif [✅ UNIQUE-COMPL]
QUALITY         → command_validator fusionné+étendu [PARTIEL] · exec allowlist test-only
AUTOEVAL        → _run_verifier_subagent natif étendu [au lieu de autoeval REDONDANT] · observer via SSE [PARTIEL]
MEMORY_OBSERVE  → governance ancestry/heartbeat [UNIQUE] · Trinité checkpoint [UNIQUE] · acontext→provider natif [✅] · trace_writer [✅]
```

Supprimés de la carte v1 (redondants) : ModelRouter(stage), RRF-nouveau-module, gateway-as-bus, Graphify-comme-search.

---

## 6. Alignement avec l'existant

- **`.planning/intel/` (7 domaines + INDEX)** : source de vérité des capacités natives + verdicts.
  M3 en dérive directement. Toute nouvelle tâche charge d'abord le domaine concerné (INDEX §1).
- **ROADMAP.md (15 phases Wave 1-4)** : décrit *quels modules ont été écrits*. M3 décrit *lesquels
  câbler vs supprimer* selon le natif. Conflit résolu par les verdicts de redondance.
- **INTEGRATION-TRACKING.md** : à mettre à jour — les Blocs marqués "à câbler" mais REDONDANTS
  passent en "à supprimer".

---

## 7. Definition of Done — Milestone 3

- [x] `stream_agent_loop` porte une phase par round (M3.0)
- [x] Kill-switch `ODYSSEUS_PHASE_TRACKER` documenté dans `.env.example`
- [~] M3.1 : `ModelRouter`/`src/llm_router.py`/`stage-model-assignment.yaml` **supprimés** ✅ · re-export `src/__init__.py` **nettoyé** ✅ · `router_advice` retargeté rôles natifs (`test_orchestrator_router_advice` vert) ✅ · **reste** : `routes/routing_routes.py` (REDONDANT mais **consommateurs REST externes** → dépréciation coordonnée, pas suppression sèche) + `model-routing.json` (reclassé **PARTIEL** : garder la politique routing, retirer seulement le bloc `providers.*` une fois DB autoritaire)
- [~] zen Go → `ModelEndpoint` natif — **slices 1-3 (2026-07-03)** : provider config dédupliquée (`_zen_provider_conn`/`_resolve_zen_endpoint_row`) ✅ · activation depuis endpoint natif derrière kill-switch `ODYSSEUS_ZEN_FROM_ENDPOINT` défaut-OFF (`zen_injection_enabled`/`zen_endpoint_registered`) ✅ · bloc `providers.opencode_zen` rendu optionnel ✅ · 14 tests · plan `docs/superpowers/plans/2026-07-03-zen-modelendpoint-migration.md` · **reste** : IDs modèles fantômes + dépréciation `routing_routes.py`
- [~] M3.2 : RRF existant réparé (fonction pure fusion vecteur+BM25) + câblé dans `search` live derrière kill-switch `ODYSSEUS_RRF_FUSION` (défaut OFF, blend 0.7/0.3 inchangé) ✅ · acontext réveillé via `MemoryProviderRegistry` (provider observe-only `on_session_end`, gate `ACONTEXT_ENABLED` défaut OFF, remplace le POST synchrone hardcodé du agent_loop) ✅ · Trinité : sémantique doc déléguée au `VectorRAG` natif, Graphify fantôme supprimé ✅ · **reste** : câbler `checkpoint` dans la boucle live (DÉCISION requise : pré-gen vs MEMORY_OBSERVE post-round ; valeur dépend de CBM-HTTP + Obsidian-MCP en ligne)
- [~] M3.3 : observer consomme le SSE metrics + tool_events (`Observer.ingest_metrics`, câblé dans le bloc live, 8 tests) ✅ · **reste** : `_run_verifier_subagent` étendu pour AUTOEVAL keep/revert (DÉCISION requise : la boucle live n'a pas de notion de phase, `CanonicalLoop` n'a pas de hook d'exécution, revert = `git reset --hard` destructif)
- [x] M3.4 sécurité : run_script/run_local/ssh couverts ✅ · SAFE_PREFIXES chaînage corrigé ✅ · phantom `run_command` retiré ✅ · fusion patterns REJETÉE (2 politiques, cf. §M3.4) · api-shell admin non gatée (intention humaine) · **governance ancestry live** ✅ (2026-07-04, commit abe389a) : `src/orchestrator/ancestry_tracker.py` — bridge kill-switché `ODYSSEUS_GOVERNANCE_ANCESTRY` défaut-OFF, no-op sans `project_id` (contexte goal orchestré), câblé dans `stream_agent_loop` MEMORY_OBSERVE, réutilise `create_task_with_ancestry` natif, tâche labellisée par le run_id de corrélation (6 tests)
- [ ] Token accounting unifié (1 writer + run_id de corrélation) (M3.X)
- [~] M3.5 : enums `ChannelType.EMAIL`/`WEBHOOK` redondants supprimés (0 caller, 5 tests) ✅ · **reste** : adapters Discord/Telegram réveillés via event_bus natif + gateway-as-bus supprimé (feature à concevoir : routing inbound→`event_bus.fire_event`, outbound→delivery natif)
- [ ] Zéro redondance résiduelle vérifiée contre `.planning/intel/INDEX.md §2`
- [ ] Zéro régression : suite verte, base product intact avec kill-switch OFF
```
