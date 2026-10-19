# AUDIT & CARTOGRAPHIE — Portage odysseusAgentOS → Rust

> Phase 23 du milestone v0.7 (PORT RUST COMPLET). Généré le 2026-10-19 depuis le repo principal.

## 1. Mesures du codebase Python

| Module | Fichiers | Lignes | Priorité de portage |
|--------|----------|--------|---------------------|
| `src/` | 184 | 65,315 | P0 — cœur agent |
| `core/` | 14 | 5,659 | P1 — infra |
| `services/` | 59 | 13,149 | P1 — services métier |
| `routes/` | 64 | 43,312 | P2 — API (partiel) |
| **Total (hors tests)** | **321** | **127,435** | — |

> Tests : 742 fichiers / 86 966 lignes (39 % du total). Non portés — la suite de tests Rust
> équivalente sera écrite en même temps que chaque portage.

## 2. Services (19 modules, 59 fichiers)

| Service | Fichiers | Lignes | Rôle | Port Omega |
|---------|----------|--------|------|-----------|
| `memory/` | 11 | 3,370 | memory | MemoryStore + SkillStore — P0 |
| `hwfit/` | 6 | 2,852 | hwfit | Selector (hardware fit) — P2 |
| `search/` | 10 | 2,719 | search | CatalogSource — P0 |
| `research/` | 3 | 677 | research | ContextEngine (web research) — P2 |
| `pipelines/` | 4 | 654 | pipelines | WorkflowEngine — P1 |
| `code/` | 2 | 391 | code | ContextEngine (code index) — P1 |
| `observability/` | 3 | 332 | observability | TelemetrySink + EventStore — P0 |
| `youtube/` | 2 | 322 | youtube | — — P3 |
| `tts/` | 2 | 314 | tts | ToolExecutor (TTS) — P3 |
| `vector/` | 2 | 291 | vector | CatalogSource (vector) — P1 |
| `acontext/` | 1 | 235 | acontext | ContextEngine — P2 |
| `security/` | 2 | 222 | security | PermissionPolicy + ExecPolicy — P0 |
| `stt/` | 2 | 215 | stt | ToolExecutor (STT) — P3 |
| `shell/` | 2 | 168 | shell | ToolExecutor (shell) — P1 |
| `notifications/` | 2 | 142 | notifications | Hook — P2 |
| `documents/` | 2 | 132 | documents | ToolExecutor (documents) — P2 |
| `docs/` | 2 | 112 | docs | ToolExecutor (docs) — P2 |

## 3. Fichiers pivots (top 15 par taille)

| Fichier | Lignes | Contenu | Cible Rust |
|---------|--------|---------|-------------|
| `src/agent_loop.py` | 4 986 | Boucle agent streaming + exécution d'outils multi-rounds | `omega-core::AgentLoop` (existant) — enrichir |
| `routes/email_routes.py` | 4 104 | Routes email (polling, templates) | `omega-server` (route module) |
| `routes/cookbook_routes.py` | 3 910 | Routes cookbook (recettes) | `omega-server` |
| `core/database.py` | 2 755 | Couche DB (SQLite, migrations) | `omega-server::EventStore` + `omega-app::FileEventStore` |
| `src/task_scheduler.py` | 2 747 | Planificateur de tâches (cron, queue) | `omega-app::Scheduler` |
| `src/llm_core.py` | 2 663 | Streaming LLM + fallback multi-provider | `omega-providers` |
| `routes/model_routes.py` | 2 604 | CRUD modèles | `omega-server` + `omega-router::CatalogSource` |
| `src/builtin_actions.py` | 2 567 | Actions builtin (fichiers, shell, web) | `omega-app::BuiltinActions` |
| `routes/gallery_routes.py` | 2 043 | Routes galerie (images) | P3 (non porté) |
| `src/tool_schemas.py` | 1 907 | Schémas JSON des outils | `omega-context::ToolSurfaceIndex` |
| `routes/document_routes.py` | 1 880 | Routes documents | `omega-server` |
| `routes/skills_routes.py` | 1 828 | Routes skills | `omega-server` + `SkillStore` |
| `src/tools/cookbook.py` | 1 644 | Recettes (cookbook) | `omega-app::RecipeStore` |
| `routes/shell_routes.py` | 1 709 | Routes shell | `omega-server` + `ExecPolicy` |
| `src/ai_interaction.py` | 1 207 | Interaction LLM (facade) | `omega-providers` |

## 4. Mapping composant → port Omega

