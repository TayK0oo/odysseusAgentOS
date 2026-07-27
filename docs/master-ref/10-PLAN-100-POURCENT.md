# Agent OS — Plan Final pour 100%

> **Score actuel :** 83% | **Cible :** 100%

---

## Reste à faire (par priorité)

### 1. Goal-Ancestry Live (15 min) → +5%
```python
# Dans src/opencode_engine.py, après la boucle des phases :
def _create_goal_ancestry(self, session_id, message, phases_completed):
    from core.database import SessionLocal, Goal
    db = SessionLocal()
    try:
        goal = Goal(
            name=message[:200],
            description=f"Agent OS project — {phases_completed} phases",
            created_at=datetime.now(timezone.utc)
        )
        db.add(goal)
        db.commit()
        self.bus.emit("governance", "goal_created", {"id": goal.id})
    finally:
        db.close()
```
**Fichier :** `src/opencode_engine.py:231`

### 2. Heartbeat Scheduling (1h) → +5%
```python
# Plugin @agentos/sfd-heartbeat
# Hook session.idle → planifier prochaine exécution
# Utilise APScheduler (déjà installé)
```
**Fichier :** `packages/sfd-heartbeat/index.ts`

### 3. npm Packages Publish (10 min) → +2%
```bash
cd packages/sfd-eventbus && npm publish --access public
# répéter pour les 8 autres packages
```

### 4. VPS Deployment (1h) → +5%
```bash
docker compose --profile production up -d
# Traefik + Let's Encrypt configurés automatiquement
```

---

## Score projeté après implémentation

```
AXE 1 — Orchestration     90% → 95%  (heartbeat)
AXE 2 — Modularité        90% → 95%  (npm published)
AXE 3 — Routing           85% → 90%  (per-agent model live)
AXE 4 — Trinité           90% → 95%  (dashboard UI)
AXE 5 — Sandbox           80% → 85%  (gVisor)
AXE 6 — Gouvernance       65% → 90%  (goal-ancestry + heartbeat)
AXE 7 — Apprentissage     70% → 80%  (auto skills)
AXE 8 — UI Cockpit        95% → 100% (budget live)

GLOBAL : 83% → 93%+
```

---

## Pour atteindre 100%

Les 7% restants nécessitent :
- Dashboard Trinité unifié (UI custom)
- Auto-skills generation (acontext integration)
- Per-agent model routing live (OpenCode native)
- gVisor runtime activation
- VPS deployment testé
