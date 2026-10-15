//! Catalogue local Ollama : source `CatalogSource` + record canonique cout nul.
//!
//! Alimente le registre depuis `GET {base_url}/api/tags` (aucun LLM, aucun cout).
//! La regle de preference self-hosted est deterministe (cf. [`self_hosted_first`]).

use std::collections::HashMap;

use async_trait::async_trait;
use omega_types::{
    Capabilities, ModelRecord, ModelTier, OmegaError, OmegaResult, QualityAxes, RawModel,
};
use serde::Deserialize;

/// Marge de preference self-hosted (points de score composite, cf. §2.4 benchmarks).
///
/// Enonce : un candidat local eligible gagne si `score_local + marge >= score_distant`.
/// Valeur faible et explicite : elle ne renverse pas un ecart de qualite reel,
/// elle tranche les quasi-egalites en faveur du cout nul.
pub const SELF_HOST_MARGIN: f64 = 0.05;

/// Contexte par defaut (tokens) quand Ollama ne le declare pas.
///
/// `/api/tags` n'expose pas `num_ctx` ; sans indice explicite on retient une
/// valeur conservatrice (ordre de grandeur du `num_ctx` par defaut d'Ollama).
pub const DEFAULT_LOCAL_CONTEXT: u32 = 8192;

/// Axes de qualite conservateurs d'un modele local.
///
/// Un petit modele self-hosted ne rivalise pas avec un `Elite` cloud (80-95) ;
/// ces valeurs modestes servent au tie-break/Pareto, pas a un classement absolu.
pub const LOCAL_AXES: QualityAxes = QualityAxes {
    reasoning: 50,
    code: 65,
    agentic: 35,
    knowledge: 45,
    chat: 55,
    vision: 0,
};

/// Reponse `GET /api/tags` (sous-ensemble utile : seul `name` est contractuel).
#[derive(Debug, Clone, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagsEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct TagsEntry {
    #[serde(default)]
    name: String,
}

/// Source de catalogue adossee a un daemon Ollama local.
///
/// `GET {base_url}/api/tags` -> un [`RawModel`] par entree :
/// `id = "ollama/<name>"`, `provider = "ollama"`,
/// `context_length` = indice connu si fourni, sinon `None`.
pub struct OllamaCatalogSource {
    base_url: String,
    client: reqwest::Client,
    known_context: HashMap<String, u32>,
}

impl OllamaCatalogSource {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
            known_context: HashMap::new(),
        }
    }

    /// Declare un contexte connu pour un modele (ex. via `/api/show` ou config).
    pub fn with_known_context(mut self, name: impl Into<String>, context: u32) -> Self {
        self.known_context.insert(name.into(), context);
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn endpoint(&self) -> String {
        format!("{}/api/tags", self.base_url)
    }

    /// Mappe une reponse `/api/tags` deja deserialisee (pur, sans reseau).
    pub fn parse_tags(&self, value: serde_json::Value) -> Vec<RawModel> {
        parse_tags_response(value, &self.known_context)
    }
}

/// Mappe une reponse `/api/tags` (fixture ou reseau) vers des [`RawModel`].
///
/// Robuste : `models` absent ou non-tableau -> vecteur vide ; entrees sans
/// `name` (vide) -> ignorees. Fonction libre pour test sans instance.
pub fn parse_tags_response(
    value: serde_json::Value,
    known_context: &HashMap<String, u32>,
) -> Vec<RawModel> {
    let parsed: TagsResponse =
        serde_json::from_value(value).unwrap_or(TagsResponse { models: Vec::new() });
    parsed
        .models
        .into_iter()
        .map(|e| e.name.trim().to_string())
        .filter(|name| !name.is_empty())
        .map(|name| RawModel {
            id: format!("ollama/{name}"),
            provider: "ollama".to_string(),
            context_length: known_context.get(&name).copied(),
        })
        .collect()
}

