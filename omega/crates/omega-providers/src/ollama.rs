//! Adaptateur `ollama` : implémente le port `Provider` via l'API OpenAI-compatible
//! d'Ollama (`POST {base_url}/v1/chat/completions`, `stream: false`).
//!
//! Premier adaptateur **réel** du registre (W2 inc3) : branché, un `App` exécute un
//! run de bout en bout sur modèle local, coût nul.

use async_trait::async_trait;
use omega_types::{ChatRequest, ChatResponse, Message, ModelId, OmegaError, OmegaResult, Role, Usage};
use serde::{Deserialize, Serialize};

/// Requête OpenAI-compatible (sous-ensemble utilisé).
#[derive(Debug, Clone, PartialEq, Serialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<WireMessage>,
    stream: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WireMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionsResponse {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<ResponseUsage>,
}

#[derive(Debug, Clone, Deserialize)]
struct Choice {
    #[serde(default)]
    message: Option<WireMessage>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ResponseUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

fn role_to_wire(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

fn wire_to_message(m: WireMessage) -> Message {
    let role = match m.role.as_str() {
        "system" => Role::System,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        _ => Role::User,
    };
    Message::new(role, m.content)
}

/// Provider Ollama (modèle local, coût nul).
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn model_id(&self) -> ModelId {
        ModelId::new(format!("ollama/{}", self.model))
    }

    fn endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }
}

#[async_trait]
impl omega_ports::Provider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    async fn complete(&self, req: ChatRequest) -> OmegaResult<ChatResponse> {
        let body = ChatCompletionsRequest {
            model: self.model.clone(),
            messages: req
                .messages
                .into_iter()
                .map(|m| WireMessage {
                    role: role_to_wire(m.role).to_string(),
                    content: m.content,
                })
                .collect(),
            stream: false,
        };
        let resp = self
            .client
            .post(self.endpoint())
            .json(&body)
            .send()
            .await
            .map_err(|e| OmegaError::Io(format!("ollama requete : {e}")))?;
        if !resp.status().is_success() {
            return Err(OmegaError::Io(format!(
                "ollama statut HTTP {}",
                resp.status()
            )));
        }
        let parsed: ChatCompletionsResponse = resp
            .json()
            .await
            .map_err(|e| OmegaError::Io(format!("ollama reponse illisible : {e}")))?;
        let message = parsed
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message)
            .map(wire_to_message)
            .ok_or_else(|| OmegaError::Io("ollama : aucun choix dans la reponse".to_string()))?;
        let usage = parsed.usage.unwrap_or_default();
        Ok(ChatResponse {
            message,
            usage: Usage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_openai_compatible() {
        let body = ChatCompletionsRequest {
            model: "qwen2.5-coder:7b".to_string(),
            messages: vec![WireMessage {
                role: "user".to_string(),
                content: "hi".to_string(),
            }],
            stream: false,
        };
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v["model"], "qwen2.5-coder:7b");
        assert_eq!(v["messages"][0]["role"], "user");
        assert_eq!(v["stream"], false);
    }

    #[test]
    fn response_parses_usage_and_choice() {
        let raw = serde_json::json!({
            "choices": [{"message": {"role": "assistant", "content": "bonjour"}}],
            "usage": {"prompt_tokens": 12, "completion_tokens": 3}
        });
        let parsed: ChatCompletionsResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(parsed.usage.clone().unwrap().prompt_tokens, 12);
        let msg = wire_to_message(parsed.choices[0].message.clone().unwrap());
        assert_eq!(msg.content, "bonjour");
        assert_eq!(msg.role, Role::Assistant);
    }

    #[test]
    fn empty_choices_is_typed_error_shape() {
        let raw = serde_json::json!({"choices": []});
        let parsed: ChatCompletionsResponse = serde_json::from_value(raw).unwrap();
        assert!(parsed.choices.is_empty());
    }

    #[test]
    fn endpoint_trims_trailing_slash() {
        let p = OllamaProvider::new("http://localhost:11434/", "m");
        assert_eq!(
            p.endpoint(),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(p.model_id().as_str(), "ollama/m");
    }
}
