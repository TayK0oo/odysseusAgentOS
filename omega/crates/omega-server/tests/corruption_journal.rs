//! Corruption (P2) — tronquer la dernière ligne du journal sur disque ne doit
//! jamais être absorbé silencieusement : reprise = récupération avec rejet
//! explicite (trace) OU erreur explicite.

use omega_app::FileEventStore;
use omega_ports::EventStore;
use omega_types::DurableEvent;

fn ev(run_id: &str, name: &str) -> DurableEvent {
    DurableEvent::new(run_id, name, serde_json::json!({"op": name}))
        .with_operation_id(Some(name.to_string()))
}

#[test]
fn journal_tronque_rejette_la_queue_sans_fusion_silencieuse() {
    let dir = std::env::temp_dir().join("omega-corr-events");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let store = FileEventStore::new(dir.clone());
    store.append(ev("run-c", "a")).unwrap();
    store.append(ev("run-c", "b")).unwrap();

    let path = dir.join("run-c.jsonl");
    let before = store.read_run("run-c").unwrap();
    assert_eq!(before.len(), 2);

    // Tronque la dernière ligne (garde une queue partielle sans newline).
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.truncate(bytes.len().saturating_sub(12));
    std::fs::write(&path, &bytes).unwrap();

    // Reprise : la queue tronquée est rejetée, pas fusionnée silencieusement.
    let after = store.read_run("run-c").unwrap();
    assert!(
        after.len() < before.len(),
        "la ligne tronquée ne doit pas être conservée telle quelle : {} events",
        after.len()
    );

    // Récupération : un append suivant réattribue un seq cohérent et ne
    // ressuscite pas la ligne partielle.
    let seq = store.append(ev("run-c", "c")).unwrap();
    assert_eq!(seq, after.len() as u64);
    let final_journal = store.read_run("run-c").unwrap();
    assert_eq!(final_journal.len(), after.len() + 1);
    assert_eq!(final_journal.last().unwrap().name, "c");

    let _ = std::fs::remove_dir_all(&dir);
}
