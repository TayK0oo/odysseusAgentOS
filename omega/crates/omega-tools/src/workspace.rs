//! `workspace` — port de `GetWorkspaceTool` (contexte projet envoyé au modèle).

use super::{ToolCtx, ToolOutput};
use std::collections::BTreeMap;

/// Statistiques d'un workspace (fichiers, lignes, langages).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct WorkspaceInfo {
    pub root: String,
    pub file_count: usize,
    pub total_lines: usize,
    pub by_extension: BTreeMap<String, usize>,
    pub largest_files: Vec<(String, usize)>,
}

/// Analyse le workspace (port simplifié de `GetWorkspaceTool`).
pub fn describe(ctx: &ToolCtx) -> ToolOutput {
    let root = super::normalize(&ctx.root);
    let mut files = Vec::new();
    super::glob::walk(&root, &mut files);

    let mut by_ext: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_lines = 0usize;
    let mut largest: Vec<(String, usize)> = Vec::new();

    for f in &files {
        let ext = f
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "(aucune)".into());
        *by_ext.entry(ext).or_insert(0) += 1;
        if let Ok(c) = std::fs::read_to_string(f) {
            let lines = c.lines().count();
            total_lines += lines;
            if let Ok(rel) = f.strip_prefix(&root) {
                largest.push((rel.to_string_lossy().to_string(), lines));
            }
        }
    }
    largest.sort_by_key(|p| std::cmp::Reverse(p.1));
    largest.truncate(10);

    ToolOutput::ok(
        serde_json::to_value(WorkspaceInfo {
            root: root.to_string_lossy().to_string(),
            file_count: files.len(),
            total_lines,
            by_extension: by_ext,
            largest_files: largest,
        })
        .unwrap_or(serde_json::Value::Null),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_workspace() {
        let p = std::env::temp_dir().join("omega-ws-test");
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::write(p.join("src/a.rs"), "a\nb\nc\n").unwrap();
        std::fs::write(p.join("src/b.py"), "x\n").unwrap();
        let ctx = ToolCtx::new(&p);

        match describe(&ctx) {
            ToolOutput::Ok { content } => {
                assert_eq!(content["file_count"], 2);
                assert_eq!(content["total_lines"], 4);
                assert_eq!(content["by_extension"]["rs"], 1);
                assert_eq!(content["largest_files"][0][1], 3);
            }
            _ => panic!("describe échoué"),
        }
        let _ = std::fs::remove_dir_all(&p);
    }
}
