//! `glob` — port de `filesystem_tools.py::_glob_to_regex` + `GlobTool`.
//!
//! `**/` → zéro ou plus de répertoires ; `*` → dans un segment ; `?` → 1 char.

use super::{normalize, ToolCtx, ToolOutput};
use regex::Regex;
use std::path::Path;

/// Compile un motif glob (slash-forward) en regex.
pub fn glob_to_regex(pat: &str) -> Regex {
    let chars: Vec<char> = pat.chars().collect();
    let (mut i, n) = (0usize, chars.len());
    let mut out = String::new();
    while i < n {
        if chars[i] == '*' && i + 2 < n && chars[i + 1] == '*' && chars[i + 2] == '/' {
            out.push_str("(?:[^/]+/)*");
            i += 3;
        } else if chars[i] == '*' && i + 1 < n && chars[i + 1] == '*' {
            out.push_str(".*");
            i += 2;
        } else if chars[i] == '*' {
            out.push_str("[^/]*");
            i += 1;
        } else if chars[i] == '?' {
            out.push_str("[^/]");
            i += 1;
        } else if chars[i] == '.' || chars[i] == '+' || chars[i] == '(' || chars[i] == ')' {
            out.push('\\');
            out.push(chars[i]);
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    Regex::new(&format!("^{}$", out)).expect("glob regex valide")
}

/// Liste récursive des fichiers sous `dir` (saute les répertoires cachés).
pub fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// Port de `GlobTool.execute`.
pub fn glob(ctx: &ToolCtx, pattern: &str) -> ToolOutput {
    let root = normalize(&ctx.root);
    let mut files = Vec::new();
    walk(&root, &mut files);

    let re = glob_to_regex(pattern.trim_start_matches("./"));

    let mut matches: Vec<String> = files
        .iter()
        .filter_map(|p| p.strip_prefix(&root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .filter(|rel| re.is_match(rel))
        .collect();
    matches.sort();

    ToolOutput::ok(serde_json::json!({
        "pattern": pattern,
        "matches": matches,
        "count": matches.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_stays_in_segment() {
        let re = glob_to_regex("*.rs");
        assert!(re.is_match("main.rs"));
        assert!(!re.is_match("src/main.rs"));
    }

    #[test]
    fn double_star_crosses_directories() {
        let re = glob_to_regex("**/*.rs");
        assert!(re.is_match("main.rs"));
        assert!(re.is_match("src/deep/nested/main.rs"));
    }

    #[test]
    fn question_mark_single_char() {
        let re = glob_to_regex("file?.txt");
        assert!(re.is_match("file1.txt"));
        assert!(!re.is_match("file12.txt"));
    }

    #[test]
    fn glob_finds_files_on_disk() {
        let c = super::ToolCtx::new(std::env::temp_dir().join("omega-tools-glob"));
        let _ = std::fs::remove_dir_all(&c.root);
        std::fs::create_dir_all(c.root.join("src")).unwrap();
        std::fs::write(c.root.join("src/a.rs"), "x").unwrap();
        std::fs::write(c.root.join("src/b.py"), "x").unwrap();
        match glob(&c, "**/*.rs") {
            super::ToolOutput::Ok { content } => {
                assert_eq!(content["count"], 1);
                assert_eq!(content["matches"][0], "src/a.rs");
            }
            _ => panic!("glob échoué"),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }
}
