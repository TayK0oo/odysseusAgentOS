# PORTING-SCOPE — Ce qui est porté en Rust, ce qui ne le sera pas

> Décision structurante du milestone v0.7 (2026-10-19). Source : `AUDIT-ODYSSEUS-PORTAGE.md`.

## ✅ Porté en Rust (natif)

| Crate Omega | Origine Python | Port Omega | Tests |
|---|---|---|---|
| `omega-memory` | `services/memory/` (memory, service, skill_format, skills, skill_importer) | `MemoryStore`, `SkillStore` | 12 |
| `omega-search` | `services/search/` (query, ranking) | utilitaires CatalogSource | 9 |
| `omega-policy` | `src/budget_enforcer.py` + `services/security/opa_client.py` | `PermissionPolicy`, `ExecPolicy` | 10 |
| `omega-tools` | `src/agent_tools/filesystem_tools.py` | `ToolSurface`, `ToolExecutor` | 27 |

## 🔀 Bridge JSON-RPC (Python reste maître)

| Domaine | Raison |
|---------|--------|
| `routes/chat_routes.py`, `session_routes.py` | Sessions/conversation stateful — migration progressive via bridge |
| `routes/model_routes.py` | CRUD modèles :\Omega-router a déjà son `CatalogSource` |
| `routes/shell_routes.py` | Exécution shell : `ExecPolicy` + sandbox Rust déjà en place, routes restent Python |
| `integrations/claude`, `integrations/codex` | Sub-agents externes (Phase D v0.2 déjà fonctionnels) |

## ❌ Non porté (documenté, P3)

| Module | Lignes | Raison |
|--------|--------|--------|
| `routes/email_*` (7 fichiers) | 5 930 | Spécifique Gmail/IMAP/polling — hors cœur agent |
| `routes/cookbook_*` (4 fichiers) | 5 411 | Recettes produit / LLM serving — non générique |
| `routes/gallery_routes.py` | 2 043 | Galerie d'images — UI produit |
| `src/agent_tools/admin_tools.py` | 1 012 | Admin UI (spécifique hébergement) |
| `services/stt`, `services/tts` | ~400 | Modèles médias — dépendances lourdes (whisper, TTS) |
| `services/hwfit` | ~800 | Détection hardware + image models — P3 |
| `src/visual_report.py` | 1 934 | Rendu rapport HTML — non critique |
| `routes/calendar_routes.py` + `caldav_*` | ~2 400 | Intégration CalDAV — P3 |

**Total non porté : ~20 000 lignes** (sur 223 253 = 9 %).

## Règle de décision (à l'avenir)

> On porte en Rust ce qui est **cœur agent** : ports, boucle, outils, mémoire, politique.
> On garde en Python ce qui est **spécifique produit** ou **lourd en dépendances**.
> Le bridge JSON-RPC est le canal de migration : pas de fork de code, pas de double maintenance.
