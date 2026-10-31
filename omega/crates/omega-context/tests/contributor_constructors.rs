//! Contrats comportementaux de `ContextContributor` (module `contributor.rs`).
//!
//! Cible : les mutants survivants du fichier `contributor.rs`
//! (`mutants.out-context-fresh`). Les constructeurs `with_memory`,
//! `with_memory_store` et `with_store` sont **équipotents** à `Default::default()`
//! sur l'API publique : le `MemoryStore` n'est branche que dans le moteur
//! interne (`DefaultContextEngine::with_memory`), qui n'est observable que via
//! `last_injection()`/`injection_journal()` sur le concret — accesseurs non
//! exposés par le registre (`resolve_dyn::<dyn ContextEngine>`). Aucun test
//! comportemental ne les tue. Registre Livrable D.

use std::sync::Arc;

use async_trait::async_trait;
use omega_context::{ContextContributor, PinnedFact, PinnedFacts};
use omega_plugin_api::{compose, Contributor, InstallContext};
use omega_ports::{ContextEngine, Provider};
use omega_types::{Capability, ChatRequest, ChatResponse, Message, OmegaResult, Role, Usage};

struct StubProvider;
#[async_trait]
impl Provider for StubProvider {
    fn id(&self) -> &str {
        "stub"
    }
    async fn complete(&self, _req: ChatRequest) -> OmegaResult<ChatResponse> {
        Ok(ChatResponse {
            message: Message::new(Role::Assistant, "stub"),
            usage: Usage {
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
            },
        })
    }
}

// --- tue :123 ×2 (`name` -> "" et "xyzzy") -------------------------------
#[test]
fn contributor_name_is_omega_context() {
    let c = ContextContributor::new();
    assert_eq!(c.name(), "omega-context");
}

// --- tue :80 (`with_pinned_facts -> Default::default()`) -------------------
#[test]
fn with_pinned_facts_registers_pinned_facts_capability() {
    let pf = PinnedFacts::new();
    pf.upsert(PinnedFact::new("f1", "durable fact", "constraint"));

    // Constructeur : la capacité `PinnedFacts` est enregistrée et contient le fait.
    let contributor = ContextContributor::new().with_pinned_facts(pf);
    let registry = compose(&[&contributor], &InstallContext::new()).unwrap();
    let facts = registry
        .resolve::<PinnedFacts>(Capability::PinnedFacts)
        .expect("la capacité PinnedFacts doit être enregistrée");
    assert_eq!(facts.len(), 1);

    // Mutant : Default::default() -> aucun enregistrement -> resolve échoue.
    let default_contrib = ContextContributor::new();
    let reg2 = compose(&[&default_contrib], &InstallContext::new()).unwrap();
    assert!(
        reg2.resolve::<PinnedFacts>(Capability::PinnedFacts)
            .is_err(),
        "le mutant Default::default() n'enregistrerait pas la capacité"
    );
}

// --- tue :88 (`with_llm_compaction -> Default::default()`) ----------------
#[test]
fn with_llm_compaction_switches_engine_to_llm_compaction() {
    let provider = Arc::new(StubProvider);
    let contributor = ContextContributor::new().with_llm_compaction(provider);
    let registry = compose(&[&contributor], &InstallContext::new()).unwrap();
    let engine = registry
        .resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine)
        .unwrap();
    assert_eq!(engine.name(), "llm-compaction");
    assert!(engine.owns_compaction());

    // Mutant : Default::default() -> moteur par défaut, non LLM.
    let def = ContextContributor::new();
    let reg2 = compose(&[&def], &InstallContext::new()).unwrap();
    let e2 = reg2
        .resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine)
        .unwrap();
    assert_eq!(e2.name(), "default");
    assert!(!e2.owns_compaction());
}
