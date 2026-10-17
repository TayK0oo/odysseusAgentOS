# Dissection — Tabby

## 1. Fiche

- **Éditeur :** TabbyML
- **Licence :** Apache-2.0 pour le contenu hors `ee/` ; `ee/` sous licence Enterprise Tabby (voir `ee/LICENSE`). GitHub renvoie `NOASSERTION`.
- **Langage :** Rust / TypeScript (clients IDE en TypeScript)
- **Repo :** https://github.com/TabbyML/tabby
- **Version observée :** 0.33.0-dev.0 (`Cargo.toml` workspace.package)
- **Architecture :** serveur Rust (axum) embarquant llama.cpp pour l'inférence locale, exposant des API OpenAI-compatibles (`/v1/completions`, `/v1/chat/completions`) et indexant le code via Tantivy (embeddings + BM25 + RRF). Une couche enterprise (`ee/`) ajoute un webserver GraphQL, l'authentification, la gestion des dépôts, l'answer engine et des jobs d'arrière-plan. Des clients IDE (VSCode/Vim/IntelliJ/Eclipse) consomment l'API.

## 2. Entrées

- Binaires CLI : `tabby serve`, `tabby download` (`crates/tabby/src/main.rs`).
- API HTTP : `/v1/completions`, `/v1/chat/completions`, `/v1/events`, `/v1/health`, `/v1beta/models`, `/v1beta/server_setting` (`crates/tabby/src/serve.rs::api_router`).
- Webserver enterprise (feature `ee`) : attachement à l'API/UI, GraphQL, auth, jobs (`ee/tabby-webserver`).
- Extensions IDE : `clients/vscode`, `clients/vim`, `clients/intellij`, `clients/eclipse`, `clients/tabby-agent`.
- Configuration : fichier TOML `~/.tabby/config.toml` (`tabby-common/src/config.rs::Config::load`), variables d'environnement (`TABBY_EMBEDDING_ENABLED`, `TABBY_DISABLE_USAGE_COLLECTION`, etc.).
- Modèles : registre TabbyML (`crates/tabby-common/src/registry.rs`), téléchargement via `tabby-download`.

## 3. Inventaire fonctionnel

| Fonctionnalité | Domaine | Ce que ça fait | Où (file:line) | Cible Omega | Statut | Coût |
|---|---|---|---|---|---|---|
| tb-inference-server | execution | Serveur Rust embarquant llama.cpp (CPU/GPU) | crates/tabby/src/serve.rs:117-232 | llm-provider/Provider | planned | L |
| tb-code-completion-rag | contexte | Completion inline RAG (embeddings+BM25+RRF) | crates/tabby/src/services/completion.rs:150-260 | context-engine/ContextEngine | planned | M |
| tb-openai-chat | execution | Chat completions OpenAI SSE | crates/tabby/src/routes/chat.rs:46-86 | llm-provider/Provider | planned | S |
| tb-embeddings | contexte | Service d'embeddings local | crates/tabby/src/serve.rs:128-135 | — | planned | M |
| tb-code-index-tantivy | contexte | Indexation code Tantivy + chunks | crates/tabby-index/src/indexer.rs:1-120 | context-engine/ContextEngine | planned | L |
| tb-repo-management | configuration | Gestion depots git + GitHub/GitLab | crates/tabby-common/src/config.rs (RepositoryConfig) | config-source/ConfigSource | planned | M |
| tb-structured-doc-search | contexte | Recherche docs structurees | crates/tabby-common/src/api/structured_doc.rs:1-120 | context-engine/ContextEngine | planned | M |
| tb-web-crawler | automatisation | Crawl web + llms.txt | crates/tabby-crawler/src/lib.rs:1-120 | — | planned | M |
| tb-ingestion-api | contexte | Ingestion documents externes | crates/tabby-common/src/api/ingestion.rs:1-60 | context-engine/ContextEngine | planned | M |
| tb-answer-engine | agents | Answer Engine (threads/pages) | ee/tabby-webserver/src/webserver.rs:95-170 | agent-store/AgentStore | planned | L |
| tb-auth | securite | Auth JWT/OAuth/LDAP/tokens | ee/tabby-webserver/src/jwt.rs:1-70 | permission-policy/PermissionPolicy | planned | M |
| tb-model-registry | routage | Registre modeles + download | crates/tabby-common/src/registry.rs:40-180 | llm-provider/Provider | planned | M |
| tb-telemetry-usage | observabilite | Telemetrie usage + evenements | crates/tabby-common/src/usage.rs:1-120 | telemetry-sink/TelemetrySink | planned | S |
| tb-ide-extensions | extension | Extensions IDE | clients/vscode/src/extension.ts | — | planned | L |
| tb-background-jobs | automatisation | Jobs d'arriere-plan | ee/tabby-webserver/src/service/background_job/mod.rs:1-120 | — | planned | M |
| tb-admin-ui | ui | UI admin + GraphQL | ee/tabby-webserver/src/routes/ui.rs | — | planned | L |

## 4. Traduction en plugins

Pour chaque fonctionnalité retenue, triplet `capacité → port → adaptateur` :

- `tb-code-completion-rag` : `context-engine → ContextEngine → omega-context::DefaultContextEngine` (plugin)
- `tb-code-index-tantivy` : `context-engine → ContextEngine → omega-context::DefaultContextEngine` (plugin)
- `tb-structured-doc-search` : `context-engine → ContextEngine → omega-context::DefaultContextEngine` (plugin)
- `tb-ingestion-api` : `context-engine → ContextEngine → omega-context::DefaultContextEngine` (plugin)
- `tb-repo-management` : `config-source → ConfigSource → omega-config` (plugin)
- `tb-telemetry-usage` : `telemetry-sink → TelemetrySink → omega-otel::{Stdout,File}Sink` (plugin)
- `tb-model-registry` : `llm-provider → Provider → (adaptateur tabby planned)` (plugin)
- `tb-auth` : `permission-policy → PermissionPolicy → omega-sandbox::ApprovalQueue` (service)
- `tb-answer-engine` : `agent-store → AgentStore → (adaptateur planned)` (plugin)

Les autres (inference server, embeddings, crawler, IDE extensions, background jobs, admin UI) restent des services externes derrière des ports ou sont rejetés selon le périmètre Rust.

## 5. Écarté

- Runtime JS des clients IDE : hors périmètre Rust (clients TypeScript), à consommer comme clients externes.
- Licence enterprise (`ee/`) : code sous licence Enterprise Tabby, non réutilisable tel quel ; seules les interfaces/contrats sont repris.
- Code Mode sandbox JS (non présent chez Tabby) : non applicable.
- Télémétrie vers TabbyML cloud : remplacée par nos sinks locaux (stdout/fichier) pour la traçabilité.

## 6. Sources

- Clone : `/home/tayk0oo/omega-re/tabby` (lecture seule)
- Preuve REA : `docs/omega/research/rea/evidence/intake-tabby.summary.json` (nature js, 8144 noeuds, 10307 arêtes, 650 limitations)
- LICENSE : `LICENSE` (Apache 2.0 hors `ee/`), `ee/LICENSE` (Enterprise)
- Docs officielles : https://tabby.tabbyml.com/docs/
- Code : chemins cités ci-dessus.