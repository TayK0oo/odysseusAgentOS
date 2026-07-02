# ROADMAP M3 — Orchestration & Convergence des boucles

> **Statut :** planification validée 2026-07-02. Colonne vertébrale du câblage live.
> **Objectif Milestone 3 :** faire converger la boucle **live** (`stream_agent_loop`) et la
> boucle **orchestrée** (`CanonicalLoop`) pour que tous les modules du harness codés en
> Wave 1-4 s'activent réellement dans le chat, sans jamais casser le base product.

---

## 1. Diagnostic — pourquoi ~35 % câblé

Il existe **deux boucles distinctes** dans le code :

| | Boucle **live** | Boucle **orchestrée** |
|---|---|---|
| Fichier | `src/agent_loop.py:1934` `stream_agent_loop` (3321 lignes) | `src/orchestrator/loop.py` `CanonicalLoop` |
| Déclenchée par | **chaque message de chat** | `dispatch()` uniquement (runs isolés) |
| Round loop | `for round_num in range(1, max_rounds+1)` @2513 | `advance()` / `goto()` sur 7 phases |
| Appelle `set_phase()` ? | ❌ **jamais** | ✅ via `apply()` |
| Active phase-lock / forced_tools / ModelRouter ? | ❌ | ✅ |

**Conséquence vérifiée :** `stream_agent_loop` n'a **aucune** référence à `CanonicalLoop`,
`orchestrator`, ni `tool_registry.set_phase`. Chaque session live tombe donc sur
`default_phase: BUILD` (permissif) → le phase-lock, les forced_tools, le ModelRouter par
stage, l'intent_gate, le RRF, l'observer, l'autoeval… tous branchés sur `CanonicalLoop`,
ne sont **jamais atteints par le chat**. D'où : codés, unit-testés, dormants.

**La racine unique du "35 %" : les modules pendent à une boucle que le chat n'emprunte pas.**

---

## 2. Décision d'architecture — Option C (convergence incrémentale)

Trois options envisagées :

- **A — délégation d'un bloc** : `stream_agent_loop` appelle `CanonicalLoop.apply()` par round.
  Rejeté : chirurgie big-bang sur 3321 lignes, risque de régression produit.
- **B — réécriture** : `CanonicalLoop` devient l'exécuteur, `stream_agent_loop` une couche mince.
  Rejeté : casse le base product qui marche à 100 %.
- **C — convergence incrémentale via `PhaseTracker`** ✅ **RETENUE**.
  On extrait un middleware léger que **les deux boucles partagent**. Chaque module dormant
  s'accroche à sa phase, un par un, derrière kill-switch. Zéro big-bang, produit jamais cassé.

### Invariant de sécurité M3

> Tout câblage live qui **restreint** le comportement (phase-lock, gate, budget bloquant)
> arrive derrière un **kill-switch env par défaut OFF** (patron du gate destructif
> `ODYSSEUS_DESTRUCTIVE_GATE`). On câble d'abord (observe-only / tracking), on teste, on
> ajuste le mapping, **puis** on flippe l'enforcement phase par phase. Le base product ne
> change de comportement qu'une fois le mapping prouvé sûr.

---

## 3. Les 6 vagues (séquencées par dépendance)

Chaque vague est **livrable seule**, **testable seule**, et **débloque la suivante**.
Les codes (A0, Bloc A…) renvoient à `INTEGRATION-TRACKING.md`.

### M3.0 — Le pont (`PhaseTracker`) 🔴 LINCHPIN
- **But :** `stream_agent_loop` porte enfin une phase par round via un `PhaseTracker`
  partagé, qui appelle `ToolRegistry.set_phase(session_id, phase)`.
- **Sécurité :** kill-switch `ODYSSEUS_PHASE_TRACKER` (défaut **OFF**). OFF → comportement
  live identique à aujourd'hui (BUILD permissif). ON → phases inférées + phase-lock actif.
- **Débloque :** phase-lock live, forced_tools, et toute la carte d'articulation §5.
- **Dépend de :** rien (gate A0 déjà livré).
- **Plan détaillé :** `docs/superpowers/plans/2026-07-02-m3.0-phase-bridge.md`

### M3.1 — Routing & garde en tête de boucle (Bloc A)
- `ModelRouter.route` + `intent_gate` appelés au début de chaque round live (phase CLASSIFY).
- Purge des modèles fantômes de `config/stage-model.yaml` → vrais modèles.
- **Débloque :** sélection de modèle par stage, refus d'intents interdits.
- **Dépend de :** M3.0 (besoin de la phase pour router par stage).

