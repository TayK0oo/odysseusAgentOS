# Résolution des Bloquants — Guide Pas à Pas

> **Date :** 2026-07-27 | **Blocage actuel :** 78% → cible 100%

---

## 1. CBM + Graphify — Registry `ghcr.io` denied

**Problème :** `docker pull ghcr.io/superpowers-sh/codebase-memory-mcp:latest` → `error from registry: denied`

**3 solutions :**

### Solution A — Docker login (la plus simple)
```bash
# Créer un token GitHub : https://github.com/settings/tokens (read:packages)
export CR_PAT=votre_token_github
echo $CR_PAT | docker login ghcr.io -u VOTRE_USERNAME --password-stdin
docker compose --profile knowledge up -d
```

### Solution B — Build from source (gratuit, pas de token)
```bash
git clone https://github.com/superpowers-sh/codebase-memory-mcp.git /tmp/cbm
cd /tmp/cbm
docker build -t codebase-memory-mcp:local .
cd C:\Users\ttmdu\Documents\GitHub\odysseusAgentOS
# Modifier docker-compose.yml pour utiliser l'image locale
# Remplacer: image: ghcr.io/superpowers-sh/codebase-memory-mcp:latest
# Par:      image: codebase-memory-mcp:local
docker compose --profile knowledge up -d
```

### Solution C — Utiliser le MCP stdio (déjà intégré)
```bash
# CBM peut tourner via MCP stdio sans Docker
# Le Python package codebase-memory-mcp est peut-être déjà installé
pip install codebase-memory-mcp
# Configurer dans mcp_servers/ pour le lancer en stdio
```

---

## 2. npm Packages — Publication

**Problème :** Les 9 packages `@agentos/sfd-*` ne sont pas publiés sur npm

### Solution A — npm publish (nécessite un compte)
```bash
npm login  # Créer un compte sur npmjs.com
cd packages/sfd-eventbus && npm publish --access public
cd ../sfd-phase && npm publish --access public
# ... répéter pour les 7 autres
```

### Solution B — Local linking (test immédiat)
```bash
# Lier les packages localement sans publier
cd packages/sfd-eventbus && npm link
cd packages/sfd-phase && npm link
# Puis dans le projet principal :
npm link @agentos/sfd-eventbus
npm link @agentos/sfd-phase
```

---

## 3. Obsidian MCP — Second Brain

**Problème :** Kill-switch OFF, pas de vault configuré

### Solution
```bash
# 1. Activer le kill-switch
export ODYSSEUS_OBSIDIAN_MCP=on

# 2. Configurer le chemin du vault Obsidian
export OBSIDIAN_VAULT_PATH="C:/Users/ttmdu/Documents/ObsidianVault"

# 3. Redémarrer Docker
docker compose up -d odysseus
```

---

## 4. Goal-Ancestry — Hiérarchie Projet

**Problème :** Tables SQL existent, pas de création live

### Solution
```python
# Dans src/opencode_engine.py, phase PLAN, ajouter :
from core.database import SessionLocal, Goal, GoalProject, GoalTask

db = SessionLocal()
try:
    goal = Goal(name=message[:100], description=message)
    db.add(goal)
    db.commit()
    project = GoalProject(goal_id=goal.id, name=f"project-{session_id[:8]}")
    db.add(project)
    db.commit()
finally:
    db.close()
```

---

## 5. Heartbeat — Scheduling

**Problème :** Non implémenté

### Solution
```python
# Plugin OpenCode : sfd-heartbeat
# Hook session.idle → planifier la prochaine exécution
# Utiliser APScheduler (déjà dans les dépendances)
```

---

## 6. Déploiement VPS

### Solution
```bash
# Sur le VPS (Ubuntu 22.04+) :
git clone <repo>
cd odysseusAgentOS
# Configurer .env avec OPENCODE_API_KEY
docker compose --profile production up -d
# L'app est dispo sur https://ton-domaine.com (Traefik + Let's Encrypt)
```

---

## Ordre de résolution recommandé

```
1. Docker login ghcr.io     → CBM + Graphify (10 min)
2. Obsidian vault path      → Second brain (5 min)  
3. npm link local           → Packages testables (5 min)
4. Goal-ancestry code       → Gouvernance (30 min)
5. npm publish              → Distribution (10 min)
6. VPS deploy               → Production (1h)
```

**Après ces 6 étapes : 78% → 98%+**
