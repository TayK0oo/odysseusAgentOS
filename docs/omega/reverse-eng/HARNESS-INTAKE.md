# Intake des harnais — clone + REA (popularite mesuree)

> Genere par `omega/scripts/harness_intake.py`. Analyse profonde REA pour le JS/TS ; vue d'ensemble sinon (REA n'a pas d'analyseur de source Python/Rust/Go).

| # | Harnais | Etoiles | Nature | Clone | REA | Provider | Limites |
|---|---|---|---|---|---|---|---|
| 1 | AutoGPT | 187,614 | other | ok | ✅ analysee | rea-javascript-application | 6294 |
| 2 | Gemini CLI | 107,412 | js | ok | ✅ analysee | rea-javascript-application | 3666 |
| 3 | MetaGPT | 70,887 | python | ok | ✅ analysee | rea-javascript-application | 7 |
| 4 | CrewAI | 59,775 | python | ok | ✅ analysee | rea-javascript-application | 5 |
| 5 | GPT Engineer | 55,024 | python | ok | ✅ analysee | rea-javascript-application | 5 |
| 6 | LangGraph | 43,380 | other | ok | ✅ analysee | rea-javascript-application | 6 |
| 7 | AgentGPT | 36,279 | other | ok | ✅ analysee | rea-javascript-application | 6 |
| 8 | Continue | 36,235 | js | ok | ✅ analysee | rea-javascript-application | 2215 |
| 9 | Tabby | 33,917 | js | ok | ✅ analysee | rea-javascript-application | 650 |
| 10 | GPT Pilot | 33,654 | python | ok | ✅ analysee | rea-javascript-application | 7 |
| 11 | Void | 28,750 | js | ok | ⚠️ indisponible | — | — |
| 12 | Crush | 28,674 | go | ok | ✅ analysee | rea-javascript-application | 8 |
| 13 | Qwen Code | 28,528 | js | ok | ⚠️ indisponible | — | — |
| 14 | Kilo Code | 27,627 | js | ok | ✅ analysee | rea-javascript-application | 11687 |

## Detail

### AutoGPT (187,614 etoiles) — `Significant-Gravitas/AutoGPT`

- nature : other ; licence : NOASSERTION
- evidence : `ev_7f447ff1df9072a3b2bcb2c62dfc12945475275fd123c19d2b177c08c562615b`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=15594, graph_nodes=14136

### Gemini CLI (107,412 etoiles) — `google-gemini/gemini-cli`

- nature : js ; licence : Apache-2.0
- evidence : `ev_2312eb67ed7303ab0eec8e5f62203267e51a2da171c1a18ffb173615faff70fa`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=13543, graph_nodes=9177

### MetaGPT (70,887 etoiles) — `geekan/MetaGPT`

- nature : python ; licence : MIT
- evidence : `ev_f9dc49e5c2c417c4655a654920d786ed112e051091828090741bb3f5648c82d9`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=325, graph_nodes=249

### CrewAI (59,775 etoiles) — `crewAIInc/crewAI`

- nature : python ; licence : MIT
- evidence : `ev_bcbccb43319cf02289440b3f5bb7bf6bc12b89c872a5efd39405861433afe076`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=28, graph_nodes=26

### GPT Engineer (55,024 etoiles) — `AntonOsika/gpt-engineer`

- nature : python ; licence : MIT
- evidence : `ev_18307aac2fe0fd68dbcc088c75af9262b799510742b8f4f9281b0374aefdbb02`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=2, graph_nodes=3

### LangGraph (43,380 etoiles) — `langchain-ai/langgraph`

- nature : other ; licence : MIT
- evidence : `ev_8abbd5145b340eddcaac9d33362587b0cfc0cc017c222890b25cb62e8a56a3db`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=109, graph_nodes=89

### AgentGPT (36,279 etoiles) — `reworkd/AgentGPT`

- nature : other ; licence : GPL-3.0
- evidence : `ev_e59053561ead5707984c331c3a32af0483a704d5a382e2b5facdd84bed92c372`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=2055, graph_nodes=1854

### Continue (36,235 etoiles) — `continuedev/continue`

- nature : js ; licence : Apache-2.0
- evidence : `ev_0d9306c4cfcaf5133cc7147645b7c8dda05f6cadcbb12d0e6d66d3d47e9ccb1b`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=14639, graph_nodes=11622

### Tabby (33,917 etoiles) — `TabbyML/tabby`

- nature : js ; licence : NOASSERTION
- evidence : `ev_e1927731304e0df8c4cfed70ab9a6b39dfcc58fa2b28515d24d513defddb3377`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=10307, graph_nodes=8144

### GPT Pilot (33,654 etoiles) — `Pythagora-io/gpt-pilot`

- nature : python ; licence : NOASSERTION
- evidence : `ev_1bd564912823baa1484147b7be3bdb5bb94693f93881b52c19a374faa6fd5e13`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=784, graph_nodes=614

### Void (28,750 etoiles) — `voideditor/void`

- nature : js ; licence : Apache-2.0
- **unavailable** : resource_constraint The parent's result capacity was exhausted after retaining static facts for src/vs/workbench/contrib/notebook/browser/contrib/find/notebookFindReplaceWidget.ts; remaining sources were not analyzed

### Crush (28,674 etoiles) — `charmbracelet/crush`

- nature : go ; licence : NOASSERTION
- evidence : `ev_9001047720c19566671aac242ac0c757e35243c01d9261f851a25d55c54c1bdb`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=15, graph_nodes=15

### Qwen Code (28,528 etoiles) — `QwenLM/qwen-code`

- nature : js ; licence : Apache-2.0
- **unavailable** : resource_constraint The parent's result capacity was exhausted after retaining static facts for packages/live-host/src/preload/audio-engine.ts; remaining sources were not analyzed

### Kilo Code (27,627 etoiles) — `Kilo-Org/kilocode`

- nature : js ; licence : MIT
- evidence : `ev_1d445d65dad83b6b19f2d566af78a39f3db74814c24e45df136a1108c0532da5`
- provider : `rea-javascript-application` ; confiance : **derived**
- structure : graph_edges=26397, graph_nodes=24843

