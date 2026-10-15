# W7-cockpit — Adaptateur de bord `omega-server` + UI branchée (inc2)

> **Workstream :** W7 (Cockpit UI) · **Incrément :** inc2 · **Date :** 2026-10-15
> **Branche :** `feat/omega-w7-server` · **Arbre :** `omega-par/w7-server`
> **Statut :** livré (endpoints + UI + tests + preuve live `[observed]`).
> **Provenance :** `[observed]` (commandes jouées) · `[inferred]` (choix documentés).

---

## 1. Objet

Exposer l'`App` assemblée (registre gelé + boucle + journal durable) derrière
une **API HTTP typée et versionnée** (`omega-server`, catégorie `app-edge` du
blueprint §2.1), et brancher le Cockpit React/TS dessus (`fetch('/api/...')`
avec états chargement/erreur + repli fixtures hors-ligne).

Rappel blueprint §7 : le journal durable alimente le **replay** ; la frontière
Rust↔TS est **versionnée** (`schema_version: 1` partout).

---

## 2. Contrats d'API

Base : `http://127.0.0.1:8787` (port configurable, cf. §6). Tous les corps
JSON portent `schema_version: 1` (`omega-protocol::SCHEMA_VERSION`), y compris
les erreurs (`ApiErrorBody`).

### Table des endpoints

