# REA — « Reverse Engineer Anything »

> **Statut :** identifié et à intégrer · **Date :** 2026-10-17
> **Source d'origine :** corpus TikTok `@0xloucash`, **vidéo #3** (`7694323778330856726`).
> **Provenance :** `[observed]` (README, recherche GitHub) · `[stated]` (référence utilisateur).

---

## 1. Identification (chaîne de preuve)

1. La **légende** de la vidéo #3 dit : *« un repo GitHub te donne la capacité de faire du
   reverse engineering à ton IA… la décortiquer »* — **sans nommer le repo** `[observed]`.
2. La **transcription** (Whisper) confirme, toujours sans nom `[observed]`.
3. L'**OCR des frames** a lu le README à l'écran :
   *« …rse Engineer Anything … across binaries, applications, and runtime behavior.
   Understand how it works, down to the binary level. »* `[observed]`
4. **Recherche GitHub** (API, phrase exacte) → `morluto/rea` :
   *« Reverse engineer anything with agents, from app behavior down to native binaries »*,
   **★ ≈ 250 000**, **MIT**, paquet npm **`rea-agents`** `[observed]`.

➡️ **REA = `morluto/rea`.** Il *était* dans le corpus ; seule la lecture du nom à
   l'écran manquait (OCR bruité).

> **Correctif de méthode.** Ce défaut a été corrigé : le corpus est désormais lu à
> **trois niveaux** — légendes (`videos_enriched.json`), **oral** (`transcripts.json`,
> 78 transcriptions) et **écran** (`ocr.json`, 78 vidéos). Un nom qui n'apparaît
> qu'à l'écran est maintenant atteignable.

## 2. Ce que c'est

**Un serveur MCP de reverse-engineering.** « One MCP for reverse engineering across
binaries, applications, and runtime behavior. »

- **Analyse locale** ; chaque conclusion est renvoyée **avec ses preuves et ses limites**.
- **Cibles** : binaires natifs (Hopper / Ghidra / IDA / objdump / Rizin), ELF, Go, Rust
  (cargo-auditable), WASM, EVM, SQLite, crashes Linux, JavaScript/Electron, sites web,
  captures réseau (HAR/SAZ/pcap), .NET, APK Android + appareils (adb), JEB, ressources
  Android (apktool), Flutter, firmware (binwalk/unblob), paquets/ressources, comportement
  de process.
- **Distribution** : serveur MCP + skill + CLI (`npx rea-agents setup`,
  `npx -y rea-agents@latest analyze-javascript-application <path> --json`).
- **Agents supportés** : Claude Code, Codex, Cursor, Gemini CLI, Grok Build, autres
  clients MCP.
- **Licences** : REA = MIT ; les moteurs natifs (IDA, Hopper, JEB) restent sous licence
  commerciale de leurs éditeurs — REA n'en fournit aucune.

## 3. Pourquoi ça compte pour Omega

| Axe | Apport |
|---|---|
| **Pilier 2 (reverse-engineering)** | C'est *l'outil du pilier*. Il industrialise ce qu'on a fait à la main sur 16 apps. |
| **Comblement d'un trou réel** | Nos 3 apps fermées (Devin, Windsurf, Cursor/Claude Code en partie) n'ont été disséquées **que par docs**. REA analyse JS/Electron et binaires **sans source** → on peut disséquer le *produit* et pas seulement sa doc. |
| **Everything is a plugin** | REA est **MCP** → il s'enregistre comme simple **fournisseur d'outils** (`capacité tool`, adaptateur `plugins/mcp-rea`), sans toucher au noyau. |
| **Économie** | Analyse **locale**, pas d'upload → cohérent avec la mentalité self-hosted/gratuit. |
| **Traçabilité** | Ses résultats portent **preuves + limites** → correspondance directe avec notre `Provenance` (`[observed]`/`[inferred]`) et notre refus d'affirmer sans source. |

## 4. Plan d'intégration (à faire)

1. **Dissection** (`omega-harness/data/apps/rea.json`) — en cours : catalogue d'outils MCP,
   contrats d'évidence, providers, CLI.
2. **Adaptateur `plugins/mcp-rea`** : enregistrer le serveur MCP de REA comme
   `ToolExecutor`/`ToolSurface` dans le registre (bridge MCP générique, cf. `go-mcp-extensions`).
3. **Usage interne de la chaîne** : brancher REA sur les apps fermées pour étendre les
   dissections au-delà de la documentation — et **re-vérifier** les statuts `planned`
   avec des preuves machines.
4. **Provenance** : mapper « preuves + limites » de REA sur notre `ProvenanceSink`.

## 5. Sources

