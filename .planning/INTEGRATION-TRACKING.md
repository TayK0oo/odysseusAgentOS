# INTEGRATION TRACKING — Vision (AGENT-OS-ANALYSE-PROFONDE) vs Réalité

> **But de ce document.** Base de référence unique et vérifiée. Il mappe 1:1 la vision du fichier
> `AGENT-OS-ANALYSE-PROFONDE(2).md` (60 outils, architecture §A.6 / stack §5.5) sur ce qui est
> **réellement câblé** dans le système (Odysseus amélioré). Chaque ligne porte une preuve `fichier:ligne`.
> Ce n'est PAS un rapport « le fichier existe donc c'est fait » — c'est un audit du **câblage runtime réel**.
>
> **Méthode.** 4 agents d'audit parallèles (2026-07-01) ont grep les imports + call-sites dans le chemin
> d'exécution live (`agent_loop.py`, `tool_execution.py`, `builtin_actions.py`, `chat_processor.py`).
> Vérifié à la main pour les claims critiques.
>
> **Comment mettre à jour.** Quand un composant passe à 🟢, changer le statut + ajouter la preuve du câblage
> + cocher dans le « Tracker d'avancement » en bas. Ne jamais marquer 🟢 sans call-site live prouvé.
>
> **Documents liés :**
> - Vision cible : [`.planning/AGENT-OS-ANALYSE-PROFONDE.md`](AGENT-OS-ANALYSE-PROFONDE.md) (copie durable de `~/Downloads/AGENT-OS-ANALYSE-PROFONDE(2).md`)
> - Design Milestone 2 (approuvé) : [`docs/superpowers/specs/2026-07-01-odysseus-integration-milestone2-design.md`](../docs/superpowers/specs/2026-07-01-odysseus-integration-milestone2-design.md)
> - Cadre projet : [`PROJECT.md`](PROJECT.md) · [`ROADMAP.md`](ROADMAP.md) · [`REQUIREMENTS.md`](REQUIREMENTS.md) · [`STATE.md`](STATE.md)

## Légende de statut

| Symbole | Signification |
|---|---|
| 🟢 | **Câblé et fonctionnel** — appelé dans le chemin live, produit un effet réel |
| 🟡 | **Codé mais pas appelé** — le module existe et marche, mais aucun call-site live |
| 🟠 | **Stub / placeholder** — interface ou coquille sans la logique réelle |
| 🔴 | **Absent** — n'existe pas |

---

## RÉSUMÉ EXÉCUTIF (la manchette honnête)

Le socle **Odysseus** (chat, web search, shell, mémoire ChromaDB, cookbook, email, calendar, documents,
gallery) est **100% fonctionnel**. L'abonnement OpenCode (`opencode.ai/zen/go/v1`, 20 modèles) fonctionne,
switch manuel de modèle OK.

La **couche d'intelligence greffée** (routing, mémoire auto-apprenante, governance, canaux, pipeline)
est majoritairement un **squelette** : les modules sont écrits et unit-testés (48 tests verts), mais
**la plupart ne sont pas branchés dans le loop live**. Les tests passent parce qu'ils testent les modules
en isolation, pas leur intégration.

**Ce qui est VRAIMENT vivant dans le harness :**
- `trace_writer` — écrit une trace après chaque outil 🟢
- `budget_enforcer` — **cap d'itérations** hard-stop 🟢 (mais pas le cap token/coût)
- `command_validator` — bloque `rm -rf /` sur **1 chemin** shell 🟡 (2 chemins le contournent)
- `risk_classifier` — classe et loggue, mais **ne bloque PAS** le destructif 🟡

**Découverte structurelle majeure (transverse) :** les **10 agents `.opencode/agents/*.md`**
(gsd-planner, gsd-executor, debate-5-personas, security-audit, constitution, design-extract…) sont
**INACTIFS dans l'UI web Odysseus**. Zéro référence Python ne les charge — ils ne servent qu'au **CLI
OpenCode**, pas à l'application FastAPI qui tourne sur :7000. La « planification par sous-agents
spécialisés » n'existe donc pas dans le produit qui tourne.

