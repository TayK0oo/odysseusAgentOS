//! `omega-sandbox` — **sandbox** : politique d'exécution typée + backend local.
//!
//! Catégorie `adapters` du garde-fou : ne dépend que de `types`, `ports`,
//! `registry`, `plugin-api` (jamais `core` ni `app`).

mod approvals;
mod backend;
mod contributor;
mod policy;

pub use approvals::{Approval, ApprovalId, ApprovalQueue, ApprovalState};
pub use backend::LocalSandboxBackend;
pub use contributor::SandboxContributor;
pub use policy::{has_any_substitution, has_unquoted_metachars, ExecRule, RuleBasedExecPolicy};
