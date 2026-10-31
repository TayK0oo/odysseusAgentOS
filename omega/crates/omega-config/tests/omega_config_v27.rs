//! Tests v27 — `omega-config`.

use omega_config::{loader, ConfigLayer, ConfigLayerSource, LayeredConfig};

#[test]
fn precedence_order() {
    assert!(ConfigLayerSource::Defaults.precedence() < ConfigLayerSource::User.precedence());
    assert!(ConfigLayerSource::User.precedence() < ConfigLayerSource::Project.precedence());
    assert!(ConfigLayerSource::Project.precedence() < ConfigLayerSource::Session.precedence());
}

#[test]
fn layered_config_merge() {
    let mut cfg = LayeredConfig::new();
    let mut layer = ConfigLayer::new(ConfigLayerSource::Defaults);
    layer = layer.set("key", serde_json::json!("default"));
    cfg.push(layer);
    assert_eq!(cfg.get("key").unwrap(), serde_json::json!("default"));
}

#[test]
fn layered_config_override() {
    let mut cfg = LayeredConfig::new();
    let mut d = ConfigLayer::new(ConfigLayerSource::Defaults);
    d = d.set("key", serde_json::json!("default"));
    cfg.push(d);
    let mut p = ConfigLayer::new(ConfigLayerSource::Project);
    p = p.set("key", serde_json::json!("project"));
    cfg.push(p);
    assert_eq!(cfg.get("key").unwrap(), serde_json::json!("project"));
}

#[test]
fn source_of_returns_layer() {
    let mut cfg = LayeredConfig::new();
    let mut d = ConfigLayer::new(ConfigLayerSource::Defaults);
    d = d.set("key", serde_json::json!("default"));
    cfg.push(d);
    assert_eq!(cfg.source_of("key"), Some(ConfigLayerSource::Defaults));
}

#[test]
fn from_toml_str() {
    let layer = loader::from_toml_str("[event_store]\npath = \"custom\"\n").unwrap();
    assert_eq!(layer.values["event_store"]["path"], "custom");
}

#[test]
fn from_toml_str_rejects_invalid() {
    assert!(loader::from_toml_str("not [ valid toml").is_err());
}
