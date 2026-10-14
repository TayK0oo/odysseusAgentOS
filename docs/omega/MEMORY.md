# MEMORY-OMEGA — Mémoire persistante de la session

> Mémoire durable, versionnée dans Git. **Source de vérité** pour reprendre la session
> à froid. Mettre à jour à chaque décision / découverte. Tags de provenance :
> `[stated]` (dit par l'utilisateur) · `[observed]` (vérifié) · `[inferred]` (déduit) ·
> `[todo]` (à faire) · `[blocked]` (en attente).

---

## 1. Identité de la session

| Champ | Valeur |
|---|---|
| Branche de travail | `feat/agentos-omega` `[observed]` |
| Branche de base | `feat/inventaire-global-v1` `[observed]` |
| **Répertoire de travail** | **`/home/tayk0oo/Projets/odysseusAgentOS-omega`** (worktree isolé) `[observed]` |
| Dépôt principal | `/home/tayk0oo/Projets/odysseusAgentOS` (reste sur `feat/inventaire-global-v1`) `[observed]` |
| Dépôt distant | `github.com/TayK0oo/odysseusAgentOS` (fork de `arcahyadi/odysseus`) `[observed]` |
| Date d'ouverture | 2026-10-15 `[observed]` |
| Modèle orchestrateur | DeepSeek V4.1 Flash (opencode-go) `[stated]` |

## 2. Constat du dépôt de base (état au fork) `[observed]`

- 1071 fichiers `.py`, 136 modules `src/`, 63 routes, ~503 endpoints.
- SFD v3.1 : 25 principes · 22 modules · 21 UC · 21 NF (`docs/master-ref/01-SFD-v3.1.md`).
- Couverture code : **6/22 principes actifs**, **6/19 UC actifs**. Environnement non exécutable (pas de venv).
- Modularité : god node `core.database` (64 deps), `route_loader` non dynamique (58 imports), 71 shims legacy.
- Vision existante : 5 couches (CONNAISSANCE → EXÉCUTION → DESIGN → OBSERVABILITÉ → GOUVERNANCE),
  8 axes, Constitution en 13 étapes (`docs/VISION-INTEGRATION.md`).
- Config OpenCode : plugins TS `@agentos/sfd-*`, agents `sfd-orchestrator/executor/planner/reviewer`.
- UI : `static/` (HTML/CSS/JS) — pas de React/TS.

## 3. ⚠️ Concurrence détectée (isolation en place) `[observed]`

- Un **agent concurrent** écrivait dans le répertoire principal pendant cette session
  (refactor « MOD-5 AppContext » sur 30 fichiers `src/*.py` + `src/app_context.py` +
  `tests/test_sprint5_mod5_appcontext.py`, écrits entre 00:42 et 00:47).
- **Cause probable :** une autre session OpenCode travaillant sur le socle.
- **Action :** travail Omega isolé dans un `git worktree` dédié ; répertoire principal
  restitué à `feat/inventaire-global-v1` avec ses modifications intactes.
- **Conséquence :** ne jamais opérer Omega depuis le dépôt principal.

## 4. Recherche — corpus TikTok `@0xloucash` `[observed]`

- Profil : **Loucash**, 235K abonnés. Bio : « Openclaw / Tech / Polymarket ».
- **78 vidéos** énumérées (`yt-dlp`), ≈2,96 M vues cumulées, période ~2026-03 → 2026-10.
- Fichiers : `docs/omega/research/loucash/` (`videos.json`, `videos_enriched.json`,
  `classified.json`) + vault `obsidian-vault/loucash/` (78 notes + 15 MOC + `_INDEX.md`).
- Top catégories : claude-tooling (10), skills-plugins (9), web-design (9), ai-agents (8),
  automation (6), business-monetization (5), free-alternatives (5), github-repos (5)…
- Pistes clés pour Omega : Hermès (vidéo #25), 1 plugin = 68 agents/300 skills (#14),
  system prompt Claude fuité (#30), repo de reverse-engineering (#3), contournement des
  limites Claude Code via modèles OSS (#27), mémoire + skill auto-évolutive (#26),
  second cerveau Obsidian (#50), alternatives gratuites installées par prompt (#5).

## 5. États de l'art produits `[observed]`

| Doc | Contenu |
|---|---|
| `docs/omega/research/HARNESS-LANDSCAPE.md` | 25 harnais, patterns plugins, **top-10 features à reverse-engineer** |
| `docs/omega/research/MODEL-ROUTING-BENCHMARKS.md` | Sources de benchmarks, composite de score, taxonomie de tâches, architecture routeur |

## 6. Décisions (log)

| # | Décision | Date | Statut | ADR |
|---|---|---|---|---|
| D1 | Nouvelle branche dédiée à la vision « ultime » | 2026-10-15 | ✅ | — |
| D2 | Nom de branche `feat/agentos-omega` | 2026-10-15 | ✅ | — |
| D3 | Ancrage doc `docs/omega/` ; règles `.opencode/rules/` | 2026-10-15 | ✅ | — |
| D4 | Recherche classée dans `docs/omega/research/` + vault Obsidian | 2026-10-15 | ✅ | — |
| D5 | **Moteur : réécriture complète en Rust** | 2026-10-15 | ✅ `[stated]` | ADR-0001 (à écrire) |
| D6 | **UI : nouvelle app React/TS à côté** (pas de refonte incrémentale) | 2026-10-15 | ✅ `[stated]` | ADR-0002 (à écrire) |
| D7 | **Premier chantier de code : W5 reverse-engineering** | 2026-10-15 | ✅ `[stated]` | — |
| D8 | Isolation du travail Omega via git worktree (concurrence) | 2026-10-15 | ✅ `[observed]` | — |

## 7. Credentials & accès

Annoncés par l'utilisateur comme fournissables `[stated]` — à déposer dans un `.env`
**non versionné** (jamais dans Git). Statut : `[blocked]` tant que non fournis.

| Ressource | Variable | Statut | Usage |
|---|---|---|---|
| OpenCode ZenRouter | `OPENCODE_API_KEY` | `[blocked]` annoncé | Routage moteur principal |
| GitHub | `GITHUB_TOKEN` | `[blocked]` annoncé | Clones, reverse-eng, rate-limit |
| HuggingFace | `HF_TOKEN` | `[blocked]` annoncé | Benchmarks, modèles |
| TikTok (session) | cookies `sessionid` | `[blocked]` annoncé | Transcription / scraping profond |
| Compute élastique | `MODAL_TOKEN` / SSH | `[blocked]` annoncé | Exécution à la demande |
| Clés LLM multi-fournisseurs | `OPENAI/ANTHROPIC/…` | `[todo]` non fourni | Registre routeur (non requis au démarrage) |

## 8. Prochaines étapes (backlog vivant)

- [x] Enrichir les 78 vidéos (dates, counts, hashtags)
- [x] Classification riche + vault Obsidian
- [x] Paysage des harnais + benchmarks/routing
- [x] ADR-0001 (Rust) · ADR-0002 (UI React) · ADR-0003 (architecture plugin)
- [x] `00-VISION-OMEGA.md` (vision cible)
- [x] **W5 — fiches RE** : `RE-OPENCLAW-HERMES.md` (975 l.) · `RE-GOOSE-CODEX.md` (695 l.) · `RE-OPENCODE-CLAUDECODE.md` (396 l.)
- [x] **W5 — consolidation** : `FEATURE-MATRIX.md` (96 features) · `OMEGA-BLUEPRINT.md` (18 crates) · `features.json`
- [x] **W2 — incrément 1** : workspace `omega/` + crates `types/ports/config/registry/plugin-api`, 13 tests verts, clippy propre
- [x] **W2 — incrément 2** : crates `omega-core` (boucle d'agent) + `omega-protocol` (wire types), `omega/scripts/check_dep_graph.py` (graphe + garde-fou), 25 tests verts, clippy/fmt propre
- [ ] Décomposition détaillée en workstreams + estimation
- [ ] Obtenir credentials (déposer dans `.env` puis `python3 scripts/omega_check_env.py`)

### Blocage de facturation
Le compte des **subagents** est à sec (`Insufficient account funds`, 2026-10-15). Le travail
parallèle par subagents est suspendu ; l'orchestrateur exécute lui-même en attendant.

### Clones de travail (nettoyables)
`/tmp/opencode/re/` : codex (131M), goose (646M), hermes-agent (344M), openclaw (939M), opencode (230M).

## 9. Pointeurs utiles

| Sujet | Fichier |
|---|---|
| Règles session | `docs/omega/rules/RULES-OMEGA.md` |
| Vision (à écrire) | `docs/omega/00-VISION-OMEGA.md` |
| Workstreams | `docs/omega/workstreams/` |
| Reverse-engineering | `docs/omega/reverse-eng/` |
| Recherche | `docs/omega/research/` |
| ADR | `docs/omega/adr/` |
| Vision existante | `docs/VISION-INTEGRATION.md` |
| SFD de référence | `docs/master-ref/01-SFD-v3.1.md` |