**Correction de mes affirmations antérieures :** j'avais dit que le risk classifier « gate » le destructif
(FAUX — il loggue seulement), que le Channel Gateway n'avait « aucun adapter » (FAUX — les adapters
Discord/Telegram existent dans `src/adapters/`, mais ne sont jamais enregistrés au démarrage), et que les
10 agents « peuvent être invoqués » (FAUX — inactifs dans l'UI web).

---

## COUCHE 0 — CONSTITUTION / HARNESS (§5.1, §3.1)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| trace_writer | Trace riche par action | 🟢 | `tool_execution.py:613` `write_trace()` après chaque outil | — |
| risk_classifier | Le risque **change la loop** (gate destructif) | 🟡 | `tool_execution.py:550` classe ; `:553-558` ne fait que logger + set string `"gate_required"` ; action exécutée quand même `:596` | **Faire respecter le gate** : return/block sur DESTRUCTIVE + approbation humaine |
| phase-lock | Retrait réel des outils par phase | 🟡 | code de blocage réel `tool_execution.py:583-587` ; MAIS `set_phase` appelé seulement par `routes/phase_routes.py:22` (API manuelle) → phase toujours `BUILD` (blocklist vide `phase-lock.yaml:46`) | **Piloter la phase depuis le loop** selon l'étape GSD |
| tool_registry | Registre de dispatch phase-aware | 🟡 | `tool_execution.py:571` consulté, mais dispatch réel = imports directs `:642+` ; overlay inerte car phase=BUILD | Brancher au vrai dispatch OU piloter la phase |
| intent_gate | Filtre d'intention avant lancement | 🟡 | `src/__init__.py:4` export seul ; seul consommateur = ModelRouter (lui-même non appelé) | Appeler en tête de loop |
| budgets par projet | step/time/token/cost/tool-call | 🟡 | itération = hard-stop `agent_loop.py:2515-2519` 🟢 ; token/coût `:2699` **retour ignoré** → jamais bloqué | Vérifier le retour `consume_tokens()["ok"]` |

**Note.** `agent_loop.py:3135` appelle bien `execute_tool_block` (le wrapper qui contient les hooks
risk/trace/phase-lock) — donc le chemin est live ; c'est la **logique de gate** qui est neutralisée.

---

## COUCHE ROUTING — Model Router (§5.4-1, §1.6 OmO, §A.1 stages GSD)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| llm_router / ModelRouter | Auto-sélection modèle par intent/stage | 🟡 | Défini `src/llm_router.py` ; **0 call-site live** — références = `__init__.py:3` + tests seulement. Routing live réel = `llm_core.py` (+ `model-routing.json`) | **Brancher au loop** : intent → route() → modèle |
| stage-model-assignment.yaml | Stages GSD → modèle | 🔴 | Modèles listés = `openrouter/anthropic/claude-opus-4`, `deepseek-coder`, `gemini-2.0-flash`… **AUCUN n'existe** dans la liste servie ; provider `openrouter` = `enabled:false` (`model-routing.json:20`) | **Réécrire** avec les vrais modèles zen (minimax-m3, kimi-k2.7-code, deepseek-v4-flash, glm-5.2, qwen3.7…) |
| Modèles réels disponibles | — | 🟢 | `model-routing.json:25-62` via `opencode_zen` /zen/go/v1 : deepseek-v4-pro, kimi-k2.6/2.7-code, glm-5.2, qwen3.7-plus, deepseek-v4-flash… | référence pour réécrire le YAML |

---

## COUCHE MÉMOIRE + CONNAISSANCE — Trinité + Acontext (§1.2, §1.7, §A.2, §4A.1 RRF)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| Acontext distillation | **INDISPENSABLE** : run → distille → SKILL.md → Obsidian | 🟠 | `services/acontext/app.py` = 38 lignes, `write_text(json.dumps(payload))` ; renvoie `"distilled"` = **string mensongère** ; 0 LLM/skill/prompt. Dockerfile installe seulement fastapi+uvicorn | **Réécrire le service** avec vrai moteur de distillation |
| Acontext hook | Déclenché en fin de session | 🟡 | `agent_loop.py:3502-3514` POST fire-and-forget vers :8029 ; outcome hardcodé `"completed"` ; service derrière `profiles:[acontext]` (off) | Activer profil + enrichir le payload |
| RAG RRF hybride | vector + BM25 reciprocal rank fusion | 🟡 | RRF **implémenté** `rag_vector.py:701-783` (`_rrf_score` = 1/(k+rank+1)) mais **jamais appelé en prod** (réfs = tests) ; étape vector = hack `inspect.getmembers` cassé → résultats vides | Brancher `hybrid_search` au chemin chat OU corriger le blend |
| RAG chat réel | — | 🟢 | `chat_processor.py:257` → `rag_manager.search` → `VectorRAG.search` `:343-398` : blend naïf `0.7*vector + 0.3*keyword` (pas RRF) | c'est la mémoire réelle du « 1 recalled » |
| CBM (Trinité structure) | Graphe code always-on | 🟢 | service `codebase-memory` compose `:224-236` (profile `knowledge`) | — |
| Graphify (Trinité sémantique) | KG multi-input | 🔴 | route attend `:9750` mais **aucun service** dans compose (grep graphify = 0) | Ajouter service OU retirer la route |
| Obsidian (Trinité mémoire) | Second-brain lecture/écriture | 🟡 | `mcp_servers/obsidian_mcp.py` a read/write réels mais **jamais importé** (`src/` grep = 0) ; vault monté `:ro` (écriture échouerait) ; `OBSIDIAN_MCP_ENABLED` lu par **aucun code** | Démarrer le MCP + monter en rw + brancher |
| Checkpoint CBM→Graphify→Obsidian | Obligatoire avant génération | 🟡 | `/checkpoint` `knowledge_routes.py:116-147` = route passive ; `src/` grep checkpoint = 0 → **jamais appelé** avant génération | Appeler en étape KNOW du loop |

---

## COUCHE GOVERNANCE (§2.11 Paperclip, §A.3)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| goal-ancestry (mission→goal→project→task) | Colonne vertébrale projet | 🟡 | Tables SQLAlchemy **réelles** : `Goal`, `GoalProject`, `GoalTask.ancestry_path` (`core/database.py:2417`), `AgentBudget:2422`, `AgentHeartbeat:2451` ; MAIS `GovernanceManager` instancié seulement dans `governance_routes.py:17` — loop ne crée aucune ancestry | Créer l'ancestry au démarrage d'un run |
| budget auto-pause 100% | Hard-stop + blocage nouvelles tâches | 🟡 | `budget_enforcer.py:198` `auto_pause_if_exceeded` = **mort** (0 call-site) ; itération OK mais token/coût non | Brancher au consume_tokens |
| heartbeat scheduling | Réveil + contexte injecté | 🟡 | route `/api/governance/heartbeat/{id}` existe ; loop ne bat jamais | Scheduler + injection contexte |
| approval + rollback versionné | Config versionnée | 🟠 | partiel | — |

---

## COUCHE OBSERVABILITÉ (§2.9 CodeBurn)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| observer / drift score | LOW/MED/HIGH → re-loop/escalade | 🟡 | `Observer()` instancié post-loop `agent_loop.py:3519`, drift calculé `:3522` mais **loggue seulement** `:3523` ; nouvel Observer par run = pas d'état | Persister + agir sur HIGH |
| CodeBurn ingestion | one-shot rate, waste, $↔commits | 🟡/🔴 | `record_codeburn_report` `observer.py:40` = **0 call-site** ; l'outil npm CodeBurn n'est **jamais invoqué** | Invoquer CodeBurn OU retirer |

---

## COUCHE AUTOEVAL (§1.8 autoresearch, §5.3)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| autoeval loop keep/revert | eval figé → métrique → git revert si pire | 🟡 | revert **réel** `git reset --hard` `autoeval_loop.py:130-134` ; eval réel `:56-69` ; MAIS call-sites = `/api/autoeval` (manuel) + tests ; loop ne le pilote jamais | Piloter depuis l'étape AUTOEVAL du loop |

---

## COUCHE EXÉCUTION — MCP tools (§1.3-1.5, §2.x)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| Scrapling (web) | Brique web unique, MCP | 🟡 | container `:207` (profile scrapling) + `SCRAPLING_TOOLS` schema `tool_schemas.py:1426` + proxy REST `mcp_tools_routes.py` ; MAIS **jamais importé** dans tool_registry/execution → LLM ne peut pas l'appeler | Enregistrer les tools dans le registry agent |
| Kroki (diagrammes) | 25+ formats REST | 🟡 | container `:170` + proxy REST ; `tools:[]` par design (`mcp_manager.py:38`) → REST-only | Exposer comme tool si voulu |
| Serena (édition sémantique) | Édition reference-aware, P0 | 🟠 | container `:181` seul ; **aucun client/schema**, health-ping seulement `mcp_tools_routes.py:79` | Client + schema tools |
| Supabase MCP (DB/state) | Persistance état sessions | 🟠 | `supabase_mcp.py` client réel + `SUPABASE_TOOLS:41` ; MAIS **aucun container** (grep=0) + non câblé | Container + câblage OU abandon |
| Faker (fixtures) | Tests non-flaky | 🟡 | `package.json:8` seul ; non installé, non utilisé | Installer + usage dans tests |
| Playwright (E2E) | Validation réelle | 🟢 | dispo via MCP global | — |

---

## COUCHE DESIGN (§2.7, §2.8)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| Design Extract (designlang) | URL→tokens | 🟡 | CLI `designlang` v12.15.0 installé **global** ; agent MD `.opencode/agents/design-extract.md` non chargé par l'app | Câbler comme tool OU via CLI |
| Open Design | Génération design | 🔴/🟡 | binaire non installé ; MD prompt seul | — |

---

## COUCHE CANAUX — Channel Gateway (§A.5)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| Bus inbound/outbound | 1 bus, N adapters | 🟢 | `channel_gateway.py` : `register_adapter:65`, `broadcast:82`, singleton `:106` | — |
| DiscordAdapter | Bot officiel Discord | 🟡 | **EXISTE** `src/adapters/discord_adapter.py:11` (discord.py réel) ; MAIS `register_adapter` **jamais appelé** au démarrage (grep app/launcher = 0) → `_adapters` vide. Bug latent : `.send` requiert `_client` défini seulement dans `start_listening` | Enregistrer au démarrage + fix send-only |
| TelegramAdapter | Bot API Telegram | 🟡 | **EXISTE** `src/adapters/telegram_adapter.py:11` (python-telegram-bot réel) ; idem non enregistré | Enregistrer au démarrage |
| Hook broadcast loop | Diffuse chaque réponse | 🟡 | `agent_loop.py:3490-3500` gardé par `if _gateway._adapters:` → **no-op** (0 adapter) | Actif dès enregistrement |

---

## COUCHE PLANNING — GSD (§A.1)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| Pipeline Lists (WAIT/MOVE/SKIP) | Orchestration déclarative pré-compilée | 🔴 | Absent — seul `stage-model-assignment.yaml` existe (et phantom) | Concevoir + implémenter |
| Subagents gsd-*.md | Planner/executor/verifier… | 🔴 | 10 MD dans `.opencode/agents/` mais **inactifs dans l'UI web** (grep py `opencode/agents` = 0) — CLI OpenCode seulement | Décider : porter dans l'app OU rester CLI |
| Stage model assignment | Modèles par stage | 🔴 | voir couche routing (phantom) | Réécrire |

---

## COUCHE SANDBOX / SÉCURITÉ (§3.2, §3.3, §5.2)

| Composant | Vision | Statut | Preuve | Écart / action |
|---|---|---|---|---|
| command_validator | Validation cmd/scope (HexStrike défensif) | 🟡 | bloque `rm -rf /` `builtin_actions.py:319-322` ; MAIS `action_run_script:340` + `action_run_local:352` appellent `_run_subprocess(shell=True)` **sans validation** → contournement réel | Valider TOUS les chemins shell |
| Docker durci | cap_drop, no-new-priv, s6 | 🟡 | `no-new-privileges:true` actif `:87` ; `cap_drop:[ALL]` **commenté** `:88` ; `read_only:false` ; socket docker monté r/w `:26` (privilège) ; **pas de s6** | Activer cap_drop progressif |
| AgentSeal probes | Sécurité injection/MCP | 🔴 | absent | v2 |

---

## TRACKER D'AVANCEMENT (à cocher au fur et à mesure)

Regroupé par sous-système (= futurs blocs de plan GSD). `[ ]` = à faire, `[x]` = câblé & vérifié.

### Bloc A — Routing auto-modèle
- [ ] Réécrire `stage-model-assignment.yaml` avec les vrais modèles zen
- [ ] Brancher `ModelRouter.route(intent, stage)` dans `agent_loop.py` (remplacer/compléter llm_core)
- [ ] Appeler `intent_gate` en tête de loop pour classifier
- [ ] Vérifier switch auto en conditions réelles (research→deep, code→kimi-code, verify→flash)

### Bloc B — Mémoire auto-apprenante (Acontext réel)
- [ ] Réécrire `services/acontext/app.py` avec moteur de distillation (LLM → SKILL.md)
- [ ] Enrichir le hook `agent_loop.py:3502` (outcome réel, messages)
- [ ] Sortie distillation → vault Obsidian (rw)
- [ ] Recall SKILL.md au démarrage d'un run
- [ ] Activer le profil acontext par défaut

### Bloc C — Governance active
- [ ] Brancher `consume_tokens()["ok"]` → hard-stop coût/token (`agent_loop.py:2699`)
- [ ] Créer goal-ancestry au démarrage d'un run (GovernanceManager dans le loop)
- [ ] `auto_pause_if_exceeded` appelé réellement
- [ ] Observer : persister l'état + agir sur drift HIGH
- [ ] Décision CodeBurn : invoquer OU retirer `record_codeburn_report`

### Bloc D — Canaux
- [ ] Enregistrer DiscordAdapter + TelegramAdapter au démarrage (si tokens présents)
- [ ] Fix bug send-only DiscordAdapter (`_client`)
- [ ] Brancher inbound bus → agent (piloter depuis chat)
- [ ] Vérifier broadcast sortant réel

### Bloc E — Pipeline Lists (GSD déclaratif)
- [ ] Décider portée : app web vs CLI OpenCode
- [ ] Concevoir format WAIT/MOVE/SKIP
- [ ] Piloter la phase (phase-lock) depuis les transitions

### Bloc F — Outils exécution
- [ ] Enregistrer Scrapling comme tool agent (pas juste REST)
- [ ] Serena : client + schema tools
- [ ] Graphify : ajouter service OU retirer route
- [ ] Obsidian MCP : démarrer + rw + brancher
- [ ] Décision Supabase / Faker

### Bloc G — Sandbox durci
- [ ] command_validator sur `action_run_script` + `action_run_local`
- [ ] Faire respecter le gate risk DESTRUCTIVE
- [ ] cap_drop progressif
- [ ] (v2) AgentSeal

### Transverse — Décision structurelle
- [ ] **Trancher : les agents `.opencode/agents/*.md` doivent-ils être actifs dans l'UI web Odysseus ?**
      (Aujourd'hui inactifs — CLI OpenCode seulement. Impacte planning/design/debate/audit.)

---

_Dernière mise à jour : 2026-07-01 — audit initial 4 agents + vérifs manuelles._
