# Traçabilité SFD v3.0 ↔ Code — vérifiable

*Généré: 2026-09-25 · branche feat/inventaire-global-v1 · 35 kill-switches inventoriés*

## 1. Modules SFD (§5.1→§5.20) → implémentation (existence vérifiée)

| Module SFD | Fichiers présents | Kill-switch (défaut) | Tests~ |
|---|---|---|---|
| §5.1 Décision multi-agent & orchestration | 4/4 · ✅ | ODYSSEUS_LIVE_ORCHESTRATION=off, ODYSSEUS_PHASE_TRACKER=off | 14 |
| §5.2 Ingénierie du contexte | 3/3 · ✅ | — | 8 |
| §5.3 Planification intelligente | 2/2 · ✅ | — | 0 |
| §5.4 Exécution contrôlée/sécurisée | 3/3 · ✅ | ODYSSEUS_DESTRUCTIVE_GATE=on | 0 |
| §5.5 Exécution durable | 2/2 · ✅ | ODYSSEUS_DURABLE_EXEC=? | 0 |
| §5.6 Routage modèles | 3/3 · ✅ | ODYSSEUS_MODEL_ROUTER=off, ODYSSEUS_ZEN_FROM_ENDPOINT=off | 0 |
| §5.7 Mémoire boucle fermée | 5/5 · ✅ | ODYSSEUS_MEMORY_PROVENANCE=?, ODYSSEUS_OBSIDIAN_MCP=off | 0 |
| §5.8 Communication multi-canal | 2/2 · ✅ | ODYSSEUS_INPROCESS_DISCORD=off, ODYSSEUS_INPROCESS_TELEGRAM=off, ODYSSEUS_CHANNEL_AGENT_REPLY=off | 0 |
| §5.9 Gouvernance & budgets | 3/3 · ✅ | ODYSSEUS_GOVERNANCE_ANCESTRY=off | 5 |
| §5.10 Auto-évaluation & qualité | 1/1 · ✅ | ODYSSEUS_AUTOEVAL=off, ODYSSEUS_DEEPEVAL=off | 0 |
| §5.11 Observabilité | 3/3 · ✅ | ODYSSEUS_LANGFUSE=off, ODYSSEUS_CODEBURN=off | 0 |
| §5.12 Workspaces & worktrees | natif OpenCode | — | 0 |
| §5.13 Extensibilité & MCP | 4/4 · ✅ | ODYSSEUS_DISABLE_MCP=off, ODYSSEUS_CBM=off, ODYSSEUS_GRAPHIFY=off, ODYSSEUS_SERENA_MCP=off | 13 |
| §5.14 Config intégrale versionnée | 2/2 · ✅ | — | 0 |
| §5.15 Préférences utilisateur | 2/2 · ✅ | ODYSSEUS_PREFERENCES=? | 0 |
| §5.16 Recherche conversations | 1/1 · ✅ | — | 0 |
| §5.17 Système de skills | 3/3 · ✅ | — | 16 |
| §5.18 Sortie visuelle | 3/3 · ✅ | ODYSSEUS_VISUAL_OUTPUT=? | 0 |
| §5.19 Classification & rétention | 2/2 · ✅ | ODYSSEUS_DATA_CLASSIFICATION=? | 4 |
| §5.20 Sécurité des contenus | 4/4 · ✅ | ODYSSEUS_CONTENT_SECURITY=? | 0 |

## 2. Kill-switches (registre vérifiable)

