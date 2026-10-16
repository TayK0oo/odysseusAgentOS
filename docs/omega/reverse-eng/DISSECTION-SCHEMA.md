# Schéma de dissection par application (reverse-engineering exhaustif)

> **Objet :** décortiquer *chaque* application (fonctionnalité par fonctionnalité) et
> l'intégrer **nativement** dans Omega par la philosophie *everything is a plugin*.
> **Sortie par app :** `omega/crates/omega-harness/data/apps/<id>.json` (données) +
> `docs/omega/reverse-eng/dissections/<id>.md` (récit).
> **Validé par :** test `embedded_dissections_are_valid_and_complete` (omega-harness).

## Fichier JSON (une app)

```json
{
  "schema_version": 1,
  "id": "hermes",
  "name": "Hermes Agent",
  "editor": "Nous Research",
  "oss": true,
  "language": "Python",
  "license": "MIT",
  "repo": "https://github.com/NousResearch/hermes-agent",
  "architecture": "… 1-2 phrases …",
  "entrypoints": ["hermes CLI", "gateway messagerie"],
  "functionalities": [ /* >= 8 */ ]
}
```

## Une fonctionnalité

```json
{
  "id": "he-tool-search-bm25",
  "domain": "outils",
  "name": "Tool Search BM25 + gate rarest-token",
  "what": "ce que ça fait (une phrase)",
  "how": "comment c'est implémenté dans l'app d'origine (observé, précis)",
  "omega_capability": "tool-surface",
  "omega_port": "ToolSurface",
  "omega_adapter": "omega-context::ToolSurfaceIndex",
  "integration": "plugin",
  "status": "adopted",
  "rust_cost": "M",
  "refs": ["RE-OPENCLAW-HERMES.md", "tools/tool_search_catalog.py:118-208"]
}
```

### Contraintes (vérifiées par test)

- `id` unique par app ; `domain` non vide ; `what`, `how`, `refs` non vides.
- `integration` ∈ `plugin | service | native | rejected`.
- `status` ∈ `adopted | planned | rejected`.
- `rust_cost` ∈ `S | M | L` (MAJUSCULES — piège : le serde est `UPPERCASE`).
- `status = adopted` ⇒ `omega_adapter` renseigné.
- `integration = plugin` ⇒ `omega_capability` **et** `omega_port` renseignés.
- Au moins 8 fonctionnalités par app.

### Domaines suggérés

`contexte`, `memoire`, `outils`, `agents`, `execution`, `automatisation`,
`extension`, `configuration`, `observabilite`, `routage`, `securite`, `ui`.

### Règle de statut (honnêteté obligatoire)

`adopted` **uniquement** si la fonctionnalité existe réellement dans notre code :

| Capacité | Port | Adaptateur existant |
|---|---|---|
| `llm-provider` | `Provider` | `omega-providers::OllamaProvider` |
| `context-engine` | `ContextEngine` | `omega-context::DefaultContextEngine` |
| `tool-surface` | `ToolSurface` | `omega-context::ToolSurfaceIndex` |
| `skill-store` | `SkillStore` | `omega-context::SkillIndex` |
| `event-store` | `EventStore` | `omega-app::{File,InMemory}EventStore` |
| `config-source` | `ConfigSource` | `omega-config` (couches + precedence) |
| `registry` (slots) | `Registre` | `omega-registry` (Slot/Ensemble + quarantaine) |
| `telemetry-sink` | `TelemetrySink` | `omega-otel::{Stdout,File}Sink` |
| `provenance` | — | `omega-otel::Provenance` |
| `event-store` (replay) | `EventStore` | `omega-otel::TraceReplayer` |
| `exec-policy` | `ExecPolicy` | `omega-sandbox::RuleBasedExecPolicy` |
| `sandbox-backend` | `SandboxBackend` | `omega-sandbox::LocalSandboxBackend` |
| `permission-policy` | `PermissionPolicy` | `omega-sandbox::ApprovalQueue` |
| `task-classifier` | `TaskClassifier` | `omega-router::HeuristicTaskClassifier` |
| `router-policy` | `ScoringPolicy`/`Selector`/`FallbackPolicy` | `omega-router::Composite…`/`Pareto…`/`TierFallback…` |
| `harness-catalog` | — | `omega-harness` |

Sinon `planned`. `rejected` si on décide de ne pas le reprendre (dire pourquoi dans `how`).

## Récit markdown (`dissections/<id>.md`)

1. **Fiche** : éditeur, licence, langage, repo, architecture.
2. **Entrées** : binaires/CLI, points d'extension, fichiers de config.
3. **Inventaire fonctionnel** : table `fonctionnalité | domaine | ce que ça fait | où (file:line) | cible Omega | statut | coût`.
4. **Traduction en plugins** : pour chaque fonctionnalité retenue, le triplet
   `capacité → port → adaptateur`, et le mode d'intégration.
5. **Écarté** : ce qu'on ne reprend pas et pourquoi.
6. **Sources** : URLs + chemins.
