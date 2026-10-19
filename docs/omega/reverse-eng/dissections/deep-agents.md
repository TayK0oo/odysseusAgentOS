# Dissection — Deep Agents (LangChain)

> `langchain-ai/deepagents` · **30 347 ★** · MIT · Python · clone du 2026-10-17.
> Référence : *« The batteries-included agent harness »*.

## 1. Fiche

| | |
|---|---|
| Éditeur | LangChain |
| Langage | Python (monorepo `libs/`) |
| Licence | MIT |
| Nature | **harnais** (pas un framework) : « extend, override, or replace any piece » |
| Preuve machine | `docs/omega/research/rea/evidence/intake-deep-agents.summary.json` |

## 2. Ce que c'est

Trois couches explicites (`libs/ARCHITECTURE.md:22-49`) :

```txt
Deep Agents   harnais opinionne : defauts, middleware, backends, profils
LangChain     abstraction agent : modele + outils + middleware -> boucle
LangGraph     runtime : etat, checkpoints, streaming, interrupts
```

Deep Agents **n'introduit pas de runtime** : `create_deep_agent()`
(`libs/deepagents/deepagents/graph.py`) assemble une **pile de middleware**, un
**backend**, des **sous-agents**, des **skills**, une **memoire** et des
**profils**. Le monorepo se decoupe en `libs/deepagents` (coeur), `libs/code`
(CLI/TUI/extensions/plugins/hooks), `libs/acp` (editeurs), `libs/talon`,
`libs/evals`, `libs/partners`.

## 3. Inventaire (18 fonctionnalites)

| # | Fonctionnalite | Capacite Omega | Statut | Cout |
|---|---|---|---|---|
| 1 | Pile de middleware en 3 couches | `registry` | adopted | S |
| 2 | Sous-agents a contexte isole | `subagent-runner` | planned | M |
| 3 | Sous-agents asynchrones / distants | `subagent-runner` | planned | L |
| 4 | Outils de fichiers (ls/read/write/edit/grep/glob) | `tool-surface` | adopted | M |
| 5 | Backends interchangeables (8 implementations) | `sandbox-backend` | adopted | M |
| 6 | Resume de fil long + offload sur disque | `context-engine` | adopted | M |
| 7 | Memoire persistante pluggable | `memory-store` | planned | M |
| 8 | Skills (SKILL.md) + outils de chargement | `skill-store` | adopted | S |
| 9 | Human-in-the-loop (approuver/editer/rejeter) | `permission-policy` | adopted | S |
| 10 | Prompt caching par middleware | `prompt-cache-policy` | planned | M |
| 11 | Eviction de messages + rognage de debordement | `context-engine` | adopted | S |
| 12 | Auto-evaluation par rubrique | `router-feedback` | planned | M |
| 13 | Reparation des appels d'outils malformes | `tool-surface` | planned | M |
| 14 | Profils fournisseur / harnais | `config-source` | adopted | S |
| 15 | Plugins, hooks et extensions de la CLI | `hook` | planned | M |
| 16 | Fournisseurs MCP pre-integres | `tool` | adopted | S |
| 17 | Agent Client Protocol (editeurs) | `channel` | planned | L |
| 18 | TUI du harnais de code | `ui-panel` | planned | L |

## 4. Les apports les plus forts pour Omega

### 4.1 Le backend unique pour fichiers + memoire + shell

`BackendProtocol` (`backends/protocol.py:404`) est implemente **huit fois** :
`composite`, `filesystem`, `local_shell`, `sandbox`, `state`, `store`,
`langsmith`, `context_hub`. Chaque operation a **son type de resultat** —
`ReadResult:204`, `WriteResult:278`, `EditResult:295`, `GrepResult:344`,
`GlobResult:382` — jamais un dictionnaire libre.

**A reprendre** : notre `SandboxBackend` ne couvre que l'execution. Ici, le
**meme** contrat sert au systeme de fichiers, a la memoire et au shell : changer
de backend deplace tout d'un coup. C'est exactement la « couture » de
`NOYAU-ULTIME.md` §2 (`ctx.fs` + `ctx.exec` + `ctx.memory` derriere une seule clé).

### 4.2 Trois mecanismes de contexte, pas un

`summarization.py` (resume), `_message_eviction.py` (eviction),
`_overflow_clip.py` (rognage) sont **trois middlewares separes**, chacun avec son
seuil. Plus `_blob_offload.py` : les grosses sorties d'outils partent **sur
disque** au lieu de rester dans la fenetre.

**A reprendre** : notre `ContextBudget` fait de la troncature a marqueur — c'est
le plus faible des quatre mecanismes. La combinaison offload + resume est ce qui
tient un run long.

### 4.3 La reparation des appels d'outils

`PatchToolCallsMiddleware` (`middleware/patch_tool_calls.py:11`) **repare** les
appels malformes au lieu de les rejeter. Meme philosophie que le reparateur de
hunks de GPT Engineer, appliquee aux appels d'outils.

**A reprendre** : indispensable avec les modeles **locaux**, qui emettent
frequemment du JSON imparfait. C'est un prerequis de notre pilier self-hosted.

### 4.4 Le prompt caching en bout de pile

`_prompt_caching.py` est un middleware **de queue** (`ARCHITECTURE.md:56-63`) :
il s'applique apres les exclusions d'outils et le prompt final. Un reglage de
fournisseur ne peut pas faire cela.

## 5. Ecarte

| Element | Raison |
|---|---|
| Dependance a `langchain-core` / `LangGraph` | notre noyau Rust a ses propres ports ; seuls les concepts sont repris |
| LangSmith (tracing/deploiement) | service editeur, hors perimetre self-hosted |
| `libs/talon`, `libs/evals`, `libs/partners` | outils internes / integrations tierces, sans transfert |

## 6. Provenance

- Toutes les affirmations viennent du clone (`fichier:ligne` dans le JSON).
- La preuve REA (`intake-deep-agents.summary.json`) ne corrobore que l'identite
  (depot, 30 347 ★, Python, MIT, clone) : `confidence: derived`, analyse
  JavaScript sur un depot **Python** — elle ne valide aucune fonctionnalite.
- Dissection realisee par l'orchestrateur (les modeles gratuits etaient throttles
  par rate limit) ; refs verifiees par lecture directe du clone.