| Catégorie | Env var | Défaut | Source |
|---|---|---|---|
| Agents | `ODYSSEUS_AGENT_CATALOG` | off | app.py:1254 |
| Channels | `ODYSSEUS_CHANNEL_AGENT_REPLY` | off | src/channel_bootstrap.py:59 |
| Channels | `ODYSSEUS_INPROCESS_DISCORD` | off | src/channel_bootstrap.py:45 |
| Channels | `ODYSSEUS_INPROCESS_TELEGRAM` | off | src/channel_bootstrap.py:50 |
| Code Parsing | `ODYSSEUS_TREESITTER` | off | services/code/treesitter_parser.py |
| Document Processing | `ODYSSEUS_DOCLING` | off | src/docling_runtime.py |
| Governance/Memory | `ODYSSEUS_AUTOEVAL` | off | src/orchestrator/autoeval.py:44 |
| Governance/Memory | `ODYSSEUS_AUTOEVOLVE` | off | src/orchestrator/autoevolve.py:18 |
| Governance/Memory | `ODYSSEUS_CHECKPOINT` | off | src/orchestrator/checkpoint_tracker.py:39 |
| Governance/Memory | `ODYSSEUS_CODEBURN` | off | src/orchestrator/codeburn_runner.py:26 |
| Governance/Memory | `ODYSSEUS_GOVERNANCE_ANCESTRY` | off | src/orchestrator/ancestry_tracker.py:26 |
| Governance/Memory | `ODYSSEUS_LANGFUSE` | off | services/observability/langfuse_tracer.py:24 |
| Governance/Memory | `ODYSSEUS_MEMORY_IMPACT` | off | src/memory_impact.py:29 |
| Governance/Memory | `ODYSSEUS_PROGRESSIVE_DISCLOSURE` | off | src/progressive_disclosure.py:36 |
| Governance/Memory | `ODYSSEUS_UNIFIED_TOKENS` | off | src/trace_writer.py:146 |
| MCP/Services | `ODYSSEUS_AGENTSEAL` | off | src/agentseal_runner.py:85 |
| MCP/Services | `ODYSSEUS_BROWSER_HARNESS` | off | src/agent_tools/web_tools.py |
| MCP/Services | `ODYSSEUS_CBM` | off | src/cbm_client.py:17 |
| MCP/Services | `ODYSSEUS_DISABLE_MCP` | off | src/builtin_mcp.py:89 |
| MCP/Services | `ODYSSEUS_GRAPHIFY` | off | src/builtin_mcp.py:107 |
| MCP/Services | `ODYSSEUS_N8N` | off | routes/n8n_routes.py |
| MCP/Services | `ODYSSEUS_OBSIDIAN_MCP` | off | src/builtin_mcp.py:98 |
| MCP/Services | `ODYSSEUS_PLAYWRIGHT` | off | src/builtin_mcp.py:81 |
| MCP/Services | `ODYSSEUS_SERENA_MCP` | off | src/serena_client.py:19 |
| MCP/Services | `ODYSSEUS_SUPABASE` | off | src/integrations.py |
| MCP/Services | `ODYSSEUS_VAULTWARDEN` | off | src/tools/vault.py |
| MCP/Services | `ODYSSEUS_ZEN_FROM_ENDPOINT` | off | src/zen_router.py:131 |
| Orchestration | `ODYSSEUS_DESTRUCTIVE_GATE` | on | src/orchestrator/gate.py:26 |
| Orchestration | `ODYSSEUS_LANGGRAPH` | off | src/orchestrator/langgraph_loop.py:33 |
| Orchestration | `ODYSSEUS_LANGGRAPH_INTERRUPT` | on | src/orchestrator/langgraph_loop.py:441 |
| Orchestration | `ODYSSEUS_LIVE_ORCHESTRATION` | off | src/agent_loop.py:2493 |
| Orchestration | `ODYSSEUS_MODEL_ROUTER` | off | src/orchestrator/router_advice.py:42 |
| Orchestration | `ODYSSEUS_PHASE_TRACKER` | off | src/orchestrator/phase_tracker.py:25 |
| Quality | `ODYSSEUS_DEEPEVAL` | off | tests/quality/test_llm_quality.py |
| RAG | `ODYSSEUS_RRF_FUSION` | off | src/rag_vector.py:50 |

**35 switches · 2 ON par défaut · 33 OFF (dormants)**

ON: `ODYSSEUS_DESTRUCTIVE_GATE`, `ODYSSEUS_LANGGRAPH_INTERRUPT`
