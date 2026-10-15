//! Démonstration live : run Omega de bout en bout sur modèle local (Ollama).
//!
//! Chaîne complète : classification déterministe → sélection Pareto → assemblage
//! (ContextEngine du registre) → inférence (Provider Ollama du registre) → journal
//! durable → résumé rejoué. Coût : zéro.
//!
//! Prérequis : `ollama serve` + un modèle tiré (ex. `ollama pull qwen2.5-coder:7b`).
//! Lancement : `cargo run -p omega-app --example live_ollama`.

use std::sync::Arc;

use omega_app::{AppBuilder, FileEventStore, InMemoryEventStore, RunInput};
use omega_context::ContextContributor;
use omega_core::Operation;
use omega_otel::TraceReplayer;
use omega_plugin_api::{Contributor, InstallContext};
use omega_ports::{EventStore, Provider};
use omega_registry::{Manifest, RegistryBuilder};
use omega_router::{CompositeScoringPolicy, HeuristicTaskClassifier, Selector, TaskClassifier};
use omega_router::selector::ParetoSelector;
use omega_types::{
    Capabilities, Capability, Message, ModelId, ModelRecord, ModelTier, OmegaResult, QualityAxes,
    Role, RoutingRequest,
};

struct ProviderContributor {
    provider: Arc<dyn Provider>,
}

impl Contributor for ProviderContributor {
    fn name(&self) -> &str {
        "ollama-provider"
    }

    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        builder.register_dyn(
            Manifest::new("ollama", Capability::LlmProvider, "1"),
            self.provider.clone(),
        )
    }
}

struct StoreContributor {
    store: Arc<dyn EventStore>,
}

impl Contributor for StoreContributor {
    fn name(&self) -> &str {
        "store"
    }

    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        builder.register_dyn(
            Manifest::new("file", Capability::EventStore, "1"),
            self.store.clone(),
        )
    }
}

fn local_record() -> ModelRecord {
    ModelRecord {
        id: "ollama/qwen2.5-coder:7b".to_string(),
        provider: "ollama".to_string(),
        context_length: 32768,
        cost_input_per_1m: 0.0,
        cost_output_per_1m: 0.0,
        tokens_per_second: 40.0,
        ttft_ms: 500,
        axes: QualityAxes {
            reasoning: 55,
            code: 70,
            agentic: 40,
            knowledge: 50,
            chat: 60,
            vision: 0,
        },
        capabilities: Capabilities::default(),
        tier: ModelTier::SelfHosted,
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("LIVE-ECHEC : {e:?}");
        std::process::exit(1);
    }
}

async fn run() -> OmegaResult<()> {
    let store = Arc::new(FileEventStore::new(
        std::env::temp_dir().join("omega-live"),
    ));
    let provider: Arc<dyn Provider> = Arc::new(omega_providers::OllamaProvider::new(
        "http://localhost:11434",
        "qwen2.5-coder:7b",
    ));

    let app = AppBuilder::new()
        .add_contributor(Box::new(ProviderContributor {
            provider: provider.clone(),
        }))
        .add_contributor(Box::new(ContextContributor::new().with_max_tokens(8000)))
        .add_contributor(Box::new(StoreContributor {
            store: store.clone(),
        }))
        .build()?;

    // 1. Classification déterministe (zéro LLM).
    let text = "Résume ce paragraphe en trois points : Omega est un harnais d'agent \
                où chaque brique est un plugin interchangeable, avec un moteur en Rust.";
    let req = RoutingRequest::new(text);
    let task = HeuristicTaskClassifier::new().classify(&req);
    println!("tache : {} ({})", task.code(), format!("{task:?}"));

    // 2. Sélection (Pareto + score composite).
    let selector = ParetoSelector::new(CompositeScoringPolicy::new());
    let decision = selector
        .select(
            task,
            Capabilities::default(),
            4000,
            &[local_record()],
        )
        .map_err(|e| omega_types::OmegaError::Config(format!("selection : {e:?}")))?;
    println!(
        "decision : {} (score {:.3}) — {}",
        decision.model_id, decision.score, decision.reason
    );

    // 3. Run : assemble (ContextEngine du registre) + inférence (Provider du registre).
    let messages = vec![Message::new(Role::User, text)];
    let ops: Vec<Box<dyn Operation>> = vec![
        app.assemble_operation(messages.clone())?,
        app.infer_operation(ModelId::new(&decision.model_id), messages)?,
    ];
    let outcome = app.run(RunInput::new(ops).with_run_id("live-1")).await?;
    println!(
        "run {} : {:?} (executes={}, sautes={})",
        outcome.run_id, outcome.status, outcome.executed_operations, outcome.skipped_operations
    );

    // 4. Preuve par le journal : `step.completed(infer)` = l'inférence live a abouti.
    let journal = store.read_run("live-1")?;
    let summary = TraceReplayer::summarize("live-1", journal);
    println!(
        "journal : {} evenements, {}/{} etapes OK, issue={:?}",
        summary.total_events, summary.steps_completed,
        summary.steps_completed + summary.steps_failed, summary.outcome
    );
    if summary.steps_completed < 2 {
        return Err(omega_types::OmegaError::Config(
            "l'inference live n'a pas abouti (voir journal)".to_string(),
        ));
    }

    // 5. Réponse du modèle (relue depuis l'état via une seconde inférence légère ? non :
    // on affiche le résumé — la preuve suffit, le contenu vit dans le RunState).
    let _ = (provider, InMemoryEventStore::new());
    println!("LIVE-OK : run local de bout en bout, cout nul.");
    Ok(())
}
