//! Câblage des sinks de télémétrie dans le registre (capacité `telemetry-sink`).

use std::sync::Arc;

use omega_plugin_api::{Contributor, InstallContext};
use omega_registry::{Manifest, RegistryBuilder};
use omega_types::{Capability, OmegaResult};

/// Contributeur qui enregistre des sinks (`Arc<dyn TelemetrySink>`, voie `dyn`).
pub struct OtelContributor {
    id_prefix: String,
    sinks: Vec<(String, Arc<dyn omega_ports::TelemetrySink>)>,
}

impl OtelContributor {
    pub fn new(id_prefix: impl Into<String>) -> Self {
        Self {
            id_prefix: id_prefix.into(),
            sinks: Vec::new(),
        }
    }

    pub fn with_sink(
        mut self,
        id: impl Into<String>,
        sink: Arc<dyn omega_ports::TelemetrySink>,
    ) -> Self {
        self.sinks.push((id.into(), sink));
        self
    }
}

impl Contributor for OtelContributor {
    fn name(&self) -> &str {
        &self.id_prefix
    }

    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        for (id, sink) in &self.sinks {
            builder.register_dyn(
                Manifest::new(
                    format!("{}-{id}", self.id_prefix),
                    Capability::TelemetrySink,
                    "1",
                ),
                sink.clone(),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sinks::StdoutSink;

    #[test]
    fn contributor_registers_sinks_resolvable_as_ports() {
        let mut builder = RegistryBuilder::new();
        let c = OtelContributor::new("otel").with_sink("stdout", Arc::new(StdoutSink::new()));
        c.contribute(&mut builder, &InstallContext::new()).unwrap();
        let registry = builder.build();
        assert_eq!(
            registry.active_ids(Capability::TelemetrySink),
            vec!["otel-stdout".to_string()]
        );
        let sink = registry
            .resolve_dyn::<dyn omega_ports::TelemetrySink>(Capability::TelemetrySink)
            .unwrap();
        assert_eq!(sink.name(), "stdout");
    }
}
