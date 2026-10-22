//! Port `RunBudget` — plafond d'exécution d'un run (P1-3).
//!
//! Le moteur de politique (`omega-policy`) sait calculer un cumul monotone
//! (`Budget::consume_call`). Mais la **boucle d'outils** vit dans
//! `omega-core`, qui ne peut pas dependre d'`omega-policy` (categories :
//! `core` ne depend que de `types` + `ports` — cf. ADR-0004). D'ou ce port :
//! la boucle consomme un budget abstrait, la politique en fournit
//! l'implémentation.
//!
//! Regle de budget (plan §6 P1-3) : une consommation est **tout-ou-rien** et
//! **monotone** — un refus ne debite rien. L'implémentation de reference est
//! `omega_policy::Budget`.

/// Consommation demandee pour un appel (un seul axe suffit ; les autres a 0).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BudgetDemand {
    /// Nombre d'etapes (appels d'outils).
    pub steps: u64,
    /// Tokens consommes.
    pub tokens: u64,
    /// Cout en USD.
    pub cost_usd: f64,
    /// Temps ecoule en millisecondes.
    pub time_ms: u64,
}

/// Cumul consomme, observable (preuve, pas affirmation).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BudgetUsage {
    pub steps: u64,
    pub tokens: u64,
    pub cost_usd: f64,
    pub time_ms: u64,
}

/// Refus motive : la raison est **nommee** (jamais un refus muet).
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetRefusal {
    /// Axe qui a derive (`None` = refus global).
    pub axis: Option<&'static str>,
    /// Raison lisible.
    pub reason: String,
}

/// Budget d'un run : plafonds + consommation monotone.
///
/// Regime : *slot exclusif* par run (un budget par run, pas par appel).
pub trait RunBudget: Send + Sync {
    /// Consomme `demand` de facon atomique : tout-ou-rien.
    ///
    /// `Ok(usage)` = consomme ; `Err(refus)` = refuse **sans rien debiter**.
    fn consume(&self, demand: BudgetDemand) -> Result<BudgetUsage, BudgetRefusal>;

    /// Cumul consomme a l'instant.
    fn usage(&self) -> BudgetUsage;
}
