//! Contrats comportementaux des helpers purs de `omega-memory` (lib.rs).
//!
//! Cible : les mutants survivants de la baseline brute sur les helpers purs.
//! Les fonctions I/O (`load_records`, `save_records`, `unique_tmp_path`,
//! `now_secs`, `now_nanos`) sont volontairement hors périmètre (beaucoup
//! d'unviable attendus — cf. Livrable D).

use omega_memory::{
    confidence_for, jaccard, normalize_provenance, provenance_tag, tokenize, MemoryRecord,
};
use std::collections::HashSet;

// --- normalize_provenance ---------------------------------------------------

#[test]
fn normalize_provenance_canonical_values() {
    assert_eq!(normalize_provenance("stated"), "stated");
    assert_eq!(normalize_provenance("OBSERVED"), "observed");
    assert_eq!(normalize_provenance("  [Inferred]  "), "inferred");
    assert_eq!(normalize_provenance("unknown"), "stated");
}

#[test]
fn normalize_provenance_unknown_defaults_to_stated() {
    assert_eq!(normalize_provenance(""), "stated");
    assert_eq!(normalize_provenance("garbage"), "stated");
}

// --- provenance_tag ---------------------------------------------------------

#[test]
fn provenance_tag_wraps_normalized() {
    assert_eq!(provenance_tag("observed"), "[observed]");
    assert_eq!(provenance_tag("GARBAGE"), "[stated]");
}

// --- confidence_for ---------------------------------------------------------

#[test]
fn confidence_for_each_provenance() {
    assert_eq!(confidence_for("stated"), 1.0);
    assert_eq!(confidence_for("observed"), 0.7);
    assert_eq!(confidence_for("inferred"), 0.4);
    assert_eq!(confidence_for("unknown"), 1.0);
}

// --- tokenize --------------------------------------------------------------

#[test]
fn tokenize_splits_and_filters() {
    let t = tokenize("Hello, world! a bb");
    assert_eq!(
        t,
        HashSet::from(["hello".to_string(), "world".to_string(), "bb".to_string()])
    );
}

#[test]
fn tokenize_empty_returns_empty() {
    assert!(tokenize("").is_empty());
    assert!(tokenize("   ").is_empty());
}

// --- jaccard ---------------------------------------------------------------

#[test]
fn jaccard_identical_is_one() {
    let a = HashSet::from(["x".to_string(), "y".to_string()]);
    assert_eq!(jaccard(&a, &a), 1.0);
}

#[test]
fn jaccard_disjoint_is_zero() {
    let a = HashSet::from(["x".to_string()]);
    let b = HashSet::from(["y".to_string()]);
    assert_eq!(jaccard(&a, &b), 0.0);
}

#[test]
fn jaccard_empty_is_zero() {
    let a: HashSet<String> = HashSet::new();
    let b = HashSet::from(["y".to_string()]);
    assert_eq!(jaccard(&a, &b), 0.0);
}

// --- is_expired ------------------------------------------------------------

#[test]
fn is_expired_respects_ttl_boundary() {
    let r = MemoryRecord::new("id", "text");
    let ts = r.timestamp;
    // ttl 10 : expiré à ts+10, pas à ts+9.
    let mut r = r;
    r.ttl_seconds = Some(10);
    assert!(!r.is_expired(ts + 9));
    assert!(r.is_expired(ts + 10));
}

#[test]
fn is_expired_negative_ttl_never_expires() {
    let mut r = MemoryRecord::new("id", "text");
    r.ttl_seconds = Some(-1);
    assert!(!r.is_expired(r.timestamp + 1000));
}

#[test]
fn is_expired_none_never_expires() {
    let r = MemoryRecord::new("id", "text");
    assert!(!r.is_expired(r.timestamp + 1_000_000));
}

// --- is_expired_now ---------------------------------------------------------

#[test]
fn is_expired_now_true_when_ttl_zero() {
    let mut r = MemoryRecord::new("id", "text");
    r.ttl_seconds = Some(0);
    assert!(r.is_expired_now());
}

