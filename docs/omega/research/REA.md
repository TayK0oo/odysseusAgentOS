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
