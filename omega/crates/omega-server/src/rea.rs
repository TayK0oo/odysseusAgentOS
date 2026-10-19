//! Endpoints **REA** (pont MCP) — W5/W7 : l'outil de reverse-engineering est
//! observable et appelable depuis le cockpit.
//!
//! - `GET  /api/rea`       → instantané embarqué (`rea doctor`/`capabilities`) ;
//! - `GET  /api/rea/tools` → catalogue **live** (`tools/list` du serveur MCP) ;
//! - `POST /api/rea/call`  → appel d'un outil (`tools/call`).
//!
//! Les deux derniers lancent `npx -y rea-agents@latest --mcp` : ils exigent Node
//! et le réseau (première fois). L'instantané, lui, est **déterministe** (données
//! embarquées) et alimente la vue sans dépendance externe.

use std::time::Duration;

use axum::extract::State;
use axum::Json;
use omega_mcp::{rea_capabilities, rea_snapshot, McpToolSurface};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::routes::ApiError;
use crate::state::AppState;

/// Délai maximal d'un appel live (le spawn `npx` peut être lent à froid).
const REA_TIMEOUT: Duration = Duration::from_secs(90);

/// Surface REA, préfixe d'outils `rea::`.
fn surface() -> McpToolSurface {
    McpToolSurface::rea().with_timeout(REA_TIMEOUT)
}

/// Instantané REA : version, catalogue (153 commandes CLI, 207 outils MCP,
/// 6 prompts), digests, providers disponibles sur cette machine.
pub async fn rea() -> Json<Value> {
    let snapshot = rea_snapshot();
    let capabilities = rea_capabilities();
    Json(json!({
        "snapshot": snapshot,
        "capabilities": capabilities,
        "bridge": {
            "plugin_id": "mcp:rea",
            "spec": McpToolSurface::rea().spec(),
            "prefix": "rea::",
            "protocol_version": omega_mcp::MCP_PROTOCOL_VERSION,
        }
    }))
}

#[derive(Debug, Serialize)]
pub struct ReaToolsResponse {
    pub count: usize,
    pub tools: Vec<Value>,
}

/// Catalogue live des outils MCP de REA.
pub async fn rea_tools() -> Result<Json<ReaToolsResponse>, ApiError> {
    let tools = surface()
        .list_tools()
        .await
        .map_err(ApiError::from_domain)?;
    let tools = tools
        .into_iter()
        .map(|t| {
            json!({
                "name": t.name,
                "prefixed": format!("rea::{}", t.name),
                "description": t.description,
                "input_schema": t.input_schema,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(ReaToolsResponse {
        count: tools.len(),
        tools,
    }))
}

/// Corps de `POST /api/rea/call`.
#[derive(Debug, Deserialize)]
pub struct ReaCallRequest {
    /// Nom **nu** de l'outil (ex. `analyze`, `inspect-binary-layout`).
    pub tool: String,
    #[serde(default)]
    pub arguments: Value,
}

/// Appel d'un outil REA ; renvoie le résultat MCP brut (contenu + Evidence).
pub async fn rea_call(
    State(_state): State<std::sync::Arc<AppState>>,
    Json(req): Json<ReaCallRequest>,
) -> Result<Json<Value>, ApiError> {
    if req.tool.trim().is_empty() {
        return Err(ApiError::bad_request("champ 'tool' requis".to_string()));
    }
    let result = surface()
        .call_tool(&req.tool, req.arguments)
        .await
        .map_err(ApiError::from_domain)?;
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn snapshot_endpoint_embeds_catalog_and_bridge() {
        let Json(v) = rea().await;
        assert_eq!(v["snapshot"]["rea_version"], "6.5.0");
        assert_eq!(v["snapshot"]["catalog"]["mcp_tools"], 207);
        assert_eq!(v["bridge"]["prefix"], "rea::");
        assert_eq!(v["bridge"]["spec"]["command"], "npx");
        assert!(v["capabilities"]["capabilities"].is_array());
    }

    #[tokio::test]
    async fn call_requires_tool_name() {
        let state = crate::state::build_app_state_with(&crate::config::ServerConfig::default())
            .expect("etat");
        let err = rea_call(
            State(std::sync::Arc::new(state)),
            Json(ReaCallRequest {
                tool: "  ".into(),
                arguments: json!({}),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
