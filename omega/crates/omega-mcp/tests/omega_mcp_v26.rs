//! Tests v26 — `omega-mcp`.

use omega_mcp::{McpServerSpec, MCP_PROTOCOL_VERSION};

#[test]
fn mcp_protocol_version() {
    assert_eq!(MCP_PROTOCOL_VERSION, "2025-06-18");
}

#[test]
fn mcp_server_spec_new() {
    let spec = McpServerSpec::new("npx", ["-y", "rea-agents@latest", "--mcp"]);
    assert_eq!(spec.command, "npx");
    assert_eq!(spec.args.len(), 3);
}

#[test]
fn mcp_server_spec_with_env() {
    let spec = McpServerSpec::new("npx", ["--mcp"]).with_env("KEY", "val");
    assert_eq!(spec.env.get("KEY").unwrap(), "val");
}

#[test]
fn mcp_server_spec_with_cwd() {
    let spec = McpServerSpec::new("npx", ["--mcp"]).with_cwd("/tmp");
    assert_eq!(spec.cwd.as_deref(), Some("/tmp"));
}

#[test]
fn mcp_server_spec_rea() {
    let spec = McpServerSpec::rea();
    assert_eq!(spec.command, "npx");
    assert!(spec.args.contains(&"rea-agents@latest".to_string()));
}
