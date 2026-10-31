//! W3 — tests exacts sur `unified_diff` (mutants diff.rs).

use omega_tools::diff::unified_diff;

#[test]
fn aucun_changement_est_none() {
    assert!(unified_diff("a\nb", "a\nb", "f.txt").is_none());
}

#[test]
fn nouveau_fichier_indices_et_comptes() {
    let d = unified_diff("", "hello", "f.txt").unwrap();
    assert!(d.new_file);
    assert_eq!(d.added, 1);
    assert_eq!(d.removed, 0);
    assert_eq!(d.text.lines().count(), 3);
}

#[test]
fn modification_simple_comptes_et_texte() {
    let d = unified_diff("alpha\nbeta\ngamma", "alpha\nBETA\ngamma", "f.txt").unwrap();
    assert_eq!(d.added, 1);
    assert_eq!(d.removed, 1);
    assert!(d.text.contains("-beta"));
    assert!(d.text.contains("+BETA"));
    assert!(d.text.contains(" alpha"));
}

#[test]
fn asymetrie_ancien_plus_grand() {
    let d = unified_diff("x\na\nb\nc\ny", "x\ny", "a.txt").unwrap();
    assert_eq!(d.added, 0);
    assert_eq!(d.removed, 3);
    assert_eq!(d.text.lines().count(), 7);
}

#[test]
fn asymetrie_nouveau_plus_grand() {
    let d = unified_diff("x\ny", "x\na\nb\nc\ny", "a.txt").unwrap();
    assert_eq!(d.added, 3);
    assert_eq!(d.removed, 0);
    assert_eq!(d.text.lines().count(), 7);
}

#[test]
fn lcs_reordonne_comptes_exacts() {
    let d = unified_diff("a\nb\nc\nd\ne", "a\nc\nb\nd\ne", "r.txt").unwrap();
    assert_eq!(d.added, 1);
    assert_eq!(d.removed, 1);
}

#[test]
fn truncation_exactement_max_non_tronque() {
    // 398 lignes supprimées + 2 en-têtes = 400 = MAX_DIFF_LINES : pas de troncature.
    let old: String = (0..398).map(|i| format!("line {}\n", i)).collect();
    let d = unified_diff(&old, "", "big.txt").unwrap();
    assert_eq!(d.removed, 398);
    assert_eq!(d.text.lines().count(), 400);
    assert!(!d.text.contains("diff truncated"));
}

#[test]
fn truncation_au_dela_de_max_annotee() {
    let old: String = (0..399).map(|i| format!("line {}\n", i)).collect();
    let d = unified_diff(&old, "", "big.txt").unwrap();
    assert!(d.text.contains("diff truncated"));
    // 400 premières lignes conservées + annotation
    assert_eq!(d.text.lines().count(), 401);
}

#[test]
fn nom_fichier_tempdir_unique() {
    let dir = std::env::temp_dir().join(format!("w3-diff-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("sample.txt");
    let d = unified_diff("a", "b", path.to_str().unwrap()).unwrap();
    assert_eq!(d.file, "sample.txt");
    let _ = std::fs::remove_dir_all(&dir);
}

use proptest::prelude::*;

fn oracle(old: &str, new: &str, path: &str) -> Option<(String, usize, usize, bool, String)> {
    if old == new {
        return None;
    }
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
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
    let truncated = out.len() > omega_tools::MAX_DIFF_LINES;
    if truncated {
        out.truncate(omega_tools::MAX_DIFF_LINES);
    }
    let mut text = out.join("\n");
    if truncated {
        text.push_str(&format!(
            "\n… diff truncated at {}",
            omega_tools::MAX_DIFF_LINES
        ));
    }
    let file = std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "file".into());
    Some((text, added, removed, old.is_empty(), file))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4096))]
    #[test]
    fn oracle_diff_sur_petites_entrees(
        old in prop::collection::vec("[abc]", 0..8),
        new in prop::collection::vec("[abc]", 0..8),
    ) {
        let old = old.join("\n");
        let new = new.join("\n");
        if old == new { return Ok(()); }
        let d = omega_tools::diff::unified_diff(&old, &new, "t.txt").unwrap();
        let (ot, oa, or_, _, _) = oracle(&old, &new, "t.txt").unwrap();
        prop_assert_eq!(d.text, ot);
        prop_assert_eq!(d.added, oa);
        prop_assert_eq!(d.removed, or_);
    }
}