/// Construit le [`ModelRecord`] canonique d'un modele local.
///
/// - couts `0.0` (self-hosted, electricite negligee a l'echelle du run) ;
/// - `tier = SelfHosted` ;
/// - axes [`LOCAL_AXES`] (modestes, documentes) ;
/// - capacites `default` (aucune garantie tool-calling/vision/raisonnement
///   pilotable : le filtre dur `covers` reste honnete) ;
/// - debit/TTFT d'ordre de grandeur local (`40 tok/s`, `500 ms`) ;
/// - contexte fourni ou [`DEFAULT_LOCAL_CONTEXT`].
pub fn local_record(name: &str, context_length: Option<u32>) -> ModelRecord {
    ModelRecord {
        id: format!("ollama/{name}"),
        provider: "ollama".to_string(),
        context_length: context_length.unwrap_or(DEFAULT_LOCAL_CONTEXT),
        cost_input_per_1m: 0.0,
        cost_output_per_1m: 0.0,
        tokens_per_second: 40.0,
        ttft_ms: 500,
        axes: LOCAL_AXES,
        capabilities: Capabilities::default(),
        tier: ModelTier::SelfHosted,
    }
}

/// Regle self-host-first (deterministe, sans LLM).
///
/// Enonce exact : **un candidat `tier == SelfHosted` qui couvre les capacites
/// requises et le contexte requis est prefere des que
/// `score_local + SELF_HOST_MARGIN >= score_distant`.**
///
/// - `local.tier != SelfHosted` -> `false` ;
/// - local ineligible (capacites ou contexte) -> `false` (le local perd
///   si le contexte est insuffisant) ;
/// - distant ineligible alors que le local est eligible -> `true` ;
/// - sinon comparaison des scores avec marge.
pub fn self_hosted_first(
    local: &ModelRecord,
    local_score: f64,
    remote: &ModelRecord,
    remote_score: f64,
    required_capabilities: Capabilities,
    required_context: u32,
) -> bool {
    if !matches!(local.tier, ModelTier::SelfHosted) {
        return false;
    }
    if !local.capabilities.covers(required_capabilities) || local.context_length < required_context
    {
        return false;
    }
    if !remote.capabilities.covers(required_capabilities)
        || remote.context_length < required_context
    {
        return true;
    }
    local_score + SELF_HOST_MARGIN >= remote_score
}

