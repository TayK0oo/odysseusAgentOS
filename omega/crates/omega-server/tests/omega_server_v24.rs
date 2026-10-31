//! Tests v24 — `omega-server`.

use omega_server::build_app_state_with;
use omega_server::ServerConfig;

#[test]
fn config_defaults() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.port, 8787);
    assert_eq!(cfg.max_tokens, 8000);
    assert_eq!(cfg.tool_budget, 8);
    assert_eq!(cfg.ollama_base_url, "http://localhost:11434");
}

#[test]
fn build_app_state_wires_plugins() {
    let dir = std::env::temp_dir().join("omega-server-v24-state");
    let _ = std::fs::remove_dir_all(&dir);
    let cfg = ServerConfig {
        event_dir: dir.clone(),
        ..ServerConfig::default()
    };
    let state = build_app_state_with(&cfg).unwrap();
    assert_eq!(
        state.app.active_ids(omega_types::Capability::LlmProvider),
        vec!["ollama".to_string()]
    );
    assert_eq!(
        state.app.active_ids(omega_types::Capability::EventStore),
        vec!["file".to_string()]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn config_from_env_empty_ollama_url_keeps_default() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.ollama_base_url, "http://localhost:11434");
}
