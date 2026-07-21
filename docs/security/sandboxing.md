# Security — Sandboxing

## Couches de sécurité

```
┌─────────────────────────────────────────┐
│  OPA (Rego policies)                    │  ← Optionnel
│  Autorisation déclarative, testable     │
├─────────────────────────────────────────┤
│  gVisor (runsc)                         │  ← Optionnel
│  Kernel user-space, syscall filtering   │
├─────────────────────────────────────────┤
│  Docker hardening                       │  ← Par défaut
│  cap_drop: ALL, no-new-privileges       │
├─────────────────────────────────────────┤
│  Destructive Gate                       │  ← ON par défaut
│  Bloque commandes shell dangereuses     │
├─────────────────────────────────────────┤
│  Phase-lock                             │  ← OFF par défaut
│  Outils retirés par phase               │
└─────────────────────────────────────────┘
```

## Docker hardening (actif par défaut)

```yaml
# docker-compose.yml
security_opt:
  - no-new-privileges:true
cap_drop:
  - ALL
cap_add:  # Uniquement pour odysseus
  - CHOWN
  - SETUID
  - SETGID
  - DAC_OVERRIDE
```

Services avec `cap_drop: ALL` : chromadb, searxng, ntfy, kroki, serena-mcp, meilisearch, qdrant, opa, postgres

## gVisor (optionnel)

```bash
# Installation
# https://gvisor.dev/docs/user_guide/install/

# Activation
ODYSSEUS_GVISOR=on
ODYSSEUS_GVISOR_RUNTIME=runsc
```

Services sandboxés : serena-mcp, scrapling-mcp, codebase-memory, graphify, decision-engine

**Overhead :** ~5-15% pour les workloads I/O

## OPA (optionnel)

```bash
docker compose --profile security up -d
ODYSSEUS_OPA=on
```

Politiques Rego dans `config/policies/` :
- `tool_access.rego` — autorisation des outils par phase
- `phase_lock.rego` — transitions de phase valides

Test des politiques :
```bash
opa test config/policies/
```

## Command Validator

Intégré dans `builtin_actions.py`, valide les commandes shell avant exécution :
- Bloque les patterns dangereux (`rm -rf`, `mkfs`, `dd`)
- Bloque les fork bombs (`:(){ :|:& };:`)
- Vérifie les chemins (pas d'écriture hors workspace)

---

→ Voir aussi : [Modèle de menace](threat-model.md) · [Auth & Permissions](auth-permissions.md) · `docker-compose.yml` · `config/policies/`
