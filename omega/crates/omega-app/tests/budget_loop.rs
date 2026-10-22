//! B2 (P1-3) — la boucle d'outils applique le **budget de politique**.
//!
//! Le moteur (`omega-policy`) sait calculer un cumul monotone. La boucle
//! (`omega-core`) ne peut pas le dependre (catégorie `core` → `types` +
//! `ports` seulement, ADR-0004) : elle consomme donc le **port** `RunBudget`.
//! Ce test prouve que la boucle **consulte réellement** ce port — pas que la
//! politique existe à côté.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use omega_core::{EventBus, Operation};
use omega_ports::{BudgetDemand, BudgetRefusal, BudgetUsage, Provider, RunBudget};
use omega_types::{ChatRequest, ChatResponse, Message, Role, Usage};

/// Budget de test : refuse au bout de `max` steps, et **ne débite rien** au
/// refus (monotone, tout-ou-rien).
struct CountingBudget {
    max: u64,
    used: AtomicUsize,
}

impl RunBudget for CountingBudget {
    fn consume(&self, _demand: BudgetDemand) -> Result<BudgetUsage, BudgetRefusal> {
        let n = self.used.fetch_add(1, Ordering::SeqCst) as u64;
        if n >= self.max {
            // Refus : on rend le compteur (rien n'est débité).
            self.used.store(self.max as usize, Ordering::SeqCst);
            return Err(BudgetRefusal {
                axis: Some("steps"),
                reason: "plafond de steps atteint".to_string(),
            });
        }
        Ok(BudgetUsage {
            steps: n + 1,
            ..BudgetUsage::default()
        })
    }

    fn usage(&self) -> BudgetUsage {
        BudgetUsage {
            steps: self.used.load(Ordering::SeqCst) as u64,
            ..BudgetUsage::default()
        }
    }
}

/// Provider qui propose sans fin le même outil.
struct LoopProvider;

#[async_trait]
impl Provider for LoopProvider {
    fn id(&self) -> &str {
        "loop-provider"
    }
    async fn complete(&self, _req: ChatRequest) -> omega_types::OmegaResult<ChatResponse> {
        Ok(ChatResponse {
            message: Message::new(
                Role::Assistant,
                "{\"tool\": \"ls\", \"arguments\": {\"path\": \".\"}}",
            ),
            usage: Usage::default(),
        })
    }
}

#[tokio::test]
async fn la_boucle_arrete_au_budget_de_politique_et_renseigne_la_raison() {
    let root = std::env::temp_dir().join("omega-b2-root");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let surface = Arc::new(omega_tools::surface::FsSurface::new(
        omega_tools::ToolCtx::new(&root),
    ));
    let budget = Arc::new(CountingBudget {
        max: 2,
        used: AtomicUsize::new(0),
    });

    let op = omega_core::tool_loop::ToolLoopOperation {
        provider: Arc::new(LoopProvider),
        model: omega_types::ModelId::new("test"),
        surface,
        messages: vec![Message::new(Role::User, "liste en boucle")],
        // Plafond « technique » large : c'est le budget de POLITIQUE qui doit
        // arreter la boucle, pas celui-ci.
        budget: 1000,
        policy_budget: Some(budget.clone() as Arc<dyn RunBudget>),
        run_id: "b2-run".to_string(),
        trace: None,
        store: None,
    };

    let bus = omega_core::InMemoryBus::default();
    let (token, _handle) = omega_core::cancellation_pair();
    let mut state = omega_core::RunState::default();
    let mut ctx = omega_core::RunContext {
        bus: &bus,
        token: &token,
        state: &mut state,
    };

    op.run(&mut ctx)
        .await
        .expect("la boucle doit s'arreter proprement");

    // 1. Le budget de politique a ete consulte et respecte.
    assert_eq!(
        budget.usage().steps,
        2,
        "la boucle doit s'arreter au plafond de politique"
    );

    // 2. L'arret est TRACE et MOTIVE (jamais un refus muet).
    let events = bus.events();
    let budget_events: Vec<&omega_core::Event> = events
        .iter()
        .filter(|e| e.name == "tool.budget.exhausted")
        .collect();
    assert_eq!(
        budget_events.len(),
        1,
        "un seul evenement de budget ; evenements = {:?}",
        events.iter().map(|e| &e.name).collect::<Vec<_>>()
    );
    let payload = &budget_events[0].payload;
    assert_eq!(payload["source"], "policy");
    assert!(
        payload["reason"].as_str().is_some_and(|r| !r.is_empty()),
        "le refus doit etre motive : {payload:?}"
    );

    // 3. Le client recoit une reponse finale (contrat `infer.completed`).
    let infer = events
        .iter()
        .find(|e| e.name == "infer.completed")
        .expect("la boucle doit publier infer.completed");
    assert_eq!(infer.payload["budget_exhausted"], true);

    let _ = std::fs::remove_dir_all(&root);
}

/// Sans budget de politique, la boucle s'arrete sur son plafond technique.
#[tokio::test]
async fn sans_budget_de_politique_le_plafond_technique_suffit() {
    let root = std::env::temp_dir().join("omega-b2-root-2");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let surface = Arc::new(omega_tools::surface::FsSurface::new(
        omega_tools::ToolCtx::new(&root),
    ));
    let op = omega_core::tool_loop::ToolLoopOperation {
        provider: Arc::new(LoopProvider),
        model: omega_types::ModelId::new("test"),
        surface,
        messages: vec![Message::new(Role::User, "liste en boucle")],
        budget: 3,
        policy_budget: None,
        run_id: "b2-run-2".to_string(),
        trace: None,
        store: None,
    };

    let bus = omega_core::InMemoryBus::default();
    let (token, _handle) = omega_core::cancellation_pair();
    let mut state = omega_core::RunState::default();
    let mut ctx = omega_core::RunContext {
        bus: &bus,
        token: &token,
        state: &mut state,
    };
    op.run(&mut ctx)
        .await
        .expect("la boucle doit s'arreter proprement");

    let calls = bus
        .events()
        .iter()
        .filter(|e| e.name == "tool.call.completed")
        .count();
    assert_eq!(calls, 3, "exactement 3 executions (plafond = 3)");

    let _ = std::fs::remove_dir_all(&root);
}
