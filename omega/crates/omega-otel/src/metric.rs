//! Noms de métriques hiérarchiques : `omega.<domaine>.<objet>[.<détail>]`.
//!
//! Exemples : `omega.router.choose`, `omega.tool.call`, `omega.run.step`,
//! `omega.context.assemble`. Le préfixe et le charset sont validés à la construction.

use omega_types::{OmegaError, OmegaResult};

/// Nom de métrique validé.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetricName(String);

impl MetricName {
    /// Valide et construit un nom de métrique.
    ///
    /// Règles : préfixe `omega.`, au moins 3 segments, segments en
    /// `[a-z0-9_]+` (minuscules, chiffres, underscore).
    pub fn parse(name: impl Into<String>) -> OmegaResult<Self> {
        let name = name.into();
        let mut parts = name.split('.');
        match (parts.next(), parts.next(), parts.next()) {
            (Some("omega"), Some(_), Some(_)) => {}
            _ => {
                return Err(OmegaError::Config(format!(
                    "nom de metrique invalide (attendu omega.<domaine>.<objet>) : {name}"
                )))
            }
        }
        if !name.split('.').skip(1).all(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        }) {
            return Err(OmegaError::Config(format!(
                "nom de metrique invalide (segments [a-z0-9_]) : {name}"
            )));
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MetricName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_names() {
        for n in [
            "omega.router.choose",
            "omega.tool.call",
            "omega.run.step",
            "omega.context.assemble",
        ] {
            assert!(MetricName::parse(n).is_ok(), "{n}");
        }
    }

    #[test]
    fn rejects_bad_names() {
        for n in [
            "router.choose",       // pas de prefixe
            "omega.choose",        // trop court
            "omega.Router.choose", // majuscule
            "omega.tool.call!",    // caractere interdit
            "omega..call",         // segment vide
            "",
        ] {
            assert!(MetricName::parse(n).is_err(), "{n}");
        }
    }
}
