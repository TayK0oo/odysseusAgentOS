//! Budget de contexte : `estimate -> check -> enforce`, depassement = erreur typee.

use omega_types::{OmegaError, OmegaResult};

use crate::token::TokenEstimator;

/// Budget de tokens d'un appel modele (plafond + consommation).
#[derive(Debug, Clone)]
pub struct ContextBudget {
    max_tokens: usize,
    used: usize,
    estimator: TokenEstimator,
}

impl ContextBudget {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            used: 0,
            estimator: TokenEstimator::new(),
        }
    }

    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    pub fn used(&self) -> usize {
        self.used
    }

    pub fn remaining(&self) -> usize {
        self.max_tokens.saturating_sub(self.used)
    }

    /// Estime (deterministe, chars/4) le cout en tokens d'un texte.
    pub fn estimate(&self, text: &str) -> usize {
        self.estimator.estimate(text)
    }

    /// Verifie que l'ajout de `additional` tokens ne depasse pas le plafond.
    pub fn check(&self, additional: usize) -> OmegaResult<()> {
        if self.used.saturating_add(additional) > self.max_tokens {
            return Err(OmegaError::BudgetExceeded {
                max: self.max_tokens,
                used: self.used,
                additional,
            });
        }
        Ok(())
    }

    /// Consomme `additional` tokens (verifie puis incremente `used`).
    pub fn enforce(&mut self, additional: usize) -> OmegaResult<()> {
        self.check(additional)?;
        self.used += additional;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::OmegaError;

    #[test]
    fn estimate_delegates_chars_per_4() {
        let b = ContextBudget::new(100);
        assert_eq!(b.estimate("12345678"), 2);
    }

    #[test]
    fn check_under_budget_ok() {
        let b = ContextBudget::new(10);
        b.check(10).unwrap();
    }

    #[test]
    fn check_over_budget_is_typed_error() {
        let b = ContextBudget::new(10);
        let err = b.check(11).unwrap_err();
        assert!(matches!(err, OmegaError::BudgetExceeded { max: 10, .. }));
    }

    #[test]
    fn enforce_consumes_tokens() {
        let mut b = ContextBudget::new(10);
        b.enforce(4).unwrap();
        assert_eq!(b.used(), 4);
        assert_eq!(b.remaining(), 6);
    }

    #[test]
    fn enforce_overflow_is_typed_and_does_not_consume() {
        let mut b = ContextBudget::new(10);
        b.enforce(6).unwrap();
        let err = b.enforce(5).unwrap_err();
        assert!(matches!(err, OmegaError::BudgetExceeded { .. }));
        assert_eq!(b.used(), 6);
    }

    #[test]
    fn remaining_reaches_zero_when_full() {
        let mut b = ContextBudget::new(3);
        b.enforce(3).unwrap();
        assert_eq!(b.remaining(), 0);
    }
}
