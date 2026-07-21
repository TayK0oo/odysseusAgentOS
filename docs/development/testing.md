# Development — Tests

## Configuration

```toml
# pyproject.toml
[tool.pytest.ini_options]
testpaths = ["tests"]
asyncio_mode = "auto"
```

## Exécution

```bash
# Tous les tests
python -m pytest tests/ -x -q

# Par marqueur
python -m pytest tests/ -m area_security
python -m pytest tests/ -m area_routes
python -m pytest tests/ -m area_services

# Sans les tests lents
python -m pytest tests/ -m "not slow"

# Avec timeout
python -m pytest tests/ --timeout=30
```

## Marqueurs de taxonomie

| Marker | Description |
|--------|-------------|
| `area_security` | Auth, owner-scope, SSRF, XSS |
| `area_routes` | HTTP route / API behavior |
| `area_services` | LLM, cookbook, email, calendar |
| `area_cli` | CLI / script behavior |
| `area_js` | JavaScript / Node-backed |
| `area_helpers` | Test helpers self-tests |
| `area_unit` | Pure parser / utility |
| `slow` | Tests connus lents (exclus par défaut) |

## Validation avant PR

```bash
# 1. Syntaxe Python
python -m compileall src/ routes/ core/

# 2. Tests
python -m pytest tests/ -x -q

# 3. Import shim
python -c "from src.tool_implementations import do_search_chats; print('OK')"

# 4. Boot smoke test
timeout 5 python app.py 2>&1 | head -5 || true

# 5. JS syntax
node --check static/js/<file>.js

# 6. Docker
docker compose config
```

## Tests de qualité (optionnels)

```bash
# Promptfoo (prompt regression)
npx promptfoo eval

# DeepEval (LLM quality metrics)
python -m pytest tests/quality/test_llm_quality.py -v

# Ragas (RAG evaluation)
python -m pytest tests/quality/test_rag_quality.py -v
```

## État actuel

- **Total :** 662 fichiers, 4566 items collectés
- **Pass :** 4393 (96.7%)
- **Fail :** 149 (95% environnementaux : `rg`/`npx` absents, symlinks Windows, GPU)

## CI/CD

9 workflows GitHub Actions :
- `ci.yml` — tests principaux
- `docker-publish.yml` — build et push images
- `container-scan.yml` — scan de sécurité conteneur
- `secret-scan.yml` — détection de secrets
- `prompt-eval.yml` — Promptfoo (si configuré)
- + workflows de sécurité

---

→ Voir aussi : [Conventions](guidelines.md) · [Debug](debugging.md) · `tests/README.md`
