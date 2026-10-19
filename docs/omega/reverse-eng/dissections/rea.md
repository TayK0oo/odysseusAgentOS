# Dissection — REA (Reverse Engineer Anything)

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | morluto (projet ouvert, ` skills.sh/morluto/rea`, site `rea.tools`) |
| Licence | MIT (`LICENSE`), paquet npm `rea-agents` v6.5.0 |
| Langage | TypeScript / Node.js 22/24/26 (`package.json`), bridges Python (Hopper), Java (Ghidra/JADX), Swift (natif) |
| Repo | https://github.com/morluto/rea |
| Architecture | Serveur MCP stdio + CLI one-shot partageant la même couche `src/application/` ; routeur de sessions binaires vers providers natifs (Hopper/Ghidra/IDA/Rizin/objdump/Cutter/GDB) + adaptateurs statiques ciblés (JS, .NET, APK, WASM, EVM, firmware, pcap, SQLite) ; tout résultat = record **Evidence** canonique avec couverture/limites explicites |

## 2. Entrées

- **Serveur MCP stdio** : `src/main.ts` + `src/main/`, dispatch `scripts/rea.mjs`, traduction `src/server/`.
- **CLI one-shot** : `rea` (≈50 modules `src/cli/*Commands.ts` enregistrés dans `src/cli.ts:1-60`), chaque invocation = un process séparé.
- **Setup** : `rea setup` (multi-agents, preview + approbation), `rea capabilities`, doctor (`src/application/Doctor.ts:186-210`).
- **Skill** : `.agents/skills/reverse-engineer-anything/SKILL.md` (355 lignes) + doctrine `docs/tool-design.md`.
- **Config** : chemins d'outils caller-supplied via env (`REA_BINWALK_COMMAND`, `REA_WABT_BIN_DIRECTORY`, `REA_TSHARK_COMMAND`…) — REA n'installe jamais les outils externes.

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Catalogue MCP versionné (~140 outils) | outils | Schémas stricts, `binary_session` comme source de vérité, catalog digest | `src/contracts/sessionToolContracts.ts:71-127`, `docs/mcp-contracts.md:851-862` | tool-surface → `ToolSurface` → `plugins/mcp-rea` | planned | M |
| Contrat d'évidence canonique | observabilite | Résultat normalisé + couverture + limites ; erreurs typées retryable | `src/domain/evidence.ts:37-291`, `docs/mcp-contracts.md:9-150` | provenance → `Provenance` → `plugins/tool-rea-evidence` | planned | M |
| Routeur multi-providers natifs | outils | Hopper/Ghidra/IDA/Rizin/objdump/Cutter/GDB, liaison immuable, pas de fallback | `src/ghidra/GhidraProvider.ts:1-40`, `src/ida/IdaProvider.ts:39-60`, `docs/reverse-engineering-tools.md:1-50` | tool → `ToolExecutor` → `plugins/tool-rea-native` | planned | L |
| Workflows de fonctions | outils | analyze/compare/call-graph/call-path, comparaison stricte | `src/contracts/sessionToolContracts.ts:112-127`, `docs/native-investigation.md:1-60` | tool → `ToolExecutor` → `plugins/tool-rea-functions` | planned | M |
| JS/Electron | outils | Graphe d'app canonique, bundles AST, fuses, runtime passif | `docs/javascript-application-graph.md:1-60`, `docs/cli.md:1-60` | tool → `ToolExecutor` → `plugins/tool-rea-js` | planned | L |
| .NET managé | outils | Metadata/CIL sans exécution, 7 outils inspect→project | `docs/managed-code-analysis.md:1-60`, `src/dotnet/ManagedPeReader.ts:1-40` | tool → `ToolExecutor` → `plugins/tool-rea-dotnet` | planned | M |
| Android + ADB | outils | JADX épinglé, apktool, ADB complet | `docs/android-analysis.md:1-60`, `src/adb/AdbProvider.ts:1-40` | tool → `ToolExecutor` → `plugins/tool-rea-android` | planned | M |
| WASM statique | outils | WABT caller-supplied, jamais exécuté | `docs/wasm-artifacts.md:1-60` | tool → `ToolExecutor` → `plugins/tool-rea-wasm` | planned | S |
| EVM off-chain | outils | Interface inférée (EVMole), jamais exécuté | `docs/evm-bytecode.md:1-60` | tool → `ToolExecutor` → `plugins/tool-rea-evm` | planned | S |
| Firmware | outils | Binwalk régions + Unblob extraction, process possédés | `docs/firmware-analysis.md:1-60` | tool → `ToolExecutor` → `plugins/tool-rea-firmware` | planned | M |
| Captures réseau/web | outils | pcap/tshark, HAR/SAZ, scénarios browser rejouables | `docs/pcap-analysis.md:1-60`, `docs/web-network-captures.md:1-60` | tool → `ToolExecutor` → `plugins/tool-rea-captures` | planned | M |
| Process Capture bornée | execution | Une commande, PTY, snapshots FS, comparaison de runs (pas un sandbox) | `docs/process-capture.md:1-80` | sandbox-backend → `SandboxBackend` → `plugins/sandbox-rea-capture` | planned | M |
| SQLite snapshots | outils | Copie + requêtes bornées, lecture seule | `docs/sqlite-databases.md:1-60` | tool → `ToolExecutor` → `plugins/tool-rea-sqlite` | planned | S |
| Crashes/cores/diagnostics | outils | trace_crash, ELF/core offline, Go/Rust statiques | `docs/recorded-crashes.md:1-60`, `docs/go-binaries.md:1-40` | tool → `ToolExecutor` → `plugins/tool-rea-crash` | planned | M |
| Sessions + bundles | memoire | Snapshots, ledger evidence/inconnues, export/import atomique | `docs/mcp-contracts.md:190-333` | skill-store → `SkillStore` → `omega-context::SkillIndex` | planned | M |
| CLI miroir du MCP | extension | Mêmes workflows en one-shot scriptable `--json` | `src/cli.ts:1-60`, `docs/cli.md:1-60` | harness-catalog → `Registre` → `plugins/tool-rea-cli` | planned | S |
| Setup + doctor | configuration | Install multi-agents, readiness par tâche | `src/application/Doctor.ts:186-210`, `docs/installation.md:1-60` | config-source → `ConfigSource` → `omega-config` | planned | M |
| Skill + tool-design | extension | Doctrine de conception d'outils réutilisable | `.agents/skills/reverse-engineer-anything/SKILL.md:1-60`, `docs/tool-design.md:1-80` | tool-surface → `ToolSurface` → `plugins/skill-rea` | planned | S |
| Limites/refus | securite | Pas d'exécution, unknown ≠ absence, pas de fallback silencieux | `docs/tool-design.md:1-80`, `docs/mcp-contracts.md:489-534` | exec-policy → `ExecPolicy` | planned | S |