#[async_trait]
impl omega_ports::CatalogSource for OllamaCatalogSource {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn fetch(&self) -> OmegaResult<Vec<RawModel>> {
        let resp = self
            .client
            .get(self.endpoint())
            .send()
            .await
            .map_err(|e| OmegaError::Io(format!("ollama catalogue requete : {e}")))?;
        if !resp.status().is_success() {
            return Err(OmegaError::Io(format!(
                "ollama catalogue statut HTTP {}",
                resp.status()
            )));
        }
        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| OmegaError::Io(format!("ollama catalogue reponse illisible : {e}")))?;
        Ok(self.parse_tags(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_ports::CatalogSource;

    fn fixture_tags() -> serde_json::Value {
        serde_json::json!({
            "models": [
                {"name": "qwen2.5-coder:7b", "model": "qwen2.5-coder:7b", "size": 4683075277u64},
                {"name": "llama3.1:8b", "model": "llama3.1:8b", "size": 4920753324u64},
                {"name": "  ", "model": "vide"},
                {"model": "sans-nom"}
            ]
        })
    }

    fn remote_record(id: &str, ctx: u32, caps: Capabilities) -> ModelRecord {
        ModelRecord {
            id: id.to_string(),
            provider: "openrouter".to_string(),
            context_length: ctx,
            cost_input_per_1m: 0.5,
            cost_output_per_1m: 1.5,
            tokens_per_second: 120.0,
            ttft_ms: 800,
            axes: QualityAxes {
                reasoning: 80,
                code: 85,
                agentic: 70,
                knowledge: 80,
                chat: 80,
                vision: 0,
            },
            capabilities: caps,
            tier: ModelTier::Standard,
        }
    }

    #[test]
    fn parse_tags_fixture_maps_ids_provider_and_skips_blank() {
        let raws = parse_tags_response(fixture_tags(), &HashMap::new());
        let ids: Vec<&str> = raws.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec!["ollama/qwen2.5-coder:7b", "ollama/llama3.1:8b"]);
        assert!(raws.iter().all(|r| r.provider == "ollama"));
        assert!(raws.iter().all(|r| r.context_length.is_none()));
    }

    #[test]
    fn parse_tags_applies_known_context_hints() {
        let mut known = HashMap::new();
        known.insert("qwen2.5-coder:7b".to_string(), 32_768);
        let raws = parse_tags_response(fixture_tags(), &known);
        assert_eq!(raws[0].context_length, Some(32_768));
        assert_eq!(raws[1].context_length, None);
    }

    #[test]
    fn parse_tags_tolerates_missing_models_key() {
        let raws = parse_tags_response(serde_json::json!({"autre": 1}), &HashMap::new());
        assert!(raws.is_empty());
    }

    #[test]
    fn source_parse_tags_uses_instance_hints_and_trims_base_url() {
        let src = OllamaCatalogSource::new("http://localhost:11434/")
            .with_known_context("llama3.1:8b", 8192);
        assert_eq!(src.base_url(), "http://localhost:11434");
        assert_eq!(src.name(), "ollama");
        let raws = src.parse_tags(fixture_tags());
        assert_eq!(raws[1].context_length, Some(8192));
    }

    #[test]
    fn local_record_is_zero_cost_self_hosted() {
        let rec = local_record("qwen2.5-coder:7b", Some(32_768));
        assert_eq!(rec.id, "ollama/qwen2.5-coder:7b");
        assert_eq!(rec.provider, "ollama");
        assert_eq!(rec.context_length, 32_768);
        assert_eq!(rec.cost_input_per_1m, 0.0);
        assert_eq!(rec.cost_output_per_1m, 0.0);
        assert_eq!(rec.blended_cost_per_1m(), 0.0);
        assert!(matches!(rec.tier, ModelTier::SelfHosted));
        assert_eq!(rec.capabilities, Capabilities::default());
        assert_eq!(rec.axes, LOCAL_AXES);
    }

    #[test]
    fn local_record_defaults_context_when_unknown() {
        let rec = local_record("tiny:1b", None);
        assert_eq!(rec.context_length, DEFAULT_LOCAL_CONTEXT);
    }

    #[test]
    fn self_host_first_wins_at_close_score() {
        let local = local_record("qwen2.5-coder:7b", Some(32_768));
        let remote = remote_record("cloud-elite", 128_000, Capabilities::default());
        // Ecart inferieur a la marge -> le local cout nul gagne.
        assert!(self_hosted_first(
            &local,
            0.70,
            &remote,
            0.74,
            Capabilities::default(),
            4000
        ));
        // Egalite stricte -> le local gagne.
        assert!(self_hosted_first(
            &local,
            0.70,
            &remote,
            0.70,
            Capabilities::default(),
            4000
        ));
    }

    #[test]
    fn self_host_first_loses_on_large_quality_gap() {
        let local = local_record("qwen2.5-coder:7b", Some(32_768));
        let remote = remote_record("cloud-elite", 128_000, Capabilities::default());
        assert!(!self_hosted_first(
            &local,
            0.50,
            &remote,
            0.90,
            Capabilities::default(),
            4000
        ));
    }

    #[test]
    fn self_host_first_loses_when_local_context_insufficient() {
        let local = local_record("tiny:1b", Some(4096));
        let remote = remote_record("cloud-elite", 128_000, Capabilities::default());
        assert!(!self_hosted_first(
            &local,
            0.95,
            &remote,
            0.50,
            Capabilities::default(),
            8000
        ));
    }

    #[test]
    fn self_host_first_loses_when_local_misses_capability() {
        let local = local_record("qwen2.5-coder:7b", Some(32_768));
        let remote = remote_record(
            "cloud-tools",
            128_000,
            Capabilities {
                function_calling: true,
                ..Default::default()
            },
        );
        let required = Capabilities {
            function_calling: true,
            ..Default::default()
        };
        assert!(!self_hosted_first(&local, 0.99, &remote, 0.10, required, 0));
    }

    #[test]
    fn self_host_first_requires_self_hosted_tier_and_prefers_eligible_over_ineligible_remote() {
        let not_local = remote_record("cloud-a", 128_000, Capabilities::default());
        let remote = remote_record("cloud-b", 128_000, Capabilities::default());
        // Un non-SelfHosted ne declenche jamais la regle, meme a score superieur.
        assert!(!self_hosted_first(
            &not_local,
            0.99,
            &remote,
            0.10,
            Capabilities::default(),
            0
        ));
        // Local eligible vs distant hors-contexte -> le local gagne quel que soit le score.
        let local = local_record("qwen2.5-coder:7b", Some(32_768));
        let small_remote = remote_record("cloud-small", 1024, Capabilities::default());
        assert!(self_hosted_first(
            &local,
            0.10,
            &small_remote,
            0.99,
            Capabilities::default(),
            4000
        ));
    }
}
