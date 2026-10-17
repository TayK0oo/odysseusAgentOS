# REA — sur quoi il peut agir concrètement (inventaire vérifié)

> État au **2026-10-17**, machine Nobara 44 (x86_64). Vérifié par `rea providers`
> et par des analyses réelles. Toolchain : `/home/tayk0oo/omega-re/rea-tools.env`
> (installé en `/etc/profile.d/rea-tools.sh` + `~/.config/environment.d/rea-tools.conf`).

## 1. Deux couches

**Couche REA (toujours là, rien à installer)** : analyse JavaScript/Electron,
graphe d'artefacts, analyse .NET statique, inspection SQLite, adaptateur ELF
(pwntools/pyelftools/Unicorn fournis par l'appelant).

**Couche moteurs externes (installés ici)** : REA **pilote** des outils tiers, il
ne les embarque pas et **n'installe jamais** rien lui-même.

## 2. Ce qui est **opérationnel sur cette machine**

| Cible | Moteur | État | Preuve |
|---|---|---|---|
| **JS/TS** (graphe de modules, frontières Electron, endpoints, storage) | REA `rea-javascript-application` | ✅ | 18 harnais + notre cockpit |
| **ELF** (sections, segments, symboles, relocations, mitigations) | `pwntools-elf` | ✅ | binaire de crate, `observed` |
| **Binaires natifs — analyse PROFONDE** (fonctions, appels, types, instructions, décompilation) | **Ghidra 12.1.4** | ✅ **21 groupes d'ops** — 8/8 contrôles OK | `GHIDRA_INSTALL_DIR`, JDK 25 accepté |
| **.NET / PE managé** | `rea-dotnet-static` + **ilspycmd 11.1.0** | ✅ | 3 ops disponibles |
| **Android** (APK, dex, ressources) | **jadx 1.5.6** + **apktool 2.11.1** + **adb** | ✅ | installés, chemins déclarés |
| **Réseau** (pcap/HAR) | **tshark 4.6.9** + **capinfos** | ✅ | `REA_TSHARK_COMMAND` |
| **Firmware** | **binwalk 3.1.0** + **unblob** | ✅ | `REA_BINWALK_COMMAND`, `REA_UNBLOB_COMMAND` |
| **Graphe d'artefacts** (archives, ASAR, catalogues) | REA `rea-artifact-graph` | ✅ | 5 ops |
| **SQLite** | REA `sqlite` | ✅ | 1 op |
| **Désassemblage alternatif** | **rizin** | ✅ | `REA_RIZIN_COMMAND` |
| **Débogage** | **gdb 17.2** | ✅ | `REA_GDB_COMMAND` |
| **Capture navigateur / web** | **chromium** | ✅ | `REA_BROWSER_EXECUTABLE` |
| **Instrumentation dynamique** | **frida** | ✅ | venv dédié |
| **WASM** | **wabt** (`wasm2wat`) | ✅ | `REA_WABT_BIN_DIRECTORY` |
| **Signaux logiques** | **sigrok-cli** | ✅ | `REA_SIGROK_COMMAND` |
| **Interception réseau** | **mitmdump** | ✅ | `REA_MITMDUMP_COMMAND` |

**Résultat mesurable** : `rea providers` est passé de **4 providers** (dont 1
inutilisable) à **5 providers**, dont **Ghidra avec 21 groupes d'opérations
disponibles** — c'est-à-dire la décompilation et l'analyse de fonctions sur
**n'importe quel binaire**, y compris notre moteur Rust.

## 3. Ce qui reste **impossible** (et pourquoi)

| Cible | Raison |
|---|---|
| Inspection **macOS** (Mach-O UI, asset catalogs, Swift) | `unsupported_host` — nécessite macOS. Structurel. |
| **Hopper** | payant (macOS/Linux commercial) |
| **IDA Pro** | payant (licence commerciale) |
| **JEB** | payant (licence commerciale) |
| **Cutter** (GUI rizin) | non installé (rizin CLI suffit à REA) |
| **Wakaru** (JS déobfuscation) | non installé (`REA_WAKARU_COMMAND`) |

## 4. Limites de REA lui-même (même tout installé)

- **`REA_PWNTOOLS_MAX_INPUT_BYTES`** : plafond d'entrée pour l'adaptateur ELF
  (32 Mio par défaut ; relevé à **256 Mio** ici).
- **`resource_constraint`** sur les dépôts très volumineux (OpenClaw 945 Mo,
  Void, Qwen Code) → cibler un sous-dossier.
- **Pas d'analyseur de source Python/Rust/Go** : sur ces dépôts, seule la partie
  JS incidente est vue (graphe minuscule, `derived`).
- **Analyse statique** : REA ne prouve pas le comportement d'exécution ; il le
  déclare dans ses `limitations` (5 à 12 par analyse, parfois des milliers sur
  les monorepos).
- **Sessions éphémères** : les bases Ghidra sont temporaires, supprimées à la
  fermeture ; les octets d'origine ne sont jamais modifiés.

## 5. Configuration

```bash
# Le toolchain est décrit dans un seul fichier, versionnable :
cat /home/tayk0oo/omega-re/rea-tools.env

# Notre pont Rust le charge automatiquement (omega-mcp) :
#   $OMEGA_REA_ENV  >  ~/.config/omega/rea-tools.env  >  /etc/profile.d/rea-tools.sh

# Vérifier ce que REA voit :
npx -y rea-agents@latest providers
npx -y rea-agents@latest doctor
```

Variables clés : `GHIDRA_INSTALL_DIR`, `JAVA_HOME`, `REA_PWNTOOLS_PYTHON`,
`REA_JADX_MCP_JAR`/`REA_JADX_HEAP_MIB`, `REA_APKTOOL_COMMAND`, `REA_TSHARK_COMMAND`,
`REA_BINWALK_COMMAND`, `REA_UNBLOB_COMMAND`, `REA_RIZIN_COMMAND`, `REA_GDB_COMMAND`,
`REA_ILSPY_CMD_PATH`, `REA_WABT_BIN_DIRECTORY`, `REA_SIGROK_COMMAND`,
`REA_BROWSER_EXECUTABLE`, `REA_ADB_PATH`, `REA_MITMDUMP_COMMAND`,
`REA_GHIDRA_STARTUP_TIMEOUT_MS`.
