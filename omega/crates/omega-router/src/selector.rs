//! Selection de modele : filtres durs, frontiere de Pareto, argmax du score.

use omega_ports::Selector;
use omega_types::{
    format_reason, Capabilities, ModelRecord, ModelTier, RouterDecision, RouterError, TaskType,
};

/// Selecteur avec filtre dur + Pareto + scoring composite.
pub struct ParetoSelector<S: omega_ports::ScoringPolicy> {
    scoring: S,
}

impl<S: omega_ports::ScoringPolicy> ParetoSelector<S> {
    pub fn new(scoring: S) -> Self {
        Self { scoring }
    }
}

impl<S: omega_ports::ScoringPolicy> Selector for ParetoSelector<S> {
    fn select(
        &self,
        task: TaskType,
        required_capabilities: Capabilities,
        required_context: u32,
        candidates: &[ModelRecord],
    ) -> Result<RouterDecision, RouterError> {
        if candidates.is_empty() {
            return Err(RouterError::EmptyRegistry);
        }

        // 1. Filtres durs.
        let eligible: Vec<&ModelRecord> = candidates
            .iter()
            .filter(|m| {
                m.context_length >= required_context && m.capabilities.covers(required_capabilities)
            })
            .collect();

        if eligible.is_empty() {
            return Err(RouterError::NoEligibleModel);
        }

        // 2. Frontiere de Pareto sur (Q_eff, cout inverse).
        let pareto = pareto_frontier(&self.scoring, task, &eligible);
        let pareto_len = pareto.len();

        // 3. Score composite et argmax avec tie-break.
        let best = pareto
            .into_iter()
            .map(|m| {
                let score = self.scoring.score(task, required_context, candidates, m);
                (m, score)
            })
            .max_by(|(a, sa), (b, sb)| {
                sa.partial_cmp(sb)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        // Tie-break : cout croissant.
                        a.blended_cost_per_1m()
                            .partial_cmp(&b.blended_cost_per_1m())
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .reverse()
                    })
                    .then_with(|| {
                        // Tie-break : vitesse decroissante.
                        a.tokens_per_second
                            .partial_cmp(&b.tokens_per_second)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| {
                        // Tie-break : preference self-hosted.
                        matches!(a.tier, ModelTier::SelfHosted)
                            .cmp(&matches!(b.tier, ModelTier::SelfHosted))
                    })
            })
            .expect("pareto non vide");

        let reason = format_reason(task.code(), eligible.len(), pareto_len, &best.0.id, best.1);

        Ok(RouterDecision {
            model_id: best.0.id.clone(),
            score: best.1,
            reason,
        })
    }
}