### M3.2 — Connaissance réelle (Bloc RAG / E)
- RRF hybride appelé en phase KNOW (fin du blend naïf 0.7/0.3 dans le chat).
- `acontext` : remplacer le stub JSON par le service réel (KNOW + MEMORY_OBSERVE).
- checkpoint Trinité appelé en phase MEMORY_OBSERVE.
- **Débloque :** récupération de connaissance de qualité, mémoire persistante réelle.
- **Dépend de :** M3.0 (phases KNOW/MEMORY_OBSERVE doivent exister en live).

### M3.3 — Boucle qualité fermée (Bloc autoeval + observer)
- autoeval keep/revert piloté par la boucle en phase AUTOEVAL (plus API-only).
- observer : de log-only à signal exploité (budget token/coût réel, pas juste itérations).
- **Débloque :** auto-correction, budget économique réel.
- **Dépend de :** M3.0, M3.1.

### M3.4 — Gouvernance & sécurité résiduelle (Bloc G suite + governance)
- command_validator sur `run_script` / `run_local` (fermer les contournements du gate A0).
- governance : la boucle crée l'ancestry/lineage (plus API-only).
- **Débloque :** couverture sécurité complète, traçabilité des décisions.
- **Dépend de :** M3.0.

### M3.5 — Surface externe (Bloc gateway + agents)
- Enregistrer les adapters Discord/Telegram sur le bus (bus vide aujourd'hui).
- Charger les 10 agents `.opencode/agents/*.md` dans l'UI web (pipeline lists Phase 6).
- Graphify : brancher un vrai service (0 svc aujourd'hui).
- **Débloque :** canaux externes, subagents visibles, graphe de code.
- **Dépend de :** M3.0..M3.3 (les agents héritent du loop câblé).

---

## 4. Graphe de dépendances

```
                 M3.0 (PhaseTracker) ──┬──> M3.1 (routing/gate) ──> M3.3 (qualité)
                                       ├──> M3.2 (connaissance) ──> M3.3
                                       ├──> M3.4 (gouvernance/sécu)
                                       └──> M3.5 (surface externe, après M3.1-3.3)
```

M3.0 est le seul prérequis dur universel. M3.4 peut avancer en parallèle de M3.1-3.3.

---

## 5. Carte d'articulation — module → phase canonique

```
CLASSIFY        → intent_gate · risk_classifier (gate ✅) · ModelRouter(stage)
KNOW            → RRF hybride · acontext · RAG vector · Trinité(read)
PLAN            → GSD pipeline lists · ModelRouter(planner stage)
BUILD           → ToolRegistry.set_phase (phase-lock) · forced_tools · gate destructif ✅
QUALITY         → command_validator (run_script/run_local) · agents Debate/Audit
AUTOEVAL        → autoeval keep/revert · observer(budget)
MEMORY_OBSERVE  → governance ancestry · Trinité checkpoint · trace_writer ✅ · CBM
```

Les ✅ sont déjà vivants (M2). Tout le reste s'allume quand sa phase existe en live (M3.0)
puis quand sa vague le branche.

---

## 6. Alignement avec l'existant

- **ROADMAP.md (15 phases Wave 1-4)** : décrit *quels modules écrire*. M3 décrit *comment
  les câbler live*. Pas de conflit : M3 consomme les modules livrés par les 15 phases.
- **Vision §5.6 (build order harness 1→8 avant 9→15)** : respecté — M3.0-3.3 câblent le
  harness (routing, connaissance, qualité) avant M3.5 (surface/governance étendue).
- **INTEGRATION-TRACKING.md** : les Blocs A-G y sont trackés ; M3.x = leur séquencement.

---

## 7. Definition of Done — Milestone 3

- [ ] `stream_agent_loop` porte une phase par round (M3.0)
- [ ] Kill-switch `ODYSSEUS_PHASE_TRACKER` documenté dans `.env.example`
- [ ] ModelRouter + intent_gate actifs en tête de boucle live (M3.1)
- [ ] RRF + acontext + Trinité actifs dans le chat (M3.2)
- [ ] autoeval + observer pilotés par la boucle (M3.3)
- [ ] command_validator couvre run_script/run_local ; governance crée l'ancestry (M3.4)
- [ ] adapters + agents + Graphify actifs (M3.5)
- [ ] Câblage live réel ≥ 85 % (mesure INTEGRATION-TRACKING)
- [ ] Zéro régression : suite de tests verte, base product intact avec kill-switch OFF
