# Dissection — Aider (Aider-AI, Apache-2.0, Python)

> Source : clone `--depth 1` de https://github.com/Aider-AI/aider dans `/home/tayk0oo/omega-re/aider`.
> Données : `omega/crates/omega-harness/data/apps/aider.json` (17 fonctionnalités).

## 1. Fiche

| Champ | Valeur |
|---|---|
| Éditeur | Aider-AI (Paul Gauthier) |
| Licence | Apache-2.0 |
| Langage | Python |
| Repo | https://github.com/Aider-AI/aider |
| Architecture | CLI pair-programming : boucle REPL (`Coder.run`) → repo map compressée en prompt → un *edit-format* impose la grammaire de réponse du LLM → application floue + lint/tests + auto-commit git. Modèles via litellm avec registre de settings par modèle. |

## 2. Entrées

- **CLI** `aider` (`aider/__main__.py`, `aider/main.py`, options dans `aider/args.py`).
- **Watch** `--watch-files` (`aider/watch.py`) : commentaires IA sur changements externes.
- **Browser** `--browser` (`aider/gui.py`, `CaptureIO`) + copier/coller web (`aider/copypaste.py`).
- **Benchmark** `benchmark/benchmark.py` (exercices polyglottes versionnés).
- **Config** : `.aider.conf.yml` (découverte git-root → cwd → home), `.aider.model.settings.yml`, `.aider.model.metadata.json`, `.env`, `.aiderignore`.

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| Repo map tree-sitter + PageRank | contexte | Carte du dépôt classée par PageRank personnalisé sous budget tokens | aider/repomap.py:42-103,279-328,365-529,576-710 | context-engine → plugins/repomap-pagerank | planned | L |
| SEARCH/REPLACE flou (diff) | execution | Blocs SEARCH/REPLACE appliqués avec appariement flou multi-stratégies | aider/coders/editblock_coder.py:15-18, aider/coders/search_replace.py:434-565 | edit-apply → plugins/edit-search-replace | planned | M |
| Fichier entier (whole) | execution | Réécriture intégrale, repli robuste | aider/coders/wholefile_coder.py:10-13 | edit-apply → plugins/edit-wholefile | planned | S |
| Diff unifié + patch V4A | execution | Formats machine natifs (udiff, mini-langage V4A) | aider/coders/udiff_coder.py:46-49, aider/coders/patch_coder.py:13-46,210-217 | edit-apply → plugins/edit-udiff-patch | planned | M |
| Architecte → éditeur | agents | Gros modèle conçoit, modèle éditeur applique | aider/coders/architect_coder.py:6-48 | subagent-runner → plugins/architect-loop | planned | M |
| Modes ask / context | agents | Q/R et repérage de fichiers sans écriture | aider/coders/ask_coder.py:5-9, aider/coders/context_coder.py:5-53 | agent-mode → plugins/readonly-modes | planned | S |
| Réflexion lint/tests | execution | Erreurs renvoyées au modèle (≤3 réflexions) | aider/coders/base_coder.py:924-944,1599-1623 | exec-policy → plugins/reflect-lint-test | planned | M |
| Git auto-commit | automatisation | Commit auto + message du weak model | aider/coders/base_coder.py:2375-2400, aider/repo.py:131-270,326-375 | vcs-commit → plugins/git-auto-commit | planned | S |
| Undo garde-fou | securite | Annule uniquement le dernier commit aider, avec vérifs | aider/commands.py:553-600 | vcs-commit → plugins/git-auto-commit | planned | S |
| Linter multi-langage | execution | Vérif syntaxique immédiate après édition | aider/linter.py:21-82,118-172 | lint-check → plugins/lint-after-edit | planned | S |
| Registre modèles + routage | routage | Capacités par modèle, délégation au weak model | aider/models.py:128-160,161-280,329-370,603-647 | registry → omega-registry | planned | M |
| Prompt caching | contexte | Préfixe gelé + breakpoints Anthropic + compta tokens | aider/coders/chat_chunks.py:6-64, aider/coders/base_coder.py:1335-1341,2004-2009 | context-engine → plugins/prompt-cache | planned | M |
| Commandes slash | ui | ~35 commandes REPL (contexte, git, modèles) | aider/commands.py:799-967,993-1064 | tool-surface → plugins/slash-commands | planned | M |
| Résumé d'historique | memoire | Compaction sous plafond max_input/16 | aider/history.py:7-98 | context-engine → omega-context::truncate_compact | planned | M |
| Watch + commentaires IA | automatisation | Revue auto des changements externes | aider/watch.py:65-181,257-284 | scheduler-job → plugins/file-watcher | planned | M |
| Benchmark + leaderboards | observabilite | Exercices versionnés, classements publics | benchmark/benchmark.py:162-258,679-800 | telemetry-sink → plugins/eval-harness | planned | L |
| Config en couches | configuration | Précédence fichiers + settings par modèle | aider/args.py:22-35,121-129,794-800 | config-source → omega-config | planned | S |

