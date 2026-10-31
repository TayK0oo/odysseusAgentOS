//! Integration tests v30 — ContextBudget + TokenEstimator (cmd-budget).

use omega_context::{ContextBudget, DefaultContextEngine, TokenEstimator};
use omega_ports::ContextEngine;
use omega_types::{Message, Role};

#[tokio::test]
async fn assemble_respects_budget_end_to_end() {
    let engine = DefaultContextEngine::new(500);
    let messages: Vec<Message> = (0..10)
        .map(|i| Message::new(Role::User, format!("msg {} {}", i, "x".repeat(400))))
        .collect();
    let result = engine.assemble(messages).await.unwrap();
    let est = TokenEstimator::new();
    let tokens = est.estimate_messages(&result);
    assert!(tokens <= 500, "budget respecté: {} <= 500", tokens);
}

#[tokio::test]
async fn assemble_returns_budget_exceeded_when_system_alone_over_budget() {
    let engine = DefaultContextEngine::new(10);
    let messages = vec![Message::new(Role::System, "x".repeat(100))];
    let result = engine.assemble(messages).await;
    assert!(result.is_err(), "system seul > budget → erreur");
}

#[tokio::test]
async fn context_budget_enforce_drops_when_exceeded() {
    let mut b = ContextBudget::new(100);
    assert!(b.enforce(60).is_ok());
    assert_eq!(b.used(), 60);
    assert_eq!(b.remaining(), 40);
    assert!(b.enforce(50).is_err());
    assert_eq!(b.used(), 60);
}

#[tokio::test]
async fn context_budget_check_without_consume() {
    let b = ContextBudget::new(100);
    assert!(b.check(50).is_ok());
    assert_eq!(b.used(), 0);
    assert!(b.check(200).is_err());
}

#[tokio::test]
async fn context_budget_single_message_finite() {
    let mut b = ContextBudget::new(50);
    let est = TokenEstimator::new();
    let tokens = est.estimate("hello world");
    assert!(b.enforce(tokens).is_ok());
    assert!(b.used() > 0);
}
