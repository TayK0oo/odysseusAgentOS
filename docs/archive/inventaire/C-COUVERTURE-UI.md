# C — COUVERTURE UI & INDICATEURS VISUELS (Axe C)

**Date** : 2026-07-07 | **Commit** : `2821daa`
**Mise à jour** : 2026-07-08 — après le lot « UI Veracity & Visibility » (cockpit live A, dashboard kill-switches B, panneau Trinité C, honnêteté chip phase `af7fe87`, panneau Drift `922c1ac`, panneau Budgets `44b8fa8`). Les lignes marquées ✅ MAJ ci-dessous sont passées de 🔴 ANGLE MORT à couvert.

---

## C.1 — SURFACES UI RECENSÉES

| Surface | Type | Emplacement | Routes | Technologie | Statut |
|---|---|---|---|---|---|
| SPA principale | Application web | `static/index.html` + `static/js/` | `/`, `/notes`, `/calendar`, `/cookbook`, `/email`, `/memory`, `/gallery`, `/tasks`, `/library` | HTML/CSS/JS vanilla (modules ES) | 🟢 |
| Login | Page dédiée | `static/login.html` | `/login` | HTML/CSS/JS vanilla | 🟢 |
| Backgrounds sandbox | Page sandbox | `static/backgrounds.html` | `/backgrounds` | HTML/CSS (prototypage effets) | 🟢 |
| PWA/Desktop | Application desktop | `build-windows-portable.ps1`, `build-macos-app.sh`, `Odysseus.spec` | - | PyInstaller (Windows), script shell (macOS) | 🟡 (build existant, distrib non vérifiée) |
| Companion mobile | API pairing | `companion/pairing.py`, `companion/routes.py` | `/api/companion/*` | FastAPI (backend only) | 🟡 (backend existant, UI mobile non vérifiée) |

---

## C.2 — MATRICE DE COUVERTURE : CAPACITÉ BACKEND → UI

