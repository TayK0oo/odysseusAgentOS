//! `omega-otel` — **traçabilité** : provenance, télémétrie nommée, sinks, replay.
//!
//! Catégorie `adapters` du garde-fou : ne dépend que de `types`, `ports`, `config`,
//! `registry`, `plugin-api` (jamais `core` ni `app`).

mod contributor;
mod metric;
mod provenance;
mod replay;
mod sinks;

pub use contributor::OtelContributor;
pub use metric::MetricName;
pub use provenance::{tag, Provenance, ProvenanceKind};
pub use replay::{
    ReplayedOutcome, ReplayedRun, ReplayedStep, ReplayedStepStatus, RunSummary, TraceReplayer,
};
pub use sinks::{FileSink, StdoutSink};
