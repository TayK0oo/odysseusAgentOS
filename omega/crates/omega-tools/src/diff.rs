//! `diff` — port de `filesystem_tools.py::_unified_diff` (difflib.unified_diff).

/// Diff unifié simplifié, borné à [`crate::MAX_DIFF_LINES`] lignes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UnifiedDiff {
    pub text: String,
    pub added: usize,
    pub removed: usize,
    pub new_file: bool,
    pub file: String,
}

/// Calcule le diff unifié ligne à ligne (LCS simplifié : détection par
/// position pour les lignes modifiées, ajout/suppression détectés par
/// recherche de la ligne la plus proche).
pub fn unified_diff(old: &str, new: &str, path: &str) -> Option<UnifiedDiff> {
    if old == new {
        return None;
    }
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // LCS via programme dynamique (Tailles modestes : documents / fichiers source).
    let (a, b) = (old_lines.len(), new_lines.len());
    let mut dp = vec![vec![0usize; b + 1]; a + 1];
    for i in (0..a).rev() {
        for j in (0..b).rev() {
            dp[i][j] = if old_lines[i] == new_lines[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let label = if path.is_empty() { "file" } else { path };
    let mut out: Vec<String> = vec![format!("--- a/{}", label), format!("+++ b/{}", label)];
    let mut added = 0usize;
    let mut removed = 0usize;

    let (mut i, mut j) = (0usize, 0usize);
    while i < a && j < b {
        if old_lines[i] == new_lines[j] {
            out.push(format!(" {}", old_lines[i]));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            out.push(format!("-{}", old_lines[i]));
            removed += 1;
            i += 1;
        } else {
            out.push(format!("+{}", new_lines[j]));
            added += 1;
            j += 1;
        }
    }
    while i < a {
        out.push(format!("-{}", old_lines[i]));
        removed += 1;
        i += 1;
    }
    while j < b {
        out.push(format!("+{}", new_lines[j]));
        added += 1;
        j += 1;
    }

    let truncated = out.len() > crate::MAX_DIFF_LINES;
    if truncated {
        out.truncate(crate::MAX_DIFF_LINES);
    }
    let mut text = out.join("\n");
    if truncated {
        text.push_str(&format!(
            "\n… diff truncated at {} lines",
            crate::MAX_DIFF_LINES
        ));
    }

    Some(UnifiedDiff {
        text,
        added,
        removed,
        new_file: old.is_empty(),
        file: std::path::Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "file".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change_is_none() {
        assert!(unified_diff("a\nb", "a\nb", "f.txt").is_none());
    }

    #[test]
    fn new_file_detected() {
        let d = unified_diff("", "hello", "f.txt").unwrap();
        assert!(d.new_file);
        assert_eq!(d.added, 1);
        assert_eq!(d.removed, 0);
        assert_eq!(d.file, "f.txt");
    }

    #[test]
    fn single_line_modification() {
        let d = unified_diff("alpha\nbeta\ngamma", "alpha\nBETA\ngamma", "f.txt").unwrap();
        assert_eq!(d.added, 1);
        assert_eq!(d.removed, 1);
        assert!(d.text.contains("-beta"));
        assert!(d.text.contains("+BETA"));
        assert!(d.text.contains(" alpha"));
    }

    #[test]
    fn addition_only() {
        let d = unified_diff("a", "a\nb\nc", "x.txt").unwrap();
        assert_eq!(d.added, 2);
        assert_eq!(d.removed, 0);
    }

    #[test]
    fn deletion_only() {
        let d = unified_diff("a\nb\nc", "a", "x.txt").unwrap();
        assert_eq!(d.added, 0);
        assert_eq!(d.removed, 2);
    }

    #[test]
    fn truncation_is_annotated() {
        let old: String = (0..300).map(|i| format!("line {}\n", i)).collect();
        let new: String = (0..300).map(|i| format!("LINE {}\n", i)).collect();
        let d = unified_diff(&old, &new, "big.txt").unwrap();
        assert!(d.text.contains("diff truncated"));
    }
}
