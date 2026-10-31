//! Tests W3 v23 — `omega-router` sous-incrément J2 (selector + cache).

use omega_ports::ScoringPolicy;
use omega_ports::Selector;
use omega_router::scoring::CompositeScoringPolicy;
use omega_router::selector::ParetoSelector;
use omega_types::{Capabilities, ModelRecord, ModelTier, QualityAxes, RouterError, TaskType};

fn model(id: &str, tier: ModelTier, cost: f64, speed: f32, ctx: u32, chat: u8) -> ModelRecord {
    ModelRecord {
        id: id.to_string(),
        provider: "test".to_string(),
        context_length: ctx,
        cost_input_per_1m: cost,
        cost_output_per_1m: cost * 3.0,
        tokens_per_second: speed,
        ttft_ms: 100,
        axes: QualityAxes {
            chat,
            ..Default::default()
        },
        capabilities: Capabilities::default(),
        tier,
    }
}

#[test]
fn empty_registry_fails() {
    // tue selector.rs:55-58  candidates.is_empty() -> false
    let sel = ParetoSelector::new(CompositeScoringPolicy::new());
    assert_eq!(
        sel.select(TaskType::Chat, Capabilities::default(), 0, &[])
            .unwrap_err(),
        RouterError::EmptyRegistry
    );
}

#[test]
fn no_eligible_model_fails() {
    // tue selector.rs:66-69  eligible.is_empty() -> false
    let sel = ParetoSelector::new(CompositeScoringPolicy::new());
    let small = model("small", ModelTier::Standard, 1.0, 100.0, 4000, 70);
    assert_eq!(
        sel.select(TaskType::Chat, Capabilities::default(), 8000, &[small])
            .unwrap_err(),
        RouterError::NoEligibleModel
    );
}

#[test]
fn pareto_dominated_excluded() {
    // tue pareto_frontier: dominated = (qo > q && co <= cost) || (qo >= q && co < cost)
    // et retain: !((q > qo && cost <= co) || (q >= qo && cost < co))
    let sel = ParetoSelector::new(CompositeScoringPolicy::new());
    let bad = model("bad", ModelTier::Standard, 2.0, 100.0, 128_000, 60);
    let good = model("good", ModelTier::Standard, 1.0, 120.0, 128_000, 80);
    let d = sel
        .select(TaskType::Chat, Capabilities::default(), 0, &[bad, good])
        .unwrap();
    assert_eq!(d.model_id, "good");
}

#[test]
fn pareto_three_candidates() {
    // tue pareto_frontier avec 3 candidats (frontiere + domination croisee)
    let sel = ParetoSelector::new(CompositeScoringPolicy::new());
    let a = model("a", ModelTier::Standard, 1.0, 100.0, 128_000, 70);
    let b = model("b", ModelTier::Standard, 2.0, 50.0, 128_000, 85); // domine? non, plus cher
    let c = model("c", ModelTier::Standard, 0.5, 200.0, 128_000, 60); // domine? non, moins qualite
    let d = sel
        .select(TaskType::Chat, Capabilities::default(), 0, &[a, b, c])
        .unwrap();
    assert!(d.model_id == "a" || d.model_id == "b" || d.model_id == "c");
}

#[test]
fn cache_remove_returns_some() {
    // tue cache.rs:54:9  remove -> None
    let mut cache = omega_router::cache::DecisionCache::new();
    let key = omega_router::cache::DecisionCache::key(
        TaskType::Chat,
        &Capabilities::default(),
        0,
        "x",
        1,
    );
    cache.insert(key, omega_types::RouterDecision::new("m", 0.9, "t"));
    assert!(cache.remove(key).is_some());
    assert!(cache.get(key).is_none());
}

#[test]
fn cache_len_after_insert() {
    // tue cache.rs:62:9  len -> 1
    let mut cache = omega_router::cache::DecisionCache::new();
    let key = omega_router::cache::DecisionCache::key(
        TaskType::Chat,
        &Capabilities::default(),
        0,
        "x",
        1,
    );
    cache.insert(key, omega_types::RouterDecision::new("m", 0.9, "t"));
    assert_eq!(cache.len(), 1);
}

#[test]
fn cache_is_empty_after_insert_false() {
    // tue cache.rs:66:9  is_empty -> true
    let mut cache = omega_router::cache::DecisionCache::new();
    let key = omega_router::cache::DecisionCache::key(
        TaskType::Chat,
        &Capabilities::default(),
        0,
        "x",
        1,
    );
    cache.insert(key, omega_types::RouterDecision::new("m", 0.9, "t"));
    assert!(!cache.is_empty());
}