Détail du PageRank (`aider/repomap.py:490-523`) : multiplicateurs mention ×10, snake/camel ≥8 chars ×10, privé ×0.1, >5 définisseurs ×0.1, fichier en chat ×50, racine des refs ; `nx.pagerank(G, weight="weight", personalization=…)` (`:525`).
Détail réflexion (`aider/coders/base_coder.py:101`) : `max_reflections = 3`, retry exponentiel `:1449-1495`.
Détail cache (`aider/coders/chat_chunks.py:28-43`) : `add_cache_control_headers` + `cacheable_messages`.

## 4. Traduction en plugins

- `context-engine` → `ContextEngine` → `plugins/repomap-pagerank` (plugin, L) : tags tree-sitter + PageRank personnalisé + budget.
- `edit-apply` → `EditApplier` → `plugins/edit-{search-replace,wholefile,udiff-patch}` (plugin, S/M) : grammaire de réponse par modèle.
- `subagent-runner` → `SubagentRunner` → `plugins/architect-loop` (plugin, M) : concevoir puis appliquer.
- `exec-policy` → `ExecPolicy` → `plugins/reflect-lint-test` (plugin, M) : fixpoint lint/tests borné.
- `vcs-commit` → `VcsCommit` → `plugins/git-auto-commit` (plugin, S) : commit auto + undo gardé.
- `registry` → `Registre` → `omega-registry` slots modèles (plugin, M) : edit_format/use_repo_map/cache par modèle + weak model.
- `context-engine` → `ContextEngine` → `plugins/prompt-cache` (plugin, M) : préfixe gelé + compta cache.
- `telemetry-sink` → `TelemetrySink` → `plugins/eval-harness` (service, L) : benchmark versionné.
- `config-source` → `ConfigSource` → `omega-config` (plugin, S).

## 5. Écarté

- **Voice input** (`aider/voice.py:33-116`) : dictée terminale, hors philosophie *coût-système* — rejeté (faible valeur / dépendances audio).
- **Attribution git aider** (Co-authored-by, préfixes) : cosmétique, non repris tel quel.
- **GUI navigateur** (`aider/gui.py`) : simple capture d'IO vers web, pas de vrai cockpit — notre cockpit React reste la cible.

## 6. Sources

- https://github.com/Aider-AI/aider (clone `/home/tayk0oo/omega-re/aider`)
- `aider/repomap.py`, `aider/coders/` (base, editblock, wholefile, udiff, patch, architect, ask, context, search_replace, chat_chunks), `aider/repo.py`, `aider/linter.py`, `aider/models.py`, `aider/commands.py`, `aider/history.py`, `aider/watch.py`, `aider/analytics.py`, `aider/args.py`, `benchmark/benchmark.py`
- Leaderboards : https://aider.chat/docs/leaderboards/
