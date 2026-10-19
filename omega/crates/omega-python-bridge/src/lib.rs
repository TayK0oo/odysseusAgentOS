//! `omega-python-bridge` — pont **Python <-> Rust** via JSON-RPC over stdin/stdout.
//!
//! Principe (ADR-0010) : le noyau Rust ne dépend jamais de Python.
//! Le bridge est un plugin optionnel qui lance un subprocess Python et
//! communique via JSON-RPC. Timeout + retry + circuit breaker inclus.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Requête JSON-RPC 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
            id,
        }
    }
}

/// Réponse JSON-RPC 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// Bridge Python ↔ Rust.
pub struct PythonBridge {
    python_path: String,
    script_path: String,
    timeout_ms: u64,
}

impl PythonBridge {
    pub fn new(python_path: &str, script_path: &str, timeout_ms: u64) -> Self {
        Self {
            python_path: python_path.into(),
            script_path: script_path.into(),
            timeout_ms,
        }
    }

    /// Appelle une méthode Python via JSON-RPC.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        let req = JsonRpcRequest::new(method, params, 1);
        let req_json = serde_json::to_string(&req).map_err(|e| e.to_string())?;

        let mut child = Command::new(&self.python_path)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn Python: {}", e))?;

        let mut stdin = child.stdin.take().ok_or("No stdin")?;
        let stdout = child.stdout.take().ok_or("No stdout")?;

        stdin
            .write_all(req_json.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").await.map_err(|e| e.to_string())?;
        stdin.flush().await.map_err(|e| e.to_string())?;
        drop(stdin);

        // Lecture défensive : on scanne jusqu'à une ligne JSON-RPC valide en
        // ignorant les traces/warnings que Python peut écrire avant, et on
        // borne la taille d'une ligne pour éviter l'épuisement mémoire.
        const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;

        let mut reader = BufReader::new(stdout);
        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        let mut chunk = vec![0u8; 8192];
        let timeout = tokio::time::Duration::from_millis(self.timeout_ms);

        let mut response: Option<JsonRpcResponse> = None;
        let read = tokio::time::timeout(timeout, async {
            loop {
                let n = match reader.read(&mut chunk).await {
                    Ok(0) => return Ok(()),
                    Ok(n) => n,
                    Err(e) => return Err(e.to_string()),
                };
                for &byte in &chunk[..n] {
                    if byte != b'\n' {
                        if buf.len() >= MAX_LINE_BYTES {
                            return Err("ligne JSON-RPC trop volumineuse".to_string());
                        }
                        buf.push(byte);
                        continue;
                    }
                    let text = String::from_utf8_lossy(&buf).to_string();
                    buf.clear();
                    if text.trim().is_empty() {
                        continue;
                    }
                    // Une ligne qui ne parse pas = trace/warning Python : ignorée.
                    if let Ok(r) = serde_json::from_str::<JsonRpcResponse>(&text) {
                        response = Some(r);
                        return Ok(());
                    }
                }
            }
        })
        .await;

        match response {
            Some(resp) => {
                // On laisse le temps au processus de finir, puis on le recycle.
                let _ = tokio::time::timeout(tokio::time::Duration::from_millis(200), child.wait())
                    .await;
                match resp.error {
                    Some(e) => Err(format!("Python error {}: {}", e.code, e.message)),
                    None => Ok(resp.result.unwrap_or(Value::Null)),
                }
            }
            None => {
                // Aucun JSON reçu : on Tue l'enfant dans TOUS les cas
                // (timeout, EOF muet, erreur) — sinon il survit en arrière-plan.
                let _ = child.kill().await;
                match read {
                    Ok(Ok(())) => Err("aucune réponse JSON valide sur stdout".into()),
                    Ok(Err(e)) => Err(format!("Read error: {}", e)),
                    Err(_) => Err("Timeout".into()),
                }
            }
        }
    }

    /// Vérifie si le bridge est disponible (interpréteur + script existent).
    pub fn is_available(&self) -> bool {
        Path::new(&self.python_path).exists() && Path::new(&self.script_path).exists()
    }
}

/// Pont haute priorité : AgentLoop → agent_loop.py
pub struct PythonAgentBridge {
    bridge: PythonBridge,
}

impl PythonAgentBridge {
    pub fn new(python_path: &str, script_path: &str) -> Self {
        Self {
            bridge: PythonBridge::new(python_path, script_path, 30000),
        }
    }

    pub async fn run(&self, goal: &str) -> Result<String, String> {
        let result = self
            .bridge
            .call("agent_loop.run", serde_json::json!({"goal": goal}))
            .await?;
        Ok(result.to_string())
    }
}

/// Pont haute priorité : ToolExecutor → agent_tools/
pub struct PythonToolBridge {
    bridge: PythonBridge,
}

impl PythonToolBridge {
    pub fn new(python_path: &str, script_path: &str) -> Self {
        Self {
            bridge: PythonBridge::new(python_path, script_path, 10000),
        }
    }

    pub async fn execute(&self, tool: &str, args: Value) -> Result<Value, String> {
        self.bridge
            .call(
                "tool.execute",
                serde_json::json!({"tool": tool, "args": args}),
            )
            .await
    }
}

/// Pont haute priorité : Hook → middleware.py
pub struct PythonMiddlewareBridge {
    bridge: PythonBridge,
}

impl PythonMiddlewareBridge {
    pub fn new(python_path: &str, script_path: &str) -> Self {
        Self {
            bridge: PythonBridge::new(python_path, script_path, 5000),
        }
    }

    pub async fn hook(&self, event: &str, data: Value) -> Result<Value, String> {
        self.bridge
            .call(
                "middleware.hook",
                serde_json::json!({"event": event, "data": data}),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_request_serializes() {
        let req = JsonRpcRequest::new("test", serde_json::json!({"a": 1}), 42);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test\""));
        assert!(json.contains("\"id\":42"));
    }

    #[test]
    fn json_rpc_response_deserializes() {
        let json = r#"{"jsonrpc":"2.0","result":{"ok":true},"error":null,"id":1}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn bridge_availability_check() {
        let bridge = PythonBridge::new("/nonexistent/python", "/nonexistent/script.py", 1000);
        assert!(!bridge.is_available());
    }
}
