# W10 — Self-hosting (catalogue local, cout nul par defaut)

> **Workstream :** W10 · **Increment :** 1 (catalogue local + self-host-first)
> **Date :** 2026-10-15 · **Branche :** `feat/omega-w10-local`
> **Provenance :** `[observed]` (releve `/api/tags`, code) · `[inferred]` (regle, matrice)

## Objectif de l'increment

Un flux complet tourne a **cout nul** (modele local Ollama) sans changer le noyau :
le catalogue local est une source comme une autre (port `CatalogSource`), et la
preference self-hosted est une **regle deterministe** (pas de LLM).

## Livrables

1. **`omega/crates/omega-providers/src/ollama_catalog.rs`** (nouveau) —
   `OllamaCatalogSource`, `local_record`, `self_hosted_first` + 11 tests sans reseau.
2. **`omega/crates/omega-providers/src/lib.rs`** — ajout `mod ollama_catalog` + exports.
3. **`docs/omega/workstreams/W10-selfhosting.md`** (ce document) — contrats, regle, matrice cout.

## Contrats

### `CatalogSource` (port, `omega-ports`) — inchange `[observed]`

```rust
trait CatalogSource {
    fn name(&self) -> &str;
    async fn fetch(&self) -> OmegaResult<Vec<RawModel>>;
}
```

### `OllamaCatalogSource` (adaptateur, `omega-providers`) `[observed]`

- Source : `GET {base_url}/api/tags` (format Ollama : `{ "models": [{ "name": ... }] }`).
- Mapping par modele : `RawModel { id: "ollama/<name>", provider: "ollama",
  context_length: indice_connu(name) }` — `None` si inconnu (`/api/tags`
  n'expose pas `num_ctx`).
- Indices de contexte injectes par configuration (`with_known_context`, ex. via
  `/api/show` ou manifeste) — jamais devines par LLM.
- Robustesse : `models` absent/non-tableau -> `[]` ; entrees sans `name` -> ignorees.

### `local_record(name, context_length)` (fabrique canonique) `[observed]`

| Champ | Valeur | Justification |
|---|---|---|
| `id` / `provider` | `ollama/<name>` / `ollama` | canonique, coherent avec `OllamaProvider::model_id` |
| `cost_input/output_per_1m` | `0.0` | cout nul par defaut (objectif W10) |
| `tier` | `SelfHosted` | declenche la regle de preference + fallback existant |
| `axes` | `LOCAL_AXES` (raisonnement 50, code 65, agentique 35, connaissance 45, chat 55, vision 0) | modestes `[inferred]` : sous les `Elite` cloud (80-95), pour tie-break/Pareto |
| `capabilities` | `default` (aucune) | honnete : pas de garantie tool-calling/vision ; le filtre dur `covers` s'applique |
| `tokens_per_second` / `ttft_ms` | `40.0` / `500` | ordre de grandeur local `[inferred]` (cf. exemple `live_ollama`) |
| `context_length` | fourni ou `DEFAULT_LOCAL_CONTEXT = 8192` | conservateur `[inferred]` (defaut Ollama `num_ctx`) |

## Regle de preference self-host-first `[inferred]`

**Enonce exact :** un candidat `tier == SelfHosted` qui couvre les capacites
requises et le contexte requis est prefere des que
`score_local + SELF_HOST_MARGIN >= score_distant`, avec
`SELF_HOST_MARGIN = 0.05`.

```
self_hosted_first(local, score_local, distant, score_distant, requis_caps, requis_ctx):
  1. local.tier != SelfHosted            -> false
  2. local ineligible (caps ou contexte) -> false   # le local perd si contexte insuffisant
  3. distant ineligible & local eligible  -> true
  4. sinon score_local + 0.05 >= score_distant
```

Proprietes testees (sans reseau) :

- le local gagne a qualite proche (`0.70` vs `0.74`) et a egalite ;
- le local perd sur grand ecart (`0.50` vs `0.90`) ;
- le local perd si contexte insuffisant (`4096 < 8000` requis) ;
- le local perd si capacite manquante (`function_calling` requis) ;
- un non-`SelfHosted` ne declenche jamais la regle ;
- local eligible vs distant ineligible -> local gagne quel que soit le score.

Position dans le routeur : cette regle complete (sans les modifier) les deux
mecanismes W4 existants — tie-break `SelfHosted` du `ParetoSelector` et repli
`TierFallbackPolicy` vers self-hosted. Integration cible (inc. suivant) :
appliquer `self_hosted_first` apres `select` quand le gagnant n'est pas local
mais qu'un candidat local eligible est a `0.05` — `omega-router` hors scope ici.

## Matrice cout `[inferred]` (USD/1M, ordres de grandeur benchmarks §7)

| Classe | Exemple | Entree | Sortie | Cout/tache type | Contexte |
|---|---|---:|---:|---:|---:|
| **Self-hosted (Ollama)** | `qwen2.5-coder:7b` local | **0** | **0** | **0 $** | 8-32k (machine) |
| Gratuit dist. (`:free`) | `gemma-4-12b:free` | 0 | 0 | 0 $ (quota) | 262k |
| Flash efficace | `deepseek-v4.1-flash` | ~0.02 | ~0.89 | ~0.27 $ | 1M |
| Standard | `qwen3.5-122b` | ~0.10 | ~0.40 | ~0.32 $ | 262k |
| Elite raisonnement max | `claude-opus-5.5 (max)` | 4 | 20 | ~5.98 $ | 1M |

Lecture routeur : a qualite suffisante pour T1/T2/T3, le local (`0 $`) domine
toujours le Pareto cout ; la marge `0.05` evite seulement de sacrifier une
tache T4-T6 qui exigerait vraiment un `Elite`.

## Verifications

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
python3 omega/scripts/check_dep_graph.py && python3 omega/scripts/check_dep_graph.py --self-test
```

Tests : 11 nouveaux (`ollama_catalog`), 0 reseau (fixture `/api/tags` en dur).
