# Trinité Knowledge Panel — Design (Sub-projet C)

**Date:** 2026-07-08
**Statut:** Validé, prêt pour planification
**Dépend de:** Sub-projet A (classes de chips cockpit) et B (pattern admin-card + placement System panel).

## Objectif

Rendre **visible depuis l'UI** l'état de santé en temps réel de la **Trinité de connaissance** d'odysseusAgentOS : les trois jambes qui alimentent la génération contextuelle. Aujourd'hui leur seul moyen d'inspection est un appel manuel à `GET /api/knowledge/status`. Le panneau affiche l'état de chaque jambe, dans le style visuel du cockpit M4 et de la carte kill-switches (B).

**Portée de cette itération : lecture seule, santé uniquement.** Pas de boîte de recherche, pas de statistiques (comptes de documents/notes/nœuds), pas de nouvel endpoint backend. Ces extensions relèvent d'un incrément ultérieur.

## Principe directeur : honnêteté

Comme A et B, le panneau ne fabrique jamais de valeur.
- CBM et VectorRAG exposent un **vrai** état `online`/`offline` (health-check réel).
- Obsidian n'a **pas** de health-check : `/status` renvoie une chaîne descriptive fixe
  (`"delegated (native checkpoint_tracker + ODYSSEUS_OBSIDIAN_MCP gate)"`). Son chip doit
  donc être **neutre** (`chip-muted`, libellé « delegated »), jamais vert « online ».
- Un fetch échoué → chip muted « indisponible », jamais un état deviné.

## La Trinité (ancrée dans le code)

Source : `routes/knowledge_routes.py:164-176`.

| Jambe | Rôle | État exposé par `/status` | Nature |
|---|---|---|---|
| **CBM** | Graphe de code (fonctions/classes/routes), port 9749 | `"online"` / `"offline"` | health-check réel (`GET :9749/health`) |
| **VectorRAG** | Recherche sémantique documentaire (ChromaDB) | `"online"` / `"offline"` | présence du singleton RAG |
| **Obsidian** | Mémoire persistante (notes markdown) | chaîne fixe `"delegated (…)"` | **pas** de health-check — statut descriptif |

**Hors périmètre :** Graphify (4ᵉ jambe expérimentale, service « fantôme » jamais démarré,
gate `ODYSSEUS_GRAPHIFY`, endpoint séparé `/graph/status`). Exclu (YAGNI). L'état de son
kill-switch est déjà visible via la carte kill-switches (Sub-projet B).

## Architecture

Réutilise strictement les conventions de A et B. **Aucun nouveau backend.**

```
routes/knowledge_routes.py  (EXISTANT, non modifié)
        │  GET /api/knowledge/status → {"trinite": {"cbm","rag","obsidian"}}
        ▼
static/index.html #settings-modal → panneau "System"
        │  nouveau <div class="admin-card"> Trinité, après la carte kill-switches (B)
        ▼
static/js/admin.js  initKnowledgeView()  (façon initKillswitchesView())
        │  fetch /api/knowledge/status → rendu chips (classes cockpit de A)
```

### Unité 1 — Backend (aucun changement)

