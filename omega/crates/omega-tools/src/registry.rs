//! `registry` — catalogue des outils filesystem (schémas + dispatch).
//!
//! Port de `src/tool_schemas.py::FUNCTION_TOOL_SCHEMAS` + `tool_execution.py`
//! pour la sous-famille filesystem.

use super::{
    append_file, edit_file, list_dir, read_file, workspace, write_file, ToolCtx, ToolOutput,
};
use omega_types::OmegaResult;
use omega_types::ToolSpec;

/// Nom des outils exposés (identiques à `TOOL_TAGS` d'odysseus).
pub const TOOL_NAMES: [&str; 8] = [
    "read_file",
    "write_file",
    "append_file",
    "edit_file",
    "ls",
    "glob",
    "grep",
    "get_workspace",
];

/// Schémas JSON des outils filesystem.
pub fn schemas() -> Vec<serde_json::Value> {
    vec![
        schema(
            "read_file",
            "Lit un fichier texte du workspace.",
            vec!["path"],
            vec!["path"],
        ),
        schema(
            "write_file",
            "Écrit (ou crée) un fichier texte.",
            vec!["path", "content"],
            vec!["path", "content"],
        ),
        schema(
            "append_file",
            "Ajoute une ligne (canary) a un fichier texte (append, creation si absent).",
            vec!["path"],
            vec!["path", "content"],
        ),
        schema(
            "edit_file",
            "Remplace une chaîne exacte dans un fichier.",
            vec!["path", "old_string", "new_string"],
            vec!["path", "old_string", "new_string"],
        ),
        schema(
            "ls",
            "Liste le contenu d'un répertoire.",
            vec!["path"],
            vec!["path"],
        ),
        schema(
            "glob",
            "Trouve des fichiers par motif glob.",
            vec!["pattern"],
            vec!["pattern"],
        ),
        schema(
            "grep",
            "Recherche une expression régulière dans le workspace.",
            vec!["pattern"],
            vec!["pattern"],
        ),
        schema(
            "get_workspace",
            "Décrit le workspace (fichiers, lignes, langages).",
            vec![],
            vec![],
        ),
    ]
}

fn schema(name: &str, desc: &str, required: Vec<&str>, props: Vec<&str>) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    for p in &props {
        let ty = match *p {
            "content" | "new_string" | "old_string" => "string",
            "path" => "string",
            "pattern" => "string",
            _ => "string",
        };
        properties.insert((*p).to_string(), serde_json::json!({ "type": ty }));
    }
    serde_json::json!({
        "type": "function",
        "function": {
            "name": name,
            "description": desc,
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": required,
            }
        }
    })
}

/// Dispatche un appel d'outil filesystem.
pub fn dispatch(ctx: &ToolCtx, tool: &str, args: &serde_json::Value) -> ToolOutput {
    let s = |k: &str| args.get(k).and_then(|v| v.as_str()).unwrap_or("");
    match tool {
        "read_file" => read_file(ctx, s("path")),
        "write_file" => write_file(ctx, s("path"), s("content")),
        "append_file" => append_file(ctx, s("path"), s("content")),
        "edit_file" => edit_file(ctx, s("path"), s("old_string"), s("new_string")),
        "ls" => list_dir(ctx, s("path")),
        "glob" => super::glob::glob(ctx, s("pattern")),
        "grep" => {
            let g = args.get("glob").and_then(|v| v.as_str());
            let max = args
                .get("max_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as usize;
            super::grep::grep(ctx, s("pattern"), g, max)
        }
        "get_workspace" => workspace::describe(ctx),
        other => ToolOutput::err(format!("outil inconnu : {}", other)),
    }
}

/// `ToolSpec` (port `omega-ports`) d'un outil du catalogue.
pub fn spec(tool: &str) -> OmegaResult<ToolSpec> {
    schemas()
        .into_iter()
        .find(|s| s["function"]["name"] == tool)
        .map(|s| ToolSpec {
            name: tool.to_string(),
            description: s["function"]["description"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            schema: s["function"]["parameters"].clone(),
        })
        .ok_or_else(|| omega_types::OmegaError::Io(format!("outil inconnu : {}", tool)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(name: &str) -> ToolCtx {
        let p = std::env::temp_dir().join(format!("omega-reg-{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        ToolCtx::new(&p)
    }

    #[test]
    fn seven_tools_declared() {
        assert_eq!(TOOL_NAMES.len(), 8);
        assert_eq!(schemas().len(), 8);
    }

    #[test]
    fn dispatch_write_read_cycle() {
        let c = ctx("cycle");
        let w = dispatch(
            &c,
            "write_file",
            &serde_json::json!({"path":"a.txt","content":"x"}),
        );
        assert!(w.is_ok());
        let r = dispatch(&c, "read_file", &serde_json::json!({"path":"a.txt"}));
        match r {
            ToolOutput::Ok { content } => assert_eq!(content["content"], "x"),
            _ => panic!(),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn unknown_tool_errors() {
        let c = ctx("unknown");
        assert!(!dispatch(&c, "rm_rf", &serde_json::json!({})).is_ok());
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn spec_is_resolvable() {
        let s = spec("grep").unwrap();
        assert_eq!(s.name, "grep");
        assert!(spec("nonexistent").is_err());
    }

    #[test]
    fn edit_through_dispatch_produces_diff() {
        let c = ctx("editdiff");
        dispatch(
            &c,
            "write_file",
            &serde_json::json!({"path":"f","content":"a\nb"}),
        );
        let r = dispatch(
            &c,
            "edit_file",
            &serde_json::json!({"path":"f","old_string":"b","new_string":"c"}),
        );
        match r {
            ToolOutput::Ok { content } => {
                assert_eq!(content["diff"]["added"], 1);
                assert_eq!(content["diff"]["removed"], 1);
            }
            _ => panic!(),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }
}
