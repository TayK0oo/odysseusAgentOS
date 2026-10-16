//! Serveur **MCP stub** (JSON-RPC 2.0 sur stdio) — uniquement pour les tests.
//!
//! Il expose volontairement un catalogue minuscule et déterministe :
//! - `echo` : renvoie `{"echoed": <text>}` ;
//! - `sum` : renvoie la somme des entiers de `numbers` ;
//! - `boom` : renvoie une erreur JSON-RPC (pour tester le mapping d'erreur) ;
//! - toute méthode inconnue : erreur `-32601`.
//!
//! Protocole : une requête JSON par ligne sur stdin, une réponse par ligne sur stdout.
//! Utilisé par `cargo test` via `env!("CARGO_BIN_EXE_mcp_stub")`.

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        // Notification (pas d'`id`) : rien à répondre.
        let id = match req.get("id") {
            Some(id) => id.clone(),
            None => continue,
        };
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(Value::Null);

        let response = match method {
            "initialize" => ok(
                id,
                json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": { "name": "mcp-stub", "version": "0.1.0" }
                }),
            ),
            "tools/list" => ok(
                id,
                json!({
                    "tools": [
                        {
                            "name": "echo",
                            "description": "Renvoie le texte fourni.",
                            "inputSchema": {
                                "type": "object",
                                "properties": { "text": { "type": "string" } },
                                "required": ["text"]
                            }
                        },
                        {
                            "name": "sum",
                            "description": "Somme une liste d'entiers.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "numbers": { "type": "array", "items": { "type": "integer" } }
                                },
                                "required": ["numbers"]
                            }
                        },
                        {
                            "name": "boom",
                            "description": "Echoue toujours (test d'erreur).",
                            "inputSchema": { "type": "object", "properties": {} }
                        }
                    ]
                }),
            ),
            "tools/call" => call(&id, &params),
            other => err(id, -32601, &format!("methode inconnue : {other}")),
        };

        writeln!(out, "{}", serde_json::to_string(&response).unwrap())?;
        out.flush()?;
    }
    Ok(())
}

fn call(id: &Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match name {
        "echo" => {
            let text = args.get("text").and_then(Value::as_str).unwrap_or("");
            ok(
                id.clone(),
                json!({
                    "content": [{ "type": "text", "text": text }],
                    "structuredContent": { "echoed": text },
                    "isError": false
                }),
            )
        }
        "sum" => {
            let total: i64 = args
                .get("numbers")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_i64).sum())
                .unwrap_or(0);
            ok(
                id.clone(),
                json!({
                    "content": [{ "type": "text", "text": total.to_string() }],
                    "structuredContent": { "sum": total },
                    "isError": false
                }),
            )
        }
        "boom" => ok(
            id.clone(),
            json!({
                "content": [{ "type": "text", "text": "outil en echec volontaire" }],
                "isError": true
            }),
        ),
        other => err(id.clone(), -32602, &format!("outil inconnu : {other}")),
    }
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
