# RÉCAP COMPLET — AgentOS Omega (session du 15→17 octobre 2026)

> ## ⚠️ HISTORIQUE — snapshot des 15→17 octobre 2026
>
> Récit de session, non maintenu. **C'est ici que vivent les chiffres historiques**
> (comptes de tests, de routes et d'endpoints d'alors) : ils ne sont pas vérifiés par
> `STATUS.json` et ne doivent pas être cités comme l'état courant.
> **État mesuré : `docs/omega/plan/STATUS.md` · plan : `docs/omega/PLAN-MAITRE.md`.**

> **Branche** : `feat/agentos-omega` · **Worktree** : `/home/tayk0oo/Projets/odysseusAgentOS-omega`
> **Dernier commit** : `3f10f5bb` · **Fichiers versionnés** : 1 962 (hors `target/`, `node_modules/`, `.git/`)
> **Doctrine** : *everything is a plugin* — Rust au cœur, React/TS au bord, zéro couplage caché, tout traçable.

---

## 0. Résumé exécutif

| Axe | Résultat |
|---|---|
| **Moteur Rust** | 16 crates, **293 tests** verts, clippy/fmt verts, graphe 0 cycle |
| **Cockpit UI** | 16 vues React/TS, origine unique, isolation par capacité/run |
| **Corpus reverse-engineering** | **35 harnais / 745 fonctionnalités / 2 764 preuves** (167 adoptées · 556 planifiées · 22 écartées) |
| **Capacités agrégées** | **108**, consensus **pondéré par popularité mesurée** (API GitHub) |
| **REA** | branché comme plugin (`mcp:rea`) + toolchain complet (Ghidra 12.1.4, 15 moteurs) |
| **Règle des modèles** | posée **et appliquée** (gratuit par défaut, payant `opencode-go/*` en exception) |
| **Scripts déterministes** | 11 (zéro LLM) |
| **Clones + outils + preuves lourdes** | `/home/tayk0oo/omega-re/` — **38 dépôts**, 14 Go |

---

## 1. Chronologie des commits de la session

### 15 octobre — fondations (W5→W7)
```
f84d4b89  revue finale zero-erreur (traversal run_id, preference locale, garde plan, prune)
04ae01fa  correction optimale REVIEW-FINAL-2 (timeouts HTTP, eligibilite, approbations)
63827d87  B1 timeout HTTP, B2 eligibilite candidate local, mineurs
34c94621  revue independante increments recents (AJOURNE 6/10)
383e6545  trace des commits de correction
1bb1e695  W7 inc3 — origine unique (UI servie par le serveur) + reponse event-sourced
41c8a024  W7 inc4 — cockpit complet 10 panneaux + 7 endpoints d'inspection
ed2f2a77  W7 inc5 — observatoire complet (13 vues, isolation capacite/run)
d32e856b  W7 inc2 — catalogue de harnais executable (15 harnais, 48 principes)
4ad6df3f  W5 inc3 — pipeline de dissection par application (schema, types, endpoints, vue UI)
7491a1a9  W5 inc3b — dissections OpenClaw/Codex/Goose/OpenCode/Cline (103 fct)
1eaa9209  W5 inc3c — dissections wave 2 (Aider, OpenHands, gptme, Plandex, Zed, Open Interpreter)
6b0f517a  plan d'integration par consensus (script deterministe) — 55 capacites
a4cf2527  W5 inc3d — wave 3 (Claude Code, Cursor, Windsurf, Devin) : 16 apps, 269 fct
```

### 16 octobre — corpus + REA
```
e2476bf3  transcripteur Whisper local du corpus TikTok (contournement PyAV)
0107d7fd  78 transcriptions Whisper (74k caracteres) + OCR des frames
2b35868f  REA identifie (morluto/rea) depuis la video #3 — chaine de preuve
6e15ace9  W5 inc3e — dissection REA (19 fct) : 17 apps, 288 fct
f7ad30b4  brancher REA au noyau (crate omega-mcp) + balayage de nos artefacts
39ea7491  mode --harnesses du balayage REA
798f3028  REA applique aux harnais clones (5/6 ; openclaw = resource_constraint)
d8ba7a3d  recensement deterministe des harnais (popularite GitHub mesuree)
2d4f0639  OMEGA-COMPLET — le combine unifie (72 capacites)
9a08b0d5  intake de 14 harnais populaires — clone + analyse REA (12/14)
a1641e39  programme reproductible en 4 etapes
686a74ba  garde-fou des dissections (structure + resolution des preuves)
41dffc68  limite majeure : REA n'analyse pas Python/Rust/Go
8499baaa  enregistrement deterministe des dissections (APP_DISSECTIONS + seuil)
4cb150bc  corpus elargi a 31 harnais / 641 fct / 2335 preuves
```

