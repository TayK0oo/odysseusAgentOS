# Dissection — Sweep (sweepai, Apache-2.0, Python)

> Source : clone `--depth 1` de https://github.com/sweepai/sweep dans `/home/tayk0oo/omega-re/sweep`.
> Donnees : `omega/crates/omega-harness/data/apps/sweep.json` (9 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | sweepai |
| Langage | Python |
| Repo | https://github.com/sweepai/sweep |
| Architecture | Agent GitHub (Python/FastAPI) : bus de webhooks ticket/PR/comment, SweepBot a contexte vectorielle par LLM-guided ripgrep + embedding, creation PR automatique + review annotations, global threads multi-PR. |

## 2. Inventaire fonctionnel

### `swp-webhook-bus` — Fondation GitHub par webhook : ticket/PR/comment check suite

- **domaine** : automatisation · **capacite** : event-store · **port** : EventStore · **cout** : S · **statut** : adopted
- **what** : Un bus de continuation par webhooks GitHub : on_ticket/on_comment/on_merge/on_check_suite/reserve de PR évène.
- **how** : sweepai/handlers/{on_ticket.py,on_comment.py,on_merge.py,on_check_suite.py,on_failing_github_actions.py}. Omega : EventStore (webhook ingress -> events).
- **adaptateur Omega** : omega-server (webhook ingress events)
- **preuves** : `sweepai/handlers/on_ticket.py` · `sweepai/handlers/on_comment.py`

### `swp-context-pruning` — Context pruning LLM-guidé : snippet manager par ripgrep entities

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : M · **statut** : planned
- **what** : Un contexte trouvé par LLM guidance : explore des entités du code par ripgrep requests, re-prompt au LLM, Snippets au RepoContextManager, expand directories.
- **how** : sweepai/core/context_pruning.py:167-200 (escape_ripgrep, run_ripgrep_command, can_add_snippet) ; class RepoContextManager (context_pruning.py:200) top_snippet_paths/expand_all_directories (217-224) ; lexical_search.py. Omega : ToolSurface (explore) + ContextEngine : le pattern de context pruning LLM-guidé est aussi de l'ordre de collecte apprend.
- **adaptateur Omega** : omega-context (context pruning par ripgrep LLM-guidé)
- **preuves** : `sweepai/core/context_pruning.py:167-224`

### `swp-vector-db` — Vector DB : embeddings snips voyage/openai, batché par token count

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : M · **statut** : planned
- **what** : Une base vectorielle : embeddings par arrays (voyage/openai embeddings), batchées par token count et cosine similarity.
- **how** : sweepai/core/vector_db.py:34-129 (cosine_similarity, chunk, multi_get_query_texts_similarity, batch_by_token_count_for_voyage, embed_text_array, openai_call_embedding_router). Omega : TOOL recherche embedder par CatalogSource.
- **adaptateur Omega** : omega-context (embedding retrieval similarité)
- **preuves** : `sweepai/core/vector_db.py:34-129`

### `swp-sweep-bot` — SweepBot : un agent de PR creation par files_snippets + changelog + new prs

- **domaine** : agents · **capacite** : subagent-runner · **port** : SubagentRunner · **cout** : M · **statut** : planned
- **what** : Un agent structuré : le SweepBot projette des files crees/mods para PR (title/body/gh_harness), et clean code.
- **how** : sweepai/core/sweep_bot.py + prompts.py + create_pr.py (sweepai/handlers/create_pr.py). Omega : LocalSubagentRunner + EventStore couvert.
- **adaptateur Omega** : omega-app (SweepBot generación PR)
- **preuves** : `sweepai/core/sweep_bot.py` · `sweepai/handlers/create_pr.py`

### `swp-review` — PR reviewer : annotations + review_utils prompts

- **domaine** : agents · **capacite** : subagent-runner · **port** : SubagentRunner · **cout** : M · **statut** : planned
- **what** : Un agent de review séparé : re-post annotated PR diffs, propose merge or fail and rewrite requests (review_prompts.py).
- **how** : sweepai/core/review_utils.py + review_prompts.py + review_annotations.py + handlers/review_pr.py. Omega : reviewer de code comme plugin independant (LocalSubagentRunner + verifier).
- **adaptateur Omega** : plugin/code-reviewer (review loop)
- **preuves** : `sweepai/core/review_utils.py`

### `swp-dynamic-context` — Dynamic context bot : les fichiers ont de stalce à un pas du contexte

- **domaine** : contexte · **capacite** : context-engine · **port** : ContextEngine · **cout** : S · **statut** : planned
- **what** : Un bot de context : dynamic_context_bot décide quels fichiers rafraîchir en contexte lors d'une degenerligence (help), et de context تغها (expuls autoritaire snips).
- **how** : sweepai/core/dynamic_context_bot.py + snippet_utils.py. Omega : ContextEngine; le stale detection de context est une adjace de la mémoire.
- **adaptateur Omega** : omega-context (dynamic context bot)
- **preuves** : `sweepai/core/dynamic_context_bot.py`

### `swp-str-bus` — Threads globaux : global_threads.py multi-PR en parallèle

- **domaine** : automatisation · **capacite** : scheduler-job · **port** : — · **cout** : S · **statut** : planned
- **what** : Une orchestration multi-PR : global_threads.py; les PRs sont en assembly par lamentos; útil pour le run par repository.
- **how** : sweepai/global_threads.py. Omega : omega-app::App (fenêtre multi-run), à la pacer.
- **adaptateur Omega** : omega-app (threads multi-run)
- **preuves** : `sweepai/global_threads.py`

### `swp-chat` — sweep_chat : chat interactif sur une PR ouverte

- **domaine** : agents · **capacite** : agent-loop · **port** : — · **cout** : M · **statut** : planned
- **what** : Un mode chat embarque : conversation sur une issue/PR existante, reprise du contexte repo, generation de follow-up PRs.
- **how** : sweepai/chat/ + sweep_chat/. Omega : AgentLoop + SessionStore deja couverts.
- **adaptateur Omega** : omega-app (chat sur PR)
- **preuves** : `sweepai/chat` · `sweepai/core/chat.py`

### `swp-external-searcher` — ExternalSearcher : recherche d'issues similaires sur GitHub avant de traiter

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : S · **statut** : planned
- **what** : Un agent secondaire qui cherche dans les issues/PRs historiques du depot les exemples pertinents pour enrichir le contexte de la PR.
- **how** : sweepai/core/external_searcher.py + pr_reader.py. Omega : CatalogSource (recherche historique du repo) deja note ; le pattern de recherche GitHub avant planification est un enrichisseur de contexte.
- **adaptateur Omega** : omega-context (recherche issic GitHub)
- **preuves** : `sweepai/core/external_searcher.py` · `sweepai/core/pr_reader.py`

