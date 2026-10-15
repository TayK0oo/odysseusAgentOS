//! Politique de fallback + circuit breaker simple.

use std::collections::HashMap;

use omega_providers::{Capabilities, ModelRecord, ModelTier};

use crate::scoring::ScoringPolicy;
use crate::selector::RouterDecision;
use crate::task::TaskType;

/// Port de la politique de fallback.
pub trait FallbackPolicy {
    /// Enregistre un echec pour un modele.
    fn record_failure(&mut self, model_id: &str);

    /// Indique si le modele est disponible (circuit breaker ferme).
    fn is_available(&self, model_id: &str) -> bool;

    /// Choisit le prochain modele de fallback pour une decision echouee.
    fn next_fallback(
        &self,
        task: TaskType,
        required_capabilities: Capabilities,
        required_context: u32,
        candidates: &[ModelRecord],
        failed: &RouterDecision,
    ) -> Option<RouterDecision>;
}

/// Fallback par tier avec circuit breaker a compteur d'echecs.
pub struct TierFallbackPolicy<S: ScoringPolicy> {
    scoring: S,
    max_failures: u32,
    failures: HashMap<String, u32>,
}

impl<S: ScoringPolicy> TierFallbackPolicy<S> {
    pub fn new(scoring: S, max_failures: u32) -> Self {
        Self {
            scoring,
            max_failures,
            failures: HashMap::new(),
        }
    }

    pub fn failure_count(&self, model_id: &str) -> u32 {
        self.failures.get(model_id).copied().unwrap_or(0)
    }
}

impl<S: ScoringPolicy> FallbackPolicy for TierFallbackPolicy<S> {
    fn record_failure(&mut self, model_id: &str) {
        *self.failures.entry(model_id.to_string()).or_insert(0) += 1;
    }

    fn is_available(&self, model_id: &str) -> bool {
        self.failure_count(model_id) < self.max_failures
    }

    fn next_fallback(
        &self,
        task: TaskType,
        required_capabilities: Capabilities,
        required_context: u32,
        candidates: &[ModelRecord],
        failed: &RouterDecision,
    ) -> Option<RouterDecision> {
        let failed_model = candidates.iter().find(|m| m.id == failed.model_id)?;
        let current_tier_rank = failed_model.tier.rank();

        // Candidats eligibles et disponibles.
        let eligible: Vec<&ModelRecord> = candidates
            .iter()
            .filter(|m| {
                m.id != failed.model_id
                    && m.context_length >= required_context
                    && m.capabilities.covers(required_capabilities)
                    && self.is_available(&m.id)
            })
            .collect();

        if eligible.is_empty() {
            return None;
        }

        // 1. Meme tier (redondance catalogue).
        let same_tier: Vec<&ModelRecord> = eligible
            .iter()
            .copied()
            .filter(|m| m.tier.rank() == current_tier_rank)
            .collect();
        if let Some(next) = pick_best(
            &self.scoring,
            task,
            required_context,
            candidates,
            &same_tier,
        ) {
            return Some(next);
        }

        // 2. Tier superieur.
        let higher_tier: Vec<&ModelRecord> = eligible
            .iter()
            .copied()
            .filter(|m| m.tier.rank() > current_tier_rank)
            .collect();
        if let Some(next) = pick_best(
            &self.scoring,
            task,
            required_context,
            candidates,
            &higher_tier,
        ) {
            return Some(next);
        }

        // 3. Self-hosted.
        let self_hosted: Vec<&ModelRecord> = eligible
            .iter()
            .copied()
            .filter(|m| matches!(m.tier, ModelTier::SelfHosted))
            .collect();
        pick_best(
            &self.scoring,
            task,
            required_context,
            candidates,
            &self_hosted,
        )
    }
}

fn pick_best<S: ScoringPolicy>(
    scoring: &S,
    task: TaskType,
    required_context: u32,
    all: &[ModelRecord],
    subset: &[&ModelRecord],
) -> Option<RouterDecision> {
    subset
        .iter()
        .max_by(|a, b| {
            let sa = scoring.score(task, required_context, all, a);
            let sb = scoring.score(task, required_context, all, b);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| RouterDecision {
            model_id: m.id.clone(),
            score: scoring.score(task, required_context, all, m),
            reason: format!("fallback from failed model to {}", m.id),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::CompositeScoringPolicy;
    use omega_providers::{Capabilities, ModelTier, QualityAxes};

    #[allow(clippy::too_many_arguments)]
    fn model(
        id: &str,
        tier: ModelTier,
        cost_in: f64,
        cost_out: f64,
        speed: f32,
        ctx: u32,
        caps: Capabilities,
        axes: QualityAxes,
    ) -> ModelRecord {
        ModelRecord {
            id: id.to_string(),
            provider: "test".to_string(),
            context_length: ctx,
            cost_input_per_1m: cost_in,
            cost_output_per_1m: cost_out,
            tokens_per_second: speed,
            ttft_ms: 100,
            axes,
            capabilities: caps,
            tier,
        }
    }

    #[test]
    fn circuit_breaker_trips_after_repeated_failures() {
        let mut fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        assert!(fallback.is_available("m1"));
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        assert!(fallback.is_available("m1"));
        fallback.record_failure("m1");
        assert!(!fallback.is_available("m1"));
    }

    #[test]
    fn fallback_picks_different_model() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let m1 = model(
            "m1",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let m2 = model(
            "m2",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 80,
                ..Default::default()
            },
        );
        let failed = RouterDecision {
            model_id: "m1".to_string(),
            score: 0.5,
            reason: "initial".to_string(),
        };
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[m1, m2],
                &failed,
            )
            .unwrap();
        assert_eq!(next.model_id, "m2");
    }

    #[test]
    fn fallback_escalates_tier_when_same_tier_exhausted() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let standard = model(
            "standard",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let elite = model(
            "elite",
            ModelTier::Elite,
            5.0,
            15.0,
            50.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 90,
                ..Default::default()
            },
        );
        let failed = RouterDecision {
            model_id: "standard".to_string(),
            score: 0.5,
            reason: "initial".to_string(),
        };
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[standard, elite],
                &failed,
            )
            .unwrap();
        assert_eq!(next.model_id, "elite");
    }
}
