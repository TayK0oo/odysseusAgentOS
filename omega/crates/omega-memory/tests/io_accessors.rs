//! Contrats comportementaux des I/O helpers via API publique (incr-I6).

use omega_memory::JsonlMemoryStore;
use std::fs;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("omega-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

// --- now_nanos (via remember_full distinct ids) -----------------------------

#[tokio::test]
async fn remember_full_generates_distinct_ids() {
    let dir = temp_dir("distinct-ids");
    let store = JsonlMemoryStore::new(&dir);
    store
        .remember_full("fact one", "stated", "fact", None, None, None)
        .await
        .unwrap();
    store
        .remember_full("fact two", "stated", "fact", None, None, None)
        .await
        .unwrap();
    let all = store.all(10);
    assert_eq!(all.len(), 2);
    assert_ne!(
        all[0].id, all[1].id,
        "deux remember_full doivent produire des ids distincts"
    );
    let _ = fs::remove_dir_all(&dir);
}

// --- load_records (via all()) ----------------------------------------------

#[test]
fn all_reads_existing_file() {
    let dir = temp_dir("read-existing");
    fs::write(
        dir.join("memories.jsonl"),
        r#"{"id":"a","text":"hello","timestamp":1}"#.to_string() + "\n",
    )
    .unwrap();
    let store = JsonlMemoryStore::new(&dir);
    let all = store.all(10);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].text, "hello");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn all_ignores_corrupted_lines() {
    let dir = temp_dir("read-corrupt");
    fs::write(
        dir.join("memories.jsonl"),
        "garbage line\n{\"id\":\"b\",\"text\":\"ok\",\"timestamp\":2}\nanother garbage\n",
    )
    .unwrap();
    let store = JsonlMemoryStore::new(&dir);
    let all = store.all(10);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, "b");
    let _ = fs::remove_dir_all(&dir);
}

// --- save_records (via remember) -------------------------------------------

#[tokio::test]
async fn remember_persists_to_disk() {
    let dir = temp_dir("persist");
    let store = JsonlMemoryStore::new(&dir);
    store.remember("persistent fact").await.unwrap();
    let content = fs::read_to_string(dir.join("memories.jsonl")).unwrap();
    assert!(
        content.contains("persistent fact"),
        "le fichier doit contenir le fait"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn remember_leaves_no_tmp_file() {
    let dir = temp_dir("no-tmp");
    let store = JsonlMemoryStore::new(&dir);
    store.remember("fact").await.unwrap();
    let tmps: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "tmp").unwrap_or(false))
        .collect();
    assert!(tmp_files_is_empty(&tmps), "aucun .tmp ne doit rester");
    let _ = fs::remove_dir_all(&dir);
}

fn tmp_files_is_empty(tmps: &[std::fs::DirEntry]) -> bool {
    tmps.is_empty()
}

#[tokio::test]
async fn remember_five_facts_all_persist() {
    // tue :lib.rs:251 unique_tmp_path ^ -> & (collision) et ^ -> |
    // Si le hash XOR est remplacé par AND, des collisions de tmp path peuvent
    // écraser des fichiers et réduire le nombre de records persistés.
    let dir = temp_dir("five-facts");
    let store = JsonlMemoryStore::new(&dir);
    for i in 0..5 {
        store.remember(&format!("fact number {}", i)).await.unwrap();
    }
    let all = JsonlMemoryStore::new(&dir).all(10);
    assert_eq!(
        all.len(),
        5,
        "5 faits distincts doivent persister (aucune collision tmp)"
    );
    let unique: std::collections::HashSet<_> = all.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(unique.len(), 5, "aucun fait ne doit être écrasé");
    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn search_filters_by_namespace() {
    let dir = temp_dir("ns-filter");
    let store = JsonlMemoryStore::new_with_namespace(&dir, "ns1");
    store
        .remember_full(
            "fact in ns1",
            "stated",
            "fact",
            Some("ns1".into()),
            None,
            None,
        )
        .await
        .unwrap();
    // Un record dans un autre namespace ne doit pas apparaître.
    let store2 = JsonlMemoryStore::new(&dir);
    store2
        .remember_full(
            "fact in ns2",
            "stated",
            "fact",
            Some("ns2".into()),
            None,
            None,
        )
        .await
        .unwrap();
    // Le store namespacé ne voit que ns1.
    let hits = store.search("fact", 10);
    assert!(
        hits.iter()
            .all(|s| s.record.namespace.as_deref() == Some("ns1")),
        "filtrage par namespace"
    );
    let _ = fs::remove_dir_all(&dir);
}

// --- FileSkillStore::name + load_usage/save_usage ---------------------------

#[test]
fn file_skill_store_name_is_file_skill_store() {
    use omega_memory::skill::FileSkillStore;
    use omega_ports::SkillStore;
    let dir = temp_dir("skill-name");
    let store = FileSkillStore::new(&dir);
    assert_eq!(store.name(), "file-skill-store");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn file_skill_store_load_usage_restores_counts() {
    use omega_memory::skill::FileSkillStore;
    let dir = temp_dir("skill-usage");
    let skill_dir = dir.join("skills").join("general").join("myskill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: myskill\n---\n").unwrap();

    // Écrire _usage.json manuellement.
    let usage_dir = dir.join("skills");
    fs::write(
        usage_dir.join("_usage.json"),
        r#"{"myskill":{"uses":7,"last_used":1234567890}}"#,
    )
    .unwrap();

    let store = FileSkillStore::new(&dir);
    store.load_usage();
    let skills = store.all();
    assert!(!skills.is_empty());
    assert_eq!(skills[0].uses, 7, "uses doit être restauré");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn file_skill_store_save_usage_writes_file() {
    use omega_memory::skill::FileSkillStore;
    let dir = temp_dir("skill-save-usage");
    let skill_dir = dir.join("skills").join("general").join("myskill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: myskill\n---\n").unwrap();

    let store = FileSkillStore::new(&dir);
    store.record_use("myskill");
    store.record_use("myskill");
    store.save_usage();

    let usage_path = dir.join("skills").join("_usage.json");
    assert!(usage_path.exists(), "save_usage doit écrire _usage.json");
    let content = fs::read_to_string(&usage_path).unwrap();
    assert!(content.contains("\"uses\": 2"), "compte persisté");
    let _ = fs::remove_dir_all(&dir);
}
