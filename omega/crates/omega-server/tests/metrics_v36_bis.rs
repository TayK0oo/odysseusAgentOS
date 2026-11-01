//! Mesure tokens/run — ferme la dette v36 (tokens/run non mesuré).

use omega_context::{DefaultContextEngine, TokenEstimator};
use omega_ports::ContextEngine;
use omega_types::{Message, Role};

fn build_benchmark_messages(n: usize) -> Vec<Message> {
    (0..n)
        .map(|i| Message::new(Role::User, format!("benchmark message {} of {}", i + 1, n)))
        .collect()
}

#[tokio::test]
async fn measure_tokens_per_run() {
    let engine = DefaultContextEngine::new(8000);
    let messages = build_benchmark_messages(10);
    let result = engine.assemble(messages).await.unwrap();
    let estimator = TokenEstimator::new();
    let tokens = estimator.estimate_messages(&result);
    println!("METRIC tokens_per_run = {}", tokens);
    assert!(tokens <= 8000, "tokens {} doit être <= 8000", tokens);
}
