//! Politique de scoring composite multi-criteres.

use omega_providers::ModelRecord;
use omega_types::TaskType;

/// Poids d'un profil de tache. Tous les champs somment a 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weights {
    pub reasoning: f64,
    pub code: f64,
    pub agentic: f64,
    pub knowledge: f64,
    pub chat: f64,
    pub vision: f64,
    pub speed: f64,
    pub cost: f64,
    pub context: f64,
}

impl Weights {
    /// Verifie que les poids somment approximativement a 1 (tolerance flottante).
    pub fn is_normalized(&self) -> bool {
        let sum = self.reasoning
            + self.code
            + self.agentic
            + self.knowledge
            + self.chat
            + self.vision
            + self.speed
            + self.cost
            + self.context;
        (sum - 1.0).abs() < 1e-6
    }
}

/// Port du moteur de scoring.
pub trait ScoringPolicy {
    /// Poids associes a un type de tache.
    fn weights(&self, task: TaskType) -> Weights;

    /// Score composite d'un modele pour une tache donnee.
    /// Les normalisations se font relativement a `candidates`.
    fn score(
        &self,
        task: TaskType,
        required_context: u32,
        candidates: &[ModelRecord],
        model: &ModelRecord,
    ) -> f64;

    /// Qualite effective Q_eff(m) = melange des axes pour la tache.
    fn effective_quality(&self, task: TaskType, model: &ModelRecord) -> f64;
}

/// Scoring composite par defaut : axes + vitesse + cout + contexte.
pub struct CompositeScoringPolicy;

impl Default for CompositeScoringPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeScoringPolicy {
    pub fn new() -> Self {
        Self
    }

    fn weights_for(task: TaskType) -> Weights {
        match task {
            TaskType::TrivialCompletion => Weights {
                reasoning: 0.0,
                code: 0.0,
                agentic: 0.0,
                knowledge: 0.05,
                chat: 0.15,
                vision: 0.0,
                speed: 0.50,
                cost: 0.20,
                context: 0.10,
            },
            TaskType::Summarization => Weights {
                reasoning: 0.0,
                code: 0.0,
                agentic: 0.0,
                knowledge: 0.10,
                chat: 0.10,
                vision: 0.0,
                speed: 0.35,
                cost: 0.30,
                context: 0.15,
            },
            TaskType::TargetedEdit => Weights {
                reasoning: 0.05,
                code: 0.40,
                agentic: 0.0,
                knowledge: 0.05,
                chat: 0.10,
                vision: 0.0,
                speed: 0.15,
                cost: 0.15,
                context: 0.10,
            },
            TaskType::Refactor => Weights {
                reasoning: 0.15,
                code: 0.30,
                agentic: 0.10,
                knowledge: 0.10,
                chat: 0.10,
                vision: 0.0,
                speed: 0.05,
                cost: 0.10,
                context: 0.10,
            },
            TaskType::Implementation => Weights {
                reasoning: 0.20,
                code: 0.30,
                agentic: 0.15,
                knowledge: 0.05,
                chat: 0.10,
                vision: 0.0,
                speed: 0.05,
                cost: 0.10,
                context: 0.05,
            },
            TaskType::LongReasoning => Weights {
                reasoning: 0.50,
                code: 0.10,
                agentic: 0.0,
                knowledge: 0.20,
                chat: 0.0,
                vision: 0.0,
                speed: 0.05,
                cost: 0.05,
                context: 0.10,
            },
            TaskType::Math => Weights {
                reasoning: 0.50,
                code: 0.10,
                agentic: 0.0,
                knowledge: 0.10,
                chat: 0.10,
                vision: 0.0,
                speed: 0.05,
                cost: 0.05,
                context: 0.10,
            },
            TaskType::Research => Weights {
                reasoning: 0.15,
                code: 0.0,
                agentic: 0.10,
                knowledge: 0.35,
                chat: 0.05,
                vision: 0.0,
                speed: 0.05,
                cost: 0.10,
                context: 0.20,
            },
            TaskType::AgenticTerminal => Weights {
                reasoning: 0.05,
                code: 0.15,
                agentic: 0.40,
                knowledge: 0.10,
                chat: 0.05,
                vision: 0.0,
                speed: 0.10,
                cost: 0.10,
                context: 0.05,
            },
            TaskType::Vision => Weights {
                reasoning: 0.0,
                code: 0.0,
                agentic: 0.0,
                knowledge: 0.0,
                chat: 0.10,
                vision: 0.60,
                speed: 0.10,
                cost: 0.15,
                context: 0.05,
            },
            TaskType::Audio => Weights {
                reasoning: 0.0,
                code: 0.0,
                agentic: 0.0,
                knowledge: 0.20,
                chat: 0.20,
                vision: 0.0,
                speed: 0.25,
                cost: 0.35,
                context: 0.0,
            },
            TaskType::Embedding => Weights {
                reasoning: 0.0,
                code: 0.0,
                agentic: 0.0,
                knowledge: 0.0,
                chat: 0.0,
                vision: 0.0,
                speed: 0.15,
                cost: 0.55,
                context: 0.30,
            },
            TaskType::MediaGeneration => Weights {
                reasoning: 0.10,
                code: 0.10,
                agentic: 0.0,
                knowledge: 0.10,
                chat: 0.0,
                vision: 0.40,
                speed: 0.10,
                cost: 0.20,
                context: 0.0,
            },
            TaskType::Chat => Weights {
                reasoning: 0.10,
                code: 0.05,
                agentic: 0.05,
                knowledge: 0.25,
                chat: 0.25,
                vision: 0.0,
                speed: 0.15,
                cost: 0.10,
                context: 0.05,
            },
        }
    }
}

