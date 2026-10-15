//! Câblage sandbox : politique (`exec-policy`, slot) + backend (`sandbox-backend`, ensemble).

use std::sync::Arc;
use std::time::Duration;

use omega_plugin_api::{Contributor, InstallContext};
use omega_ports::{ExecPolicy, SandboxBackend};
use omega_registry::{Manifest, RegistryBuilder};
use omega_types::{Capability, OmegaResult};

/// Contributeur sandbox : une politique + un backend local borné.
pub struct SandboxContributor {
    policy_id: String,
    policy: Arc<dyn ExecPolicy>,
    backend_id: String,
    timeout: Duration,
}

impl SandboxContributor {
    pub fn new(
        policy_id: impl Into<String>,
        policy: Arc<dyn ExecPolicy>,
        backend_id: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            policy_id: policy_id.into(),
            policy,
            backend_id: backend_id.into(),
            timeout,
        }
    }
}

impl Contributor for SandboxContributor {
    fn name(&self) -> &str {
        &self.backend_id
    }

    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        builder.register_dyn(
            Manifest::new(self.policy_id.clone(), Capability::ExecPolicy, "1"),
            self.policy.clone(),
        )?;
        let backend: Arc<dyn SandboxBackend> = Arc::new(crate::LocalSandboxBackend::new(
            self.policy.clone(),
            self.timeout,
        ));
        builder.register_dyn(
            Manifest::new(self.backend_id.clone(), Capability::SandboxBackend, "1"),
            backend,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuleBasedExecPolicy;
    use omega_types::ExecDecision;

    #[test]
    fn contributor_registers_policy_and_backend() {
        let policy: Arc<dyn ExecPolicy> =
            Arc::new(RuleBasedExecPolicy::new("p").rule("echo", ExecDecision::Allow));
        let c = SandboxContributor::new("policy", policy, "local", Duration::from_secs(5));
        let mut builder = RegistryBuilder::new();
        c.contribute(&mut builder, &InstallContext::new()).unwrap();
        let registry = builder.build();
        assert_eq!(
            registry.active_ids(Capability::ExecPolicy),
            vec!["policy".to_string()]
        );
        assert_eq!(
            registry.active_ids(Capability::SandboxBackend),
            vec!["local".to_string()]
        );
        let p = registry
            .resolve_dyn::<dyn ExecPolicy>(Capability::ExecPolicy)
            .unwrap();
        assert_eq!(p.name(), "p");
    }
}
