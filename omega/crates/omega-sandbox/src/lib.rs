//! `omega-sandbox` — **sandbox** : politique d'exécution typée + backend local.

mod approvals;
pub mod bwrap_backend;
mod backend;
mod contributor;
mod namespace_backend;
mod policy;
mod timeout;

pub use approvals::{Approval, ApprovalId, ApprovalQueue, ApprovalState};
pub use backend::LocalSandboxBackend;
pub use contributor::SandboxContributor;
pub use namespace_backend::{NamespaceConfig, NamespaceSandboxBackend};
pub use policy::{has_any_substitution, has_unquoted_metachars, ExecRule, RuleBasedExecPolicy};
