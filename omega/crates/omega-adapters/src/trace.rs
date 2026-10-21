//! Trace par appel d'outil (P1-4) — via le port `EventStore` existant.
//!
//! Chaque appel d'outil laisse un événement durable `tool.called` dans le
//! journal du run : `run_id` (cle du journal), `call_id` (mis aussi dans
//! `operation_id`, clé d'idempotence de la reprise), `goal_id`, et
//! `origine_du_contenu` (`scripted` / `user` / `tool`). Le journal reste
//! rejouable tel quel : `TraceReplayer` trie par `seq` et compte les
//! événements qu'il ne séquence pas (aucune perte silencieuse).

use omega_types::DurableEvent;

/// Nom canonique de l'événement de trace d'appel d'outil.
pub const TOOL_CALLED: &str = "tool.called";

/// Origine du contenu qui a déclenché l'appel (`origine_du_contenu`).
///
/// - `Scripted` : provider scripté (banc d'acceptation, CI) ;
/// - `User` : demande directe de l'utilisateur ;
/// - `Tool` : contenu produit par un outil (sorties analysees en P4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentOrigin {
    Scripted,
    User,
    Tool,
}

/// Construit l'événement durable d'un appel d'outil.
///
/// Forme constante (la relecture A4 compare des traces d'or) : les quatre
/// champs de traçabilité sont toujours présents (`goal_id` = `null` si
/// l'appel n'a pas de but identifié), plus `tool` et `arguments` qui
/// permettent de rejouer l'appel sans ré-exécuter le run.
pub fn tool_call_event(
    run_id: &str,
    call_id: &str,
    goal_id: Option<&str>,
    origin: ContentOrigin,
    tool: &str,
    arguments: serde_json::Value,
) -> DurableEvent {
    DurableEvent::new(
        run_id,
        TOOL_CALLED,
        serde_json::json!({
            "tool": tool,
            "call_id": call_id,
            "goal_id": goal_id,
            "origin": origin,
            "arguments": arguments,
        }),
    )
    .with_operation_id(Some(call_id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_otel::TraceReplayer;
    use omega_ports::EventStore;
    use std::sync::{Arc, Mutex};

    /// Store mémoire minimal : le vrai store (`FileEventStore`) est couvert
    /// par ses propres tests ; ici on prouve la géométrie des événements.
    struct MemStore {
        events: Mutex<Vec<DurableEvent>>,
    }

    impl EventStore for MemStore {
        fn name(&self) -> &str {
            "mem"
        }
        fn append(&self, mut event: DurableEvent) -> omega_types::OmegaResult<u64> {
            let mut v = self.events.lock().unwrap();
            let seq = v.len() as u64;
            event.seq = seq;
            v.push(event);
            Ok(seq)
        }
        fn read_run(&self, run_id: &str) -> omega_types::OmegaResult<Vec<DurableEvent>> {
            Ok(self
                .events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.run_id == run_id)
                .cloned()
                .collect())
        }
    }

    #[test]
    fn origin_serializes_to_stable_tags() {
        assert_eq!(
            serde_json::to_value(ContentOrigin::Scripted).unwrap(),
            serde_json::json!("scripted")
        );
        assert_eq!(
            serde_json::to_value(ContentOrigin::User).unwrap(),
            serde_json::json!("user")
        );
        assert_eq!(
            serde_json::to_value(ContentOrigin::Tool).unwrap(),
            serde_json::json!("tool")
        );
    }

    #[test]
    fn tool_call_event_carries_the_four_trace_fields() {
        let e = tool_call_event(
            "gold-1",
            "call-2",
            Some("S1.liste"),
            ContentOrigin::Scripted,
            "fs.list",
            serde_json::json!({ "champ": "x" }),
        );
        assert_eq!(e.run_id, "gold-1");
        assert_eq!(e.name, "tool.called");
        assert_eq!(e.operation_id.as_deref(), Some("call-2"));
        let p = &e.payload;
        assert_eq!(p["tool"], "fs.list");
        assert_eq!(p["call_id"], "call-2");
        assert_eq!(p["goal_id"], "S1.liste");
        assert_eq!(p["origin"], "scripted");
        assert_eq!(p["arguments"]["champ"], "x");
    }

    #[test]
    fn goal_id_is_constant_null_when_absent() {
        // Forme constante : le champ est present (null), pas (absent=null).
        let e = tool_call_event(
            "r",
            "c",
            None,
            ContentOrigin::User,
            "fs.write",
            serde_json::json!({}),
        );
        assert!(e.payload["goal_id"].is_null());
    }

    /// Trace d'or (A4) : trois appels (scripted/user/tool) journalises puis
    /// rejoues — JSONL serde correspond, le replay est deterministe et
    /// compte les `tool.called` (jamais perdus en silence).
    #[test]
    fn golden_trace_round_trips_and_replays_deterministically() {
        let store = Arc::new(MemStore {
            events: Mutex::new(Vec::new()),
        });
        for (call, goal, origin, tool) in [
            ("call-1", "S1.1", ContentOrigin::Scripted, "fs.list"),
            ("call-2", "S1.1", ContentOrigin::User, "fs.write"),
            ("call-3", "S1.2", ContentOrigin::Tool, "fs.read"),
        ] {
            store
                .append(tool_call_event(
                    "gold-1",
                    call,
                    Some(goal),
                    origin,
                    tool,
                    serde_json::json!({}),
                ))
                .unwrap();
        }
        let journal = store.read_run("gold-1").unwrap();
        assert_eq!(journal.len(), 3);
        assert!(journal.iter().all(|e| e.name == "tool.called"));

        // Round-trip serde_json (FileEventStore persiste une ligne JSONL).
        let jsonl = serde_json::to_string(&journal).unwrap();
        let replayed_journal: Vec<DurableEvent> = serde_json::from_str(&jsonl).unwrap();
        assert_eq!(replayed_journal, journal);

        // Replay deterministe : deux passes, meme resume, appels ordonnes.
        let s1 = TraceReplayer::summarize("gold-1", journal.clone());
        let s2 = TraceReplayer::summarize("gold-1", replayed_journal);
        assert_eq!(s1, s2);
        assert_eq!(s1.total_events, 3);
        let calls: Vec<&str> = journal
            .iter()
            .filter_map(|e| e.payload["call_id"].as_str())
            .collect();
        assert_eq!(calls, vec!["call-1", "call-2", "call-3"]);
        let origins: Vec<&str> = journal
            .iter()
            .filter_map(|e| e.payload["origin"].as_str())
            .collect();
        assert_eq!(origins, vec!["scripted", "user", "tool"]);
    }
}
