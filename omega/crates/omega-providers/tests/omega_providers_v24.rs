//! Tests v24 — `omega-providers`.

use omega_providers::{
    local_record, parse_tags_response, self_hosted_first, Capabilities, ModelRecord, ModelTier,
    QualityAxes,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn self_hosted_first_context_boundary() {
    // tue ollama_catalog.rs:166:82  <  ->  <=  (local context < required)
    let local = local_record("q", Some(4096));
    let remote = ModelRecord {
        id: "r".into(),
        provider: "p".into(),
        context_length: 128_000,
        cost_input_per_1m: 1.0,
        cost_output_per_1m: 3.0,
        tokens_per_second: 100.0,
        ttft_ms: 100,
        axes: QualityAxes::default(),
        capabilities: Capabilities::default(),
        tier: ModelTier::Standard,
    };
    // context == required → local should still win (eligible)
    assert!(self_hosted_first(
        &local,
        0.9,
        &remote,
        0.5,
        Capabilities::default(),
        4096
    ));
    // context < required → local loses
    assert!(!self_hosted_first(
        &local,
        0.9,
        &remote,
        0.5,
        Capabilities::default(),
        8192
    ));
}

#[test]
fn self_hosted_first_remote_context_boundary() {
    // tue ollama_catalog.rs:171:34  <  ->  <=  (remote context < required)
    let local = local_record("q", Some(32_768));
    let remote_small = ModelRecord {
        id: "r".into(),
        provider: "p".into(),
        context_length: 1024,
        cost_input_per_1m: 1.0,
        cost_output_per_1m: 3.0,
        tokens_per_second: 100.0,
        ttft_ms: 100,
        axes: QualityAxes::default(),
        capabilities: Capabilities::default(),
        tier: ModelTier::Standard,
    };
    // remote context < required → local wins regardless of score
    assert!(self_hosted_first(
        &local,
        0.1,
        &remote_small,
        0.99,
        Capabilities::default(),
        4000
    ));
}

#[test]
fn parse_tags_skips_blank() {
    let raw = json!({"models": [{"name": "qwen2.5-coder:7b", "model": "qwen2.5-coder:7b", "size": 100u64}, {"name": "  ", "model": "vide"}]});
    let raws = parse_tags_response(raw, &HashMap::new());
    assert_eq!(raws.len(), 1);
    assert_eq!(raws[0].id, "ollama/qwen2.5-coder:7b");
}

#[test]
fn local_record_defaults_context() {
    let rec = local_record("tiny:1b", None);
    assert_eq!(rec.context_length, 8192);
    assert_eq!(rec.cost_input_per_1m, 0.0);
    assert!(matches!(rec.tier, ModelTier::SelfHosted));
}

#[test]
fn self_hosted_first_requires_self_hosted_tier() {
    let remote1 = ModelRecord {
        id: "a".into(),
        provider: "p".into(),
        context_length: 128_000,
        cost_input_per_1m: 1.0,
        cost_output_per_1m: 3.0,
        tokens_per_second: 100.0,
        ttft_ms: 100,
        axes: QualityAxes::default(),
        capabilities: Capabilities::default(),
        tier: ModelTier::Standard,
    };
    let remote2 = remote1.clone();
    // non-SelfHosted local → false
    assert!(!self_hosted_first(
        &remote1,
        0.99,
        &remote2,
        0.1,
        Capabilities::default(),
        0
    ));
}
