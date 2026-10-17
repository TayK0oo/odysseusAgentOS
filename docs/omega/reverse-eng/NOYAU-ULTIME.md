# NOYAU ULTIME — architecture cible de notre système

> Synthèse du corpus reverse-engineering (`OMEGA-COMPLET.md`, 32 harnais,
> 683 fonctionnalités) appliquée à **notre** noyau. Générée après la dissection
> de **DeepSeek Harness** (`deepseek-ai/deepseek-harness`, **251 234 ★**,
> *« Everything is a Plugin »*) — la validation externe de notre doctrine.

## 0. Doctrine (déjà la nôtre, désormais prouvée à 251 k★)

**Aucun cœur privilégié.** Boucle d'agent, registre d'outils, journal de session,
adaptateur de modèle, sandbox : **tout est plugin**. Rien n'est « câblé » dans un
main qu'on patche. `docs/architecture.md:11` de DeepSeek Harness le dit
littéralement : *« There is no privileged core to patch »*.

Notre équivalent existe déjà (workspace de 16 crates, `omega-registry` +
`omega-plugin-api` + ports) — mais il lui manque **cinq mécanismes** que le
corpus désigne comme structurels. Ils sont ci-dessous.

## 1. Les 7 règles de conception (déduites du corpus)

| # | Règle | Source dans le corpus | Ce que ça évite |
|---|---|---|---|
| 1 | **Effets réversibles** — toute inscription (registre, hook, timer, watcher) retourne un *disposer* | DeepSeek Harness `ctx.effect` → disposer | fuites au unload, état fantôme après reload |
| 2 | **Couches nommées** — toute valeur de config vient d'une couche identifiable, traçable jusqu'à sa source | DeepSeek Harness `--dump-config` nomme la couche ; Gemini CLI (5 scopes `precedence()`) | « d'où vient cette valeur ? » impossible à répondre |
| 3 | **Reload atomique** — un plugin se recharge entier ; les écritures de config sont sérialisées avec les reloads | DeepSeek Harness (file sérialisée) | config à moitié appliquée, plugin à moitié rechargé |
| 4 | **Guard monotone** — les politiques ne peuvent que **restreindre**, jamais élargir | DeepSeek Harness (pas de variante `Allow`) | escalade de privilèges par composition |
| 5 | **Fail-closed** — à chaque couture, l'absence de canal/provider/décision = **refus** | DeepSeek Harness ; Codex (`HookResult::FailedAbort`) | exécution silencieuse sans politique |
| 6 | **Couture explicite** — chaque sous-système interchangeable est relié à une **clé stable** ; échanger le provider déplace tout le sous-système d'un coup | DeepSeek Harness `docs/capability-seams.md` ; Void (registre de providers) | 15 `if/else` de fournisseurs en dur (défaut constaté chez Void) |
| 7 | **Matrice d'événements vérifiée** — producteurs et consommateurs déclarés et contrôlés automatiquement | DeepSeek Harness (matrice générée) | événements orphelins, hooks jamais appelés |

## 2. Le modèle de coutures appliqué à nos crates

| Couture (clé stable) | Service | Providers possibles | Notre état |
|---|---|---|---|
| `ctx.models` | inférence | Ollama (local), OpenAI-compat, MCP-relay | ✅ `Provider` + registre |
| `ctx.tools` | surface d'outils | natifs, **MCP** (`omega-mcp`) | ✅ `ToolExecutor` + `McpToolSurface` |
| `ctx.subagents` | délégation | spawn local, fork worktree, **Claude Code/Codex/ACP en providers externes** | ⚠️ port `SubagentRunner` **sans adaptateur** |
| `ctx.fs` | accès fichiers | direct, sandboxé | ⚠️ partiel |
| `ctx.exec` | exécution | local, sandbox (bubblewrap/seatbelt) | ⚠️ `LocalSandboxBackend` = `sh -c` + timeout |
| `ctx.events` | journal | fichier (`FileEventStore`), distant | ✅ |
| `ctx.provenance` | preuves | OTel, REA (`Evidence` + limites) | ✅ |
| `ctx.context` | contexte | budget, BM25, compaction | ✅ |
| `ctx.memory` | mémoire | — | ⚠️ port `MemoryStore` **sans adaptateur** |
| `ctx.hooks` | cycle de vie | — | ⚠️ port `Hook` **sans adaptateur** |

