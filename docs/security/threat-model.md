# Security — Modèle de menace (synthèse)

> Document complet : `THREAT_MODEL.md`

## Trust Boundary

Odysseus est conçu pour des **utilisateurs de confiance sur un réseau privé**. Un admin peut exécuter des commandes shell, lire/écrire des fichiers, envoyer des emails.

## Ce qui est protégé

- **Accès non authentifié** — sessions bcrypt + TOTP 2FA
- **Non-admins → admin** — `require_admin` sur les routes sensibles
- **Prompt injection** — `UNTRUSTED_CONTEXT_POLICY` sur contenu web/email/mémoire
- **Services internes** — ChromaDB, SearXNG, Ollama non exposés

## Rôles

| Capacité | Admin | Non-admin |
|----------|-------|-----------|
| Chat agent | ✅ | ✅ |
| Shell / Python | ✅ | ❌ |
| File read/write | ✅ | ❌ |
| Email send/read | ✅ | ❌ |
| Model serving | ✅ | ❌ |
| Settings | ✅ | ❌ |
| Vault | ✅ | ❌ |

## 4 Gates de sécurité

| Gate | Kill-switch | Défaut |
|------|------------|--------|
| **Destructive** — bloque `rm -rf`, `mkfs`, fork bombs | `ODYSSEUS_DESTRUCTIVE_GATE` | **ON** |
| **Phase-lock** — bloque outils hors phase | `ODYSSEUS_PHASE_TRACKER` | OFF |
| **Admin** — bloque outils admin non-admins | — | ON |
| **Tool policy** — bloque selon politique guide-only | — | ON |

## Sandboxing

| Couche | Technologie | Activation |
|--------|------------|-----------|
| Docker hardening | `cap_drop: ALL`, `no-new-privileges` | Par défaut |
| gVisor (kernel) | `runsc` runtime | `ODYSSEUS_GVISOR=on` |
| OPA (policies) | Rego policies | `ODYSSEUS_OPA=on` |

## Gaps connus

1. **Pas de sandbox filesystem** — l'agent bash/read/write tourne comme l'utilisateur du process
2. **SSRF via `/api/v1/chat` `base_url`** — PR #1039 en cours
3. **Token scopes grossiers** — pas de granularité par capacité

## Security Headers

- `X-Frame-Options: DENY` + `frame-ancestors 'none'`
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: no-referrer`
- CSP nonce-based : `script-src 'self' 'nonce-{nonce}'`

---

→ Voir aussi : [Auth & Permissions](auth-permissions.md) · [Sandboxing](sandboxing.md) · `SECURITY.md` · `THREAT_MODEL.md`
