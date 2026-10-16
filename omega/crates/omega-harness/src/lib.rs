//! `omega-harness` — **catalogue de reverse-engineering** des grands harnais.
//!
//! Chaque harnais (Claude Code, Codex, Cursor, Cline, Aider, Goose, OpenHands,
//! Hermes, OpenClaw, OpenCode, Windsurf, Zed, Plandex, gptme, Open Interpreter)
//! est décrit par ses **principes** transférables, avec leur statut réel dans
//! Omega (`adopted` / `planned` / `rejected`) et leur cible (`omega::…`).
//!
//! Le corpus est **embarqué** (`include_str!`) : il est versionné avec le code et
//! immuable à l'exécution. C'est notre mémoire de veille, exposée en données.
//!
//! Catégorie `adapters` du garde-fou : dépend de `types`, `ports`, `registry`,
//! `plugin-api`.

use serde::{Deserialize, Serialize};

/// Statut d'un principe vis-à-vis d'Omega.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipleStatus {
    /// Repris et implémenté.
    Adopted,
    /// Repris, pas encore implémenté.
    Planned,
    /// Volontairement écarté (le motif est dans `why`).
    Rejected,
}

/// Coût de portage en Rust (S/M/L).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RustCost {
    S,
    M,
    L,
}

/// Un principe transférable d'un harnais.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Principle {
    pub id: String,
    pub name: String,
    /// Ce que ça fait.
    pub what: String,
    /// Pourquoi ça compte pour nous.
    pub why: String,
    pub status: PrincipleStatus,
    /// Cible dans Omega (crate/type), ou `null`.
    #[serde(default)]
    pub omega: Option<String>,
    pub rust_cost: RustCost,
    #[serde(default)]
    pub refs: Vec<String>,
}

/// Un harnais observé.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Harness {
    pub id: String,
    pub name: String,
    pub editor: String,
    pub oss: bool,
    pub language: String,
    /// Note d'architecture (résumé du RE).
    pub architecture: String,
    pub principles: Vec<Principle>,
}

/// Le corpus complet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    pub schema_version: u32,
    pub note: String,
    pub harnesses: Vec<Harness>,
}

/// Erreur de chargement du corpus (ne devrait jamais arriver : embarqué et testé).
pub const CATALOG_JSON: &str = include_str!("../data/harnesses.json");

impl Catalog {
    /// Corpus embarqué, désérialisé.
    ///
    /// L'échec est impossible en pratique : un test garantit que le JSON embarqué
    /// est valide (échec à la compilation des tests, pas à l'exécution).
    pub fn embedded() -> Self {
        serde_json::from_str(CATALOG_JSON).expect("catalogue de harnais invalide (embarque)")
    }

    /// Tous les harnais.
    pub fn harnesses(&self) -> &[Harness] {
        &self.harnesses
    }

    /// Un harnais par identifiant.
    pub fn by_id(&self, id: &str) -> Option<&Harness> {
        self.harnesses.iter().find(|h| h.id == id)
    }

    /// Tous les principes (tous harnais confondus).
    pub fn principles(&self) -> impl Iterator<Item = (&Harness, &Principle)> {
        self.harnesses
            .iter()
            .flat_map(|h| h.principles.iter().map(move |p| (h, p)))
    }

    /// Comptage par statut : (adopted, planned, rejected).
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut a = 0;
        let mut p = 0;
        let mut r = 0;
        for (_, pr) in self.principles() {
            match pr.status {
                PrincipleStatus::Adopted => a += 1,
                PrincipleStatus::Planned => p += 1,
                PrincipleStatus::Rejected => r += 1,
            }
        }
        (a, p, r)
    }
}

/// Port de lecture du catalogue (capacité `harness-catalog`).
pub trait HarnessCatalog: Send + Sync {
    fn catalog(&self) -> Catalog;
}

/// Implémentation par défaut : le corpus embarqué.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmbeddedCatalog;

impl HarnessCatalog for EmbeddedCatalog {
    fn catalog(&self) -> Catalog {
        Catalog::embedded()
    }
}

