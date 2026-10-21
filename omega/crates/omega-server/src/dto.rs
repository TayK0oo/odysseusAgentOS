//! DTO de l'API HTTP (serde, versionnes).
//!
//! Les resumes rejoues reutilisent les types canoniques d'`omega-otel`
//! (`ReplayedRun`, `RunSummary`) : serialises tels quels, jamais recopies.
//! Les entrees/sorties propres au bord (`ApiRunRequest`, `PluginEntry`,
//! `RunResponse`...) portent `schema_version` aligne sur
//! `omega-protocol::SCHEMA_VERSION`.

use omega_core::RunStatus;
use omega_otel::{ReplayedRun, RunSummary};
use omega_protocol::SCHEMA_VERSION;
use omega_types::{Capability, Message, Role, RouterDecision};
use serde::{Deserialize, Serialize};

/// Message tel qu'envoye par l'UI (`role` en clair, valide a l'entree).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: String,
}

/// Corps de `POST /api/run`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiRunRequest {
    pub messages: Vec<ApiMessage>,
    pub run_id: Option<String>,
}

/// Convertit un role en clair vers le domaine. Insensible a la casse ;
/// rejette tout le reste (400, jamais de repli silencieux vers `user`).
pub fn parse_role(raw: &str) -> Result<Role, String> {
    match raw.trim().to_lowercase().as_str() {
        "system" => Ok(Role::System),
        "user" => Ok(Role::User),
        "assistant" => Ok(Role::Assistant),
        "tool" => Ok(Role::Tool),
        other => Err(format!(
            "role inconnu : '{other}' (system|user|assistant|tool)"
        )),
    }
}

impl ApiRunRequest {
    /// Valide et convertit vers les messages du domaine.
    ///
    /// Erreurs (400) : liste vide, contenu vide, role inconnu.
    pub fn to_messages(&self) -> Result<Vec<Message>, String> {
        if self.messages.is_empty() {
            return Err("aucun message fourni".to_string());
        }
        self.messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                if m.content.trim().is_empty() {
                    return Err(format!("message #{i} : contenu vide"));
                }
                Ok(Message::new(parse_role(&m.role)?, m.content.clone()))
            })
            .collect()
    }
}

/// Corps de `GET /api/health`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub schema_version: u32,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            schema_version: SCHEMA_VERSION,
        }
    }
}

/// Entree de `GET /api/plugins` : plugin actif (`active` toujours vrai —
/// la liste provient d'`App::active_ids`, quarantaine deja appliquee).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginEntry {
    pub schema_version: u32,
    pub id: String,
    pub capability: String,
    pub active: bool,
}

impl PluginEntry {
    pub fn active(id: String, capability: Capability) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id,
            capability: capability.as_str().to_string(),
            active: true,
        }
    }

    /// Entree du registre de composition (P1-2) : `active` reflète réellement
    /// l'état de la composition (enregistré = listé, branché = actif).
    pub fn registered(id: String, capability: &str, active: bool) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id,
            capability: capability.to_string(),
            active,
        }
    }
}

/// Corps de `POST /api/run` (succes) : `summary` = `RunSummary` serialise,
/// `answer` = derniere reponse du modele relue dans le journal (`infer.completed`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunResponse {
    pub run_id: String,
    pub status: RunStatus,
    pub decision: RouterDecision,
    pub summary: RunSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
}

/// Corps de `GET /api/trace/:run_id` : journal rejoue + resume deterministe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceResponse {
    pub run_id: String,
    pub replayed: ReplayedRun,
    pub summary: RunSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
}

/// Corps d'erreur JSON (toujours versionne, pour l'UI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub schema_version: u32,
    pub error: String,
}

impl ApiErrorBody {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            error: error.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<T>(value: &T) -> T
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let json = serde_json::to_string(value).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn parse_role_accepts_known_roles_case_insensitive() {
        assert_eq!(parse_role("user").unwrap(), Role::User);
        assert_eq!(parse_role("User").unwrap(), Role::User);
        assert_eq!(parse_role(" SYSTEM ").unwrap(), Role::System);
        assert_eq!(parse_role("assistant").unwrap(), Role::Assistant);
        assert_eq!(parse_role("tool").unwrap(), Role::Tool);
    }