| Méthode | Chemin | Entrée | Sortie (200) | Erreurs |
|---|---|---|---|---|
| `GET` | `/api/health` | — | `{status:"ok", schema_version:1}` | — |
| `GET` | `/api/plugins` | — | `[{schema_version, id, capability, active:true}]` depuis `App::active_ids` par capacité (25 capacités, ordre stable ; priorité intra-capacité du registre) | — |
| `POST` | `/api/run` | `{messages:[{role,content}], run_id?}` (`role ∈ system\|user\|assistant\|tool`, insensible à la casse) | `{run_id, status, decision, summary}` où `status ∈ ok\|failed\|cancelled` (`RunStatus` kebab-case), `decision = RouterDecision{model_id,score,reason}`, `summary = RunSummary` canonique | `400` entrée invalide (vide / rôle inconnu / contenu vide) ; `502` provider injoignable (`OmegaError::Io`, ex. Ollama éteint) ; `500` autre |
| `GET` | `/api/trace/:run_id` | `run_id` en chemin | `{run_id, replayed: ReplayedRun, summary: RunSummary}` relus du `FileEventStore` | `500` I/O. Run inconnu → `200` avec séquence vide + `outcome:"unknown"` (pas de 404 : l'UI affiche « run inconnu ») |

### Pipeline `POST /api/run` (déterministe avant inférence)

1. **Validate** : `ApiRunRequest::to_messages()` (400 si KO — jamais de repli
   silencieux vers `user`).
2. **Classify** : `HeuristicTaskClassifier` sur le texte concaténé (zéro LLM).
3. **Select** : `ParetoSelector<CompositeScoringPolicy>` sur le **seul candidat
   local** `ollama/<modèle>` (`ModelTier::SelfHosted`, coût 0 — garantie de
   coût nul structurelle, pas documentaire).
4. **Run** : `assemble` (ContextEngine du registre) + `infer` (Provider du
   registre) via `App::run` ; `run_id` client conservé, sinon généré (`run-N`).
5. **Prove** : `TraceReplayer::summarize(run_id, journal)` — le résumé est une
   preuve relue du journal, pas une affirmation.

---

## 3. UI branchée (`omega/ui/src/`)

- **`protocol.ts`** : ajoute les miroirs manquants — `RunResponse`,
  `RouterDecision`, `HealthResponse`, `ReplayedRun`, `ReplayedStep`,
  `ReplayedStepStatus`, `ReplayedOutcome`, `RunSummary`, `TraceResponse`,
  `ApiErrorBody` (dette `ts-rs` toujours documentée en tête de fichier).
- **`api.ts`** (nouveau) : `getHealth/getPlugins/postRun/getTrace` sur `/api/...`
  (même origine ; proxy Vite `5173 → 8787` déjà présent dans
  `omega/ui/vite.config.ts`). Toute erreur réseau/HTTP → exception.
- **`App.tsx`** : chaque panneau a ses états **chargement / données / erreur** ;
  les **fixtures restent en repli** si l'API est injoignable (bandeau explicite,
  jamais d'écran vide). `Cockpit` lance un vrai run (formulaire) et remonte la
  décision au `Router Dashboard` (`lastRun`) ; `Trace Viewer` rejoue un `run_id`
  quelconque (défaut : dernier run). `Context Inspector` reste statique :
  endpoint dédié = incrément suivant.

---

## 4. Construction serveur (`omega-server`, lib + binaire)

- `build_app_state()` : `OllamaProvider(localhost:11434)` + `ContextContributor`
  + `FileEventStore` (dossier temporaire) — même `Arc` dans le registre
  (`event-store`) et dans l'état (relecture traces).
- `ServerConfig` : port **8787** par défaut, `OMEGA_PORT` puis `PORT`,
  `OLLAMA_URL`, `OLLAMA_MODEL`, `OMEGA_EVENT_DIR` (repli systématique sur les
  défauts, jamais de panique sur variable malformée).
- `create_router(state)` : chemins absolus `/api/...` (montable tel quel ou
  imbriqué). Handlers = fonctions pures de `(état, requête)` → **testables
  sans réseau** (cf. §7).

---

## 5. Dépendances `app-edge` (garde-fou `check_dep_graph.py`)

`ARCH_RULES["app-edge"]` passe de `{types, app, protocol}` à
`{types, ports, config, protocol, core, registry, plugin-api, adapters, router, app}`.
Justification par arête réellement utilisée :

| Dépendance | Catégorie | Usage dans `omega-server` |
|---|---|---|
| `omega-types` | types | `Message`, `Role`, `ModelId`, `RouterDecision`, `Capability`, erreurs |
| `omega-ports` | ports | `Provider`/`EventStore` (contributeurs + lecture journal) ; ports du routeur (`Selector`…) |
| `omega-config` | config | **réservé** : `ServerConfig` reste aujourd'hui lu de l'env ; le câblage sur les couches (profils/port) est le chemin prévu — allowance sans usage immédiat, assumée |
| `omega-protocol` | protocol | `SCHEMA_VERSION` source unique (tous les DTO portent le schéma) |
| `omega-core` | core | `Operation` (plan du run), `RunStatus` (réponse) |
| `omega-registry` | registry | `Manifest`, `RegistryBuilder` (contributeurs provider/store) |
| `omega-plugin-api` | plugin-api | `Contributor`, `InstallContext` (enregistrement) |
| `omega-context` | adapters | `ContextContributor` (slot `context-engine`) |
| `omega-providers` | adapters | `OllamaProvider` (slot `llm-provider`, self-hosted) |
| `omega-otel` | adapters | `TraceReplayer`, `ReplayedRun`, `RunSummary` (replay + preuve) |
| `omega-router` | router | `HeuristicTaskClassifier`, `ParetoSelector`, `CompositeScoringPolicy` |
| `omega-app` | app | `App`/`AppBuilder`, `RunInput`, `FileEventStore` (assemblage + exécution durable) |

Le sens reste **bord → tout, jamais l'inverse** : aucun crate ne dépend
d'`omega-server` (zéro cycle vérifié, 14 crates). `plugins` n'est **pas**
autorisé (le serveur ne consomme aucun crate plugin directement).

---

## 6. Décisions

| # | Décision | Alternative écartée | Raison |
|---|---|---|---|
| D1 | `axum 0.7` + `tokio` (multithread) | `actix-web`, HTTP maison | Standard de facto, `Router` testable, MSRV compatible |
| D2 | Rôle inconnu → `400` (pas de repli `user`) | Repli silencieux | Traçabilité : une entrée invalide doit se voir, pas se deviner |
| D3 | `502` si provider KO, `500` sinon | Tout en 500 | L'UI distingue « Ollama éteint » (actionnable) d'un bug interne |
| D4 | Trace inconnue → `200` vide (`unknown`) | `404` | L'UI affiche un état, pas une erreur ; le résumé reste typé |
| D5 | Handlers appelés directement dans les tests (pas de client HTTP) | `tower::oneshot` / serveur éphémère | Zéro dépendance de test réseau ; le smoke test live (§8) couvre le HTTP réel |
| D6 | Candidat de sélection unique et local | Registre multi-modèles | Coût nul **garanti par construction** (W10) ; le registre viendra avec W4 |
| D7 | `summary`/`replayed` = types `omega-otel` sérialisés, pas recopiés | DTO locaux dupliqués | Source unique (blueprint leçon n°7) ; `protocol.ts` les mire à la main en attendant `ts-rs` |

---

## 7. Tests (21, sans LLM ni réseau — sauf §8)

| Couche | Tests |
|---|---|
| Parsing requêtes | rôles OK (casse/espaces), rôles rejetés, `to_messages` (vide / rôle KO / contenu vide), désérialisation de la forme UI `{messages, run_id?}` |
| Sérialisation DTO | `HealthResponse`, `PluginEntry`, `RunResponse` (avec vrai `RunSummary`), `TraceResponse`, round-trips `serde_json` |
| Config/état | défauts W7 (port 8787, Ollama, budget), câblage `ollama/default/file` résolu |
| Pipeline (provider bouché) | run OK sans réseau (`steps_completed=2`, `outcome=completed`, `model_id=ollama/…`), rejet des messages vides, candidat local self-hosted gratuit |
| Handlers (appel direct, store fichier en dossier temporaire) | `health`, `plugins` (3 plugins câblés, `active=true`), `run` 200 + décision/résumé, `run` 400, `trace` après run (2 étapes + résumé cohérent), `trace` run inconnu (vide/`unknown`), smoke `create_router` |

Commandes : `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all --check`, `python3 omega/scripts/check_dep_graph.py` (+ `--self-test`),
`npx tsc --noEmit` dans `omega/ui`.

---

## 8. Preuve live `[observed]` (2026-10-15, Ollama local présent)

```bash
cargo run -p omega-server &                                   # écoute http://127.0.0.1:8787
curl localhost:8787/api/health                                # {"status":"ok","schema_version":1}
curl localhost:8787/api/plugins                               # ollama / default / file, active:true
curl -X POST localhost:8787/api/run \
  -H 'content-type: application/json' \
  -d '{"messages":[{"role":"user","content":"bonjour"}],"run_id":"smoke-1"}'
# → status:ok, decision ollama/qwen2.5-coder:7b (task=T1, score 0.9150),
#   summary: 6 événements, 2/2 étapes, outcome completed
curl localhost:8787/api/trace/smoke-1                         # assemble#0 + infer#1 rejoués, seq 2/4
```

Test live reproductible (prérequis : `ollama serve` + `ollama pull qwen2.5-coder:7b`) :
lancer le serveur, jouer les `curl` ci-dessus, vérifier `status:ok` et
`steps_completed:2`. **Sans Ollama**, `POST /api/run` répond `502` (comportement
voulu, D3) et tout le reste fonctionne.

---

## 9. Suivis (hors inc2, non démarrés)

- Servir le build UI (`dist/`) depuis `omega-server` (fichiers statiques) ou
  documenter le déploiement séparé ( CORS / base URL configurable côté UI).
- Endpoint `GET /api/context/:run_id` pour le Context Inspector live
  (`AssembleReport`).
- Génération TS depuis `protocol` (`ts-rs`, `omega-ui-bindings`) — fin de la
  dette de miroir manuel.
- Authentification du bord (écoute `127.0.0.1` seule protection actuelle) et
  endpoints d'administration (quarantaine, reload) — cf. W9.
