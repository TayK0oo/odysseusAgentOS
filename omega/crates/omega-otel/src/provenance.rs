//! Provenance des faits : tout fait est tagué `[stated]` / `[observed]` / `[inferred]`
//! avec sa source (RULES-OMEGA §4).

use serde::{Deserialize, Serialize};

/// Origine d'un fait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProvenanceKind {
    /// Dit par l'utilisateur (ou une source externe citée).
    Stated,
    /// Vérifié dans le code / le système / une mesure.
    Observed,
    /// Déduit par raisonnement (à vérifier avant usage critique).
    Inferred,
}

impl ProvenanceKind {
    /// Tag court stable (`[stated]`, `[observed]`, `[inferred]`).
    pub fn tag(self) -> &'static str {
        match self {
            Self::Stated => "[stated]",
            Self::Observed => "[observed]",
            Self::Inferred => "[inferred]",
        }
    }
}

/// Un fait tagué avec sa provenance et sa source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub kind: ProvenanceKind,
    /// Fichier, URL, commande ou run-id. Jamais vide en pratique.
    pub source: String,
}

/// Tague un fait avec son origine et sa source.
pub fn tag(kind: ProvenanceKind, source: impl Into<String>) -> Provenance {
    Provenance {
        kind,
        source: source.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_serialize_to_stable_tags() {
        assert_eq!(
            serde_json::to_value(ProvenanceKind::Stated).unwrap(),
            serde_json::json!("stated")
        );
        assert_eq!(
            serde_json::to_value(ProvenanceKind::Observed).unwrap(),
            serde_json::json!("observed")
        );
        assert_eq!(
            serde_json::to_value(ProvenanceKind::Inferred).unwrap(),
            serde_json::json!("inferred")
        );
        assert_eq!(ProvenanceKind::Stated.tag(), "[stated]");
        assert_eq!(ProvenanceKind::Observed.tag(), "[observed]");
        assert_eq!(ProvenanceKind::Inferred.tag(), "[inferred]");
    }

    #[test]
    fn tag_carries_source() {
        let p = tag(ProvenanceKind::Observed, "cargo test");
        assert_eq!(p.kind, ProvenanceKind::Observed);
        assert_eq!(p.source, "cargo test");
    }
}
