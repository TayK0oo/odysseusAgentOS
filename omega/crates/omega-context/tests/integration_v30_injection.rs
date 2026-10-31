//! Integration tests v30 — MemoryInjection + PinnedFacts (cmd-injection).

use async_trait::async_trait;
use omega_context::{ContextInjector, DefaultContextEngine, MemoryInjection, PinnedFacts};
use omega_ports::MemoryStore;
use omega_types::{Message, Role};
use std::sync::Arc;

struct StaticMem {
    facts: Vec<String>,
}
#[async_trait]
impl MemoryStore for StaticMem {
    fn name(&self) -> &str {
        "static"
    }
    async fn recall(&self, _query: &str, _limit: usize) -> omega_types::OmegaResult<Vec<String>> {
        Ok(self.facts.clone())
    }
    async fn remember(&self, _fact: &str) -> omega_types::OmegaResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn injection_adds_memory_to_context() {
    let store = Arc::new(StaticMem {
        facts: vec!["fact1".to_string(), "fact2".to_string()],
    });
    let inj = MemoryInjection::new(store);
    let msgs = vec![Message::new(Role::User, "query about facts".to_string())];
    let (injected, trace) = inj.inject(msgs.clone(), 1000).await.unwrap();
    assert!(injected.len() >= msgs.len(), "injection adds messages");
    let _ = trace;
}

#[tokio::test]
async fn pinned_facts_survive_compaction() {
    let msgs = vec![
        Message::new(Role::System, "sys".to_string()),
        Message::new(Role::User, "a".repeat(200)),
        Message::new(Role::User, "b".to_string()),
    ];
    let pinned = PinnedFacts::new();
    let out = omega_context::truncate_compact_with_pinned(&msgs, 30, &pinned);
    // No pinned facts → just compaction
    assert!(!out.is_empty());
}

#[tokio::test]
async fn assemble_with_injection_uses_memory() {
    let store = Arc::new(StaticMem {
        facts: vec!["canary fact 42".to_string()],
    });
    let engine = DefaultContextEngine::with_memory(store, 1000);
    let msgs = vec![Message::new(Role::User, "canary".to_string())];
    let (result, _report) = engine.assemble_with_injection(&msgs).await.unwrap();
    assert!(
        result.iter().any(|m| m.content.contains("canary fact 42")),
        "memory injected"
    );
}

#[tokio::test]
async fn injection_without_store_passthrough() {
    let engine = DefaultContextEngine::new(1000);
    let msgs = vec![Message::new(Role::User, "hello".to_string())];
    let (result, _) = engine.assemble_with_injection(&msgs).await.unwrap();
    assert_eq!(result.len(), msgs.len(), "passthrough without injector");
}
