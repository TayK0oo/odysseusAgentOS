# Intégration odysseusAgentOS — Ponts et adaptateurs

> Genere le 2026-10-19. Phase 13 du milestone v0.4 (PLUGIN CONVERGENCE).

## 1. Travail existant dans odysseusAgentOS (branche principale)

### 1.1 Architecture Python (src/)

| Module | Rôle | Port Omega correspondant | Adaptateur proposé |
|--------|------|------------------------|-------------------|
| `agent_loop.py` | Boucle d'agent 7 phases | `AgentLoop` | `omega-app::PythonAgentBridge` |
| `agent_tools/` | Outils d'agent (100+ fichiers) | `ToolExecutor` + `ToolSurface` | `omega-app::PythonToolBridge` |
| `ai_interaction.py` | Interaction LLM | `Provider` | `omega-providers::PythonProviderBridge` |
| `app_context.py` | Contexte applicatif | `ContextEngine` | `omega-context::PythonContextBridge` |
| `app_helpers.py` | Helpers applicatifs | `ConfigSource` | `omega-config::PythonConfigBridge` |
| `budget_enforcer.py` | Enforcement budget | `ExecPolicy` | `omega-core::PythonBudgetBridge` |
| `builtin_actions.py` | Actions builtin | `ToolExecutor` | `omega-app::BuiltinActionBridge` |
| `builtin_mcp.py` | MCP builtin | `ToolSurface` | `omega-app::McpBridge` |
| `session_manager.py` | Gestion de session | `EventStore` | `omega-server::PythonSessionBridge` |

### 1.2 Core Python (core/)

| Module | Rôle | Port Omega correspondant | Adaptateur proposé |
|--------|------|------------------------|-------------------|
| `atomic_io.py` | I/O atomique | `SandboxBackend` | `omega-sandbox::AtomicIoBridge` |
| `auth.py` | Authentification | `PermissionPolicy` | `omega-core::PythonAuthBridge` |
| `database.py` | Base de données | `MemoryStore` | `omega-app::PythonDbBridge` |
| `middleware.py` | Middleware | `Hook` | `omega-core::PythonMiddlewareBridge` |
| `session_manager.py` | Sessions | `EventStore` | `omega-server::PythonSessionBridge` |

### 1.3 Services (services/)

| Service | Rôle | Port Omega correspondant | Adaptateur proposé |
|---------|------|------------------------|-------------------|
| `agent_runs.py` | Runs d'agents | `AgentLoop` + `EventStore` | `omega-app::AgentRunBridge` |
| `agent_tools/` | Outils d'agent | `ToolExecutor` | `omega-app::AgentToolBridge` |
| `bg_jobs.py` | Jobs en arrière-plan | `Scheduler` | `omega-app::BgJobBridge` |
| `autoeval_loop.py` | Auto-évaluation | `TelemetrySink` | `omega-otel::AutoEvalBridge` |

### 1.4 Routes API (routes/)

| Route | Rôle | Port Omega correspondant | Adaptateur proposé |
|-------|------|------------------------|-------------------|
| `api_key_manager.py` | Gestion clés API | `ConfigSource` | `omega-config::ApiKeyBridge` |
| `auth_helpers.py` | Helpers auth | `PermissionPolicy` | `omega-core::AuthHelperBridge` |
| `caldav_sync.py` | Sync CalDAV | `ToolExecutor` | `omega-app::CalDavBridge` |

### 1.5 Documentation (docs/master-ref/)

| Document | Contenu | Statut intégration |
|----------|---------|-------------------|
| `01-SFD-v3.1.md` | Spécification fonctionnelle de référence | ✅ Référencé dans ADR 0010 |
| `10-ARCHITECTURE-MODULAIRE.md` | Analyse combinatoire de 68 outils | ✅ Intégré dans CONVERGENCE-TOUT.md |
| `09-PLAN-ACTIONS-RESTANTES.md` | Plan d'actions restantes | 🔄 À intégrer |
| `06-ENGINE-OUTILS.md` | Moteur d'outils | 🔄 À intégrer |
| `07-ODYSSEUS-AUDIT.md` | Audit Odysseus | 🔄 À intégrer |

## 2. Stratégie de pont (Bridge)

### 2.1 Principe

> **Le pont (bridge) est un adaptateur qui traduit les appels Python en appels Rust et vice versa.**
> **Le noyau Rust ne dépend jamais de Python. Le bridge est un plugin optionnel.**

### 2.2 Architecture du bridge

