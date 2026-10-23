//! `omega-context` — **économie de contexte** (W6).
//!
//! Brique d'adaptateur (categorie `adapters` du garde-fou) : elle depend de
//! `omega-types`/`omega-ports`/`omega-registry`/`omega-plugin-api`, jamais de `omega-core`
//! ni de `omega-app`. Tous les calculs (budget, recherche, compaction) sont
//! **deterministes** — aucun appel LLM (RULES-OMEGA §3.2).
//!
//! Contenu :
//! - [`TokenEstimator`] : estimation `ceil(chars / 4)` ;
//! - [`ContextBudget`] : `estimate → check → enforce`, depassement = `BudgetExceeded` ;
//! - [`DefaultContextEngine`] : moteur de contexte par defaut (port `ContextEngine`) ;
//! - [`LlmCompactionEngine`] : moteur double couche (truncation + resume LLM) ;
//! - [`ToolSurfaceIndex`] : BM25 simplifie + gate par le token le plus rare ;
//! - [`SkillIndex`] : progressive disclosure 3 niveaux + cache par signature ;
//! - [`truncate_compact`] : compaction deterministe sans LLM ;
//! - [`PinnedFacts`] : faits épinglés survivant à la compaction ;
//! - [`ContextContributor`] : cablage moteur + index via le registre.

mod budget;
mod compaction;
mod contributor;
mod engine;
mod injection;
mod pinned_facts;
mod skills;
mod token;
mod tool_surface;

pub use budget::ContextBudget;
pub use compaction::{
    compaction_marker, truncate_compact, truncate_compact_with_pinned, LlmCompactionEngine,
};
pub use contributor::{ContextContributor, DEFAULT_MAX_TOKENS};
pub use engine::{AssembleReport, DefaultContextEngine};
pub use injection::{ContextInjector, InjectionLine, InjectionTrace, MemoryInjection};
pub use pinned_facts::{
    extract_durable_facts, extract_facts, extract_pinned_facts, PinnedFact, PinnedFacts,
};
pub use skills::{Skill, SkillIndex, SkillSummary};
pub use token::{TokenEstimator, CHARS_PER_TOKEN};
pub use tool_surface::{ToolDoc, ToolSurfaceIndex};
