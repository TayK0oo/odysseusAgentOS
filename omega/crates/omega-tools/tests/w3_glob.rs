//! W3 — tests exacts sur `glob_to_regex` et `walk` (mutants glob.rs).

use omega_tools::glob::{glob_to_regex, walk};
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("w3-glob-{}", name));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn empty_pattern_matches_only_empty() {
    let re = glob_to_regex("");
    assert!(re.is_match(""));
    assert!(!re.is_match("a"));
}

#[test]
fn star_rests_in_segment() {
    let re = glob_to_regex("*.rs");
    assert!(re.is_match("main.rs"));
    assert!(!re.is_match("src/main.rs"));
}

#[test]
fn double_star_crosses_directories() {
    let re = glob_to_regex("**/*.rs");
    assert!(re.is_match("main.rs"));
    assert!(re.is_match("src/deep/nested/main.rs"));
    assert!(!re.is_match("main.py"));
}

#[test]
fn double_star_at_end_crosses() {
    let re = glob_to_regex("**");
    assert!(re.is_match("a/b/c"));
    assert!(re.is_match(""));
}

#[test]
fn double_star_slash_prefix_boundary() {
    let re = glob_to_regex("**/");
    assert!(re.is_match(""));
    assert!(re.is_match("src/"));
    assert!(!re.is_match("src"));
}

#[test]
fn question_mark_single_char() {
    let re = glob_to_regex("file?.txt");
    assert!(re.is_match("file1.txt"));
    assert!(!re.is_match("file12.txt"));
    assert!(!re.is_match("file.txt"));
}

#[test]
fn star_single_before_slash_not_double() {
    let re = glob_to_regex("*x/");
    assert!(re.is_match("abcx/"));
    assert!(!re.is_match("abc/"));
}

#[test]
fn star_alone_boundary() {
    let re = glob_to_regex("*");
    assert!(re.is_match("abc"));
    assert!(!re.is_match("a/b"));
}

#[test]
fn regex_special_chars_escaped() {
    let re = glob_to_regex("a.b+c(d)e");
    assert!(re.is_match("a.b+c(d)e"));
    assert!(!re.is_match("axbxcxdxe"));
}

#[test]
fn asymmetric_segments() {
    let re = glob_to_regex("a/*/b");
    assert!(re.is_match("a/x/b"));
    assert!(!re.is_match("a/x/y/b"));

    let re = glob_to_regex("a/**/b");
    assert!(re.is_match("a/b"));
    assert!(re.is_match("a/x/y/b"));
    assert!(!re.is_match("a/x/y/c"));
}

#[test]
fn walk_skips_hidden() {
    let root = tmp("hidden");
    std::fs::create_dir_all(root.join(".secret")).unwrap();
    std::fs::write(root.join(".secret/x.rs"), "x").unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/a.rs"), "x").unwrap();

    let mut out = Vec::new();
    walk(&root, &mut out);
    let names: Vec<String> = out
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(names.iter().any(|n| n.ends_with("src/a.rs")));
    assert!(!names.iter().any(|n| n.contains(".secret")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn walk_skips_target() {
    let root = tmp("target");
    std::fs::create_dir_all(root.join("target")).unwrap();
    std::fs::write(root.join("target/x.rs"), "x").unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/a.rs"), "x").unwrap();

    let mut out = Vec::new();
    walk(&root, &mut out);
    let names: Vec<String> = out
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(names.iter().any(|n| n.ends_with("src/a.rs")));
    assert!(!names.iter().any(|n| n.contains("/target/")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn walk_skips_node_modules() {
    let root = tmp("nm");
    std::fs::create_dir_all(root.join("node_modules")).unwrap();
    std::fs::write(root.join("node_modules/x.rs"), "x").unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/a.rs"), "x").unwrap();

    let mut out = Vec::new();
    walk(&root, &mut out);
    let names: Vec<String> = out
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(names.iter().any(|n| n.ends_with("src/a.rs")));
    assert!(!names.iter().any(|n| n.contains("node_modules")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn walk_symlink_loop_terminates() {
    let root = tmp("symlink");
    std::fs::create_dir_all(root.join("d")).unwrap();
    std::os::unix::fs::symlink(root.join("d"), root.join("d/self")).unwrap();

    let mut out = Vec::new();
    walk(&root, &mut out);
    // Le test réussit s'il se termine (pas de boucle infinie).
    let _ = std::fs::remove_dir_all(&root);
}
