# Balayage REA — REA applique a nos artefacts

> Genere par `omega/scripts/rea-sweep.py` (regenerable). Preuve complete hors depot ; resumes ci-dessous.

| Cible | Sujet | Statut | Provider | Confiance | Evidence | Limites |
|---|---|---|---|---|---|---|
| `ui-dist` | Cockpit React builde (JS/Electron-style) | ✅ analysee | `rea-javascript-application` | derived | `ev_0d8d614b1558fdd5e582b…` | 5 |
| `omega-crate-elf` | Binaire ELF de nos crates (omega_types-87bafd0760c1235a) | ✅ analysee | `pwntools-elf` | observed | `ev_c779a8e39c3e2af2250d9…` | 12 |
| `repo-overview` | Vue d'ensemble des sources du cockpit (omega/ui/src) | ✅ analysee | `rea-javascript-application` | derived | `ev_d7e98cf71465ab7291ebf…` | 5 |

## Detail

### `ui-dist` — Cockpit React builde (JS/Electron-style)

- sujet : `/home/tayk0oo/Projets/odysseusAgentOS-omega/omega/ui/dist`
- provider : `rea-javascript-application` (1)
- confiance : **derived** ; operation : `analyze_javascript_application` ; autorite : `shipped-artifact`
- evidence : `ev_0d8d614b1558fdd5e582b473ff1c0622f836fcab9bfae56f2279dfa49b461385`
- digest : `{'sha256': 'abe504e64a289ffb49a7fc159b5bbbc7a52bb5e5fd326b9404427afd1f4194ab'}`
- structure : graph_edges=6, graph_nodes=5
- limites declarees par REA :
  - CommonJS and ESM binding relationships were recovered from inert syntax and resolved only within the inventoried artifact container.
  - JavaScript and HTML were parsed as inert text; bundle bootstrap code was never executed.
  - Static imports, entrypoints, workers, endpoints, and storage relationships do not prove runtime execution.
  - Static paths and relationships may remain unresolved when expressions are dynamic or obfuscated.
  - Webpack/Rspack factories were recovered from AST literals; REA did not invoke push handlers or bundle bootstrap code.

### `omega-crate-elf` — Binaire ELF de nos crates (omega_types-87bafd0760c1235a)

- sujet : `/home/tayk0oo/Projets/odysseusAgentOS-omega/omega/target/debug/deps/omega_types-87bafd0760c1235a`
- provider : `pwntools-elf` (pwntools@4.15.0;pyelftools@0.33;unicorn@2.1.2)
- confiance : **observed** ; operation : `inspect_binary_layout` ; autorite : `shipped-artifact`
- evidence : `ev_c779a8e39c3e2af2250d9f46d1025982438226135ec6ad8a0173e5ac39222b35`
- digest : `{'sha256': 'b4f35dd1fcfbc69905ebc37092f68103afd348e95c29169e87a110fca562f7af'}`
- structure : relocations=2349, sections=44, segments=12, symbols=2248
- limites declarees par REA :
  - PT_NULL payload fields are unused and retain their reported numbers without file-byte or permission claims. Zero-file-size segments provide no file bytes.
  - SHN_XINDEX symbol values remain unresolved in this upstream representation; an external index table does not establish a resolved index in this report.
  - Reported values preserve linked addresses, section offsets, TLS offsets, alignment and absolute values separately. Runtime load base and library paths are unkno
  - Names are display strings plus raw bytes and file ranges where resolvable; section/table/entry indices preserve identity and duplicates. Name ranges include the
  - Ordinary relocation rows cover REL/RELA. RELR tables retain complete encoded bytes and upstream-decoded offsets as derived evidence; per-offset packed-word loca
  - Relocation symbol index zero uses a zero symbol value without a table lookup. Positive indices reference validated original symbol-table entries; raw table and 
  - Relocatable relocation targets with SHN_UNDEF or inactive SHT_NULL headers retain their reported section index/offset as unknown; they do not establish an assoc
  - GOT/PLT are derived upstream convenience maps; PLT inference can emulate selected instructions in Unicorn. They do not establish target runtime behavior; aliase

### `repo-overview` — Vue d'ensemble des sources du cockpit (omega/ui/src)

- sujet : `/home/tayk0oo/Projets/odysseusAgentOS-omega/omega/ui/src`
- provider : `rea-javascript-application` (1)
- confiance : **derived** ; operation : `analyze_javascript_application` ; autorite : `shipped-artifact`
- evidence : `ev_d7e98cf71465ab7291ebf0b97ce865f4f4bea816fdaedf18e6898c242fae2f0f`
- digest : `{'sha256': 'ed95c6d5c0db3c326425528d58dea881bcd13827ebc7d563fb7a183cd1561051'}`
- structure : graph_edges=125, graph_nodes=84
- limites declarees par REA :
  - CommonJS and ESM binding relationships were recovered from inert syntax and resolved only within the inventoried artifact container.
  - JavaScript and HTML were parsed as inert text; bundle bootstrap code was never executed.
  - Static imports, entrypoints, workers, endpoints, and storage relationships do not prove runtime execution.
  - Static paths and relationships may remain unresolved when expressions are dynamic or obfuscated.
  - Webpack/Rspack factories were recovered from AST literals; REA did not invoke push handlers or bundle bootstrap code.