### 17 octobre — toolchain + règle + corpus final
```
5a548403  regle globale des modeles — gratuits par defaut, payant Go en exception
4c9e76d8  toolchain complet installe + inventaire verifie
a5e7032e  memoire : regle modeles + toolchain REA
de40b162  preuve Ghidra sur notre binaire Rust — 1458 procedures, 2668 chaines
2db5ec0b  wakaru (deobfuscation JS) + pont jadx-ai-mcp
4b16d22f  dissection DeepSeek Harness (42 fct) + intake des 4 entrants 2026
3dfda7b7  section NOYAU du combine — capacites de squelette + 7 regles de conception
ed7ab4b7  NOYAU-ULTIME — architecture cible de notre noyau
933d4712  corpus 33 harnais / 709 fct
6b281bf9  +DSH Web (18 fct) — corpus 34 harnais / 727 fct
3f10f5bb  +Deep Agents (18 fct) — corpus 35 harnais / 745 fct
```

---

## 2. Le code — `omega/`

### 2.1 Les 16 crates (workspace Rust)

| Crate | Catégorie | Rôle |
|---|---|---|
`omega-types` | types | domaine, erreurs, 29 capacités, régimes slot/ensemble |
`omega-ports` | ports | **17 traits/ports, aucune implémentation** |
`omega-config` | config | config à couches + precedence |
`omega-registry` | registry | registre (slots + quarantaine) |
`omega-plugin-api` | plugin-api | surface d'extension (manifeste → fabrique → registre) |
`omega-core` | core | boucle d'exécution |
`omega-protocol` | protocol | schéma wire versionné |
`omega-app` | app | exécution durable + reprise + garde de plan |
`omega-providers` | adapters | Ollama + catalogue local + capacités |
`omega-router` | router | classifieur + Pareto + scoring + fallback + circuit breaker |
`omega-context` | adapters | budget, BM25, skills, compaction, ToolSurfaceIndex |
`omega-otel` | adapters | provenance, sinks, rejeu |
`omega-sandbox` | adapters | politique + approbations + backend local |
`omega-harness` | adapters | catalogue de harnais + **35 dissections embarquées** |
`omega-mcp` | adapters | **pont MCP** (client stdio + surface + contributeur) |
`omega-server` | app-edge | API axum + service UI |

### 2.2 Les 11 scripts déterministes — `omega/scripts/`

| Script | Rôle |
|---|---|
`check_dep_graph.py` | garde-fou architecture (0 cycle, 0 arête interdite) |
`harness_census.py` | recensement des harnais (popularité **mesurée** via API GitHub) |
`harness_intake.py` | clone + analyse REA par harnais |
`validate_dissections.py` | structure **+ résolution des preuves** (`file:line` → clone) |
`register_dissections.py` | `APP_DISSECTIONS` + seuil du test (idempotent) |
`integration_plan.py` | consensus par nombre de harnais demandeurs |
`omega_complet.py` | **le combiné unifié** (108 capacités, section NOYAU) |
`model_policy.py` | `--check` (cohérence live), `--list-free`, `--for <usage>`, `--apply-agents` |
`rea-sweep.py` | balayage REA de nos artefacts + harnais clones |
`run-local.sh` | lancement complet (UI + serveur, port 8787) |

### 2.3 L'UI — `omega/ui/`

`src/App.tsx` (16 vues) · `src/panels.tsx` (panneaux) · `src/api.ts` (client HTTP) · `src/protocol.ts` (types wire) · `src/components.tsx` · `src/styles.css`

**16 vues** : Vue d'ensemble · Cockpit · Système · Harnais · Dissections · **REA** · Registre · Plugins · Modèles · Routeur · Classifieur · Runs · Traces · Sandbox · Contexte · Config/API

---

## 3. Le corpus reverse-engineering

### 3.1 Les 35 applications disséquées

