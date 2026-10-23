//! `surface` — `FsSurface` : l'implémentation `ToolSurface` du catalogue
//! filesystem (dépassement `dispatch` sur le port `omega-ports`).
//!
//! Contrat du port : toute observation est structurée — `ToolOutput::Error`
//! devient `ok=false` + `{"error": ...}`, jamais de panic.

use crate::registry::dispatch;

use crate::ToolCtx;

/// Surface d'outils filesystem : racine du workspace + deny-list via [`ToolCtx`].
pub struct FsSurface {
    pub ctx: ToolCtx,
}

impl FsSurface {
    pub fn new(ctx: ToolCtx) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl omega_ports::ToolSurface for FsSurface {
    async fn execute(
        &self,
        call: &omega_ports::ToolCall,
    ) -> omega_types::OmegaResult<omega_ports::ToolObservation> {
        let out = dispatch(&self.ctx, &call.name, &call.arguments);
        let obs = match out {
            crate::ToolOutput::Ok { content } => {
                omega_ports::ToolObservation::ok(call.call_id.clone(), content)
            }
            crate::ToolOutput::Error { message } => {
                omega_ports::ToolObservation::error(call.call_id.clone(), message)
            }
        };
        Ok(obs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_ports::{ToolCall, ToolCallOrigin, ToolSurface};

    fn call(name: &str, args: serde_json::Value) -> ToolCall {
        ToolCall {
            call_id: "call-test".to_string(),
            name: name.to_string(),
            arguments: args,
            origin: ToolCallOrigin::ReAct,
        }
    }

    fn ctx(name: &str) -> ToolCtx {
        let p = std::env::temp_dir().join(format!("omega-surface-{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        ToolCtx::new(&p)
    }

    fn surface(name: &str) -> FsSurface {
        FsSurface::new(ctx(name))
    }

    #[tokio::test]
    async fn write_then_read_canary() {
        // (T1 puis T0) l'écriture canari est precedee de la jail, la lecture
        // revoit exactement le contenu ecrit.
        let s = surface("canary");
        let w = s
            .execute(&call(
                "write_file",
                serde_json::json!({"path": "canari.txt", "content": "code canari"}),
            ))
            .await
            .unwrap();
        assert!(w.ok, "écriture : {:?}", w.content);
        assert_eq!(w.content["bytes"], 11);

        let r = s
            .execute(&call(
                "read_file",
                serde_json::json!({"path": "canari.txt"}),
            ))
            .await
            .unwrap();
        assert!(r.ok);
        assert_eq!(r.content["content"], "code canari");
        let _ = std::fs::remove_dir_all(&s.ctx.root);
    }

    #[tokio::test]
    async fn escape_is_structured_error_not_panic() {
        // (Cas limite plan §6) sortie de la jail : ok=false structuré.
        let s = surface("escape");
        let o = s
            .execute(&call(
                "read_file",
                serde_json::json!({"path": "../../etc/passwd"}),
            ))
            .await
            .unwrap();
        assert!(!o.ok);
        assert!(o.content["error"].is_string());
        let _ = std::fs::remove_dir_all(&s.ctx.root);
    }

    #[test]
    fn tool_surface_contains_memory_tools() {
        // La memoire est un outil de premiere classe de la surface (pas un
        // port interne) : elle doit figurer au catalogue avec son schema.
        for name in ["memory_store", "memory_recall"] {
            assert!(
                crate::registry::TOOL_NAMES.contains(&name),
                "{} absent du catalogue",
                name
            );
            assert!(crate::registry::spec(name).is_ok(), "{} sans schema", name);
        }
    }

    #[tokio::test]
    async fn memory_store_recall() {
        let s = surface("memory");
        let w = s
            .execute(&call(
                "memory_store",
                serde_json::json!({"content": "le canari est vert", "provenance": "test:surface"}),
            ))
            .await
            .unwrap();
        assert!(w.ok, "store : {:?}", w.content);

        let r = s
            .execute(&call(
                "memory_recall",
                serde_json::json!({"query": "canari"}),
            ))
            .await
            .unwrap();
        assert!(r.ok);
        let hits = r.content["results"].as_array().unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["content"], "le canari est vert");
        assert_eq!(hits[0]["provenance"], "test:surface");

        // Persistance T1 : le fait est ecrit dans le workspace (JSONL append).
        let raw = std::fs::read_to_string(s.ctx.root.join(".omega/memory.jsonl")).unwrap();
        assert_eq!(raw.lines().count(), 1);
        let _ = std::fs::remove_dir_all(&s.ctx.root);
    }

    #[tokio::test]
    async fn unknown_tool_is_structured_error() {
        let s = surface("unknown");
        let o = s
            .execute(&call("rm_rf", serde_json::json!({})))
            .await
            .unwrap();
        assert!(!o.ok);
        assert_eq!(o.content["error"], "outil inconnu : rm_rf");
        let _ = std::fs::remove_dir_all(&s.ctx.root);
    }
}