- `https://github.com/morluto/rea` · `https://rea.tools/` · npm `rea-agents`
- Corpus : `docs/omega/research/loucash/` — `videos.json`, `transcripts.json`, `ocr.json`
- Vidéo #3 : `https://www.tiktok.com/@0xloucash/video/7694323778330856726`

---

## 6. Pont branché — crate `omega-mcp` (fait)

REA est **branché au noyau comme un plugin**, sans modification du noyau :

| Élément | Valeur |
|---|---|
| Crate | `omega/crates/omega-mcp` (catégorie *adapters* du garde-fou) |
| Capacité | `ToolSurface` — plugin **`mcp:rea`** (visible dans `GET /api/plugins`) |
| Lancement | `npx -y rea-agents@latest --mcp` (MCP stdio, révision `2025-06-18`) |
| Préfixe d'outils | `rea::` (ex. `rea::analyze`, `rea::inspect-binary-layout`) |
| Ports | `McpToolSurface` (surface) + `McpToolExecutor` (`ToolExecutor`, « un outil = un port ») |
| Endpoints | `GET /api/rea` (instantané embarqué) · `GET /api/rea/tools` (catalogue live) · `POST /api/rea/call` (appel) |
| UI | onglet **REA** (snapshot, pont, providers, catalogue live filtrable) |
| Tests | 6 unitaires + 8 d'intégration contre un serveur MCP **stub** (`mcp_stub`), déterministes, hors réseau |

**Cycle de vie Phase 1** : chaque opération lance un serveur éphémère
(`spawn → initialize → list/call → kill`). Sûr et déterministe ; un mode
persistant viendra si la latence l'exige.

**Honnêteté** : un outil qui renvoie `isError: true` est mappé en
`OmegaError::PluginFailed` (un échec n'est jamais un succès) ; les erreurs
JSON-RPC sont remontées avec leur code.

## 7. REA appliqué à nos artefacts — `omega/scripts/rea-sweep.py`

Balayage reproductible (`docs/omega/research/rea/SWEEP.md`, régénérable) :

| Cible | Provider | Confiance | Structure |
|---|---|---|---|
| `omega/ui/dist` (cockpit bâti) | `rea-javascript-application` | derived | 5 nœuds / 6 arêtes |
| `omega/ui/src` (sources cockpit) | `rea-javascript-application` | derived | graphe de modules |
| binaire ELF de nos crates | `pwntools-elf` | **observed** | 44 sections, 12 segments, 2248 symboles, 2349 relocations |

Chaque analyse est conservée **avec ses limites déclarées** (5 à 12 selon la
cible) : c'est exactement le contrat de provenance d'Omega (`[observed]` vs
`[derived]`).

**Limites de la machine (constatées, non contournées)** :
- pas de Ghidra/Hopper/IDA → pas d'analyse binaire profonde (seul `objdump` système) ;
- pas de navigateur → `analyze-web-bundle` exige un CDP fourni par l'appelant
  (activé via `REA_CDP_ENDPOINT`/`REA_CDP_TARGET`) ;
- `REA_PWNTOOLS_PYTHON` doit pointer un Python fourni (venv `python3.12` :
  pwntools 4.15.0, pyelftools 0.33, unicorn 2.1.2) — REA n'installe jamais de paquets ;
- entrée limitée à 32 Mio (le binaire debug de 108 Mo est refusé ; on analyse un
  binaire de crate de 7 Mo).

## 8. REA appliqué aux harnais clones — `--harnesses`

Même balayage, ciblé sur les dépôts OSS que nous avons disséqués à la main
(`docs/omega/research/rea/HARNESS-SWEEP.md`) :

| Harnais | Statut | Limites déclarées |
|---|---|---|
| `cline` | ✅ analysé | 5567 |
| `opencode` | ✅ analysé | 9466 |
| `openhands` | ✅ analysé | 4025 |
| `codex` | ✅ analysé | 9 |
| `open-interpreter` | ✅ analysé | 9 |
| `openclaw` | ⚠️ `resource_constraint` | clone de **945 Mo** : REA épuise sa capacité de résultat (`extensions/memory-core/src/tools.runtime.ts`) |

Le nombre élevé de limites sur cline/opencode/openhands est **informatif** : ce
sont des monorepos à résolution dynamique, et REA déclare explicitement tout ce
qu'il ne peut pas prouver statiquement. `openclaw` doit être ciblé sur un
sous-dossier (ou le clone réduit) — limite documentée, non contournée.

**Intérêt pour Omega** : ces graphes sont une **seconde source** (machine) à
confronter à nos dissections manuelles (`data/apps/*.json`) — piste de
vérification croisée des statuts `adopted/planned`.