| App | Nom | Fct | Adoptées | Planifiées | Écartées |
|---|---|---|---|---|---|
| `agentgpt` | AgentGPT | 23 | 5 | 18 | 0 |
| `aider` | Aider | 17 | 0 | 17 | 0 |
| `autogpt` | AutoGPT | 31 | 8 | 23 | 0 |
| `claude-code` | Claude Code | 19 | 4 | 14 | 1 |
| `cline` | Cline | 18 | 7 | 10 | 1 |
| `codex` | Codex CLI | 19 | 13 | 6 | 0 |
| `continue` | Continue | 29 | 5 | 23 | 1 |
| `crewai` | CrewAI | 26 | 1 | 24 | 1 |
| `crush` | Crush | 26 | 11 | 14 | 1 |
| `cursor` | Cursor | 17 | 0 | 16 | 1 |
| `deep-agents` | Deep Agents | 18 | 9 | 9 | 0 |
| `deepseek-harness` | DeepSeek Harness | 42 | 9 | 32 | 1 |
| `devin` | Devin | 14 | 0 | 14 | 0 |
| `dsh-desktop` | DSH Desktop | 26 | 3 | 22 | 1 |
| `dsh-web` | DSH Web | 18 | 6 | 9 | 3 |
| `gemini-cli` | Gemini CLI | 35 | 9 | 26 | 0 |
| `goose` | Goose | 19 | 5 | 13 | 1 |
| `gpt-engineer` | GPT Engineer | 20 | 7 | 12 | 1 |
| `gpt-pilot` | GPT Pilot | 18 | 3 | 15 | 0 |
| `gptme` | gptme | 16 | 4 | 12 | 0 |
| `hermes` | Hermes Agent | 12 | 3 | 9 | 0 |
| `kilo-code` | Kilo Code | 31 | 11 | 19 | 1 |
| `langgraph` | LangGraph | 24 | 4 | 20 | 0 |
| `metagpt` | MetaGPT | 24 | 4 | 19 | 1 |
| `open-interpreter` | Open Interpreter | 18 | 3 | 14 | 1 |
| `openclaw` | OpenClaw | 17 | 6 | 11 | 0 |
| `opencode` | OpenCode | 18 | 4 | 13 | 1 |
| `openhands` | OpenHands | 17 | 5 | 12 | 0 |
| `plandex` | Plandex | 15 | 0 | 14 | 1 |
| `qwen-code` | Qwen Code | 23 | 8 | 14 | 1 |
| `rea` | REA | 19 | 0 | 19 | 0 |
| `tabby` | Tabby | 16 | 0 | 16 | 0 |
| `void` | Void | 27 | 5 | 20 | 2 |
| `windsurf` | Windsurf / Cascade | 16 | 0 | 15 | 1 |
| `zed` | Zed | 17 | 5 | 12 | 0 |
| **TOTAL** | | **745** | **167** | **556** | **22** |

### 3.2 Les artefacts du corpus

| Artefact | Chemin | Contenu |
|---|---|---|
Données JSON (embarquées) | `omega/crates/omega-harness/data/apps/*.json` | 35 dissections structurées |
Catalogue de principes | `omega/crates/omega-harness/data/harnesses.json` | 15 harnais, 48 principes |
Recensement | `omega/crates/omega-harness/data/harnesses_census.json` | 66 harnais, popularité mesurée |
Fiches narratives | `docs/omega/reverse-eng/dissections/*.md` | **34 fiches** (récit + traduction plugins) |
Schéma | `docs/omega/reverse-eng/DISSECTION-SCHEMA.md` | contrat JSON + table d'honnêteté |
Matrice | `docs/omega/reverse-eng/FEATURE-MATRIX.md` + `features.json` | vue croisée |
Recensement (lisible) | `docs/omega/reverse-eng/HARNESS-CENSUS.md` | classement + écarts |
Intake (lisible) | `docs/omega/reverse-eng/HARNESS-INTAKE.md` | clone + REA par harnais |
Combiné | `docs/omega/reverse-eng/OMEGA-COMPLET.md` | 108 capacités, section NOYAU |
Plan de consensus | `docs/omega/reverse-eng/PLAN-INTEGRATION.md` | priorités par nombre de harnais |
Méthode | `docs/omega/reverse-eng/PROGRAMME-RE.md` | les 4 étapes reproductibles |
**Architecture cible** | `docs/omega/reverse-eng/NOYAU-ULTIME.md` | 7 règles, coutures, écart, plan 4 phases |
Fiches RE (antérieures) | `docs/omega/reverse-eng/RE-*.md` | Goose, OpenClaw/Hermes, OpenCode/Claude Code |

