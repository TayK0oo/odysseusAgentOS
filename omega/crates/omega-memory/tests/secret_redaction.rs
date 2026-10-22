//! P2-6 — masquage des secrets dans la mémoire (preuve par relecture fs).

use omega_memory::JsonlMemoryStore;
use omega_ports::MemoryStore;
use std::fs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("p2-6-mem-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[tokio::test]
async fn secret_redacted_on_disk_memory() {
    let dir = tmp("redact");
    let store = JsonlMemoryStore::new(&dir);
    let canary_sk = "sk_live_0123456789abcdef";
    let canary_ak = "AKIA0123456789ABCDEF";
    let fact = format!("token={} and key={}", canary_sk, canary_ak);
    store.remember(&fact).await.unwrap();

    let path = dir.join("memories.jsonl");
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        !content.contains(canary_sk),
        "sk_live canary leaked in memory file"
    );
    assert!(
        !content.contains(canary_ak),
        "AKIA canary leaked in memory file"
    );
    assert!(content.contains("[REDACTED:"), "no redaction marker");
    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn normal_text_intact_memory() {
    let dir = tmp("normal");
    let store = JsonlMemoryStore::new(&dir);
    let fact = "hello world no secrets here";
    store.remember(fact).await.unwrap();

    let path = dir.join("memories.jsonl");
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains(fact), "normal text must remain intact");
    let _ = fs::remove_dir_all(&dir);
}
