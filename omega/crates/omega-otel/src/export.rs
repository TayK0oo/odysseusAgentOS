//! Export OTel : mapping `Event` → `Span` OTel (ou fallback JSON).

use omega_types::{Event, Provenance};
use serde_json::json;

/// Nom du span OTel pour un Event (selon son `name`).
pub fn span_name(event: &Event) -> &'static str {
    match event.name.as_str() {
        "decision" => "omega.decision",
        "model_call" | "step.completed" => "omega.model_call",
        "tool_call" => "omega.tool_call",
        "cost" => "omega.cost",
        "error" | "step.failed" | "run.failed" => "omega.error",
        _ => "omega.event",
    }
}

/// Exporte un Event vers un span OTel (format JSON si pas de crate OTel réel).
///
/// Retourne une représentation JSON compatible OTel (nom, timestamp, attributs).
pub fn export_event_json(event: &Event) -> String {
    let provenance_str = match event.provenance {
        Provenance::Stated => "stated",
        Provenance::Observed => "observed",
        Provenance::Inferred => "inferred",
    };
    json!({
        "name": span_name(event),
        "attributes": {
            "provenance": provenance_str,
            "event_name": event.name,
            "durable": event.durable,
        }
    })
    .to_string()
}

/// Tag court de provenance.
pub fn provenance_tag(p: Provenance) -> &'static str {
    match p {
        Provenance::Stated => "[stated]",
        Provenance::Observed => "[observed]",
        Provenance::Inferred => "[inferred]",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::{Event, Provenance};

    #[test]
    fn export_event_json_contains_provenance() {
        let mut ev = Event::new("model_call", serde_json::json!({"model": "test"}));
        ev = ev.with_provenance(Provenance::Observed);
        let json = export_event_json(&ev);
        assert!(
            json.contains("\"provenance\":\"observed\""),
            "provenance must be in JSON"
        );
        assert!(
            json.contains("omega.model_call"),
            "span name must be present"
        );
    }

    #[test]
    fn provenance_tag_returns_correct_tag() {
        assert_eq!(provenance_tag(Provenance::Stated), "[stated]");
        assert_eq!(provenance_tag(Provenance::Observed), "[observed]");
        assert_eq!(provenance_tag(Provenance::Inferred), "[inferred]");
    }

    #[test]
    fn event_serializes_provenance() {
        let ev = Event::new("test", serde_json::json!(null)).with_provenance(Provenance::Stated);
        let json = serde_json::to_string(&ev).unwrap();
        assert!(
            json.contains("\"provenance\":\"stated\""),
            "provenance must serialize"
        );
    }
}
