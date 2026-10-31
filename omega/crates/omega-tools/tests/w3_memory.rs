//! Tests W3 — `omega-tools::memory` (store / recall / forget).
//!
//! Un tempdir unique par test (chemin dérivé d'un compteur atomique + PID) pour
//! éviter toute collision inter-tests. Assertions sur la valeur exacte du retour
//! et sur le contenu du fichier. 200 créations concurrentes pour exercer
//! `unique_tmp_id`. Cas fichier manquant = mémoire vide (pas une erreur).

use omega_tools::memory::{forget, memory_forget, recall, store, MEMORY_PATH, RECALL_LIMIT};
use omega_tools::{ToolCtx, ToolOutput};
use std::sync::atomic::{AtomicU64, Ordering};

static CTR: AtomicU64 = AtomicU64::new(0);

fn ctx(tag: &str) -> ToolCtx {
    let n = CTR.fetch_add(1, Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("omega-w3mem-{}-{}-{}", std::process::id(), n, tag));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    ToolCtx::new(&p)
}

fn ok_content(out: ToolOutput) -> serde_json::Value {
    match out {
        ToolOutput::Ok { content } => content,
        ToolOutput::Error { message } => panic!("attendu Ok, reçu Error: {}", message),
    }
}

#[test]
fn store_returns_exact_value_and_writes_exact_line() {
    let c = ctx("store-exact");
    let out = store(&c, "le canari est vert", "prov-src");
    let content = ok_content(out);
    assert_eq!(content["path"], MEMORY_PATH);
    assert_eq!(content["provenance"], "prov-src");
    let line = format!(
        "{}\n",
        serde_json::json!({"content":"le canari est vert","provenance":"prov-src"})
    );
    assert_eq!(content["bytes"], line.len());
    let file = std::fs::read_to_string(c.root.join(MEMORY_PATH)).unwrap();
    assert_eq!(file, line);
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn store_refuses_empty() {
    let c = ctx("store-empty");
    assert!(!store(&c, "   ", "p").is_ok());
    assert!(!store(&c, "fait", "").is_ok());
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn recall_exact_filter_and_count() {
    let c = ctx("recall-exact");
    assert!(store(&c, "le canari est vert", "t").is_ok());
    assert!(store(&c, "le chat dort", "t").is_ok());
    let content = ok_content(recall(&c, "canari"));
    assert_eq!(content["query"], "canari");
    assert_eq!(content["count"], 1);
    let hits = content["results"].as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["content"], "le canari est vert");
    assert_eq!(hits[0]["provenance"], "t");
    // empty query => empty results
    let content = ok_content(recall(&c, "   "));
    assert_eq!(content["count"], 0);
    assert_eq!(content["results"].as_array().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn recall_bounded_by_recall_limit() {
    let c = ctx("recall-bound");
    for i in 0..(RECALL_LIMIT + 5) {
        assert!(store(&c, &format!("fait-{}", i), "t").is_ok());
    }
    let content = ok_content(recall(&c, "fait-"));
    assert_eq!(content["results"].as_array().unwrap().len(), RECALL_LIMIT);
    assert_eq!(content["count"], RECALL_LIMIT);
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn recall_on_missing_file_is_empty() {
    let c = ctx("recall-missing");
    // fichier jamais écrit : zéro résultat, pas une erreur
    let content = ok_content(recall(&c, "x"));
    assert_eq!(content["count"], 0);
    assert_eq!(content["results"].as_array().unwrap().len(), 0);
    // ligne corrompue ignorée
    std::fs::create_dir_all(c.root.join(".omega")).unwrap();
    std::fs::write(
        c.root.join(MEMORY_PATH),
        "pas du json\n{\"content\":\"fait lisible\",\"provenance\":\"t\"}\n",
    )
    .unwrap();
    let content = ok_content(recall(&c, "fait"));
    assert_eq!(content["count"], 1);
    assert_eq!(content["results"][0]["provenance"], "t");
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn forget_exact_value_and_file_content() {
    let c = ctx("forget-exact");
    assert!(store(&c, "le canari est vert", "t").is_ok());
    assert!(store(&c, "le chat dort", "t").is_ok());
    assert!(store(&c, "le chien court", "t").is_ok());
    let content = ok_content(forget(&c, "chat"));
    assert_eq!(content["query"], "chat");
    assert_eq!(content["removed"], 1);
    assert_eq!(content["remaining"], 2);
    // fichier réécrit avec les 2 faits restants
    let file = std::fs::read_to_string(c.root.join(MEMORY_PATH)).unwrap();
    assert!(file.contains("canari"));
    assert!(!file.contains("chat dort"));
    assert!(file.contains("chien"));
    // recall confirme
    let content = ok_content(recall(&c, "chat"));
    assert_eq!(content["count"], 0);
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn forget_empty_query_is_error() {
    let c = ctx("forget-empty");
    assert!(!forget(&c, "  ").is_ok());
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn forget_on_missing_file_creates_empty_file() {
    let c = ctx("forget-missing");
    std::fs::create_dir_all(c.root.join(".omega")).unwrap();
    let content = ok_content(forget(&c, "x"));
    assert_eq!(content["removed"], 0);
    assert_eq!(content["remaining"], 0);
    // atomic_write doit réellement écrire (pas un no-op)
    assert!(c.root.join(MEMORY_PATH).exists());
    let file = std::fs::read_to_string(c.root.join(MEMORY_PATH)).unwrap();
    assert_eq!(file, "");
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn memory_forget_alias_matches_forget() {
    let c = ctx("alias");
    assert!(store(&c, "fait alpha", "t").is_ok());
    let a = ok_content(forget(&c, "alpha"));
    // reset
    let c2 = ctx("alias2");
    assert!(store(&c2, "fait alpha", "t").is_ok());
    let b = ok_content(memory_forget(&c2, "alpha"));
    assert_eq!(a["removed"], b["removed"]);
    assert_eq!(a["remaining"], b["remaining"]);
    let _ = std::fs::remove_dir_all(&c.root);
    let _ = std::fs::remove_dir_all(&c2.root);
}

#[test]
fn forget_corrupt_line_is_kept() {
    let c = ctx("forget-corrupt");
    std::fs::create_dir_all(c.root.join(".omega")).unwrap();
    std::fs::write(
        c.root.join(MEMORY_PATH),
        "pas du json\n{\"content\":\"chat dort\",\"provenance\":\"t\"}\n",
    )
    .unwrap();
    let content = ok_content(forget(&c, "chat"));
    assert_eq!(content["removed"], 1);
    assert_eq!(content["remaining"], 1);
    let file = std::fs::read_to_string(c.root.join(MEMORY_PATH)).unwrap();
    assert!(file.contains("pas du json"));
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn unique_tmp_id_concurrent_200() {
    let c = std::sync::Arc::new(ctx("concurrent"));
    std::fs::create_dir_all(c.root.join(".omega")).unwrap();
    let mut handles = Vec::with_capacity(200);
    for _ in 0..200 {
        let c = std::sync::Arc::clone(&c);
        handles.push(std::thread::spawn(move || forget(&c, "rien-a-supprimer")));
    }
    let mut ok_count = 0;
    for h in handles {
        let out = h.join().unwrap();
        if out.is_ok() {
            ok_count += 1;
        }
    }
    assert_eq!(
        ok_count, 200,
        "toutes les écritures concurrentes doivent réussir"
    );
    assert!(c.root.join(MEMORY_PATH).exists());
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn recall_on_non_notfound_error_is_error() {
    // Tue memory.rs:87:19 (guard e.kind() == NotFound -> true) :
    // si le chemin est un repertoire, read_to_string echoue avec une erreur
    // qui n'est pas NotFound ; l'original renvoie Err, la mutation renvoie Ok(empty).
    let c = ctx("recall-non-notfound");
    std::fs::create_dir_all(c.root.join(".omega")).unwrap();
    // .omega/memory.jsonl est un repertoire -> erreur non-NotFound
    std::fs::create_dir_all(c.root.join(MEMORY_PATH)).unwrap();
    assert!(
        !recall(&c, "x").is_ok(),
        "recall sur repertoire doit echouer"
    );
    let _ = std::fs::remove_dir_all(&c.root);
}

#[test]
fn forget_on_non_notfound_error_is_error() {
    // Tue memory.rs:130:19 (guard e.kind() == NotFound -> true/false) :
    // fichier present mais illisible (permission refusee, != NotFound) ->
    // l'original renvoie Err, la mutation renvoie Ok(empty).
    let c = ctx("forget-non-notfound");
    std::fs::create_dir_all(c.root.join(".omega")).unwrap();
    std::fs::write(c.root.join(MEMORY_PATH), "x").unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(
        c.root.join(MEMORY_PATH),
        std::fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    assert!(
        !forget(&c, "x").is_ok(),
        "forget sur fichier illisible doit echouer"
    );
    // restaure les permissions pour le cleanup
    std::fs::set_permissions(
        c.root.join(MEMORY_PATH),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(&c.root);
}