```
┌─────────────────────────────────────────────────────────────┐
│                    omega-core (Rust)                        │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │
│  │ Provider │ │Context  │ │ Memory  │ │ Tool    │  ...     │
│  │  port   │ │ Engine  │ │ Store   │ │ Executor│          │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘          │
│       │           │           │           │                │
│  ┌────┴───────────┴───────────┴───────────┴────┐          │
│  │           PythonBridge (plugin optionnel)    │          │
│  │  - subprocess Python (isolation process)     │          │
│  │  - JSON-RPC over stdin/stdout               │          │
│  │  - timeout + retry + circuit breaker         │          │
│  └────────────────────┬────────────────────────┘          │
└───────────────────────┼─────────────────────────────────────┘
                        │
┌───────────────────────┼─────────────────────────────────────┐
│                    Python (odysseusAgentOS)                 │
│  ┌────────────────────┴────────────────────────┐          │
│  │  Bridge handler (JSON-RPC dispatcher)       │          │
│  │  - routes calls to agent_loop, agent_tools,  │          │
│  │    ai_interaction, etc.                     │          │
│  └─────────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 Ponts proposés

| Pont | Direction | Port Omega | Module Python | Priorité |
|------|-----------|------------|---------------|----------|
| `PythonAgentBridge` | Rust → Python | `AgentLoop` | `agent_loop.py` | Haute |
| `PythonToolBridge` | Rust → Python | `ToolExecutor` | `agent_tools/` | Haute |
| `PythonProviderBridge` | Rust → Python | `Provider` | `ai_interaction.py` | Moyenne |
| `PythonContextBridge` | Rust → Python | `ContextEngine` | `app_context.py` | Moyenne |
| `PythonConfigBridge` | Rust → Python | `ConfigSource` | `app_helpers.py` | Basse |
| `PythonBudgetBridge` | Rust → Python | `ExecPolicy` | `budget_enforcer.py` | Basse |
| `PythonDbBridge` | Rust → Python | `MemoryStore` | `database.py` | Moyenne |
| `PythonMiddlewareBridge` | Rust → Python | `Hook` | `middleware.py` | Haute |
| `PythonSessionBridge` | Rust → Python | `EventStore` | `session_manager.py` | Moyenne |

### 2.4 Stratégie d'intégration

1. **Phase 1 : Ponts haute priorité** (AgentLoop, ToolExecutor, Hook)
   - Implémenter `PythonAgentBridge` et `PythonToolBridge`
   - Tester l'exécution d'agents Python depuis Rust
   - Valider le round-trip JSON-RPC

2. **Phase 2 : Ponts moyenne priorité** (Provider, ContextEngine, MemoryStore, EventStore)
   - Implémenter les bridges restants
   - Tester l'interopérabilité

3. **Phase 3 : Ponts basse priorité** (ConfigSource, ExecPolicy)
   - Implémenter les bridges optionnels
   - Documentation complète

## 3. Travail déjà réalisé (à réutiliser)

### 3.1 De odysseusAgentOS (main branch)

| Composant | Statut | Réutilisable |
|-----------|--------|-------------|
| `core/atomic_io.py` | ✅ Fonctionnel | Oui (pattern d'I/O atomique) |
| `core/database.py` | ✅ Fonctionnel | Oui (pattern de DB) |
| `core/middleware.py` | ✅ Fonctionnel | Oui (pattern de middleware) |
| `src/agent_loop.py` | ✅ Fonctionnel | Oui (logique de boucle d'agent) |
| `src/agent_tools/` | ✅ Fonctionnel | Oui (100+ outils) |
| `src/ai_interaction.py` | ✅ Fonctionnel | Oui (pattern LLM) |
| `docs/master-ref/10-ARCHITECTURE-MODULAIRE.md` | ✅ Complet | Oui (analyse combinatoire) |
| `docs/master-ref/01-SFD-v3.1.md` | ✅ Complet | Oui (spécification) |

### 3.2 De omega (branche feat/agentos-omega)

| Composant | Statut | Réutilisable |
|-----------|--------|-------------|
| `omega-ports` (16+8 ports) | ✅ Fonctionnel | Oui (contrats) |
| `omega-core` (noyau) | ✅ Fonctionnel | Oui (logique Rust) |
| `omega-providers` | ✅ Fonctionnel | Oui (adaptateurs LLM) |
| `omega-context` | ✅ Fonctionnel | Oui (adaptateurs contexte) |
| `omega-router` | ✅ Fonctionnel | Oui (routing) |
| `omega-sandbox` | ✅ Fonctionnel | Oui (isolation) |
| `omega-server` | ✅ Fonctionnel | Oui (API) |
| `omega-app` | ✅ Fonctionnel | Oui (orchestration) |
| `omega-otel` | ✅ Fonctionnel | Oui (télémétrie) |
| `omega-harness` | ✅ Fonctionnel | Oui (tests) |

## 4. Plan d'action — Phase 13

### Étape 1 : Créer le bridge Rust (PythonBridge)

Créer un crate `omega-python-bridge` qui :
- Lance un subprocess Python
- Communique via JSON-RPC over stdin/stdout
- Gère timeout + retry + circuit breaker
- Implémente les ports omega comme des adaptateurs qui délèguent à Python

### Étape 2 : Implémenter les ponts haute priorité

1. `PythonAgentBridge` → `AgentLoop`
2. `PythonToolBridge` → `ToolExecutor`
3. `PythonMiddlewareBridge` → `Hook`

### Étape 3 : Tester l'interopérabilité

- Tests unitaires pour chaque pont
- Tests d'intégration Rust ↔ Python
- Validation dans le sandbox

### Étape 4 : Documenter

- ADR pour le bridge
- Guide d'utilisation
- Exemples de plugins Python

## 5. Prochaines actions

1. Créer `omega/crates/omega-python-bridge/`
2. Implémenter `PythonAgentBridge` (pont AgentLoop → agent_loop.py)
3. Implémenter `PythonToolBridge` (pont ToolExecutor → agent_tools/)
4. Tester l'interopérabilité
5. Commiter et avancer vers Phase 14
