# Dissection — Claude Engineer (Doriandarko, MIT, Python)

> Source : clone `--depth 1` de https://github.com/Doriandarko/claude-engineer dans `/home/tayk0oo/omega-re/claude-engineer`.
> Donnees : `omega/crates/omega-harness/data/apps/claude-engineer.json` (8 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Doriandarko |
| Langage | Python |
| Repo | https://github.com/Doriandarko/claude-engineer |
| Architecture | Console + Flask app d'agent pair-programming Anthropic : un Assistant (ce3.py) qui decouvre/compile des tools Python par ABC BaseTool (input_schema JSON), self-extension via ToolCreator, deps auto-installees par uv, sandbox e2b pour le run code. |

## 2. Inventaire fonctionnel

### `ce-tools-schema` — BaseTool : ABC declare name/description/input_schema/execute

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : S · **statut** : planned
- **what** : Un contrat Python d'outils simple : chaque outil est une classe ABC avec name, description, input_schema (JSON Schema) et execute(kwargs)->str.
- **how** : tools/base.py:4-24 class BaseTool(ABC) : name (:7), description (:13), input_schema (:19), execute (:24). Omega : ToolSurface (spec JSON) : le pattern de extraction des schemas par ABC est le meme, retenu. La surface Omega est deja schema-specified (ToolSurfaceIndex).
- **adaptateur Omega** : omega-context::ToolSurfaceIndex (schema JSON de base ABC)
- **preuves** : `tools/base.py:4-24`

### `ce-tool-auto-refresh` — Auto-decouverte/register des outils par dossier au run

- **domaine** : configuration · **capacite** : registry · **port** : — · **cout** : S · **statut** : rejected
- **what** : Les outils sont auto-decouverte au run (parsés par module), refresh a chaud.
- **how** : _load_tools (ce3.py:75) scan le dossier tools/, _extract_tools_from_module (ce3.py:141) ; refresh_tools (:159) pour reload unاحة parfait. Omega : registre de plugins + scan deja couvert.
- **adaptateur Omega** : omega-app (auto-discover tools)
- **preuves** : `ce3.py:75-141`

### `ce-uv-package-manager` — uv package manager : installation dependances missing a chaud

- **domaine** : configuration · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : rejected
- **what** : Quand le model manque un module Python, l'agent extraie le nom du module et l'execute uv install en background.
- **how** : _clean_and_install (_execute_uv_install : ce3.py:55-60) : parse le missing dependency (ce3.py:130) et execute 'uv pip install' pour l'installer a chaud. Omega : l'execution de paquets automatique est un plugin tool 'install dependencies', pas un noyau.
- **adaptateur Omega** : plugin/install-deps (uv install automatique)
- **preuves** : `ce3.py:55-60` · `ce3.py:130-141`

### `ce-toolcreator` — ToolCreator : l'agent cree ses propres outils a chaud

- **domaine** : extension · **capacite** : registry · **port** : — · **cout** : M · **statut** : planned
- **what** : Un outil qui demande au modele d'ecrire un module Python generant une nouvelle classe BaseTool, puis refresh les outils — l'agent s'agrandit son arsenal.
- **how** : tools/toolcreator.py:12-44 class ToolCreatorTool generate un module Python avec une classe qui suit BaseTool, avec sanitation du nom (_validate_tool_name :40) ; refresh_tools ensuite (ce3.py:159). Omega : self-extension de l'agent : est possible via un tool 'build plugin', à condition que les plugins soient valides (schema-validé manifeste) ; le pattern est bon
- **adaptateur Omega** : omega-app (tool-creation par l'agent)
- **preuves** : `tools/toolcreator.py:12-44`

### `ce-browser` — Browser + DuckDuckGo + webscraper : le trio de recherche web

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : S · **statut** : planned
- **what** : Trois outils web indépendants : browser (navigateur), duckduckgo (search), webscraper (contenu).
- **how** : tools/{browsertool.py,duckduckgotool.py,webscrapertool.py}. Omega : ToolSurface ; la browser est un plugin de web tools (pas un noyau).
- **adaptateur Omega** : plugin/web-tools (browser+search+scrape)
- **preuves** : `tools/browsertool.py` · `tools/duckduckgotool.py`

### `ce-diff-editor` — FileEditor + DiffEditor + Linting : l'edition de code avec pathes diff et lint en passe

- **domaine** : execution · **capacite** : tool-surface · **port** : ToolExecutor · **cout** : S · **statut** : planned
- **what** : Un trio de tools d'edition : file creator/reader, diffeditor (patch), linting (check).
- **how** : tools/{filecreatortool.py,fileedittool.py,diffeditortool.py,lintingtool.py}. Omega : ToolExecutor couvert ; le lint est un effect réversible.
- **adaptateur Omega** : omega-app (edition + lint checks)
- **preuves** : `tools/filecreatortool.py` · `tools/diffeditortool.py`

### `ce-e2b` — E2B : l'execution de code distant s'effectue par sandbox e2b

- **domaine** : securite · **capacite** : sandbox-backend · **port** : SandboxBackend · **cout** : S · **statut** : planned
- **what** : Un tool d'execution de code Python distant en sandbox E2B (cloud) pour isolation.
- **how** : tools/e2bcodetool.py : e2b client docker. Omega : SandboxBackend ; un fournisseur cloud supplementaire (E2B) est deja note dans le census Onlook (code-provider) ; ce tool est le memeU.
- **adaptateur Omega** : plugin/e2b sandbox adapter
- **preuves** : `tools/e2bcodetool.py`

### `ce-flask-ui` — Flask UI basique : page form d'agent + screen buff

- **domaine** : ui · **capacite** : (ui) · **port** : UiPanel · **cout** : S · **statut** : rejected
- **what** : Un frontend Flask/Jinja templates lightning minimal (app.py) pour envoyer les objective et voir les sorties de l'agent.
- **how** : app.py (Flask app) + templates + static. Omega : omega-server (UI serve par une origine) couvert.
- **adaptateur Omega** : omega-server (UI Flask -> cockpit)
- **preuves** : `app.py`

