# MEMORY-OMEGA — Mémoire persistante de la session

> Mémoire durable, versionnée dans Git. **Source de vérité** pour reprendre la session
> à froid. Mettre à jour à chaque décision / découverte. Tags de provenance :
> `[stated]` (dit par l'utilisateur) · `[observed]` (vérifié dans le code/système) ·
> `[inferred]` (déduit) · `[todo]` (à faire) · `[blocked]` (en attente).

---

## 1. Identité de la session

| Champ | Valeur |
|---|---|
| Branche de travail | `feat/agentos-omega` `[observed]` |
| Branche de base | `feat/inventaire-global-v1` `[observed]` |
| Dépôt | `github.com/TayK0oo/odysseusAgentOS` (fork de `arcahyadi/odysseus`) `[observed]` |
| Date d'ouverture | 2026-10-15 `[observed]` |
| Modèle orchestrateur | DeepSeek V4.1 Flash (opencode-go) `[stated]` |

## 2. Constat du dépôt de base (état au fork) `[observed]`

- 1071 fichiers `.py`, 136 modules `src/`, 63 routes, ~503 endpoints.
- SFD v3.1 : 25 principes · 22 modules · 21 UC · 21 NF (`docs/master-ref/01-SFD-v3.1.md`).
- Couverture code : **6/22 principes actifs**, **6/19 UC actifs**.
- Modularité : god node `core.database` (64 deps), `route_loader` non dynamique (58 imports),
  71 shims legacy. Faible découplage → cible d'« Everything is a plugin ».
- Vision existante : 5 couches (CONNAISSANCE → EXÉCUTION → DESIGN → OBSERVABILITÉ →
  GOUVERNANCE), 8 axes, Constitution en 13 étapes (`docs/VISION-INTEGRATION.md`).
- Config OpenCode : plugins TS `@agentos/sfd-*`, agents `sfd-orchestrator/executor/planner/reviewer`.
- UI : `static/` (HTML/CSS/JS classique) — pas encore de React/TS.

## 3. Recherche — compte TikTok `@0xloucash` `[observed]`

- Profil : **Loucash**, 235K abonnés, 262.2K likes. Bio : « Openclaw / Tech / Polymarket ».
- **78 vidéos** énumérées via `yt-dlp` (≈2 964 563 vues cumulées).
- Corpus : `docs/omega/research/loucash/videos.json` (+ `videos_enriched.json`).
- Thèmes dominants : skills/plugins Claude, repos GitHub gratuits, agents IA,
  reverse-engineering (Playwright/browser), OSINT, gouvernance/YouTube croisé,
  Polymarket/trading bots, web design (Blink, Stitch), alternances gratuites
  (Zapier/Canva/Perplexity), Hermes (mémoire persistante), Openclaw, ManyChat, Higgsfield,
  Arcads, Gamma, Pennylane MCP, MiniMax, Gemma4, Google Stitch.
- Pistes directement alignées sur notre vision :
  - **Harnais / câblage d'agents** : Claude Code subagents, Telegram remote, scheduled tasks,
    remote sessions, Hermes (mémoire auto-évolutive), self-improving agent.
  - **Reverse engineering** : repo « reverse engineering to your AI », Playwright-style.
  - **Contexte / économie** : skills à charger, design.md, 4 fichiers pour x10 Claude Code.
  - **Modèles gratuits / self-host** : MiniMax (vidéo), GLM, Gemma4, injection de system prompt.
  - **Routage / limites** : « ne plus atteindre la limite Claude Code » via modèles OSS.

## 4. Décisions (log)

| # | Décision | Date | Statut | ADR |
|---|---|---|---|---|
| D1 | Nouvelle branche dédiée pour la vision « ultime » | 2026-10-15 | ✅ | — |
| D2 | Nom de branche `feat/agentos-omega` | 2026-10-15 | ✅ | — |
| D3 | Ancrage doc : `docs/omega/` ; règles `.opencode/rules/` | 2026-10-15 | ✅ | — |
| D4 | Recherche classée dans `docs/omega/research/` + vault Obsidian | 2026-10-15 | ✅ | — |

## 5. Credentials & accès

| Ressource | Variable | Statut | Usage |
|---|---|---|---|
| OpenCode ZenRouter | `OPENCODE_API_KEY` | `[todo]` attendu | Routage moteur principal |
| GitHub | `GITHUB_TOKEN` | `[todo]` attendu | Clones, reverse-eng, rate-limit |
| HuggingFace | `HF_TOKEN` | `[todo]` attendu | Benchmarks, modèles |
| Fournisseurs LLM (multi) | `OPENAI/ANTHROPIC/GEMINI/GROQ/XAI/OPENROUTER/DEEPSEEK` | `[todo]` attendu | Registre routeur |
| TikTok (session) | cookies `sessionid` | `[todo]` optionnel | Transcription / scraping profond |
| Compute élastique | `MODAL_TOKEN` / SSH | `[todo]` optionnel | Exécution à la demande |

## 6. Prochaines étapes (backlog vivant)

- [ ] Enrichir les 78 vidéos (dates, counts, hashtags) → `videos_enriched.json`
- [ ] Classification riche + vault Obsidian + graphe Graphify
- [ ] Recherche paysage harnais (Claude Code, Codex, Hermes, Openclaw, OpenHands, Aider…)
- [ ] Rédiger la VISION-OMEGA + architecture plugin
- [ ] Décomposer en workstreams + estimation
- [ ] Obtenir credentials (voir §5)

## 7. Pointeurs utiles

| Sujet | Fichier |
|---|---|
| Règles session | `docs/omega/rules/RULES-OMEGA.md` |
| Vision (à écrire) | `docs/omega/00-VISION-OMEGA.md` |
| Recherche | `docs/omega/research/` |
| ADR | `docs/omega/adr/` |
| Vision existante | `docs/VISION-INTEGRATION.md` |
| SFD de référence | `docs/master-ref/01-SFD-v3.1.md` |