/// Construit la frontiere de Pareto sur (qualite effective, cout) :
/// garde les modeles non domines (pas d'autre modele avec Q >= et cout <=).
fn pareto_frontier<'a, S: omega_ports::ScoringPolicy>(
    scoring: &S,
    task: TaskType,
    candidates: &[&'a ModelRecord],
) -> Vec<&'a ModelRecord> {
    let mut frontier: Vec<&'a ModelRecord> = Vec::new();
    for &candidate in candidates {
        let q = scoring.effective_quality(task, candidate);
        let cost = candidate.blended_cost_per_1m();

        let dominated = frontier.iter().any(|&other| {
            let qo = scoring.effective_quality(task, other);
            let co = other.blended_cost_per_1m();
            (qo > q && co <= cost) || (qo >= q && co < cost)
        });

        if !dominated {
            frontier.retain(|&other| {
                let qo = scoring.effective_quality(task, other);
                let co = other.blended_cost_per_1m();
                !((q > qo && cost <= co) || (q >= qo && cost < co))
            });
            frontier.push(candidate);
        }
    }
    frontier
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
    fn t1_routes_to_fast_cheap() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let cheap = model(
            "cheap",
            ModelTier::FastCheap,
            0.01,
            0.02,
            1000.0,
            4000,
            Capabilities::default(),
            QualityAxes {
                chat: 60,
                knowledge: 40,
                ..Default::default()
            },
        );
        let elite = model(
            "elite",
            ModelTier::Elite,
            5.0,
            20.0,
            50.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 80,
                knowledge: 85,
                ..Default::default()
            },
        );
        let decision = selector
            .select(
                TaskType::TrivialCompletion,
                Capabilities::default(),
                0,
                &[cheap, elite],
            )
            .unwrap();
        assert_eq!(decision.model_id, "cheap");
    }

    #[test]
    fn t5_routes_to_elite() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let cheap = model(
            "cheap",
            ModelTier::FastCheap,
            0.01,
            0.02,
            1000.0,
            4000,
            Capabilities::default(),
            QualityAxes {
                code: 30,
                reasoning: 20,
                agentic: 20,
                chat: 60,
                ..Default::default()
            },
        );
        let elite = model(
            "elite",
            ModelTier::Elite,
            5.0,
            20.0,
            50.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                code: 95,
                reasoning: 95,
                agentic: 90,
                chat: 80,
                ..Default::default()
            },
        );
        let decision = selector
            .select(
                TaskType::Implementation,
                Capabilities::default(),
                0,
                &[cheap, elite],
            )
            .unwrap();
        assert_eq!(decision.model_id, "elite");
    }

    #[test]
    fn hard_filter_excludes_insufficient_context() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let small_ctx = model(
            "small",
            ModelTier::Standard,
            0.5,
            1.5,
            200.0,
            4000,
            Capabilities::default(),
            QualityAxes {
                reasoning: 80,
                ..Default::default()
            },
        );
        let big_ctx = model(
            "big",
            ModelTier::Standard,
            0.5,
            1.5,
            200.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                reasoning: 80,
                ..Default::default()
            },
        );
        let decision = selector
            .select(
                TaskType::LongReasoning,
                Capabilities::default(),
                8000,
                &[small_ctx, big_ctx],
            )
            .unwrap();
        assert_eq!(decision.model_id, "big");
    }

    #[test]
    fn hard_filter_excludes_missing_function_calling() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let no_tools = model(
            "no-tools",
            ModelTier::Standard,
            0.5,
            1.5,
            200.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                agentic: 90,
                ..Default::default()
            },
        );
        let with_tools = model(
            "with-tools",
            ModelTier::Standard,
            0.5,
            1.5,
            200.0,
            128_000,
            Capabilities {
                function_calling: true,
                ..Default::default()
            },
            QualityAxes {
                agentic: 90,
                ..Default::default()
            },
        );
        let decision = selector
            .select(
                TaskType::AgenticTerminal,
                Capabilities {
                    function_calling: true,
                    ..Default::default()
                },
                0,
                &[no_tools, with_tools],
            )
            .unwrap();
        assert_eq!(decision.model_id, "with-tools");
    }

    #[test]
    fn empty_registry_fails() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        assert_eq!(
            selector
                .select(TaskType::Chat, Capabilities::default(), 0, &[])
                .unwrap_err(),
            RouterError::EmptyRegistry
        );
    }

    #[test]
    fn no_eligible_model_fails() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let small_ctx = model(
            "small",
            ModelTier::Standard,
            0.5,
            1.5,
            200.0,
            4000,
            Capabilities::default(),
            QualityAxes {
                reasoning: 80,
                ..Default::default()
            },
        );
        assert_eq!(
            selector
                .select(
                    TaskType::LongReasoning,
                    Capabilities::default(),
                    8000,
                    &[small_ctx],
                )
                .unwrap_err(),
            RouterError::NoEligibleModel
        );
    }

    #[test]
    fn dominated_model_is_excluded_from_pareto_frontier() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        // `bad` est domine par `good` (meilleure qualite ET meilleur cout).
        let bad = model(
            "bad",
            ModelTier::Standard,
            2.0,
            6.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 60,
                ..Default::default()
            },
        );
        let good = model(
            "good",
            ModelTier::Standard,
            1.0,
            3.0,
            120.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 80,
                ..Default::default()
            },
        );
        let decision = selector
            .select(TaskType::Chat, Capabilities::default(), 0, &[bad, good])
            .unwrap();
        assert_eq!(decision.model_id, "good");
    }

    #[test]
    fn tie_break_prefers_cheaper_faster_self_hosted() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let cheap_fast = model(
            "cheap-fast",
            ModelTier::Standard,
            0.5,
            1.5,
            200.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let expensive_slow = model(
            "expensive-slow",
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
        let decision = selector
            .select(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[expensive_slow, cheap_fast],
            )
            .unwrap();
        assert_eq!(decision.model_id, "cheap-fast");
    }

    #[test]
    fn determinism_on_permuted_input() {
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let a = model(
            "a",
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
        let b = model(
            "b",
            ModelTier::Standard,
            2.0,
            6.0,
            50.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 85,
                ..Default::default()
            },
        );
        let d1 = selector
            .select(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[a.clone(), b.clone()],
            )
            .unwrap();
        let d2 = selector
            .select(TaskType::Chat, Capabilities::default(), 0, &[b, a])
            .unwrap();
        assert_eq!(d1.model_id, d2.model_id);
    }
}
