//! Contrats comportementaux : valeurs par défaut serde (lib.rs + skill.rs)
//! et méthodes pures de `JsonlMemoryStore`.

use omega_memory::{skill::Skill, MemoryRecord};

// --- MemoryRecord defaults (lib.rs) -----------------------------------------

#[test]
fn memory_record_default_provenance_is_stated() {
    let r: MemoryRecord = serde_json::from_str(r#"{"id":"x","text":"y","timestamp":1}"#).unwrap();
    assert_eq!(r.provenance, "stated");
}

#[test]
fn memory_record_default_confidence_is_one() {
    let r: MemoryRecord = serde_json::from_str(r#"{"id":"x","text":"y","timestamp":1}"#).unwrap();
    assert_eq!(r.confidence, 1.0);
}

// (classification and status use #[serde(default)] with no default fn, so
// their serde default is the empty string; the "fact"/"committed" values are
// applied by load_records normalization, not by deserialization. No mutant
// targets them — no test needed here.)

#[test]
fn memory_record_default_session_id_is_none() {
    let r: MemoryRecord = serde_json::from_str(r#"{"id":"x","text":"y","timestamp":1}"#).unwrap();
    assert!(r.session_id.is_none());
}

#[test]
fn memory_record_default_ttl_is_none() {
    let r: MemoryRecord = serde_json::from_str(r#"{"id":"x","text":"y","timestamp":1}"#).unwrap();
    assert!(r.ttl_seconds.is_none());
}

#[test]
fn memory_record_default_namespace_is_none() {
    let r: MemoryRecord = serde_json::from_str(r#"{"id":"x","text":"y","timestamp":1}"#).unwrap();
    assert!(r.namespace.is_none());
    assert!(r.user_id.is_none());
}

#[test]
fn memory_record_default_version_is_zero() {
    // serde default for u64 is 0 (pas de default fn) — vérifie que la valeur
    // désérialisée n'est pas altérée par un mutant sur les champs suivants.
    let r: MemoryRecord = serde_json::from_str(r#"{"id":"x","text":"y","timestamp":1}"#).unwrap();
    assert_eq!(r.version, 0);
}

// --- Skill defaults (skill.rs) ---------------------------------------------

#[test]
fn skill_default_version_is_1_0_0() {
    let s: Skill = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
    assert_eq!(s.version, "1.0.0");
}

#[test]
fn skill_default_category_is_general() {
    let s: Skill = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
    assert_eq!(s.category, "general");
}

#[test]
fn skill_default_status_is_draft() {
    let s: Skill = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
    assert_eq!(s.status, "draft");
}

#[test]
fn skill_default_confidence_is_0_8() {
    let s: Skill = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
    assert_eq!(s.confidence, 0.8);
}

#[test]
fn skill_default_source_is_learned() {
    let s: Skill = serde_json::from_str(r#"{"name":"t"}"#).unwrap();
    assert_eq!(s.source, "learned");
}

// --- parse_body / parse_list_inline (via from_markdown) ---------------------

#[test]
fn parse_body_extracts_unknown_section_into_body_extra() {
    let md = "---\nname: t\n---\n\n## When to Use\nfoo\n\n## Foo\nbar\n";
    let s = Skill::from_markdown(md);
    assert_eq!(s.when_to_use, "foo");
    assert_eq!(s.body_extra, "bar");
}

#[test]
fn parse_list_inline_parses_tags() {
    let md = "---\nname: t\ntags: [a, b]\n---\n";
    let s = Skill::from_markdown(md);
    assert_eq!(s.tags, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn jsonl_namespace_returns_configured_value() {
    use omega_memory::JsonlMemoryStore;
    let dir = std::env::temp_dir().join("omega-jsonl-ns-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = JsonlMemoryStore::new_with_namespace(&dir, "ns1");
    assert_eq!(store.namespace(), Some("ns1"));
    let store2 = JsonlMemoryStore::new(&dir);
    assert_eq!(store2.namespace(), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn jsonl_vector_count_is_zero_on_fresh_store() {
    use omega_memory::JsonlMemoryStore;
    let dir = std::env::temp_dir().join("omega-jsonl-vc-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = JsonlMemoryStore::new(&dir);
    assert_eq!(store.vector_count(), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn jsonl_commit_draft_succeeds_after_propose() {
    use omega_memory::JsonlMemoryStore;
    let dir = std::env::temp_dir().join("omega-jsonl-draft-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = JsonlMemoryStore::new(&dir);
    let id = store.propose_draft("a draft fact").await.unwrap();
    assert!(
        store.commit_draft(&id).await.unwrap(),
        "le brouillon doit être commitable"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
