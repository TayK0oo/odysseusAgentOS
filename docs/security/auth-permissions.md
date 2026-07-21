# Security — Authentification et permissions

## Authentification

- **bcrypt** → hashage des mots de passe
- **Sessions** → tokens de 7 jours, stockage atomique `data/sessions.json`
- **2FA** → TOTP avec 8 codes de backup
- **API tokens** → `Bearer ody_<base64>`, préfixe pour lookup rapide
- **Internal tool** → token loopback `X-Odysseus-Internal-Token` (admin inconditionnel)

## Niveaux de risque

| Niveau | Description | Exemples |
|--------|-------------|----------|
| **READ** | Lecture seule | `read_file`, `grep`, `glob`, `ls` |
| **DRAFT** | Brouillon | `suggest_document` |
| **WRITE** | Écriture disque | `write_file`, `edit_file`, `manage_memory` |
| **EXEC** | Exécution code | `bash`, `python`, `web_search` |
| **DESTRUCTIVE** | Suppression | `remove_dir`, `cancel_download` |

## Matrice outils × phases

| Outil | Risque | CLASSIFY | KNOW | PLAN | BUILD | QUALITY | AUTOEVAL |
|-------|--------|----------|------|------|-------|---------|----------|
| `read_file` | READ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `web_search` | EXEC | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `write_file` | WRITE | ❌ | ❌ | ⚠️ | ✅ | ❌ | ✅ |
| `bash` | EXEC | ❌ | ❌ | ❌ | ✅ | ⚠️ | ✅ |
| `remove_dir` | DESTR | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |

## Destructive Gate (patterns bloqués)

```
rm -rf /, rm -rf /*, mkfs.*, dd if=, :(){ :|:& };:, 
chmod 777 /, chmod -R 777, > /dev/sda, fork bomb
```

## Admin Gate

Routes protégées par `require_admin` :
- `/api/admin/*` — wipes
- `/api/auth/users` — gestion utilisateurs
- `/api/model-endpoints` — gestion endpoints
- `/api/tokens` — gestion tokens API
- `/api/mcp/servers` — gestion MCP
- `/api/settings` — settings admin

---

→ Voir aussi : [Modèle de menace](threat-model.md) · [Sandboxing](sandboxing.md) · `permission-matrix.md` · `config/phase-lock.yaml`
