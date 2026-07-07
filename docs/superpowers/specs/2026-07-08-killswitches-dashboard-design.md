# Kill-switches Dashboard — Design (Sub-projet B)

**Date:** 2026-07-08
**Statut:** Validé, prêt pour planification
**Dépend de:** Sub-projet A (live-indicators cockpit) — réutilise ses classes CSS de chips.

## Objectif

Rendre **visibles depuis l'UI** les ~35 kill-switches (`ODYSSEUS_*` et gates apparentés) qui pilotent les modules dormants d'odysseusAgentOS. Aujourd'hui leur seul moyen d'inspection est la lecture des variables d'environnement / `docker-compose.yml`. Le dashboard affiche l'état réel de chaque switch, groupé par catégorie, dans le style visuel du cockpit M4.

**Portée de cette itération : lecture seule.** Aucun toggle, aucune mutation d'état. Un incrément ultérieur (option B — écriture `.env` avec redémarrage) pourra s'ajouter ; il est hors périmètre ici.

## Principe directeur : honnêteté

Comme le cockpit de A, le dashboard ne fabrique jamais de valeur. Pour chaque switch :
- Si la variable d'env est settée → afficher sa valeur effective + son état ON/OFF résolu.
- Si elle n'est pas settée → afficher le **défaut du code** explicitement labellé `(défaut)`, chip muted.
- Chaque descripteur cite sa **source** (`fichier:ligne`) où le switch est réellement lu — traçabilité vérifiable.
- Le champ `timing` distingue `startup` (lu une fois au boot — un futur toggle serait sans effet sans redémarrage) de `runtime` (lu par run).

## Architecture

Trois unités, alignées sur les conventions existantes du repo :

```
src/killswitch_registry.py   (module pur — comme src/sse_indicators.py)
        │  read_states()  → liste de descripteurs enrichis (effective, is_default)
        ▼
routes/killswitch_routes.py  (bare APIRouter — comme routes/phase_routes.py)
        │  GET /api/killswitches  → {"ok", "categories", "switches"}
        ▼
static/index.html #settings-modal → panneau "System" (existant)
        │  nouveau <div class="admin-card"> kill-switches
        ▼
static/js/admin.js  initKillswitchesView()  (façon initLogsView())
        │  fetch /api/killswitches → rendu chips (classes cockpit de A)
```

### Unité 1 — `src/killswitch_registry.py` (nouveau)

Module pur, sans I/O de fichier, testable via `monkeypatch.setenv`.

**Constante `_SWITCHES`** : liste curée de descripteurs. Un descripteur =
```python
{
    "name": "Live orchestration",          # libellé humain
    "env_var": "ODYSSEUS_LIVE_ORCHESTRATION",
    "default": "off",                       # défaut tel que lu dans le code
    "category": "Orchestration",
    "timing": "runtime",                    # "startup" | "runtime"
    "desc": "Active la CanonicalLoop 7 phases.",
    "source": "src/agent_loop.py:2493",     # lieu de lecture réel
}
```

**Fonction `read_states() -> list[dict]`** : pour chaque descripteur, lit
`os.environ.get(env_var)` et ajoute :
- `raw`: la valeur brute d'env (`None` si non settée),
- `is_default`: `True` si la var n'est pas settée,
- `effective`: bool résolu selon la convention truthy du repo
  (`.strip().lower() in ("1","true","yes","on")`), OU la valeur brute pour
  les switches non-booléens (ex. `ODYSSEUS_MISTRAL_REASONING_EFFORT`).

**Fonction `categories() -> list[str]`** : ordre d'affichage stable des catégories.

Aucun effet de bord. Ne lève jamais (best-effort par descripteur).

#### Set canonique de switches (ancré dans le code, non inventé)

Enumérés par `rg 'os\.(environ\.get|getenv)\("ODYSSEUS_'` + les gates agents/channels/services.
La tâche d'implémentation fige la liste exacte depuis le code ; défauts vérifiés ci-dessous.

