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
    /// Règles : préfixe `omega.`, 3 à 8 segments, segments en
    /// `[a-z0-9_-]+` (minuscules, chiffres, underscore, tiret — le tiret est
    /// courant dans les conventions OpenMetrics/OTel).
    pub fn parse(name: impl Into<String>) -> OmegaResult<Self> {
        let name = name.into();
        let segments: Vec<&str> = name.split('.').collect();
        if segments.len() < 3 || segments.len() > 8 || segments[0] != "omega" {
            return Err(OmegaError::Config(format!(
                "nom de metrique invalide (attendu omega.<domaine>.<objet>[...], 3-8 segments) : {name}"
            )));
        }
        if !segments[1..].iter().all(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        }) {
            return Err(OmegaError::Config(format!(
                "nom de metrique invalide (segments [a-z0-9_-]) : {name}"
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
            "router.choose",           // pas de prefixe
            "omega.choose",            // trop court
            "omega.Router.choose",     // majuscule
            "omega.tool.call!",        // caractere interdit
            "omega..call",             // segment vide
            "omega.a.b.c.d.e.f.g.h.i", // trop profond (> 8 segments)
            "",
        ] {
            assert!(MetricName::parse(n).is_err(), "{n}");
        }
    }

    #[test]
    fn accepts_hyphenated_names() {
        assert!(MetricName::parse("omega.http.request-count").is_ok());
    }
}
