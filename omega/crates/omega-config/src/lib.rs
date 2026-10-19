//! `omega-config` — configuration **a couches** avec provenance et precedence testable.
//!
//! Inspiré de Codex (`ConfigLayerSource::precedence()`, cf. `RE-GOOSE-CODEX.md`), mais
//! volontairement reduit a 4 couches (defaults < user < projet < session).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub mod loader;

/// Origine d'une couche de configuration. L'ordre de declaration **est** la precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigLayerSource {
    Defaults,
    User,
    Project,
    Session,
}

impl ConfigLayerSource {
    /// Precedence numerique : plus grand = gagne. Testable (cf. tests).
    pub fn precedence(self) -> i16 {
        match self {
            Self::Defaults => 0,
            Self::User => 10,
            Self::Project => 20,
            Self::Session => 30,
        }
    }
}

/// Une couche : une origine + des valeurs clé/valeur.
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    pub source: ConfigLayerSource,
    pub values: Map<String, Value>,
}

impl ConfigLayer {
    pub fn new(source: ConfigLayerSource) -> Self {
        Self {
            source,
            values: Map::new(),
        }
    }

    /// Ajoute/ecrase une valeur dans cette couche (builder).
    pub fn set(mut self, key: impl Into<String>, value: Value) -> Self {
        self.values.insert(key.into(), value);
        self
    }
}

/// Configuration multi-couches. Les couches basses sont ecrasees par les hautes.
#[derive(Debug, Clone, Default)]
pub struct LayeredConfig {
    layers: Vec<ConfigLayer>,
}

impl LayeredConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Empile une couche (builder).
    pub fn with(mut self, layer: ConfigLayer) -> Self {
        self.push(layer);
        self
    }

    pub fn push(&mut self, layer: ConfigLayer) {
        self.layers.push(layer);
    }

    /// Fusionne toutes les couches selon la precedence.
    pub fn merged(&self) -> Map<String, Value> {
        let mut ordered = self.layers.clone();
        ordered.sort_by_key(|l| l.source.precedence());
        let mut out = Map::new();
        for layer in ordered {
            for (k, v) in layer.values {
                out.insert(k, v);
            }
        }
        out
    }

    /// Valeur effective d'une clé.
    pub fn get(&self, key: &str) -> Option<Value> {
        self.merged().get(key).cloned()
    }

    /// Provenance d'une valeur : la couche (la plus haute en precedence) qui la fournit.
    /// Règle n°2 du noyau : toute valeur de config vient d'une couche identifiable.
    pub fn source_of(&self, key: &str) -> Option<ConfigLayerSource> {
        let mut ordered = self.layers.clone();
        ordered.sort_by_key(|l| l.source.precedence());
        let mut found: Option<ConfigLayerSource> = None;
        for layer in ordered {
            if layer.values.contains_key(key) || has_nested(&layer.values, key) {
                found = Some(layer.source);
            }
        }
        found
    }

    /// Valeur effective + sa couche d'origine (provenance).
    pub fn get_with_provenance(&self, key: &str) -> Option<(Value, ConfigLayerSource)> {
        self.get(key).map(|v| {
            (
                v,
                self.source_of(key).unwrap_or(ConfigLayerSource::Defaults),
            )
        })
    }
}

/// Indique si une table contient (recursivement) la cle `a.b.c`.
fn has_nested(map: &serde_json::Map<String, Value>, key: &str) -> bool {
    let mut cursor = map;
    let mut segs = key.split('.').peekable();
    while let Some(seg) = segs.next() {
        match cursor.get(seg) {
            Some(Value::Object(m)) if segs.peek().is_some() => cursor = m,
            Some(_) => return true,
            None => return false,
        }
    }
    true
}

// ═══════════════════════════════════════════════════════════════════════════
// Port ConfigSource — adaptateur omega-config
// ═══════════════════════════════════════════════════════════════════════════

use async_trait::async_trait;
use omega_ports::ConfigSource as ConfigSourcePort;

impl std::fmt::Display for ConfigLayerSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Defaults => write!(f, "defaults"),
            Self::User => write!(f, "user"),
            Self::Project => write!(f, "project"),
            Self::Session => write!(f, "session"),
        }
    }
}

pub struct LayeredConfigSource {
    name: String,
    layers: std::sync::RwLock<Vec<ConfigLayer>>,
}

impl Default for LayeredConfigSource {
    fn default() -> Self {
        Self::new()
    }
}

impl LayeredConfigSource {
    pub fn new() -> Self {
        Self {
            name: "layered-config".into(),
            layers: std::sync::RwLock::new(vec![]),
        }
    }

    pub fn add_layer(&self, layer: ConfigLayer) {
        self.layers.write().unwrap().push(layer);
    }
}

#[async_trait]
impl ConfigSourcePort for LayeredConfigSource {
    fn name(&self) -> &str {
        &self.name
    }
    async fn load(&self, layer: &str) -> omega_types::OmegaResult<Value> {
        let layers = self.layers.read().unwrap();
        for l in layers.iter().rev() {
            if l.source.to_string() == layer || layer == "all" {
                return Ok(Value::Object(l.values.clone()));
            }
        }
        Ok(Value::Object(serde_json::Map::new()))
    }
    async fn save(&self, _layer: &str, _value: Value) -> omega_types::OmegaResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn precedence_order_is_stable() {
        assert!(ConfigLayerSource::Defaults < ConfigLayerSource::User);
        assert!(ConfigLayerSource::User < ConfigLayerSource::Project);
        assert!(ConfigLayerSource::Project < ConfigLayerSource::Session);
        assert_eq!(ConfigLayerSource::Session.precedence(), 30);
    }

    #[test]
    fn higher_layer_wins() {
        // (d) precedence : user < projet < session.
        let cfg = LayeredConfig::new()
            .with(
                ConfigLayer::new(ConfigLayerSource::User)
                    .set("model", json!("cheap"))
                    .set("budget", json!(100)),
            )
            .with(ConfigLayer::new(ConfigLayerSource::Project).set("model", json!("standard")))
            .with(ConfigLayer::new(ConfigLayerSource::Session).set("model", json!("elite")));

        assert_eq!(cfg.get("model"), Some(json!("elite")));
        assert_eq!(cfg.get("budget"), Some(json!(100)));
    }

    #[test]
    fn merge_is_order_independent_of_insertion() {
        let a = ConfigLayer::new(ConfigLayerSource::Project).set("x", json!(1));
        let b = ConfigLayer::new(ConfigLayerSource::User).set("x", json!(2));
        // insere dans l'ordre inverse de la precedence : le resultat doit rester identique
        let cfg = LayeredConfig::new().with(a).with(b);
        assert_eq!(cfg.get("x"), Some(json!(1)));
    }

    #[test]
    fn provenance_reports_originating_layer() {
        let cfg = LayeredConfig::new()
            .with(ConfigLayer::new(ConfigLayerSource::User).set("model", json!("cheap")))
            .with(ConfigLayer::new(ConfigLayerSource::Project).set("model", json!("standard")));
        assert_eq!(cfg.source_of("model"), Some(ConfigLayerSource::Project));
        let (v, src) = cfg.get_with_provenance("model").unwrap();
        assert_eq!(v, json!("standard"));
        assert_eq!(src, ConfigLayerSource::Project);
    }
}
