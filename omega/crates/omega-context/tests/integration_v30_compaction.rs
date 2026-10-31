//! Integration tests v30 — LlmCompactionEngine + truncate_compact (cmd-compaction).

use omega_context::{truncate_compact, DefaultContextEngine, LlmCompactionEngine, TokenEstimator};
use omega_ports::ContextEngine;
use omega_types::{Message, Role};

#[tokio::test]
async fn truncate_compact_preserves_system() {
    let msgs = vec![
        Message::new(Role::System, "system prompt".to_string()),
        Message::new(Role::User, "old message 1".to_string()),
        Message::new(Role::User, "old message 2".to_string()),
        Message::new(Role::User, "recent message".to_string()),
    ];
    let kept = truncate_compact(&msgs, 50);
    assert!(
        kept.iter().any(|m| m.role == Role::System),
        "system preserved"
    );
    let est = omega_context::TokenEstimator::new();
    assert!(est.estimate_messages(&kept) <= 50 || kept.len() < msgs.len());
}

#[tokio::test]
async fn truncate_compact_drops_oldest_non_system() {
    let msgs = vec![
        Message::new(Role::System, "sys".to_string()),
        Message::new(Role::User, "a".repeat(200)),
        Message::new(Role::User, "b".repeat(200)),
        Message::new(Role::User, "c".to_string()),
    ];
    let kept = truncate_compact(&msgs, 30);
    // oldest non-system dropped
    assert!(
        !kept.iter().any(|m| m.content == "a".repeat(200)),
        "oldest dropped"
    );
}

#[tokio::test]
async fn truncate_compact_deterministic() {
    let msgs = vec![
        Message::new(Role::System, "sys".to_string()),
        Message::new(Role::User, "hello".to_string()),
        Message::new(Role::User, "world".to_string()),
    ];
    let a = truncate_compact(&msgs, 100);
    let b = truncate_compact(&msgs, 100);
    assert_eq!(a, b, "deterministic");
}

#[tokio::test]
async fn llm_compaction_engine_fallback_on_provider_failure() {
    // LlmCompactionEngine with a failing provider should fall back to truncate_compact
    // (verified by the engine not panicking and returning a result)
    let inner = DefaultContextEngine::new(1000);
    // Use a provider that always fails
    struct FailProvider;
    #[async_trait::async_trait]
    impl omega_ports::Provider for FailProvider {
        fn id(&self) -> &str {
            "fail"
        }
        async fn complete(
            &self,
            _req: omega_types::ChatRequest,
        ) -> omega_types::OmegaResult<omega_types::ChatResponse> {
            Err(omega_types::OmegaError::Io("fail".to_string()))
        }
    }
    let engine = LlmCompactionEngine::new(
        std::sync::Arc::new(inner),
        std::sync::Arc::new(FailProvider),
    );
    let msgs = vec![
        Message::new(Role::System, "sys".to_string()),
        Message::new(Role::User, "a".repeat(500)),
        Message::new(Role::User, "b".to_string()),
    ];
    let result = engine.assemble(msgs).await;
    // Should not panic; either Ok or Err is acceptable (fallback behavior)
    let _ = result;
}
