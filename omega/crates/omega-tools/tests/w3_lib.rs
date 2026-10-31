//! Tests W3 — mutants lib.rs (resolve, read_file, list_dir).
//!
//! Chaque test passe sur l'original et échoue sur le mutant correspondant.

use omega_tools::{list_dir, read_file, ToolCtx};
use std::time::{SystemTime, UNIX_EPOCH};

fn uniq() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn ctx(name: &str) -> ToolCtx {
    let p = std::env::temp_dir().join(format!(
        "omega-w3-{}-{}-{}",
        std::process::id(),
        uniq(),
        name
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    ToolCtx::new(&p)
}

fn ctx_resolve_ok(c: &ToolCtx, rel: &str) -> bool {
    c.resolve(rel).is_ok()
}

fn ctx_resolve_err(c: &ToolCtx, rel: &str) -> bool {
    c.resolve(rel).is_err()
}

#[test]
fn resolve_accepts_path_of_exact_max_length() {
    let c = ctx("len");
    let long = "a".repeat(omega_tools::MAX_PATH_CHARS);
    assert!(ctx_resolve_ok(&c, &long));
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn resolve_denies_nested_deny_component() {
    let c = ctx("deny");
    std::fs::create_dir_all(c.root.join(".docker")).unwrap();
    std::fs::write(c.root.join(".docker/config.json"), "secret").unwrap();
    assert!(ctx_resolve_err(&c, ".docker/config.json"));
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn read_file_at_max_minus_one_ok() {
    let c = ctx("rmin");
    let n = omega_tools::MAX_READ_BYTES - 1;
    std::fs::write(c.root.join("f.bin"), vec![b'x'; n]).unwrap();
    assert!(read_file(&c, "f.bin").is_ok());
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn read_file_at_max_ok() {
    let c = ctx("rmax");
    let n = omega_tools::MAX_READ_BYTES;
    std::fs::write(c.root.join("f.bin"), vec![b'x'; n]).unwrap();
    assert!(read_file(&c, "f.bin").is_ok());
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn read_file_at_max_plus_one_err() {
    let c = ctx("rmaxp");
    let n = omega_tools::MAX_READ_BYTES + 1;
    std::fs::write(c.root.join("f.bin"), vec![b'x'; n]).unwrap();
    assert!(!read_file(&c, "f.bin").is_ok());
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn list_dir_empty_root_named_credentials_ok() {
    let base = std::env::temp_dir().join(format!("omega-w3-{}-{}-ls", std::process::id(), uniq()));
    let _ = std::fs::remove_dir_all(&base);
    let root = base.join("credentials");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("afile.txt"), "x").unwrap();
    let c = ToolCtx::new(&root);
    let out = list_dir(&c, "");
    assert!(out.is_ok());
    let _ = std::fs::remove_dir_all(&base);
}
