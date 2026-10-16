# Balayage REA — REA applique aux harnais clones (JS/TS)

> Genere par `omega/scripts/rea-sweep.py` (regenerable). Preuve complete hors depot ; resumes ci-dessous.

| Cible | Sujet | Statut | Provider | Confiance | Evidence | Limites |
|---|---|---|---|---|---|---|
| `harness-cline` | Harnais clone : cline (JS/TS) | ✅ analysee | `rea-javascript-application` | derived | `ev_7722f930a2c89a6fa80b0…` | 5567 |
| `harness-codex` | Harnais clone : codex (JS/TS) | ✅ analysee | `rea-javascript-application` | derived | `ev_63e3def413ca0e700c6bd…` | 9 |
| `harness-open-interpreter` | Harnais clone : open-interpreter (JS/TS) | ✅ analysee | `rea-javascript-application` | derived | `ev_510526db0cfa4f0a149d1…` | 9 |
| `harness-openclaw` | Harnais clone : openclaw (JS/TS) | ⚠️ indisponible | — | — | resource_constraint | — |
| `harness-opencode` | Harnais clone : opencode (JS/TS) | ✅ analysee | `rea-javascript-application` | derived | `ev_3c24f145513dd24d30a17…` | 9466 |
| `harness-openhands` | Harnais clone : openhands (JS/TS) | ✅ analysee | `rea-javascript-application` | derived | `ev_33c0029805cfdb5178c3b…` | 4025 |

## Detail

### `harness-cline` — Harnais clone : cline (JS/TS)

- sujet : `/home/tayk0oo/omega-re/cline`
- provider : `rea-javascript-application` (1)
- confiance : **derived** ; operation : `analyze_javascript_application` ; autorite : `shipped-artifact`
- evidence : `ev_7722f930a2c89a6fa80b08e4ded9ec03d074c037c9b67e4d2ee351ecdf00d5c4`
- digest : `{'sha256': '5795cb7f4bafda12742d463230cf8bb90078264f77a6988baa9ddccda55a9ba7'}`
- structure : graph_edges=19281, graph_nodes=16001
- limites declarees par REA :
  - CommonJS and ESM binding relationships were recovered from inert syntax and resolved only within the inventoried artifact container.
  - Graph coverage is incomplete; unavailable facts remain explicit and must not be read as absence.
  - JavaScript and HTML were parsed as inert text; bundle bootstrap code was never executed.
  - Resource constraint during analyze_javascript_application: Expanded application relationships for apps/examples/desktop-app/sidecar/composio.ts exceed the paren
  - Resource constraint during analyze_javascript_application: Expanded application relationships for apps/examples/desktop-app/sidecar/connectors.test.ts exceed th
  - Resource constraint during analyze_javascript_application: Expanded application relationships for apps/examples/desktop-app/sidecar/connectors.ts exceed the par
  - Resource constraint during analyze_javascript_application: Expanded application relationships for apps/examples/desktop-app/sidecar/context.test.ts exceed the p
  - Resource constraint during analyze_javascript_application: Expanded application relationships for apps/examples/desktop-app/sidecar/context.ts exceed the parent

### `harness-codex` — Harnais clone : codex (JS/TS)

- sujet : `/home/tayk0oo/omega-re/codex`
- provider : `rea-javascript-application` (1)
- confiance : **derived** ; operation : `analyze_javascript_application` ; autorite : `shipped-artifact`
- evidence : `ev_63e3def413ca0e700c6bdab045e3de22d9d8d9b0b1120d41e212c584d9f048d7`
- digest : `{'sha256': 'a101e99be761172662a35a80f65078b59e08ba43e79188c73d9afcf7aa3ae1c4'}`
- structure : graph_edges=2211, graph_nodes=2150
- limites declarees par REA :
  - CommonJS and ESM binding relationships were recovered from inert syntax and resolved only within the inventoried artifact container.
  - Graph coverage is incomplete; unavailable facts remain explicit and must not be read as absence.
  - JavaScript and HTML were parsed as inert text; bundle bootstrap code was never executed.
  - Static imports, entrypoints, workers, endpoints, and storage relationships do not prove runtime execution.
  - Static paths and relationships may remain unresolved when expressions are dynamic or obfuscated.
  - Webpack/Rspack factories were recovered from AST literals; REA did not invoke push handlers or bundle bootstrap code.
  - codex-rs/login/src/assets/success.html has 1 inline JavaScript body starting at line 199 that was not analyzed; endpoints, storage, imports and other facts in i
  - codex-rs/login/src/assets/success_legacy.html has 1 inline JavaScript body starting at line 199 that was not analyzed; endpoints, storage, imports and other fac

### `harness-open-interpreter` — Harnais clone : open-interpreter (JS/TS)

