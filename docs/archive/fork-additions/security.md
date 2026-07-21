# Sécurité — AgentOS Odysseus

## Modèle de menace

Voir THREAT_MODEL.md pour l'analyse complète.

## Sandbox layers

### Layer 1 — Phase-lock (src/tool_registry.py)
- Tools retirés selon la phase courante (RESEARCH = read-only)
- Config déclarative : `config/phase-lock.yaml`

### Layer 2 — Command validator (src/command_validator.py)
- Whitelist de commandes sûres
- Blocklist de patterns dangereux (rm -rf /, fork bombs, pipe curl→bash)
- Warnings pour les patterns risqués

### Layer 3 — Docker hardening
- `no-new-privileges: true` sur tous les services agents
- `cap_drop: ALL` préparé (décommenter après validation)
- `/tmp` en tmpfs (100MB)
- Docker socket exposé uniquement pour le service Cookbook

### Layer 4 — AgentSeal (CI)
- `pip install agentseal && agentseal probe` en CI
- 300+ probes : injection, MCP empoisonnés, skills malveillants
- Rapport hebdomadaire + pre-merge

## Surfaces non-fiables

Ces surfaces peuvent contenir des instructions malveillantes. Elles sont taguées [UNTRUSTED] dans les traces :

- Skills utilisateur (`.opencode/skills/`)
- Notes utilisateur
- Documents personnels
- Mémoires auto-générées (Acontext)
- Contenu web scraped (Scrapling)
- Messages inbound (Discord/Telegram)

## Audit prompt-injection

```bash
# Tester une surface pour injection
agentseal probe --surface notes --target http://localhost:7000
agentseal probe --surface skills --target http://localhost:7000
```

## Approval gates

Les actions DESTRUCTIVE (rm -rf, DROP TABLE, git push --force) :
1. Bloquées par command_validator si sur whitelist blocklist
2. Taguées `DESTRUCTIVE` par risk_classifier
3. Loguées dans data/traces/ avec `permission_decision: gate_required`
4. En mode interactive : demande de confirmation humaine
