//! Rejeu de journaux : reconstruit la séquence d'un run **sans ré-exécuter** les effets.
//!
//! Entrée : un journal `Vec<DurableEvent>` (lu via `EventStore::read_run`).
//! Sortie : la séquence ordonnée des étapes + l'issue du run + un résumé déterministe.

use omega_types::DurableEvent;
use serde::{Deserialize, Serialize};

/// Statut rejoué d'une étape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplayedStepStatus {
    Completed,
    Failed,
}

/// Une étape rejouée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayedStep {
    pub operation_id: String,
    pub status: ReplayedStepStatus,
    pub seq: u64,
}

/// Issue rejouée d'un run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplayedOutcome {
    Completed,
    Failed,
    /// Aucun marqueur terminal (`run.completed`/`run.failed`) dans le journal.
    Unknown,
}

/// Séquence rejouée d'un run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayedRun {
    pub run_id: String,
    pub steps: Vec<ReplayedStep>,
    pub outcome: ReplayedOutcome,
    /// Étapes `step.*` sans `operation_id` : ignorées du séquençage mais COMPTÉES
    /// (jamais de perte silencieuse — REVIEW-W8-W9 B3).
    pub ignored_events: usize,
    /// Vrai si le journal contient à la fois `run.completed` et `run.failed`.
    /// L'issue retenue est alors celle du **premier** marqueur (déterministe).
    pub outcome_conflict: bool,
}

/// Résumé déterministe d'un journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: String,
    pub total_events: usize,
    pub steps_completed: usize,
    pub steps_failed: usize,
    pub outcome: ReplayedOutcome,
    pub ignored_events: usize,
    pub outcome_conflict: bool,
}

/// Rejoue un journal : trie par `seq`, extrait les étapes, détecte l'issue.
///
/// Aucun effet n'est ré-exécuté : pure fonction du journal.
pub struct TraceReplayer;

impl TraceReplayer {
    pub fn replay(run_id: &str, mut journal: Vec<DurableEvent>) -> ReplayedRun {
        journal.sort_by_key(|e| e.seq);
        let mut steps = Vec::new();
        let mut ignored_events = 0usize;
        let mut outcome = ReplayedOutcome::Unknown;
        let mut seen_terminal: Option<ReplayedOutcome> = None;
        let mut outcome_conflict = false;
        for e in &journal {
            match e.name.as_str() {
                "step.completed" | "step.failed" => {
                    let Some(op) = e.operation_id.clone() else {
                        ignored_events += 1;
                        continue;
                    };
                    steps.push(ReplayedStep {
                        operation_id: op,
                        status: if e.name == "step.completed" {
                            ReplayedStepStatus::Completed
                        } else {
                            ReplayedStepStatus::Failed
                        },
                        seq: e.seq,
                    });
                }
                "run.completed" | "run.failed" => {
                    let marker = if e.name == "run.completed" {
                        ReplayedOutcome::Completed
                    } else {
                        ReplayedOutcome::Failed
                    };
                    match seen_terminal {
                        None => {
                            seen_terminal = Some(marker);
                            outcome = marker;
                        }
                        Some(first) if first != marker => {
                            // Double marqueur contradictoire : le PREMIER gagne
                            // (déterministe), le conflit est signalé.
                            outcome_conflict = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        ReplayedRun {
            run_id: run_id.to_string(),
            steps,
            outcome,
            ignored_events,
            outcome_conflict,
        }
    }

    /// Résumé déterministe : même journal → même résumé.
    pub fn summarize(run_id: &str, journal: Vec<DurableEvent>) -> RunSummary {
        let total_events = journal.len();
        let replayed = Self::replay(run_id, journal);
        RunSummary {
            run_id: replayed.run_id,
            total_events,
            steps_completed: replayed
                .steps
                .iter()
                .filter(|s| s.status == ReplayedStepStatus::Completed)
                .count(),
            steps_failed: replayed
                .steps
                .iter()
                .filter(|s| s.status == ReplayedStepStatus::Failed)
                .count(),
            outcome: replayed.outcome,
            ignored_events: replayed.ignored_events,
            outcome_conflict: replayed.outcome_conflict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ev(seq: u64, name: &str, op: Option<&str>) -> DurableEvent {
        DurableEvent::new("run-1", name, json!(null))
            .with_seq(seq)
            .with_operation_id(op.map(str::to_string))
    }

    fn journal() -> Vec<DurableEvent> {
        vec![
            ev(0, "run.started", None),
            ev(1, "step.completed", Some("assemble")),
            ev(2, "step.completed", Some("infer")),
            ev(3, "run.completed", None),
        ]
    }

    #[test]
    fn replay_rebuilds_sequence_without_side_effects() {
        let r = TraceReplayer::replay("run-1", journal());
        assert_eq!(r.run_id, "run-1");
        assert_eq!(r.steps.len(), 2);
        assert_eq!(r.steps[0].operation_id, "assemble");
        assert_eq!(r.steps[0].status, ReplayedStepStatus::Completed);
        assert_eq!(r.outcome, ReplayedOutcome::Completed);
    }

    #[test]
    fn replay_detects_failed_run() {
        let mut j = journal();
        j.pop();
        j.push(ev(3, "step.failed", Some("infer")));
        j.push(ev(4, "run.failed", None));
        let r = TraceReplayer::replay("run-1", j);
        assert_eq!(r.outcome, ReplayedOutcome::Failed);
        assert_eq!(r.steps.len(), 3);
        assert_eq!(r.steps[2].status, ReplayedStepStatus::Failed);
        assert_eq!(r.steps[2].operation_id, "infer");
    }

    #[test]
    fn replay_is_order_stable_on_unordered_input() {
        let mut j = journal();
        j.reverse();
        let r = TraceReplayer::replay("run-1", j);
        assert_eq!(r.steps[0].operation_id, "assemble");
        assert_eq!(r.steps[1].operation_id, "infer");
    }

    #[test]
    fn summary_is_deterministic() {
        let a = TraceReplayer::summarize("run-1", journal());
        let b = TraceReplayer::summarize("run-1", journal());
        assert_eq!(a, b);
        assert_eq!(a.total_events, 4);
        assert_eq!(a.steps_completed, 2);
        assert_eq!(a.steps_failed, 0);
        assert_eq!(a.outcome, ReplayedOutcome::Completed);
        assert_eq!(a.ignored_events, 0);
        assert!(!a.outcome_conflict);
    }

    #[test]
    fn steps_without_operation_id_are_counted_not_lost() {
        // (B3) une étape sans `operation_id` est comptée, pas perdue en silence.
        let mut j = journal();
        j.push(ev(4, "step.completed", None));
        let r = TraceReplayer::replay("run-1", j);
        assert_eq!(r.steps.len(), 2);
        assert_eq!(r.ignored_events, 1);
        let full = TraceReplayer::summarize("run-1", {
            let mut jj = journal();
            jj.push(ev(4, "step.completed", None));
            jj
        });
        assert_eq!(full.total_events, 5);
        assert_eq!(full.ignored_events, 1);
    }

    #[test]
    fn conflicting_terminal_markers_first_wins_and_conflict_flagged() {
        // (B3) double marqueur contradictoire : le premier gagne, conflit signalé.
        let mut j = journal();
        j.push(ev(4, "run.failed", None));
        let r = TraceReplayer::replay("run-1", j);
        assert_eq!(r.outcome, ReplayedOutcome::Completed);
        assert!(r.outcome_conflict);
    }
}
