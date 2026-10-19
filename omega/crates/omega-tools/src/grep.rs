//! `grep` — port de `filesystem_tools.py::GrepTool` (recherche regex récursive).

use super::{normalize, ToolCtx, ToolOutput};
use regex::Regex;
use std::path::Path;

/// Parcourt `dir` et renvoie les lignes correspondant à `re`.
pub fn search(dir: &Path, re: &Regex, max_results: usize) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let mut files = Vec::new();
    super::glob::walk(dir, &mut files);
    for f in files {
        if out.len() >= max_results {
            break;
        }
        let Ok(content) = std::fs::read_to_string(&f) else {
            continue;
        };
        for (i, line) in content.lines().enumerate() {
            if out.len() >= max_results {
                break;
            }
            if re.is_match(line) {
                if let Ok(rel) = f.strip_prefix(dir) {
                    out.push(serde_json::json!({
                        "file": rel.to_string_lossy(),
                        "line": i + 1,
                        "text": line.chars().take(300).collect::<String>(),
                    }));
                }
            }
        }
    }
    out
}

/// Port de `GrepTool.execute`.
pub fn grep(
    ctx: &ToolCtx,
    pattern: &str,
    glob_filter: Option<&str>,
    max_results: usize,
) -> ToolOutput {
    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => return ToolOutput::err(format!("regex invalide : {}", e)),
    };
    let root = normalize(&ctx.root);
    let mut hits = search(&root, &re, max_results);

    if let Some(g) = glob_filter {
        let g = super::glob::glob_to_regex(g.trim_start_matches("./"));
        hits.retain(|h| h["file"].as_str().map(|f| g.is_match(f)).unwrap_or(false));
    }

    ToolOutput::ok(serde_json::json!({
        "pattern": pattern,
        "matches": hits,
        "count": hits.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(name: &str) -> ToolCtx {
        let p = std::env::temp_dir().join(format!("omega-grep-{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        ToolCtx::new(&p)
    }

    #[test]
    fn finds_matching_lines() {
        let c = ctx("basic");
        std::fs::create_dir_all(c.root.join("src")).unwrap();
        std::fs::write(c.root.join("src/a.rs"), "fn main() {}\n// TODO: fix\n").unwrap();
        match grep(&c, "TODO", None, 10) {
            ToolOutput::Ok { content } => {
                assert_eq!(content["count"], 1);
                assert_eq!(content["matches"][0]["line"], 2);
            }
            _ => panic!("grep échoué"),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn respects_max_results() {
        let c = ctx("max");
        std::fs::write(c.root.join("a.txt"), "hit\nhit\nhit\nhit\nhit\n").unwrap();
        match grep(&c, "hit", None, 2) {
            ToolOutput::Ok { content } => assert_eq!(content["count"], 2),
            _ => panic!(),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn glob_filter_narrows_results() {
        let c = ctx("filter");
        std::fs::write(c.root.join("a.rs"), "target_hit").unwrap();
        std::fs::write(c.root.join("b.py"), "target_hit").unwrap();
        match grep(&c, "target_hit", Some("*.rs"), 10) {
            ToolOutput::Ok { content } => assert_eq!(content["count"], 1),
            _ => panic!(),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn invalid_regex_errors() {
        let c = ctx("bad");
        assert!(!grep(&c, "[unclosed", None, 5).is_ok());
        let _ = std::fs::remove_dir_all(&c.root);
    }
}