| Composant Python | Port Omega existant | État | Effort |
|------------------|----------------------|------|--------|
| `src/agent_loop.py` | `AgentLoop` (omega-core) | Trait existe, implémentation à porter | Élevé |
| `src/agent_tools/*` (10 fichiers) | `ToolExecutor` + `ToolSurface` | Port existe | Moyen |
| `src/llm_core.py` | `Provider` | Port existe, omega-providers ≠ llm_core | Moyen |
| `src/tool_schemas.py` | `ToolSurface` | Port existe | Moyen |
| `services/memory/*` (11) | `MemoryStore` | Port existe, `FileMemoryStore` ≠ memory.py | Moyen |
| `services/skills` (via memory) | `SkillStore` | Port créé (v0.4) | Moyen |
| `services/search/*` (10) | `CatalogSource` | Port existe | Moyen |
| `services/observability/*` (3) | `TelemetrySink` | Port existe, omega-otel ≠ langfuse/otel_setup | Faible |
| `services/security/opa_client.py` | `PermissionPolicy` | Port existe | Faible |
| `core/database.py` | `EventStore` | Port existe, FileEventStore ≠ database.py | Élevé |
| `core/middleware.py` | `Hook` | Port existe | Faible |
| `src/task_scheduler.py` | `Scheduler` | **Port créé (v0.4)** | Élevé |
| `src/orchestrator/*` (multi_agent, registry, loop) | `SubagentRunner` | Port existe | Moyen |
| `src/budget_enforcer.py` | `ExecPolicy` | Port existe | Faible |
| `routes/*` (64) | HTTP (omega-server) | Pas de port HTTP formel | Moyen |
| `integrations/claude`, `integrations/codex` | `SubagentRunner` | Phase D v0.2 ✅ | — |

## 5. Priorisation de portage

### P0 — Cœur (bloquant pour tout le reste)
1. `services/memory` → `MemoryStore` (11 fichiers, 1 500 lignes)
2. `services/search` → `CatalogSource` (10 fichiers, 1 200 lignes)
3. `services/observability` → `TelemetrySink` (3 fichiers, 300 lignes)
4. `services/security` → `PermissionPolicy` (1 fichier, 221 lignes)

### P1 — Outils & orchestration
5. `src/agent_tools` → `ToolExecutor` (10 fichiers, 2 500 lignes)
6. `src/tool_schemas` → `ToolSurface` (1 907 lignes)
7. `src/orchestrator` → `SubagentRunner` (12 fichiers, 3 000 lignes)
8. `services/shell`, `services/pipelines` → `ToolExecutor` + `WorkflowEngine`

### P2 — Routes API (sélection)
9. `routes/chat_routes.py`, `session_routes.py`, `model_routes.py`, `shell_routes.py`, `skills_routes.py`
10. `core/middleware.py`, `core/route_loader.py`

### P3 — Non porté (documenté)
- `routes/email_*` (5 930 lignes) — spécifique Gmail/IMAP, pas un port
- `routes/cookbook_*` (5 411 lignes) — recettes produit, non cœur
- `routes/gallery_routes.py` (2 043) — galerie d'images, non cœur
- `services/stt`, `services/tts`, `services/hwfit` — médias/hardware
- `src/visual_report.py` (1 934) — rendu rapport, P3

## 6. Décision : stratégie de portage

**Portage direct (Rust natif)** pour P0 et P1 :
- Chaque composant Python → un adaptateur Rust derrière le port correspondant
- Le trait existe déjà dans `omega-ports` — il faut l'implémentation
- Tests : port des tests pytest concernés en tests Rust (`#[cfg(test)]`)

**Bridge JSON-RPC** pour P2 (routes) :
- Les 64 routes FastAPI ne sont pas portées une à une
- Un `omega-server` routeur générique + bridge vers Python pour l'existant
- Migration progressive route par route

**Non porté (P3)** : documenté dans `PORTING-SCOPE.md`.

## 7. Estimation d'effort

| Phase | Contenu | Fichiers Python | LOC Python | Effort Rust estimé |
|-------|---------|-----------------|------------|--------------------|
| 24 | Services core | 25 | 3 000 | ~4 000 LOC Rust |
| 25 | Outils d'agent | 23 | 8 700 | ~9 000 LOC Rust |
| 26 | Routes (sélection) | 10 | 8 000 | ~6 000 LOC Rust |
| 27 | Validation | — | — | ~1 500 LOC Rust (tests) |

**Total estimé : ~20 500 LOC Rust** (≈ 2.3× la surface Python du périmètre retenu).

## 8. Fichiers à créer

```
omega/crates/omega-memory/       # MemoryStore + SkillStore
omega/crates/omega-search/       # CatalogSource
omega/crates/omega-observability/ # TelemetrySink (langfuse + otel)
omega/crates/omega-policy/       # PermissionPolicy + ExecPolicy (OPA)
omega/crates/omega-tools/        # ToolExecutor + ToolSurface (agent_tools)
omega/crates/omega-orchestrator/ # SubagentRunner + WorkflowEngine + Scheduler
```

## 9. Références

- `INTEGRATION-ODYSSEUS.md` — cartographie ports ↔ modules (v0.5)
- `omega/crates/omega-ports/src/lib.rs` — 24 ports (16 + 8)
- `/home/tayk0oo/Projets/odysseusAgentOS/` — repo principal (Python)
- ADR 0010 — architecture "everything is a plugin"
