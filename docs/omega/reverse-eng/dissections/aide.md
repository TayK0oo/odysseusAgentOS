# Dissection — Aide (nicepkg, AGPL-3.0, TypeScript)

> Source : clone `--depth 1` de https://github.com/nicepkg/aide dans `/home/tayk0oo/omega-re/aide`.
> Donnees : `omega/crates/omega-harness/data/apps/aide.json` (9 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | nicepkg |
| Langage | TypeScript |
| Repo | https://github.com/nicepkg/aide |
| Architecture | Extension VSCode de refonte de code : commandes de transformation (batch-processor, expert-code-enhancer, smart-paste...), providers LLM LangChain (azure-openai/claude/openai) a base URLs custom, context reference paths, webview React, i18n, snapshot tmp + reversion avant refonte. |

## 2. Inventaire fonctionnel

### `ad-model-providers` — ModelProviders LangChain : azure-openai/claude/openai sur BaseModelProvider

- **domaine** : routage · **capacite** : llm-provider · **port** : Provider · **cout** : S · **statut** : planned
- **what** : Trois providers LLM (azure-openai, claude, openai) derives d'un BaseModelProvider LangChain avec options communes (structured output).
- **how** : src/extension/ai/model-providers/base.ts:37 (abstract class BaseModelProvider<Model extends BaseChatModel>) + {azure-openai,claude,openai}.ts. Omega : omega-providers couvert.
- **adaptateur Omega** : omega-providers (BaseModelProvider LangChain)
- **preuves** : `src/extension/ai/model-providers/base.ts:37`

### `ad-commands` — Commands VSCode : 9 commandes de transformation de code

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : S · **statut** : planned
- **what** : Un jeu de 9 commandes VSCode : batch-processor, code-convert, expert-code-enhancer, rename-variable, smart-paste, ask-ai, copy-as-prompt, comment-shortcuts, plus webview.
- **how** : src/extension/commands/ 9 dossiers + index.ts par commande ; expert-code-enhancer/default-prompt-list.ts (prompts). Omega : ToolSurface (commandes = tools declares).
- **adaptateur Omega** : omega-app (commandes = interface tools)
- **preuves** : `src/extension/commands/expert-code-enhancer/index.ts` · `src/extension/commands/batch-processor/write-and-save-tmp-file.ts`

### `ad-reference-paths` — get-reference-file-paths : collecte du contexte de references

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : S · **statut** : planned
- **what** : Une collecte de chemins de fichiers de reference pour la commande courante (contexte implicite lie a l'action).
- **how** : src/extension/ai/get-reference-file-paths.ts + src/extension/context.ts. Omega : CatalogSource (contexte structure).
- **adaptateur Omega** : omega-context (chemins de reference)
- **preuves** : `src/extension/ai/get-reference-file-paths.ts`

### `ad-custom-llm-url` — parse-model-base-url : endpoints custom (OpenAI-compatible) par modele

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : Configuration de base URLs custom par fournisseur (proxies, endpoints compatibles OpenAI) parse par parse-model-base-url.
- **how** : src/extension/ai/parse-model-base-url.ts + src/extension/config.ts. Omega : omega-providers (endpoint override) deja note ; retenu : le parse des base-url custom.
- **adaptateur Omega** : omega-providers (base-url custom par modele)
- **preuves** : `src/extension/ai/parse-model-base-url.ts`

### `ad-storage` — storage : persistence locale des preferences et historiques

- **domaine** : memoire · **capacite** : memory-store · **port** : MemoryStore · **cout** : S · **statut** : planned
- **what** : Une couche de stockage locale (storage.ts) pour preferences/historiques du webview.
- **how** : src/extension/storage.ts. Omega : MemoryStore couvert.
- **adaptateur Omega** : omega-app::FileMemoryStore
- **preuves** : `src/extension/storage.ts`

### `ad-webview` — Webview React : toute l'UI dans le panel VSCode

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : S · **statut** : planned
- **what** : L'UI est un webview React (src/webview) communique avec l'extension par messages.
- **how** : src/webview/ + src/extension/webview.ts. Omega : omega-ui couvert ; le pattern webview est standard IDE.
- **adaptateur Omega** : omega-ui (webview panel)
- **preuves** : `src/webview`

### `ad-git-diff` — Snapshot/diff de refonte : sauvegarde temporaire puis reversion

- **domaine** : agents · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : S · **statut** : planned
- **what** : Les refontes batch savent un tmp file avant application : prevention de revient + diff git.
- **how** : src/extension/commands/batch-processor/write-and-save-tmp-file.ts. Omega : effets reversibles (snapshot) deja couvert (Phase A).
- **adaptateur Omega** : omega-sandbox (snapshot + reversion)
- **preuves** : `src/extension/commands/batch-processor/write-and-save-tmp-file.ts`

### `ad-i18n` — i18n EN/ZH-JP : localization des commandes et webview

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : S · **statut** : planned
- **what** : L'internationalisation des commandes et webview est une couche separate (package.nls.en.json + zh-cn, i18n.ts).
- **how** : package.nls.en.json + package.nls.zh-cn.json + src/extension/i18n.ts. Omega : externabilise la langagier.
- **adaptateur Omega** : omega-config (i18n en couche)
- **preuves** : `src/extension/i18n.ts` · `package.nls.en.json`

### `ad-auto-open` — Auto-open correspondant : fichier jumeau ouvert a l'edition

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : S · **statut** : planned
- **what** : Un confort d'IDE : auto-open-corresponding-files.ts ouvre le fichier correspondant (service/interface, test/source) de celui en edition.
- **how** : src/extension/auto-open-corresponding-files.ts. Omega : a l'agent, pas au centre : plugin d'IDE.
- **adaptateur Omega** : omega-ui (auto-open jumeau)
- **preuves** : `src/extension/auto-open-corresponding-files.ts`

