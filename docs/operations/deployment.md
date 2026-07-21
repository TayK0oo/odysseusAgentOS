# Operations — Déploiement

## Docker Compose (recommandé)

```bash
# Déploiement standard
docker compose up -d

# Avec profils supplémentaires
docker compose --profile knowledge --profile observability up -d

# Avec GPU
docker compose -f docker-compose.yml -f docker-compose.gpu-nvidia.yml up -d
```

## Traefik (HTTPS + reverse proxy)

```bash
# Activer le profil gateway
docker compose --profile gateway up -d

# Configuration .env
DOMAIN=votre-domaine.com
ACME_EMAIL=admin@votre-domaine.com
ODYSSEUS_TRAEFIK=on
```

Traefik fournit automatiquement :
- Certificats SSL via Let's Encrypt
- Redirection HTTP → HTTPS
- Rate limiting (100 req/s moyenne, 50 burst)
- Dashboard interne sur `:8080`

## Production

```bash
# PostgreSQL au lieu de SQLite
docker compose --profile production up -d
DATABASE_URL=postgresql+asyncpg://odysseus:password@postgres:5432/odysseus

# Migrer les données
python scripts/migrate_sqlite_to_pg.py
```

## Variables d'environnement critiques

```bash
# Sécurité — TOUJOURS activer en production
AUTH_ENABLED=true
LOCALHOST_BYPASS=false
SECURE_COOKIES=true

# Admin
ODYSSEUS_ADMIN_USER=admin
ODYSSEUS_ADMIN_PASSWORD=<mot-de-passe-fort>
```

## gVisor (sandbox kernel)

```bash
# Installer runsc: https://gvisor.dev/docs/user_guide/install/
# Puis activer:
ODYSSEUS_GVISOR=on
ODYSSEUS_GVISOR_RUNTIME=runsc
```

## Service systemd

```bash
sudo cp odysseus-ui.service /etc/systemd/system/
sudo systemctl enable --now odysseus-ui
```

## Mise à jour

```bash
git pull origin dev
docker compose up -d --build
```

---

→ Voir aussi : [Installation](../setup/installation.md) · [Sauvegarde](backup-restore.md) · [Sécurité](../security/threat-model.md)
