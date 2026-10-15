//! Politique de fallback + circuit breaker simple.

use std::collections::HashSet;

use omega_ports::FallbackPolicy;
use omega_types::{Capabilities, ModelRecord, ModelTier, RouterDecision, TaskType};

/// Fallback par tier avec circuit breaker a compteur d'echecs.
///
/// Le circuit breaker (`failures`) persiste sur l'instance. L'ensemble des
/// modeles deja essayes dans un run (`tried`) est porte par l'appelant et
/// reinitialise a chaque requete : il ne fuit pas d'une requete a l'autre
/// (REVUE-W4 N2).
pub struct TierFallbackPolicy<S: omega_ports::ScoringPolicy> {
    scoring: S,
    max_failures: u32,
    failures: std::collections::HashMap<String, u32>,
}

impl<S: omega_ports::ScoringPolicy> TierFallbackPolicy<S> {
    pub fn new(scoring: S, max_failures: u32) -> Self {
        Self {
            scoring,
            max_failures,
            failures: std::collections::HashMap::new(),
        }
    }

    pub fn failure_count(&self, model_id: &str) -> u32 {
        self.failures.get(model_id).copied().unwrap_or(0)
    }
}

impl<S: omega_ports::ScoringPolicy> FallbackPolicy for TierFallbackPolicy<S> {
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
        tried: &mut HashSet<String>,
    ) -> Option<RouterDecision> {
        let failed_model = candidates.iter().find(|m| m.id == failed.model_id)?;
        let current_tier_rank = failed_model.tier.rank();

        // Memorise le modele qui vient d'echouer comme "deja essaye".
        tried.insert(failed.model_id.clone());

        // Candidats eligibles, disponibles et non deja essayes.
        let eligible: Vec<&ModelRecord> = candidates
            .iter()
            .filter(|m| {
                m.id != failed.model_id
                    && !tried.contains(&m.id)
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
            tried.insert(next.model_id.clone());
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
            tried.insert(next.model_id.clone());
            return Some(next);
        }

        // 3. Self-hosted.
        let self_hosted: Vec<&ModelRecord> = eligible
            .iter()
            .copied()
            .filter(|m| matches!(m.tier, ModelTier::SelfHosted))
            .collect();
        let next = pick_best(
            &self.scoring,
            task,
            required_context,
            candidates,
            &self_hosted,
        )?;
        tried.insert(next.model_id.clone());
        Some(next)
    }
}

fn pick_best<S: omega_ports::ScoringPolicy>(
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
    use omega_types::{Capabilities, ModelTier, QualityAxes};

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
        let failed = RouterDecision::new("m1", 0.5, "initial");
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[m1, m2],
                &failed,
                &mut HashSet::new(),
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
        let failed = RouterDecision::new("standard", 0.5, "initial");
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[standard, elite],
                &failed,
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(next.model_id, "elite");
    }

    #[test]
    fn fallback_avoids_ping_pong() {
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

        // m1 echoue -> fallback m2.
        let mut tried = HashSet::new();
        let step0 = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[m1.clone(), m2.clone()],
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut tried,
            )
            .unwrap();
        assert_eq!(step0.model_id, "m2");

        // m2 echoue -> on ne doit PAS reselectionner m1 (deja essaye).
        let step1 = fallback.next_fallback(
            TaskType::Chat,
            Capabilities::default(),
            0,
            &[m1, m2],
            &RouterDecision::new("m2", 0.5, "fallback"),
            &mut tried,
        );
        assert!(step1.is_none(), "should exhaust candidates, not ping-pong");
    }

    #[test]
    fn fallback_escalates_to_self_hosted_after_exhaustion() {
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
        let self_hosted = model(
            "self-hosted",
            ModelTier::SelfHosted,
            0.0,
            0.0,
            80.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 65,
                ..Default::default()
            },
        );

        let failed = RouterDecision::new("standard", 0.5, "initial");
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[standard, self_hosted],
                &failed,
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(next.model_id, "self-hosted");
    }

    #[test]
    fn fallback_returns_none_when_failed_model_unknown() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let m1 = model(
            "m1",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes::default(),
        );
        let next = fallback.next_fallback(
            TaskType::Chat,
            Capabilities::default(),
            0,
            &[m1],
            &RouterDecision::new("unknown", 0.5, "initial"),
            &mut HashSet::new(),
        );
        assert!(next.is_none());
    }

    #[test]
    fn tried_state_is_per_run_not_shared_between_requests() {
        // REVUE-W4 N2 : l'ensemble "deja essaye" ne doit pas fuir d'une requete
        // a l'autre. Meme instance, deux runs avec un `tried` frais => meme
        // fallback complet depuis zero.
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
        let candidates = [m1, m2];

        // Run 1 : m1 echoue -> m2.
        let run1 = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &candidates,
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(run1.model_id, "m2");

        // Run 2 : `tried` reinitialise => le fallback repart de zero (m2 a nouveau).
        let run2 = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &candidates,
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(run2.model_id, "m2");
    }
}
