# AgentOS — Odysseus

## What This Is

Workspace AI self-hosté complet bâti sur OpenCode (fork Odysseus), transformé en système d'agents autonomes de niveau production. L'objectif est de construire un harness déterministe multi-couches (Connaissance → Exécution → Automatisation → Gouvernance) capable de faire tourner des agents longue durée, bornés par des budgets et des objectifs, avec une mémoire d'apprentissage continue.

## Core Value

Un agent doit pouvoir recevoir un objectif, l'exécuter jusqu'au bout sans dériver, apprendre de chaque run, et ne jamais dépasser ses limites — sans intervention humaine constante.

## Requirements

### Validated

- ✓ FastAPI orchestrateur + agent loop streaming multi-round — existant
- ✓ LLM provider routing avec fallback (opencode_zen, ollama, openrouter) — existant dans model-routing.json
- ✓ MCP servers email/memory/image/RAG — existants dans mcp_servers/
- ✓ Deep Research multi-step — existant dans src/deep_research.py
- ✓ Task scheduler cron — existant dans src/task_scheduler.py
- ✓ Context compactor — existant dans src/context_compactor.py
- ✓ CalDAV sync + Email IMAP/SMTP — existants
- ✓ PWA + UI workspace (Odysseus) — existant dans static/
- ✓ Cookbook VRAM-aware (hwfit) — existant dans services/hwfit/
- ✓ Auth bcrypt + CSRF + scope-gated endpoints — existant
- ✓ Claude Code + Codex integrations — existants dans integrations/
- ✓ ChromaDB RAG pour documents — existant dans src/rag_vector.py

### Active

- [ ] Loop canonique + 10 invariants (Constitution agents-best-practices)
- [ ] Model Router LiteLLM + IntentGate + hash-anchored edits
- [ ] Phase-lock réel (Serena ToolMarker + ToolRegistry + mapping vibecode)
- [ ] Serena MCP (édition symbolique LSP-backed)
- [ ] Scrapling MCP (web scraping adaptatif, remplace SearXNG pour scraping)
- [ ] Supabase MCP (DB branching + state agents)
- [ ] Observations structurées JSONL + budget enforcer par projet
- [ ] CodeBurn intégré (one-shot rate + waste patterns)
- [ ] AgentSeal (probes sécurité injection/MCP empoisonnés)
- [ ] GSD comme moteur de workflow/planning (Pipeline Lists, subagents spécialisés)
- [ ] PROJECT.yaml + autoeval loop (keep/revert, pattern autoresearch)
- [ ] Acontext self-hosted (distillation run→SKILL.md→Obsidian second-brain)
- [ ] Governance (goal-ancestry, budgets granulaires, approval+rollback, heartbeat)
- [ ] Channel Gateway (Discord + Telegram + outbound MCP connectors)
- [ ] Trinité branchée dans l'UI (CBM + Graphify + Obsidian MCP dans docker-compose)
- [ ] Decision Engine intégré (depuis ConfigOpenCodeNew/decision-engine/)
- [ ] Sandbox durci (cap_drop, command validator, s6-overlay)
- [ ] Design Extract + Open Design (MCP design)
- [ ] Debate 5 personas + edge-case generator (agents Markdown)
- [ ] Autoeval loop (keep/revert pattern autoresearch)
- [ ] RRF hybrid search + BYOX corpus indexé
- [ ] Bugs/refactor amont (CSS, prompt injection audit, provider probing)

### Out of Scope

- n8n — remplacé par Decision Engine Python (moins lourd, même langage)
- ChromaDB comme mémoire agentique — gardé pour RAG docs mais pas pour la mémoire d'apprentissage (Acontext)
- OpenWA/Chatwoot — risque ban WhatsApp ; PWA + email + Discord/Telegram suffisent
- Almanac MCP — redondant avec Context7/CBM
- Arsenal offensif HexStrike — seuls les patterns défensifs (validation cmd/scope/whitelist/audit)
- Fork complet Paperclip — patterns absorbés dans Decision Engine
- Fork complet vibecode — patterns RIPER-5/phase-lock/drift absorbés

## Context

**Analyse de 60 outils réalisée** (document AGENT-OS-ANALYSE-PROFONDE.md, juin 2026) — 5 batches complets + addendum avec décisions finales tranchées.

**Architecture finale décidée** (A.6 + A.7) :
- UI : ce repo (fork Odysseus, MIT) — codebase à s'approprier
- Workflow/Planning : GSD (gsd-opencode) = moteur cœur
- Mémoire : Acontext full self-hosted (indispensable)
- Phase-lock : Serena (moteur) + mapping vibecode (politique)
- Routing : Model Router (OmO patterns) + LiteLLM, alimenté par stages GSD
- Canaux : Channel Gateway — Discord + Telegram + Email (sûrs), outbound via MCP adapters
- Orchestration runtime : Decision Engine = host/glue (Governance + Gateway + observer)

**Séquence de build** (Constitution agents-best-practices) : loop manuel → tools → permissions → observations → budgets → tracing → planning → context/memory → compaction → skills/connectors → goal loop → subagents. Ne jamais sauter d'étape.

**Principe directeur** : le harness > le modèle. La fiabilité vient de la structure déterministe autour du LLM, pas d'un modèle plus gros.

## Constraints

- **Tech stack** : Python/FastAPI (src/) + TypeScript (static/) — ne pas changer le socle
- **Docker** : architecture multi-services, tout nouveau composant = service docker-compose
- **Markdown-first** : agents, skills, mémoire, design systems → fichiers .md versionnés Git
- **MCP-first** : chaque capacité nouvelle = MCP server activable par flag dans config
- **Séquence** : ne pas implémenter mémoire/governance avant que le harness (phases 1-8) soit stable

## Key Decisions

| Décision | Rationale | Outcome |
|----------|-----------|---------|
| Fork Odysseus comme base UI | MIT, PWA+email+Cookbook VRAM-aware déjà là — mois gagnés | — Pending |
| GSD comme moteur de workflow | Pipeline Lists > decision-tree.yaml ; stage model assignment natif | — Pending |
| Acontext full self-hosted | Moteur de distillation = cœur dur, pas reconstructible facilement | — Pending |
| Paperclip absorbé (pas forké) | UI = Odysseus, pas besoin de doublon React | — Pending |
| Phase-lock via Serena | ToolMarker natif fiable, pas de réinvention | — Pending |
| LiteLLM comme abstraction routing | Un seul SDK pour tous les providers | — Pending |
| n8n écarté | Decision Engine Python = plus léger, même langage, contrôle total | ✓ Good |
| ChromaDB conservé pour RAG docs | Use case distinct de la mémoire agentique | — Pending |
| Serena + Scrapling + Supabase en MCP | MCP-first, activables par flag | — Pending |
| Discord + Telegram (pas WhatsApp) | APIs officielles, risque ban WhatsApp trop élevé | ✓ Good |

---
*Last updated: 2026-06-27 après analyse profonde 60 outils + initialisation GSD*
