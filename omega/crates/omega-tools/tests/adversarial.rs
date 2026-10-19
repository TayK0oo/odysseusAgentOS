//! Tests adversariaux — tentatives de contournement des garde-fous `omega-tools`.
//!
//! Ces tests ne vérifient pas le comportement nominal (déjà couvert dans `lib.rs`) :
//! ils *attaquent* les garde-fous. Un test qui passe = la menace est couverte.
//! Un test qui échoue = menace OUVERTE, à corriger avant ship.

use omega_tools::{edit_file, list_dir, read_file, ToolCtx, ToolOutput};
use omega_tools::glob::glob;
use omega_tools::grep::grep;

fn ctx(name: &str) -> ToolCtx {
    let p = std::env::temp_dir().join(format!("omega-adv-{}", name));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    ToolCtx::new(&p)
}

#[test]
fn atk_absolute_path_escape() {
    // Le workspace est /tmp/omega-adv-abs ; on tente d'écrire dans /tmp directement.
    let c = ctx("abs");
    std::fs::write("/tmp/omega-adv-ABSOLUTE-PWNED", "x").ok();
    let out = write_outside(&c, "/tmp/omega-adv-ABSOLUTE-PWNED");
    assert!(
        !out.is_ok(),
        "THREAT OPEN: un chemin absolu hors workspace a été accepté"
    );
    std::fs::remove_file("/tmp/omega-adv-ABSOLUTE-PWNED").ok();
}

fn write_outside(c: &ToolCtx, abs: &str) -> ToolOutput {
    // write_file passe par ToolCtx::resolve -> si l'abs sort de la racine, refus.
    omega_tools::write_file(c, abs, "pwned")
}

#[test]
fn atk_dotdot_escape() {
    let c = ctx("dotdot");
    let out = omega_tools::write_file(&c, "../../escape-pwned.txt", "x");
    assert!(!out.is_ok(), "THREAT OPEN: ../ a permis une écriture hors workspace");
    assert!(
        !std::path::Path::new("/tmp/escape-pwned.txt").exists(),
        "THREAT OPEN: le fichier a été créé hors workspace"
    );
}

#[test]
fn atk_nested_dotdot_escape() {
    let c = ctx("dotdot2");
    std::fs::create_dir_all(c.root.join("sub")).unwrap();
    let out = omega_tools::write_file(&c, "sub/../../escape-pwned2.txt", "x");
    assert!(!out.is_ok(), "THREAT OPEN: sub/../../ a échappé");
    assert!(!std::path::Path::new("/tmp/escape-pwned2.txt").exists());
}

#[test]
fn atk_symlink_escape() {
    // Un symlien dans le workspace pointant vers /tmp : write_file doit-il suivre ?
    let c = ctx("symlink");
    let link = c.root.join("escape-link");
    std::os::unix::fs::symlink("/tmp", &link).unwrap();

    let out = omega_tools::write_file(&c, "escape-link/symlink-pwned.txt", "x");
    // Le resolve() normalise lexicalement : il ne résout PAS les symlinks.
    // Si l'écriture passe, c'est une menace ouverte (TOCTOU / traversal via symlink).
    assert!(
        !out.is_ok(),
        "THREAT OPEN: un symlink vers /tmp a permis l'écriture (résolution lexicale only)"
    );
    assert!(
        !std::path::Path::new("/tmp/symlink-pwned.txt").exists(),
        "THREAT OPEN: écriture effective hors workspace via symlink"
    );
}

#[test]
fn atk_nested_secret_via_parent_dir() {
    // .env à la racine du workspace parent : deny-listmatch-t-elle ?
    let c = ctx("secret");
    let sub = c.root.join("cfg");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join(".env"), "SECRET=1").unwrap();

    let out = read_file(&c, "cfg/.env");
    assert!(!out.is_ok(), "THREAT OPEN: .env imbriqué non protégé");
}

#[test]
fn atk_secret_via_dot_prefix() {
    let c = ctx("secret2");
    std::fs::create_dir_all(c.root.join(".config")).unwrap();
    std::fs::write(c.root.join(".config/id_rsa"), "PRIVATE").unwrap();
    assert!(
        !read_file(&c, ".config/id_rsa").is_ok(),
        "THREAT OPEN: clé privée .config/id_rsa lisible"
    );
}

#[test]
fn atk_glob_follows_symlink_loop() {
    // walk() n'honore pas les symlinks : un lien vers son propre parent
    // doit provoquer une boucle infinie. Timeout du test = menace ouverte.
    let c = ctx("loop");
    std::fs::create_dir_all(c.root.join("d")).unwrap();
    std::os::unix::fs::symlink(c.root.join("d"), c.root.join("d/self")).unwrap();

    let out = glob(&c, "**/*.rs");
    assert!(matches!(out, ToolOutput::Ok { .. }), "glob doit terminer");
    // Note: si ce test hang, walk() suit les symlinks -> threat ouverte à corriger
    // (is_symlink() filter dans glob::walk).
}

#[test]
fn atk_grep_respects_max_results() {
    let c = ctx("grepbound");
    let big = (0..5_000).map(|i| format!("needle line {}\n", i)).collect::<String>();
    std::fs::write(c.root.join("big.txt"), big).unwrap();

    match grep(&c, "needle", None, 10) {
        ToolOutput::Ok { content } => {
            assert!(
                content["count"].as_u64().unwrap() <= 10,
                "THREAT OPEN: grep a dépassé max_results (épuisement mémoire)"
            );
        }
        _ => panic!("grep a échoué"),
    }
}

#[test]
fn atk_glob_does_not_leak_paths_outside_root() {
    let c = ctx("globscope");
    std::fs::create_dir_all(c.root.join("in")).unwrap();
    std::fs::write(c.root.join("in/a.rs"), "x").unwrap();

    match glob(&c, "**/*.rs") {
        ToolOutput::Ok { content } => {
            for m in content["matches"].as_array().unwrap() {
                let p = m.as_str().unwrap();
                assert!(
                    !p.starts_with("..") && !p.starts_with('/'),
                    "THREAT OPEN: glob a retourné un chemin hors workspace : {}",
                    p
                );
            }
        }
        _ => panic!(),
    }
}

#[test]
fn atk_edit_cannot_write_outside() {
    let c = ctx("editesc");
    std::fs::write("/tmp/omega-adv-edit-target.txt", "alpha\nbeta").ok();
    let out = edit_file(&c, "/tmp/omega-adv-edit-target.txt", "beta", "BETA");
    assert!(!out.is_ok(), "THREAT OPEN: edit_file a modifié un fichier hors workspace");
    let content = std::fs::read_to_string("/tmp/omega-adv-edit-target.txt").unwrap_or_default();
    assert!(content.contains("beta") && !content.contains("BETA"), "le fichier cible a été modifié");
    std::fs::remove_file("/tmp/omega-adv-edit-target.txt").ok();
}

#[test]
fn atk_list_dir_cannot_escape() {
    let c = ctx("lsscope");
    std::fs::write("/tmp/omega-adv-ls-secret.txt", "x").ok();
    let out = list_dir(&c, "/tmp");
    assert!(!out.is_ok(), "THREAT OPEN: ls a listé un répertoire hors workspace");
    std::fs::remove_file("/tmp/omega-adv-ls-secret.txt").ok();
}