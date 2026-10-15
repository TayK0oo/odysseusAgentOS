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

    /// Sélection + préférence self-hosted (W10) : après `select`, si le gagnant
    /// n'est pas local et qu'un candidat `SelfHosted` éligible existe, applique
    /// la règle `self_hosted_first` (marge `SELF_HOST_MARGIN`). Le local ne gagne
    /// que sur quasi-égalité — jamais sur un écart de qualité réel.
    pub fn select_with_local_preference(
        &self,
        task: TaskType,
        required_capabilities: Capabilities,
        required_context: u32,
        candidates: &[ModelRecord],
    ) -> Result<RouterDecision, RouterError> {
        let decision = self.select(task, required_capabilities, required_context, candidates)?;
        let winner = candidates.iter().find(|m| m.id == decision.model_id);
        let Some(winner) = winner else {
            return Ok(decision);
        };
        if matches!(winner.tier, ModelTier::SelfHosted) {
            return Ok(decision);
        }
        let mut best_local: Option<(&ModelRecord, f64)> = None;
        for m in candidates
            .iter()
            .filter(|m| matches!(m.tier, ModelTier::SelfHosted))
        {
            let s = self.scoring.score(task, required_context, candidates, m);
            if best_local.map(|(_, bs)| s > bs).unwrap_or(true) {
                best_local = Some((m, s));
            }
        }
        let Some((local, local_score)) = best_local else {
            return Ok(decision);
        };
        if omega_providers::ollama_catalog::self_hosted_first(
            local,
            local_score,
            winner,
            decision.score,
            required_capabilities,
            required_context,
        ) {
            Ok(RouterDecision::new(
                local.id.clone(),
                local_score,
                format!(
                    "self-host-first: {} (score {local_score:.4} + marge, cout nul)",
                    local.id
                ),
            ))
        } else {
            Ok(decision)
        }
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

    fn local_candidate() -> ModelRecord {
        model(
            "ollama/qwen2.5-coder:7b",
            ModelTier::SelfHosted,
            0.0,
            0.0,
            40.0,
            32_768,
            Capabilities::default(),
            QualityAxes {
                reasoning: 50,
                code: 65,
                ..Default::default()
            },
        )
    }

    fn remote_candidate() -> ModelRecord {
        model(
            "cloud-elite",
            ModelTier::Elite,
            5.0,
            15.0,
            60.0,
            200_000,
            Capabilities::default(),
            QualityAxes {
                reasoning: 95,
                code: 95,
                ..Default::default()
            },
        )
    }

    /// Scoring bouché à scores fixes : rend la préférence locale déterministe,
    /// indépendamment des poids du composite.
    struct FixedScoring {
        scores: std::collections::HashMap<String, f64>,
    }

    impl omega_ports::ScoringPolicy for FixedScoring {
        fn weights(&self, _task: TaskType) -> omega_types::Weights {
            omega_types::Weights {
                reasoning: 0.2,
                code: 0.2,
                agentic: 0.1,
                knowledge: 0.1,
                chat: 0.1,
                vision: 0.0,
                speed: 0.1,
                cost: 0.1,
                context: 0.1,
            }
        }

        fn score(
            &self,
            _task: TaskType,
            _required_context: u32,
            _candidates: &[ModelRecord],
            model: &ModelRecord,
        ) -> f64 {
            *self.scores.get(&model.id).unwrap_or(&0.0)
        }

        fn effective_quality(&self, _task: TaskType, model: &ModelRecord) -> f64 {
            *self.scores.get(&model.id).unwrap_or(&0.0)
        }
    }

    fn fixed_selector(local_score: f64, remote_score: f64) -> ParetoSelector<FixedScoring> {
        let mut scores = std::collections::HashMap::new();
        scores.insert("ollama/qwen2.5-coder:7b".to_string(), local_score);
        scores.insert("cloud-elite".to_string(), remote_score);
        ParetoSelector::new(FixedScoring { scores })
    }

    #[test]
    fn local_preference_wins_at_close_score() {
        // (W10) scores bouchés 0.70 vs 0.74 (écart < marge 0.05) : le brut
        // choisit le cloud, la préférence le renverse vers le local.
        let selector = fixed_selector(0.70, 0.74);
        let candidates = vec![local_candidate(), remote_candidate()];
        let raw = selector
            .select(TaskType::Chat, Capabilities::default(), 0, &candidates)
            .unwrap();
        assert_eq!(raw.model_id, "cloud-elite");
        let d = selector
            .select_with_local_preference(TaskType::Chat, Capabilities::default(), 0, &candidates)
            .unwrap();
        assert_eq!(d.model_id, "ollama/qwen2.5-coder:7b");
        assert!(d.reason.contains("self-host-first"));
    }

    #[test]
    fn local_preference_keeps_remote_on_large_gap() {
        // (W10) écart de qualité réel → le distant reste (la marge ne renverse pas).
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let local = model(
            "ollama/tiny",
            ModelTier::SelfHosted,
            0.0,
            0.0,
            40.0,
            32_768,
            Capabilities::default(),
            QualityAxes {
                reasoning: 5,
                code: 5,
                ..Default::default()
            },
        );
        let d = selector
            .select_with_local_preference(
                TaskType::LongReasoning,
                Capabilities::default(),
                4000,
                &[local, remote_candidate()],
            )
            .unwrap();
        assert_eq!(d.model_id, "cloud-elite");
    }

    #[test]
    fn local_preference_noop_on_single_or_local_winner() {
        // (W10) un seul candidat ou gagnant déjà local → décision inchangée.
        let selector = ParetoSelector::new(CompositeScoringPolicy::new());
        let d = selector
            .select_with_local_preference(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[local_candidate()],
            )
            .unwrap();
        assert_eq!(d.model_id, "ollama/qwen2.5-coder:7b");
    }
}
