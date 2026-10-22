//! P2-6 — masquage des secrets dans les traces (preuve par relecture fs).

use omega_app::FileEventStore;
use omega_ports::EventStore;
use omega_types::DurableEvent;
use std::fs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("p2-6-trace-{}", name));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn secret_redacted_on_disk_trace() {
    let dir = tmp("redact");
    let store = FileEventStore::new(&dir);
    let canary_sk = "sk_live_0123456789abcdef";
    let canary_ak = "AKIA0123456789ABCDEF";
    let ev = DurableEvent::new(
        "run1",
        "tool.called",
        serde_json::json!({
            "arguments": { "token": canary_sk, "key": canary_ak }
        }),
    );
    store.append(ev).unwrap();

    let path = dir.join("run1.jsonl");
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        !content.contains(canary_sk),
        "sk_live canary leaked in trace file"
    );
    assert!(
        !content.contains(canary_ak),
        "AKIA canary leaked in trace file"
    );
    assert!(content.contains("[REDACTED:"), "no redaction marker");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn normal_text_intact_trace() {
    let dir = tmp("normal");
    let store = FileEventStore::new(&dir);
    let ev = DurableEvent::new(
        "run2",
        "tool.called",
        serde_json::json!({
            "arguments": { "msg": "hello world no secrets" }
        }),
    );
    store.append(ev).unwrap();

    let path = dir.join("run2.jsonl");
    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("hello world no secrets"),
        "normal text must remain intact"
    );
    let _ = fs::remove_dir_all(&dir);
}