| Catégorie | Switch | Défaut | Timing | Source |
|---|---|---|---|---|
| Orchestration | `ODYSSEUS_LIVE_ORCHESTRATION` | off | runtime | src/agent_loop.py:2493 |
| Orchestration | `ODYSSEUS_PHASE_TRACKER` | off | runtime | src/orchestrator/phase_tracker.py:25 |
| Orchestration | `ODYSSEUS_MODEL_ROUTER` | off | runtime | src/orchestrator/router_advice.py:42 |
| Orchestration | `ODYSSEUS_DESTRUCTIVE_GATE` | **on** | runtime | src/orchestrator/gate.py:26 |
| Gov/Mémoire | `ODYSSEUS_AUTOEVAL` | off | runtime | src/orchestrator/autoeval.py:44 |
| Gov/Mémoire | `ODYSSEUS_AUTOEVOLVE` | off | runtime | src/orchestrator/autoevolve.py:18 |
| Gov/Mémoire | `ODYSSEUS_CODEBURN` | off | runtime | src/orchestrator/codeburn_runner.py:26 |
| Gov/Mémoire | `ODYSSEUS_GOVERNANCE_ANCESTRY` | off | runtime | src/orchestrator/ancestry_tracker.py:26 |
| Gov/Mémoire | `ODYSSEUS_CHECKPOINT` | off | runtime | src/orchestrator/checkpoint_tracker.py:39 |
| Gov/Mémoire | `ODYSSEUS_UNIFIED_TOKENS` | off | runtime | src/trace_writer.py:146 |
| RAG | `ODYSSEUS_RRF_FUSION` | off | runtime | src/rag_vector.py:50 |
| MCP/Services | `ODYSSEUS_DISABLE_MCP` | off | **startup** | src/builtin_mcp.py:89 |
| MCP/Services | `ODYSSEUS_OBSIDIAN_MCP` | off | runtime | src/builtin_mcp.py:98 |
| MCP/Services | `ODYSSEUS_GRAPHIFY` | off | runtime | src/builtin_mcp.py:107 |
| MCP/Services | `ODYSSEUS_ZEN_FROM_ENDPOINT` | off | runtime | src/zen_router.py:131 |
| Channels | `ODYSSEUS_INPROCESS_DISCORD` | off | startup | src/channel_bootstrap.py |
| Channels | `ODYSSEUS_INPROCESS_TELEGRAM` | off | startup | src/channel_bootstrap.py |
| Channels | `ODYSSEUS_CHANNEL_AGENT_REPLY` | off | runtime | src/channel_bootstrap.py |
| Agents | `ODYSSEUS_AGENT_CATALOG` | off | startup | app.py:1254 |
| Agents | `ODYSSEUS_AGENT_CONSTITUTION` … `_OPEN_DESIGN` (12) | off | runtime | src/orchestrator/agent_dispatcher.py |

Note : `ODYSSEUS_DATA_DIR`, `ODYSSEUS_MAIL_ATTACHMENTS_DIR`, `ODYSSEUS_INTERNAL_BASE`,
`ODYSSEUS_SCRIPT_HOST`, `ODYSSEUS_COPILOT_*`, `ODYSSEUS_MISTRAL_REASONING_EFFORT` sont des
**paramètres de configuration**, pas des kill-switches booléens. Ils sont **exclus** de cette
itération (YAGNI) — le dashboard cible les on/off de modules.

### Unité 2 — `routes/killswitch_routes.py` (nouveau)

Calqué strictement sur `routes/phase_routes.py` :

```python
from fastapi import APIRouter
from src.killswitch_registry import read_states, categories

router = APIRouter(prefix="/api/killswitches", tags=["killswitches"])

@router.get("")
async def list_killswitches():
    """Retourne l'état réel de tous les kill-switches (lecture seule)."""
    try:
        return {"ok": True, "categories": categories(), "switches": read_states()}
    except Exception as e:
        return {"ok": False, "error": str(e)}
```

**Enregistrement** dans `app.py`, bloc M3 (~ligne 816, après autoeval) :
```python
# ========= KILL-SWITCHES DASHBOARD (Sub-projet B — lecture seule) =========
from routes.killswitch_routes import router as killswitch_router
app.include_router(killswitch_router)
```

### Unité 3 — Frontend (panneau System existant)