---

## 4. REA — branché et outillé

### 4.1 Le pont — `omega/crates/omega-mcp/`

| Fichier | Rôle |
|---|---|
`src/lib.rs` | `McpClient` (JSON-RPC stdio), `McpToolSurface`, `McpToolExecutor`, `McpContributor`, `McpServerSpec::rea()` (charge le toolchain) |
`src/bin/mcp_stub.rs` | serveur MCP **stub** (tests déterministes) |
`tests/bridge.rs` | 8 tests d'intégration |
`data/rea-snapshot.json` | instantané embarqué (207 outils MCP, 153 CLI, 6 prompts) |
`data/rea-capabilities.json` | providers + opérations |

**Enregistré** : plugin **`mcp:rea`** en `Capability::ToolSurface` (visible `/api/plugins`).
**Endpoints** : `GET /api/rea` · `GET /api/rea/tools` (live) · `POST /api/rea/call`.
**Preuve live** : 207 outils MCP listés en **2,2 s** par notre client Rust.

### 4.2 Le toolchain — installé et câblé

| Moteur | Version | Variable REA |
|---|---|---|
**Ghidra** | 12.1.4 | `GHIDRA_INSTALL_DIR` + `JAVA_HOME` (JDK 25) |
jadx | 1.5.6 | `REA_JADX_MCP_JAR` (jadx-ai-mcp 6.4.1) |
apktool | 2.11.1 | `REA_APKTOOL_COMMAND` |
tshark / capinfos | 4.6.9 | `REA_TSHARK_COMMAND` |
binwalk / unblob | 3.1.0 | `REA_BINWALK_COMMAND`, `REA_UNBLOB_COMMAND` |
rizin / gdb | 17.2 | `REA_RIZIN_COMMAND`, `REA_GDB_COMMAND` |
ilspycmd | 11.1.0 | `REA_ILSPY_CMD_PATH` |
wabt | 1.0.40 | `REA_WABT_BIN_DIRECTORY` |
sigrok-cli | — | `REA_SIGROK_COMMAND` |
frida / mitmdump | — | venv dédié |
wakaru | 1.14.0 | `REA_WAKARU_COMMAND` |
chromium | — | `REA_BROWSER_EXECUTABLE` |
pwntools/pyelftools/unicorn | 4.15.0/0.33/2.1.2 | `REA_PWNTOOLS_PYTHON` (venv python3.12) |

**Env persistant** : `/home/tayk0oo/omega-re/rea-tools.env` → `/etc/profile.d/rea-tools.sh` + `~/.config/environment.d/rea-tools.conf` + `docs/omega/rules/rea-tools.env`.
**Installateur** : `/home/tayk0oo/omega-re/install-rea-tools.sh` (idempotent).
**Inventaire vérifié** : `docs/omega/research/rea/CAPABILITIES.md`.

### 4.3 Les preuves REA — `docs/omega/research/rea/`

