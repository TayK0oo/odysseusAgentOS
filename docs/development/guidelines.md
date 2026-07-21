# Development — Conventions

## Style de code

- **Python :** suivre les conventions du projet existant
- **JavaScript :** vanilla ES modules, pas de framework
- **CSS :** Tailwind CSS v4 (source: `static/tailwind.css`, build: `npm run css:build`)
- **Pas d'emojis Unicode** dans le code ou l'UI — utiliser des SVG inline
- **Police :** Fira Code (monospace) par défaut
- **Dark theme** par défaut

## Structure du projet

```
src/           → Logique métier (161 modules)
routes/        → Handlers HTTP (55 modules)
core/          → Auth, DB, middleware (9 modules)
services/      → Services backend (18 domaines)
mcp_servers/   → Serveurs MCP (6)
static/        → Frontend SPA (162 JS, Tailwind CSS)
.opencode/agents/ → Agents spécialisés (12)
config/        → YAML, Rego, JSON
tests/         → 662 fichiers de test
```

## Git

- **Branches :** `dev` (défaut), `main` (stable)
- **PRs :** toujours vers `dev`
- **Commits :** Conventional Commits — `type(scope): message`
  - Types : `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `perf`
- **Une PR = un bug ou une fonctionnalité**
- Pas de reformatage massif mélangé avec des changements fonctionnels

## Pull Requests

1. Créer une branche depuis `dev`
2. Faire les changements + tests
3. Vérifier : `python -m pytest tests/ -x -q`
4. Vérifier : `python -m compileall src/ routes/ core/`
5. Committer avec Conventional Commits
6. Créer la PR vers `dev`

## Kill-switches

Chaque nouvelle fonctionnalité d'orchestration ou service externe doit être :
- Gated derrière un kill-switch `ODYSSEUS_*` dans `.env.example`
- **OFF par défaut** (comportement byte-identical)
- Documenté dans `docker-compose.yml`

---

→ Voir aussi : [Tests](testing.md) · [Debug](debugging.md) · [INDEX-MAITRE](../../.planning/INDEX-MAITRE.md) · `CONTRIBUTING.md`
