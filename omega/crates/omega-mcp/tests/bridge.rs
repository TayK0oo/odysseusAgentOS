//! Tests d'intégration du pont MCP : dialogue réel avec le serveur **stub**
//! (`src/bin/mcp_stub.rs`) — déterministe, aucun réseau.

use omega_mcp::{McpClient, McpServerSpec, McpToolSurface, MCP_PROTOCOL_VERSION};
use omega_types::OmegaError;
use serde_json::json;
use std::sync::Arc;

fn stub_spec() -> McpServerSpec {
    McpServerSpec::new(env!("CARGO_BIN_EXE_mcp_stub"), Vec::<String>::new())
}

async fn initialized_client() -> McpClient {
    let mut c = McpClient::spawn(&stub_spec()).await.expect("spawn stub");
    c.initialize().await.expect("initialize");
    c
}

#[tokio::test]
async fn initialize_returns_server_info() {
    let c = initialized_client().await;
    let info = c.server_info().expect("serverInfo");
    assert_eq!(info.protocol_version, MCP_PROTOCOL_VERSION);
    assert_eq!(info.server_info["name"], "mcp-stub");
}

#[tokio::test]
async fn list_tools_returns_catalog() {
    let mut c = initialized_client().await;
    let tools = c.list_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, vec!["echo", "sum", "boom"]);
    assert!(!tools[0].description.is_empty());
    assert_eq!(tools[0].input_schema["type"], "object");
}

#[tokio::test]
async fn call_tool_echoes_and_sums() {
    let mut c = initialized_client().await;
    let r = c
        .call_tool("echo", json!({"text": "bonjour"}))
        .await
        .unwrap();
    assert_eq!(r["structuredContent"]["echoed"], "bonjour");
    let r = c
        .call_tool("sum", json!({"numbers": [2, 3, 5]}))
        .await
        .unwrap();
    assert_eq!(r["structuredContent"]["sum"], 10);
}

#[tokio::test]
async fn tool_failure_is_an_error_not_a_success() {
    let mut c = initialized_client().await;
    let err = c.call_tool("boom", json!({})).await.unwrap_err();
    assert!(err.to_string().contains("boom"), "{err}");
}

#[tokio::test]
async fn unknown_method_returns_plugin_failed() {
    let mut c = initialized_client().await;
    let err = c
        .call_tool("outil-inexistant", json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, OmegaError::PluginFailed(_, _)), "{err:?}");
}

#[tokio::test]
async fn spawn_failure_is_io_error() {
    let spec = McpServerSpec::new("/nonexistent/mcp-server-xyz", Vec::<String>::new());
    match McpClient::spawn(&spec).await {
        Ok(_) => panic!("le spawn aurait dû échouer"),
        Err(e) => assert!(matches!(e, OmegaError::Io(_)), "{e:?}"),
    }
}

#[tokio::test]
async fn surface_lists_and_calls_over_ephemeral_server() {
    let surface = McpToolSurface::new(stub_spec()).with_prefix("stub::");
    let tools = surface.list_tools().await.unwrap();
    assert_eq!(tools.len(), 3);
    let r = surface
        .call_tool("sum", json!({"numbers": [1, 2]}))
        .await
        .unwrap();
    assert_eq!(r["structuredContent"]["sum"], 3);
}

#[tokio::test]
async fn executors_expose_prefixed_tool_specs() {
    let surface = Arc::new(McpToolSurface::new(stub_spec()).with_prefix("stub::"));
    let tools = surface.list_tools().await.unwrap();
    let execs = surface.executors(&tools);
    assert_eq!(execs.len(), 3);
    let specs: Vec<String> = execs.iter().map(|e| e.spec().name).collect();
    assert!(specs.contains(&"stub::echo".to_string()));
    let sum = execs.iter().find(|e| e.spec().name == "stub::sum").unwrap();
    let out = sum.execute(json!({"numbers": [4, 6]})).await.unwrap();
    assert_eq!(out["structuredContent"]["sum"], 10);
}