`GET /api/knowledge/status` existe déjà et suffit. Sa forme est figée :
```json
{"trinite": {"cbm": "online|offline", "rag": "online|offline",
             "obsidian": "delegated (native checkpoint_tracker + ODYSSEUS_OBSIDIAN_MCP gate)"}}
```
Le panneau consomme ce contrat tel quel. **On ne touche pas** cet endpoint (il sert
aussi à d'autres consommateurs). Si un test révèle une régression de forme, on la
corrige dans le test, pas dans l'endpoint.

### Unité 2 — Frontend HTML (panneau System existant)

Nouveau `admin-card` dans `static/index.html`, panneau `data-settings-panel="system"`,
**après** la carte kill-switches (`id="settings-killswitches-card"`, Sub-projet B) :
```html
<div class="admin-card" id="settings-knowledge-card">
  <h2><svg …></svg>Trinité — Connaissance</h2>
  <div class="admin-toggle-sub">Santé des trois jambes de connaissance (lecture seule).</div>
  <div class="settings-system-logs-controls">
    <button type="button" class="admin-btn-sm" id="knowledge-refresh-btn">… Refresh</button>
  </div>
  <div id="knowledge-container">
    <div class="settings-system-logs-placeholder">Chargement…</div>
  </div>
</div>
```

### Unité 3 — Frontend JS

`initKnowledgeView()` ajouté à `static/js/admin.js`, façon `initKillswitchesView()` :
- `fetch('/api/knowledge/status', {credentials:'same-origin'})`.
- Lit `data.trinite`. Pour chaque jambe, rend une ligne `.ks-row` : libellé + chip.
  - CBM / RAG : valeur `"online"` → `cockpit-chip chip-ok` (« ON »/« online ») ;
    toute autre valeur → `chip-muted` (« offline »).
  - Obsidian : **toujours** `chip-muted`, libellé « delegated » (jamais online).
  - `title` de la ligne = description courte de la jambe.
- Bouton Refresh → re-fetch (pas de polling — état quasi-statique, comme B).
- Ajouté au tableau `inits` de `initAll()`, après `initKillswitchesView`.
- **Sanitisation** : réutilise `ksClean()` (déjà durci sur `<>&"'` en B) sur tout texte injecté.
- **Erreur** : fetch échoué / payload invalide → placeholder « indisponible », jamais d'état inventé.

### Unité 4 — CSS

Réutilise `.ks-row`, `.ks-name`, `.ks-cat-title` (B) et `.cockpit-chip`, `.chip-ok`,
`.chip-muted` (A). **Aucun nouveau CSS attendu.**

## Flux de données

1. Ouverture settings → onglet System → `initKnowledgeView()` (une fois).
2. `GET /api/knowledge/status` → lecture de l'état réel des jambes.
3. Rendu de 3 chips ; bouton Refresh re-fetch.
4. Aucun POST, aucune écriture, aucun effet de bord.

## Gestion d'erreurs

- Réseau / `status != 200` / JSON invalide → placeholder « indisponible » dans la carte.
- Jambe absente du payload → chip muted « ? », jamais un état deviné.
- Obsidian : statut descriptif non booléen → chip muted « delegated », par conception.

## Tests

- **`tests/test_knowledge_panel.py`** (ou extension d'un test existant) :
  - `GET /api/knowledge/status` renvoie 200 et une clé `trinite` avec les 3 jambes
    `cbm`, `rag`, `obsidian` (contrat que le frontend consomme).
- **Frontend** : vérification statique que `static/index.html` contient
  `settings-knowledge-card` + `knowledge-container` + `knowledge-refresh-btn`, et que
  `static/js/admin.js` contient `initKnowledgeView` câblé dans le tableau `inits`.
- `rtk node --check static/js/admin.js` (tolérant si node absent).
- Smoke : `import app` boote proprement.

## Hors périmètre (YAGNI)

- Boîte de recherche / knowledge-explorer interactif (checkpoint, code/search).
- Statistiques (comptes docs/notes/nœuds, timestamps) — nécessiterait un nouvel endpoint.
- Graphify (4ᵉ jambe gated) — kill-switch déjà visible en B.
- Toute mutation / écriture.

## Critères de succès

1. Le panneau System affiche les 3 jambes de la Trinité, style cockpit, avec état honnête
   (Obsidian jamais présenté comme « online »).
2. Aucun nouveau backend ; l'endpoint `/api/knowledge/status` existant est réutilisé tel quel.
3. Aucun effet de bord : relire le panneau ne change aucun comportement du runtime.
4. Test de contrat `/status` + vérifications de câblage frontend au vert ; `import app` boote.
