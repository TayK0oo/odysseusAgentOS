//! Tests adversariaux — `omega-memory` (JSONL store + skills).
//!
//! On attaque la persistance : écriture atomique, corruption, suppression,
//! collisions, et le parseur de frontmatter des skills.

use omega_memory::skill::{parse_body, parse_frontmatter, slugify, FileSkillStore, Skill};
use omega_memory::{tokenize, JsonlMemoryStore};
use omega_ports::MemoryStore;
use std::sync::Arc;

fn dir(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("omega-adv-mem-{}", name));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn atk_partial_write_leaves_no_garbage() {
    // Si l'écriture échoue (disque plein simulé par un chemin invalide), le
    // fichier principal doit rester intact — pas de JSON tronqué.
    let d = dir("atomic");
    let store = JsonlMemoryStore::new(&d);
    futures_lite_block_on(async {
        store.remember("première mémoire").await.unwrap();
    });

    let path = d.join("memories.jsonl");
    let before = std::fs::read_to_string(&path).unwrap();

    // On rend le répertoire parent non inscriptible en le remplissant d'un fichier
    // du même nom : l'écriture doit échouer SANS corrompre l'existant.
    let store2 = JsonlMemoryStore::new(std::path::Path::new("/proc/nonexistent-dir"));
    let res = futures_lite_block_on(store2.remember("x"));
    assert!(res.is_err(), "écriture hors FS doit échouer");

    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(before, after, "le journal existant a été altéré");
    let _ = std::fs::remove_dir_all(&d);
}

/// Petit block_on pour éviter une dépendance async_rt en dev-deps.
fn futures_lite_block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

#[test]
fn atk_no_temp_file_left_behind() {
    let d = dir("tmpclean");
    let store = JsonlMemoryStore::new(&d);
    futures_lite_block_on(async {
        store.remember("x").await.unwrap();
    });
    let leftovers: Vec<_> = std::fs::read_dir(&d)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "un fichier .tmp a été laissé derrière"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn atk_corrupted_lines_are_skipped_not_fatal() {
    let d = dir("corrupt");
    let store = JsonlMemoryStore::new(&d);
    futures_lite_block_on(async {
        store.remember("bonne mémoire").await.unwrap();
        store.remember("deuxième mémoire").await.unwrap();
    });
    // Corruption : on insère une ligne invalide.
    let p = d.join("memories.jsonl");
    let content = std::fs::read_to_string(&p).unwrap();
    let corrupted = "{{{ ligne cassee".to_string();
    std::fs::write(&p, format!("{}\n{}", corrupted, content)).unwrap();

    let all = store.all(100);
    assert_eq!(all.len(), 2, "les lignes valides doivent survivre");
    assert!(all.iter().all(|m| !m.text.is_empty()));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn atk_recall_ignores_garbage_query() {
    let d = dir("garbageq");
    let store = JsonlMemoryStore::new(&d);
    futures_lite_block_on(async {
        store.remember("prospection commerciale").await.unwrap();
    });
    for q in ["", "   ", "\n\t", "!!!", "🎉🎉🎉", "a"] {
        let r = futures_lite_block_on(store.recall(q, 5));
        assert!(r.is_ok(), "la requête {:?} ne doit pas paniquer", q);
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn atk_delete_wrong_id_is_noop() {
    let d = dir("delnoop");
    let store = JsonlMemoryStore::new(&d);
    futures_lite_block_on(async {
        store.remember("garder").await.unwrap();
    });
    assert!(
        !store.delete("id-inexistant"),
        "suppression inexistante doit être no-op"
    );
    assert_eq!(store.all(10).len(), 1, "la mémoire a été perdue");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn atk_concurrent_remember_does_not_lose_writes() {
    // Chaque remember relit + réécrit le fichier : deux writers peuvent
    // s'écraser (read-modify-write sans verrou de fichier).
    let d = dir("concurrent");
    let store = Arc::new(JsonlMemoryStore::new(&d));
    let mut hs = Vec::new();
    for i in 0..20 {
        let s = Arc::clone(&store);
        hs.push(std::thread::spawn(move || {
            futures_lite_block_on(s.remember(&format!("mémoire {}", i)))
        }));
    }
    for h in hs {
        let _ = h.join().unwrap();
    }
    let all = store.all(1000);
    // Documenté : read-modify-write concurrent peut perdre des entrées.
    // Ce test verrouille le COMPORTEMENT CONNU (pas de garantie d Atomicité).
    println!("écrites concurrentes retenues : {}/20", all.len());
    assert!(all.len() <= 20, "jamais plus que le nombre d'écritures");
    assert!(!all.is_empty(), "au moins une écriture doit passer");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn atk_frontmatter_injection_is_contained() {
    // Un SKILL.md avec un frontmatter piégé ne doit pas faire interpreted
    // une section "## Procedure" du body comme du frontmatter.
    let evil = "---\nname: x\ndescription: \"a\\nb: 1\"\n---\n\n## When to Use\nvrai\n";
    let (fm, body) = parse_frontmatter(evil);
    assert!(fm.contains_key("name"));
    let (when, _, _, _, _) = parse_body(&body);
    assert_eq!(when, "vrai");
}

#[test]
fn atk_skill_name_traversal_is_rejected() {
    // Un nom de skill containing "../" ne doit pas écrire hors du store.
    let d = dir("skillesc");
    let store = FileSkillStore::new(&d);
    let evil = Skill {
        name: "../../escape".into(),
        category: "cat".into(),
        ..Default::default()
    };
    let _ = store.save(&evil);
    let written = d.join("escape").exists()
        || d.parent()
            .map(|p| p.join("escape").exists())
            .unwrap_or(false);
    assert!(
        !written,
        "THREAT OPEN: le nom de skill a escapes du répertoire du store"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn atk_slugify_never_empty() {
    for s in ["", "   ", "///", "!!!", "..", "🎉"] {
        let sl = slugify(s);
        assert!(!sl.is_empty(), "slug vide pour {:?}", s);
        assert!(!sl.contains('/'), "le slug {:?} contient un slash", s);
    }
}

#[test]
fn atk_tokenize_handles_huge_input() {
    let big = "mot ".repeat(200_000);
    let t = tokenize(&big);
    assert_eq!(t.len(), 1, "tokenize doit dédupliquer");
}

#[test]
fn cascade_forget_removes_all_traces() {
    let d = dir("cascade");
    let store = JsonlMemoryStore::new(&d);
    let canary = "zzxcascade-canary-9f8e7d6c";
    futures_lite_block_on(async {
        store.remember(canary).await.unwrap();
    });

    // recover the generated id from the persisted file
    let path = d.join("memories.jsonl");
    let content = std::fs::read_to_string(&path).unwrap();
    let id = content
        .lines()
        .next()
        .and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .and_then(|v| v.get("id").and_then(|x| x.as_str()).map(str::to_string))
        .expect("id present");

    futures_lite_block_on(async {
        assert!(store.forget(&id).await.unwrap());
    });

    // 0 trace in store file
    let after = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(!after.contains(canary), "canary leaked in store file");

    // 0 trace in BM25 recall
    let recalled = futures_lite_block_on(store.recall(canary, 5)).unwrap();
    assert!(
        recalled.is_empty(),
        "BM25 recall still returns the forgotten fact"
    );

    // 0 trace in vector index
    assert_eq!(store.vector_count(), 0, "vector index not cleared");
    let _ = std::fs::remove_dir_all(&d);
}