| Fichier | Contenu |
|---|---|
`REA.md` | identification de REA (chaîne de preuve depuis la vidéo #3) + plan d'intégration |
`CAPABILITIES.md` | inventaire vérifié (opérationnel / impossible) |
`snapshot.json` | catalogue REA embarqué |
`SWEEP.md` | REA sur **nos artefacts** (cockpit JS, sources, ELF) |
`HARNESS-SWEEP.md` | REA sur **les harnais clones** (5/6) |
`evidence/` | **28 fichiers** de preuves (intake + analyses) |
`evidence/ghidra-omega-crate-elf.summary.json` | **Ghidra sur notre binaire Rust : 1 458 procédures, 2 668 chaînes, 44 segments** |

**Preuve complète (hors dépôt)** : `/home/tayk0oo/omega-re/rea-evidence/` (26 fichiers).

---

## 5. La règle globale des modèles

| Artefact | Chemin |
|---|---|
Doctrine (lisible) | `docs/omega/rules/MODEL-POLICY.md` |
Version machine | `docs/omega/rules/model-policy.json` |
Vérification live | `omega/scripts/model_policy.py` (`--check`, `--list-free`, `--for`, `--apply-agents`) |
Liste live des modèles | `docs/omega/rules/models-live.json` |
Règle de session | `.opencode/rules/model-policy.md` |
Première lecture | `AGENTS.md` §0 |
Config globale | `~/.config/opencode/opencode.json` (agents épinglés gratuits) |
Config projet | `.opencode/opencode.json` |

**Doctrine** : sous-agents sur **gratuits** par défaut ; payant **`opencode-go/*` uniquement**, après échec gratuit constaté. **11 gratuits vérifiés** le 17/10.
**Correction réelle** : nos 6 agents étaient sur `opencode/deepseek-v4-pro` (Zen payant, hors Go, retiré chez DeepSeek) → repassés sur des gratuits.

---

## 6. Les ressources rassemblées (avec chemins)

### 6.1 Corpus TikTok — `docs/omega/research/loucash/`

| Fichier | Contenu |
|---|---|
`videos.json` / `videos_enriched.json` | 78 vidéos + métadonnées enrichies |
`transcripts.json` | **78 transcriptions Whisper** (74 k caractères) |
`ocr.json` | **OCR des frames** (57 k caractères) |
`classified.json` | classification |
`raw_ytdlp.json` | données brutes yt-dlp |
`transcribe.py` | transcripteur Whisper local (contournement PyAV) |
`frames_ocr.py` | OCR des frames (tesseract fra+eng) |
`enrich.py` | enrichissement |

**Lecture à 3 niveaux** : légendes + **oral** + **écran** — c'est ce qui a permis de retrouver **REA** (`morluto/rea`).

### 6.2 Recherche harnais — `docs/omega/research/`

| Fichier | Contenu |
|---|---|
`HARNESS-LANDSCAPE.md` | paysage des harnais |
`MODEL-ROUTING-BENCHMARKS.md` | benchmarks de routage |
`REA.md` | identification + intégration de REA |

### 6.3 Recherche modèles (hors dépôt — **à ingérer**)

| Fichier | Chemin |
|---|---|
Classement gratuits | `~/Téléchargements/opencode-gratuits_classement_2026-10-17.md` |
Classement Go | `~/Téléchargements/opencode-go_classement_2026-10-17.md` |
Sources | `~/Téléchargements/opencode_sources_2026-10-17.md` |

### 6.4 Clones et outils — `/home/tayk0oo/omega-re/` (14 Go)

| Élément | Détail |
|---|---|
**38 dépôts clonés** | aider, agentgpt, autogpt, cline, claude-code, codex, continue, crewai, crush, cursor, deep-agents, deepseek-harness, devin, dsh-desktop, dsh-web, gemini-cli, goose, gpt-engineer, gptme, gpt-pilot, hermes, kilo-code, langgraph, metagpt, open-interpreter, openclaw, opencode, openhands, plandex, qwen-code, rea, tabby, void, windsurf, zed + loucash-* |
Outils | `tools/` (ghidra, jadx, apktool, jadx-ai-mcp) · `tools-venv/` (frida, unblob, mitmdump) · `rea-venv3/` (pwntools) |
Preuves | `rea-evidence/` (26 fichiers) · `loucash-audio/` · `loucash-frames/` · `loucash-video/` |
Installateur | `install-rea-tools.sh` |

---

## 7. Les fichiers créés (par chantier)

### 7.1 Code Rust — `omega/crates/`

`omega-mcp/` (pont MCP) · `omega-server/src/rea.rs` (endpoints REA) · `omega-server/src/state.rs` (contributeur `mcp:rea`) · `omega-server/src/routes.rs` · `omega-harness/src/lib.rs` (`APP_DISSECTIONS`, 35 apps) · `omega-harness/data/apps/*.json` (35) · `omega-harness/data/harnesses_census.json`

### 7.2 Scripts — `omega/scripts/`

`harness_census.py` · `harness_intake.py` · `validate_dissections.py` · `register_dissections.py` · `omega_complet.py` · `model_policy.py` · `rea-sweep.py` · `check_dep_graph.py` · `run-local.sh`

### 7.3 UI — `omega/ui/src/`

`App.tsx` (vue REA ajoutée) · `panels.tsx` (`ReaPanel`) · `api.ts` (`getRea`, `getReaTools`, `postReaCall`) · `protocol.ts` (types REA)

### 7.4 Docs — `docs/omega/`

**Règles** : `rules/MODEL-POLICY.md` · `rules/model-policy.json` · `rules/models-live.json` · `rules/rea-tools.env` · `rules/RULES-OMEGA.md`
**Recherche** : `research/REA.md` · `research/rea/CAPABILITIES.md` · `research/rea/SWEEP.md` · `research/rea/HARNESS-SWEEP.md` · `research/rea/evidence/*` (28)
**Reverse-engineering** : `reverse-eng/NOYAU-ULTIME.md` · `reverse-eng/OMEGA-COMPLET.md` · `reverse-eng/PLAN-INTEGRATION.md` · `reverse-eng/PROGRAMME-RE.md` · `reverse-eng/HARNESS-CENSUS.md` · `reverse-eng/HARNESS-INTAKE.md` · `reverse-eng/DISSECTION-SCHEMA.md` · `reverse-eng/dissections/*.md` (34)
**Mémoire** : `MEMORY.md` · `00-VISION-OMEGA.md` · `02-WORKSTREAMS.md` · `adr/0001-0003.md` · `workstreams/W2-W10.md` · `reviews/REVIEW-*.md`

### 7.5 Configuration OpenCode — `.opencode/`

`opencode.json` (agents épinglés gratuits + instructions) · `rules/model-policy.md` · `rules/omega.md` · `agents/*.md` (20 agents, 6 repassés gratuits) · `skills/` · `tools/sfd-phase.ts`

---

## 8. Le plan — encodé en 5 documents

| Document | Question | Contenu |
|---|---|---|
`PROGRAMME-RE.md` | *comment* | les 4 étapes reproductibles |
`HARNESS-CENSUS.md` | *qui* | 66 harnais, popularité mesurée |
`PLAN-INTEGRATION.md` | *par quoi* | consensus (capacité × nb de harnais) |
`OMEGA-COMPLET.md` | *le combiné* | 108 capacités, exemplaires + `file:line`, feuille de route, non-objectifs, **section NOYAU** |
`NOYAU-ULTIME.md` | *l'architecture* | 7 règles, coutures, écart mesuré, **plan en 4 phases** |

**Tête du consensus** : `context-engine` (27 harnais, 1,52 M★) · `subagent-runner` (21) · `tool-surface` (20) · `config-source` (18) · `registry` (14).

---

## 9. État et limites (honnêtes)

### Fait et vérifié
- 35 harnais disséqués, **2 764 preuves** avec `file:line` résolues ;
- REA branché et prouvé (207 outils en 2,2 s ; Ghidra : 1 458 procédures) ;
- 293 tests, clippy/fmt, graphe 0 cycle ;
- règle des modèles appliquée et vérifiable.

### Limites connues
- **REA n'analyse pas Python/Rust/Go** (preuve machine limitée au JS/TS et aux ELF) ;
- **macOS impossible** (structurel) ; **Hopper/IDA/JEB payants** ;
- **`resource_constraint`** sur les clones géants (OpenClaw 945 Mo, Void, Qwen Code) ;
- **3 ports orphelins** : `SubagentRunner`, `MemoryStore`, `Hook` ;
- **5 mécanismes de squelette manquants** (effets réversibles, couches nommées, reload atomique, guard monotone, matrice d'événements) ;
- **sandbox réel** et **compaction LLM** absents.

### Non structuré (3 trous)
- **Notre conversation** n'est pas indexée (ce document y remédie partiellement) ;
- **Tes 3 fichiers modèles** (`~/Téléchargements/`) sont hors dépôt — cités mais non versionnés ;
- **`MEMORY.md`** est maintenu à la main (peut retarder).

---

## 10. Prochaines étapes

| # | Étape | Coût | Statut |
|---|---|---|---|
1 | **Phase A du noyau** : effets réversibles, couches nommées, guard monotone, matrice d'événements | S | **prochain pas** |
2 | Adaptateurs des 3 ports orphelins (`SubagentRunner`, `MemoryStore`, `Hook`) | M | planifié |
3 | Sandbox réel (namespace + FS + réseau, fail-closed) | M | planifié |
4 | Compaction LLM double couche | M | planifié |
5 | Ingérer les 3 fichiers modèles dans `docs/omega/research/models/` | S | proposé |
6 | `MEMORY.md` auto-généré (script d'état) | S | proposé |
7 | Index racine `docs/omega/INDEX.md` (carte régénérable) | S | proposé |
8 | 23 harnais restants du recensement | — | chaîne prête |