**HTML** — nouveau `admin-card` dans `static/index.html`, panneau
`data-settings-panel="system"` (après la carte Terminal Logs, ~ligne 2357) :
```html
<div class="admin-card" id="settings-killswitches-card">
  <h2><svg …></svg>Kill-switches</h2>
  <div class="admin-toggle-sub">État réel des modules dormants (lecture seule).
    Défaut = valeur si la variable d'environnement n'est pas définie.</div>
  <div id="killswitches-container">
    <div class="settings-system-logs-placeholder">Chargement…</div>
  </div>
</div>
```

**JS** — `initKillswitchesView()` ajouté à `static/js/admin.js`, façon `initLogsView()` :
- `fetch('/api/killswitches', {credentials:'same-origin'})`.
- Groupe `switches` par `category` (ordre de `categories`).
- Par switch, rend une ligne : libellé + chip d'état + méta.
  - Chip **ON** effectif → `cockpit-chip chip-ok` (réutilise A).
  - Chip **OFF** effectif → `cockpit-chip chip-muted`.
  - `ODYSSEUS_DESTRUCTIVE_GATE` ON (protection active) → `chip-ok` ; OFF → `chip-warn`.
  - Badge `(défaut)` si `is_default`, badge `startup` si `timing==="startup"`.
  - Nom de la var en `<code>` + `title` = `desc` + `source`.
- Appelé au 1er affichage du panneau System (pas de polling — état quasi statique ;
  un bouton "Refresh" relit à la demande, comme les logs).
- **Sanitisation** : mêmes règles que A — `String(x).replace(/[<>&]/g,'')` sur tout
  texte injecté (les valeurs viennent de l'env, potentiellement arbitraires).

**CSS** — réutilise les classes de A (`.cockpit-chip`, `.chip-ok/muted/warn`).
Ajouts minimes dans `static/css/style.min.css` si besoin : `.ks-row` (flex libellé↔chip),
`.ks-badge` (petit label `(défaut)`/`startup`), `.ks-cat-title`.

## Flux de données

1. Ouverture settings → onglet System → `initKillswitchesView()` (une fois).
2. `GET /api/killswitches` → `read_states()` lit l'`os.environ` courant.
3. Rendu groupé ; bouton Refresh re-fetch.
4. Aucun POST, aucune écriture. `os.environ` n'est jamais muté.

## Gestion d'erreurs

- Registry : chaque descripteur résolu en best-effort ; un `env_var` illisible →
  `effective=None`, chip muted. La liste ne casse jamais.
- Route : `try/except` → `{"ok": False, "error": str(e)}` (convention repo).
- Frontend : fetch échoué / `ok:false` → placeholder « indisponible » dans la carte,
  jamais d'état inventé.

## Tests

- **`tests/test_killswitch_registry.py`** (pytest, `monkeypatch.setenv`) :
  - var non settée → `is_default True`, `effective` == défaut résolu.
  - var settée `"on"` → `effective True`, `is_default False`.
  - var settée `"off"` → `effective False`.
  - `ODYSSEUS_DESTRUCTIVE_GATE` non settée → `effective True` (défaut on).
  - chaque descripteur a les clés requises + un `source` non vide.
  - `categories()` couvre toutes les catégories présentes dans `_SWITCHES`.
- **Route** : test smoke via `TestClient` → `GET /api/killswitches` renvoie `ok:True`
  et une liste non vide (cohérent avec `tests/` existants).

## Hors périmètre (YAGNI)

- Toggle / écriture (`.env` ou `os.environ`) — itération B ultérieure.
- Paramètres de configuration non-booléens (chemins, IDs, modèles).
- Validation de cohérence inter-switches (ex. 2 orchestrateurs concurrents) —
  relève du Sub-projet D (consolidation orchestrateurs).
- Persistance / historique des changements.

## Critères de succès

1. `GET /api/killswitches` renvoie l'état réel de tous les switches du set canonique.
2. Le panneau System affiche les switches groupés, style cockpit, avec état honnête
   (défaut explicite quand non setté).
3. Aucun effet de bord : relire le dashboard ne change aucun comportement du runtime.
4. Tests registry + route au vert ; `import app` boote proprement.
