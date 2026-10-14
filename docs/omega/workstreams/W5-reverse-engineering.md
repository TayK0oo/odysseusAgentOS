# W5 — Reverse-engineering des harnais

> **But :** extraire des meilleurs harnais du marché les fonctionnalités qui changent
> réellement la façon de travailler, puis les re-spécifier proprement pour *AgentOS Omega*.
> **Statut :** 🟢 en cours · **Priorité utilisateur :** 1er chantier de code `[stated]`
> **Provenance :** `[observed]` (code/README) · `[inferred]` · `[todo]`.

---

## 1. Double livrable

1. **Capacité générique** — une *méthode* reproductible (et à terme un outil) pour
   reverse-engineer n'importe quel harnais : code, docs, prompts, config, protocoles.
2. **Extraction exécutée** — des fiches `RE-<harnais>.md` par cible prioritaire, avec
   contrats, schémas et coût de portage Rust (S/M/L).

## 2. Méthode en 6 étapes (répétable)

| Étape | Action | Livrable |
|---|---|---|
| **1. Acquire** | Clone superficiel (`--depth 1`) + README/docs + prompts/config publics | corpus local |
| **2. Inventory** | Arborescence, crates/modules, entrypoints, dépendances, licence | carte du harnais |
| **3. Extract** | Ports/interfaces, contrats de données, schémas de config, hooks, protocoles (MCP/ACP) | contrats cités (file:line) |
| **4. Feature matrix** | Features × (valeur pour Omega, difficulté Rust, dépendances) | tableau priorisé |
| **5. Spec** | Re-spécification *originale* (pas de copie de code propriétaire) + test d'acceptation | fiche `RE-*.md` |
| **6. Archive** | Consolidation dans un blueprint Omega | `OMEGA-BLUEPRINT.md` |

## 3. Cibles prioritaires

| # | Cible | Pourquoi | Licence | Statut |
|---|---|---|---|---|
| T1 | **OpenClaw** (`github.com/openclaw/openclaw`) | Context Engine enfichable, slots, policy-par-code — le modèle « everything is a plugin » le plus abouti | MIT | 🔄 en cours |
| T2 | **Hermes Agent** (`github.com/NousResearch/hermes-agent`) | Tool search (BM25), progressive disclosure, mémoire bornée, FTS5, 7 backends d'exécution, boucle auto-améliorante | MIT | 🔄 en cours |
| T3 | **Goose** (`github.com/block/goose`) | **Cœur Rust** (notre stack), context revision, extensions MCP, ACP | Apache-2.0 | 🔄 en cours |
| T4 | **Codex CLI** (`github.com/openai/codex`) | **Cœur Rust**, config TOML à couches, sandbox + approbations, OTel, sous-agents | Apache-2.0 | 🔄 en cours |
| T5 | **OpenCode** (`github.com/sst/opencode`) | Registre de plugins TS, agents/skills/commands déclaratifs, multi-providers | MIT | 🔄 en cours |
| T6 | Cline (`github.com/cline/cline`) | SDK hooks de cycle de vie, teams persistantes, agents scheduled | Apache-2.0 | ⏸ à planifier |
| T7 | Claude Code (propriétaire) | Compaction, prompt caching, tool search, sous-agents — RE **par docs/comportement** + system prompt fuité (corpus TikTok #30) | Propriétaire | ⏸ à planifier |

## 4. Features cibles (issues de `HARNESS-LANDSCAPE.md`)

P1 Context Engine enfichable · P2 Progressive disclosure des skills · P3 Tool search ·
P4 Compaction + stabilité de préfixe (prompt caching) · P5 Sous-agents isolés parallèles ·
P6 Registre multi-modèles + routing par capacité · P7 Plan/Act + agent de planification ·
P8 Checkpoints/worktrees/rollback · P9 Exécution sandboxée multi-backend ·
P10 Hooks d'événements de cycle de vie.

## 5. Garde-fous

- Extraire des **patterns et des idées** — jamais copier du code propriétaire.
- Respecter les licences OSS lors de toute réutilisation de code.
- Toute affirmation adossée à une source (`file:line` ou URL), taguée.
- Ne rien coder avant la fiche `RE-*.md` (le port, le contrat, le test d'acceptation).

## 6. Livrables attendus

```
docs/omega/reverse-eng/
  RE-OPENCLAW-HERMES.md       (T1+T2)  architecture , contrats , features , portage Rust
  RE-GOOSE-CODEX-OPENCODE.md  (T3+T4+T5)
  FEATURE-MATRIX.md           features × valeur × difficulté (consolidé)
  OMEGA-BLUEPRINT.md          synthèse → ce qu'Omega reprend / améliore / écarte
```
