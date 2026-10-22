//! `RunBudget` (port `omega-ports`) — implementation par la politique.
//!
//! La boucle d'outils (`omega-core`) consomme un budget abstrait ; c'est
//! `omega-policy` qui sait calculer le cumul monotone. Ce connecteur est le
//! seul endroit ou les deux se rencontrent.

use std::sync::Arc;

use omega_ports::{BudgetDemand, BudgetRefusal, BudgetUsage, RunBudget};

use crate::budget::{Budget, BudgetAxis, Demand};

/// Budget d'un run, adosse au `Budget` monotone de la politique.
pub struct PolicyRunBudget {
    inner: Arc<Budget>,
}

impl PolicyRunBudget {
    /// Enveloppe un budget de politique.
    pub fn new(inner: Arc<Budget>) -> Self {
        Self { inner }
    }
}

impl RunBudget for PolicyRunBudget {
    fn consume(&self, demand: BudgetDemand) -> Result<BudgetUsage, BudgetRefusal> {
        let d = Demand {
            steps: demand.steps,
            tokens: demand.tokens,
            cost_usd: demand.cost_usd,
            time_ms: demand.time_ms,
        };
        match self.inner.consume_call(d) {
            Ok(usage) => Ok(BudgetUsage {
                steps: usage.steps,
                tokens: usage.tokens,
                cost_usd: usage.cost_usd,
                time_ms: usage.time_ms,
            }),
            Err(exhausted) => Err(BudgetRefusal {
                axis: match exhausted.axis {
                    Some(BudgetAxis::Steps) => Some("steps"),
                    Some(BudgetAxis::Tokens) => Some("tokens"),
                    Some(BudgetAxis::Cost) => Some("cost"),
                    Some(BudgetAxis::Time) => Some("time"),
                    None => None,
                },
                reason: exhausted.to_string(),
            }),
        }
    }

    fn usage(&self) -> BudgetUsage {
        let u = self.inner.usage();
        BudgetUsage {
            steps: u.steps,
            tokens: u.tokens,
            cost_usd: u.cost_usd,
            time_ms: u.time_ms,
        }
    }
}
