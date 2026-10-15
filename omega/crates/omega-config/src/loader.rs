//! `omega-config::loader` — chargement minimal multi-sources (defaults + TOML + env).
//!
//! Sans dependance lourde : un seul crate `toml` (parsing), aucune surcouche type
//! `figment`/`config`. Les couches sont fusionnees par [`LayeredConfig`] selon leur
//! provenance (`defaults < projet < session`, cf. [`ConfigLayerSource::precedence`]).

use std::path::Path;

use serde_json::{Map, Value};

use crate::{ConfigLayer, ConfigLayerSource, LayeredConfig};
use omega_types::{OmegaError, OmegaResult};

/// Valeurs par defaut (hardcodees, minimales).
///
/// Increment 1 : uniquement le chemin du journal d'evenements durable, consomme par le
/// contributeur `FileEventStore` (cf. `omega-app`). Le reste vient de `omega.toml`/env.
pub fn defaults() -> ConfigLayer {
    let mut layer = ConfigLayer::new(ConfigLayerSource::Defaults);
    set_path(
        &mut layer.values,
        "event_store.path",
        Value::String(".omega/events".to_string()),
    );
    layer
}

/// Parse une couche `Project` depuis une chaine TOML.
pub fn from_toml_str(text: &str) -> OmegaResult<ConfigLayer> {
    let value: toml::Value =
        toml::from_str(text).map_err(|e| OmegaError::Config(format!("TOML illisible : {e}")))?;
    let json = serde_json::to_value(value)
        .map_err(|e| OmegaError::Config(format!("conversion TOML->JSON : {e}")))?;
    let map = json
        .as_object()
        .cloned()
        .ok_or_else(|| OmegaError::Config("le TOML doit etre une table racine".to_string()))?;
    Ok(ConfigLayer {
        source: ConfigLayerSource::Project,
        values: map,
    })
}

/// Charge une couche `Project` depuis un fichier TOML.
pub fn from_toml_file(path: impl AsRef<Path>) -> OmegaResult<ConfigLayer> {
    let text = std::fs::read_to_string(path.as_ref())
        .map_err(|e| OmegaError::Config(format!("lecture {} : {e}", path.as_ref().display())))?;
    from_toml_str(&text)
}

/// Couche `Session` depuis les variables d'environnement prefixees.
///
/// `OMEGA_EVENT_STORE__PATH=...` -> cle `event_store.path`. Le separateur `__` cree la
/// hierarchie ; `_` est conserve dans le segment (minusculise). Les valeurs sont
/// interpretees comme JSON quand c'est possible (nombre, booleen, ...), sinon comme chaine.
pub fn from_env(prefix: &str) -> ConfigLayer {
    let mut layer = ConfigLayer::new(ConfigLayerSource::Session);
    for (key, raw) in std::env::vars() {
        let Some(rest) = key.strip_prefix(prefix) else {
            continue;
        };
        let path = env_key_to_path(rest);
        if path.is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(&raw).unwrap_or(Value::String(raw));
        set_path(&mut layer.values, &path, value);
    }
    layer
}

/// Charge la config minimale : defaults + `omega.toml` optionnel + env.
///
/// Le fichier `omega.toml` absent est ignore (pas d'erreur) : c'est la source optionnelle.
pub fn load_minimal(project_toml: Option<&Path>, env_prefix: &str) -> OmegaResult<LayeredConfig> {
    let mut cfg = LayeredConfig::new().with(defaults());
    if let Some(path) = project_toml {
        if path.is_file() {
            cfg.push(from_toml_file(path)?);
        }
    }
    cfg.push(from_env(env_prefix));
    Ok(cfg)
}

/// Convertit le reste d'une variable d'environnement en chemin de cle (`a.b.c`).
fn env_key_to_path(rest: &str) -> String {
    rest.trim_start_matches('_')
        .split("__")
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(".")
}

/// Insere `value` a la cle `a.b.c` d'une table JSON (cree les tables intermediaires).
fn set_path(map: &mut Map<String, Value>, path: &str, value: Value) {
    let mut segments: Vec<&str> = path.split('.').collect();
    let Some(last) = segments.pop() else {
        return;
    };
    let mut cursor = map;
    for seg in segments {
        let entry = cursor
            .entry(seg.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(table) = entry.as_object_mut() else {
            return;
        };
        cursor = table;
    }
    cursor.insert(last.to_string(), value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConfigLayerSource;

    #[test]
    fn toml_layer_parses_nested_table() {
        let layer =
            from_toml_str("[event_store]\npath = \"custom\"\n[run]\nresume = true\n").unwrap();
        assert_eq!(layer.source, ConfigLayerSource::Project);
        assert_eq!(layer.values["event_store"]["path"], "custom");
        assert_eq!(layer.values["run"]["resume"], true);
    }

    #[test]
    fn toml_rejects_invalid() {
        let err = from_toml_str("not [ valid toml").unwrap_err();
        assert!(matches!(err, OmegaError::Config(_)));
    }

    #[test]
    fn env_key_to_path_maps_nesting() {
        assert_eq!(env_key_to_path("EVENT_STORE__PATH"), "event_store.path");
        assert_eq!(env_key_to_path("MODEL"), "model");
    }

    #[test]
    fn set_path_builds_nested_map() {
        let mut map = Map::new();
        set_path(&mut map, "a.b.c", Value::from(42));
        assert_eq!(map["a"]["b"]["c"], 42);
        // ecrase sans casser la hierarchie existante
        set_path(&mut map, "a.d", Value::Bool(true));
        assert_eq!(map["a"]["d"], true);
        assert_eq!(map["a"]["b"]["c"], 42);
    }

    #[test]
    fn defaults_expose_event_store_path() {
        let layer = defaults();
        assert_eq!(layer.source, ConfigLayerSource::Defaults);
        assert_eq!(layer.values["event_store"]["path"], ".omega/events");
    }

    #[test]
    fn load_minimal_merges_defaults_toml_and_env() {
        // Env : on cible un prefixe inconnu pour rester deterministe en test parallele.
        let dir = std::env::temp_dir().join(format!("omega-config-loader-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let toml_path = dir.join("omega.toml");
        std::fs::write(
            &toml_path,
            "[event_store]\npath = \"custom/events\"\n[run]\nresume = true\n",
        )
        .unwrap();

        let cfg = load_minimal(Some(&toml_path), "OMEGA_NO_SUCH_PREFIX_").unwrap();

        // Le projet ecrase le defaut ; la cle run.resume vient du TOML.
        assert_eq!(
            cfg.get("event_store"),
            Some(serde_json::json!({"path": "custom/events"}))
        );
        assert_eq!(cfg.get("run"), Some(serde_json::json!({"resume": true})));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