## 4. Traduction en plugins

Le mode d'intégration naturel est **un fournisseur MCP externe + un plugin pont** :

- `plugins/mcp-rea` (capacité `tool-surface`, port `ToolSurface`) : déclare le serveur REA comme fournisseur d'outils du registre, avec `binary_session` comme source de vérité du catalogue et digest de catalogue pour le versioning. C'est l'intégration prioritaire (a) : **utiliser** REA tel quel.
- `plugins/tool-rea-*` (capacité `tool`, port `ToolExecutor`) : un adaptateur fin par famille (native, js, dotnet, android, wasm, evm, firmware, captures, sqlite, crash, functions, evidence) qui mappe nos appels vers les outils MCP et valide les records Evidence (couverture `complete/unknown`, `limitations`).
- `plugins/sandbox-rea-capture` (capacité `sandbox-backend`, port `SandboxBackend`) : enveloppe Process Capture **en rappelant explicitement que ce n'est pas un sandbox** (hérite uid+env) — usage réservé aux comparaisons build/autorité vs reconstruction.
- `plugins/skill-rea` : la doctrine `tool-design.md` (formes inspect/search/trace/compare/workflow/observe, primitives vs workflows) comme référence pour nos propres plugins d'analyse.

Tout est `planned` : honnêteté oblige, rien de REA n'existe encore dans notre code (aucun `adopted`).

## 5. Écarté

- **Exécution de cibles** (WASM/EVM/firmware/charges) : refusé par design des deux côtés — REA ne le fait pas, nous non plus (`rea-limits-refusals`, `integration: rejected` au sens non-reprise).
- **Fallback provider silencieux et inférence d'absence** : explicitement rejetés (sélection déterministe, `unknown` ≠ absence).
- **Moteurs d'analyse propriétaires** (Hopper payant, IDA) : non requis ; Ghidra + Rizin + objdump + GDB couvrent le besoin offline, ce sont des dépendances caller-supplied.
- **Replay engines / langages d'orchestration custom** : la doctrine REA elle-même les déconseille sans besoin observé.

## 6. Sources

- Repo cloné en `/home/tayk0oo/omega-re/rea` (commit depth-1 du 2026-10-16) : `src/contracts/*`, `src/domain/evidence.ts`, `src/cli.ts`, `src/application/Doctor.ts`, `src/ghidra|hopper|ida|rizin|objdump|gdb|dotnet|firmware|evm|wasm|sqlite|process/`, `docs/*.md`, `.agents/skills/reverse-engineer-anything/SKILL.md`, `docs/architecture.mermaid`.
- URLs : https://github.com/morluto/rea, catalogue `docs/mcp-contracts.md`, installation `docs/installation.md`.