| Capacité backend (réf. Axe A) | Exposée UI ? | Composant UI | Indicateur visuel | Type | Source données prouvée ? | Statut couverture | Preuve |
|---|---|---|---|---|---|---|---|
| **Chat / Agent Loop** | | | | | | | |
| Chat temps réel (SSE) | Oui | Panneau chat principal | Streaming texte + tool calls | valeur/flux | SSE stream | 🟢 | agent_loop.py (yield SSE) |
| Agent dispatch indicators (M4) | Oui | Badges dans le chat | "agent: running/completed" avec animation | statut | SSE events `agent_dispatch` | 🟢 | agent_loop.py:2558-2560 ; commit `2b58fd3` |
| Phase courante | Oui | Cockpit chip `#cockpit-phase` | Phase (BUILD/PLAN/…), neutre si orchestration OFF | statut | SSE `run_status.phase` + `phase_active` | 🟢 ✅ MAJ | cockpit.js ; honnêteté `af7fe87` (vert seulement si orchestration active) |
| Budget itérations (restant) | Oui | Cockpit chip `#cockpit-iters` | `used/max` itérations | valeur | SSE `run_status.iters` | 🟢 ✅ MAJ | cockpit.js ; sse_indicators.run_status_event |
| **Mémoire** | | | | | | | |
| Recherche mémoire vectorielle | Oui | Panneau Memory | Résultats de recherche | valeur | `/api/memory/*` | 🟢 | routes/memory_routes.py |
| Skills extraits | Oui | Panneau Memory > Skills | Liste skills + pertinence | valeur | `/api/skills/*` | 🟢 | routes/skills_routes.py |
| Distillation Acontext (auto-evolve) | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
| **RAG / Documents** | | | | | | | |
| Documents personnels (RAG) | Oui | Panneau Personal/Library | Liste documents + recherche | valeur | `/api/personal/*` | 🟢 | routes/personal_routes.py |
| Résultats recherche RAG | Oui | Chat (inline context) | Contexte injecté dans prompt | valeur | `rag_vector.search` | 🟢 | agent_loop.py (mémoire injectée) |
| **Qualité** | | | | | | | |
| Drift score (LOW/MED/HIGH) | Oui | Cockpit chip `#cockpit-drift` + panneau `#settings-drift-card` | Niveau coloré + harness touché + one-shot | statut/valeur | SSE `run_status.drift` ; `GET /api/observer/drift` | 🟢 ✅ MAJ | cockpit.js ; observer_routes `922c1ac` |
| Autoeval keep/revert | Partiel | Badge transitoire in-chat | keep/revert + raison | statut | SSE `autoeval_result` | 🟡 | chat.js:2375 ; n'apparaît que si `ODYSSEUS_AUTOEVAL=on` (bonne UX, pas de chip permanent muet) |
| Verifier subagent | Partiel | Badge transitoire in-chat | pass/fail + détail | statut | SSE `verifier_result` | 🟡 | sse_indicators.verifier_event ; capé, OFF par défaut |
| CodeBurn (one-shot rate, waste) | Partiel | Panneau `#settings-drift-card` | Taux one-shot du dernier rapport | valeur | `GET /api/observer/drift` | 🟡 ✅ MAJ | observer_routes `922c1ac` (one-shot rate exposé ; waste patterns non détaillés) |
| Teacher escalation | Non | — | — | — | — | 🔴 ANGLE MORT | agent_loop.py:3714 (interne, non exposé) |
| **Gouvernance** | | | | | | | |
| Ancestry (mission→goal→task) | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF + pas de vue UI |
| Budget token/coût par projet | Oui | Panneau `#settings-budgets-card` | Barres usage/limite tokens, coût $, itérations + chip statut | valeur/statut | `GET /api/governance/budgets` | 🟢 ✅ MAJ | governance_routes list-all + admin.js `44b8fa8` |
| Destructive gate (blocage) | Partiel | Chat (message d'erreur) | Message "commande bloquée" | alerte | tool_execution.py:565-571 | 🟡 exposé comme erreur, pas comme statut gate |
| **Canaux** | | | | | | | |
| Discord/Telegram (statut) | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
| Channel messages inbound | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
| **Connaissance (Trinité)** | | | | | | | |
| CBM (santé) | Oui | Panneau `#settings-knowledge-card` | Chip online/offline de la jambe CBM | statut | `GET /api/knowledge/status` | 🟡 ✅ MAJ | Sous-projet C : santé exposée ; recherche CBM interactive toujours non exposée |
| Graphify | Non | — | — | — | — | 🔴 ANGLE MORT | 0 svc |
| Obsidian notes | Oui | Panneau `#settings-knowledge-card` | Chip jambe Obsidian (neutre « delegated ») | statut | `GET /api/knowledge/status` | 🟡 ✅ MAJ | Sous-projet C : jambe rendue honnêtement neutre (jamais vert) ; notes elles-mêmes non exposées |
| **Modèles** | | | | | | | |
| Modèle courant | Oui | Barre supérieure | Nom du modèle actif | statut | `/api/model/*` | 🟢 | routes/model_routes.py |
| Liste modèles disponibles | Oui | Paramètres > Modèles | Tableau modèles | valeur | `/api/model/probe` | 🟢 | routes/model_routes.py |
| Endpoints configurés | Oui | Paramètres > Endpoints | Liste endpoints avec statut | valeur/statut | `/api/model-endpoints/*` | 🟢 | routes/model_routes.py |
| Model comparison (A/B) | Oui | Panneau Compare | Sorties côte à côte | valeur | `/api/compare/*` | 🟢 | routes/compare_routes.py |
| Cookbook (download/serve) | Oui | Panneau Cookbook | Progression download, modèles servis | valeur/statut | `/api/cookbook/*` | 🟢 | routes/cookbook_routes.py |
| HW fit (compatibilité GPU) | Oui | Cookbook > What Fits? | Tableau compatibilité | valeur | `/api/hwfit/*` | 🟢 | routes/hwfit_routes.py |
| **Email** | | | | | | | |
| Inbox / compose / send | Oui | Panneau Email | Liste emails, éditeur | valeur | `/api/email/*` | 🟢 | routes/email_routes.py |
| Email urgency | Oui | Panneau Email | Badge urgence (cache) | statut | `data/email_urgency_cache` | 🟢 | routes/email_helpers.py |
| **Calendrier** | | | | | | | |
| CalDAV sync + events | Oui | Panneau Calendar | Vue calendrier | valeur | `/api/calendar/*` | 🟢 | routes/calendar_routes.py |
| **Notes / Tâches** | | | | | | | |
| Notes style Google Keep | Oui | Panneau Notes | Grille de notes | valeur | `/api/notes/*` | 🟢 | routes/note_routes.py |
| Tâches planifiées | Oui | Panneau Tasks | Liste tâches + statut | valeur/statut | `/api/tasks/*` | 🟢 | routes/task_routes.py |
| **Galerie** | | | | | | | |
| Images (upload/browse) | Oui | Panneau Gallery | Grille d'images | valeur | `/api/gallery/*` | 🟢 | routes/gallery_routes.py |
| Éditeur d'images | Oui | Panneau Editor | Canvas + filtres + outils | action | `/api/editor-drafts/*` | 🟢 | routes/editor_draft_routes.py |
| **Système** | | | | | | | |
| État de santé (health) | Oui | Cockpit chip `#cockpit-health` | ok/down (poll 15 s) | statut | `GET /api/health` | 🟢 ✅ MAJ | cockpit.js pollHealth |
| Readiness (DB, data dir) | Non (API only) | — | — | — | — | 🔴 ANGLE MORT | `/api/ready` (API seulement) |
| Version app | Oui | Settings/About | Numéro version | statut | `/api/version` | 🟢 | app.py:941-943 |
| Diagnostics | Oui | Settings > Diagnostics | Rapports + logs | valeur | `/api/diagnostics/*` | 🟢 | routes/diagnostics_routes.py |
| MCP servers status | Oui | Settings > MCP | Liste serveurs + statut connexion | statut | `/api/mcp/*` | 🟢 | routes/mcp_routes.py |
| API tokens | Oui | Settings > API Tokens | Liste tokens + scopes | valeur | `/api/tokens/*` | 🟢 | routes/api_token_routes.py |
| Webhooks | Oui | Settings > Webhooks | Config webhooks | valeur | `/api/webhooks/*` | 🟢 | routes/webhook_routes.py |
| Presets | Oui | Settings > Presets | Configurations sauvegardées | valeur | `/api/presets/*` | 🟢 | routes/preset_routes.py |
| Backup/restore | Oui | Settings > Backup | Export/import données | action | `/api/backup/*` | 🟢 | routes/backup_routes.py |
| Admin Danger Zone (wipes) | Oui | Settings > System > Danger Zone | Boutons wipe (protégés) | action (admin) | `/api/admin/wipe/*` | 🟢 | routes/admin_wipe_routes.py |
| User preferences | Oui | Settings > Preferences | Thème, police, TTS, etc. | valeur | `/api/prefs/*` | 🟢 | routes/prefs_routes.py |
| **Sécurité** | | | | | | | |
| Auth (login/signup) | Oui | Login page | Formulaire login/signup | action | `/api/auth/*` | 🟢 | routes/auth_routes.py |
| Logout | Oui | Menu utilisateur | Bouton logout | action | `/api/auth/logout` | 🟢 | routes/auth_routes.py |
| TOTP 2FA | Oui | Settings > Security | QR code + setup | action | pyotp (requirements.txt:45) | 🟡 | Interface admin, intégration UI non vérifiée en détail |
| CSP / Security headers | Non | — | — | — | — | 🔴 ANGLE MORT | core/middleware.py (headers HTTP, invisibles dans l'UI) |
| **Orchestration (M3, gated OFF)** | | | | | | | |
| Phase-lock statut | Oui | Cockpit chip `#cockpit-phase` | Phase + `phase_active` (neutre si OFF) | statut | SSE `run_status` | 🟢 ✅ MAJ | cockpit.js ; honnêteté `af7fe87` |
| Live orchestration (CanonicalLoop) | Partiel | Cockpit chip `#cockpit-phase` | Le flag `phase_active` reflète si l'orchestration live/tracker est ON (chip vert) vs OFF (neutre) | statut | SSE `run_status.phase_active` | 🟡 ✅ MAJ | Statut ON/OFF honnête via le chip ; pas de vue dédiée du walk 7-phases |
| Kill-switches dashboard | Oui | Panneau `#settings-killswitches-card` | Table des 35+ switches (état réel + défaut) | statut | `GET /api/killswitches` | 🟢 ✅ MAJ | Sous-projet B : killswitch_routes + admin.js |

---

## C.3 — GAP ANALYSIS : ANGLES MORTS OPÉRATIONNELS

#### ✅ Résolus par le lot « UI Veracity & Visibility » (2026-07-08)

| Capacité | Nouvelle représentation UI | Preuve |
|---|---|---|
| **Drift score** | Cockpit chip + panneau `#settings-drift-card` | observer_routes `922c1ac` |
| **Budget itérations/tokens** | Cockpit chips + panneau `#settings-budgets-card` | governance list-all `44b8fa8` |
| **Phase courante (phase-lock)** | Cockpit chip `#cockpit-phase` (honnête : neutre si OFF) | `af7fe87` |
| **Trinité Connaissance (santé)** | Panneau `#settings-knowledge-card` (CBM/RAG/Obsidian) | Sous-projet C |
| **État de santé** | Cockpit chip `#cockpit-health` (poll 15 s) | cockpit.js |
| **Kill-switches** | Panneau `#settings-killswitches-card` (35+ switches) | Sous-projet B |
| **CodeBurn (one-shot rate)** | Panneau Drift (partiel : taux one-shot) | `922c1ac` |
| **Autoeval keep/revert** / **Verifier** | Badges transitoires in-chat (partiel, bonne UX) | chat.js:2375 ; sse_indicators |

#### 🔴 Encore sans représentation UI (YAGNI — reportés par choix)

| Capacité | Impact opérationnel | Composant backend |
|---|---|---|
| **Graphify** | Graphe de connaissance non exposé | 0 svc |
| **Ancestry (mission→goal→task)** | Pas de vue hiérarchique des objectifs | `record_run_ancestry` — gated OFF |
| **Channel Gateway statut** | Aucune visibilité sur Discord/Telegram | `bootstrap_channels` — gated OFF |
| **Checkpoint Obsidian** | Checkpoints invisibles | `record_checkpoint` — gated OFF |
| **Acontext distillation** | Évolution mémoire invisible | `MemoryProviderRegistry` + Acontext |
| **Teacher escalation** | Escalade enseignant invisible | `agent_loop.py:3714` — interne |
| **Readiness (DB, data dir)** | `/api/ready` non affiché | `/api/ready` — API only |
| **CBM search interactif** | Santé exposée, mais recherche non exposée | `routes/knowledge_routes.py` |

### 🟡 Indicateurs présents mais potentiellement non branchés

| Indicateur | Emplacement UI | Source de données | Statut | Notes |
|---|---|---|---|---|
| Modèle actif | Barre supérieure | `/api/model/*` via `model_discovery` | 🟢 | Confirmé |
| Agent indicators | Chat | SSE `agent_dispatch` events | 🟢 | Ajouté M4 (commit `2b58fd3`) |
| Stream tool calls | Chat | SSE stream | 🟢 | Fonctionnel |
| Gallery images | Gallery | `/api/generated-image/*` | 🟢 | Ownership vérifié |

### 👻 Indicateurs sans source backend prouvée

| Indicateur suspect | Emplacement | Preuve d'absence |
|---|---|---|
| **Aucun détecté** | — | — |

---

## C.4 — SYNTHÈSE COUVERTURE

| Métrique | Avant (2026-07-07) | Après lot UI Veracity (2026-07-08) |
|---|---|---|
| Capacités backend totales (approx.) | ~45 | ~45 |
| Exposées dans l'UI (🟢) | ~24 (53%) | **~31 (69%)** |
| Partiellement exposées (🟡) | ~4 (9%) | **~10 (22%)** |
| Angles morts (🔴) | ~17 (38%) | **~8 (18%)** ⬇ |
| Indicateurs mock/non branchés (👻) | 0 détectés | 0 détectés |
| Kill-switches sans dashboard | 35+ | **0 — dashboard `#settings-killswitches-card`** ✅ |
| Routes API sans UI consommatrice | `/api/knowledge/*`, `/api/phase/*`, `/api/autoeval/*`, `/api/governance/*` | Résolues : `/api/knowledge/*` (Trinité C), `/api/governance/budgets` (panneau Budgets), `/api/observer/drift` (panneau Drift), `/api/killswitches` (dashboard B). Restent sans consommateur dédié : `/api/phase/*`, `/api/channel/*`, `/api/autoeval/*` (badge transitoire seulement). |

**Delta** : les angles morts UI passent de **38% → ~18%**. Les 7 capacités prioritaires (drift, budgets, phase, kill-switches, santé, Trinité, phase-lock) sont désormais couvertes, dans le respect du principe d'honnêteté (chip vert uniquement sur signal réel). Les ~8 angles morts restants (Graphify, Ancestry, Channel Gateway, Checkpoint, Acontext, Teacher, Readiness, recherche CBM interactive) sont **reportés par choix YAGNI**, non par oubli.
