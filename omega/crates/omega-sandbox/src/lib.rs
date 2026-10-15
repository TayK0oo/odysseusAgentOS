//! `omega-sandbox` — **sandbox** : politique d'exécution typée + backend local.
//!
//! Catégorie `adapters` du garde-fou : ne dépend que de `types`, `ports`,
//! `registry`, `plugin-api` (jamais `core` ni `app`).

mod backend;
mod contributor;
mod policy;

pub use backend::LocalSandboxBackend;
pub use contributor::SandboxContributor;
pub use policy::{ExecRule, RuleBasedExecPolicy};