// --- is_draft ---------------------------------------------------------------

#[test]
fn is_draft_true_when_draft_flag_or_status() {
    let mut r = MemoryRecord::new("id", "text");
    assert!(!r.is_draft());
    r.draft = true;
    assert!(r.is_draft());
    let mut r2 = MemoryRecord::new("id2", "text");
    r2.status = "draft".to_string();
    assert!(r2.is_draft());
}

#[test]
fn memory_record_timestamp_is_current_epoch() {
    let r = MemoryRecord::new("id", "text");
    assert!(
        r.timestamp > 1_000_000_000,
        "now_secs() doit renvoyer l'époque courante"
    );
}

#[tokio::test]
async fn commit_draft_succeeds_after_propose() {
    use omega_memory::InMemoryMemoryStore;
    let store = InMemoryMemoryStore::new();
    let id = store.propose_draft("fact").await.unwrap();
    assert!(
        store.commit_draft(&id).await.unwrap(),
        "le brouillon doit être commitable"
    );
}

#[test]
fn new_sets_stated_provenance_and_defaults() {
    let r = MemoryRecord::new("id", "text");
    assert_eq!(r.provenance, "stated");
    assert_eq!(r.confidence, 1.0);
    assert_eq!(r.classification, "fact");
    assert_eq!(r.status, "committed");
    assert!(!r.draft);
    assert_eq!(r.version, 1);
}

#[test]
fn new_with_normalizes_provenance_and_classification() {
    let r = MemoryRecord::new_with("id", "text", "OBSERVED", "", None, Some(5));
    assert_eq!(r.provenance, "observed");
    assert_eq!(r.confidence, 0.7);
    assert_eq!(r.classification, "fact");
    assert_eq!(r.ttl_seconds, Some(5));
    let r2 = MemoryRecord::new_with("id2", "text", "stated", "decision", Some("ns".into()), None);
    assert_eq!(r2.classification, "decision");
    assert_eq!(r2.namespace, Some("ns".to_string()));
}

#[test]
fn load_records_normalizes_confidence_zero() {
    let dir = std::env::temp_dir().join("omega-load-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("memories.jsonl");
    std::fs::write(&path, r#"{"id":"a","text":"ok","timestamp":1,"provenance":"observed","confidence":0.0,"status":""}"#.to_string() + "\n").unwrap();
    // Use JsonlMemoryStore to observe load_records via all()
    use omega_memory::JsonlMemoryStore;
    let store = JsonlMemoryStore::new(&dir);
    let all = store.all(10);
    assert_eq!(all.len(), 1);
    assert_eq!(
        all[0].confidence, 0.7,
        "confidence 0.0 doit être normalisée"
    );
    assert_eq!(
        all[0].status, "committed",
        "status vide doit être normalisé"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_records_preserves_draft_flag_consistency() {
    let dir = std::env::temp_dir().join("omega-load-draft-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("memories.jsonl");
    std::fs::write(
        &path,
        r#"{"id":"d","text":"draft fact","timestamp":1,"status":"draft","draft":false}"#
            .to_string()
            + "\n",
    )
    .unwrap();
    use omega_memory::JsonlMemoryStore;
    let store = JsonlMemoryStore::new(&dir);
    let all = store.all(10);
    assert_eq!(all.len(), 1);
    assert!(all[0].draft, "status \"draft\" doit mettre draft=true");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_records_does_not_flip_committed_draft_flag() {
    let dir = std::env::temp_dir().join("omega-load-committed-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("memories.jsonl");
    std::fs::write(
        &path,
        r#"{"id":"c","text":"committed fact","timestamp":1,"status":"committed","draft":false}"#
            .to_string()
            + "\n",
    )
    .unwrap();
    use omega_memory::JsonlMemoryStore;
    let store = JsonlMemoryStore::new(&dir);
    let all = store.all(10);
    assert_eq!(all.len(), 1);
    assert!(
        !all[0].draft,
        "status \"committed\" ne doit pas mettre draft=true"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
