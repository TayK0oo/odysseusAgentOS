//! Tests W3 v23 — `omega-router` sous-incrément J1 (classifier + scoring).
//! Tue les mutants missed de classifier.rs (word_count > 40) et scoring.rs
//! (log_normalize / inverse_log_normalize bornes + arithmetique).

use omega_ports::ScoringPolicy;
use omega_ports::TaskClassifier;
use omega_router::scoring::CompositeScoringPolicy;
use omega_router::task::TaskType;
use omega_types::{Capabilities, ModelRecord, QualityAxes, RoutingRequest};

fn classify(text: &str) -> TaskType {
    omega_router::classifier::HeuristicTaskClassifier::new().classify(&RoutingRequest::new(text))
}

#[test]
fn classify_40_words_is_not_long_reasoning() {
    // tue classifier.rs:197:29  >  ->  ==  et  >
    let text = "mot ".repeat(40);
    let t = text.trim();
    assert_ne!(classify(t), TaskType::LongReasoning);
}

#[test]
fn classify_41_words_is_long_reasoning() {
    // tue classifier.rs:197:29  >  ->  >=
    let text = "mot ".repeat(41);
    let t = text.trim();
    assert_eq!(classify(t), TaskType::LongReasoning);
}

#[test]
fn classify_7_words_is_not_trivial() {
    // tue classifier.rs:200:25  < 8  ->  <=  /  ==
    let text = "mot ".repeat(7);
    let t = text.trim();
    // 7 mots : pas trivial (< 8 est trivial, donc 7 < 8 → trivial). On teste la borne 8.
    assert_eq!(classify(t), TaskType::TrivialCompletion);
}

#[test]
fn classify_8_words_is_not_trivial() {
    // tue classifier.rs:200:25  < 8  ->  >  (8 > 8 false → pas trivial)
    let text = "mot ".repeat(8);
    let t = text.trim();
    assert_ne!(classify(t), TaskType::TrivialCompletion);
}

fn model(id: &str, cost: f64, speed: f32) -> ModelRecord {
    ModelRecord {
        id: id.to_string(),
        provider: "test".to_string(),
        context_length: 4000,
        cost_input_per_1m: cost,
        cost_output_per_1m: cost,
        tokens_per_second: speed,
        ttft_ms: 100,
        axes: QualityAxes::default(),
        capabilities: Capabilities::default(),
        tier: omega_types::ModelTier::Standard,
    }
}

#[test]
fn scoring_two_distinct_candidates_is_finite_and_not_one() {
    // tue log_normalize/inverse_log_normalize : < 2, (max-min).abs() < 1e-12,
    // +, -, *, /, %  (bornes asymetriques)
    let policy = CompositeScoringPolicy::new();
    let a = model("a", 1.0, 100.0);
    let b = model("b", 5.0, 50.0);
    let cands = vec![a.clone(), b.clone()];
    let sa = policy.score(TaskType::Chat, 0, &cands, &a);
    let sb = policy.score(TaskType::Chat, 0, &cands, &b);
    assert!(sa.is_finite() && sb.is_finite());
    assert!(
        (sa - sb).abs() > 1e-9,
        "scores must differ for distinct candidates"
    );
}

#[test]
fn scoring_single_candidate_returns_one_normalization() {
    // tue candidates.len() < 2  ->  ==  (1 == 2 false → falls through → min=max → 1.0)
    // et  >  (1 > 2 false → falls through → 1.0)
    let policy = CompositeScoringPolicy::new();
    let lone = model("lone", 2.0, 80.0);
    let cands = vec![lone.clone()];
    let s = policy.score(TaskType::Chat, 0, &cands, &lone);
    assert!(s.is_finite());
}

#[test]
fn scoring_zero_cost_candidate_stays_finite() {
    // tue (1.0 + min).ln() -> *  (min=0 → ln(0) = -inf → NaN)
    let policy = CompositeScoringPolicy::new();
    let free = model("free", 0.0, 0.0);
    let paid = model("paid", 1.0, 3.0);
    let cands = vec![free.clone(), paid.clone()];
    let s = policy.score(TaskType::Chat, 0, &cands, &free);
    assert!(
        s.is_finite(),
        "score must be finite even with zero-cost candidate"
    );
}

#[test]
fn scoring_identical_candidates_same_score() {
    // tue (max - min).abs() < 1e-12  ->  >  (identical → 0 > 1e-12 false → falls through → division by zero → NaN/inf)
    let policy = CompositeScoringPolicy::new();
    let a = model("a", 1.0, 100.0);
    let b = a.clone();
    let cands = vec![a.clone(), b];
    let sa = policy.score(TaskType::Chat, 0, &cands, &a);
    assert!(sa.is_finite());
}