**Trois ports orphelins** (déclarés, jamais implémentés) : `SubagentRunner`,
`MemoryStore`, `Hook`. C'est le premier chantier du noyau.

## 3. Ce qui manque à notre noyau (écart mesuré)

| Manque | Impact | Coût | Source de référence |
|---|---|---|---|
| **Effets réversibles** dans `omega-registry` | impossible de décharger/recharger un plugin proprement | S | DeepSeek Harness `ctx.effect` |
| **Couches nommées** dans `omega-config` | valeurs non traçables | S | Gemini CLI 5 scopes ; DeepSeek Harness `--dump-config` |
| **Reload atomique** | hot-reload dangereux | M | DeepSeek Harness |
| **Guard monotone** dans `PermissionPolicy` | composition de politiques non sûre | S | DeepSeek Harness |
| **Matrice d'événements** | hooks/événements non vérifiés | S | DeepSeek Harness |
| **Adaptateurs des 3 ports orphelins** | mémoire, hooks, sous-agents absents | M | Kilo Code (memory), Codex (hooks), DeepSeek Harness (subagents) |
| **Sandbox réel** (namespace/FS/network) | `sh -c` + timeout n'isole pas | M | Kilo Code (bubblewrap/seatbelt + proxy egress) |
| **Compaction LLM** | on n'a que la troncature à marqueur | M | Qwen Code (double couche), Gemini CLI |

## 4. Plan d'implémentation (ordre imposé par le corpus)

**Phase A — squelette (coût S, à faire d'abord)**
1. Effets réversibles dans le registre (`register` → `Registration` avec `dispose()`).
2. Couches nommées dans la config (`ConfigLayerSource` + provenance de valeur).
3. Guard monotone (`PermissionPolicy` : composition = intersection, jamais union).
4. Matrice d'événements générée + test de cohérence.

**Phase B — coutures manquantes (coût M)**
5. Adaptateurs des 3 ports orphelins (`MemoryStore`, `Hook`, `SubagentRunner`).
6. Reload atomique par plugin (file sérialisée avec les écritures de config).

**Phase C — capacités lourdes (coût M/L)**
7. Sandbox réel (namespace + FS + réseau, fail-closed).
8. Compaction LLM double couche.

**Phase D — industrialisation**
9. Provider externe de sous-agents (monter Claude Code/Codex/ACP comme providers).
10. Marketplace/atelier de plugins (leçon DSH Desktop/Web).

## 5. Non-objectifs (écartés, avec raison)

- **Runtime in-process JS/Python** (PTC / `run_code` de DeepSeek Harness) : hors
  périmètre d'un noyau Rust, mêmes leçons structurelles que le Code Mode d'OpenCode.
- **Fournisseurs en dur** (Void : 16 `if/else`) : interdit par notre doctrine.
- **`crushrc` Bash** (Crush) : risque supply-chain ; on garde une config déclarative.
- **Télémétrie sortante par défaut** (GPT Pilot → `api.pythagora.io`) : contraire
  au self-hosted ; opt-in seulement.
- **Tree-of-Thought récursif** (MetaGPT) : redondant avec le routage Pareto.

## 6. Provenance

- Capacités et consensus : `docs/omega/reverse-eng/OMEGA-COMPLET.md` (§2bis « noyau »).
- DeepSeek Harness : `docs/omega/reverse-eng/dissections/deepseek-harness.md`
  (42 fonctionnalités, section « Ce que notre noyau doit copier »).
- Règles de modèle : `docs/omega/rules/MODEL-POLICY.md`.
- Aucune ligne de ce document n'est une supposition : chaque règle cite son
  exemplaire et son `file:line` dans la dissection correspondante.