- sujet : `/home/tayk0oo/omega-re/open-interpreter`
- provider : `rea-javascript-application` (1)
- confiance : **derived** ; operation : `analyze_javascript_application` ; autorite : `shipped-artifact`
- evidence : `ev_510526db0cfa4f0a149d18189664b9ff4840c7469d43abded72787ec84ecad07`
- digest : `{'sha256': 'e6d88e9b93c1d27516c25e22837cd3fa81599eacdf422913d3ca446c434db942'}`
- structure : graph_edges=2268, graph_nodes=2195
- limites declarees par REA :
  - CommonJS and ESM binding relationships were recovered from inert syntax and resolved only within the inventoried artifact container.
  - Graph coverage is incomplete; unavailable facts remain explicit and must not be read as absence.
  - JavaScript and HTML were parsed as inert text; bundle bootstrap code was never executed.
  - Static imports, entrypoints, workers, endpoints, and storage relationships do not prove runtime execution.
  - Static paths and relationships may remain unresolved when expressions are dynamic or obfuscated.
  - Webpack/Rspack factories were recovered from AST literals; REA did not invoke push handlers or bundle bootstrap code.
  - codex-rs/login/src/assets/success.html has 1 inline JavaScript body starting at line 188 that was not analyzed; endpoints, storage, imports and other facts in i
  - codex-rs/login/src/assets/success_legacy.html has 1 inline JavaScript body starting at line 158 that was not analyzed; endpoints, storage, imports and other fac

### `harness-openclaw` — Harnais clone : openclaw (JS/TS)

- sujet : `/home/tayk0oo/omega-re/openclaw`
- **indisponible** : `resource_constraint` — The parent's result capacity was exhausted after retaining static facts for extensions/memory-core/src/tools.runtime.ts; remaining sources were not analyzed
- remediation : Review the reported worker memory limits and available host memory. Retry with sufficient memory or a smaller artifact; REA retains tighter inherited limits.

### `harness-opencode` — Harnais clone : opencode (JS/TS)

- sujet : `/home/tayk0oo/omega-re/opencode`
- provider : `rea-javascript-application` (1)
- confiance : **derived** ; operation : `analyze_javascript_application` ; autorite : `shipped-artifact`
- evidence : `ev_3c24f145513dd24d30a17c30a683aac8b85fc72595ba75fabca33f655128db12`
- digest : `{'sha256': 'fb58d1ef2c39c588ad628ae7d7b0ad0f6851f2055a90e14d196a3987c20bde26'}`
- structure : graph_edges=22669, graph_nodes=17853
- limites declarees par REA :
  - CommonJS and ESM binding relationships were recovered from inert syntax and resolved only within the inventoried artifact container.
  - Dynamic IPC channel expressions remain unresolved and are never paired by textual similarity.
  - Electron API identity follows proven lexical module aliases and structurally verified module-namespace interop. Other reassignments, dynamic selections, and vis
  - Electron relationships are derived from inert syntax; runtime registration, reachability, defaults, and enforcement remain unproven.
  - Graph coverage is incomplete; unavailable facts remain explicit and must not be read as absence.
  - JavaScript and HTML were parsed as inert text; bundle bootstrap code was never executed.
  - Resource constraint during analyze_javascript_application: Expanded application relationships for packages/ai/test/provider/openai-responses-images.recorded.tes
  - Resource constraint during analyze_javascript_application: Expanded application relationships for packages/ai/test/provider/openai-responses-websocket.recorded.

### `harness-openhands` — Harnais clone : openhands (JS/TS)

- sujet : `/home/tayk0oo/omega-re/openhands`
- provider : `rea-javascript-application` (1)
- confiance : **derived** ; operation : `analyze_javascript_application` ; autorite : `shipped-artifact`
- evidence : `ev_33c0029805cfdb5178c3b864fde965b655d8efcb54f27316fda6aab3073dc873`
- digest : `{'sha256': '0f2dd53a91c0bd251c974bc568750f619c6b15531e09123cd91a407cd4c617d9'}`
- structure : graph_edges=12430, graph_nodes=10233
- limites declarees par REA :
  - CommonJS and ESM binding relationships were recovered from inert syntax and resolved only within the inventoried artifact container.
  - Electron API identity follows proven lexical module aliases and structurally verified module-namespace interop. Other reassignments, dynamic selections, and vis
  - Electron relationships are derived from inert syntax; runtime registration, reachability, defaults, and enforcement remain unproven.
  - Graph coverage is incomplete; unavailable facts remain explicit and must not be read as absence.
  - JavaScript and HTML were parsed as inert text; bundle bootstrap code was never executed.
  - Resource constraint during analyze_javascript_application: Expanded application relationships for __tests__/components/features/home/featured-automations-sectio
  - Resource constraint during analyze_javascript_application: Expanded application relationships for __tests__/components/features/home/git-repo-dropdown.test.tsx 
  - Resource constraint during analyze_javascript_application: Expanded application relationships for __tests__/components/features/home/home-chat-launcher-failed-l

