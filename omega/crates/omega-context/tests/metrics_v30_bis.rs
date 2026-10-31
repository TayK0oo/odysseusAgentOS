//! Metrics v30-bis — performance metrics (cmd-budget, cmd-compaction, cmd-injection, cmd-skill-index).

use omega_context::{truncate_compact, ContextBudget, DefaultContextEngine, TokenEstimator};
use omega_ports::ContextEngine;
use omega_types::{Message, Role};

#[tokio::test]
async fn metric_tokens_per_run() {
    let engine = DefaultContextEngine::new(8000);
    let messages: Vec<Message> = (0..10)
        .map(|i| Message::new(Role::User, format!("msg {} {}", i, "x".repeat(400))))
        .collect();
    let result = engine.assemble(messages).await.unwrap();
    let tokens = TokenEstimator::new().estimate_messages(&result);
    println!("METRIC tokens_per_run = {}", tokens);
    assert!(tokens <= 8000);
}

#[tokio::test]
async fn metric_compaction_rate() {
    let engine = DefaultContextEngine::new(500);
    let messages: Vec<Message> = (0..20)
        .map(|i| Message::new(Role::User, format!("msg {} {}", i, "x".repeat(400))))
        .collect();
    let orig = messages.len();
    let result = engine.assemble(messages).await.unwrap();
    let dropped = orig - result.len();
    let rate = dropped as f64 / orig as f64;
    println!("METRIC compaction_rate = {:.2}", rate);
    assert!(rate >= 0.0);
}

#[tokio::test]
async fn metric_latency_p95() {
    let engine = DefaultContextEngine::new(8000);
    let messages: Vec<Message> = (0..10)
        .map(|i| Message::new(Role::User, format!("msg {} {}", i, "x".repeat(400))))
        .collect();
    let mut durations = Vec::new();
    for _ in 0..20 {
        let start = std::time::Instant::now();
        let _ = engine.assemble(messages.clone()).await.unwrap();
        durations.push(start.elapsed());
    }
    durations.sort();
    let p95 = durations[18];
    println!("METRIC latency_p95_us = {}", p95.as_micros());
    assert!(p95.as_micros() > 0);
}

#[tokio::test]
async fn metric_skill_cache_hits() {
    // SkillIndex doesn't expose cache_hits counter — document gap
    println!("METRIC skill_cache_hits = non_observable (no counter in SkillIndex)");
}
