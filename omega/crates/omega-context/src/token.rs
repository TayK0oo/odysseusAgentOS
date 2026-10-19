//! Estimateur de tokens **deterministe** : `ceil(chars / 4)`, sans appel LLM.
//!
//! Regle economique (RULES-OMEGA §3.2) : l'estimation est un calcul de systeme,
//! jamais un tour de modele.

use omega_types::Message;

/// Nombre de caracteres estimes par token (heuristique `CHARS_PER_TOKEN = 4.0`,
/// cf. `RE-OPENCLAW-HERMES.md` §3.2.2).
pub const CHARS_PER_TOKEN: u32 = 4;

/// Estimateur de tokens deterministe et sans dependance reseau.
///
/// `Default` est implemente **manuellement** (pas `#[derive(Default)]`) pour renvoyer
/// l'estimateur canonique `ceil(chars / 4)`. La derive aurait produit
/// `chars_per_token = 0`, rendant `estimate()` impossible (division par zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenEstimator {
    chars_per_token: u32,
}

impl Default for TokenEstimator {
    /// Estimateur canonique `ceil(chars / 4)` — jamais un ratio nul.
    fn default() -> Self {
        Self::new()
    }
}

impl TokenEstimator {
    pub fn new() -> Self {
        Self {
            chars_per_token: CHARS_PER_TOKEN,
        }
    }

    /// Estimation en tokens d'un texte (`ceil(nb_chars / 4)`).
    pub fn estimate(&self, text: &str) -> usize {
        let chars = text.chars().count();
        if chars == 0 {
            0
        } else {
            chars.div_ceil(self.chars_per_token as usize)
        }
    }

    /// Estimation cumulee des contenus d'une liste de messages.
    pub fn estimate_messages(&self, messages: &[Message]) -> usize {
        messages.iter().map(|m| self.estimate(&m.content)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::Role;

    #[test]
    fn empty_text_is_zero() {
        let est = TokenEstimator::new();
        assert_eq!(est.estimate(""), 0);
    }

    #[test]
    fn estimate_is_deterministic() {
        let est = TokenEstimator::new();
        let text = "Le cout de calcul va au systeme, pas a l'agent.";
        assert_eq!(est.estimate(text), est.estimate(text));
    }

    #[test]
    fn ceil_boundaries() {
        let est = TokenEstimator::new();
        assert_eq!(est.estimate("1234"), 1);
        assert_eq!(est.estimate("12345"), 2);
        assert_eq!(est.estimate("12345678"), 2);
        assert_eq!(est.estimate("123456789"), 3);
    }

    #[test]
    fn estimate_messages_sums_contents() {
        let est = TokenEstimator::new();
        let msgs = vec![
            Message::new(Role::System, "1234"),
            Message::new(Role::User, "12345"),
        ];
        assert_eq!(est.estimate_messages(&msgs), 3);
    }

    #[test]
    fn default_ne_panique_pas() {
        // Non-regression REVIEW-W6 B1 : `Default` derive posait `chars_per_token = 0`
        // et `estimate("x")` paniquait (division par zero). `default()` doit renvoyer
        // l'estimateur canonique `ceil(chars / 4)`.
        let est = TokenEstimator::default();
        assert_eq!(est, TokenEstimator::new());
        assert_eq!(est.estimate("x"), 1);
        assert_eq!(est.estimate("1234"), 1);
        assert_eq!(est.estimate("12345"), 2);
    }

    #[test]
    fn estimate_counts_chars_not_bytes() {
        // Entree multi-octets / CJK / emoji : le decompte porte sur les caracteres
        // Unicode, pas sur les octets (determinisme et pas de sur-estimation).
        let est = TokenEstimator::new();
        assert_eq!(est.estimate("你好"), 1); // 2 caracteres -> ceil(2/4) = 1
        assert_eq!(est.estimate("你好世界"), 1); // 4 caracteres -> 1
        assert_eq!(est.estimate("🙂🙂🙂🙂🙂"), 2); // 5 emojis -> ceil(5/4) = 2
        assert_eq!(est.estimate("é"), 1); // 1 caractere (2 octets) -> 1
    }

    #[test]
    fn estimate_long_input_does_not_overflow() {
        let est = TokenEstimator::new();
        let text = "a".repeat(100_000);
        assert_eq!(est.estimate(&text), 25_000);
    }
}
