# C — COUVERTURE UI & INDICATEURS VISUELS (Axe C)

**Date** : 2026-07-07 | **Commit** : `2821daa`

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
| Phase courante | Non | — | — | — | — | 🔴 ANGLE MORT | PhaseTracker gated OFF |
| Budget itérations (restant) | Non | — | — | — | — | 🔴 ANGLE MORT | agent_loop.py:2572 (hard-stop interne, pas exposé) |
| **Mémoire** | | | | | | | |
| Recherche mémoire vectorielle | Oui | Panneau Memory | Résultats de recherche | valeur | `/api/memory/*` | 🟢 | routes/memory_routes.py |
| Skills extraits | Oui | Panneau Memory > Skills | Liste skills + pertinence | valeur | `/api/skills/*` | 🟢 | routes/skills_routes.py |
| Distillation Acontext (auto-evolve) | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
| **RAG / Documents** | | | | | | | |
| Documents personnels (RAG) | Oui | Panneau Personal/Library | Liste documents + recherche | valeur | `/api/personal/*` | 🟢 | routes/personal_routes.py |
| Résultats recherche RAG | Oui | Chat (inline context) | Contexte injecté dans prompt | valeur | `rag_vector.search` | 🟢 | agent_loop.py (mémoire injectée) |
| **Qualité** | | | | | | | |
| Drift score (LOW/MED/HIGH) | Non | — | — | — | — | 🔴 ANGLE MORT | agent_loop.py:3654 (log-only) |
| Autoeval keep/revert | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF + log-only |
| Verifier subagent | Non | — | — | — | — | 🔴 ANGLE MORT | agent_loop.py:1801 (capé, OFF) |
| CodeBurn (one-shot rate, waste) | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
| Teacher escalation | Non | — | — | — | — | 🔴 ANGLE MORT | agent_loop.py:3714 (interne, non exposé) |
| **Gouvernance** | | | | | | | |
| Ancestry (mission→goal→task) | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF + pas de vue UI |
| Budget token/coût par projet | Non | — | — | — | — | 🔴 ANGLE MORT | PROJECT.yaml.example seulement |
| Destructive gate (blocage) | Partiel | Chat (message d'erreur) | Message "commande bloquée" | alerte | tool_execution.py:565-571 | 🟡 exposé comme erreur, pas comme statut gate |
| **Canaux** | | | | | | | |
| Discord/Telegram (statut) | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
| Channel messages inbound | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
| **Connaissance (Trinité)** | | | | | | | |
| CBM search | Non | — | — | — | — | 🔴 ANGLE MORT | routes/knowledge_routes.py — 0 appelant frontend |
| Graphify | Non | — | — | — | — | 🔴 ANGLE MORT | 0 svc |
| Obsidian notes | Non | — | — | — | — | 🔴 ANGLE MORT | Gated OFF |
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
| État de santé (health) | Non (API only) | — | — | — | — | 🔴 ANGLE MORT | `/api/health` (API, pas d'affichage UI permanent) |
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
| Phase-lock statut | Non | — | — | — | — | 🔴 ANGLE MORT | PhaseTracker gated OFF |
| Live orchestration (CanonicalLoop) | Non | — | — | — | — | 🔴 ANGLE MORT | `ODYSSEUS_LIVE_ORCHESTRATION`=off |
| Kill-switches dashboard | Non | — | — | — | — | 🔴 ANGLE MORT | 35+ switches, aucun affichage centralisé |

---

## C.3 — GAP ANALYSIS : ANGLES MORTS OPÉRATIONNELS

### 🔴 Capacités backend sans AUCUNE représentation UI

| Capacité | Impact opérationnel | Composant backend |
|---|---|---|
| **Drift score** | L'utilisateur ne sait jamais si l'agent dérive | `Observer.compute_drift_score()` — log-only |
| **Budget itérations/tokens** | Pas de visibilité sur la consommation | `budget_enforcer` — hard-stop interne |
| **Phase courante (phase-lock)** | L'utilisateur ne sait pas dans quelle phase il est | `PhaseTracker` — phase jamais exposée |
| **Autoeval keep/revert** | Décisions de revert invisibles | `apply_autoeval` — gated OFF |
| **CodeBurn (qualité code)** | one-shot rate, waste patterns invisibles | `run_codeburn` — gated OFF |
| **Ancestry (mission→goal→task)** | Pas de vue hiérarchique des objectifs | `record_run_ancestry` — gated OFF |
| **Trinité Connaissance** | CBM/Graphify/Obsidian inaccessibles depuis l'UI | `routes/knowledge_routes.py` — 0 appelant |
| **Channel Gateway statut** | Aucune visibilité sur Discord/Telegram | `bootstrap_channels` — gated OFF |
| **État de santé** | Health/readiness non affichés | `/api/health`, `/api/ready` |
| **Kill-switches** | 35+ switches, aucun dashboard | Variables d'env uniquement |
| **Live orchestration** | CanonicalLoop invisible | Gated OFF |
| **Checkpoint Obsidian** | Checkpoints invisibles | `record_checkpoint` — gated OFF |
| **Acontext distillation** | Évolution mémoire invisible | `MemoryProviderRegistry` + Acontext |
| **Teacher escalation** | Escalade enseignant invisible | `agent_loop.py:3714` — interne |
| **Verifier subagent** | Vérification qualité invisible | `_run_verifier_subagent` — capé/OFF |

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

| Métrique | Valeur |
|---|---|
| Capacités backend totales (approx.) | ~45 |
| Exposées dans l'UI (🟢) | ~24 (53%) |
| Partiellement exposées (🟡) | ~4 (9%) |
| Angles morts (🔴) | ~17 (38%) |
| Indicateurs mock/non branchés (👻) | 0 détectés |
| Kill-switches sans dashboard | 35+ |
| Routes API sans UI consommatrice | `/api/knowledge/*`, `/api/phase/*`, `/api/autoeval/*`, `/api/governance/*` |