impl ScoringPolicy for CompositeScoringPolicy {
    fn weights(&self, task: TaskType) -> Weights {
        Self::weights_for(task)
    }

    fn effective_quality(&self, task: TaskType, model: &ModelRecord) -> f64 {
        let w = self.weights(task);
        let axes = model.axes.normalized();
        w.reasoning * axes[0]
            + w.code * axes[1]
            + w.agentic * axes[2]
            + w.knowledge * axes[3]
            + w.chat * axes[4]
            + w.vision * axes[5]
    }

    fn score(
        &self,
        task: TaskType,
        required_context: u32,
        candidates: &[ModelRecord],
        model: &ModelRecord,
    ) -> f64 {
        let q_eff = self.effective_quality(task, model);
        let speed_norm = log_normalize(f64::from(model.tokens_per_second), candidates, |m| {
            f64::from(m.tokens_per_second)
        });
        let cost_norm = inverse_log_normalize(model.blended_cost_per_1m(), candidates, |m| {
            m.blended_cost_per_1m()
        });
        let context_norm = if required_context == 0 {
            1.0
        } else {
            (f64::from(model.context_length) / f64::from(required_context)).min(1.0)
        };

        q_eff
            + w_speed(task) * speed_norm
            + w_cost(task) * cost_norm
            + w_context(task) * context_norm
    }
}

fn w_speed(task: TaskType) -> f64 {
    CompositeScoringPolicy::weights_for(task).speed
}

fn w_cost(task: TaskType) -> f64 {
    CompositeScoringPolicy::weights_for(task).cost
}

fn w_context(task: TaskType) -> f64 {
    CompositeScoringPolicy::weights_for(task).context
}

/// Normalise `value` dans [0,1] par log par rapport a l'ensemble des candidats.
fn log_normalize<F>(value: f64, candidates: &[ModelRecord], metric: F) -> f64
where
    F: Fn(&ModelRecord) -> f64,
{
    if candidates.len() < 2 {
        return 1.0;
    }
    let values: Vec<f64> = candidates.iter().map(&metric).collect();
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < 1e-12 {
        return 1.0;
    }
    let log_min = (1.0 + min).ln();
    let log_max = (1.0 + max).ln();
    let log_val = (1.0 + value).ln();
    (log_val - log_min) / (log_max - log_min)
}

/// Inverse log-normalisation : valeur forte → 1, valeur faible → 0.
fn inverse_log_normalize<F>(value: f64, candidates: &[ModelRecord], metric: F) -> f64
where
    F: Fn(&ModelRecord) -> f64,
{
    if candidates.len() < 2 {
        return 1.0;
    }
    let values: Vec<f64> = candidates.iter().map(&metric).collect();
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < 1e-12 {
        return 1.0;
    }
    let log_min = (1.0 + min).ln();
    let log_max = (1.0 + max).ln();
    let log_val = (1.0 + value).ln();
    (log_max - log_val) / (log_max - log_min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_providers::{Capabilities, ModelTier, QualityAxes};

    fn model(
        id: &str,
        cost_in: f64,
        cost_out: f64,
        speed: f32,
        ctx: u32,
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
            capabilities: Capabilities::default(),
            tier: ModelTier::Standard,
        }
    }

    #[test]
    fn weights_sum_to_one() {
        let policy = CompositeScoringPolicy::new();
        for variant in [
            TaskType::TrivialCompletion,
            TaskType::Summarization,
            TaskType::TargetedEdit,
            TaskType::Refactor,
            TaskType::Implementation,
            TaskType::LongReasoning,
            TaskType::Math,
            TaskType::Research,
            TaskType::AgenticTerminal,
            TaskType::Vision,
            TaskType::Audio,
            TaskType::Embedding,
            TaskType::MediaGeneration,
            TaskType::Chat,
        ] {
            assert!(
                policy.weights(variant).is_normalized(),
                "weights for {:?} do not sum to 1",
                variant
            );
        }
    }

    #[test]
    fn trivial_prefers_fast_cheap() {
        let policy = CompositeScoringPolicy::new();
        let cheap = model(
            "cheap",
            0.01,
            0.02,
            1000.0,
            4000,
            QualityAxes {
                chat: 60,
                knowledge: 40,
                ..Default::default()
            },
        );
        let elite = model(
            "elite",
            5.0,
            20.0,
            50.0,
            128_000,
            QualityAxes {
                chat: 80,
                knowledge: 85,
                ..Default::default()
            },
        );
        let candidates = vec![cheap.clone(), elite.clone()];
        let s_cheap = policy.score(TaskType::TrivialCompletion, 0, &candidates, &cheap);
        let s_elite = policy.score(TaskType::TrivialCompletion, 0, &candidates, &elite);
        assert!(s_cheap > s_elite, "cheap should win on T1");
    }

    #[test]
    fn implementation_prefers_elite() {
        let policy = CompositeScoringPolicy::new();
        let cheap = model(
            "cheap",
            0.01,
            0.02,
            1000.0,
            4000,
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
            5.0,
            20.0,
            50.0,
            128_000,
            QualityAxes {
                code: 95,
                reasoning: 95,
                agentic: 90,
                chat: 80,
                ..Default::default()
            },
        );
        let candidates = vec![cheap.clone(), elite.clone()];
        let s_cheap = policy.score(TaskType::Implementation, 0, &candidates, &cheap);
        let s_elite = policy.score(TaskType::Implementation, 0, &candidates, &elite);
        assert!(s_elite > s_cheap, "elite should win on T5");
    }
}