/// Contributeur de registre : expose le catalogue comme capacité `harness-catalog`.
pub struct HarnessCatalogContributor {
    id: String,
    catalog: EmbeddedCatalog,
}

impl Default for HarnessCatalogContributor {
    fn default() -> Self {
        Self::new()
    }
}

impl HarnessCatalogContributor {
    pub fn new() -> Self {
        Self {
            id: "embedded".to_string(),
            catalog: EmbeddedCatalog,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

impl omega_plugin_api::Contributor for HarnessCatalogContributor {
    fn name(&self) -> &str {
        &self.id
    }

    fn contribute(
        &self,
        builder: &mut omega_registry::RegistryBuilder,
        _ctx: &omega_plugin_api::InstallContext,
    ) -> omega_types::OmegaResult<()> {
        builder.register_dyn(
            omega_registry::Manifest::new(
                self.id.clone(),
                omega_types::Capability::HarnessCatalog,
                "1",
            ),
            std::sync::Arc::new(self.catalog),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn embedded_catalog_parses_and_is_non_empty() {
        let c = Catalog::embedded();
        assert_eq!(c.schema_version, 1);
        assert!(c.harnesses.len() >= 12, "au moins 12 harnais");
        assert!(c.principles().count() >= 40, "au moins 40 principes");
    }

    #[test]
    fn ids_are_unique_and_well_formed() {
        let c = Catalog::embedded();
        let mut hids = HashSet::new();
        let mut pids = HashSet::new();
        for h in c.harnesses() {
            assert!(hids.insert(h.id.clone()), "harnais duplique : {}", h.id);
            assert!(!h.name.is_empty());
            assert!(!h.architecture.is_empty());
            for p in &h.principles {
                assert!(pids.insert(p.id.clone()), "principe duplique : {}", p.id);
                assert!(!p.what.is_empty(), "{} sans `what`", p.id);
                assert!(!p.why.is_empty(), "{} sans `why`", p.id);
                assert!(!p.refs.is_empty(), "{} sans source", p.id);
            }
        }
    }

    #[test]
    fn required_harnesses_are_present() {
        // Les harnais explicitement demandés par la vision.
        let c = Catalog::embedded();
        for id in [
            "cursor",
            "claude-code",
            "codex",
            "hermes",
            "openclaw",
            "aider",
            "goose",
            "opencode",
            "cline",
            "zed",
        ] {
            assert!(c.by_id(id).is_some(), "harnais manquant : {id}");
        }
    }

    #[test]
    fn adopted_principles_target_omega_code() {
        // Un principe « adopted » doit pointer une cible réelle dans Omega.
        let c = Catalog::embedded();
        for (h, p) in c.principles() {
            if p.status == PrincipleStatus::Adopted {
                assert!(
                    p.omega.as_ref().map(|o| !o.is_empty()).unwrap_or(false),
                    "{} / {} : adopted sans cible omega",
                    h.id,
                    p.id
                );
            }
        }
    }

    #[test]
    fn counts_are_consistent() {
        let c = Catalog::embedded();
        let (a, p, r) = c.counts();
        assert_eq!(a + p + r, c.principles().count());
        assert!(
            a >= 10,
            "au moins 10 principes adoptes (sinon le RE ne sert a rien)"
        );
    }

    #[test]
    fn by_id_returns_none_for_unknown() {
        assert!(Catalog::embedded().by_id("n-existe-pas").is_none());
    }

    #[test]
    fn contributor_registers_catalog_as_capability() {
        use omega_plugin_api::{compose, Contributor, InstallContext};
        let c = HarnessCatalogContributor::new();
        let registry = compose(&[&c as &dyn Contributor], &InstallContext::new()).unwrap();
        assert_eq!(
            registry.active_ids(omega_types::Capability::HarnessCatalog),
            vec!["embedded".to_string()]
        );
        let cat = registry
            .resolve_dyn::<EmbeddedCatalog>(omega_types::Capability::HarnessCatalog)
            .unwrap();
        assert!(cat.catalog().harnesses.len() >= 12);
    }
}