    #[test]
    fn parse_role_rejects_unknown() {
        assert!(parse_role("admin").is_err());
        assert!(parse_role("").is_err());
    }

    #[test]
    fn to_messages_validates_input() {
        let ok = ApiRunRequest {
            messages: vec![ApiMessage {
                role: "user".to_string(),
                content: "bonjour".to_string(),
            }],
            run_id: None,
        };
        assert_eq!(
            ok.to_messages().unwrap(),
            vec![Message::new(Role::User, "bonjour")]
        );

        let empty = ApiRunRequest {
            messages: vec![],
            run_id: None,
        };
        assert!(empty.to_messages().is_err());

        let bad_role = ApiRunRequest {
            messages: vec![ApiMessage {
                role: "admin".to_string(),
                content: "x".to_string(),
            }],
            run_id: None,
        };
        assert!(bad_role.to_messages().is_err());

        let blank = ApiRunRequest {
            messages: vec![ApiMessage {
                role: "user".to_string(),
                content: "   ".to_string(),
            }],
            run_id: None,
        };
        assert!(blank.to_messages().is_err());
    }

    #[test]
    fn health_response_is_versioned() {
        let h = HealthResponse::ok();
        assert_eq!(h.status, "ok");
        assert_eq!(h.schema_version, SCHEMA_VERSION);
        assert_eq!(round_trip(&h), h);
    }

    #[test]
    fn plugin_entry_round_trip() {
        let e = PluginEntry::active("ollama".to_string(), Capability::LlmProvider);
        assert!(e.active);
        assert_eq!(e.capability, "llm-provider");
        assert_eq!(round_trip(&e), e);
    }

    #[test]
    fn run_response_round_trip_with_replayed_summary() {
        // `summary` est le `RunSummary` canonique (pas une copie locale).
        use omega_otel::{ReplayedOutcome, TraceReplayer};
        use omega_types::DurableEvent;
        let journal = vec![
            DurableEvent::new("run-1", "run.started", serde_json::json!(null)),
            DurableEvent::new(
                "run-1",
                "step.completed",
                serde_json::json!({"operation": "assemble#0"}),
            )
            .with_operation_id(Some("assemble#0".to_string())),
            DurableEvent::new("run-1", "run.completed", serde_json::json!(null)),
        ];
        let summary = TraceReplayer::summarize("run-1", journal);
        assert_eq!(summary.outcome, ReplayedOutcome::Completed);
        let resp = RunResponse {
            run_id: "run-1".to_string(),
            status: RunStatus::Ok,
            decision: RouterDecision::new("ollama/qwen2.5-coder:7b", 0.91, "task=T3 eligible=1"),
            summary,
            answer: Some("bonjour".to_string()),
        };
        assert_eq!(round_trip(&resp), resp);
    }

    #[test]
    fn trace_response_round_trip() {
        use omega_otel::TraceReplayer;
        let journal = vec![];
        let replayed = TraceReplayer::replay("run-x", journal.clone());
        let summary = TraceReplayer::summarize("run-x", journal);
        let resp = TraceResponse {
            run_id: "run-x".to_string(),
            replayed,
            summary,
            answer: None,
        };
        assert_eq!(round_trip(&resp), resp);
        assert_eq!(resp.summary.total_events, 0);
    }

    #[test]
    fn api_request_deserializes_ui_shape() {
        // Forme envoyee par l'UI (`protocol.ts` : {messages, run_id?}).
        let req: ApiRunRequest = serde_json::from_str(
            r#"{"messages":[{"role":"user","content":"resume ceci"}],"run_id":"r1"}"#,
        )
        .unwrap();
        assert_eq!(req.run_id.as_deref(), Some("r1"));
        assert_eq!(req.to_messages().unwrap().len(), 1);
    }
}
